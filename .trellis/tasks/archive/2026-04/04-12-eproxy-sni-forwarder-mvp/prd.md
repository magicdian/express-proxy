# Build eproxy SNI forwarder MVP

## Goal
Build a lightweight local forwarder named `eproxy` for model APIs.
The tool listens on localhost and forwards to upstream HTTPS over IX IP routes, while preserving TLS validation by setting the expected SNI/domain behavior.

## What I already know
- User wants a local endpoint such as `http://127.0.0.1:9718/` for model clients.
- Upstream is reached via IP (`a.b.c.d:port`) for better route quality.
- TLS cert validation requires domain-based SNI (e.g. `xxx.domain.com`) instead of raw IP.
- Preferred language is Rust.
- Runtime form is CLI:
  - Foreground: `eproxy run`
  - Background: `eproxy daemon`
  - Linux install: `eproxy install` (check `systemctl` availability first)
- Primary runtime target is Linux; cross-platform support is desired.
- Runtime data should live under `~/.eproxy/` (`config.toml`, `logs/`).
- Multiple listener-to-upstream mappings may be needed.
- Current repository is newly initialized and effectively empty (no codebase constraints yet).

## Assumptions (confirmed for MVP)
- MVP uses local HTTP listener only (no local HTTPS listener).
- Upstream TLS verification remains enabled (secure by default).
- `Host` header should default to the configured domain unless explicitly overridden.
- eBPF is explicitly deferred.
- Config hot-reload is deferred.
- Default local binding is loopback-only.

## Open Questions
- None for MVP implementation start.

## Requirements
- Build a Rust CLI application named `eproxy`.
- Provide commands:
  - `eproxy run`
  - `eproxy daemon`
  - `eproxy install` (Linux-specific)
- Local listener supports HTTP on loopback (default `127.0.0.1:9718`).
- Forward to HTTPS upstream while enabling stable IX routing through configured IP:
  - TLS SNI to configured domain.
  - HTTP Host to configured domain by default.
- Support request and streaming response pass-through (SSE/chunked).
- Config file at `~/.eproxy/config.toml`.
- Log output under `~/.eproxy/logs/`.
- Support one or more forwarding rules.
- `eproxy install` must check if `systemctl` is available before attempting setup.

## Acceptance Criteria
- [ ] Running `eproxy run` starts configured listeners and forwards traffic successfully.
- [ ] Upstream TLS handshake uses configured SNI and passes certificate verification.
- [ ] `Host` header behavior matches configured domain default.
- [ ] Streaming responses are relayed without truncation or buffering breakage.
- [ ] Running `eproxy daemon` starts the same forwarding logic in background mode.
- [ ] On Linux, `eproxy install` checks `systemctl`, creates/updates a systemd unit, and enables service when available.
- [ ] Logs are emitted to `~/.eproxy/logs/` and include request-level failures.

## Definition of Done
- Unit/integration tests for config and core forwarding behavior.
- Lint/build checks pass.
- Basic runtime docs for config + run/daemon/install usage.
- Risks and deferred items documented.

## Technical Approach
- Rust + Tokio async runtime.
- HTTP server via Axum.
- Upstream forwarding via Reqwest with rustls TLS.
- Use reqwest DNS override (`resolve`) to connect to IX IP while preserving domain-based TLS/SNI validation.
- Configurable per-listener routing entries loaded from TOML.
- `daemon` command uses child-process detachment strategy.
- `install` command writes Linux systemd service and executes `systemctl` flow when available.

## Decision (ADR-lite)
Context: Need a reliable, maintainable MVP for SNI-based API forwarding.
Decision: Use pure userspace Rust implementation first; defer eBPF.
Consequences:
- Faster implementation and easier cross-platform validation.
- Better maintainability and simpler debugging.
- Peak performance optimization can be revisited in later Linux-specific versions.

## Out of Scope
- Full API gateway features (rate limit, plugin auth chain, admin dashboard).
- Dynamic service discovery/control plane.
- Deep kernel datapath acceleration (eBPF/XDP) in first release.
- Local HTTPS listener.
- Config hot-reload.

## Technical Notes
- Task directory: `.trellis/tasks/04-12-eproxy-sni-forwarder-mvp/`
- Primary risk: preserving streaming correctness and TLS verification simultaneously.
