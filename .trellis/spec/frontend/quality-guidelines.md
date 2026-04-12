# Quality Guidelines

> Code quality standards for frontend development.

---

## Current Reality

No frontend code exists yet.
These are baseline quality gates for first and future frontend work.

---

## Forbidden Patterns

- Direct DOM manipulation bypassing framework model.
- UI components making side-effectful network calls inline without hook/service layer.
- Silent error swallowing in async flows.

---

## Required Patterns

- Lint and typecheck must pass before merge.
- UI states include loading, success, error, and empty-state handling where relevant.
- New components must include accessibility checks.

---

## Testing Requirements

Minimum for new frontend features:

1. Component behavior tests for user-visible logic.
2. Hook tests for custom state/data logic.
3. Typecheck and lint in CI.

---

## Code Review Checklist

- Is state scope minimal and intentional?
- Are API contracts typed and validated?
- Is error/loading UX explicitly handled?
- Is accessibility considered for interactive elements?
