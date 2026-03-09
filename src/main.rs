use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use bitchy::config::{Config, ServerConfig};
use bitchy::irc::client::{ClientCommand, IrcClient};
use bitchy::ui::app::App;

#[derive(Debug, Parser)]
#[command(
    name = "bitchy",
    version,
    about = "BitchY - unofficial Rust IRC client inspired by BitchX"
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

    /// Disable TLS (connect in plain text)
    #[arg(long)]
    no_tls: bool,

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
    #[arg(long, requires = "server")]
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

    config = apply_cli_overrides(config, &cli)?;

    if should_show_startup_banner(&cli) {
        bitchy::ui::ansi_art::print_startup_banner();
        println!("  Press any key to continue...");
        crossterm::terminal::enable_raw_mode()?;
        let _ = crossterm::event::read();
        crossterm::terminal::disable_raw_mode()?;
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
                .or_insert_with(|| bitchy::irc::channel::Channel::new(channel.clone()));
        }

        let result = app.run(&mut terminal).await;
        ratatui::restore();
        result?;
    }

    client_handle.abort();
    Ok(())
}

fn apply_cli_overrides(mut config: Config, cli: &Cli) -> Result<Config> {
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
        let use_tls = !cli.no_tls;
        let (host, port) = parse_server(server_str, cli.port, use_tls)?;
        let server_config = ServerConfig {
            host,
            port,
            tls: use_tls,
            password: cli.password.clone(),
            sasl_user: None,
            sasl_pass: None,
        };
        config.servers = vec![server_config];
        return Ok(config);
    }

    if cli.no_tls {
        for server in &mut config.servers {
            server.tls = false;
        }
    }

    if let Some(port) = cli.port {
        for server in &mut config.servers {
            server.port = port;
        }
    }

    Ok(config)
}

fn should_show_startup_banner(cli: &Cli) -> bool {
    !cli.no_ansi && !cli.dumb
}

fn parse_server(server_str: &str, cli_port: Option<u16>, use_tls: bool) -> Result<(String, u16)> {
    if let Some(stripped) = server_str.strip_prefix('[') {
        let (host, remainder) = stripped
            .split_once(']')
            .ok_or_else(|| anyhow::anyhow!("invalid bracketed server address: {server_str}"))?;

        if remainder.is_empty() {
            return Ok((host.to_string(), cli_port.unwrap_or(default_port(use_tls))));
        }

        let port_str = remainder
            .strip_prefix(':')
            .ok_or_else(|| anyhow::anyhow!("invalid bracketed server address: {server_str}"))?;
        let port = parse_embedded_port(port_str, server_str)?;
        return Ok((host.to_string(), cli_port.unwrap_or(port)));
    }

    if server_str.matches(':').count() == 1 {
        let (host, port_str) = server_str
            .rsplit_once(':')
            .ok_or_else(|| anyhow::anyhow!("invalid server address: {server_str}"))?;
        let port = parse_embedded_port(port_str, server_str)?;
        return Ok((host.to_string(), cli_port.unwrap_or(port)));
    }

    Ok((
        server_str.to_string(),
        cli_port.unwrap_or(default_port(use_tls)),
    ))
}

fn parse_embedded_port(port_str: &str, server_str: &str) -> Result<u16> {
    if port_str.is_empty() {
        anyhow::bail!("missing port in server address: {server_str}");
    }

    port_str
        .parse::<u16>()
        .map_err(|_| anyhow::anyhow!("invalid port in server address: {server_str}"))
}

const fn default_port(use_tls: bool) -> u16 {
    if use_tls {
        6697
    } else {
        6667
    }
}

async fn run_dumb_mode(
    mut event_rx: tokio::sync::mpsc::UnboundedReceiver<bitchy::irc::client::IrcEvent>,
    _cmd_tx: tokio::sync::mpsc::UnboundedSender<ClientCommand>,
) -> Result<()> {
    use bitchy::irc::client::IrcEvent;

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
    use clap::{error::ErrorKind, CommandFactory};

    #[test]
    fn parse_server_matrix() {
        struct Case {
            name: &'static str,
            server: &'static str,
            cli_port: Option<u16>,
            use_tls: bool,
            expected: Result<(&'static str, u16), &'static str>,
        }

        let cases = [
            Case {
                name: "host_with_embedded_port",
                server: "irc.libera.chat:6697",
                cli_port: None,
                use_tls: true,
                expected: Ok(("irc.libera.chat", 6697)),
            },
            Case {
                name: "host_without_port_tls_default",
                server: "irc.libera.chat",
                cli_port: None,
                use_tls: true,
                expected: Ok(("irc.libera.chat", 6697)),
            },
            Case {
                name: "host_without_port_plain_default",
                server: "irc.libera.chat",
                cli_port: None,
                use_tls: false,
                expected: Ok(("irc.libera.chat", 6667)),
            },
            Case {
                name: "cli_port_overrides_embedded_port",
                server: "irc.libera.chat:6697",
                cli_port: Some(7000),
                use_tls: true,
                expected: Ok(("irc.libera.chat", 7000)),
            },
            Case {
                name: "host_only_with_cli_port",
                server: "irc.example.com",
                cli_port: Some(7000),
                use_tls: true,
                expected: Ok(("irc.example.com", 7000)),
            },
            Case {
                name: "bare_ipv6_keeps_full_host",
                server: "2001:db8::1",
                cli_port: None,
                use_tls: true,
                expected: Ok(("2001:db8::1", 6697)),
            },
            Case {
                name: "bracketed_ipv6_with_port",
                server: "[2001:db8::1]:7000",
                cli_port: None,
                use_tls: true,
                expected: Ok(("2001:db8::1", 7000)),
            },
            Case {
                name: "bracketed_ipv6_with_cli_port_override",
                server: "[2001:db8::1]:7000",
                cli_port: Some(9000),
                use_tls: true,
                expected: Ok(("2001:db8::1", 9000)),
            },
            Case {
                name: "invalid_embedded_port_is_rejected",
                server: "irc.example.com:notaport",
                cli_port: None,
                use_tls: true,
                expected: Err("invalid port"),
            },
            Case {
                name: "missing_embedded_port_is_rejected",
                server: "irc.example.com:",
                cli_port: None,
                use_tls: true,
                expected: Err("missing port"),
            },
            Case {
                name: "invalid_bracketed_ipv6_is_rejected",
                server: "[2001:db8::1",
                cli_port: None,
                use_tls: true,
                expected: Err("invalid bracketed server address"),
            },
        ];

        for case in cases {
            let result = parse_server(case.server, case.cli_port, case.use_tls);
            match (result, case.expected) {
                (Ok((host, port)), Ok((expected_host, expected_port))) => {
                    assert_eq!(host, expected_host, "{}", case.name);
                    assert_eq!(port, expected_port, "{}", case.name);
                }
                (Err(err), Err(expected_message)) => {
                    assert!(
                        err.to_string().contains(expected_message),
                        "{}: expected error containing {expected_message:?}, got {err}",
                        case.name
                    );
                }
                (Ok((host, port)), Err(expected_message)) => {
                    panic!(
                        "{}: expected error containing {expected_message:?}, got success {host}:{port}",
                        case.name
                    );
                }
                (Err(err), Ok((expected_host, expected_port))) => {
                    panic!(
                        "{}: expected success {expected_host}:{expected_port}, got error {err}",
                        case.name
                    );
                }
            }
        }
    }

    #[test]
    fn cli_default_values() {
        let cli = Cli::parse_from(["bitchy"]);
        assert!(cli.nick.is_none());
        assert!(cli.server.is_none());
        assert!(cli.channel.is_none());
        assert!(cli.config.is_none());
        assert!(!cli.no_tls);
        assert!(cli.port.is_none());
        assert!(!cli.no_verify);
        assert!(!cli.dumb);
        assert!(!cli.no_ansi);
        assert!(cli.password.is_none());
        assert!(!cli.debug);
    }

    #[test]
    fn cli_with_nick_and_server() {
        let cli = Cli::parse_from(["bitchy", "--nick", "testbot", "--server", "irc.libera.chat"]);
        assert_eq!(cli.nick.as_deref(), Some("testbot"));
        assert_eq!(cli.server.as_deref(), Some("irc.libera.chat"));
    }

    #[test]
    fn cli_with_channels() {
        let cli = Cli::parse_from(["bitchy", "--channel", "#rust", "--channel", "#bitchy"]);
        assert_eq!(
            cli.channel.as_deref(),
            Some(vec!["#rust".to_string(), "#bitchy".to_string()].as_slice())
        );
    }

    #[test]
    fn cli_dumb_mode() {
        let cli = Cli::parse_from(["bitchy", "--dumb"]);
        assert!(cli.dumb);
    }

    #[test]
    fn cli_debug_mode() {
        let cli = Cli::parse_from(["bitchy", "-x"]);
        assert!(cli.debug);
    }

    #[test]
    fn cli_no_ansi() {
        let cli = Cli::parse_from(["bitchy", "-A"]);
        assert!(cli.no_ansi);
    }

    #[test]
    fn cli_port_flag() {
        let cli = Cli::parse_from(["bitchy", "--port", "7000"]);
        assert_eq!(cli.port, Some(7000));
    }

    #[test]
    fn cli_no_verify() {
        let cli = Cli::parse_from(["bitchy", "--no-verify"]);
        assert!(cli.no_verify);
    }

    #[test]
    fn cli_config_path() {
        let cli = Cli::parse_from(["bitchy", "-f", "/etc/bitchy.toml"]);
        assert_eq!(
            cli.config,
            Some(std::path::PathBuf::from("/etc/bitchy.toml"))
        );
    }

    #[test]
    fn cli_password() {
        let cli = Cli::parse_from([
            "bitchy",
            "--server",
            "irc.libera.chat",
            "--password",
            "secret",
        ]);
        assert_eq!(cli.password.as_deref(), Some("secret"));
    }

    #[test]
    fn cli_parser_matrix_accepts_expected_combinations() {
        let cases = [
            vec!["bitchx", "-n", "testbot", "-s", "irc.libera.chat"],
            vec![
                "bitchx",
                "--server",
                "irc.libera.chat",
                "--channel",
                "#rust",
                "--channel",
                "#bitchx",
            ],
            vec![
                "bitchx",
                "--server",
                "irc.libera.chat",
                "--password",
                "secret",
                "--no-tls",
            ],
            vec!["bitchx", "-f", "/etc/bitchx.toml", "-A", "-x", "--dumb"],
            vec!["bitchx", "--server", "[2001:db8::1]:6697", "--port", "7000"],
        ];

        for args in cases {
            Cli::try_parse_from(args).unwrap();
        }
    }

    #[test]
    fn cli_parser_matrix_rejects_invalid_combinations() {
        let cases = [
            (vec!["bitchx", "--unknown"], ErrorKind::UnknownArgument),
            (vec!["bitchx", "--server"], ErrorKind::InvalidValue),
            (
                vec!["bitchx", "--port", "not-a-port"],
                ErrorKind::ValueValidation,
            ),
            (
                vec!["bitchx", "--port", "70000"],
                ErrorKind::ValueValidation,
            ),
            (
                vec!["bitchx", "--password", "secret"],
                ErrorKind::MissingRequiredArgument,
            ),
            (
                vec!["bitchx", "--server", "irc.libera.chat", "--password"],
                ErrorKind::InvalidValue,
            ),
        ];

        for (args, expected_kind) in cases {
            let err = Cli::try_parse_from(args).unwrap_err();
            assert_eq!(err.kind(), expected_kind);
        }
    }

    #[test]
    fn cli_help_and_version_matrix_return_display_errors() {
        let cases = [
            (vec!["bitchx", "--help"], ErrorKind::DisplayHelp),
            (vec!["bitchx", "-h"], ErrorKind::DisplayHelp),
            (vec!["bitchx", "--version"], ErrorKind::DisplayVersion),
            (vec!["bitchx", "-V"], ErrorKind::DisplayVersion),
        ];

        for (args, expected_kind) in cases {
            let err = Cli::try_parse_from(args).unwrap_err();
            assert_eq!(err.kind(), expected_kind);
        }
    }

    #[test]
    fn cli_help_lists_core_flags() {
        let help = Cli::command().render_long_help().to_string();
        for expected_flag in [
            "--server",
            "--nick",
            "--channel",
            "--no-tls",
            "--port",
            "--no-verify",
            "--dumb",
            "--password",
        ] {
            assert!(
                help.contains(expected_flag),
                "missing {expected_flag} in help text"
            );
        }
    }

    #[test]
    fn cli_override_matrix_applies_expected_config_changes() {
        struct Case {
            name: &'static str,
            args: Vec<&'static str>,
            base_config: Config,
            expected_nick: &'static str,
            expected_channels: Vec<&'static str>,
            expected_verify_certs: bool,
            expected_servers: Vec<(&'static str, u16, bool, Option<&'static str>)>,
        }

        let configured_servers = vec![
            ServerConfig {
                host: "irc.one.example".into(),
                port: 6697,
                tls: true,
                password: None,
                sasl_user: None,
                sasl_pass: None,
            },
            ServerConfig {
                host: "irc.two.example".into(),
                port: 7000,
                tls: true,
                password: None,
                sasl_user: None,
                sasl_pass: None,
            },
        ];

        let cases = [
            Case {
                name: "single_server_cli_overrides",
                args: vec![
                    "bitchx",
                    "--nick",
                    "matrixbot",
                    "--server",
                    "irc.libera.chat:6697",
                    "--port",
                    "7000",
                    "--password",
                    "secret",
                    "--channel",
                    "#rust",
                    "--no-verify",
                ],
                base_config: Config::default(),
                expected_nick: "matrixbot",
                expected_channels: vec!["#rust"],
                expected_verify_certs: false,
                expected_servers: vec![("irc.libera.chat", 7000, true, Some("secret"))],
            },
            Case {
                name: "config_servers_get_no_tls_and_port_override",
                args: vec!["bitchx", "--no-tls", "--port", "6667"],
                base_config: Config {
                    nick: "configured-nick".into(),
                    servers: configured_servers.clone(),
                    ..Config::default()
                },
                expected_nick: "configured-nick",
                expected_channels: vec![],
                expected_verify_certs: true,
                expected_servers: vec![
                    ("irc.one.example", 6667, false, None),
                    ("irc.two.example", 6667, false, None),
                ],
            },
            Case {
                name: "bracketed_ipv6_server_cli_path",
                args: vec!["bitchx", "--server", "[2001:db8::1]:6697", "--no-tls"],
                base_config: Config {
                    nick: "configured-nick".into(),
                    ..Config::default()
                },
                expected_nick: "configured-nick",
                expected_channels: vec![],
                expected_verify_certs: true,
                expected_servers: vec![("2001:db8::1", 6697, false, None)],
            },
        ];

        for case in cases {
            let cli = Cli::try_parse_from(case.args).unwrap();
            let updated = apply_cli_overrides(case.base_config, &cli).unwrap();

            assert_eq!(updated.nick, case.expected_nick, "{}", case.name);
            assert_eq!(
                updated.auto_join,
                case.expected_channels
                    .into_iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>(),
                "{}",
                case.name
            );
            assert_eq!(
                updated.tls.verify_certs, case.expected_verify_certs,
                "{}",
                case.name
            );
            assert_eq!(
                updated.servers.len(),
                case.expected_servers.len(),
                "{}",
                case.name
            );

            for (server, (expected_host, expected_port, expected_tls, expected_password)) in
                updated.servers.iter().zip(case.expected_servers.iter())
            {
                assert_eq!(server.host, *expected_host, "{}", case.name);
                assert_eq!(server.port, *expected_port, "{}", case.name);
                assert_eq!(server.tls, *expected_tls, "{}", case.name);
                assert_eq!(
                    server.password.as_deref(),
                    *expected_password,
                    "{}",
                    case.name
                );
            }
        }
    }

    #[test]
    fn startup_banner_matrix() {
        let cases = [
            (vec!["bitchx"], true),
            (vec!["bitchx", "--dumb"], false),
            (vec!["bitchx", "--no-ansi"], false),
            (vec!["bitchx", "--dumb", "--no-ansi"], false),
        ];

        for (args, expected) in cases {
            let cli = Cli::try_parse_from(args).unwrap();
            assert_eq!(should_show_startup_banner(&cli), expected);
        }
    }
}
