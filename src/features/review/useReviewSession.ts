import { useCallback, useEffect, useRef, useState } from 'react';
import type { Api } from '../../shared/api';
import type { ReviewView } from '../../shared/generated/ReviewView';
import type { RateInput } from '../../shared/generated/RateInput';
import type { Rating } from '../../shared/generated/Rating';
import type { ReviewFilter } from '../../shared/generated/ReviewFilter';
import type { ChoiceSubmission } from '../../shared/generated/ChoiceSubmission';
import type { ConfirmChoicesInput } from '../../shared/generated/ConfirmChoicesInput';

type ReviewAction =
  { kind: 'rateCard'; input: RateInput } | { kind: 'confirmChoices'; input: ConfirmChoicesInput };

export function useReviewSession(api: Api, sessionId: string) {
  const [view, setView] = useState<ReviewView | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [errorIsBackground, setErrorIsBackground] = useState(false);
  const errorSource = useRef<{ code: string; background: boolean } | null>(null);
  const [busy, setBusy] = useState(false);
  const [pendingAction, setPendingAction] = useState<ReviewAction['kind'] | null>(null);
  const [message, setMessage] = useState('');
  const current = useRef<ReviewView | null>(null);
  const alive = useRef(true);
  const humanBusy = useRef(false);
  const pollBusy = useRef(false);
  const pending = useRef<ReviewAction | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const clearError = useCallback(() => {
    errorSource.current = null;
    setError(null);
    setErrorIsBackground(false);
  }, []);
  const reportError = useCallback((error: unknown, background: boolean) => {
    if (!alive.current) return;
    if (background && pending.current && errorSource.current?.background === false) return;
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
      let result: ReviewView | undefined;
      await serialize(async () => {
        if (!alive.current) return;
        try {
          const response = await operation();
          if (alive.current) {
            result = response;
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
      return result;
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
    setPendingAction(null);
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
  const sendAction = (action: ReviewAction) =>
    run(
      async () => {
        if (!pending.current) {
          const latest = current.current;
          if (
            latest?.state !== 'question' ||
            latest.payload.presentationId !== action.input.presentationId ||
            latest.payload.cardRevision !== action.input.expectedCardRevision
          )
            throw { code: 'conflict' };
          pending.current = action;
          setPendingAction(action.kind);
        }
        if (pending.current !== action) throw { code: 'conflict' };
        const response =
          action.kind === 'rateCard'
            ? await api.call('rateCard', action.input)
            : await api.call('confirmChoices', action.input);
        pending.current = null;
        setPendingAction(null);
        return response;
      },
      true,
      action.kind === 'rateCard' ? 'حُفظ التقييم.' : '',
    );
  const identity = (frame: Extract<ReviewView, { state: 'question' }>['payload']) => ({
    sessionId,
    presentationId: frame.presentationId,
    expectedCardRevision: frame.cardRevision,
    actionId: crypto.randomUUID(),
  });
  const rate = (rating: Rating) => {
    if (view?.state === 'question' && view.payload.canRate && !pending.current)
      return sendAction({ kind: 'rateCard', input: { ...identity(view.payload), rating } });
    return Promise.resolve();
  };
  const confirmChoices = (submission: ChoiceSubmission) => {
    if (
      view?.state === 'question' &&
      view.payload.choices &&
      !view.payload.choices.confirmedCorrect &&
      !pending.current
    )
      return sendAction({
        kind: 'confirmChoices',
        input: { ...identity(view.payload), submission },
      });
    return Promise.resolve();
  };
  const retryRating = () => {
    if (pending.current) return sendAction(pending.current);
    return Promise.resolve();
  };
  const continueFeedback = () => {
    if (view?.state !== 'choiceFeedback') return Promise.resolve();
    const { feedbackId } = view.payload;
    return run(() => api.call('continueChoiceFeedback', { sessionId, feedbackId }));
  };
  const recover = () =>
    run(
      async () => {
        const response = await api.call('readSession', { sessionId });
        pending.current = null;
        setPendingAction(null);
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
    pendingRating: pendingAction !== null,
    pendingAction,
    message,
    reveal,
    rate,
    confirmChoices,
    continueFeedback,
    retryRating,
    recover,
    undo,
    setFilter,
    end,
    refresh,
  };
}
