use clap::Parser;
use lgtvctl::{
    cli::{Cli, Command, MuteState, VolumeAction},
    server, webos::WebOsClient,
    wol, Config, LgtvctlError, Result,
};
use serde_json::Value;
use std::path::PathBuf;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    let cli = Cli::parse();
    let cli_config_path = cli.config.clone();
    let mut config = Config::load(cli.config.as_deref())?;
    config.apply_overrides(cli.host, cli.port, cli.mac, cli.wol_broadcast, cli.listen);

    debug!(?config, "configuration loaded");

    if cli.dry_run {
        print_dry_run(&cli.command, &config)?;
        return Ok(());
    }

    run(cli.command, config, cli_config_path).await?;
    Ok(())
}

async fn run(command: Command, config: Config, cli_config_path: Option<PathBuf>) -> Result<()> {
    match command {
        Command::Config => {
            print_config(&config);
            Ok(())
        }
        Command::Pair => pair(config, cli_config_path).await,
        Command::On => on(config).await,
        Command::Off => authenticated_command(config, "off", "ssap://system/turnOff", None).await,
        Command::Probe => probe(config).await,
        Command::Status => status(config).await,
        Command::Volume(args) => volume(config, args.action).await,
        Command::Mute(args) => mute(config, args.state).await,
        Command::Hdmi { .. } => not_implemented("hdmi"),
        Command::App { .. } => not_implemented("app"),
        Command::Key { .. } => not_implemented("key"),
        Command::Serve => server::serve(config).await,
    }
}

fn print_dry_run(command: &Command, config: &Config) -> Result<()> {
    match command {
        Command::On => {
            let mac = config.mac.as_deref().unwrap_or("<not set>");
            let broadcast = config.wol_broadcast.as_deref().unwrap_or("255.255.255.255");
            println!("dry-run: mac={mac} wol_broadcast={broadcast} command=on");
        }
        Command::Serve => {
            println!("dry-run: listen={} command=serve", config.http_listen);
        }
        _ => {
            let host = config.require_host()?;
            println!(
                "dry-run: host={host} port={} command={}",
                config.port,
                command_name(command)
            );
        }
    }
    Ok(())
}

fn print_config(config: &Config) {
    println!("host = {}", config.host.as_deref().unwrap_or("<not set>"));
    println!("port = {}", config.port);
    println!(
        "client_key = {}",
        if config.client_key.as_deref().unwrap_or_default().is_empty() {
            "<empty>"
        } else {
            "<set>"
        }
    );
    println!("verify_certificate = {}", config.verify_certificate);
    println!("timeout_ms = {}", config.timeout_ms);
    println!("pair_timeout_ms = {}", config.pair_timeout_ms);
    println!("mac = {}", config.mac.as_deref().unwrap_or("<not set>"));
    println!(
        "wol_broadcast = {}",
        config.wol_broadcast.as_deref().unwrap_or("<not set>")
    );
    println!("http_listen = {}", config.http_listen);
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
        Command::Serve => "serve".to_string(),
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

async fn pair(config: Config, cli_config_path: Option<PathBuf>) -> Result<()> {
    println!("Pairing request sent. Confirm the prompt on the TV screen...");
    let result = WebOsClient::new(config.clone()).pair().await?;
    let saved_path = config.save_with_client_key(cli_config_path.as_deref(), result.client_key.clone())?;

    println!("paired: client_key=<set>");
    println!("saved_config={}", saved_path.display());
    Ok(())
}

async fn on(config: Config) -> Result<()> {
    let result = wol::wake(&config).await?;
    println!("wol_sent: mac={} target={}", result.mac, result.target);
    Ok(())
}

async fn status(config: Config) -> Result<()> {
    let response = WebOsClient::new(config)
        .request("status", "ssap://audio/getVolume", None)
        .await?;
    print_response("status", &response);
    Ok(())
}

async fn volume(config: Config, action: VolumeAction) -> Result<()> {
    match action {
        VolumeAction::Up => authenticated_command(config, "volume_up", "ssap://audio/volumeUp", None).await,
        VolumeAction::Down => authenticated_command(config, "volume_down", "ssap://audio/volumeDown", None).await,
        VolumeAction::Set { level } => {
            authenticated_command(
                config,
                "volume_set",
                "ssap://audio/setVolume",
                Some(serde_json::json!({ "volume": level })),
            )
            .await
        }
    }
}

async fn mute(config: Config, state: MuteState) -> Result<()> {
    match state {
        MuteState::On => authenticated_command(
            config,
            "mute_on",
            "ssap://audio/setMute",
            Some(serde_json::json!({ "mute": true })),
        )
        .await,
        MuteState::Off => authenticated_command(
            config,
            "mute_off",
            "ssap://audio/setMute",
            Some(serde_json::json!({ "mute": false })),
        )
        .await,
        MuteState::Toggle => {
            let client = WebOsClient::new(config);
            let status = client
                .request("mute_status", "ssap://audio/getVolume", None)
                .await?;
            let current = status
                .get("mute")
                .and_then(Value::as_bool)
                .or_else(|| status.get("muted").and_then(Value::as_bool))
                .unwrap_or(false);
            let response = client
                .request(
                    "mute_toggle",
                    "ssap://audio/setMute",
                    Some(serde_json::json!({ "mute": !current })),
                )
                .await?;
            print_response("mute_toggle", &response);
            Ok(())
        }
    }
}

async fn authenticated_command(
    config: Config,
    name: &'static str,
    uri: &'static str,
    payload: Option<Value>,
) -> Result<()> {
    let response = WebOsClient::new(config).request(name, uri, payload).await?;
    print_response(name, &response);
    Ok(())
}

fn print_response(command: &str, response: &Value) {
    if let Some(return_value) = response.get("returnValue").and_then(Value::as_bool) {
        println!("{command}: returnValue={return_value}");
    } else {
        println!("{command}: {response}");
    }
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
