import { useCallback, useEffect, useRef, useState } from 'react';
import type { Api } from '../../shared/api';
import type { Navigate } from '../../app/navigation';
import type { ProfileDetail } from '../../shared/generated/ProfileDetail';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import { ProfileEditor } from './ProfileEditor';
import { Dialog } from '../../components/Dialog';
import { Field } from '../../components/Field';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { number } from '../../shared/format';
function ProfileRow({
  profile,
  decks,
  api,
  onChanged,
}: {
  profile: ProfileDetail;
  decks: DeckDetail[];
  api: Api;
  onChanged: () => Promise<void>;
}) {
  const [open, setOpen] = useState(false);
  const summary = useRef<HTMLElement>(null);
  return (
    <details
      className="disclosure"
      onToggle={(e) => {
        if (e.currentTarget.open) setOpen(true);
      }}
    >
      <summary ref={summary}>
        <h2>{profile.name}</h2>
      </summary>
      <div className="disclosure-body">
        {open && (
          <ProfileEditor
            key={profile.id}
            profile={profile}
            decks={decks}
            api={api}
            onChanged={async () => {
              await onChanged();
              summary.current?.focus();
            }}
          />
        )}
      </div>
    </details>
  );
}
export function ProfilesPage({ api, navigate }: { api: Api; navigate: Navigate }) {
  const [profiles, setProfiles] = useState<ProfileDetail[]>([]);
  const [decks, setDecks] = useState<DeckDetail[]>([]);
  const [error, setError] = useState<unknown>(null);
  const [creating, setCreating] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const load = useCallback(async () => {
    const [profiles, decks] = await Promise.all([
      api.call('listProfiles', {}),
      api.call('listDecks', {}),
    ]);
    setProfiles(profiles);
    setDecks(decks);
  }, [api]);
  useEffect(() => {
    void load().catch(setError);
  }, [load]);
  async function create(name: string) {
    setBusy(true);
    setError(null);
    try {
      await api.call('createProfile', { name });
      await load();
      setCreating(false);
      setMessage('أُنشئ نموذج مستقل بأوزان المحرك الافتراضية.');
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  void navigate;
  return (
    <>
      <div className="page-heading">
        <h1 tabIndex={-1}>نماذج التحسين</h1>
        <button
          className="primary"
          onClick={() => {
            setError(null);
            setCreating(true);
          }}
        >
          إنشاء نموذج تحسين
        </button>
      </div>
      <p>
        كل مجموعة تبدأ بنموذج مستقل. اربط المجموعات صراحةً لمشاركة أوزان وتكوين واحد؛ عند الفصل
        تحتفظ المجموعة بالنموذج الحالي ويصبح تدريبها مستقلًا.
      </p>
      <StatusRegion message={message} />
      {!creating && <FormErrors error={error} />}
      <p className="muted">{number(profiles.length)} نموذج</p>
      {profiles.map((p) => (
        <ProfileRow key={p.id} profile={p} decks={decks} api={api} onChanged={load} />
      ))}
      {profiles.length === 0 && <p>لا توجد نماذج بعد. أنشئ نموذجًا أو مجموعة للبدء.</p>}
      {error && (
        <button
          onClick={() =>
            void load()
              .then(() => setError(null))
              .catch(setError)
          }
        >
          تحديث النماذج
        </button>
      )}
      {creating && (
        <Dialog
          title="إنشاء نموذج تحسين"
          onDismiss={() => {
            if (!busy) setCreating(false);
          }}
        >
          <FormErrors error={error} />
          <form
            noValidate
            onSubmit={(e) => {
              e.preventDefault();
              void create(String(new FormData(e.currentTarget).get('name')));
            }}
          >
            <Field
              name="name"
              label="اسم نموذج التحسين"
              required
              maxLength={256}
              dir="auto"
              error={error}
            />
            <div className="actions">
              <button className="primary" disabled={busy} type="submit">
                إنشاء النموذج
              </button>
              <button type="button" disabled={busy} onClick={() => setCreating(false)}>
                إلغاء
              </button>
            </div>
          </form>
        </Dialog>
      )}
    </>
  );
}
