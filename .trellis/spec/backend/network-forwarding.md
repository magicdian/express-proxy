# Network Forwarding Code-Spec

> Executable contract for eproxy local HTTP -> upstream HTTPS forwarding.

## 1. Scope / Trigger

Use this spec whenever changes touch any of:

- CLI `run`/`daemon` behavior affecting listeners
- Upstream transport configuration (SNI, host routing, connect target)
- Header forwarding / streaming behavior
- Linux install flow that changes runtime launch command

## 2. Signatures

### CLI

- `eproxy run [--config <path>]`
- `eproxy daemon [--config <path>]`
- `eproxy install [--config <path>]` (Linux-only)

### Config file

Path default: `~/.eproxy/config.toml`

```toml
[log]
level = "info"

[[listeners]]
name = "default"
listen = "127.0.0.1:9718"
# Domain/path used for TLS SNI + Host semantics.
upstream = "https://xxx.domain.com"
connect_ip = "a.b.c.d"
connect_port = 8443
# sni = "xxx.domain.com"
# host_header = "xxx.domain.com"
```

## 3. Contracts

### Request contract

- Incoming protocol: local HTTP listener (`listen`)
- Outgoing protocol: HTTPS only (`upstream` must be `https://`)
- TCP connect target: `connect_ip:connect_port`
- `upstream` host/path defines TLS/HTTP semantic target; runtime dial port is `connect_port`
- TLS ServerName (SNI): `sni` if provided, otherwise upstream host
- HTTP Host header: `host_header` if provided, otherwise SNI/domain
- Hop-by-hop request headers must not be forwarded

### Response contract

- Preserve upstream status code
- Preserve non-hop-by-hop response headers
- Preserve streaming/chunked behavior (do not force full buffering)

### Runtime files

- Config: `~/.eproxy/config.toml`
- Logs: `~/.eproxy/logs/`
- PID file: `~/.eproxy/eproxy.pid` (daemon mode)

## 4. Validation & Error Matrix

| Validation point | Rule | Failure behavior |
|---|---|---|
| Config exists | If missing, write template first | CLI exits non-zero with path hint |
| Listener list | At least one `[[listeners]]` required | CLI exits non-zero |
| Upstream scheme | Must be `https` | CLI exits non-zero |
| `listen` parse | Must be valid socket addr | CLI exits non-zero |
| `connect_ip` parse | Must be valid IP | CLI exits non-zero |
| `connect_port` parse | Must be valid TCP port (1-65535) | CLI exits non-zero |
| Host/SNI parse | Must be valid host/header value | CLI exits non-zero |
| Upstream request | Network/TLS failure | Return HTTP 502 |
| Response build | Local response construction failure | Return HTTP 500 |
| Linux install | `systemctl` unavailable/fails | Install command exits non-zero |

## 5. Good / Base / Bad Cases

### Good

- Configured `upstream=https://api.example.com:8443`, `connect_ip=1.2.3.4`, `connect_port=8443`
- Local `http://127.0.0.1:9718/v1/chat/completions` forwards successfully
- TLS cert validates against `api.example.com`
- SSE stream is relayed chunk-by-chunk

### Base

- Single listener with defaults (`sni` and `host_header` omitted)
- Tool derives SNI/Host from upstream domain

### Bad

- `upstream = "http://..."` -> reject on startup
- `connect_ip = "not-an-ip"` -> reject on startup
- `connect_port = 0` -> reject on startup
- Missing config -> template is created, process exits for user edit

## 6. Tests Required (with assertion points)

1. URL rewrite test
- Assert merged path/query equals expected upstream URL.

2. Header chain test
- Assert `x-forwarded-for` appends peer IP correctly.

3. Startup validation tests
- Assert invalid config cases return specific failure messages.

4. (Recommended integration) TLS SNI behavior
- With test upstream cert for domain, assert connect via IP still validates with configured SNI.

## 7. Wrong vs Correct

### Wrong

```rust
// Connect directly by IP URL and skip SNI/domain contract.
let url = format!("https://{}:{}", connect_ip, port);
let client = reqwest::Client::new();
```

Why wrong: cert validation may fail; Host semantics drift from domain contract.

### Correct

```rust
// Keep domain URL for TLS/Host semantics, override DNS resolution to target IP.
let client = reqwest::Client::builder()
    .resolve(&server_name, connect_addr)
    .build()?;
```

Why correct: preserves TLS SNI + domain validation while routing over desired IP path.
