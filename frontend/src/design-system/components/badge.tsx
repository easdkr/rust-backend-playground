import { forwardRef, type HTMLAttributes } from 'react';
import { cn } from '~/design-system/utils';

export type BadgeTone = 'neutral' | 'info' | 'success' | 'warn' | 'danger';

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  tone?: BadgeTone;
}

const toneMap: Record<BadgeTone, string> = {
  neutral: 'bg-surface-2 text-text-muted border-border-subtle',
  info: 'bg-surface-2 text-accent border-border-subtle',
  success: 'bg-surface-2 text-success border-border-subtle',
  warn: 'bg-surface-2 text-warning border-border-subtle',
  danger: 'bg-surface-2 text-danger border-border-subtle',
};

/**
 * Badge — small pill used to surface metadata, status, or counts.
 * Inline by default; pair with `text-xs`/`text-sm` for sizing. Tones
 * intentionally use a single background (surface-2) and vary only the
 * foreground — keeps the row of badges visually quiet on dark surfaces.
 */
export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(function Badge(
  { tone = 'neutral', className, children, ...rest },
  ref,
) {
  return (
    <span
      ref={ref}
      data-ui="badge"
      data-tone={tone}
      className={cn(
        'inline-flex items-center gap-1 rounded-pill border px-2 py-0.5 ' +
          'text-xs font-medium',
        toneMap[tone],
        className,
      )}
      {...rest}
    >
      {children}
    </span>
  );
});
