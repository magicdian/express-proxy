# Database Guidelines

> Database patterns and conventions for this project.

---

## Current Status

This project currently has **no database**.
Do not add persistence casually; the current runtime model is stateless forwarding + file-based config/logs in `~/.eproxy/`.

---

## If Database Is Introduced Later

Treat DB introduction as an architecture change requiring:

1. A new code-spec update in this directory
2. Migration tooling decision (for example `sqlx migrate`)
3. Backup/rollback plan
4. Test coverage for migration and rollback paths

---

## Naming Conventions (Future Contract)

If added, default naming should be:

- Tables: snake_case plural (`listener_routes`)
- Columns: snake_case (`created_at`)
- Indexes: `idx_<table>_<column(s)>`

---

## Query Patterns (Future Contract)

- Keep SQL/query code in dedicated module(s), not mixed with proxy request handlers.
- Wrap multi-step mutations in transactions.
- Never log secrets/tokens in query logs.

---

## Common Mistakes To Avoid

- Mixing runtime cache and durable storage semantics.
- Coupling live request path with blocking DB operations.
- Introducing DB without migration and rollback tests.
