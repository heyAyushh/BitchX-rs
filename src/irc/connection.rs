use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

use crate::error::{BitchYError, Result};

pub enum IrcStream {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

pub struct IrcConnection {
    reader: IrcReader,
    writer: IrcWriter,
}

enum IrcReader {
    Plain(BufReader<ReadHalf<TcpStream>>),
    Tls(BufReader<ReadHalf<TlsStream<TcpStream>>>),
}

enum IrcWriter {
    Plain(WriteHalf<TcpStream>),
    Tls(WriteHalf<TlsStream<TcpStream>>),
}

impl IrcConnection {
    pub async fn connect(host: &str, port: u16, use_tls: bool, verify_certs: bool) -> Result<Self> {
        let tcp = TcpStream::connect((host, port)).await?;

        if use_tls {
            let _ = rustls::crypto::ring::default_provider().install_default();

            let config = if verify_certs {
                rustls::ClientConfig::builder()
                    .with_root_certificates(root_store())
                    .with_no_client_auth()
            } else {
                rustls::ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerifier))
                    .with_no_client_auth()
            };

            let connector = TlsConnector::from(Arc::new(config));
            let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
                .map_err(|e| BitchYError::Tls(format!("Invalid server name: {e}")))?;

            let tls_stream = connector
                .connect(server_name, tcp)
                .await
                .map_err(|e| BitchYError::Tls(format!("TLS handshake failed: {e}")))?;

            let (read_half, write_half) = tokio::io::split(tls_stream);
            Ok(Self {
                reader: IrcReader::Tls(BufReader::new(read_half)),
                writer: IrcWriter::Tls(write_half),
            })
        } else {
            let (read_half, write_half) = tokio::io::split(tcp);
            Ok(Self {
                reader: IrcReader::Plain(BufReader::new(read_half)),
                writer: IrcWriter::Plain(write_half),
            })
        }
    }

    pub async fn send(&mut self, line: &str) -> Result<()> {
        let data = if line.ends_with("\r\n") {
            line.to_string()
        } else {
            format!("{line}\r\n")
        };
        match &mut self.writer {
            IrcWriter::Plain(w) => w.write_all(data.as_bytes()).await?,
            IrcWriter::Tls(w) => w.write_all(data.as_bytes()).await?,
        }
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<String>> {
        let mut line = String::new();
        let n = match &mut self.reader {
            IrcReader::Plain(r) => r.read_line(&mut line).await?,
            IrcReader::Tls(r) => r.read_line(&mut line).await?,
        };
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
        Ok(Some(trimmed))
    }

    pub fn split(self) -> (ConnectionReader, ConnectionWriter) {
        (
            ConnectionReader {
                reader: self.reader,
            },
            ConnectionWriter {
                writer: self.writer,
            },
        )
    }
}

pub struct ConnectionReader {
    reader: IrcReader,
}

pub struct ConnectionWriter {
    writer: IrcWriter,
}

impl ConnectionReader {
    pub async fn recv(&mut self) -> Result<Option<String>> {
        let mut line = String::new();
        let n = match &mut self.reader {
            IrcReader::Plain(r) => r.read_line(&mut line).await?,
            IrcReader::Tls(r) => r.read_line(&mut line).await?,
        };
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
        Ok(Some(trimmed))
    }
}

impl ConnectionWriter {
    pub async fn send(&mut self, line: &str) -> Result<()> {
        let data = if line.ends_with("\r\n") {
            line.to_string()
        } else {
            format!("{line}\r\n")
        };
        match &mut self.writer {
            IrcWriter::Plain(w) => w.write_all(data.as_bytes()).await?,
            IrcWriter::Tls(w) => w.write_all(data.as_bytes()).await?,
        }
        Ok(())
    }
}

fn root_store() -> rustls::RootCertStore {
    let mut store = rustls::RootCertStore::empty();
    store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    store
}

#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn connect_plain_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 256];
            let n = sock.read(&mut buf).await.unwrap();
            let received = String::from_utf8_lossy(&buf[..n]).to_string();
            sock.write_all(b":server PONG :token\r\n").await.unwrap();
            received
        });

        let mut conn = IrcConnection::connect(&addr.ip().to_string(), addr.port(), false, false)
            .await
            .unwrap();

        conn.send("PING :token").await.unwrap();
        let response = conn.recv().await.unwrap();

        let sent = server.await.unwrap();
        assert_eq!(sent, "PING :token\r\n");
        assert_eq!(response, Some(":server PONG :token".to_string()));
    }

    #[tokio::test]
    async fn recv_returns_none_on_eof() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            drop(sock);
        });

        let mut conn = IrcConnection::connect(&addr.ip().to_string(), addr.port(), false, false)
            .await
            .unwrap();

        server.await.unwrap();
        let result = conn.recv().await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn split_send_recv() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 256];
            let n = sock.read(&mut buf).await.unwrap();
            let received = String::from_utf8_lossy(&buf[..n]).to_string();
            sock.write_all(b":server 001 nick :Welcome\r\n")
                .await
                .unwrap();
            received
        });

        let conn = IrcConnection::connect(&addr.ip().to_string(), addr.port(), false, false)
            .await
            .unwrap();

        let (mut reader, mut writer) = conn.split();

        writer.send("NICK :testnick").await.unwrap();
        let msg = reader.recv().await.unwrap();

        let sent = server.await.unwrap();
        assert_eq!(sent, "NICK :testnick\r\n");
        assert_eq!(msg, Some(":server 001 nick :Welcome".to_string()));
    }

    #[tokio::test]
    async fn send_already_terminated() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 256];
            let n = sock.read(&mut buf).await.unwrap();
            String::from_utf8_lossy(&buf[..n]).to_string()
        });

        let mut conn = IrcConnection::connect(&addr.ip().to_string(), addr.port(), false, false)
            .await
            .unwrap();

        conn.send("QUIT :bye\r\n").await.unwrap();
        drop(conn);

        let sent = server.await.unwrap();
        assert_eq!(sent, "QUIT :bye\r\n");
    }

    #[tokio::test]
    async fn recv_multiple_lines() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            sock.write_all(b"PING :one\r\nPING :two\r\nPING :three\r\n")
                .await
                .unwrap();
            drop(sock);
        });

        let mut conn = IrcConnection::connect(&addr.ip().to_string(), addr.port(), false, false)
            .await
            .unwrap();

        server.await.unwrap();

        let l1 = conn.recv().await.unwrap();
        let l2 = conn.recv().await.unwrap();
        let l3 = conn.recv().await.unwrap();
        let l4 = conn.recv().await.unwrap();

        assert_eq!(l1, Some("PING :one".to_string()));
        assert_eq!(l2, Some("PING :two".to_string()));
        assert_eq!(l3, Some("PING :three".to_string()));
        assert_eq!(l4, None);
    }

    #[tokio::test]
    async fn connect_to_invalid_address_fails() {
        let result = IrcConnection::connect("127.0.0.1", 1, false, false).await;
        assert!(result.is_err());
    }

    #[test]
    fn root_store_has_certs() {
        let store = root_store();
        assert!(!store.is_empty());
    }
}
