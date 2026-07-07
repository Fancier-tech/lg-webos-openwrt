# lgtvctl

`lgtvctl` is a small Rust CLI utility for controlling LG webOS TVs from Linux/OpenWrt.

Target use case: LG G5 TV → router/OpenWrt helper → HTTP/API bridge → Alice/Yandex smart home scenarios.

## Stage 1 status

This repository currently contains the compile-ready project skeleton:

- CLI structure;
- config loading;
- safe config printing;
- dry-run mode;
- placeholders for future TV commands.

No network connection to TV is implemented in Stage 1 yet.

## Planned commands

```bash
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

## First local check

```bash
cargo run -- config
cargo run -- --dry-run --host 192.168.0.116 status
```

Expected result for `status` in Stage 1:

```text
dry-run: host=192.168.0.116 port=3001 command=status
```

Without `--dry-run`, command placeholders return `not implemented yet in stage 1`.

## Next stages

Stage 2:

- WebSocket/WSS transport;
- connection to `wss://<tv-ip>:3001`;
- SSL certificate verification switch.

Stage 3:

- `pair` command;
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
