import { useCallback, useEffect, useState } from 'react';
import type { Api, NativeApi } from '../../shared/api';
import type { Navigate } from '../../app/navigation';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import type { CardDetail } from '../../shared/generated/CardDetail';
import { CardEditor } from './CardEditor';
import { CardList } from './CardList';
import { ImportDialog } from '../import/ImportDialog';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { focusHeading } from '../../shared/focus';
import { number } from '../../shared/format';
import { DeckStatistics } from '../statistics/DeckStatistics';
import { SchedulingSettings } from '../settings/SchedulingSettings';
import { useMountedView } from '../../shared/useMountedView';
export function DeckPage({
  deckId,
  api,
  native,
  navigate,
  pageSize = 20,
  timezone = 'Africa/Cairo',
}: {
  deckId: string;
  api: Api;
  native: NativeApi;
  navigate: Navigate;
  pageSize?: number;
  timezone?: string;
}) {
  const [deck, setDeck] = useState<DeckDetail | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [editor, setEditor] = useState<{ card: CardDetail | null } | null>(null);
  const [importing, setImporting] = useState(false);
  const [deleting, setDeleting] = useState<CardDetail | null>(null);
  const [busy, setBusy] = useState(false);
  const [open, setOpen] = useState(false);
  const [revision, setRevision] = useState(0);
  const [message, setMessage] = useState('');
  const [statisticsOpen, setStatisticsOpen] = useState(false);
  const [schedulingOpen, setSchedulingOpen] = useState(false);
  const [policyDirty, setPolicyDirty] = useState(false);
  const mounted = useMountedView();
  const refresh = useCallback(async () => {
    setDeck(await api.call('getDeck', { deckId }));
    setRevision((n) => n + 1);
  }, [api, deckId]);
  useEffect(() => {
    void refresh()
      .then(() => focusHeading())
      .catch(setError);
  }, [refresh]);
  async function start() {
    setBusy(true);
    setError(null);
    try {
      const view = await api.call('startSession', { deckId });
      if (!mounted.current) return;
      navigate({ page: 'review', sessionId: view.payload.sessionId, deckId });
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function remove() {
    if (!deleting) return;
    setBusy(true);
    setError(null);
    try {
      await api.call('deleteCard', { cardId: deleting.id, expectedRevision: deleting.revision });
      setDeleting(null);
      await refresh();
      setMessage('حُذفت البطاقة وسجل مراجعاتها.');
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  if (!deck)
    return (
      <>
        <h1 tabIndex={-1}>المجموعة</h1>
        <FormErrors error={error} />
        <p role="status">جارٍ تحميل المجموعة…</p>
        <button onClick={() => void refresh().catch(setError)}>إعادة المحاولة</button>
      </>
    );
  return (
    <>
      <button onClick={() => navigate({ page: 'library' })}>العودة إلى المجموعات</button>
      <div className="page-heading">
        <div>
          <h1 tabIndex={-1} dir="auto">
            {deck.name}
          </h1>
          <p className="muted">
            {number(deck.logicalCards)} بطاقة، {number(deck.dueNow)} مستحقة الآن،{' '}
            {number(deck.newCards)} جديدة.
          </p>
        </div>
        <button className="primary" disabled={busy || policyDirty} onClick={() => void start()}>
          بدء المراجعة
        </button>
      </div>
      <div className="actions">
        <button disabled={policyDirty} onClick={() => setEditor({ card: null })}>
          إنشاء بطاقة
        </button>
        <button disabled={policyDirty} onClick={() => setImporting(true)}>
          استيراد بطاقات JSON
        </button>
      </div>
      <StatusRegion message={message} />
      {policyDirty && (
        <p>احفظ تعديلات الجدولة والتحسين أو تجاهلها قبل تعديل البطاقات أو بدء المراجعة.</p>
      )}
      {!deleting && <FormErrors error={error} />}
      {error && (
        <button
          onClick={() =>
            void refresh()
              .then(() => setError(null))
              .catch(setError)
          }
        >
          تحديث المجموعة
        </button>
      )}
      <details className="disclosure" onToggle={(e) => setOpen(e.currentTarget.open)}>
        <summary>
          <h2>البطاقات</h2>
        </summary>
        <div className="disclosure-body">
          {open && (
            <CardList
              deckId={deckId}
              api={api}
              pageSize={pageSize}
              onEdit={(card) => setEditor({ card })}
              onDelete={setDeleting}
              revision={revision}
              timezone={timezone}
              editingDisabled={policyDirty}
            />
          )}
        </div>
      </details>
      <details
        className="disclosure"
        onToggle={(e) => {
          if (e.currentTarget.open) setStatisticsOpen(true);
        }}
      >
        <summary>
          <h2>إحصاءات المجموعة</h2>
        </summary>
        <div className="disclosure-body">
          {statisticsOpen && (
            <DeckStatistics
              deckId={deckId}
              api={api}
              timezone={timezone}
              revision={deck.revision}
            />
          )}
        </div>
      </details>
      <details
        className="disclosure"
        onToggle={(e) => {
          if (e.currentTarget.open) setSchedulingOpen(true);
        }}
      >
        <summary>
          <h2>الجدولة والتحسين</h2>
        </summary>
        <div className="disclosure-body">
          {schedulingOpen && (
            <SchedulingSettings
              key={deck.revision}
              deck={deck}
              api={api}
              navigate={navigate}
              onSaved={refresh}
              onDirtyChange={setPolicyDirty}
            />
          )}
        </div>
      </details>
      {editor && (
        <CardEditor
          card={editor.card}
          deckId={deckId}
          api={api}
          onCancel={() => setEditor(null)}
          onSaved={() => {
            setEditor(null);
            setMessage('حُفظت البطاقة.');
            void refresh().catch(setError);
          }}
        />
      )}
      {importing && (
        <ImportDialog
          deckId={deckId}
          api={api}
          native={native}
          onCancel={() => setImporting(false)}
          onComplete={(report) => {
            setImporting(false);
            setMessage(
              `استُوردت ${number(report.insertedCards)} بطاقة و${number(report.insertedForms)} صيغة. استُبعدت ${number(report.excludedCards)} بطاقة و${number(report.excludedForms)} صيغة.`,
            );
            void refresh().catch(setError);
          }}
        />
      )}
      {deleting && (
        <Dialog
          title="حذف البطاقة"
          onDismiss={() => {
            if (!busy) setDeleting(null);
          }}
        >
          <FormErrors error={error} />
          <p>ستُحذف البطاقة وكل صيغها وسجل مراجعاتها نهائيًا.</p>
          <div className="actions">
            <button data-autofocus disabled={busy} onClick={() => setDeleting(null)}>
              إلغاء
            </button>
            <button className="danger" disabled={busy} onClick={() => void remove()}>
              حذف البطاقة نهائيًا
            </button>
          </div>
        </Dialog>
      )}
    </>
  );
}
