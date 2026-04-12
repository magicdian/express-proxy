# State Management

> How state should be managed in this project.

---

## Current Reality

No frontend state layer exists yet.
This document defines decision boundaries for first implementation.

---

## State Categories

- Local UI state: component-local (`open`, `selectedTab`).
- Feature state: shared inside one feature module.
- Global app state: cross-route/session-level state.
- Server state: remote API data and cache.

---

## Promotion Rules

Promote state scope only when needed:

1. Keep local by default.
2. Promote to feature-level when 2+ sibling components need it.
3. Promote to global only for true app-wide concerns.

---

## Server State Rules

- Treat server state as cache, not source of truth for local form edits.
- Use explicit stale/revalidate policy per query.
- Keep request key construction deterministic.

---

## Common Mistakes

- Putting ephemeral UI state into global store.
- Duplicating server state into multiple local stores.
- Forgetting loading/error UX states.
