use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::error::{BitchXError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum DccType {
    Chat,
    Send,
    Get,
}

#[derive(Debug, Clone)]
pub struct DccOffer {
    pub kind: DccType,
    pub from: String,
    pub filename: Option<String>,
    pub address: IpAddr,
    pub port: u16,
    pub size: Option<u64>,
}

#[derive(Debug)]
pub struct DccTransfer {
    pub kind: DccType,
    pub peer: String,
    pub filename: Option<PathBuf>,
    pub bytes_transferred: u64,
    pub total_bytes: Option<u64>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

fn ip_from_long(long: u64) -> IpAddr {
    IpAddr::V4(Ipv4Addr::from((long as u32).to_be_bytes()))
}

fn ip_to_long(addr: IpAddr) -> u64 {
    match addr {
        IpAddr::V4(v4) => u32::from_be_bytes(v4.octets()) as u64,
        IpAddr::V6(_) => 0,
    }
}

impl DccOffer {
    pub fn parse(from: &str, params: &str) -> Option<Self> {
        let parts: Vec<&str> = params.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let dcc_type = parts[0].to_ascii_uppercase();
        match dcc_type.as_str() {
            "SEND" => {
                if parts.len() < 4 {
                    return None;
                }
                let filename = parts[1].to_string();
                let ip: u64 = parts[2].parse().ok()?;
                let port: u16 = parts[3].parse().ok()?;
                let size: Option<u64> = parts.get(4).and_then(|s| s.parse().ok());

                Some(DccOffer {
                    kind: DccType::Send,
                    from: from.to_string(),
                    filename: Some(filename),
                    address: ip_from_long(ip),
                    port,
                    size,
                })
            }
            "CHAT" => {
                let ip: u64 = parts[2].parse().ok()?;
                let port: u16 = parts[3].parse().ok()?;

                Some(DccOffer {
                    kind: DccType::Chat,
                    from: from.to_string(),
                    filename: None,
                    address: ip_from_long(ip),
                    port,
                    size: None,
                })
            }
            _ => None,
        }
    }

    pub fn send_request(filename: &str, addr: IpAddr, port: u16, size: u64) -> String {
        format!(
            "\x01DCC SEND {} {} {} {}\x01",
            filename,
            ip_to_long(addr),
            port,
            size
        )
    }

    pub fn chat_request(addr: IpAddr, port: u16) -> String {
        format!("\x01DCC CHAT chat {} {}\x01", ip_to_long(addr), port)
    }
}

pub async fn receive_file(
    offer: &DccOffer,
    save_path: &Path,
    progress_tx: tokio::sync::mpsc::UnboundedSender<(u64, Option<u64>)>,
) -> Result<u64> {
    let addr = SocketAddr::new(offer.address, offer.port);
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|e| BitchXError::Dcc(format!("Failed to connect: {e}")))?;

    let mut file = tokio::fs::File::create(save_path)
        .await
        .map_err(|e| BitchXError::Dcc(format!("Failed to create file: {e}")))?;

    let mut total: u64 = 0;
    let mut buf = vec![0u8; 8192];

    loop {
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| BitchXError::Dcc(format!("Read error: {e}")))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .await
            .map_err(|e| BitchXError::Dcc(format!("Write error: {e}")))?;
        total += n as u64;

        // Send DCC acknowledgement (4-byte big-endian total bytes)
        let ack = (total as u32).to_be_bytes();
        let _ = stream.write_all(&ack).await;

        let _ = progress_tx.send((total, offer.size));
    }

    Ok(total)
}

pub async fn send_file(
    path: &Path,
    listener: TcpListener,
    block_size: usize,
    progress_tx: tokio::sync::mpsc::UnboundedSender<(u64, Option<u64>)>,
) -> Result<u64> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|e| BitchXError::Dcc(format!("Failed to stat file: {e}")))?;
    let file_size = metadata.len();

    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| BitchXError::Dcc(format!("Accept error: {e}")))?;

    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| BitchXError::Dcc(format!("Failed to open file: {e}")))?;

    let mut total: u64 = 0;
    let mut buf = vec![0u8; block_size];

    loop {
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| BitchXError::Dcc(format!("Read error: {e}")))?;
        if n == 0 {
            break;
        }
        stream
            .write_all(&buf[..n])
            .await
            .map_err(|e| BitchXError::Dcc(format!("Write error: {e}")))?;
        total += n as u64;

        let _ = progress_tx.send((total, Some(file_size)));
    }

    stream
        .shutdown()
        .await
        .map_err(|e| BitchXError::Dcc(format!("Shutdown error: {e}")))?;

    Ok(total)
}

pub async fn accept_chat(offer: &DccOffer) -> Result<TcpStream> {
    let addr = SocketAddr::new(offer.address, offer.port);
    TcpStream::connect(addr)
        .await
        .map_err(|e| BitchXError::Dcc(format!("Failed to connect for DCC chat: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dcc_send() {
        let offer = DccOffer::parse("Alice", "SEND test.txt 3232235777 5000 1024").unwrap();
        assert_eq!(offer.kind, DccType::Send);
        assert_eq!(offer.from, "Alice");
        assert_eq!(offer.filename.as_deref(), Some("test.txt"));
        assert_eq!(offer.address, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(offer.port, 5000);
        assert_eq!(offer.size, Some(1024));
    }

    #[test]
    fn test_parse_dcc_send_no_size() {
        let offer = DccOffer::parse("Alice", "SEND test.txt 3232235777 5000").unwrap();
        assert_eq!(offer.kind, DccType::Send);
        assert!(offer.size.is_none());
    }

    #[test]
    fn test_parse_dcc_chat() {
        let offer = DccOffer::parse("Bob", "CHAT chat 2130706433 6000").unwrap();
        assert_eq!(offer.kind, DccType::Chat);
        assert_eq!(offer.from, "Bob");
        assert!(offer.filename.is_none());
        assert_eq!(offer.address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(offer.port, 6000);
    }

    #[test]
    fn test_parse_dcc_invalid() {
        assert!(DccOffer::parse("X", "SEND").is_none());
        assert!(DccOffer::parse("X", "").is_none());
        assert!(DccOffer::parse("X", "BLAH a b c").is_none());
    }

    #[test]
    fn test_parse_dcc_bad_ip() {
        assert!(DccOffer::parse("X", "SEND file notanumber 5000").is_none());
    }

    #[test]
    fn test_parse_dcc_bad_port() {
        assert!(DccOffer::parse("X", "SEND file 3232235777 notaport").is_none());
    }

    #[test]
    fn test_ip_from_long_localhost() {
        let ip = ip_from_long(2130706433);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn test_ip_to_long_localhost() {
        let long = ip_to_long(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(long, 2130706433);
    }

    #[test]
    fn test_ip_roundtrip() {
        let addr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let long = ip_to_long(addr);
        let back = ip_from_long(long);
        assert_eq!(addr, back);
    }

    #[test]
    fn test_send_request() {
        let req = DccOffer::send_request(
            "file.txt",
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            5000,
            1024,
        );
        assert_eq!(req, "\x01DCC SEND file.txt 3232235777 5000 1024\x01");
    }

    #[test]
    fn test_chat_request() {
        let req = DccOffer::chat_request(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6000);
        assert_eq!(req, "\x01DCC CHAT chat 2130706433 6000\x01");
    }

    #[test]
    fn test_parse_case_insensitive() {
        let offer = DccOffer::parse("X", "send file.txt 2130706433 5000 512").unwrap();
        assert_eq!(offer.kind, DccType::Send);
    }

    #[tokio::test]
    async fn test_send_and_receive_file() {
        use tokio::sync::mpsc;

        let dir = std::env::temp_dir();
        let src_path = dir.join("dcc_test_src.dat");
        let dst_path = dir.join("dcc_test_dst.dat");

        let data = b"Hello DCC file transfer!";
        tokio::fs::write(&src_path, data).await.unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (send_tx, _send_rx) = mpsc::unbounded_channel();
        let (recv_tx, _recv_rx) = mpsc::unbounded_channel();

        let src = src_path.clone();
        let send_handle =
            tokio::spawn(async move { send_file(&src, listener, 4096, send_tx).await.unwrap() });

        let offer = DccOffer {
            kind: DccType::Send,
            from: "test".into(),
            filename: Some("test.dat".into()),
            address: addr.ip(),
            port: addr.port(),
            size: Some(data.len() as u64),
        };

        let dst = dst_path.clone();
        let recv_handle =
            tokio::spawn(async move { receive_file(&offer, &dst, recv_tx).await.unwrap() });

        let received = recv_handle.await.unwrap();
        let sent = send_handle.await.unwrap();

        assert_eq!(sent, data.len() as u64);
        assert_eq!(received, data.len() as u64);

        let received_data = tokio::fs::read(&dst_path).await.unwrap();
        assert_eq!(received_data, data);

        let _ = tokio::fs::remove_file(&src_path).await;
        let _ = tokio::fs::remove_file(&dst_path).await;
    }

    #[tokio::test]
    async fn test_accept_chat() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let offer = DccOffer {
            kind: DccType::Chat,
            from: "test".into(),
            filename: None,
            address: addr.ip(),
            port: addr.port(),
            size: None,
        };

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            stream
        });

        let client = accept_chat(&offer).await.unwrap();
        let _server = server_handle.await.unwrap();

        assert!(client.peer_addr().is_ok());
    }
}
