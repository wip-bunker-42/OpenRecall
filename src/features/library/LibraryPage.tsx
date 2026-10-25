import { useState } from 'react';
import type { Api } from '../../shared/api';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import type { Bootstrap } from '../../shared/generated/Bootstrap';
import type { Navigate } from '../../app/navigation';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { number } from '../../shared/format';
type Editing = { action: 'create' } | { action: 'rename' | 'delete'; deck: DeckDetail };
export function LibraryPage({
  bootstrap,
  api,
  navigate,
  refresh,
}: {
  bootstrap: Bootstrap;
  api: Api;
  navigate: Navigate;
  refresh: () => Promise<void>;
}) {
  const [editing, setEditing] = useState<Editing | null>(null);
  const [name, setName] = useState('');
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  function edit(value: Editing) {
    setError(null);
    setName(value.action === 'create' ? '' : value.deck.name);
    setEditing(value);
  }
  async function save() {
    if (!editing) return;
    setBusy(true);
    setError(null);
    try {
      if (editing.action === 'create') await api.call('createDeck', { name, profileId: null });
      else if (editing.action === 'rename')
        await api.call('renameDeck', {
          deckId: editing.deck.id,
          name,
          expectedRevision: editing.deck.revision,
        });
      else
        await api.call('deleteDeck', {
          deckId: editing.deck.id,
          expectedRevision: editing.deck.revision,
        });
      await refresh();
      setMessage(
        editing.action === 'delete'
          ? 'حُذفت المجموعة.'
          : editing.action === 'create'
            ? 'أُنشئت المجموعة.'
            : 'حُفظ الاسم.',
      );
      setEditing(null);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function review(deckId: string) {
    setBusy(true);
    setError(null);
    try {
      const view = await api.call('startSession', { deckId });
      navigate({ page: 'review', sessionId: view.payload.sessionId });
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <div className="page-heading">
        <div>
          <h1 tabIndex={-1}>مجموعاتي</h1>
          <p className="muted">مساحة للمذاكرة، على إيقاعك.</p>
        </div>
        <button className="primary" onClick={() => edit({ action: 'create' })}>
          إنشاء مجموعة
        </button>
      </div>
      <StatusRegion message={message} />
      {!editing && <FormErrors error={error} />}
      {bootstrap.resumableSessions.length > 0 && (
        <section aria-label="جلسات غير مكتملة">
          <h2>أكمل من حيث توقفت</h2>
          {bootstrap.resumableSessions.map((s) => (
            <button
              key={s.sessionId}
              onClick={() => navigate({ page: 'review', sessionId: s.sessionId })}
            >
              استكمال المراجعة
            </button>
          ))}
        </section>
      )}
      {bootstrap.decks.length === 0 ? (
        <p className="empty">أنشئ مجموعتك الأولى، ثم أضف بطاقاتك أو استورد ملف JSON.</p>
      ) : (
        <ul className="collection-list">
          {bootstrap.decks.map((d) => (
            <li key={d.id} className="collection">
              <h2>
                <button
                  className="collection-title"
                  onClick={() => navigate({ page: 'deck', deckId: d.id })}
                  dir="auto"
                >
                  {d.name}
                </button>
              </h2>
              <p className="muted">
                {number(d.logicalCards)} بطاقة، {number(d.dueNow)} مستحقة الآن، {number(d.newCards)}{' '}
                جديدة
              </p>
              <div className="actions">
                <button
                  className="primary"
                  disabled={busy}
                  onClick={() => void review(d.id)}
                  aria-label={`بدء مراجعة ${d.name}`}
                >
                  بدء المراجعة
                </button>
                <button
                  onClick={() => edit({ action: 'rename', deck: d })}
                  aria-label={`إعادة تسمية ${d.name}`}
                >
                  إعادة تسمية
                </button>
                <button
                  className="danger"
                  onClick={() => edit({ action: 'delete', deck: d })}
                  aria-label={`حذف ${d.name}`}
                >
                  حذف
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
      {editing && (
        <Dialog
          title={
            editing.action === 'create'
              ? 'إنشاء مجموعة'
              : editing.action === 'rename'
                ? 'إعادة تسمية المجموعة'
                : 'حذف المجموعة'
          }
          onDismiss={() => {
            if (!busy) setEditing(null);
          }}
        >
          <form
            noValidate
            onSubmit={(e) => {
              e.preventDefault();
              void save();
            }}
          >
            <FormErrors error={error} />
            {editing.action === 'delete' ? (
              <p>
                ستُحذف مجموعة «{editing.deck.name}» وبطاقاتها وسجل مراجعاتها نهائيًا. صدّر نسخة
                احتياطية للاحتفاظ بها.
              </p>
            ) : (
              <label>
                اسم المجموعة
                <input
                  data-autofocus
                  required
                  maxLength={256}
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  dir="auto"
                />
              </label>
            )}
            <div className="actions">
              <button
                className={editing.action === 'delete' ? 'danger' : 'primary'}
                disabled={busy}
                type="submit"
              >
                {editing.action === 'create'
                  ? 'إنشاء'
                  : editing.action === 'rename'
                    ? 'حفظ الاسم'
                    : 'حذف المجموعة نهائيًا'}
              </button>
              <button type="button" disabled={busy} onClick={() => setEditing(null)}>
                إلغاء
              </button>
            </div>
          </form>
        </Dialog>
      )}
    </>
  );
}
