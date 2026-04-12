# Logging Guidelines

> How logging is done in this project.

---

## Overview

Use `tracing` + `tracing-subscriber` for structured logs.
Logs should go to both stdout and rolling file output under `~/.eproxy/logs/`.

---

## Log Levels

- `info`: lifecycle events (startup, listener bound, shutdown signal)
- `warn`: recoverable anomalies where processing can continue
- `error`: request/listener failures or fatal operational issues
- `debug`/`trace`: optional deep diagnostics (enabled by config log level)

---

## Structured Logging

Required fields where available:

- `listener`
- `bind` or `target`
- `method`
- `error`

Examples from current implementation:

```rust
info!(listener = %listener_name, bind = %bind_addr, "listener started");
```

```rust
error!(listener = %state.listener_name, method = %method, target = %target_url, error = %err, "upstream request failed");
```

---

## What to Log

- Service start/stop events
- Listener startup and binding
- Upstream request failures
- Join/shutdown task errors

---

## What NOT to Log

- API keys, bearer tokens, secrets in headers/body
- Full request body payloads by default
- User PII unless explicitly required and approved
