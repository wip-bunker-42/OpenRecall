import { useCallback, useEffect, useRef, useState } from 'react';
/** Refresh native results; the browser never decides the study-day boundary. */
export function useStatistics<T>(read: () => Promise<T>, revision: string) {
  const [stats, setStats] = useState<T | null>(null);
  const [error, setError] = useState<unknown>(null);
  const alive = useRef(false);
  const sequence = useRef(0);
  const refresh = useCallback(async () => {
    const request = ++sequence.current;
    try {
      const result = await read();
      if (alive.current && request === sequence.current) {
        setStats(result);
        setError(null);
      }
    } catch (e) {
      if (alive.current && request === sequence.current) setError(e);
    }
  }, [read]);
  useEffect(() => {
    alive.current = true;
    void refresh();
    const visible = () => {
      if (document.visibilityState === 'visible') void refresh();
    };
    const timer = window.setInterval(visible, 60000);
    window.addEventListener('focus', visible);
    document.addEventListener('visibilitychange', visible);
    return () => {
      alive.current = false;
      ++sequence.current;
      window.clearInterval(timer);
      window.removeEventListener('focus', visible);
      document.removeEventListener('visibilitychange', visible);
    };
  }, [refresh, revision]);
  return { stats, error, refresh };
}
