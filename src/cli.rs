use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "lgtvctl")]
#[command(version, about = "Control LG webOS TV from CLI/OpenWrt")]
pub struct Cli {
    /// Path to config file. If omitted, lgtvctl checks LGTVCTL_CONFIG, ./lgtvctl.toml,
    /// ./config/lgtvctl.toml and /etc/lgtvctl.toml.
    #[arg(short, long, env = "LGTVCTL_CONFIG")]
    pub config: Option<PathBuf>,

    /// TV host/IP override.
    #[arg(long, env = "LGTVCTL_HOST")]
    pub host: Option<String>,

    /// TV WebSocket port override. LG webOS secure port is usually 3001.
    #[arg(long, env = "LGTVCTL_PORT")]
    pub port: Option<u16>,

    /// TV MAC address override for Wake-on-LAN, e.g. AA:BB:CC:DD:EE:FF.
    #[arg(long, env = "LGTVCTL_MAC")]
    pub mac: Option<String>,

    /// Wake-on-LAN broadcast address override, e.g. 192.168.0.255.
    #[arg(long, env = "LGTVCTL_WOL_BROADCAST")]
    pub wol_broadcast: Option<String>,

    /// HTTP API bind address for `serve`, e.g. 127.0.0.1:8765 or 0.0.0.0:8765.
    #[arg(long, env = "LGTVCTL_HTTP_LISTEN")]
    pub listen: Option<String>,

    /// Print what would be executed without connecting to TV.
    #[arg(long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Pair this client with the TV and save a client key.
    Pair,

    /// Wake TV using Wake-on-LAN. Requires mac in config or --mac.
    On,

    /// Turn TV off.
    Off,

    /// Open WSS connection to TV and close it. Useful before pairing.
    Probe,

    /// Read basic TV status.
    Status,

    /// Change volume.
    Volume(VolumeArgs),

    /// Toggle or set mute.
    Mute(MuteArgs),

    /// Switch to HDMI input number. Implemented in a later stage.
    Hdmi { number: u8 },

    /// Launch app by common alias or app id, e.g. youtube, kodi, netflix. Implemented in a later stage.
    App { name: String },

    /// Send remote-control button/key, e.g. HOME, BACK, UP, DOWN, ENTER. Implemented in a later stage.
    Key { key: String },

    /// Run local HTTP API service for Alice/OpenWrt automations.
    Serve,

    /// Print resolved configuration without sensitive values.
    Config,
}

#[derive(Debug, Args)]
pub struct VolumeArgs {
    #[command(subcommand)]
    pub action: VolumeAction,
}

#[derive(Debug, Subcommand)]
pub enum VolumeAction {
    Up,
    Down,
    Set { level: u8 },
}

#[derive(Debug, Args)]
pub struct MuteArgs {
    #[arg(value_enum, default_value = "toggle")]
    pub state: MuteState,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MuteState {
    Toggle,
    On,
    Off,
}
