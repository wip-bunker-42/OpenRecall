import { useId, useLayoutEffect, useRef } from 'react';
import type { ChoiceSetDraft } from '../../shared/generated/ChoiceSetDraft';
import { number } from '../../shared/format';

export const newChoice = () => ({ id: null, key: crypto.randomUUID(), text: '' });

export function ChoiceEditor({
  value,
  onChange,
}: {
  value: ChoiceSetDraft;
  onChange: (value: ChoiceSetDraft) => void;
}) {
  const group = useId();
  const focus = useRef<string | null>(null);
  const inputs = useRef(new Map<string, HTMLTextAreaElement>());
  useLayoutEffect(() => {
    if (focus.current) inputs.current.get(focus.current)?.focus();
    focus.current = null;
  }, [value]);
  const multiple = value.selectionMode === 'multiple';
  return (
    <fieldset>
      <legend>الاختيارات والإجابات الصحيحة</legend>
      <label>
        عدد الإجابات الصحيحة
        <select
          value={value.selectionMode}
          onChange={(event) => {
            const selectionMode = event.target.value === 'multiple' ? 'multiple' : 'single';
            onChange({
              ...value,
              selectionMode,
              correctChoiceKeys:
                selectionMode === 'single'
                  ? value.correctChoiceKeys.slice(0, 1)
                  : value.correctChoiceKeys,
            });
          }}
        >
          <option value="single">إجابة واحدة</option>
          <option value="multiple">إجابة واحدة أو أكثر</option>
        </select>
      </label>
      <label>
        <input
          type="checkbox"
          checked={value.shuffleChoices}
          onChange={(event) => onChange({ ...value, shuffleChoices: event.target.checked })}
        />
        خلط ترتيب الاختيارات
      </label>
      {value.choices.map((choice, index) => (
        <div className="choice-editor-option" key={choice.key}>
          <label>
            نص الاختيار {number(index + 1)}
            <textarea
              dir="auto"
              value={choice.text}
              required
              ref={(node) => {
                if (node) inputs.current.set(choice.key, node);
                else inputs.current.delete(choice.key);
              }}
              onChange={(event) =>
                onChange({
                  ...value,
                  choices: value.choices.map((c) =>
                    c.key === choice.key ? { ...c, text: event.target.value } : c,
                  ),
                })
              }
            />
          </label>
          <label>
            <input
              type={multiple ? 'checkbox' : 'radio'}
              name={group}
              checked={value.correctChoiceKeys.includes(choice.key)}
              onChange={(event) =>
                onChange({
                  ...value,
                  correctChoiceKeys: multiple
                    ? event.target.checked
                      ? [...value.correctChoiceKeys, choice.key]
                      : value.correctChoiceKeys.filter((key) => key !== choice.key)
                    : [choice.key],
                })
              }
            />
            الاختيار {number(index + 1)} إجابة صحيحة
          </label>
          <button
            type="button"
            disabled={value.choices.length <= 2}
            onClick={() => {
              const choices = value.choices.filter((c) => c.key !== choice.key);
              focus.current = choices[Math.min(index, choices.length - 1)].key;
              onChange({
                ...value,
                choices,
                correctChoiceKeys: value.correctChoiceKeys.filter((key) => key !== choice.key),
              });
            }}
          >
            حذف الاختيار {number(index + 1)}
          </button>
        </div>
      ))}
      <button
        type="button"
        disabled={value.choices.length >= 128}
        onClick={() => {
          const choice = newChoice();
          focus.current = choice.key;
          onChange({ ...value, choices: [...value.choices, choice] });
        }}
      >
        إضافة اختيار
      </button>
      <p className="muted">من اختيارين إلى ١٢٨ اختيارًا. حد النص ١٦ كيلوبايت لكل اختيار.</p>
    </fieldset>
  );
}
