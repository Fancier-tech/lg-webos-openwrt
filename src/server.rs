use crate::{wol, Config, LgtvctlError, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

type ApiResult = (StatusCode, Json<Value>);

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[derive(Debug, Deserialize)]
struct VolumeSetQuery {
    value: u8,
}

pub async fn serve(config: Config) -> Result<()> {
    let listen = config.http_listen.clone();
    let state = AppState {
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/tv/on", get(tv_on).post(tv_on))
        .route("/tv/off", get(tv_off).post(tv_off))
        .route("/tv/status", get(tv_status).post(tv_status))
        .route("/tv/volume/up", get(volume_up).post(volume_up))
        .route("/tv/volume/down", get(volume_down).post(volume_down))
        .route("/tv/volume/set", get(volume_set_query).post(volume_set_query))
        .route("/tv/volume/set/{level}", get(volume_set_path).post(volume_set_path))
        .route("/tv/mute", get(mute_toggle).post(mute_toggle))
        .route("/tv/mute/on", get(mute_on).post(mute_on))
        .route("/tv/mute/off", get(mute_off).post(mute_off))
        .with_state(state);

    let listener = TcpListener::bind(&listen).await?;
    info!(listen, "HTTP API listening");
    println!("http_listen={listen}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> ApiResult {
    api_ok(json!({
        "ok": true,
        "service": "lgtvctl",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn tv_on(State(state): State<AppState>) -> ApiResult {
    match wol::wake(&state.config).await {
        Ok(result) => api_ok(json!({
            "ok": true,
            "command": "on",
            "mac": result.mac,
            "target": result.target.to_string()
        })),
        Err(error) => api_error(error),
    }
}

async fn tv_off(State(state): State<AppState>) -> ApiResult {
    webos_request(state, "off", "ssap://system/turnOff", None).await
}

async fn tv_status(State(state): State<AppState>) -> ApiResult {
    webos_request(state, "status", "ssap://audio/getVolume", None).await
}

async fn volume_up(State(state): State<AppState>) -> ApiResult {
    webos_request(state, "volume_up", "ssap://audio/volumeUp", None).await
}

async fn volume_down(State(state): State<AppState>) -> ApiResult {
    webos_request(state, "volume_down", "ssap://audio/volumeDown", None).await
}

async fn volume_set_query(State(state): State<AppState>, Query(query): Query<VolumeSetQuery>) -> ApiResult {
    volume_set_inner(state, query.value).await
}

async fn volume_set_path(State(state): State<AppState>, Path(level): Path<u8>) -> ApiResult {
    volume_set_inner(state, level).await
}

async fn volume_set_inner(state: AppState, level: u8) -> ApiResult {
    if level > 100 {
        return api_bad_request("volume must be between 0 and 100");
    }

    webos_request(
        state,
        "volume_set",
        "ssap://audio/setVolume",
        Some(json!({ "volume": level })),
    )
    .await
}

async fn mute_toggle(State(state): State<AppState>) -> ApiResult {
    let client = crate::webos::WebOsClient::new((*state.config).clone());
    match client.request("mute_status", "ssap://audio/getVolume", None).await {
        Ok(status) => {
            let current = status
                .get("mute")
                .and_then(Value::as_bool)
                .or_else(|| status.get("muted").and_then(Value::as_bool))
                .unwrap_or(false);
            match client
                .request(
                    "mute_toggle",
                    "ssap://audio/setMute",
                    Some(json!({ "mute": !current })),
                )
                .await
            {
                Ok(payload) => api_ok(json!({
                    "ok": true,
                    "command": "mute_toggle",
                    "mute": !current,
                    "payload": payload
                })),
                Err(error) => api_error(error),
            }
        }
        Err(error) => api_error(error),
    }
}

async fn mute_on(State(state): State<AppState>) -> ApiResult {
    webos_request(
        state,
        "mute_on",
        "ssap://audio/setMute",
        Some(json!({ "mute": true })),
    )
    .await
}

async fn mute_off(State(state): State<AppState>) -> ApiResult {
    webos_request(
        state,
        "mute_off",
        "ssap://audio/setMute",
        Some(json!({ "mute": false })),
    )
    .await
}

async fn webos_request(
    state: AppState,
    command: &'static str,
    uri: &'static str,
    payload: Option<Value>,
) -> ApiResult {
    let client = crate::webos::WebOsClient::new((*state.config).clone());
    match client.request(command, uri, payload).await {
        Ok(payload) => api_ok(json!({
            "ok": true,
            "command": command,
            "payload": payload
        })),
        Err(error) => api_error(error),
    }
}

fn api_ok(value: Value) -> ApiResult {
    (StatusCode::OK, Json(value))
}

fn api_bad_request(message: &str) -> ApiResult {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "ok": false,
            "error": message
        })),
    )
}

fn api_error(error: LgtvctlError) -> ApiResult {
    let status = match &error {
        LgtvctlError::MissingHost
        | LgtvctlError::MissingClientKeyConfig
        | LgtvctlError::MissingMac
        | LgtvctlError::InvalidMac(_) => StatusCode::BAD_REQUEST,
        LgtvctlError::Timeout { .. } => StatusCode::GATEWAY_TIMEOUT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status,
        Json(json!({
            "ok": false,
            "error": error.to_string()
        })),
    )
}
