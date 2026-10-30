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
}: {
  api: Api;
  frame: ReviewFrame;
  onRate: (rating: Rating) => void;
  disabled: boolean;
}) {
  const input = useRef<HTMLTextAreaElement>(null);
  const id = useId();
  useEffect(() => {
    input.current?.focus();
    input.current?.setSelectionRange(0, 0);
  }, [frame.presentationId]);
  return (
    <div className="study-column">
      <section aria-labelledby={`${id}-word`}>
        <h1 id={`${id}-word`} tabIndex={-1}>
          الكلمة
        </h1>
        <p className="study-text spelling-word" lang="en" dir="ltr">
          {frame.form.front}
        </p>
      </section>
      <label htmlFor={`${id}-practice`}>مساحة الكتابة</label>
      <textarea
        ref={input}
        defaultValue={`${frame.form.front}\n`}
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
        الكلمة في السطر الأول، وتحته سطر للكتابة الحرة دون تصحيح أو مقارنة. تختفي الكتابة بمجرد
        مغادرة البطاقة؛ عند العودة يبدأ الحقل بالكلمة من جديد.
      </p>
      <RatingButtons frame={frame} onRate={onRate} disabled={disabled || !frame.canRate} />
      <SourceContexts api={api} cardId={frame.card.id} />
    </div>
  );
}
