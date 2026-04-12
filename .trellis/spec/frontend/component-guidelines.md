# Component Guidelines

> How components should be built in this project.

---

## Current Reality

No frontend component code exists yet.
Use these rules as the baseline contract for first components.

---

## Component Structure

Recommended order inside component file:

1. imports
2. local types/interfaces
3. component function
4. small private helpers

Minimal example contract:

```tsx
type ModelCardProps = {
  modelId: string;
  latencyMs: number;
};

export function ModelCard({ modelId, latencyMs }: ModelCardProps) {
  return <article>{modelId} ({latencyMs}ms)</article>;
}
```

---

## Props Conventions

- Use explicit prop types; avoid `any`.
- Prefer required props; optional only when truly optional in UI semantics.
- Prefer primitive and stable object props over deep mutable structures.

---

## Styling Patterns

- Keep style strategy consistent within one app (CSS Modules or utility-first or CSS-in-JS).
- Do not mix multiple styling paradigms in one feature unless justified.
- Avoid inline style objects for complex styling.

---

## Accessibility

- Interactive elements must be keyboard reachable.
- Use semantic HTML first (`button`, `label`, `nav`, `main`).
- Inputs require accessible labels.

---

## Common Mistakes

- Dumping business logic into presentational components.
- Overusing optional props instead of separate components.
- Replacing semantic elements with generic `div` nodes.
