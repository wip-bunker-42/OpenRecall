import { useId, type InputHTMLAttributes } from 'react';
import { fieldError } from '../shared/forms';
export function Field({
  label,
  help,
  error,
  name,
  ...input
}: {
  label: string;
  help?: string;
  error?: unknown;
  name: string;
} & InputHTMLAttributes<HTMLInputElement>) {
  const id = useId();
  const issue = fieldError(error, name);
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      <input
        {...input}
        id={id}
        name={name}
        aria-invalid={issue ? true : undefined}
        aria-describedby={
          [help ? `${id}-help` : '', issue ? `${id}-error` : ''].filter(Boolean).join(' ') ||
          undefined
        }
      />
      {help && (
        <p className="field-help" id={`${id}-help`}>
          {help}
        </p>
      )}
      {issue && (
        <p className="field-error" id={`${id}-error`}>
          {issue}
        </p>
      )}
    </div>
  );
}
