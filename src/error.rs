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

    #[error("command is not implemented yet in stage 1: {0}")]
    NotImplemented(&'static str),
}
