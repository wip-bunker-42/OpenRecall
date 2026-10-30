import { useEffect, useId, useRef } from 'react';
import type { Api } from '../../shared/api';
import type { ReviewFrame } from '../../shared/generated/ReviewFrame';
import type { Rating } from '../../shared/generated/Rating';
import { RatingButtons } from '../review/RatingButtons';
import { SourceContexts } from './SourceContexts';
export function SpellingQuestion({
  api,
  frame,
  onRate,
  disabled,
  timezone,
}: {
  api: Api;
  frame: ReviewFrame;
  onRate: (rating: Rating) => void;
  disabled: boolean;
  timezone: string;
}) {
  const heading = useRef<HTMLHeadingElement>(null);
  const id = useId();
  useEffect(() => {
    heading.current?.focus();
  }, [frame.presentationId]);
  return (
    <div className="study-column">
      <section aria-labelledby={`${id}-word`}>
        <h1 id={`${id}-word`} tabIndex={-1} ref={heading}>
          الكلمة
        </h1>
        <p className="study-text spelling-word" lang="en" dir="ltr">
          {frame.form.front}
        </p>
      </section>
      <label htmlFor={`${id}-practice`}>مساحة الكتابة</label>
      <textarea
        id={`${id}-practice`}
        aria-describedby={`${id}-help`}
        lang="en"
        dir="ltr"
        spellCheck={false}
        autoCorrect="off"
        autoComplete="off"
        autoCapitalize="off"
      />
      <p className="field-help" id={`${id}-help`}>
        كتابة حرة دون تصحيح أو مقارنة. تختفي الكتابة بمجرد مغادرة البطاقة، حتى عند التراجع عن
        التقييم.
      </p>
      <RatingButtons
        frame={frame}
        onRate={onRate}
        disabled={disabled || !frame.canRate}
        timezone={timezone}
      />
      <SourceContexts api={api} cardId={frame.card.id} />
    </div>
  );
}
