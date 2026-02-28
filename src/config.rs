use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_nick")]
    pub nick: String,
    #[serde(default)]
    pub alt_nicks: Vec<String>,
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_realname")]
    pub realname: String,
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default)]
    pub auto_join: Vec<String>,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub flood: FloodConfig,
    #[serde(default)]
    pub dcc: DccConfig,
    #[serde(default)]
    pub logging: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub sasl_user: Option<String>,
    #[serde(default)]
    pub sasl_pass: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default = "default_true")]
    pub verify_certs: bool,
    #[serde(default)]
    pub client_cert: Option<PathBuf>,
    #[serde(default)]
    pub client_key: Option<PathBuf>,
    #[serde(default)]
    pub ca_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub show_nick_list: bool,
    #[serde(default = "default_true")]
    pub show_status_bar: bool,
    #[serde(default = "default_true")]
    pub timestamps: bool,
    #[serde(default = "default_scrollback")]
    pub scrollback_lines: usize,
    #[serde(default = "default_nick_width")]
    pub nick_list_width: u16,
    #[serde(default = "default_true")]
    pub colors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloodConfig {
    #[serde(default = "default_flood_max")]
    pub max_messages: u32,
    #[serde(default = "default_flood_interval")]
    pub interval_secs: u64,
    #[serde(default = "default_true")]
    pub ctcp_protection: bool,
    #[serde(default = "default_true")]
    pub join_protection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DccConfig {
    #[serde(default)]
    pub download_dir: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub auto_accept: bool,
    #[serde(default = "default_block_size")]
    pub block_size: usize,
    #[serde(default = "default_dcc_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub port_range: Option<(u16, u16)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub directory: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub per_channel: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            nick: default_nick(),
            alt_nicks: vec![],
            username: default_username(),
            realname: default_realname(),
            servers: vec![],
            auto_join: vec![],
            tls: TlsConfig::default(),
            ui: UiConfig::default(),
            flood: FloodConfig::default(),
            dcc: DccConfig::default(),
            logging: LogConfig::default(),
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_certs: true,
            client_cert: None,
            client_key: None,
            ca_file: None,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_nick_list: true,
            show_status_bar: true,
            timestamps: true,
            scrollback_lines: default_scrollback(),
            nick_list_width: default_nick_width(),
            colors: true,
        }
    }
}

impl Default for FloodConfig {
    fn default() -> Self {
        Self {
            max_messages: default_flood_max(),
            interval_secs: default_flood_interval(),
            ctcp_protection: true,
            join_protection: true,
        }
    }
}

impl Default for DccConfig {
    fn default() -> Self {
        Self {
            download_dir: None,
            auto_accept: true,
            block_size: default_block_size(),
            timeout_secs: default_dcc_timeout(),
            port_range: None,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: None,
            per_channel: true,
        }
    }
}

impl Config {
    pub fn load(path: &std::path::Path) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::BitchXError::Config(format!("Failed to read config: {e}"))
        })?;
        toml::from_str(&content)
            .map_err(|e| crate::error::BitchXError::Config(format!("Failed to parse config: {e}")))
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("bitchx")
    }

    pub fn default_config_path() -> PathBuf {
        Self::config_dir().join("bitchx.toml")
    }
}

fn default_nick() -> String {
    whoami().unwrap_or_else(|| "BitchX".into())
}
fn default_username() -> String {
    whoami().unwrap_or_else(|| "bitchx".into())
}
fn default_realname() -> String {
    "BitchX 2.0.0-rs - relay (リレー) release candidate".into()
}
fn default_port() -> u16 {
    6697
}
fn default_true() -> bool {
    true
}
fn default_scrollback() -> usize {
    1000
}
fn default_nick_width() -> u16 {
    16
}
fn default_flood_max() -> u32 {
    5
}
fn default_flood_interval() -> u64 {
    10
}
fn default_block_size() -> usize {
    8192
}
fn default_dcc_timeout() -> u64 {
    120
}

fn whoami() -> Option<String> {
    std::env::var("USER")
        .ok()
        .or_else(|| std::env::var("LOGNAME").ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_complete_toml_config() {
        let toml_str = r##"
            nick = "mybot"
            alt_nicks = ["mybot_", "mybot__"]
            username = "botuser"
            realname = "My Bot"
            auto_join = ["#rust", "#bitchx"]

            [[servers]]
            host = "irc.libera.chat"
            port = 6697
            tls = true
            password = "secret"
            sasl_user = "mybot"
            sasl_pass = "saslpass"

            [[servers]]
            host = "irc.efnet.org"
            port = 6667
            tls = false

            [tls]
            verify_certs = true
            client_cert = "/path/to/cert.pem"
            client_key = "/path/to/key.pem"
            ca_file = "/path/to/ca.pem"

            [ui]
            show_nick_list = false
            show_status_bar = true
            timestamps = false
            scrollback_lines = 5000
            nick_list_width = 20
            colors = true

            [flood]
            max_messages = 10
            interval_secs = 30
            ctcp_protection = false
            join_protection = true

            [dcc]
            download_dir = "/tmp/downloads"
            auto_accept = false
            block_size = 4096
            timeout_secs = 60
            port_range = [1024, 65535]

            [logging]
            enabled = true
            directory = "/var/log/irc"
            per_channel = false
        "##;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.nick, "mybot");
        assert_eq!(config.alt_nicks, vec!["mybot_", "mybot__"]);
        assert_eq!(config.username, "botuser");
        assert_eq!(config.realname, "My Bot");
        assert_eq!(config.auto_join, vec!["#rust", "#bitchx"]);
        assert_eq!(config.servers.len(), 2);
        assert_eq!(config.servers[0].host, "irc.libera.chat");
        assert_eq!(config.servers[0].port, 6697);
        assert!(config.servers[0].tls);
        assert_eq!(config.servers[0].password.as_deref(), Some("secret"));
        assert_eq!(config.servers[0].sasl_user.as_deref(), Some("mybot"));
        assert_eq!(config.servers[1].host, "irc.efnet.org");
        assert_eq!(config.servers[1].port, 6667);
        assert!(!config.servers[1].tls);
        assert!(config.servers[1].password.is_none());
        assert!(config.tls.verify_certs);
        assert_eq!(
            config.tls.client_cert.as_deref(),
            Some(std::path::Path::new("/path/to/cert.pem"))
        );
        assert!(!config.ui.show_nick_list);
        assert!(config.ui.show_status_bar);
        assert!(!config.ui.timestamps);
        assert_eq!(config.ui.scrollback_lines, 5000);
        assert_eq!(config.ui.nick_list_width, 20);
        assert_eq!(config.flood.max_messages, 10);
        assert_eq!(config.flood.interval_secs, 30);
        assert!(!config.flood.ctcp_protection);
        assert!(!config.dcc.auto_accept);
        assert_eq!(config.dcc.block_size, 4096);
        assert_eq!(config.dcc.timeout_secs, 60);
        assert_eq!(config.dcc.port_range, Some((1024, 65535)));
        assert!(config.logging.enabled);
        assert_eq!(
            config.logging.directory.as_deref(),
            Some(std::path::Path::new("/var/log/irc"))
        );
        assert!(!config.logging.per_channel);
    }

    #[test]
    fn parse_minimal_config_with_defaults() {
        let toml_str = "";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.nick, default_nick());
        assert!(config.alt_nicks.is_empty());
        assert_eq!(config.username, default_username());
        assert_eq!(config.realname, "BitchX 2.0.0-rs - relay (リレー) release candidate");
        assert!(config.servers.is_empty());
        assert!(config.auto_join.is_empty());
    }

    #[test]
    fn parse_config_with_tls_settings() {
        let toml_str = r##"
            [tls]
            verify_certs = false
            client_cert = "/my/cert.pem"
        "##;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.tls.verify_certs);
        assert_eq!(
            config.tls.client_cert.as_deref(),
            Some(std::path::Path::new("/my/cert.pem"))
        );
        assert!(config.tls.client_key.is_none());
        assert!(config.tls.ca_file.is_none());
    }

    #[test]
    fn parse_config_with_server_list() {
        let toml_str = r##"
            [[servers]]
            host = "irc.libera.chat"
            port = 6697
            tls = true

            [[servers]]
            host = "irc.efnet.org"
        "##;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.servers.len(), 2);
        assert_eq!(config.servers[0].host, "irc.libera.chat");
        assert_eq!(config.servers[0].port, 6697);
        assert!(config.servers[0].tls);
        assert_eq!(config.servers[1].host, "irc.efnet.org");
        assert_eq!(config.servers[1].port, 6697); // default
        assert!(!config.servers[1].tls); // default false
    }

    #[test]
    fn parse_config_with_flood_settings() {
        let toml_str = r##"
            [flood]
            max_messages = 20
            interval_secs = 5
            ctcp_protection = false
            join_protection = false
        "##;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.flood.max_messages, 20);
        assert_eq!(config.flood.interval_secs, 5);
        assert!(!config.flood.ctcp_protection);
        assert!(!config.flood.join_protection);
    }

    #[test]
    fn default_values_are_correct() {
        let config = Config::default();
        assert_eq!(config.realname, "BitchX 2.0.0-rs - relay (リレー) release candidate");
        assert!(config.servers.is_empty());
        assert!(config.auto_join.is_empty());

        assert!(config.tls.verify_certs);
        assert!(config.tls.client_cert.is_none());
        assert!(config.tls.client_key.is_none());
        assert!(config.tls.ca_file.is_none());

        assert!(config.ui.show_nick_list);
        assert!(config.ui.show_status_bar);
        assert!(config.ui.timestamps);
        assert_eq!(config.ui.scrollback_lines, 1000);
        assert_eq!(config.ui.nick_list_width, 16);
        assert!(config.ui.colors);

        assert_eq!(config.flood.max_messages, 5);
        assert_eq!(config.flood.interval_secs, 10);
        assert!(config.flood.ctcp_protection);
        assert!(config.flood.join_protection);

        assert!(config.dcc.download_dir.is_none());
        assert!(config.dcc.auto_accept);
        assert_eq!(config.dcc.block_size, 8192);
        assert_eq!(config.dcc.timeout_secs, 120);
        assert!(config.dcc.port_range.is_none());

        assert!(!config.logging.enabled);
        assert!(config.logging.directory.is_none());
        assert!(config.logging.per_channel);
    }

    #[test]
    fn config_dir_returns_valid_path() {
        let dir = Config::config_dir();
        assert!(dir.ends_with("bitchx"));
    }

    #[test]
    fn default_config_path_ends_with_toml() {
        let path = Config::default_config_path();
        assert!(path.ends_with("bitchx.toml"));
        assert!(path.starts_with(Config::config_dir()));
    }

    #[test]
    fn server_config_deserialization() {
        let toml_str = r##"
            host = "irc.libera.chat"
            port = 6697
            tls = true
            password = "secret"
            sasl_user = "user"
            sasl_pass = "pass"
        "##;

        let server: ServerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(server.host, "irc.libera.chat");
        assert_eq!(server.port, 6697);
        assert!(server.tls);
        assert_eq!(server.password.as_deref(), Some("secret"));
        assert_eq!(server.sasl_user.as_deref(), Some("user"));
        assert_eq!(server.sasl_pass.as_deref(), Some("pass"));
    }

    #[test]
    fn server_config_defaults() {
        let toml_str = r#"host = "irc.example.com""#;
        let server: ServerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(server.host, "irc.example.com");
        assert_eq!(server.port, 6697); // default_port
        assert!(!server.tls);
        assert!(server.password.is_none());
        assert!(server.sasl_user.is_none());
        assert!(server.sasl_pass.is_none());
    }

    #[test]
    fn config_load_nonexistent_file_returns_error() {
        let result = Config::load(std::path::Path::new("/nonexistent/path/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn config_load_from_temp_file() {
        let dir = std::env::temp_dir().join("bitchx_test_config");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");
        std::fs::write(&path, r#"nick = "testbot""#).unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.nick, "testbot");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn config_load_invalid_toml_returns_error() {
        let dir = std::env::temp_dir().join("bitchx_test_bad_config");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.toml");
        std::fs::write(&path, "this is not valid toml [[[").unwrap();

        let result = Config::load(&path);
        assert!(result.is_err());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tls_config_default() {
        let tls = TlsConfig::default();
        assert!(tls.verify_certs);
        assert!(tls.client_cert.is_none());
        assert!(tls.client_key.is_none());
        assert!(tls.ca_file.is_none());
    }

    #[test]
    fn ui_config_default() {
        let ui = UiConfig::default();
        assert!(ui.show_nick_list);
        assert!(ui.show_status_bar);
        assert!(ui.timestamps);
        assert_eq!(ui.scrollback_lines, 1000);
        assert_eq!(ui.nick_list_width, 16);
        assert!(ui.colors);
    }

    #[test]
    fn flood_config_default() {
        let flood = FloodConfig::default();
        assert_eq!(flood.max_messages, 5);
        assert_eq!(flood.interval_secs, 10);
        assert!(flood.ctcp_protection);
        assert!(flood.join_protection);
    }

    #[test]
    fn dcc_config_default() {
        let dcc = DccConfig::default();
        assert!(dcc.download_dir.is_none());
        assert!(dcc.auto_accept);
        assert_eq!(dcc.block_size, 8192);
        assert_eq!(dcc.timeout_secs, 120);
        assert!(dcc.port_range.is_none());
    }

    #[test]
    fn log_config_default() {
        let log = LogConfig::default();
        assert!(!log.enabled);
        assert!(log.directory.is_none());
        assert!(log.per_channel);
    }

    #[test]
    fn config_serialization_round_trip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.nick, config.nick);
        assert_eq!(parsed.username, config.username);
        assert_eq!(parsed.realname, config.realname);
        assert_eq!(parsed.ui.scrollback_lines, config.ui.scrollback_lines);
    }
}
