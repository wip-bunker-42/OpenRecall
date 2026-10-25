import { useState } from 'react';
import type { ImportPreview as Preview } from '../../shared/generated/ImportPreview';
import type { CommitImportInput } from '../../shared/generated/CommitImportInput';
import type { ImportForm } from '../../shared/generated/ImportForm';
import { number } from '../../shared/format';
import { StatusRegion } from '../../components/StatusRegion';
const reasons: Record<string, string> = {
  required: 'قيمة مطلوبة وغير فارغة',
  stringExpected: 'يلزم نص',
  objectExpected: 'يلزم كائن بطاقة',
  arrayExpected: 'يلزم مصفوفة',
  nestedVariants: 'الصيغ المتداخلة غير مسموحة',
  unknownField: 'حقل غير معروف سيُتجاهل',
  duplicate: 'صيغة مكررة',
};
function FormPreview({ form }: { form: ImportForm }) {
  return (
    <>
      <p className="study-text" dir="auto">
        {form.front}
      </p>
      <details>
        <summary>عرض الإجابة والملاحظات</summary>
        <p className="study-text" dir="auto">
          {form.back}
        </p>
        {form.notes && (
          <p className="study-text" dir="auto">
            {form.notes}
          </p>
        )}
      </details>
      {form.duplicate && (
        <p className="error">
          صيغة مكررة{' '}
          {form.duplicateOf?.scope === 'library' ? 'موجودة بالفعل في مكتبتك' : 'داخل الملف'}. يجب
          استبعادها.
        </p>
      )}
      {form.issues.length > 0 && (
        <ul className="error">
          {form.issues.map((issue, i) => (
            <li key={i}>
              {reasons[issue.code] ?? issue.code} <bdi>{issue.field}</bdi>
            </li>
          ))}
        </ul>
      )}
      {form.warnings.length > 0 && (
        <ul>
          {form.warnings.map((w, i) => (
            <li key={i}>
              {reasons[w.code] ?? w.code} <bdi>{w.field}</bdi>
            </li>
          ))}
        </ul>
      )}
    </>
  );
}
export function ImportPreview({
  preview,
  onCommit,
  onBack,
  onCancel,
  busy,
}: {
  preview: Preview;
  onCommit: (input: CommitImportInput) => void;
  onBack: () => void;
  onCancel: () => void;
  busy: boolean;
}) {
  const [selected, setSelected] = useState(
    () => new Set(preview.cards.filter((c) => c.valid).map((c) => c.key)),
  );
  const [excluded, setExcluded] = useState<Set<string>>(new Set());
  const [page, setPage] = useState(1);
  const size = 20;
  const cards = preview.cards.slice((page - 1) * size, page * size);
  const selectedCards = preview.cards.filter((c) => selected.has(c.key));
  const invalid = selectedCards.some((c) =>
    c.variants.some((v) => (!v.valid || v.duplicate) && !excluded.has(v.key)),
  );
  const forms = selectedCards.reduce(
    (sum, c) => sum + 1 + c.variants.filter((v) => !excluded.has(v.key)).length,
    0,
  );
  function toggle(set: Set<string>, key: string) {
    const next = new Set(set);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    return next;
  }
  return (
    <>
      <p>راجع البطاقات والصيغ قبل الاستيراد. لن تتغير مكتبتك حتى تضغط زر الاستيراد.</p>
      <StatusRegion
        message={`${number(selected.size)} بطاقة محددة و${number(forms)} صيغة. ${number(preview.counts.invalidCards)} بطاقة غير صالحة و${number(preview.counts.duplicateCards)} مكررة في الملف.`}
      />
      {invalid && <p>استبعد الصيغ غير الصالحة والمكررة من البطاقات المحددة للمتابعة.</p>}
      <div className="actions">
        <button
          disabled={busy}
          onClick={() =>
            setSelected(new Set(preview.cards.filter((c) => c.valid).map((c) => c.key)))
          }
        >
          تحديد كل البطاقات الصالحة
        </button>
        <button disabled={busy} onClick={() => setSelected(new Set())}>
          إلغاء تحديد الكل
        </button>
        <button
          disabled={busy}
          onClick={() =>
            setExcluded(
              (s) =>
                new Set([
                  ...s,
                  ...preview.cards.flatMap((c) =>
                    c.variants.filter((v) => !v.valid || v.duplicate).map((v) => v.key),
                  ),
                ]),
            )
          }
        >
          استبعاد كل الصيغ غير الصالحة والمكررة
        </button>
      </div>
      {cards.map((card, i) => {
        const ordinal = number((page - 1) * size + i + 1);
        return (
          <fieldset key={card.key} disabled={busy}>
            <legend>البطاقة {ordinal}</legend>
            <label>
              <input
                type="checkbox"
                checked={selected.has(card.key)}
                disabled={!card.valid || busy}
                onChange={() => setSelected((s) => toggle(s, card.key))}
              />
              استيراد البطاقة {ordinal}
            </label>
            <FormPreview form={card.primary} />
            {card.issues.map((issue, i) => (
              <p className="error" key={i}>
                {reasons[issue.code] ?? issue.code} <bdi>{issue.field}</bdi>
              </p>
            ))}
            {card.variants.map((v, j) => (
              <fieldset key={v.key}>
                <legend>الصيغة البديلة {number(j + 1)}</legend>
                <label>
                  <input
                    type="checkbox"
                    checked={excluded.has(v.key)}
                    onChange={() => setExcluded((s) => toggle(s, v.key))}
                  />
                  استبعاد الصيغة البديلة {number(j + 1)} للبطاقة {ordinal}
                </label>
                <FormPreview form={v} />
              </fieldset>
            ))}
          </fieldset>
        );
      })}
      <div className="actions">
        <button disabled={busy || page === 1} onClick={() => setPage((p) => p - 1)}>
          السابق في المعاينة
        </button>
        <button
          disabled={busy || page * size >= preview.cards.length}
          onClick={() => setPage((p) => p + 1)}
        >
          التالي في المعاينة
        </button>
      </div>
      <div className="actions">
        <button
          className="primary"
          disabled={busy || selected.size === 0 || invalid}
          onClick={() =>
            onCommit({
              previewId: preview.previewId,
              libraryRevision: preview.libraryRevision,
              selectedCardKeys: [...selected],
              excludedFormKeys: [...excluded],
            })
          }
        >
          استيراد البطاقات المحددة
        </button>
        <button disabled={busy} onClick={onBack}>
          العودة إلى JSON
        </button>
        <button disabled={busy} onClick={onCancel}>
          إلغاء
        </button>
      </div>
    </>
  );
}
