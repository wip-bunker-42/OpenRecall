import { useEffect, useId, useRef } from 'react';
import type { ReviewFrame } from '../../shared/generated/ReviewFrame';
import type { Rating } from '../../shared/generated/Rating';
import { RatingButtons } from './RatingButtons';
export function ReviewQuestion({
  frame,
  onReveal,
  onRate,
  disabled,
  timezone,
}: {
  frame: ReviewFrame;
  onReveal: () => void;
  onRate: (rating: Rating) => void;
  disabled: boolean;
  timezone: string;
}) {
  const question = useRef<HTMLHeadingElement>(null);
  const answer = useRef<HTMLHeadingElement>(null);
  const id = useId();
  useEffect(() => {
    if (frame.answerRevealed) answer.current?.focus();
    else question.current?.focus();
  }, [frame.presentationId, frame.answerRevealed]);
  return (
    <div className="study-column">
      <section aria-labelledby={`${id}-question`}>
        <h1 ref={question} id={`${id}-question`} tabIndex={-1}>
          السؤال
        </h1>
        <p className="study-text question-text" dir="auto" lang={frame.form.language ?? undefined}>
          {frame.form.front}
        </p>
      </section>
      {frame.answerRevealed ? (
        <>
          <section aria-labelledby={`${id}-answer`} className="answer-block">
            <h2 id={`${id}-answer`} ref={answer} tabIndex={-1}>
              الإجابة
            </h2>
            <p className="study-text" dir="auto" lang={frame.form.language ?? undefined}>
              {frame.form.back}
            </p>
            {frame.form.notes.trim() && (
              <>
                <h3>الملاحظات</h3>
                <p className="study-text muted" dir="auto" lang={frame.form.language ?? undefined}>
                  {frame.form.notes}
                </p>
              </>
            )}
          </section>
          <RatingButtons frame={frame} onRate={onRate} disabled={disabled} timezone={timezone} />
        </>
      ) : (
        <button className="primary reveal-button" disabled={disabled} onClick={onReveal}>
          عرض الإجابة
        </button>
      )}
    </div>
  );
}
