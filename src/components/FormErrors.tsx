import { useEffect, useRef } from 'react';
import { errorText } from '../shared/errors';
export function FormErrors({ error }: { error: unknown }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (error) ref.current?.focus();
  }, [error]);
  return error ? (
    <div role="alert" className="error" tabIndex={-1} ref={ref}>
      {errorText(error)}
    </div>
  ) : null;
}
