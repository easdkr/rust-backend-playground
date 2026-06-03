import type { HTMLAttributes, ReactNode } from 'react';
import { cn } from '~/design-system/utils';

export interface ErrorMessageProps extends HTMLAttributes<HTMLParagraphElement> {
  /** When `true`, render a filled block (more padding, surface-2 bg). */
  block?: boolean;
  children: ReactNode;
}

/**
 * ErrorMessage — single-line (default) or block (filled) error copy.
 * Use inside `Field` (auto-injected with id + role="alert") or as a
 * top-level form error.
 */
export function ErrorMessage({
  block = false,
  className,
  children,
  ...rest
}: ErrorMessageProps) {
  return (
    <p
      role="alert"
      data-ui="form-error"
      className={cn(
        'text-danger text-xs',
        block && 'bg-surface-2 border border-border-subtle ' +
          'rounded-md px-3 py-2 text-sm',
        className,
      )}
      {...rest}
    >
      {children}
    </p>
  );
}
