# Directory Structure

> How backend code is organized in this project.

---

## Overview

The backend is a single Rust binary crate with domain-oriented modules under `src/`.
Keep module boundaries small and explicit: CLI orchestration in `main.rs`, behavior in dedicated modules.

---

## Directory Layout

```text
src/
├── main.rs      # CLI entrypoint, command dispatch, logging bootstrap
├── config.rs    # ~/.eproxy layout + config TOML parsing/validation
├── proxy.rs     # HTTP listener + upstream forwarding pipeline
├── daemon.rs    # background process spawn and pid management
└── install.rs   # Linux systemd --user installation flow
```

---

## Module Organization

- `main.rs` only wires command flow; it should not contain transport logic.
- `config.rs` owns config schema and validation rules.
- `proxy.rs` owns request/response forwarding behavior.
- `daemon.rs` owns process lifecycle for detached mode.
- `install.rs` owns Linux-specific service registration.

If a new feature introduces a new responsibility (for example metrics), add a new module rather than bloating existing ones.

---

## Naming Conventions

- File names: lowercase snake_case (`proxy.rs`, `install.rs`).
- Types: PascalCase (`ResolvedConfig`, `ResolvedListener`).
- Functions: snake_case (`load_or_bootstrap`, `spawn_daemon`).
- CLI subcommands: lowercase words (`run`, `daemon`, `install`).

---

## Real Examples

1. Command dispatch stays in `main.rs`, implementation delegated to module functions.
2. Config validation is centralized in `resolve_config` rather than spread across call sites.
3. Forwarding state is encapsulated in `ProxyState` to keep request handler stateless.

Reference files:
- `src/main.rs`
- `src/config.rs`
- `src/proxy.rs`
