import { useEffect, useId, useRef } from 'react';
import type { CardForm } from '../../shared/generated/CardForm';
import type { ChoiceFeedbackFrame } from '../../shared/generated/ChoiceFeedbackFrame';
import type { NativeApi } from '../../shared/api';
import { dateTime, interval } from '../../shared/format';
import { CopyQuestion, TypedPrompt } from './TypedQuestion';

export function orderedChoices(form: CardForm, order: string[]) {
  const choices = new Map(form.multipleChoice?.choices.map((choice) => [choice.id, choice]));
  return order.flatMap((id) => {
    const choice = choices.get(id);
    return choice ? [choice] : [];
  });
}

export function ChoiceResult({
  form,
  order,
  feedback,
  timezone,
}: {
  form: CardForm;
  order: string[];
  feedback?: ChoiceFeedbackFrame;
  timezone?: string;
}) {
  const id = useId();
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    heading.current?.focus();
  }, []);
  const correct = new Set(form.multipleChoice?.correctChoiceIds);
  const choices = orderedChoices(form, order);
  const selected = new Set(feedback?.selectedIds);
  const wrong = new Set(feedback?.incorrectIds);
  const missed = new Set(feedback?.missedIds);
  return (
    <section className="answer-block" aria-labelledby={id}>
      <h2 id={id} ref={heading} tabIndex={-1}>
        النتيجة
      </h2>
      <p>
        {feedback
          ? feedback.unknown
            ? 'تخطيت السؤال. سُجّل تقييم «نسيت».'
            : 'الإجابة غير صحيحة. سُجّل تقييم «نسيت».'
          : 'إجابتك صحيحة.'}
      </p>
      <p>الإجابات الصحيحة</p>
      <ul>
        {choices
          .filter((choice) => correct.has(choice.id))
          .map((choice) => (
            <li key={choice.id}>
              <bdi className="study-text" lang={form.language ?? undefined}>
                {choice.text}
              </bdi>
              {missed.has(choice.id) && <span className="muted"> — لم تخترها</span>}
            </li>
          ))}
      </ul>
      {feedback && selected.size > 0 && (
        <>
          <p>اختياراتك</p>
          <ul>
            {choices
              .filter((choice) => selected.has(choice.id))
              .map((choice) => (
                <li key={choice.id}>
                  <bdi className="study-text" lang={form.language ?? undefined}>
                    {choice.text}
                  </bdi>
                  <span className="muted">
                    {wrong.has(choice.id) ? ' — اختيار غير صحيح' : ' — اختيار صحيح'}
                  </span>
                </li>
              ))}
          </ul>
        </>
      )}
      {form.notes.trim() && (
        <>
          <h3>الملاحظات</h3>
          <p className="study-text" dir="auto" lang={form.language ?? undefined}>
            {form.notes}
          </p>
        </>
      )}
      {feedback?.lastReview && (
        <p>
          المراجعة القادمة لهذه البطاقة {interval(feedback.lastReview.intervalSeconds)}،{' '}
          {dateTime(feedback.lastReview.dueAt, timezone)}.
        </p>
      )}
    </section>
  );
}

export function ChoiceFeedback({
  frame,
  copy,
  onContinue,
  disabled,
  timezone,
}: {
  frame: ChoiceFeedbackFrame;
  copy?: NativeApi['copyQuestion'];
  onContinue: () => void;
  disabled: boolean;
  timezone: string;
}) {
  return (
    <div className="study-column">
      <TypedPrompt form={frame.form} focus={false} />
      {copy && (
        <CopyQuestion
          copy={copy}
          disabled={disabled}
          input={{
            sessionId: frame.sessionId,
            displayId: frame.feedbackId,
            expectedCardRevision: frame.cardRevision,
          }}
        />
      )}
      <ChoiceResult form={frame.form} order={frame.order} feedback={frame} timezone={timezone} />
      <button type="button" className="primary" disabled={disabled} onClick={onContinue}>
        متابعة المراجعة
      </button>
    </div>
  );
}
