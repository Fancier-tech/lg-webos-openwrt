use crate::error::{LgtvctlError, Result};
use serde::{Deserialize, Serialize};
use std::{env, path::{Path, PathBuf}, time::Duration};

const DEFAULT_PORT: u16 = 3001;
const DEFAULT_TIMEOUT_MS: u64 = 3000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// IP address or DNS name of LG webOS TV.
    pub host: Option<String>,

    /// LG webOS secure WebSocket port. Usually 3001.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Client key returned by TV after pairing.
    /// Empty/null means the tool is not paired yet.
    #[serde(default)]
    pub client_key: Option<String>,

    /// LG TVs normally use a certificate chain that small embedded systems may not validate.
    /// Stage 2 will use this flag when WebSocket transport is implemented.
    #[serde(default)]
    pub verify_certificate: bool,

    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Optional MAC address for Wake-on-LAN, e.g. AA:BB:CC:DD:EE:FF.
    #[serde(default)]
    pub mac: Option<String>,

    /// Optional broadcast address for Wake-on-LAN, e.g. 192.168.0.255.
    #[serde(default)]
    pub wol_broadcast: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: None,
            port: DEFAULT_PORT,
            client_key: None,
            verify_certificate: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            mac: None,
            wol_broadcast: None,
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path.map(PathBuf::from).or_else(default_config_path) else {
            return Ok(Self::default());
        };

        let raw = std::fs::read_to_string(&path).map_err(|source| LgtvctlError::ReadConfig {
            path: path.clone(),
            source,
        })?;

        toml::from_str(&raw).map_err(|source| LgtvctlError::ParseConfig { path, source })
    }

    pub fn apply_overrides(&mut self, host: Option<String>, port: Option<u16>) {
        if let Some(host) = host {
            self.host = Some(host);
        }
        if let Some(port) = port {
            self.port = port;
        }
    }

    pub fn require_host(&self) -> Result<&str> {
        self.host.as_deref().ok_or(LgtvctlError::MissingHost)
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

fn default_config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("LGTVCTL_CONFIG") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    let candidates = [
        PathBuf::from("./lgtvctl.toml"),
        PathBuf::from("./config/lgtvctl.toml"),
        PathBuf::from("/etc/lgtvctl.toml"),
    ];

    candidates.into_iter().find(|path| path.exists())
}
