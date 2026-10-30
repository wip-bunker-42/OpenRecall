import { useCallback, useEffect, useRef, useState } from 'react';
import type { Api } from '../../shared/api';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import type { SpellingConfig } from '../../shared/generated/SpellingConfig';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { useUnsaved } from '../../app/unsaved';
import { useMountedView } from '../../shared/useMountedView';
import { number } from '../../shared/format';
import { ExcludedWords } from './ExcludedWords';

export function SpellingSettings({
  api,
  deck,
  onChanged,
}: {
  api: Api;
  deck: DeckDetail;
  onChanged: () => Promise<void>;
}) {
  const [config, setConfig] = useState<SpellingConfig | null>(null);
  const [sources, setSources] = useState<DeckDetail[]>([]);
  const [source, setSource] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [message, setMessage] = useState('');
  const [draft, setDraft] = useState(false);
  const [excludedOpen, setExcludedOpen] = useState(false);
  const dirty = useRef(false);
  const readVersion = useRef(0);
  const mounted = useMountedView();
  const { setDirty } = useUnsaved();
  const load = useCallback(async () => {
    const version = ++readVersion.current;
    try {
      const [next, decks] = await Promise.all([
        api.call('getSpellingConfig', { deckId: deck.id }),
        api.call('listDecks', {}),
      ]);
      if (!mounted.current || version !== readVersion.current) return;
      setSources(decks.filter((d) => d.kind === 'standard'));
      if (!dirty.current) {
        setConfig(next);
        setSource(next.sourceDeckId ?? '');
      }
    } catch (e) {
      if (mounted.current && version === readVersion.current) throw e;
    }
  }, [api, deck.id, mounted]);
  useEffect(() => {
    void load().catch((e) => {
      if (mounted.current) setError(e);
    });
  }, [load, deck.dataRevision, mounted]);
  useEffect(() => () => setDirty(false), [setDirty]);
  async function changed() {
    await onChanged();
    await load();
  }
  async function save(rescan: boolean) {
    if (!config) return;
    setBusy(true);
    setError(null);
    ++readVersion.current;
    try {
      const input = { deckId: deck.id, expectedRevision: config.revision };
      const result = rescan
        ? await api.call('rescanSpellingSource', input)
        : await api.call('setSpellingSource', { ...input, sourceDeckId: source });
      if (!mounted.current) return;
      ++readVersion.current;
      dirty.current = false;
      setDraft(false);
      setDirty(false);
      setConfig(result.config);
      setSource(result.config.sourceDeckId ?? '');
      setMessage(`أُضيفت ${number(result.addedWords)} كلمة. الكلمات السابقة وتقدمها محفوظان.`);
      await changed();
    } catch (e) {
      if (mounted.current) setError(e);
    } finally {
      if (mounted.current) setBusy(false);
    }
  }
  return (
    <>
      <FormErrors error={error} />
      <StatusRegion message={message} />
      {Boolean(error) && !config && (
        <button
          disabled={busy}
          onClick={() => {
            setError(null);
            void load().catch(setError);
          }}
        >
          تحديث إعدادات المصدر
        </button>
      )}
      {config ? (
        <>
          {config.sourceDeckId ? (
            <p>
              المصدر الحالي: <bdi>{config.sourceDeckName}</bdi>.
            </p>
          ) : (
            <p>
              حُذفت المجموعة المصدر «{config.sourceDeckName}». كلمات الكتابة وتقدمها وآخر سياقاتها
              محفوظة. اختر مصدرًا جديدًا لاستقبال كلمات جديدة.
            </p>
          )}
          <p>
            تُجمع الكلمات تلقائيًا دون تكرار. تغيير المصدر أو حذفه لا يحذف الكلمات السابقة أو يعيد
            تقدمها. حذف كلمة من هذه المجموعة يستبعدها حتى تسمح بجمعها مجددًا.
          </p>
          <details>
            <summary>كيف تُجمع الكلمات؟</summary>
            <p>
              تُجمع الكلمات والمصطلحات المكتوبة بحروف لاتينية، بما فيها الأسماء والكلمات غير
              المعروفة. قد تظهر كلمات من لغات أخرى تستخدم هذه الحروف؛ احذف ما لا تريد التدرب عليه
              لاستبعاده. تُتجاهل الروابط والبريد الإلكتروني والأرقام وحدها، وتظل الكلمات ذات
              التصريفات المختلفة بطاقات مستقلة.
            </p>
          </details>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void save(false);
            }}
          >
            <label>
              المجموعة المصدر
              <select
                value={source}
                disabled={busy}
                onChange={(e) => {
                  setSource(e.target.value);
                  dirty.current = true;
                  setDraft(true);
                  setDirty(true);
                }}
              >
                <option value="">اختر مجموعة</option>
                {sources.map((d) => (
                  <option key={d.id} value={d.id}>
                    {d.name}
                  </option>
                ))}
              </select>
            </label>
            <div className="actions">
              <button type="submit" disabled={busy || !source || !draft}>
                حفظ المصدر
              </button>
              <button
                type="button"
                disabled={busy || draft || !config.sourceDeckId}
                onClick={() => void save(true)}
              >
                فحص المصدر مجددًا
              </button>
              {(draft || Boolean(error)) && (
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => {
                    dirty.current = false;
                    setDraft(false);
                    setDirty(false);
                    setError(null);
                    void load().catch(setError);
                  }}
                >
                  تجاهل الاختيار وتحديث المصدر
                </button>
              )}
            </div>
          </form>
          <details
            className="disclosure"
            onToggle={(e) => {
              if (e.currentTarget.open) setExcludedOpen(true);
            }}
          >
            <summary>
              <h3>الكلمات المستبعدة</h3>
            </summary>
            <div className="disclosure-body">
              {excludedOpen && <ExcludedWords api={api} config={config} onChanged={changed} />}
            </div>
          </details>
        </>
      ) : (
        <p role="status">جارٍ تحميل إعدادات المصدر…</p>
      )}
    </>
  );
}
