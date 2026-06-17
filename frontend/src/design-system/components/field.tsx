import { useId, type ReactNode } from 'react';
import { cn } from '~/design-system/utils';
import { ErrorMessage } from '~/design-system/components/error-message';

export interface FieldControlProps {
  /** Field-injected id. Forward to the control's `id` prop. */
  id: string;
  /** True when the field has an error or was marked invalid. */
  invalid: boolean;
  /** Field-injected `aria-describedby` targeting the hint or error node. */
  'aria-describedby'?: string;
}

export interface FieldProps {
  /** Visible label text. */
  label: string;
  /** Optional helper copy shown beneath the control. */
  hint?: ReactNode;
  /** Optional error copy; switches the hint to a danger-styled message. */
  error?: ReactNode;
  /** Pass `true` to mark the control invalid even before the first error. */
  invalid?: boolean;
  /**
   * The control. Use the render-prop form to receive the Field-injected
   * `id` / `invalid` / `aria-describedby`:
   *   <Field label="Title">{(p) => <Input {...p} />}</Field>
   */
  children: ReactNode | ((props: FieldControlProps) => ReactNode);
  /** Optional className for the outer wrapper. */
  className?: string;
  /** Explicit id for the label/control link. Falls back to a generated id. */
  htmlFor?: string;
  /** Hide the visible label while keeping it for screen readers. */
  hideLabel?: boolean;
  /** Render `children` inside a flex row (radio / check group). */
  inline?: boolean;
}

/**
 * Layout + a11y wrapper around any form control. Owns the `<label
 * htmlFor>` link, the `aria-describedby` link to the hint/error node,
 * the error styling decision, and the unique id used to wire the above.
 */
export function Field({
  label, hint, error, invalid = false, children,
  className, htmlFor, hideLabel = false, inline = false,
}: FieldProps) {
  const reactId = useId();
  const controlId = htmlFor ?? reactId;
  const showError = invalid === true || Boolean(error);
  const descId = showError
    ? `${controlId}-error`
    : hint ? `${controlId}-hint` : undefined;
  const controlProps: FieldControlProps = {
    id: controlId,
    invalid: showError,
    ...(descId ? { 'aria-describedby': descId } : {}),
  };
  return (
    <div
      data-ui="field"
      data-invalid={showError || undefined}
      className={cn('flex flex-col gap-1.5', className)}
    >
      <label
        htmlFor={controlId}
        className={cn(
          'text-text-muted text-xs font-medium',
          hideLabel && 'sr-only',
        )}
      >
        {label}
      </label>
      <div className={cn(inline && 'flex items-center gap-2')}>
        {typeof children === 'function' ? children(controlProps) : children}
      </div>
      {showError ? (
        <ErrorMessage id={descId}>{error}</ErrorMessage>
      ) : hint ? (
        <p id={descId} className="text-text-subtle text-xs">{hint}</p>
      ) : null}
    </div>
  );
}
