use crate::error::{LgtvctlError, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
};

const DEFAULT_PORT: u16 = 3001;
const DEFAULT_TIMEOUT_MS: u64 = 3000;
const DEFAULT_PAIR_TIMEOUT_MS: u64 = 60000;

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
    /// Keep false for typical local LG webOS use.
    #[serde(default)]
    pub verify_certificate: bool,

    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Timeout for interactive TV pairing prompt.
    #[serde(default = "default_pair_timeout_ms")]
    pub pair_timeout_ms: u64,

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
            pair_timeout_ms: DEFAULT_PAIR_TIMEOUT_MS,
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

    pub fn apply_overrides(
        &mut self,
        host: Option<String>,
        port: Option<u16>,
        mac: Option<String>,
        wol_broadcast: Option<String>,
    ) {
        if let Some(host) = host {
            self.host = Some(host);
        }
        if let Some(port) = port {
            self.port = port;
        }
        if let Some(mac) = mac {
            self.mac = Some(mac);
        }
        if let Some(wol_broadcast) = wol_broadcast {
            self.wol_broadcast = Some(wol_broadcast);
        }
    }

    pub fn require_host(&self) -> Result<&str> {
        self.host.as_deref().ok_or(LgtvctlError::MissingHost)
    }

    pub fn require_client_key(&self) -> Result<&str> {
        self.client_key
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or(LgtvctlError::MissingClientKeyConfig)
    }

    pub fn require_mac(&self) -> Result<&str> {
        self.mac
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or(LgtvctlError::MissingMac)
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    pub fn pair_timeout(&self) -> Duration {
        Duration::from_millis(self.pair_timeout_ms)
    }

    pub fn save_with_client_key(&self, cli_config_path: Option<&Path>, client_key: String) -> Result<PathBuf> {
        let path = config_save_path(cli_config_path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|source| LgtvctlError::WriteConfig {
                    path: path.clone(),
                    source,
                })?;
            }
        }

        let mut next = self.clone();
        next.client_key = Some(client_key);
        let raw = toml::to_string_pretty(&next)?;
        std::fs::write(&path, raw).map_err(|source| LgtvctlError::WriteConfig {
            path: path.clone(),
            source,
        })?;

        Ok(path)
    }
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

fn default_pair_timeout_ms() -> u64 {
    DEFAULT_PAIR_TIMEOUT_MS
}

fn default_config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("LGTVCTL_CONFIG") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    config_candidates().into_iter().find(|path| path.exists())
}

fn config_save_path(cli_config_path: Option<&Path>) -> PathBuf {
    if let Some(path) = cli_config_path {
        return path.to_path_buf();
    }

    if let Ok(path) = env::var("LGTVCTL_CONFIG") {
        return PathBuf::from(path);
    }

    if let Some(existing) = config_candidates().into_iter().find(|path| path.exists()) {
        return existing;
    }

    PathBuf::from("./lgtvctl.toml")
}

fn config_candidates() -> [PathBuf; 3] {
    [
        PathBuf::from("./lgtvctl.toml"),
        PathBuf::from("./config/lgtvctl.toml"),
        PathBuf::from("/etc/lgtvctl.toml"),
    ]
}
