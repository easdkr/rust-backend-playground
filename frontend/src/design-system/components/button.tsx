import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';
import { cn } from '~/design-system/utils';
import { Spinner } from '~/design-system/components/spinner';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md';

export interface ButtonProps
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  children?: ReactNode;
}

const base =
  'inline-flex items-center justify-center gap-2 font-medium ' +
  'transition-colors focus:outline-none focus-visible:ring-2 ' +
  'focus-visible:ring-accent rounded-pill ' +
  'disabled:opacity-50 disabled:cursor-not-allowed ' +
  'disabled:pointer-events-none';

const sizeMap: Record<ButtonSize, string> = {
  sm: 'px-3 py-1 text-xs',
  md: 'px-4 py-1.5 text-sm',
};

const variantMap: Record<ButtonVariant, string> = {
  primary: 'bg-accent text-accent-fg hover:opacity-90 active:opacity-80',
  secondary:
    'bg-surface-2 text-text border border-border hover:bg-surface active:bg-bg',
  ghost:
    'text-text-muted hover:text-text hover:bg-surface-2 active:bg-surface',
  danger: 'bg-danger text-accent-fg hover:opacity-90 active:opacity-80',
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  function Button(
    {
      variant = 'primary', size = 'md', loading = false, disabled,
      className, type = 'button', children, ...rest
    },
    ref,
  ) {
    const isDisabled = disabled === true || loading === true;
    return (
      <button
        ref={ref}
        type={type}
        data-ui="button"
        data-variant={variant}
        disabled={isDisabled}
        aria-disabled={isDisabled || undefined}
        aria-busy={loading || undefined}
        className={cn(
          base, sizeMap[size], variantMap[variant],
          loading && 'cursor-progress', className,
        )}
        {...rest}
      >
        {loading ? (
          <><Spinner size="sm" />{children ? <span>{children}</span> : null}</>
        ) : children}
      </button>
    );
  },
);
