# Directory Structure

> How frontend code should be organized once introduced.

---

## Current Reality

No frontend code exists in this repository yet.
The structure below is the required starting contract for first implementation.

---

## Bootstrap Directory Layout

```text
web/
├── src/
│   ├── app/          # app shell, routes, providers
│   ├── features/     # feature-based modules
│   ├── components/   # shared UI components
│   ├── hooks/        # shared hooks
│   ├── lib/          # API client, utils, constants
│   ├── types/        # shared type declarations
│   └── styles/       # global/theme styles
└── tests/
```

---

## Module Organization Rules

- Prefer feature folders for domain logic (`features/chat`, `features/models`).
- Shared logic belongs in `lib/` or `hooks/` only when reused by 2+ features.
- Keep page/router composition in `app/`, not in deep component files.

---

## Naming Conventions

- Components: `PascalCase.tsx`
- Hooks: `useXxx.ts`
- Utilities/types: `snake_case.ts` or `kebab-case.ts` (choose one and stay consistent)
- Feature folders: `kebab-case`

---

## Example Targets (to add when code exists)

When frontend code lands, add at least 2-3 real file paths from repo here.
