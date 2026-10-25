import { useEffect, useRef } from 'react';
import { errorText } from '../shared/errors';
export function FormErrors({ error, focus = true }: { error: unknown; focus?: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (error && focus) ref.current?.focus();
  }, [error, focus]);
  return error ? (
    <div role="alert" className="error" tabIndex={-1} ref={ref}>
      {errorText(error)}
    </div>
  ) : null;
}
