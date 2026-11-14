import { useId } from 'react';
import type { NativeApi } from '../../shared/api';
import type { ChoiceSubmission } from '../../shared/generated/ChoiceSubmission';
import type { Rating } from '../../shared/generated/Rating';
import type { ReviewFrame } from '../../shared/generated/ReviewFrame';
import { ChoiceResult, orderedChoices } from './ChoiceFeedback';
import { RatingButtons } from './RatingButtons';
import { CopyQuestion, TypedPrompt } from './TypedQuestion';

export function MultipleChoiceQuestion({
  frame,
  selected,
  onSelection,
  onConfirm,
  onRate,
  copy,
  disabled,
}: {
  frame: ReviewFrame;
  selected: string[];
  onSelection: (ids: string[]) => void;
  onConfirm: (submission: ChoiceSubmission) => void;
  onRate: (rating: Rating) => void;
  copy?: NativeApi['copyQuestion'];
  disabled: boolean;
}) {
  const group = useId();
  const { choices, form } = frame;
  if (!choices || !form.multipleChoice) return null;
  const multiple = form.multipleChoice.selectionMode === 'multiple';
  const confirmed = choices.confirmedCorrect;
  const selectedIds = confirmed ? choices.selectedIds : selected;
  return (
    <div className="study-column">
      <TypedPrompt form={form} focus={!confirmed} />
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
      <fieldset className="choice-options" disabled={disabled || confirmed}>
        <legend>{multiple ? 'اختر جميع الإجابات الصحيحة' : 'اختر إجابة واحدة'}</legend>
        {orderedChoices(form, choices.order).map((choice) => (
          <label key={choice.id}>
            <input
              type={multiple ? 'checkbox' : 'radio'}
              name={group}
              checked={selectedIds.includes(choice.id)}
              onChange={(event) =>
                onSelection(
                  multiple
                    ? event.target.checked
                      ? [...selectedIds, choice.id]
                      : selectedIds.filter((id) => id !== choice.id)
                    : [choice.id],
                )
              }
            />
            <bdi className="study-text" lang={form.language ?? undefined}>
              {choice.text}
            </bdi>
          </label>
        ))}
      </fieldset>
      {confirmed ? (
        <>
          <ChoiceResult form={form} order={choices.order} />
          {frame.canRate && <RatingButtons frame={frame} onRate={onRate} disabled={disabled} />}
        </>
      ) : (
        <div className="actions">
          <button
            type="button"
            className="primary"
            disabled={disabled || selectedIds.length === 0}
            onClick={() => onConfirm({ kind: 'selected', choiceIds: selectedIds })}
          >
            تحقق من الإجابة
          </button>
          <button type="button" disabled={disabled} onClick={() => onConfirm({ kind: 'unknown' })}>
            لا أعرف، اعرض الإجابة
          </button>
        </div>
      )}
    </div>
  );
}
