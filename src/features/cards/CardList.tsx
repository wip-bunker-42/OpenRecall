import { useEffect, useState } from 'react';
import type { Api } from '../../shared/api';
import type { CardDetail } from '../../shared/generated/CardDetail';
import type { Page } from '../../shared/generated/Page';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { number, dateTime } from '../../shared/format';
const phases: Record<string, string> = {
  new: 'جديدة',
  learning: 'تعلّم',
  relearning: 'إعادة تعلّم',
  review: 'مراجعة',
};
export function CardList({
  deckId,
  api,
  pageSize,
  onEdit,
  onDelete,
  revision,
  timezone = 'Africa/Cairo',
  editingDisabled = false,
}: {
  deckId: string;
  api: Api;
  pageSize: number;
  onEdit: (card: CardDetail) => void;
  onDelete: (card: CardDetail) => void;
  revision: number;
  timezone?: string;
  editingDisabled?: boolean;
}) {
  const [query, setQuery] = useState('');
  const [input, setInput] = useState('');
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<Page<CardDetail> | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [reload, setReload] = useState(0);
  useEffect(() => {
    let current = true;
    setBusy(true);
    setError(null);
    void api
      .call('listCards', { deckId, query, page, pageSize })
      .then((r) => {
        if (current) {
          if (r.items.length === 0 && page > 1) {
            setPage((p) => p - 1);
          } else setResult(r);
        }
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
  }, [api, deckId, query, page, pageSize, revision, reload]);
  return (
    <>
      <form
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          setPage(1);
          setQuery(input);
          setReload((n) => n + 1);
        }}
      >
        <label>
          البحث في البطاقات
          <input
            type="search"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            dir="auto"
          />
        </label>
        <button type="submit">بحث</button>
      </form>
      <FormErrors error={error} />
      {error && <button onClick={() => setReload((n) => n + 1)}>تحديث البطاقات</button>}
      <StatusRegion
        message={
          busy
            ? 'جارٍ تحميل البطاقات…'
            : result
              ? `${number(result.total)} بطاقة في النتائج، الصفحة ${number(page)}.`
              : ''
        }
      />
      <div aria-busy={busy}>
        {result?.items.length === 0 ? (
          <p>لا توجد بطاقات مطابقة. أضف بطاقة أو عدّل البحث.</p>
        ) : (
          <ul className="collection-list">
            {result?.items.map((card, i) => {
              const ordinal = number((page - 1) * pageSize + i + 1);
              const primary = card.forms[0];
              return (
                <li className="collection" key={card.id}>
                  <p className="study-text" dir="auto" lang={primary.language ?? undefined}>
                    {primary.front}
                  </p>
                  <p className="muted">
                    {phases[card.phase]}، {number(card.forms.length)} صيغة. الموعد:{' '}
                    {dateTime(card.dueAt, timezone)}
                  </p>
                  <details>
                    <summary>عرض إجابات وصيغ البطاقة {ordinal}</summary>
                    {card.forms.map((f, index) => (
                      <div className="card-form-preview" key={f.id}>
                        <h3>الصيغة {number(index + 1)}</h3>
                        {index > 0 && (
                          <p className="study-text" dir="auto" lang={f.language ?? undefined}>
                            {f.front}
                          </p>
                        )}
                        <p className="study-text" dir="auto" lang={f.language ?? undefined}>
                          {f.back}
                        </p>
                        {f.notes && (
                          <p className="study-text muted" dir="auto" lang={f.language ?? undefined}>
                            {f.notes}
                          </p>
                        )}
                      </div>
                    ))}
                  </details>
                  <div className="actions">
                    <button
                      disabled={busy || editingDisabled}
                      aria-label={`تعديل البطاقة ${ordinal}`}
                      onClick={() => onEdit(card)}
                    >
                      تعديل
                    </button>
                    <button
                      className="danger"
                      aria-label={`حذف البطاقة ${ordinal}`}
                      disabled={busy || editingDisabled}
                      onClick={() => onDelete(card)}
                    >
                      حذف
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>
      <div className="actions" aria-label="صفحات البطاقات">
        <button disabled={busy || page <= 1} onClick={() => setPage((p) => p - 1)}>
          الصفحة السابقة
        </button>
        <button
          disabled={busy || !result || page * pageSize >= result.total}
          onClick={() => setPage((p) => p + 1)}
        >
          الصفحة التالية
        </button>
      </div>
    </>
  );
}
