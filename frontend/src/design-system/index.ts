/**
 * Top-level design-system barrel.
 *
 * Route layer:
 *   import '~/design-system/tokens.css'   // Tailwind v4 token CSS — once in __root.tsx
 *   import { Button, Card, Field } from '~/design-system'
 *
 * No runtime exports beyond the component primitives + their types.
 */
export * from '~/design-system/components';
