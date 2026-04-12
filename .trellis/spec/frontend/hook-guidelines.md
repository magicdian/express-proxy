# Hook Guidelines

> How hooks should be used in this project.

---

## Current Reality

No frontend hooks exist yet.
Apply these conventions when introducing first custom hooks.

---

## Custom Hook Patterns

- Hook names must start with `use`.
- A hook should encapsulate one concern (fetching, polling, local derived state).
- Keep side-effects inside hook internals, not scattered in callers.

Example pattern:

```ts
export function useModelLatency(modelId: string) {
  // encapsulate request, loading/error state, and refresh behavior
}
```

---

## Data Fetching

- Choose one server-state solution for the app and stay consistent.
- Normalize error/loading contract across hooks.
- Support cancellation/cleanup for in-flight requests when component unmounts.

---

## Naming Conventions

- `useXxxQuery` for read operations
- `useXxxMutation` for write operations
- `useXxxState` for local composite state

---

## Common Mistakes

- Triggering effects with unstable dependency arrays.
- Returning overly broad mutable objects from hooks.
- Hiding critical errors inside silent fallback values.
