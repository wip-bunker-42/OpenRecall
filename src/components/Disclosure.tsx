import type { ReactNode } from 'react';
export function Disclosure({
  title,
  children,
  level = 2,
}: {
  title: string;
  children: ReactNode;
  level?: 2 | 3;
}) {
  return (
    <details className="disclosure">
      <summary>{level === 2 ? <h2>{title}</h2> : <h3>{title}</h3>}</summary>
      <div className="disclosure-body">{children}</div>
    </details>
  );
}
