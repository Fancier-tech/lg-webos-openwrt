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
lgtvctl serve
```

Still planned:

```bash
lgtvctl hdmi 1
lgtvctl app youtube
lgtvctl key HOME
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


## HTTP API service mode

Start the local API:

```powershell
.\target\debug\lgtvctl.exe --host 192.168.0.116 --mac AA:BB:CC:DD:EE:FF --wol-broadcast 192.168.0.255 serve
```

By default it listens on `127.0.0.1:8765`. Override it with `--listen` or `http_listen` in config:

```powershell
.\target\debug\lgtvctl.exe --config .\lgtvctl.toml --listen 127.0.0.1:8765 serve
```

Implemented endpoints accept both `GET` and `POST` for easier testing and local automation:

```text
GET  /health
POST /tv/on
POST /tv/off
POST /tv/status
POST /tv/volume/up
POST /tv/volume/down
POST /tv/volume/set?value=10
POST /tv/volume/set/10
POST /tv/mute
POST /tv/mute/on
POST /tv/mute/off
```

PowerShell examples:

```powershell
Invoke-RestMethod http://127.0.0.1:8765/health
Invoke-RestMethod -Method Post http://127.0.0.1:8765/tv/volume/down
Invoke-RestMethod -Method Post 'http://127.0.0.1:8765/tv/volume/set?value=10'
Invoke-RestMethod -Method Post http://127.0.0.1:8765/tv/off
Invoke-RestMethod -Method Post http://127.0.0.1:8765/tv/on
```

On OpenWrt, bind to LAN only if the network is trusted. Do not expose this HTTP API to the internet.

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
http_listen = "127.0.0.1:8765"
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
