export function StatusRegion({ message }: { message: string }) {
  return (
    <p role="status" aria-live="polite" aria-atomic="true" className="status">
      {message}
    </p>
  );
}
