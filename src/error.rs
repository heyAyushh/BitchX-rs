use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitchYError {
    #[error("IRC protocol error: {0}")]
    Protocol(String),

    #[error("Connection error: {0}")]
    Connection(#[from] std::io::Error),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("DCC error: {0}")]
    Dcc(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BitchYError>;
