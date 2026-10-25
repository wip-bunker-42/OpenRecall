import type { ReactNode } from 'react';
export function Disclosure({ title, children }: { title: string; children: ReactNode }) {
  return (
    <details className="disclosure">
      <summary>{title}</summary>
      <div className="disclosure-body">{children}</div>
    </details>
  );
}
