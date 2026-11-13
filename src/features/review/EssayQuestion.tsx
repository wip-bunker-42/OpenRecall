import type { NativeApi } from '../../shared/api';
import type { Rating } from '../../shared/generated/Rating';
import type { ReviewFrame } from '../../shared/generated/ReviewFrame';
import { RatingButtons } from './RatingButtons';
import { CopyQuestion, TypedPrompt } from './TypedQuestion';

export function EssayQuestion({
  frame,
  copy,
  onRate,
  disabled,
}: {
  frame: ReviewFrame;
  copy?: NativeApi['copyQuestion'];
  onRate: (rating: Rating) => void;
  disabled: boolean;
}) {
  return (
    <div className="study-column">
      <TypedPrompt form={frame.form} focus />
      {copy && (
        <CopyQuestion
          copy={copy}
          disabled={disabled}
          input={{
            sessionId: frame.sessionId,
            displayId: frame.presentationId,
            expectedCardRevision: frame.cardRevision,
          }}
        />
      )}
      {frame.form.notes.trim() && (
        <details>
          <summary>الملاحظات</summary>
          <p className="study-text" dir="auto" lang={frame.form.language ?? undefined}>
            {frame.form.notes}
          </p>
        </details>
      )}
      {frame.canRate && <RatingButtons frame={frame} onRate={onRate} disabled={disabled} />}
    </div>
  );
}
