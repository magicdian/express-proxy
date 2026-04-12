# Type Safety

> Type safety patterns for frontend code.

---

## Current Reality

No frontend TypeScript code exists yet.
Use this as baseline for first TS module.

---

## Type Organization

- Feature-local types stay near feature files.
- Cross-feature contracts go into shared `types/`.
- API payload types should map to backend contract fields explicitly.

---

## Validation

- Runtime input validation is required at external boundaries (API responses, URL params, storage restores).
- Prefer schema-driven validation and inferred static types.

---

## Common Patterns

- Prefer discriminated unions for state machines.
- Prefer narrow string unions over free-form strings where possible.
- Use small type guards for uncertain runtime data.

---

## Forbidden Patterns

- `any` in production code.
- Blind `as` casting for API responses.
- Optional-everything payload types without validation.
