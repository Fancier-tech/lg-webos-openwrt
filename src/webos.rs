use crate::{Config, LgtvctlError, Result};
use futures_util::{SinkExt, StreamExt};
use rustls::{ClientConfig, RootCertStore};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{http::StatusCode, Message},
    Connector, MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;

pub struct WebOsClient {
    config: Config,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub url: String,
    pub http_status: StatusCode,
}

#[derive(Debug, Clone)]
pub struct PairResult {
    pub client_key: String,
}

pub struct WebOsConnection {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebOsClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn probe(&self) -> Result<ProbeResult> {
        let (connection, result) = self.connect().await?;
        connection.close().await?;
        Ok(result)
    }

    pub async fn pair(&self) -> Result<PairResult> {
        let (mut connection, _result) = self.connect().await?;
        let id = format!("register_{}", Uuid::new_v4());

        connection
            .send_json(&json!({
                "id": id,
                "type": "register",
                "payload": {
                    "forcePairing": false,
                    "pairingType": "PROMPT",
                    "manifest": webos_manifest()
                }
            }))
            .await?;

        let response = timeout(self.config.pair_timeout(), async {
            loop {
                let message = connection.next_json().await?;
                if message.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                    continue;
                }

                match message.get("type").and_then(Value::as_str) {
                    Some("registered") => {
                        let client_key = message
                            .get("payload")
                            .and_then(|payload| payload.get("client-key"))
                            .and_then(Value::as_str)
                            .ok_or(LgtvctlError::MissingClientKey)?;

                        return Ok(PairResult {
                            client_key: client_key.to_string(),
                        });
                    }
                    Some("error") => {
                        return Err(LgtvctlError::Protocol(message.to_string()));
                    }
                    _ => {}
                }
            }
        })
        .await
        .map_err(|_| LgtvctlError::Timeout {
            operation: "pair",
            timeout_ms: self.config.pair_timeout_ms,
        })??;

        connection.close().await?;
        Ok(response)
    }

    pub async fn request(&self, command: &'static str, uri: &str, payload: Option<Value>) -> Result<Value> {
        let (mut connection, _result) = self.connect().await?;
        self.register_with_client_key(&mut connection).await?;

        let id = format!("{command}_{}", Uuid::new_v4());
        let mut request = json!({
            "id": id,
            "type": "request",
            "uri": uri,
        });

        if let Some(payload) = payload {
            request["payload"] = payload;
        }

        connection.send_json(&request).await?;
        let response = self.wait_for_response(&mut connection, &id, command).await?;
        connection.close().await?;
        Ok(response)
    }

    async fn register_with_client_key(&self, connection: &mut WebOsConnection) -> Result<()> {
        let client_key = self.config.require_client_key()?;
        let id = format!("register_{}", Uuid::new_v4());

        connection
            .send_json(&json!({
                "id": id,
                "type": "register",
                "payload": {
                    "client-key": client_key,
                    "manifest": webos_manifest()
                }
            }))
            .await?;

        timeout(self.config.timeout(), async {
            loop {
                let message = connection.next_json().await?;
                if message.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                    continue;
                }

                match message.get("type").and_then(Value::as_str) {
                    Some("registered") => return Ok(()),
                    Some("error") => return Err(LgtvctlError::Protocol(message.to_string())),
                    _ => {}
                }
            }
        })
        .await
        .map_err(|_| LgtvctlError::Timeout {
            operation: "auth",
            timeout_ms: self.config.timeout_ms,
        })?
    }

    async fn wait_for_response(
        &self,
        connection: &mut WebOsConnection,
        id: &str,
        command: &'static str,
    ) -> Result<Value> {
        timeout(self.config.timeout(), async {
            loop {
                let message = connection.next_json().await?;
                if message.get("id").and_then(Value::as_str) != Some(id) {
                    continue;
                }

                match message.get("type").and_then(Value::as_str) {
                    Some("response") => {
                        return Ok(message.get("payload").cloned().unwrap_or(Value::Null));
                    }
                    Some("error") => return Err(LgtvctlError::Protocol(message.to_string())),
                    _ => {}
                }
            }
        })
        .await
        .map_err(|_| LgtvctlError::Timeout {
            operation: command,
            timeout_ms: self.config.timeout_ms,
        })?
    }

    pub async fn connect(&self) -> Result<(WebOsConnection, ProbeResult)> {
        let url = self.url()?;
        let connector = build_tls_connector(self.config.verify_certificate);
        let connect_future = connect_async_tls_with_config(
            url.as_str(),
            None,
            false,
            Some(connector),
        );

        let (stream, response) = timeout(self.config.timeout(), connect_future)
            .await
            .map_err(|_| LgtvctlError::Timeout {
                operation: "connect",
                timeout_ms: self.config.timeout_ms,
            })??;

        Ok((
            WebOsConnection { stream },
            ProbeResult {
                url,
                http_status: response.status(),
            },
        ))
    }

    fn url(&self) -> Result<String> {
        let host = self.config.require_host()?;
        Ok(format!("wss://{host}:{}/", self.config.port))
    }
}

impl WebOsConnection {
    pub async fn close(mut self) -> Result<()> {
        self.stream.close(None).await?;
        Ok(())
    }

    async fn send_json(&mut self, value: &Value) -> Result<()> {
        self.stream
            .send(Message::Text(value.to_string().into()))
            .await?;
        Ok(())
    }

    async fn next_json(&mut self) -> Result<Value> {
        loop {
            let Some(message) = self.stream.next().await else {
                return Err(LgtvctlError::Protocol("websocket closed by TV".to_string()));
            };

            match message? {
                Message::Text(text) => {
                    return serde_json::from_str(&text)
                        .map_err(|source| LgtvctlError::Protocol(format!("invalid JSON from TV: {source}; raw={text}")));
                }
                Message::Binary(bytes) => {
                    return serde_json::from_slice(&bytes)
                        .map_err(|source| LgtvctlError::Protocol(format!("invalid binary JSON from TV: {source}")));
                }
                Message::Ping(payload) => {
                    self.stream.send(Message::Pong(payload)).await?;
                }
                Message::Pong(_) => {}
                Message::Close(frame) => {
                    return Err(LgtvctlError::Protocol(format!("websocket closed by TV: {frame:?}")));
                }
                Message::Frame(_) => {}
            }
        }
    }
}

fn webos_manifest() -> Value {
    json!({
        "manifestVersion": 1,
        "appVersion": "0.5.0",
        "appId": "com.fanciertech.lgtvctl",
        "vendorId": "com.fanciertech",
        "permissions": [
            "LAUNCH",
            "LAUNCH_WEBAPP",
            "APP_TO_APP",
            "CONTROL_AUDIO",
            "CONTROL_DISPLAY",
            "CONTROL_INPUT_JOYSTICK",
            "CONTROL_INPUT_MEDIA_PLAYBACK",
            "CONTROL_INPUT_TEXT",
            "CONTROL_INPUT_TV",
            "CONTROL_MOUSE_AND_KEYBOARD",
            "CONTROL_POWER",
            "READ_APP_STATUS",
            "READ_CURRENT_CHANNEL",
            "READ_INPUT_DEVICE_LIST",
            "READ_INSTALLED_APPS",
            "READ_LGE_SDX",
            "READ_LGE_TV_INPUT_EVENTS",
            "READ_NOTIFICATIONS",
            "READ_POWER_STATE",
            "READ_RUNNING_APPS",
            "READ_TV_CHANNEL_LIST",
            "READ_TV_CURRENT_TIME",
            "SEARCH",
            "WRITE_NOTIFICATION_TOAST"
        ],
        "localizedAppNames": {
            "": "lgtvctl",
            "en-US": "lgtvctl",
            "ru-RU": "lgtvctl"
        },
        "localizedVendorNames": {
            "": "Fancier Tech"
        }
    })
}

fn build_tls_connector(verify_certificate: bool) -> Connector {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    if !verify_certificate {
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification));
    }

    Connector::Rustls(Arc::new(config))
}

#[derive(Debug)]
struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}
