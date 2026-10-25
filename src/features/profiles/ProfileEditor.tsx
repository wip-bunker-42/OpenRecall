import { useEffect, useState } from 'react';
import type { Api } from '../../shared/api';
import type { ProfileDetail } from '../../shared/generated/ProfileDetail';
import type { TrainingConfig } from '../../shared/generated/TrainingConfig';
import type { DeckDetail } from '../../shared/generated/DeckDetail';
import { Field } from '../../components/Field';
import { Disclosure } from '../../components/Disclosure';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { numeric } from '../../shared/forms';
import { useUnsaved } from '../../app/unsaved';
import { TrainingPanel } from './TrainingPanel';
import { AssignmentDialog } from './AssignmentDialog';
import { number } from '../../shared/format';
type Confirmation =
  | { kind: 'config'; config: TrainingConfig }
  | { kind: 'reset' }
  | { kind: 'delete' }
  | { kind: 'detach'; deck: DeckDetail };
export function ProfileEditor({
  profile,
  decks,
  api,
  onChanged,
}: {
  profile: ProfileDetail;
  decks: DeckDetail[];
  api: Api;
  onChanged: () => void | Promise<void>;
}) {
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null);
  const [assignment, setAssignment] = useState<{ deck: DeckDetail; profile: ProfileDetail } | null>(
    null,
  );
  const [add, setAdd] = useState('');
  const [trainingOpen, setTrainingOpen] = useState(false);
  const [message, setMessage] = useState('');
  const [renameChanged, setRenameChanged] = useState(false);
  const [configChanged, setConfigChanged] = useState(false);
  const [draftKey, setDraftKey] = useState(0);
  const hasDraft = renameChanged || configChanged;
  const { setDirty: renameDirty } = useUnsaved();
  const { setDirty: configDirty } = useUnsaved();
  useEffect(
    () => () => {
      renameDirty(false);
      configDirty(false);
    },
    [renameDirty, configDirty],
  );
  async function rename(name: string) {
    setBusy(true);
    setError(null);
    try {
      await api.call('renameProfile', {
        profileId: profile.id,
        name,
        expectedRevision: profile.revision,
      });
      renameDirty(false);
      await onChanged();
      setRenameChanged(false);
      setMessage('حُفظ اسم النموذج.');
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function saveConfig(config: TrainingConfig) {
    setBusy(true);
    setError(null);
    try {
      await api.call('saveTrainingConfig', {
        profileId: profile.id,
        config,
        expectedRevision: profile.revision,
      });
      configDirty(false);
      setConfirmation(null);
      setConfigChanged(false);
      await onChanged();
      setMessage('حُفظت إعدادات التدريب. الأوزان الحالية لم تتغير.');
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  function readConfig(data: FormData) {
    setError(null);
    try {
      const config: TrainingConfig = {
        numEpochs: numeric(data, 'numEpochs', 1, 100, true, true),
        batchSize: numeric(data, 'batchSize', 16, 4096, true),
        learningRate: numeric(data, 'learningRate', 0.00001, 0.1, false, true),
        seed: numeric(data, 'seed', 0, 4294967295, true),
        maxSeqLen: numeric(data, 'maxSeqLen', 32, 4096, true),
        gamma: numeric(data, 'gamma', 0.1, 5),
        schedulePenalties: data.has('schedulePenalties'),
      };
      if (profile.memberDeckIds.length > 1) setConfirmation({ kind: 'config', config });
      else void saveConfig(config);
    } catch (e) {
      setError(e);
    }
  }
  async function confirm(data: FormData) {
    if (!confirmation) return;
    if (confirmation.kind === 'config') {
      await saveConfig(confirmation.config);
      return;
    }
    setBusy(true);
    setError(null);
    try {
      if (confirmation.kind === 'reset')
        await api.call('resetProfile', {
          profileId: profile.id,
          expectedRevision: profile.revision,
        });
      else if (confirmation.kind === 'delete')
        await api.call('deleteProfile', {
          profileId: profile.id,
          expectedRevision: profile.revision,
        });
      else
        await api.call('detachProfile', {
          deckId: confirmation.deck.id,
          newName: String(data.get('newName')),
          expectedDeckRevision: confirmation.deck.revision,
        });
      setConfirmation(null);
      await onChanged();
      setMessage('حُفظ التغيير.');
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function link() {
    const deck = decks.find((d) => d.id === add);
    if (!deck) return;
    setBusy(true);
    setError(null);
    try {
      setAssignment({ deck, profile: await api.call('getProfile', { profileId: profile.id }) });
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  const linked = decks.filter((d) => profile.memberDeckIds.includes(d.id));
  const available = decks.filter((d) => d.profileId !== profile.id);
  return (
    <>
      <FormErrors error={error} />
      <StatusRegion message={message} />
      <form
        key={`name-${draftKey}`}
        noValidate
        onInput={() => {
          renameDirty(true);
          setRenameChanged(true);
        }}
        onSubmit={(e) => {
          e.preventDefault();
          void rename(String(new FormData(e.currentTarget).get('name')));
        }}
      >
        <Field
          name="name"
          disabled={busy || configChanged}
          label="اسم نموذج التحسين"
          defaultValue={profile.name}
          required
          maxLength={256}
          dir="auto"
          error={error}
        />
        <button disabled={busy || configChanged} type="submit">
          حفظ اسم النموذج
        </button>
      </form>
      <p>{number(linked.length)} مجموعة مرتبطة بهذا النموذج.</p>
      <ul>
        {linked.map((d) => (
          <li key={d.id}>
            <bdi>{d.name}</bdi>{' '}
            <button
              disabled={busy || hasDraft}
              aria-label={`فصل المجموعة ${d.name} عن النموذج`}
              onClick={() => setConfirmation({ kind: 'detach', deck: d })}
            >
              فصل المجموعة
            </button>
          </li>
        ))}
      </ul>
      <label>
        مجموعة لإضافتها إلى النموذج
        <select disabled={busy || hasDraft} value={add} onChange={(e) => setAdd(e.target.value)}>
          <option value="">اختر مجموعة</option>
          {available.map((d) => (
            <option value={d.id} key={d.id}>
              {d.name}
            </option>
          ))}
        </select>
      </label>
      <button disabled={busy || hasDraft || !add} onClick={() => void link()}>
        ربط المجموعة بالنموذج
      </button>
      <Disclosure title="إعدادات التدريب المتقدمة" level={3}>
        <p>
          تُستخدم هذه الإعدادات في التدريب القادم لكل المجموعات المرتبطة، ولا تطبّق أوزانًا جديدة
          تلقائيًا.
        </p>
        <form
          key={`${profile.revision}-${draftKey}`}
          noValidate
          onInput={() => {
            configDirty(true);
            setConfigChanged(true);
          }}
          onSubmit={(e) => {
            e.preventDefault();
            readConfig(new FormData(e.currentTarget));
          }}
        >
          <fieldset disabled={busy || renameChanged}>
            <legend>تكوين محرك FSRS-7</legend>
            <label>
              <input
                type="checkbox"
                name="schedulePenalties"
                defaultChecked={profile.config.schedulePenalties}
              />
              تفعيل Schedule Penalties
            </label>
            <p className="field-help">مفعّلة افتراضيًا، وتُمرّر صراحةً إلى المحرك عند التدريب.</p>
            <Field
              name="numEpochs"
              label="عدد العصور (اختياري)"
              type="number"
              min={1}
              max={100}
              step={1}
              defaultValue={profile.config.numEpochs ?? ''}
              help="من ١ إلى ١٠٠. اتركه فارغًا لاختيار المحرك الافتراضي."
              error={error}
            />
            <Field
              name="batchSize"
              label="حجم الدفعة"
              type="number"
              min={16}
              max={4096}
              step={1}
              defaultValue={profile.config.batchSize}
              error={error}
            />
            <Field
              name="learningRate"
              label="معدل التعلم (اختياري)"
              type="number"
              min={0.00001}
              max={0.1}
              step="any"
              defaultValue={profile.config.learningRate ?? ''}
              help="من 0.00001 إلى 0.1. القيمة الفارغة تستخدم افتراضي المحرك."
              error={error}
            />
            <Field
              name="seed"
              label="بذرة العشوائية"
              type="number"
              min={0}
              max={4294967295}
              step={1}
              defaultValue={profile.config.seed}
              error={error}
            />
            <Field
              name="maxSeqLen"
              label="أقصى طول للسجل"
              type="number"
              min={32}
              max={4096}
              step={1}
              defaultValue={profile.config.maxSeqLen}
              help="من ٣٢ إلى ٤٠٩٦ تقييمًا في المثال الواحد. قد يستبعد الحد المنخفض بعض السجلات الطويلة من التدريب."
              error={error}
            />
            <Field
              name="gamma"
              label="مضاعف تنظيم L2"
              type="number"
              min={0.1}
              max={5}
              step="any"
              defaultValue={profile.config.gamma}
              help="من 0.1 إلى 5. يضبط قوة الانتظام في التدريب، ولا يغيّر وزن المراجعات حسب حداثتها."
              error={error}
            />
            <button type="submit" className="primary">
              حفظ إعدادات التدريب
            </button>
          </fieldset>
        </form>
      </Disclosure>
      <details className="disclosure" onToggle={(e) => setTrainingOpen(e.currentTarget.open)}>
        <summary>تحسين النموذج</summary>
        <div className="disclosure-body">
          {trainingOpen && (
            <TrainingPanel
              profileId={profile.id}
              api={api}
              configurationDirty={hasDraft}
              onApplied={() => onChanged()}
            />
          )}
        </div>
      </details>
      <Disclosure title="تفاصيل النموذج الحالي" level={3}>
        <Field
          name="modelRevision"
          label="معرّف النموذج الحالي"
          value={profile.activeModelRevision}
          readOnly
          dir="ltr"
        />
        <Field
          name="modelOrigin"
          label="مصدر الأوزان"
          value={
            profile.modelOrigin === 'trained'
              ? 'تدريب محلي'
              : profile.modelOrigin === 'inherited'
                ? 'موروثة عند الفصل'
                : 'افتراضي المحرك'
          }
          readOnly
        />
        <label>
          أوزان FSRS-7
          <textarea
            readOnly
            value={profile.parameters.join(', ')}
            dir="ltr"
            aria-describedby={`weights-help-${profile.id}`}
          />
        </label>
        <p className="field-help" id={`weights-help-${profile.id}`}>
          للقراءة فقط؛ عددها {number(profile.parameters.length)}. تغييرها يمر بالتدريب والاختبار ثم
          تطبيق صريح، أو استعادة افتراضي المحرك.
        </p>
        <button disabled={busy || hasDraft} onClick={() => setConfirmation({ kind: 'reset' })}>
          استعادة أوزان المحرك الافتراضية
        </button>
      </Disclosure>
      <div className="actions">
        <button
          className="danger"
          disabled={busy || hasDraft || profile.memberDeckIds.length > 0}
          aria-describedby={`delete-help-${profile.id}`}
          onClick={() => setConfirmation({ kind: 'delete' })}
        >
          حذف النموذج
        </button>
      </div>
      <p className="muted" id={`delete-help-${profile.id}`}>
        {profile.memberDeckIds.length > 0
          ? 'لا يمكن حذف النموذج وهو مرتبط بمجموعات. افصلها أو انقلها إلى نموذج آخر أولًا.'
          : hasDraft
            ? 'احفظ التعديلات أو تجاهلها قبل حذف النموذج.'
            : 'هذا النموذج غير مرتبط بمجموعات، ويمكن حذفه.'}
      </p>
      {hasDraft && (
        <>
          <p>
            احفظ النموذج الجاري تعديله أو تجاهل تعديلاته قبل تعديل القسم الآخر أو تغيير ربط
            المجموعات.
          </p>
          <button
            disabled={busy}
            onClick={() => {
              setDraftKey((n) => n + 1);
              setRenameChanged(false);
              setConfigChanged(false);
              renameDirty(false);
              configDirty(false);
              setError(null);
            }}
          >
            تجاهل التعديلات غير المحفوظة
          </button>
        </>
      )}
      {assignment && (
        <AssignmentDialog
          {...assignment}
          api={api}
          onCancel={() => setAssignment(null)}
          onSaved={async () => {
            setAssignment(null);
            setAdd('');
            await onChanged();
          }}
        />
      )}
      {confirmation && (
        <Dialog
          title={
            confirmation.kind === 'detach'
              ? 'فصل المجموعة'
              : confirmation.kind === 'delete'
                ? 'حذف نموذج التحسين'
                : confirmation.kind === 'reset'
                  ? 'استعادة الأوزان الافتراضية'
                  : 'حفظ إعدادات نموذج مشترك'
          }
          onDismiss={() => {
            if (!busy) setConfirmation(null);
          }}
        >
          <FormErrors error={error} />
          <form
            noValidate
            onSubmit={(e) => {
              e.preventDefault();
              void confirm(new FormData(e.currentTarget));
            }}
          >
            {confirmation.kind === 'detach' ? (
              <>
                <p>
                  ستصبح المجموعة مستقلة بنموذج يحتفظ بالأوزان والإعدادات الحالية، وحالات الذاكرة
                  والمواعيد والصيغ. يؤثر تدريبها التالي في نموذجها الجديد وحده.
                </p>
                <Field
                  name="newName"
                  label="اسم النموذج المستقل"
                  defaultValue={confirmation.deck.name}
                  required
                  dir="auto"
                  error={error}
                />
              </>
            ) : confirmation.kind === 'delete' ? (
              <p>
                سيُحذف النموذج غير المرتبط. تبقى مراجع النماذج التاريخية اللازمة لسجل المراجعات
                محفوظة.
              </p>
            ) : (
              <p>
                هذا التغيير يخص كل المجموعات المرتبطة: {linked.map((d) => d.name).join('، ')}.{' '}
                {confirmation.kind === 'reset'
                  ? 'ستُعاد حالات الذاكرة بالأوزان الافتراضية مع الاحتفاظ بمواعيد الاستحقاق الحالية.'
                  : 'يُستخدم التكوين الجديد في التدريب القادم، وتبقى الأوزان الحالية كما هي.'}
              </p>
            )}
            <div className="actions">
              <button type="button" disabled={busy} onClick={() => setConfirmation(null)}>
                إلغاء
              </button>
              <button
                className={confirmation.kind === 'delete' ? 'danger' : 'primary'}
                type="submit"
                disabled={busy}
              >
                {confirmation.kind === 'detach'
                  ? 'تأكيد الفصل'
                  : confirmation.kind === 'delete'
                    ? 'تأكيد حذف النموذج'
                    : confirmation.kind === 'reset'
                      ? 'تأكيد استعادة الأوزان'
                      : 'تأكيد حفظ إعدادات التدريب'}
              </button>
            </div>
          </form>
        </Dialog>
      )}
    </>
  );
}
