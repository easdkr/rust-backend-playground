import type { HTMLAttributes, ReactNode } from 'react';
import { cn } from '~/design-system/utils';

export interface EmptyStateProps
  extends Omit<HTMLAttributes<HTMLDivElement>, 'title'> {
  /** Big title (1 short line). */
  title: ReactNode;
  /** One or two sentences of supporting copy. */
  description?: ReactNode;
  /** Optional icon — pass any `ReactNode` (SVG, lucide, your own). */
  icon?: ReactNode;
  /** Optional CTA (typically a `<Button />`). */
  action?: ReactNode;
}

/**
 * EmptyState — neutral illustration block for "no data yet" panels.
 * Renders a centered icon + title + description + optional action. The
 * icon is a plain `ReactNode` so the design system stays free of an
 * icon-library dependency.
 */
export function EmptyState({
  title,
  description,
  icon,
  action,
  className,
  ...rest
}: EmptyStateProps) {
  return (
    <div
      data-ui="empty"
      className={cn(
        'flex flex-col items-center justify-center gap-3 ' +
          'rounded-md border border-border-subtle bg-surface ' +
          'px-6 py-10 text-center',
        className,
      )}
      {...rest}
    >
      {icon ? (
        <div
          aria-hidden="true"
          className="text-text-subtle flex h-10 w-10 items-center justify-center"
        >
          {icon}
        </div>
      ) : null}
      <div className="flex flex-col gap-1">
        <p className="text-text text-sm font-medium">{title}</p>
        {description ? (
          <p className="text-text-muted text-xs">{description}</p>
        ) : null}
      </div>
      {action ? <div className="mt-1">{action}</div> : null}
    </div>
  );
}
