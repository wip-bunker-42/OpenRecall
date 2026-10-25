import { useEffect, useRef } from 'react';
import type { Api } from '../../shared/api';
import type { Navigate } from '../../app/navigation';
import type { ReviewFilter } from '../../shared/generated/ReviewFilter';
import { useReviewSession } from './useReviewSession';
import { ReviewQuestion } from './ReviewQuestion';
import { ReviewSummary } from './ReviewSummary';
import { ratings } from './RatingButtons';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { dateTime, interval, number } from '../../shared/format';
const filters: { value: ReviewFilter; label: string }[] = [
  { value: 'all', label: 'كل البطاقات المستحقة' },
  { value: 'new', label: 'البطاقات الجديدة فقط' },
  { value: 'previouslyReviewed', label: 'بطاقات سبق تقييمها' },
];
export function ReviewPage({
  sessionId,
  api,
  navigate,
  shortcutsEnabled = true,
  timezone = 'Africa/Cairo',
  deckId,
  onBusyChange,
  externalBusy = false,
}: {
  sessionId: string;
  api: Api;
  navigate: Navigate;
  shortcutsEnabled?: boolean;
  timezone?: string;
  deckId?: string;
  onBusyChange?: (busy: boolean) => void;
  externalBusy?: boolean;
}) {
  const session = useReviewSession(api, sessionId);
  const { view, busy: sessionBusy, pendingRating } = session;
  const busy = sessionBusy || externalBusy;
  const latest = useRef(session);
  latest.current = session;
  const emptyHeading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    onBusyChange?.(sessionBusy);
    return () => onBusyChange?.(false);
  }, [sessionBusy, onBusyChange]);
  useEffect(() => {
    if (view?.state === 'emptyFilter') emptyHeading.current?.focus();
  }, [view?.state]);
  useEffect(() => {
    if (!shortcutsEnabled) return;
    const keydown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      const s = latest.current;
      if (
        e.repeat ||
        externalBusy ||
        e.altKey ||
        s.busy ||
        s.pendingRating ||
        target.closest('input,textarea,select,[contenteditable=true],[role=textbox]')
      )
        return;
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z' && s.view?.payload.canUndo) {
        e.preventDefault();
        void s.undo();
        return;
      }
      if (e.ctrlKey || e.metaKey) return;
      if (
        e.code === 'Space' &&
        !target.closest('button,summary') &&
        s.view?.state === 'question' &&
        !s.view.payload.answerRevealed
      ) {
        e.preventDefault();
        void s.reveal();
      } else if (
        /^[1-4]$/.test(e.key) &&
        s.view?.state === 'question' &&
        s.view.payload.answerRevealed
      ) {
        e.preventDefault();
        void s.rate(ratings[Number(e.key) - 1]);
      }
    };
    window.addEventListener('keydown', keydown);
    return () => window.removeEventListener('keydown', keydown);
  }, [shortcutsEnabled, externalBusy]);
  const last = view?.payload.lastReview;
  const lastMessage = last
    ? `موعد المراجعة الجديد: ${dateTime(last.dueAt, timezone)}، ${interval(last.intervalSeconds)}.`
    : '';
  return (
    <>
      <div className="review-toolbar">
        <button
          disabled={busy}
          onClick={() => navigate(deckId ? { page: 'deck', deckId } : { page: 'library' })}
        >
          {deckId ? 'العودة إلى المجموعة' : 'العودة إلى المجموعات'}
        </button>
        <button
          disabled={busy || pendingRating || !view?.payload.canUndo}
          onClick={() => void session.undo()}
        >
          التراجع عن آخر تقييم
        </button>
        {view && view.state !== 'summary' && (
          <button disabled={busy || pendingRating} onClick={() => void session.end()}>
            إنهاء الجلسة
          </button>
        )}
      </div>
      <FormErrors error={session.error} focus={!session.errorIsBackground} />
      {session.error && (
        <div className="actions">
          {pendingRating && (
            <button disabled={busy} onClick={() => void session.retryRating()}>
              إعادة محاولة التقييم
            </button>
          )}
          <button disabled={busy} onClick={() => void session.recover()}>
            تحديث الجلسة
          </button>
        </div>
      )}
      <StatusRegion message={[session.message, lastMessage].filter(Boolean).join(' ')} />
      {!view ? (
        <>
          <h1 tabIndex={-1}>المراجعة</h1>
          <p role="status">جارٍ فتح الجلسة…</p>
        </>
      ) : (
        <>
          {view.state !== 'summary' && (
            <>
              <fieldset className="review-filters" disabled={busy || pendingRating}>
                <legend>بطاقات هذه الجلسة</legend>
                {filters.map((f) => (
                  <label key={f.value}>
                    <input
                      type="radio"
                      name={`review-filter-${sessionId}`}
                      value={f.value}
                      checked={view.payload.filter === f.value}
                      onChange={() => void session.setFilter(f.value)}
                    />
                    {f.label}
                  </label>
                ))}
              </fieldset>
              <details className="filter-help">
                <summary>كيف تعمل الفلاتر؟</summary>
                <p>
                  الجديدة هي التي لم تُقيَّم من قبل. بعد أول تقييم، حتى «نسيت»، تنتقل إلى البطاقات
                  التي سبق تقييمها. تبديل الفلتر يحتفظ بالسؤال وصيغته وإظهار إجابته.
                </p>
              </details>
              <p className="review-progress" role="status" aria-live="polite">
                {view.state === 'question'
                  ? `البطاقة ${number(view.payload.progress.current)} من ${number(view.payload.progress.total)}`
                  : `راجعت ${number(view.payload.progress.completed)} بطاقة في الجلسة`}
              </p>
              <progress
                aria-label="تقدم الجلسة"
                value={view.payload.progress.completed}
                max={Math.max(1, view.payload.progress.total)}
              />
            </>
          )}
          {view.state === 'question' ? (
            <ReviewQuestion
              frame={view.payload}
              disabled={busy || pendingRating}
              onReveal={() => void session.reveal()}
              onRate={(r) => void session.rate(r)}
              timezone={timezone}
            />
          ) : view.state === 'emptyFilter' ? (
            <>
              <h1 tabIndex={-1} ref={emptyHeading}>
                لا توجد بطاقات مطابقة الآن
              </h1>
              <p>
                اختر فلترًا آخر أو أنهِ الجلسة. تدخل البطاقات التي يحين موعدها تلقائيًا ما دامت
                الجلسة مفتوحة.
              </p>
              {view.payload.nextDueAt && (
                <p>الاستحقاق القادم لهذا الفلتر: {dateTime(view.payload.nextDueAt, timezone)}</p>
              )}
            </>
          ) : (
            <ReviewSummary summary={view.payload} timezone={timezone} />
          )}
        </>
      )}
      {shortcutsEnabled && (
        <details className="shortcuts-help">
          <summary>اختصارات المراجعة</summary>
          <p>
            مسافة لعرض الإجابة، والأرقام 1 إلى 4 للتقييم، وCtrl+Z للتراجع. تتوقف الاختصارات داخل
            حقول النص وعلى الأزرار وعند تعطيلها في الإعدادات.
          </p>
        </details>
      )}
    </>
  );
}
