# lgtvctl

`lgtvctl` is a small Rust CLI for controlling LG webOS TVs from a PC, Linux box or OpenWrt router.
The project is intended to become a local HTTP bridge for Alice/Yandex smart-home scenarios.

## Current stage

Implemented:

```bash
lgtvctl probe
lgtvctl pair
lgtvctl status
lgtvctl off
lgtvctl on
lgtvctl volume up
lgtvctl volume down
lgtvctl volume set 15
lgtvctl mute
lgtvctl mute on
lgtvctl mute off
```

Still planned:

```bash
lgtvctl hdmi 1
lgtvctl app youtube
lgtvctl key HOME
HTTP API for Alice/OpenWrt service mode
OpenWrt package polish
```

## Windows / PC test flow

```powershell
cargo build
.\target\debug\lgtvctl.exe --host 192.168.0.116 probe
.\target\debug\lgtvctl.exe --host 192.168.0.116 pair
.\target\debug\lgtvctl.exe --host 192.168.0.116 status
.\target\debug\lgtvctl.exe --host 192.168.0.116 volume down
.\target\debug\lgtvctl.exe --host 192.168.0.116 off
```

`pair` saves `client_key` into `./lgtvctl.toml` unless `--config` or `LGTVCTL_CONFIG` is used.

## Turning TV on

`on` cannot use the webOS WebSocket API when the TV is fully asleep/offline. It sends a Wake-on-LAN magic packet instead.
You need the TV MAC address:

```powershell
.\target\debug\lgtvctl.exe --mac AA:BB:CC:DD:EE:FF on
```

With a known subnet broadcast:

```powershell
.\target\debug\lgtvctl.exe --mac AA:BB:CC:DD:EE:FF --wol-broadcast 192.168.0.255 on
```

For reliable wake-up, LG TV settings may need network standby / mobile wake / quick start enabled.

## Config

Example `lgtvctl.toml`:

```toml
host = "192.168.0.116"
port = 3001
client_key = ""
verify_certificate = false
timeout_ms = 3000
pair_timeout_ms = 60000
mac = "AA:BB:CC:DD:EE:FF"
wol_broadcast = "192.168.0.255"
```

Config lookup order:

1. `--config <path>`
2. `LGTVCTL_CONFIG`
3. `./lgtvctl.toml`
4. `./config/lgtvctl.toml`
5. `/etc/lgtvctl.toml`

## OpenWrt direction

Do not compile on the router. Build through OpenWrt SDK for the exact target/subtarget.
The Rust code avoids OpenSSL and uses `rustls` for easier cross-compilation.
