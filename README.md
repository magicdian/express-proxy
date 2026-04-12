# eproxy

`eproxy` is a local HTTP forwarder for model APIs.

It listens on localhost and forwards requests to an upstream HTTPS endpoint while connecting through a specific IP route and preserving TLS validation with the correct SNI/domain.

## Why

When direct routing to `https://xxx.domain.com:8443` is unstable, you may still have a better IX path to an IP such as `a.b.c.d`.

`eproxy` lets your SDK call local HTTP:

- client -> `http://127.0.0.1:9718/...`

Then it forwards to upstream HTTPS with:

- TCP connect target -> `a.b.c.d:8443`
- TLS SNI -> `xxx.domain.com`
- HTTP Host -> `xxx.domain.com` (default)

## Commands

- `eproxy run` - run in foreground
- `eproxy daemon` - run in background
- `eproxy install` - Linux only, install a `systemd --user` service

Global option:

- `--config <path>` - custom config path (default `~/.eproxy/config.toml`)

## Configuration

First run auto-creates a template at `~/.eproxy/config.toml` if missing.

Example:

```toml
[log]
level = "info"

[[listeners]]
name = "default"
listen = "127.0.0.1:9718"
# Domain/path used for TLS SNI + Host semantics.
upstream = "https://xxx.domain.com"
connect_ip = "a.b.c.d"
# Real TCP dial port.
connect_port = 8443
# sni = "xxx.domain.com"
# host_header = "xxx.domain.com"
```

Flow model:

- local -> listen -> connect_ip:connect_port -> upstream(domain/SNI)

## Linux service

Install and start user service:

```bash
eproxy install
```

Check status:

```bash
systemctl --user status eproxy.service
```

Disable later:

```bash
systemctl --user disable --now eproxy.service
```

## Notes

- Local listener is HTTP by design in MVP.
- Upstream must be HTTPS.
- eBPF/XDP optimization is intentionally deferred in this version.

## Versioning

- `Cargo.toml` (`[package].version`) is the single source of truth for version.
- Version format: `YYMM.d.BuildNumber`
- Example:
  - `2604.12.1` means 2026-04-12 first build.
  - `2604.9.1` means 2026-04-09 first build.
