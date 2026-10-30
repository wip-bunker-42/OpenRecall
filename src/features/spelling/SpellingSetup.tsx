import { useEffect, useState } from 'react';
import type { Api } from '../../shared/api';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import type { SpellingDeckResult } from '../../shared/generated/SpellingDeckResult';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { useUnsaved } from '../../app/unsaved';
import { useMountedView } from '../../shared/useMountedView';

export function SpellingSetup({
  api,
  sources,
  onCreated,
  onCancel,
}: {
  api: Api;
  sources: DeckDetail[];
  onCreated: (result: SpellingDeckResult) => void;
  onCancel: () => void;
}) {
  const [source, setSource] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [existing, setExisting] = useState<string | null>(null);
  const { setDirty } = useUnsaved();
  const mounted = useMountedView();
  useEffect(() => () => setDirty(false), [setDirty]);
  async function create() {
    setBusy(true);
    setError(null);
    try {
      const result = await api.call('createSpellingDeck', { sourceDeckId: source });
      if (!mounted.current) return;
      setDirty(false);
      await onCreated(result);
    } catch (e) {
      if (!mounted.current) return;
      setError(e);
      if (
        typeof e === 'object' &&
        e !== null &&
        'code' in e &&
        e.code === 'spellingDeckExists' &&
        'context' in e &&
        typeof e.context === 'string'
      )
        setExisting(e.context);
    } finally {
      if (mounted.current) setBusy(false);
    }
  }
  async function openExisting() {
    if (!existing) return;
    setBusy(true);
    setError(null);
    try {
      const [deck, config] = await Promise.all([
        api.call('getDeck', { deckId: existing }),
        api.call('getSpellingConfig', { deckId: existing }),
      ]);
      if (!mounted.current) return;
      setDirty(false);
      await onCreated({ deck, config, addedWords: 0 });
    } catch (e) {
      if (mounted.current) setError(e);
    } finally {
      if (mounted.current) setBusy(false);
    }
  }
  return (
    <Dialog
      title="إنشاء مجموعة التدرب على الكتابة"
      onDismiss={() => {
        if (!busy) onCancel();
      }}
    >
      <p>
        مجموعة واحدة في مكتبتك، تجمع الكلمات الإنجليزية الفريدة تلقائيًا من أسئلة المصدر وإجاباته
        وملاحظاته وكل صيغه، بما فيها البطاقات السابقة.
      </p>
      <p>
        حذف المصدر أو تغيير محتواه لا يحذف كلمات الكتابة أو تقدمها؛ تحتفظ كل كلمة بآخر سياق محفوظ
        لها.
      </p>
      <FormErrors error={error} />
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void create();
        }}
      >
        <label>
          المجموعة المصدر
          <select
            data-autofocus
            value={source}
            disabled={busy}
            onChange={(e) => {
              setSource(e.target.value);
              setDirty(Boolean(e.target.value));
            }}
          >
            <option value="">اختر مجموعة</option>
            {sources
              .filter((d) => d.kind === 'standard')
              .map((d) => (
                <option key={d.id} value={d.id}>
                  {d.name}
                </option>
              ))}
          </select>
        </label>
        <div className="actions">
          <button type="submit" className="primary" disabled={busy || !source}>
            إنشاء وربط المصدر
          </button>
          {existing && (
            <button type="button" disabled={busy} onClick={() => void openExisting()}>
              فتح المجموعة الموجودة
            </button>
          )}
          <button type="button" disabled={busy} onClick={onCancel}>
            إلغاء
          </button>
        </div>
      </form>
    </Dialog>
  );
}
