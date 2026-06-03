/**
 * Tiny class-name joiner. Falsy values are dropped. We intentionally
 * avoid `clsx` and `tailwind-merge` deps — the primitives only need a
 * small, predictable joiner that preserves token class order so later
 * overrides via `className` win (the consumer-supplied class is
 * appended last, matching Tailwind v4's "last-wins" cascade).
 */
export function cn(
  ...parts: Array<string | false | null | undefined>
): string {
  let out = '';
  for (const part of parts) {
    if (!part) continue;
    if (out.length > 0) out += ' ';
    out += part;
  }
  return out;
}
