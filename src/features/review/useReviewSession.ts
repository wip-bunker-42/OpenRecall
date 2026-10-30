import { useCallback, useEffect, useRef, useState } from 'react';
import type { Api } from '../../shared/api';
import type { ReviewView } from '../../shared/generated/ReviewView';
import type { RateInput } from '../../shared/generated/RateInput';
import type { Rating } from '../../shared/generated/Rating';
import type { ReviewFilter } from '../../shared/generated/ReviewFilter';
export function useReviewSession(api: Api, sessionId: string) {
  const [view, setView] = useState<ReviewView | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [errorIsBackground, setErrorIsBackground] = useState(false);
  const errorSource = useRef<{ code: string; background: boolean } | null>(null);
  const [busy, setBusy] = useState(false);
  const [pendingRating, setPendingRating] = useState(false);
  const [message, setMessage] = useState('');
  const current = useRef<ReviewView | null>(null);
  const alive = useRef(true);
  const humanBusy = useRef(false);
  const pollBusy = useRef(false);
  const pending = useRef<RateInput | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const clearError = useCallback(() => {
    errorSource.current = null;
    setError(null);
    setErrorIsBackground(false);
  }, []);
  const reportError = useCallback((error: unknown, background: boolean) => {
    if (!alive.current) return;
    const code =
      typeof error === 'object' && error !== null && 'code' in error
        ? String(error.code)
        : 'unknown';
    if (background && errorSource.current?.background && errorSource.current.code === code) return;
    errorSource.current = { code, background };
    setErrorIsBackground(background);
    setError(error);
  }, []);
  const serialize = useCallback((operation: () => Promise<void>) => {
    const result = queue.current.then(operation, operation);
    queue.current = result.catch(() => {});
    return result;
  }, []);
  const run = useCallback(
    async (operation: () => Promise<ReviewView>, human = true, success = '') => {
      if (!alive.current || (human ? humanBusy.current : pollBusy.current || humanBusy.current))
        return;
      if (human) {
        humanBusy.current = true;
        setBusy(true);
        clearError();
      } else pollBusy.current = true;
      await serialize(async () => {
        if (!alive.current) return;
        try {
          const response = await operation();
          if (alive.current) {
            current.current = response;
            setView(response);
            if (human) {
              clearError();
              setMessage(success);
            } else if (errorSource.current?.background) {
              clearError();
            }
          }
        } catch (e) {
          reportError(e, !human);
        } finally {
          if (human) {
            humanBusy.current = false;
            if (alive.current) setBusy(false);
          } else pollBusy.current = false;
        }
      });
    },
    [serialize, clearError, reportError],
  );
  const refresh = useCallback(
    () => run(() => api.call('readSession', { sessionId }), false),
    [api, sessionId, run],
  );
  const activity = useCallback(
    (active: boolean) =>
      serialize(async () => {
        try {
          await api.call('setSessionActivity', { sessionId, active });
        } catch (e) {
          reportError(e, true);
        }
      }),
    [api, sessionId, serialize, reportError],
  );
  useEffect(() => {
    alive.current = true;
    humanBusy.current = false;
    pollBusy.current = false;
    current.current = null;
    setView(null);
    pending.current = null;
    setPendingRating(false);
    void refresh();
    void activity(document.visibilityState === 'visible' && document.hasFocus());
    const focus = () => {
      void activity(document.visibilityState === 'visible');
      void refresh();
    };
    const blur = () => {
      void activity(false);
    };
    const visibility = () => {
      if (document.visibilityState === 'hidden') blur();
      else if (document.hasFocus()) focus();
    };
    window.addEventListener('focus', focus);
    window.addEventListener('blur', blur);
    document.addEventListener('visibilitychange', visibility);
    const heartbeat = window.setInterval(() => {
      if (current.current?.state !== 'summary') void refresh();
    }, 15000);
    return () => {
      alive.current = false;
      clearInterval(heartbeat);
      window.removeEventListener('focus', focus);
      window.removeEventListener('blur', blur);
      document.removeEventListener('visibilitychange', visibility);
      void activity(false);
    };
  }, [api, sessionId, refresh, activity]);
  const nextDue = view?.payload.nextDueAt;
  const state = view?.state;
  const filter = view?.payload.filter;
  useEffect(() => {
    if (!nextDue || state === 'summary') return;
    const delta = Date.parse(nextDue) - Date.now();
    const delay = Number.isFinite(delta) ? Math.min(60000, Math.max(500, delta)) : 60000;
    const timer = window.setTimeout(() => void refresh(), delay);
    return () => clearTimeout(timer);
  }, [nextDue, state, filter, refresh]);
  const reveal = () => run(() => api.call('revealAnswer', { sessionId }));
  const rate = (rating: Rating) =>
    run(
      async () => {
        if (!pending.current) {
          const v = current.current;
          if (v?.state !== 'question' || !v.payload.canRate) throw { code: 'noQuestion' };
          pending.current = {
            sessionId,
            presentationId: v.payload.presentationId,
            rating,
            actionId: crypto.randomUUID(),
            expectedCardRevision: v.payload.cardRevision,
          };
          setPendingRating(true);
        }
        const response = await api.call('rateCard', pending.current);
        pending.current = null;
        setPendingRating(false);
        return response;
      },
      true,
      'حُفظ التقييم.',
    );
  const retryRating = () => {
    if (pending.current) return rate(pending.current.rating);
    return Promise.resolve();
  };
  const recover = () =>
    run(
      async () => {
        const response = await api.call('readSession', { sessionId });
        pending.current = null;
        setPendingRating(false);
        return response;
      },
      true,
      'تم تحديث الجلسة.',
    );
  const undo = () =>
    run(
      () => api.call('undoReview', { sessionId }),
      true,
      'تم التراجع عن آخر تقييم واستعادة البطاقة.',
    );
  const setFilter = (filter: ReviewFilter) =>
    run(() => api.call('setReviewFilter', { sessionId, filter }));
  const end = () => run(() => api.call('endSession', { sessionId }), true, 'انتهت الجلسة.');
  return {
    view,
    error,
    errorIsBackground,
    busy,
    pendingRating,
    message,
    reveal,
    rate,
    retryRating,
    recover,
    undo,
    setFilter,
    end,
    refresh,
  };
}
