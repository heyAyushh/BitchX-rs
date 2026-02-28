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
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::BitchXError::Config(format!("Failed to read config: {e}")))?;
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

fn default_nick() -> String { whoami().unwrap_or_else(|| "BitchX".into()) }
fn default_username() -> String { whoami().unwrap_or_else(|| "bitchx".into()) }
fn default_realname() -> String { "BitchX 2.0 - Rust Rewrite".into() }
fn default_port() -> u16 { 6697 }
fn default_true() -> bool { true }
fn default_scrollback() -> usize { 1000 }
fn default_nick_width() -> u16 { 16 }
fn default_flood_max() -> u32 { 5 }
fn default_flood_interval() -> u64 { 10 }
fn default_block_size() -> usize { 8192 }
fn default_dcc_timeout() -> u64 { 120 }

fn whoami() -> Option<String> {
    std::env::var("USER").ok().or_else(|| std::env::var("LOGNAME").ok())
}
