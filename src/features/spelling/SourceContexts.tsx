import { useEffect, useState } from 'react';
import type { Api } from '../../shared/api';
import type { SpellingSourceView } from '../../shared/generated/SpellingSourceView';
import type { Page } from '../../shared/generated/Page';
import { FormErrors } from '../../components/FormErrors';
import { number, dateTime } from '../../shared/format';
export function SourceContexts({ api, cardId }: { api: Api; cardId: string }) {
  const [opened, setOpened] = useState(false);
  const [page, setPage] = useState(1);
  const [reload, setReload] = useState(0);
  const [result, setResult] = useState<Page<SpellingSourceView> | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  useEffect(() => {
    if (!opened) return;
    let current = true;
    setBusy(true);
    setError(null);
    setResult(null);
    void api
      .call('listSpellingSources', { cardId, page, pageSize: 20 })
      .then((next) => {
        if (current) setResult(next);
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
  }, [api, cardId, opened, page, reload]);
  return (
    <details
      className="disclosure"
      onToggle={(e) => {
        if (e.currentTarget.open) setOpened(true);
      }}
    >
      <summary>عرض البطاقة الأصلية</summary>
      <div className="disclosure-body">
        <FormErrors error={error} />
        {Boolean(error) && (
          <button disabled={busy} onClick={() => setReload((n) => n + 1)}>
            إعادة تحميل السياقات
          </button>
        )}
        {busy && <p role="status">جارٍ تحميل السياقات…</p>}
        {result && (
          <>
            <p>
              {number(result.total)} سياق محفوظ، الصفحة {number(page)}.
            </p>
            {result.items.length === 0 && <p>لا توجد سياقات محفوظة لهذه الكلمة.</p>}
            {result.items.map((context, index) => {
              const s = context.snapshot;
              return (
                <section className="source-context" key={s.id}>
                  <h2>السياق {number((page - 1) * 20 + index + 1)}</h2>
                  <p>
                    نسخة محفوظة من «<bdi>{s.sourceDeckName}</bdi>»، الصيغة{' '}
                    {number(s.sourceFormIndex + 1)}. تاريخ الحفظ: {dateTime(s.capturedAt)}.
                  </p>
                  <p>
                    {context.availability === 'deleted'
                      ? 'حُذفت البطاقة الأصلية؛ هذا آخر سياق محفوظ للكلمة.'
                      : context.availability === 'changed'
                        ? 'تغيرت البطاقة الأصلية؛ يعرض هنا آخر سياق احتوى هذه الكلمة.'
                        : 'هذا السياق يطابق المحتوى الحالي للبطاقة الأصلية.'}
                  </p>
                  <h3 dir="auto">{s.questionTitle ?? 'السؤال'}</h3>
                  <p className="study-text" dir="auto" lang={s.language ?? undefined}>
                    {s.front}
                  </p>
                  <h3>الإجابة</h3>
                  <p className="study-text" dir="auto" lang={s.language ?? undefined}>
                    {s.back}
                  </p>
                  {s.notes && (
                    <>
                      <h3>الملاحظات</h3>
                      <p className="study-text" dir="auto" lang={s.language ?? undefined}>
                        {s.notes}
                      </p>
                    </>
                  )}
                </section>
              );
            })}
            {result.total > 20 && (
              <div className="actions" aria-label="صفحات السياقات">
                <button disabled={busy || page <= 1} onClick={() => setPage((p) => p - 1)}>
                  السياقات السابقة
                </button>
                <button
                  disabled={busy || page * 20 >= result.total}
                  onClick={() => setPage((p) => p + 1)}
                >
                  السياقات التالية
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </details>
  );
}
