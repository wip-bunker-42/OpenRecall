import { useEffect, useState } from 'react';
import type { Api } from '../../shared/api';
import type { Navigate } from '../../app/navigation';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import type { ProfileDetail } from '../../shared/generated/ProfileDetail';
import { Field } from '../../components/Field';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { numeric } from '../../shared/forms';
import { AssignmentDialog } from '../profiles/AssignmentDialog';
import { useUnsaved } from '../../app/unsaved';
export function SchedulingSettings({
  deck,
  api,
  onSaved,
  navigate,
  onDirtyChange,
  disabled = false,
}: {
  deck: DeckDetail;
  api: Api;
  onSaved: () => void | Promise<void>;
  navigate: Navigate;
  onDirtyChange?: (value: boolean) => void;
  disabled?: boolean;
}) {
  const [profiles, setProfiles] = useState<ProfileDetail[]>([]);
  const [selected, setSelected] = useState(deck.profileId);
  const [confirm, setConfirm] = useState<ProfileDetail | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const [hasDraft, setHasDraft] = useState(false);
  const [resetKey, setResetKey] = useState(0);
  const { setDirty } = useUnsaved();
  useEffect(() => {
    onDirtyChange?.(hasDraft);
    return () => onDirtyChange?.(false);
  }, [hasDraft, onDirtyChange]);
  useEffect(() => {
    let mounted = true;
    void api
      .call('listProfiles', {})
      .then((p) => {
        if (mounted) setProfiles(p);
      })
      .catch((e) => {
        if (mounted) setError(e);
      });
    return () => {
      mounted = false;
      setDirty(false);
    };
  }, [api, setDirty]);
  async function save(data: FormData) {
    if (disabled || busy) return;
    setError(null);
    try {
      const value = numeric(data, 'desiredRetention', 70, 99, false, true);
      const input = {
        deckId: deck.id,
        desiredRetention: value === null ? null : value / 100,
        minIntervalSeconds: numeric(data, 'minIntervalSeconds', 10, 3600, true, true),
        maxIntervalDays: numeric(data, 'maxIntervalDays', 1, 36500, true, true),
        expectedRevision: deck.revision,
      };
      setBusy(true);
      await api.call('saveDeckPolicy', input);
      setDirty(false);
      await onSaved();
      setHasDraft(false);
      setMessage('حُفظت إعدادات المجموعة.');
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function assign() {
    if (disabled || busy) return;
    setBusy(true);
    setError(null);
    try {
      setConfirm(await api.call('getProfile', { profileId: selected }));
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <FormErrors error={error} />
      <StatusRegion message={message} />
      <form
        key={`${deck.revision}-${resetKey}`}
        noValidate
        onInput={() => {
          if (disabled || busy) return;
          setDirty(true);
          setHasDraft(true);
        }}
        onSubmit={(e) => {
          e.preventDefault();
          void save(new FormData(e.currentTarget));
        }}
      >
        <fieldset disabled={busy || disabled}>
          <legend>حدود هذه المجموعة</legend>
          <p>
            اترك الحقول فارغة لاستخدام الإعدادات العامة. لا تغيّر هذه الحقول أوزان نموذج التحسين
            المشترك.
          </p>
          <Field
            name="desiredRetention"
            label="الاحتفاظ لهذه المجموعة (٪)"
            type="number"
            min={70}
            max={99}
            step="any"
            defaultValue={deck.desiredRetention === null ? '' : deck.desiredRetention * 100}
            error={error}
          />
          <Field
            name="minIntervalSeconds"
            label="أقل فاصل لهذه المجموعة (ثانية)"
            type="number"
            min={10}
            max={3600}
            step={1}
            defaultValue={deck.minIntervalSeconds ?? ''}
            error={error}
          />
          <Field
            name="maxIntervalDays"
            label="أقصى فاصل لهذه المجموعة (يوم)"
            type="number"
            min={1}
            max={36500}
            step={1}
            defaultValue={deck.maxIntervalDays ?? ''}
            error={error}
          />
          <button className="primary" type="submit">
            حفظ إعدادات المجموعة
          </button>
        </fieldset>
      </form>
      {hasDraft && (
        <button
          disabled={busy || disabled}
          onClick={() => {
            setResetKey((n) => n + 1);
            setDirty(false);
            setHasDraft(false);
            setError(null);
          }}
        >
          تجاهل تعديلات المجموعة
        </button>
      )}
      <h3>نموذج التحسين المرتبط</h3>
      <p>{profiles.find((p) => p.id === deck.profileId)?.name ?? 'جارٍ تحميل النموذج…'}</p>
      <label>
        نموذج التحسين
        <select
          disabled={busy || disabled || hasDraft}
          value={selected}
          onChange={(e) => setSelected(e.target.value)}
        >
          {profiles.map((p) => (
            <option value={p.id} key={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </label>
      <div className="actions">
        <button
          disabled={busy || disabled || hasDraft || selected === deck.profileId}
          onClick={() => void assign()}
        >
          تغيير نموذج المجموعة
        </button>
        <button onClick={() => navigate({ page: 'profiles' })}>إدارة نماذج التحسين</button>
      </div>
      {confirm && (
        <AssignmentDialog
          deck={deck}
          profile={confirm}
          api={api}
          onCancel={() => setConfirm(null)}
          onSaved={async () => {
            setConfirm(null);
            await onSaved();
            setMessage('رُبطت المجموعة بالنموذج.');
          }}
        />
      )}
    </>
  );
}
