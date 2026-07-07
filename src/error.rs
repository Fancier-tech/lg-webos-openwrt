use std::{net::AddrParseError, path::PathBuf};

pub type Result<T> = std::result::Result<T, LgtvctlError>;

#[derive(Debug, thiserror::Error)]
pub enum LgtvctlError {
    #[error("failed to read config file {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write config file {path}: {source}")]
    WriteConfig {
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

    #[error("failed to serialize config: {0}")]
    SerializeConfig(#[from] toml::ser::Error),

    #[error("TV host is not configured. Use --host or set host in config file")]
    MissingHost,

    #[error("client_key is not configured. Run `lgtvctl pair` first or set client_key in config file")]
    MissingClientKeyConfig,

    #[error("TV MAC address is not configured. Use --mac or set mac in config file")]
    MissingMac,

    #[error("invalid MAC address `{0}`. Expected AA:BB:CC:DD:EE:FF, AA-BB-CC-DD-EE-FF or AABBCCDDEEFF")]
    InvalidMac(String),

    #[error("invalid socket address: {0}")]
    InvalidSocketAddress(#[from] AddrParseError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("operation timed out: {operation} after {timeout_ms} ms")]
    Timeout {
        operation: &'static str,
        timeout_ms: u64,
    },

    #[error("WebSocket/TLS error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("webOS protocol error: {0}")]
    Protocol(String),

    #[error("TV accepted registration but did not return client-key")]
    MissingClientKey,

    #[error("command is not implemented yet in stage 4: {0}")]
    NotImplemented(&'static str),
}
