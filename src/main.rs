use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use bitchx::config::{Config, ServerConfig};
use bitchx::irc::client::{ClientCommand, IrcClient};
use bitchx::ui::app::App;

#[derive(Parser)]
#[command(
    name = "bitchx",
    version = "2.0.0",
    about = "BitchX IRC Client - Rust Rewrite"
)]
struct Cli {
    /// Nickname to use
    #[arg(short, long)]
    nick: Option<String>,

    /// Server to connect to (host[:port])
    #[arg(short, long)]
    server: Option<String>,

    /// Channel to join on connect
    #[arg(short, long)]
    channel: Option<Vec<String>>,

    /// Config file path
    #[arg(short = 'f', long)]
    config: Option<std::path::PathBuf>,

    /// Use TLS (default: true)
    #[arg(long, default_value_t = true)]
    tls: bool,

    /// Port number (default: 6697 for TLS, 6667 for plain)
    #[arg(short, long)]
    port: Option<u16>,

    /// Skip certificate verification (insecure)
    #[arg(long)]
    no_verify: bool,

    /// Run in dumb/non-interactive mode
    #[arg(short, long)]
    dumb: bool,

    /// Don't display startup ANSI art
    #[arg(short = 'A', long)]
    no_ansi: bool,

    /// Password for server
    #[arg(long)]
    password: Option<String>,

    /// Enable debug logging
    #[arg(short = 'x', long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let mut config = match &cli.config {
        Some(path) => Config::load(path).unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to load config from {}: {e}, using defaults",
                path.display()
            );
            Config::default()
        }),
        None => {
            let default_path = Config::default_config_path();
            if default_path.exists() {
                Config::load(&default_path).unwrap_or_else(|e| {
                    tracing::warn!("Failed to load default config: {e}, using defaults");
                    Config::default()
                })
            } else {
                Config::default()
            }
        }
    };

    if let Some(nick) = &cli.nick {
        config.nick = nick.clone();
    }

    if let Some(channels) = &cli.channel {
        config.auto_join = channels.clone();
    }

    if cli.no_verify {
        config.tls.verify_certs = false;
    }

    if let Some(server_str) = &cli.server {
        let (host, port) = parse_server(server_str, cli.port, cli.tls);
        let server_config = ServerConfig {
            host,
            port,
            tls: cli.tls,
            password: cli.password.clone(),
            sasl_user: None,
            sasl_pass: None,
        };
        config.servers = vec![server_config];
    } else if let Some(port) = cli.port {
        for s in &mut config.servers {
            s.port = port;
        }
    }

    if !cli.no_ansi {
        print_banner();
    }

    let (client, cmd_tx, event_rx) = IrcClient::new();

    if let Some(server_config) = config.servers.first().cloned() {
        cmd_tx.send(ClientCommand::Connect(server_config))?;
    }

    let config_clone = config.clone();
    let client_handle = tokio::spawn(async move {
        if let Err(e) = client.run(&config_clone).await {
            tracing::error!("IRC client error: {e}");
        }
    });

    if cli.dumb {
        run_dumb_mode(event_rx, cmd_tx).await?;
    } else {
        let mut terminal = ratatui::init();
        let mut app = App::new(config, event_rx, cmd_tx);

        for channel in &app.config.auto_join.clone() {
            let _ = app
                .channels
                .entry(channel.clone())
                .or_insert_with(|| bitchx::irc::channel::Channel::new(channel.clone()));
        }

        let result = app.run(&mut terminal).await;
        ratatui::restore();
        result?;
    }

    client_handle.abort();
    Ok(())
}

fn print_banner() {
    const BANNER: &str = r#"
 ____  _ _       _   __  __
| __ )(_) |_ ___| |_\ \/ /
|  _ \| | __/ __| '_ \\  /
| |_) | | || (__| | | /  \
|____/|_|\__\___|_| |_/_/\_\

  BitchX 2.0 - Rust Rewrite
  Type /help for commands
"#;
    println!("{BANNER}");
}

fn parse_server(server_str: &str, cli_port: Option<u16>, use_tls: bool) -> (String, u16) {
    if let Some((host, port_str)) = server_str.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            return (host.to_string(), cli_port.unwrap_or(port));
        }
    }
    let default_port = if use_tls { 6697 } else { 6667 };
    (server_str.to_string(), cli_port.unwrap_or(default_port))
}

async fn run_dumb_mode(
    mut event_rx: tokio::sync::mpsc::UnboundedReceiver<bitchx::irc::client::IrcEvent>,
    _cmd_tx: tokio::sync::mpsc::UnboundedSender<ClientCommand>,
) -> Result<()> {
    use bitchx::irc::client::IrcEvent;

    tracing::info!("Running in dumb mode (non-interactive)");

    loop {
        tokio::select! {
            event = event_rx.recv() => {
                match event {
                    Some(IrcEvent::Connected) => {
                        println!("*** Connected to server");
                    }
                    Some(IrcEvent::Disconnected(reason)) => {
                        println!("*** Disconnected: {reason}");
                        break;
                    }
                    Some(IrcEvent::Message(msg)) => {
                        println!("{msg}");
                    }
                    Some(IrcEvent::Error(err)) => {
                        eprintln!("*** Error: {err}");
                    }
                    Some(IrcEvent::LagUpdate(ms)) => {
                        tracing::debug!("Lag: {ms}ms");
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n*** Interrupted");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_server_with_port() {
        let (host, port) = parse_server("irc.libera.chat:6697", None, true);
        assert_eq!(host, "irc.libera.chat");
        assert_eq!(port, 6697);
    }

    #[test]
    fn parse_server_without_port_tls() {
        let (host, port) = parse_server("irc.libera.chat", None, true);
        assert_eq!(host, "irc.libera.chat");
        assert_eq!(port, 6697);
    }

    #[test]
    fn parse_server_without_port_plain() {
        let (host, port) = parse_server("irc.libera.chat", None, false);
        assert_eq!(host, "irc.libera.chat");
        assert_eq!(port, 6667);
    }

    #[test]
    fn parse_server_cli_port_override() {
        let (host, port) = parse_server("irc.libera.chat:6697", Some(7000), true);
        assert_eq!(host, "irc.libera.chat");
        assert_eq!(port, 7000);
    }

    #[test]
    fn parse_server_host_only_with_cli_port() {
        let (host, port) = parse_server("irc.example.com", Some(7000), true);
        assert_eq!(host, "irc.example.com");
        assert_eq!(port, 7000);
    }

    #[test]
    fn cli_default_values() {
        let cli = Cli::parse_from(["bitchx"]);
        assert!(cli.nick.is_none());
        assert!(cli.server.is_none());
        assert!(cli.channel.is_none());
        assert!(cli.config.is_none());
        assert!(cli.tls);
        assert!(cli.port.is_none());
        assert!(!cli.no_verify);
        assert!(!cli.dumb);
        assert!(!cli.no_ansi);
        assert!(cli.password.is_none());
        assert!(!cli.debug);
    }

    #[test]
    fn cli_with_nick_and_server() {
        let cli = Cli::parse_from(["bitchx", "--nick", "testbot", "--server", "irc.libera.chat"]);
        assert_eq!(cli.nick.as_deref(), Some("testbot"));
        assert_eq!(cli.server.as_deref(), Some("irc.libera.chat"));
    }

    #[test]
    fn cli_with_channels() {
        let cli = Cli::parse_from(["bitchx", "--channel", "#rust", "--channel", "#bitchx"]);
        assert_eq!(
            cli.channel.as_deref(),
            Some(vec!["#rust".to_string(), "#bitchx".to_string()].as_slice())
        );
    }

    #[test]
    fn cli_dumb_mode() {
        let cli = Cli::parse_from(["bitchx", "--dumb"]);
        assert!(cli.dumb);
    }

    #[test]
    fn cli_debug_mode() {
        let cli = Cli::parse_from(["bitchx", "-x"]);
        assert!(cli.debug);
    }

    #[test]
    fn cli_no_ansi() {
        let cli = Cli::parse_from(["bitchx", "-A"]);
        assert!(cli.no_ansi);
    }

    #[test]
    fn cli_port_flag() {
        let cli = Cli::parse_from(["bitchx", "--port", "7000"]);
        assert_eq!(cli.port, Some(7000));
    }

    #[test]
    fn cli_no_verify() {
        let cli = Cli::parse_from(["bitchx", "--no-verify"]);
        assert!(cli.no_verify);
    }

    #[test]
    fn cli_config_path() {
        let cli = Cli::parse_from(["bitchx", "-f", "/etc/bitchx.toml"]);
        assert_eq!(
            cli.config,
            Some(std::path::PathBuf::from("/etc/bitchx.toml"))
        );
    }

    #[test]
    fn cli_password() {
        let cli = Cli::parse_from(["bitchx", "--password", "secret"]);
        assert_eq!(cli.password.as_deref(), Some("secret"));
    }
}
