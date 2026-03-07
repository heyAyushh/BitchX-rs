use tokio::sync::mpsc;

use super::command::IrcCommand;
use super::connection::IrcConnection;
use super::message::IrcMessage;
use crate::config::{Config, ServerConfig};

#[derive(Debug, Clone)]
pub enum IrcEvent {
    Connected,
    Disconnected(String),
    Message(IrcMessage),
    Error(String),
    LagUpdate(u64),
}

#[derive(Debug, Clone)]
pub enum ClientCommand {
    Connect(ServerConfig),
    SendRaw(String),
    Join(String),
    Part(String, Option<String>),
    Privmsg(String, String),
    Notice(String, String),
    Nick(String),
    Quit(Option<String>),
    Mode(String, String),
    Kick(String, String, Option<String>),
    Topic(String, Option<String>),
    Away(Option<String>),
    Ctcp(String, String),
    Ping(String),
}

pub struct IrcClient {
    event_tx: mpsc::UnboundedSender<IrcEvent>,
    cmd_rx: mpsc::UnboundedReceiver<ClientCommand>,
}

impl IrcClient {
    pub fn new() -> (
        Self,
        mpsc::UnboundedSender<ClientCommand>,
        mpsc::UnboundedReceiver<IrcEvent>,
    ) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        (Self { event_tx, cmd_rx }, cmd_tx, event_rx)
    }

    pub async fn run(mut self, config: &Config) -> crate::error::Result<()> {
        loop {
            let cmd = self.cmd_rx.recv().await;
            match cmd {
                Some(ClientCommand::Connect(server_config)) => {
                    let should_quit = self.handle_connection(&server_config, config).await;
                    if should_quit {
                        break;
                    }
                }
                Some(ClientCommand::Quit(_)) | None => break,
                _ => {
                    let _ = self
                        .event_tx
                        .send(IrcEvent::Error("Not connected to any server".into()));
                }
            }
        }
        Ok(())
    }

    async fn handle_connection(&mut self, server_config: &ServerConfig, config: &Config) -> bool {
        let host = &server_config.host;
        let port = server_config.port;

        let conn = match IrcConnection::connect(host, port, server_config.tls, &config.tls).await {
            Ok(c) => c,
            Err(e) => {
                let _ = self.event_tx.send(IrcEvent::Error(format!(
                    "Failed to connect to {host}:{port}: {e}"
                )));
                return false;
            }
        };

        let _ = self.event_tx.send(IrcEvent::Connected);

        let (mut reader, mut writer) = conn.split();
        let mut current_nick = config.nick.clone();
        let mut alt_nick_idx = 0usize;
        let mut auto_joined = false;

        let has_sasl = server_config.sasl_user.is_some() && server_config.sasl_pass.is_some();
        let mut cap_ended = false;
        let mut sasl_authenticating = false;

        if let Some(ref pass) = server_config.password {
            let _ = writer.send(&IrcCommand::pass(pass)).await;
        }

        if has_sasl {
            let _ = writer
                .send(&IrcCommand::cap_req(&["multi-prefix", "sasl"]))
                .await;
        } else {
            let _ = writer.send(&IrcCommand::cap_req(&["multi-prefix"])).await;
        }
        let _ = writer.send(&IrcCommand::nick(&current_nick)).await;
        let _ = writer
            .send(&IrcCommand::user(&config.username, &config.realname))
            .await;
        if !has_sasl {
            let _ = writer.send(&IrcCommand::cap_end()).await;
            cap_ended = true;
        }

        loop {
            tokio::select! {
                line = reader.recv() => {
                    match line {
                        Ok(Some(raw)) => {
                            match IrcMessage::parse(&raw) {
                                Ok(msg) => {
                                    if msg.command == "PING" {
                                        if let Some(token) = msg.trailing() {
                                            let _ = writer.send(&IrcCommand::pong(token)).await;
                                        }
                                    }

                                    if msg.command == "433" || msg.command == "436" {
                                        if alt_nick_idx < config.alt_nicks.len() {
                                            current_nick = config.alt_nicks[alt_nick_idx].clone();
                                            alt_nick_idx += 1;
                                        } else {
                                            current_nick.push('_');
                                        }
                                        let _ = writer.send(&IrcCommand::nick(&current_nick)).await;
                                    }

                                    if msg.command == "CAP" {
                                        let sub = msg.params.get(1).map(|s| s.as_str()).unwrap_or("");
                                        if sub == "ACK" {
                                            let acked = msg.trailing().unwrap_or("");
                                            if has_sasl && acked.split_whitespace().any(|c| c == "sasl") {
                                                let _ = writer.send("AUTHENTICATE PLAIN\r\n").await;
                                                sasl_authenticating = true;
                                            } else if !cap_ended {
                                                let _ = writer.send(&IrcCommand::cap_end()).await;
                                                cap_ended = true;
                                            }
                                        } else if sub == "NAK" && !cap_ended {
                                            let _ = writer.send(&IrcCommand::cap_end()).await;
                                            cap_ended = true;
                                        }
                                    }

                                    if msg.command == "AUTHENTICATE"
                                        && sasl_authenticating
                                        && msg.params.first().is_some_and(|p| p == "+")
                                    {
                                        let user = server_config.sasl_user.as_deref().unwrap_or("");
                                        let pass = server_config.sasl_pass.as_deref().unwrap_or("");
                                        let payload = format!("\0{user}\0{pass}");
                                        use base64::Engine as _;
                                        let encoded = base64::engine::general_purpose::STANDARD
                                            .encode(payload.as_bytes());
                                        let _ = writer.send(&format!("AUTHENTICATE {encoded}\r\n")).await;
                                        sasl_authenticating = false;
                                    }

                                    // 903 = RPL_SASLSUCCESS
                                    if msg.command == "903" && !cap_ended {
                                        let _ = writer.send(&IrcCommand::cap_end()).await;
                                        cap_ended = true;
                                    }

                                    // 904/905 = ERR_SASLFAIL / ERR_SASLALREADY
                                    if (msg.command == "904" || msg.command == "905") && !cap_ended {
                                        let _ = self.event_tx.send(IrcEvent::Error(
                                            "SASL authentication failed".into(),
                                        ));
                                        let _ = writer.send(&IrcCommand::cap_end()).await;
                                        cap_ended = true;
                                    }

                                    if !auto_joined
                                        && matches!(msg.command.as_str(), "001" | "376" | "422")
                                    {
                                        for channel in &config.auto_join {
                                            let _ = writer.send(&IrcCommand::join(channel)).await;
                                        }
                                        auto_joined = true;
                                    }

                                    let _ = self.event_tx.send(IrcEvent::Message(msg));
                                }
                                Err(e) => {
                                    let _ = self.event_tx.send(IrcEvent::Error(
                                        format!("Parse error: {e}"),
                                    ));
                                }
                            }
                        }
                        Ok(None) => {
                            let _ = self.event_tx.send(IrcEvent::Disconnected(
                                "Connection closed by server".into(),
                            ));
                            return false;
                        }
                        Err(e) => {
                            let _ = self.event_tx.send(IrcEvent::Disconnected(
                                format!("Read error: {e}"),
                            ));
                            return false;
                        }
                    }
                }
                cmd = self.cmd_rx.recv() => {
                    match cmd {
                        Some(ClientCommand::SendRaw(raw)) => {
                            let _ = writer.send(&raw).await;
                        }
                        Some(ClientCommand::Quit(reason)) => {
                            let _ = writer.send(&IrcCommand::quit(reason.as_deref())).await;
                            let _ = self.event_tx.send(IrcEvent::Disconnected(
                                "Client quit".into(),
                            ));
                            return true;
                        }
                        Some(ClientCommand::Privmsg(target, text)) => {
                            let _ = writer.send(&IrcCommand::privmsg(&target, &text)).await;
                        }
                        Some(ClientCommand::Notice(target, text)) => {
                            let _ = writer.send(&IrcCommand::notice(&target, &text)).await;
                        }
                        Some(ClientCommand::Join(channel)) => {
                            let _ = writer.send(&IrcCommand::join(&channel)).await;
                        }
                        Some(ClientCommand::Part(channel, reason)) => {
                            let _ = writer.send(&IrcCommand::part(&channel, reason.as_deref())).await;
                        }
                        Some(ClientCommand::Nick(nick)) => {
                            current_nick = nick.clone();
                            let _ = writer.send(&IrcCommand::nick(&nick)).await;
                        }
                        Some(ClientCommand::Topic(channel, topic)) => {
                            let _ = writer.send(&IrcCommand::topic(&channel, topic.as_deref())).await;
                        }
                        Some(ClientCommand::Away(msg)) => {
                            let _ = writer.send(&IrcCommand::away(msg.as_deref())).await;
                        }
                        Some(ClientCommand::Mode(target, mode_str)) => {
                            let raw = format!("MODE {target} {mode_str}\r\n");
                            let _ = writer.send(&raw).await;
                        }
                        Some(ClientCommand::Kick(channel, nick, reason)) => {
                            let _ = writer.send(&IrcCommand::kick(&channel, &nick, reason.as_deref())).await;
                        }
                        Some(ClientCommand::Ctcp(target, msg)) => {
                            let ctcp_msg = format!("\x01{msg}\x01");
                            let _ = writer.send(&IrcCommand::privmsg(&target, &ctcp_msg)).await;
                        }
                        Some(ClientCommand::Ping(token)) => {
                            let _ = writer.send(&IrcCommand::ping(&token)).await;
                        }
                        Some(ClientCommand::Connect(_)) => {
                            let _ = self.event_tx.send(IrcEvent::Error(
                                "Already connected".into(),
                            ));
                        }
                        None => return true,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn test_config() -> Config {
        Config {
            nick: "testbot".into(),
            username: "testuser".into(),
            realname: "Test Bot".into(),
            ..Config::default()
        }
    }

    #[tokio::test]
    async fn client_new_creates_channels() {
        let (_client, cmd_tx, mut event_rx) = IrcClient::new();
        assert!(!cmd_tx.is_closed());
        assert!(event_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn client_sends_registration_on_connect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let mut all_data = String::new();

            for _ in 0..5 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, mut event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        let connected = tokio::time::timeout(tokio::time::Duration::from_secs(2), event_rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(
            matches!(connected, IrcEvent::Connected),
            "Expected Connected event, got {:?}",
            connected
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(data.contains("CAP REQ"), "Should send CAP REQ, got: {data}");
        assert!(data.contains("NICK"), "Should send NICK, got: {data}");
        assert!(data.contains("USER"), "Should send USER, got: {data}");
        assert!(
            data.contains("testbot"),
            "Should use configured nick, got: {data}"
        );
        assert!(data.contains("CAP END"), "Should send CAP END, got: {data}");
    }

    #[tokio::test]
    async fn client_responds_to_ping() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            sock.write_all(b"PING :test123\r\n").await.unwrap();

            let mut all_data = String::new();
            for _ in 0..10 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if all_data.contains("PONG") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, _event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(
            data.contains("PONG :test123"),
            "Should respond to PING with PONG, got: {data}"
        );
    }

    #[tokio::test]
    async fn client_forwards_messages_as_events() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            sock.write_all(b":server 001 testbot :Welcome\r\n")
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            sock.shutdown().await.ok();
        });

        let (client, cmd_tx, mut event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        let mut got_welcome = false;
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);

        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(tokio::time::Duration::from_millis(200), event_rx.recv())
                .await
            {
                Ok(Some(IrcEvent::Message(msg))) => {
                    if msg.command == "001" {
                        got_welcome = true;
                        break;
                    }
                }
                Ok(Some(_)) => continue,
                _ => break,
            }
        }

        assert!(got_welcome, "Should receive 001 welcome message as event");

        drop(cmd_tx);
        let _ = client_handle.await;
    }

    #[tokio::test]
    async fn client_sends_raw_command() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let mut all_data = String::new();

            for _ in 0..10 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if all_data.contains("PRIVMSG") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, _event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        cmd_tx
            .send(ClientCommand::SendRaw(
                "PRIVMSG #test :Hello world\r\n".into(),
            ))
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(
            data.contains("PRIVMSG #test :Hello world"),
            "Should forward raw command to server, got: {data}"
        );
    }

    #[tokio::test]
    async fn client_quit_command() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let mut all_data = String::new();

            for _ in 0..10 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if all_data.contains("QUIT") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            all_data
        });

        let (client, cmd_tx, mut event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        cmd_tx
            .send(ClientCommand::Quit(Some("Goodbye".into())))
            .unwrap();

        let mut got_disconnect = false;
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);

        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(tokio::time::Duration::from_millis(200), event_rx.recv())
                .await
            {
                Ok(Some(IrcEvent::Disconnected(reason))) => {
                    assert_eq!(reason, "Client quit");
                    got_disconnect = true;
                    break;
                }
                Ok(Some(_)) => continue,
                _ => break,
            }
        }

        assert!(got_disconnect, "Should receive Disconnected event on quit");

        let _ = client_handle.await;
        let data = server_handle.await.unwrap();
        assert!(
            data.contains("QUIT :Goodbye"),
            "Should send QUIT to server, got: {data}"
        );
    }

    #[tokio::test]
    async fn client_sends_password_when_configured() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let mut all_data = String::new();

            for _ in 0..5 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, _event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: Some("secret123".into()),
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(
            data.contains("PASS :secret123"),
            "Should send PASS when password configured, got: {data}"
        );
    }

    #[tokio::test]
    async fn client_disconnect_event_on_server_close() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            drop(sock);
        });

        let (client, cmd_tx, mut event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        let mut got_disconnect = false;
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);

        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(tokio::time::Duration::from_millis(200), event_rx.recv())
                .await
            {
                Ok(Some(IrcEvent::Disconnected(reason))) => {
                    assert!(
                        reason.contains("closed")
                            || reason.contains("reset")
                            || reason.contains("error"),
                        "Unexpected disconnect reason: {reason}"
                    );
                    got_disconnect = true;
                    break;
                }
                Ok(Some(_)) => continue,
                _ => break,
            }
        }

        assert!(
            got_disconnect,
            "Should receive Disconnected event when server closes"
        );

        drop(cmd_tx);
        let _ = client_handle.await;
    }

    #[tokio::test]
    async fn client_connect_failure_sends_error_event() {
        let (client, cmd_tx, mut event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: "127.0.0.1".into(),
            port: 1,
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        let mut got_error = false;
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);

        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(tokio::time::Duration::from_millis(200), event_rx.recv())
                .await
            {
                Ok(Some(IrcEvent::Error(msg))) => {
                    assert!(msg.contains("Failed to connect"), "Error: {msg}");
                    got_error = true;
                    break;
                }
                Ok(Some(_)) => continue,
                _ => break,
            }
        }

        assert!(
            got_error,
            "Should receive Error event on connection failure"
        );

        drop(cmd_tx);
        let _ = client_handle.await;
    }

    #[tokio::test]
    async fn client_handles_nick_collision() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            sock.write_all(b":server 433 * testbot :Nickname is already in use\r\n")
                .await
                .unwrap();

            let mut all_data = String::new();
            for _ in 0..10 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if all_data.contains("testbot_") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, _event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(
            data.contains("testbot_"),
            "Should retry with underscore suffix on nick collision, got: {data}"
        );
    }

    #[tokio::test]
    async fn client_sends_privmsg_command() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let mut all_data = String::new();

            for _ in 0..10 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if all_data.contains("PRIVMSG") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, _event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        cmd_tx
            .send(ClientCommand::Privmsg("#test".into(), "Hello!".into()))
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(
            data.contains("PRIVMSG #test :Hello!"),
            "Should send PRIVMSG, got: {data}"
        );
    }

    #[tokio::test]
    async fn client_sends_join_command() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let mut all_data = String::new();

            for _ in 0..10 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if all_data.contains("JOIN") && all_data.contains("#mychan") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, _event_rx) = IrcClient::new();
        let config = test_config();

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        cmd_tx.send(ClientCommand::Join("#mychan".into())).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(
            data.contains("JOIN :#mychan"),
            "Should send JOIN, got: {data}"
        );
    }

    #[tokio::test]
    async fn client_auto_joins_configured_channels_after_welcome() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let mut all_data = String::new();
            let mut sent_welcome = false;

            for _ in 0..12 {
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(200),
                    sock.read(&mut buf),
                )
                .await
                {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        all_data.push_str(&String::from_utf8_lossy(&buf[..n]));

                        if !sent_welcome && all_data.contains("USER") {
                            sock.write_all(b":server 001 testbot :Welcome\r\n")
                                .await
                                .unwrap();
                            sent_welcome = true;
                        }

                        if all_data.contains("JOIN :#rust") && all_data.contains("JOIN :#bitchx") {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            sock.shutdown().await.ok();
            all_data
        });

        let (client, cmd_tx, _event_rx) = IrcClient::new();
        let mut config = test_config();
        config.auto_join = vec!["#rust".into(), "#bitchx".into()];

        let server_config = ServerConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            tls: false,
            password: None,
            sasl_user: None,
            sasl_pass: None,
        };

        cmd_tx.send(ClientCommand::Connect(server_config)).unwrap();

        let client_handle = tokio::spawn(async move { client.run(&config).await });

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        drop(cmd_tx);
        let _ = client_handle.await;

        let data = server_handle.await.unwrap();
        assert!(
            data.contains("JOIN :#rust"),
            "Should auto-join #rust after welcome, got: {data}"
        );
        assert!(
            data.contains("JOIN :#bitchx"),
            "Should auto-join #bitchx after welcome, got: {data}"
        );
    }
}
