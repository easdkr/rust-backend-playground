import type { HTMLAttributes } from 'react';
import { cn } from '~/design-system/utils';

export type SpinnerSize = 'sm' | 'md' | 'lg';

export interface SpinnerProps extends HTMLAttributes<HTMLSpanElement> {
  size?: SpinnerSize;
}

const sizeMap: Record<SpinnerSize, string> = {
  sm: 'h-3 w-3 border-2',
  md: 'h-4 w-4 border-2',
  lg: 'h-6 w-6 border-[3px]',
};

/**
 * Spinner — pure-CSS animated ring. Inherits the `accent` color via
 * `border-t-accent`; the rest of the ring is `border-border-subtle`
 * so it sits inside both `bg-surface` and `bg-surface-2` panels.
 */
export function Spinner({
  size = 'md',
  className,
  ...rest
}: SpinnerProps) {
  return (
    <span
      role="status"
      aria-label="Loading"
      data-ui="spinner"
      data-size={size}
      className={cn(
        'inline-block rounded-full border-border-subtle border-t-accent ' +
          'animate-spin',
        sizeMap[size],
        className,
      )}
      {...rest}
    />
  );
}
