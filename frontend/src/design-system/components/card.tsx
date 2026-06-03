import { forwardRef, type HTMLAttributes } from 'react';
import { cn } from '~/design-system/utils';

export type SurfaceVariant = 'default' | 'nested';

export interface SurfaceProps extends HTMLAttributes<HTMLDivElement> {
  variant?: SurfaceVariant;
}

const variantMap: Record<SurfaceVariant, string> = {
  default: 'bg-surface border border-border-subtle rounded-md shadow-xs p-6',
  nested: 'bg-surface-2 border border-border-subtle rounded-md p-4',
};

const Surface = forwardRef<HTMLDivElement, SurfaceProps>(function Surface(
  { variant = 'default', className, children, ...rest },
  ref,
) {
  return (
    <div
      ref={ref}
      data-ui="surface"
      data-variant={variant}
      className={cn(variantMap[variant], className)}
      {...rest}
    >
      {children}
    </div>
  );
});

/**
 * Card — a bordered, rounded panel with a subtle shadow. Use for the
 * outer wrapper of a self-contained region (form, list item, modal body).
 * `Card` is the primary public name for the panel variant; `Surface`
 * is the lower-level primitive that also exposes a quieter `nested`
 * variant.
 */
export const Card = forwardRef<HTMLDivElement, HTMLAttributes<HTMLDivElement>>(
  function Card({ className, children, ...rest }, ref) {
    return (
      <div
        ref={ref}
        data-ui="card"
        className={cn(
          'bg-surface border border-border-subtle rounded-md shadow-xs p-6',
          className,
        )}
        {...rest}
      >
        {children}
      </div>
    );
  },
);

export { Surface };
