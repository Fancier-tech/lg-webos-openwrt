# lgtvctl

`lgtvctl` is a small Rust CLI utility for controlling LG webOS TVs from Linux/OpenWrt.

Target use case: LG G5 TV → router/OpenWrt helper → HTTP/API bridge → Alice/Yandex smart home scenarios.

## Stage 2 status

This repository currently contains:

- CLI structure;
- config loading;
- safe config printing;
- dry-run mode;
- WSS transport probe to `wss://<tv-ip>:3001/`;
- certificate verification switch for LG/webOS self-signed or locally untrusted certificates.

Pairing and actual TV commands are not implemented yet. Stage 2 only checks that the tool can open a secure WebSocket connection to the TV.

## Commands

```bash
lgtvctl config
lgtvctl probe
lgtvctl pair
lgtvctl on
lgtvctl off
lgtvctl status
lgtvctl volume up
lgtvctl volume down
lgtvctl volume set 15
lgtvctl mute
lgtvctl mute on
lgtvctl mute off
lgtvctl hdmi 1
lgtvctl app youtube
lgtvctl app kodi
lgtvctl key HOME
lgtvctl key BACK
```

Only `config`, `probe`, and `--dry-run` are functional in Stage 2.

## Build

```bash
cargo build
cargo build --release
```

Rust 1.85+ is expected because this project uses Rust edition 2024.

If your OpenWrt build environment has an older Rust toolchain, change this in `Cargo.toml`:

```toml
edition = "2021"
```

and remove/adjust `rust-version`.

## Config

The tool checks config in this order:

1. `--config <path>`
2. `LGTVCTL_CONFIG`
3. `./lgtvctl.toml`
4. `./config/lgtvctl.toml`
5. `/etc/lgtvctl.toml`

Example:

```toml
host = "192.168.0.116"
port = 3001
client_key = ""
verify_certificate = false
timeout_ms = 3000
# mac = "AA:BB:CC:DD:EE:FF"
# wol_broadcast = "192.168.0.255"
```

For LG TVs, keep `verify_certificate = false` for now. LG webOS commonly presents a certificate that will not validate cleanly from a small CLI/OpenWrt environment.

## First local checks

```bash
cargo run -- config
cargo run -- --dry-run --host 192.168.0.116 status
cargo run -- --host 192.168.0.116 probe
```

Expected dry-run result:

```text
dry-run: host=192.168.0.116 port=3001 command=status
```

Expected `probe` result when the TV is on and reachable:

```text
connected: wss://192.168.0.116:3001/ http_status=101 Switching Protocols
```

If `probe` times out or returns connection refused, check:

- TV and computer/router are in the same LAN;
- TV is powered on;
- TV IP address is correct;
- no guest Wi-Fi/client isolation blocks local traffic;
- TCP port `3001` is reachable.

## Next stages

Stage 3:

- `pair` command;
- LG webOS register request;
- client key extraction;
- saving client key to config.

Stage 4:

- `status`;
- `off`;
- basic volume commands.

Stage 5:

- Wake-on-LAN;
- HDMI switching;
- app launch;
- HTTP wrapper for Alice/Yandex integration.
