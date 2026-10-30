import { useEffect, useState } from 'react';
import type { Api } from '../../shared/api';
import type { SpellingConfig } from '../../shared/generated/SpellingConfig';
import type { SpellingExclusion } from '../../shared/generated/SpellingExclusion';
import type { Page } from '../../shared/generated/Page';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { number } from '../../shared/format';
import { useMountedView } from '../../shared/useMountedView';
export function ExcludedWords({
  api,
  config,
  onChanged,
}: {
  api: Api;
  config: SpellingConfig;
  onChanged: () => Promise<void>;
}) {
  const [page, setPage] = useState(1);
  const [reload, setReload] = useState(0);
  const [result, setResult] = useState<Page<SpellingExclusion> | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const mounted = useMountedView();
  useEffect(() => {
    let current = true;
    setBusy(true);
    setError(null);
    void api
      .call('listSpellingExclusions', { deckId: config.deckId, page, pageSize: 20 })
      .then((next) => {
        if (!current) return;
        if (!next.items.length && page > 1) setPage((p) => p - 1);
        else setResult(next);
      })
      .catch((e) => {
        if (current) setError(e);
      })
      .finally(() => {
        if (current) setBusy(false);
      });
    return () => {
      current = false;
    };
  }, [api, config.deckId, config.revision, page, reload]);
  async function allow(canonical: string) {
    setBusy(true);
    setError(null);
    try {
      const result = await api.call('allowSpellingWord', {
        deckId: config.deckId,
        canonical,
        expectedRevision: config.revision,
      });
      if (!mounted.current) return;
      setMessage(
        `أُلغي الاستبعاد. أُضيفت ${number(result.addedWords)} كلمة من المصدر الحالي بتقدم جديد.`,
      );
      await onChanged();
      if (mounted.current) setReload((n) => n + 1);
    } catch (e) {
      if (mounted.current) setError(e);
    } finally {
      if (mounted.current) setBusy(false);
    }
  }
  return (
    <>
      <p>
        السماح مجددًا يجمع الكلمة من المصدر الحالي إذا كانت موجودة فيه، ويبدأ تقدمًا جديدًا لها.
      </p>
      <FormErrors error={error} />
      <StatusRegion message={message} />
      {error && (
        <button disabled={busy} onClick={() => setReload((n) => n + 1)}>
          تحديث الكلمات المستبعدة
        </button>
      )}
      <div aria-busy={busy}>
        {result?.total === 0 ? (
          <p>لا توجد كلمات مستبعدة.</p>
        ) : (
          <ul>
            {result?.items.map((w) => (
              <li key={w.canonical}>
                <bdi lang="en" dir="ltr">
                  {w.canonical}
                </bdi>{' '}
                <button
                  disabled={busy}
                  aria-label={`السماح بجمع ${w.canonical} مجددًا`}
                  onClick={() => void allow(w.canonical)}
                >
                  السماح مجددًا
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
      {result && result.total > 20 && (
        <div className="actions" aria-label="صفحات الكلمات المستبعدة">
          <button disabled={busy || page <= 1} onClick={() => setPage((p) => p - 1)}>
            السابق
          </button>
          <p>الصفحة {number(page)}</p>
          <button
            disabled={busy || page * 20 >= result.total}
            onClick={() => setPage((p) => p + 1)}
          >
            التالي
          </button>
        </div>
      )}
    </>
  );
}
