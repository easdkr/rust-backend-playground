import { forwardRef, type TextareaHTMLAttributes } from 'react';
import { cn } from '~/design-system/utils';

export interface TextareaProps
  extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  /** Force the error visual + `aria-invalid`. Pairs with Field's `error`. */
  invalid?: boolean;
}

const base =
  'block w-full bg-surface-2 text-text placeholder:text-text-subtle ' +
  'border border-border rounded-md px-3 py-2 text-sm ' +
  'transition-colors focus:outline-none focus:ring-2 focus:ring-accent ' +
  'focus:border-accent disabled:opacity-50 disabled:cursor-not-allowed ' +
  'resize-y';

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  function Textarea(
    { invalid = false, className, rows = 4, ...rest },
    ref,
  ) {
    return (
      <textarea
        ref={ref}
        rows={rows}
        data-ui="textarea"
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
