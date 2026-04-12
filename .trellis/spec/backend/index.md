# Backend Development Guidelines

> Executable backend conventions for `eproxy`.

---

## Overview

This project is currently a Rust CLI/network-forwarder backend.
Guidelines here are based on real implementation in `src/` and should be treated as implementation contracts.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Module organization and file layout | Filled |
| [Database Guidelines](./database-guidelines.md) | DB policy and future adoption contract | Filled (No DB yet) |
| [Error Handling](./error-handling.md) | Error types, propagation, user-facing behavior | Filled |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging and log safety | Filled |
| [Quality Guidelines](./quality-guidelines.md) | Quality gates and review checklist | Filled |
| [Network Forwarding Code-Spec](./network-forwarding.md) | SNI/IP forwarding executable contract | Filled |

---

## Pre-Development Checklist

Before editing backend code, read:

1. `quality-guidelines.md`
2. `error-handling.md`
3. `logging-guidelines.md`
4. `network-forwarding.md` (for proxy/transport changes)
5. `database-guidelines.md` (only if DB is introduced)

---

## Current Reality Snapshot

- Runtime: Rust + Tokio
- CLI: `run`, `daemon`, `install`
- Proxy behavior: local HTTP -> upstream HTTPS, connect by IP, validate cert with domain/SNI
- Database: not used

---

**Language**: All documentation should be written in **English**.
