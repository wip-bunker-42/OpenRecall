import { useState } from 'react';
import type { Api } from '../../shared/api';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import type { ProfileDetail } from '../../shared/generated/ProfileDetail';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
export function AssignmentDialog({
  deck,
  profile,
  api,
  onSaved,
  onCancel,
}: {
  deck: DeckDetail;
  profile: ProfileDetail;
  api: Api;
  onSaved: () => void | Promise<void>;
  onCancel: () => void;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  async function save() {
    setBusy(true);
    setError(null);
    try {
      await api.call('assignProfile', {
        deckId: deck.id,
        profileId: profile.id,
        expectedDeckRevision: deck.revision,
        expectedProfileRevision: profile.revision,
      });
      await onSaved();
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog
      title="ربط مجموعة بنموذج التحسين"
      onDismiss={() => {
        if (!busy) onCancel();
      }}
    >
      <p>
        ستتبع مجموعة «{deck.name}» نموذج «{profile.name}». التحسينات المستقبلية ستؤثر في كل
        المجموعات المرتبطة بهذا النموذج. تُعاد حسابات الذاكرة بالأوزان المختارة، وتبقى مواعيد
        الاستحقاق الحالية كما هي.
      </p>
      <FormErrors error={error} />
      <div className="actions">
        <button data-autofocus disabled={busy} onClick={onCancel}>
          إلغاء
        </button>
        <button className="primary" disabled={busy} onClick={() => void save()}>
          تأكيد الربط
        </button>
      </div>
    </Dialog>
  );
}
