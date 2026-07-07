use clap::Parser;
use lgtvctl::{cli::{Cli, Command, MuteState, VolumeAction}, webos::WebOsClient, Config, LgtvctlError, Result};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let cli = Cli::parse();
    let mut config = Config::load(cli.config.as_deref())?;
    config.apply_overrides(cli.host, cli.port);

    debug!(?config, "configuration loaded");

    if cli.dry_run {
        print_dry_run(&cli.command, &config)?;
        return Ok(());
    }

    run(cli.command, config).await?;
    Ok(())
}

async fn run(command: Command, config: Config) -> Result<()> {
    match command {
        Command::Config => {
            print_config(&config);
            Ok(())
        }
        Command::Pair => not_implemented("pair"),
        Command::On => not_implemented("on"),
        Command::Off => not_implemented("off"),
        Command::Probe => probe(config).await,
        Command::Status => not_implemented("status"),
        Command::Volume(_) => not_implemented("volume"),
        Command::Mute(_) => not_implemented("mute"),
        Command::Hdmi { .. } => not_implemented("hdmi"),
        Command::App { .. } => not_implemented("app"),
        Command::Key { .. } => not_implemented("key"),
    }
}

fn print_dry_run(command: &Command, config: &Config) -> Result<()> {
    let host = config.require_host()?;
    println!("dry-run: host={host} port={} command={}", config.port, command_name(command));
    Ok(())
}

fn print_config(config: &Config) {
    println!("host = {}", config.host.as_deref().unwrap_or("<not set>"));
    println!("port = {}", config.port);
    println!("client_key = {}", if config.client_key.as_deref().unwrap_or_default().is_empty() { "<empty>" } else { "<set>" });
    println!("verify_certificate = {}", config.verify_certificate);
    println!("timeout_ms = {}", config.timeout_ms);
    println!("mac = {}", config.mac.as_deref().unwrap_or("<not set>"));
    println!("wol_broadcast = {}", config.wol_broadcast.as_deref().unwrap_or("<not set>"));
}

fn command_name(command: &Command) -> String {
    match command {
        Command::Pair => "pair".to_string(),
        Command::On => "on".to_string(),
        Command::Off => "off".to_string(),
        Command::Probe => "probe".to_string(),
        Command::Status => "status".to_string(),
        Command::Config => "config".to_string(),
        Command::Volume(args) => match &args.action {
            VolumeAction::Up => "volume up".to_string(),
            VolumeAction::Down => "volume down".to_string(),
            VolumeAction::Set { level } => format!("volume set {level}"),
        },
        Command::Mute(args) => match args.state {
            MuteState::Toggle => "mute toggle".to_string(),
            MuteState::On => "mute on".to_string(),
            MuteState::Off => "mute off".to_string(),
        },
        Command::Hdmi { number } => format!("hdmi {number}"),
        Command::App { name } => format!("app {name}"),
        Command::Key { key } => format!("key {key}"),
    }
}

async fn probe(config: Config) -> Result<()> {
    let result = WebOsClient::new(config).probe().await?;
    println!(
        "connected: {} http_status={} {}",
        result.url,
        result.http_status.as_u16(),
        result.http_status.canonical_reason().unwrap_or("")
    );
    Ok(())
}

fn not_implemented(command: &'static str) -> Result<()> {
    info!(command, "command placeholder reached");
    Err(LgtvctlError::NotImplemented(command))
}

fn init_logging() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "lgtvctl=info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .init();
}
