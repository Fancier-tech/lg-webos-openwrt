use crate::{Config, LgtvctlError, Result};
use rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::http::StatusCode,
    Connector, MaybeTlsStream, WebSocketStream,
};

pub struct WebOsClient {
    config: Config,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub url: String,
    pub http_status: StatusCode,
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
