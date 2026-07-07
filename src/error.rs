use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, LgtvctlError>;

#[derive(Debug, thiserror::Error)]
pub enum LgtvctlError {
    #[error("failed to read config file {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config file {path}: {source}")]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("TV host is not configured. Use --host or set host in config file")]
    MissingHost,

    #[error("operation timed out: {operation} after {timeout_ms} ms")]
    Timeout {
        operation: &'static str,
        timeout_ms: u64,
    },

    #[error("WebSocket/TLS error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("command is not implemented yet in stage 2: {0}")]
    NotImplemented(&'static str),
}
