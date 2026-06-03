import { forwardRef, type InputHTMLAttributes } from 'react';
import { cn } from '~/design-system/utils';

export interface InputProps
  extends InputHTMLAttributes<HTMLInputElement> {
  /** Force the error visual + `aria-invalid`. Pairs with Field's `error`. */
  invalid?: boolean;
}

const base =
  'block w-full bg-surface-2 text-text placeholder:text-text-subtle ' +
  'border border-border rounded-md px-3 py-2 text-sm ' +
  'transition-colors focus:outline-none focus:ring-2 focus:ring-accent ' +
  'focus:border-accent disabled:opacity-50 disabled:cursor-not-allowed';

export const Input = forwardRef<HTMLInputElement, InputProps>(
  function Input(
    { invalid = false, className, type = 'text', ...rest },
    ref,
  ) {
    return (
      <input
        ref={ref}
        type={type}
        data-ui="input"
        aria-invalid={invalid || undefined}
        className={cn(
          base,
          invalid && 'border-danger focus:border-danger focus:ring-danger',
          className,
        )}
        {...rest}
      />
    );
  },
);
