# Quality Guidelines

> Code quality standards for backend development.

---

## Overview

All backend changes must pass compile + test + lint gates and keep module responsibilities clear.

---

## Forbidden Patterns

- `unwrap()` / `expect()` in production control flow (tests are allowed).
- Mixing command wiring, config parsing, and proxy transport logic in one file.
- Returning internal dependency errors directly to API clients.
- Adding platform-specific behavior without `cfg` guards.

---

## Required Patterns

- Prefer `anyhow::Result` + `.context(...)` for boundary-aware errors.
- Keep transport contracts explicit in code-spec (`network-forwarding.md`).
- Preserve streaming semantics (no forced full buffering).
- Keep Linux-specific behavior isolated in dedicated module (`install.rs`).

---

## Testing Requirements

Minimum for proxy-related changes:

1. Unit test URL/path rewrite behavior.
2. Unit test forwarding header behavior (for example `x-forwarded-for`).
3. Run full checks:
   - `cargo fmt`
   - `cargo test`
   - `cargo clippy --all-targets --all-features -- -D warnings`

---

## Code Review Checklist

- Are config fields validated with clear error messages?
- Is TLS/SNI behavior preserved for upstream cert validation?
- Are hop-by-hop headers correctly filtered?
- Are client-facing error responses sanitized?
- Are logs useful without leaking secrets?
