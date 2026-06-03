# Design system

Linear-like visual direction for the rust-backend-playground frontend.
Tokens live in **`tokens.css`** and are exposed as Tailwind v4 utility
classes through the `@theme` directive. Feature routes and components
consume them via class names only — never inline styles, never hardcoded
hex values.

## File map

```
src/design-system/
├── README.md        # this file — rationale per token group
└── tokens.css       # the single source of truth (@theme block)
```

The CSS is imported once from `src/routes/__root.tsx` so it loads on
every route.

## Direction

- **Dark by default** (`color-scheme: dark` on `<html>`). Linear's
  primary mode is dark; we mirror that. A light variant can be added
  later by wrapping the same `@theme` block in a `.light` selector.
- **Restrained palette** — three surface layers (`bg`, `surface`,
  `surface-2`) graded by lightness, plus a single accent
  (`#5E6AD2`, Linear-blue). No secondary or tertiary brand colors.
- **Hairline borders** — `border-subtle` is `#1F2024` (4% lighter
  than `bg`) and is the default divider; `border` is the heavier
  outline used for card edges.
- **Low-contrast elevation** — shadows are tuned for dark surfaces
  (higher alpha black) and include a subtle white inset hint to
  separate panels from the next-lighter surface.
- **Compact density** — body text is **14px** (Linear default),
  metadata is **13px**, headings top out at 28px. Radii are small
  (4 / 6 / 8 / pill).
- **System font stack first** — `Inter` is the preferred brand
  webfont, but the system stack is the fallback when Inter hasn't
  loaded yet (and is the only font shipped if we never add a webfont).

## Token reference

### Surfaces

| Token | Hex | Utility | Use |
|---|---|---|---|
| `bg` | `#08090A` | `bg-bg` | App background, page chrome |
| `surface` | `#0F1011` | `bg-surface` | Card, panel, modal body |
| `surface-2` | `#161719` | `bg-surface-2` | Nested surface (row hover, popover content) |

### Text

| Token | Hex | Utility | Use |
|---|---|---|---|
| `text` | `#F7F8F8` | `text-text` | Primary copy, headings |
| `text-muted` | `#B4B8BE` | `text-text-muted` | Labels, secondary metadata |
| `text-subtle` | `#62666D` | `text-text-subtle` | Placeholders, disabled labels |

### Borders

| Token | Hex | Utility | Use |
|---|---|---|---|
| `border` | `#2C2D31` | `border-border` | Card / panel outline |
| `border-subtle` | `#1F2024` | `border-border-subtle` | Hairline divider |

### Accent

| Token | Hex | Utility | Use |
|---|---|---|---|
| `accent` | `#5E6AD2` | `bg-accent`, `text-accent`, `ring-accent` | Primary actions, focus ring, links |
| `accent-fg` | `#FFFFFF` | `text-accent-fg` | Foreground on accent backgrounds |

### Semantic

| Token | Hex | Use |
|---|---|---|
| `danger` | `#EB5757` | Destructive states, errors |
| `success` | `#4CB782` | Confirmation, saved state |
| `warning` | `#F2C94C` | Caution, "almost-due" hints |

### Radius

| Token | Value | Utility |
|---|---|---|
| `sm` | `4px` | `rounded-sm` |
| `md` | `6px` | `rounded-md` |
| `lg` | `8px` | `rounded-lg` |
| `pill` | `9999px` | `rounded-pill` |

### Shadow

| Token | Use |
|---|---|
| `xs` | Barely-there lift for tags, chips |
| `sm` | Default card / panel |
| `md` | Popovers, dropdowns |
| `lg` | Modals, command palettes |

## Usage rules

1. **Never** write a hex value, `rgb()`, or `oklch()` in a feature
   file. If you need a color that isn't in the table, add it here
   first.
2. **Never** use `style={{ ... }}` to override color, radius, or
   shadow. The base layer handles the global theme; class names
   handle the rest.
3. **Prefer utility classes** (`bg-surface text-text-muted
   rounded-md shadow-xs`) over custom CSS in feature routes. Custom
   CSS lives in this directory or in component-local stylesheets for
   primitives (Task 6).
4. **Transitions** are pure CSS via `transition-colors`,
   `transition-shadow`, `transition-transform` (built into Tailwind).
   No animation libraries.
