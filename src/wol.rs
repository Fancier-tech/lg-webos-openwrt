use crate::{Config, LgtvctlError, Result};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

const DEFAULT_WOL_BROADCAST: &str = "255.255.255.255";
const DEFAULT_WOL_PORT: u16 = 9;

#[derive(Debug, Clone)]
pub struct WolResult {
    pub mac: String,
    pub target: SocketAddr,
}

pub async fn wake(config: &Config) -> Result<WolResult> {
    let mac_raw = config.require_mac()?;
    let mac = parse_mac(mac_raw)?;
    let packet = magic_packet(mac);
    let target = wol_target(config.wol_broadcast.as_deref())?;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    socket.send_to(&packet, target).await?;

    Ok(WolResult {
        mac: format_mac(mac),
        target,
    })
}

fn wol_target(value: Option<&str>) -> Result<SocketAddr> {
    let host = value.unwrap_or(DEFAULT_WOL_BROADCAST).trim();

    if host.contains(':') {
        Ok(host.parse()?)
    } else {
        Ok(format!("{host}:{DEFAULT_WOL_PORT}").parse()?)
    }
}

fn parse_mac(value: &str) -> Result<[u8; 6]> {
    let compact: String = value
        .chars()
        .filter(|ch| *ch != ':' && *ch != '-' && *ch != '.')
        .collect();

    if compact.len() != 12 || !compact.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(LgtvctlError::InvalidMac(value.to_string()));
    }

    let mut mac = [0u8; 6];
    for index in 0..6 {
        let start = index * 2;
        mac[index] = u8::from_str_radix(&compact[start..start + 2], 16)
            .map_err(|_| LgtvctlError::InvalidMac(value.to_string()))?;
    }

    Ok(mac)
}

fn magic_packet(mac: [u8; 6]) -> [u8; 102] {
    let mut packet = [0xFFu8; 102];
    for index in 0..16 {
        let start = 6 + index * 6;
        packet[start..start + 6].copy_from_slice(&mac);
    }
    packet
}

fn format_mac(mac: [u8; 6]) -> String {
    mac.iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}
