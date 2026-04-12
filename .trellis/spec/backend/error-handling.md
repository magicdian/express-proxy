# Error Handling

> How errors are handled in this project.

---

## Overview

Use `anyhow` for command-level errors and attach context at every boundary.
User-facing CLI errors must be actionable and include failing path/operation where possible.

---

## Error Types

Current project uses:

- `anyhow::Result<T>` as the default return type across modules.
- `bail!` for early exits with human-readable messages.
- `Context`/`with_context` to preserve operation boundaries.

---

## Error Handling Patterns

### Pattern: Add boundary context

```rust
let raw = fs::read_to_string(config_path)
    .with_context(|| format!("failed to read config file {}", config_path.display()))?;
```

### Pattern: Bootstrap then fail fast for missing config

```rust
if !config_path.exists() {
    write_default_config(config_path)?;
    bail!("config file not found. A template has been created at {}", config_path.display());
}
```

### Pattern: Top-level unified error exit

```rust
if let Err(err) = try_main().await {
    eprintln!("error: {err:#}");
    std::process::exit(1);
}
```

---

## Runtime Error Matrix

| Scenario | Behavior | Exit/HTTP |
|---|---|---|
| Config file missing | Create template and stop | CLI exit code 1 |
| Config parse/validation invalid | Return clear error with location/context | CLI exit code 1 |
| Upstream request failure | Log error and return generic downstream message | HTTP 502 |
| Response build failure | Log error and return internal proxy message | HTTP 500 |
| Listener task panic/join failure | Surface as process error | Process exits non-zero |

---

## API Error Responses

Proxy responses to clients should avoid leaking low-level internals.

- Upstream connectivity failures: `502 bad gateway: upstream request failed`
- Internal response construction failures: `500 internal proxy error`

Detailed error strings go to logs, not to client response body.

---

## Common Mistakes

- Returning raw upstream library errors directly to clients.
- Using `unwrap()` in runtime flow.
- Losing call-site context by returning bare errors.
