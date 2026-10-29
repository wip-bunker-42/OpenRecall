import { useEffect, useId, useRef, useState } from 'react';
import type { Api } from '../../shared/api';
import type { CardDetail } from '../../shared/generated/CardDetail';
import type { FormDraft } from '../../shared/generated/FormDraft';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { number } from '../../shared/format';
import { useUnsaved } from '../../app/unsaved';
type EditForm = FormDraft & { key: string };
const empty = (): EditForm => ({
  questionTitle: null,
  key: crypto.randomUUID(),
  id: null,
  front: '',
  back: '',
  notes: '',
  language: null,
});
export function CardEditor({
  card,
  deckId,
  api,
  onSaved,
  onCancel,
}: {
  card: CardDetail | null;
  deckId: string;
  api: Api;
  onSaved: (card: CardDetail) => void;
  onCancel: () => void;
}) {
  const [initial] = useState<EditForm[]>(() =>
    card ? card.forms.map((f) => ({ ...f, key: f.id })) : [empty()],
  );
  const [forms, setForms] = useState(initial);
  const [error, setError] = useState<unknown>(null);
  const [attempted, setAttempted] = useState(false);
  const [busy, setBusy] = useState(false);
  const [discard, setDiscard] = useState(false);
  const [focusKey, setFocusKey] = useState<string | null>(null);
  const inputs = useRef(new Map<string, HTMLTextAreaElement>());
  const id = useId();
  const { setDirty } = useUnsaved();
  const dirty = JSON.stringify(forms) !== JSON.stringify(initial);
  useEffect(() => {
    setDirty(dirty);
    return () => setDirty(false);
  }, [dirty, setDirty]);
  useEffect(() => {
    if (focusKey) {
      inputs.current.get(focusKey)?.focus();
      setFocusKey(null);
    }
  }, [focusKey, forms]);
  function change(key: string, field: keyof FormDraft, value: string) {
    setForms((fs) =>
      fs.map((f) =>
        f.key === key
          ? {
              ...f,
              [field]: field === 'language' || field === 'questionTitle' ? value || null : value,
            }
          : f,
      ),
    );
  }
  function cancel() {
    if (busy) return;
    if (dirty) setDiscard(true);
    else onCancel();
  }
  async function save() {
    setAttempted(true);
    if (forms.some((f) => !f.front.trim() || !f.back.trim())) {
      setError({ code: 'requiredCardFields' });
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const draft = {
        forms: forms.map(({ key, ...form }) => {
          void key;
          return form;
        }),
      };
      const saved = card
        ? await api.call('updateCard', { cardId: card.id, draft, expectedRevision: card.revision })
        : await api.call('createCard', { deckId, draft });
      setDirty(false);
      onSaved(saved);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <Dialog title={card ? 'تعديل البطاقة' : 'إنشاء بطاقة'} onDismiss={cancel}>
        <p>
          كل الصيغ تشترك في حالة تعلم وموعد مراجعة واحد. يظهر بديل عشوائي دون تكرار الصيغ داخل
          الدورة.
        </p>
        <form
          noValidate
          onSubmit={(e) => {
            e.preventDefault();
            void save();
          }}
        >
          <FormErrors error={error} />
          {forms.map((form, i) => (
            <fieldset key={form.key} disabled={busy}>
              <legend>{i === 0 ? 'الصيغة الأساسية' : `صيغة بديلة ${number(i)}`}</legend>
              <label htmlFor={`${id}-${form.key}-front`}>السؤال، الصيغة {number(i + 1)}</label>
              <textarea
                id={`${id}-${form.key}-front`}
                ref={(node) => {
                  if (node) inputs.current.set(form.key, node);
                  else inputs.current.delete(form.key);
                }}
                required
                aria-invalid={(attempted && !form.front.trim()) || undefined}
                value={form.front}
                onChange={(e) => change(form.key, 'front', e.target.value)}
                dir="auto"
                lang={form.language ?? undefined}
              />
              <label>
                الإجابة، الصيغة {number(i + 1)}
                <textarea
                  required
                  aria-invalid={(attempted && !form.back.trim()) || undefined}
                  value={form.back}
                  onChange={(e) => change(form.key, 'back', e.target.value)}
                  dir="auto"
                  lang={form.language ?? undefined}
                />
              </label>
              <label>
                الملاحظات، الصيغة {number(i + 1)}
                <textarea
                  value={form.notes}
                  onChange={(e) => change(form.key, 'notes', e.target.value)}
                  dir="auto"
                  lang={form.language ?? undefined}
                />
              </label>
              <label>
                عنوان السؤال، الصيغة {number(i + 1)}
                <input
                  value={form.questionTitle ?? ''}
                  onChange={(e) => change(form.key, 'questionTitle', e.target.value)}
                  aria-describedby={`${id}-${form.key}-title-help`}
                  dir="auto"
                />
              </label>
              <p id={`${id}-${form.key}-title-help`} className="muted">
                اختياري. اتركه فارغًا لاستخدام «السؤال». حتى ١٦٠ حرفًا في سطر واحد، مثل «ترجم» أو
                «أكمل».
              </p>
              <label>
                لغة الصيغة {number(i + 1)} (اختياري، مثل ar أو en)
                <input
                  value={form.language ?? ''}
                  onChange={(e) => change(form.key, 'language', e.target.value)}
                  dir="ltr"
                />
              </label>
              {forms.length > 1 && (
                <button
                  type="button"
                  className="danger"
                  onClick={() => {
                    const next = forms.filter((f) => f.key !== form.key);
                    setForms(next);
                    setFocusKey(next[Math.max(0, i - 1)].key);
                  }}
                  aria-label={`حذف الصيغة ${number(i + 1)}`}
                >
                  حذف الصيغة
                </button>
              )}
            </fieldset>
          ))}
          <button
            type="button"
            disabled={busy}
            onClick={() => {
              const form = empty();
              setForms((fs) => [...fs, form]);
              setFocusKey(form.key);
            }}
          >
            إضافة صيغة بديلة
          </button>
          <div className="actions">
            <button className="primary" disabled={busy} type="submit">
              حفظ البطاقة
            </button>
            <button type="button" disabled={busy} onClick={cancel}>
              إلغاء
            </button>
          </div>
        </form>
      </Dialog>
      {discard && (
        <Dialog title="تغييرات غير محفوظة" onDismiss={() => setDiscard(false)}>
          <p>لم تُحفظ تعديلات البطاقة. هل تريد تجاهلها؟</p>
          <div className="actions">
            <button data-autofocus onClick={() => setDiscard(false)}>
              متابعة التعديل
            </button>
            <button
              className="danger"
              onClick={() => {
                setDirty(false);
                onCancel();
              }}
            >
              تجاهل التغييرات
            </button>
          </div>
        </Dialog>
      )}
    </>
  );
}
