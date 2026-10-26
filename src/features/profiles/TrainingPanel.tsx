import { useCallback, useEffect, useRef, useState } from 'react';
import type { Api } from '../../shared/api';
import type { TrainingEligibility } from '../../shared/generated/TrainingEligibility';
import type { TrainingJob } from '../../shared/generated/TrainingJob';
import type { ProfileDetail } from '../../shared/generated/ProfileDetail';
import { EligibilityTable } from './EligibilityTable';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { number, dateTime } from '../../shared/format';
const phases: Record<string, string> = {
  queued: 'التحسين في قائمة الانتظار.',
  running: 'جارٍ تدريب النموذج على جهازك.',
  cancelled: 'أُلغي التدريب. لم تتغير الأوزان الحالية.',
  interrupted: 'توقف التدريب قبل اكتماله. يمكنك بدء محاولة جديدة.',
  failed: 'تعذر إكمال التدريب أو أصبحت نتيجته قديمة.',
  noImprovement: 'اكتمل التدريب دون نموذج أفضل.',
  ready: 'يوجد نموذج مرشح اجتاز المقارنة، بانتظار قرارك.',
  applied: 'طُبق النموذج على المجموعات المرتبطة.',
};
const active = (job: TrainingJob | null) => !!job && ['queued', 'running'].includes(job.status);
const metric = (v: number) =>
  new Intl.NumberFormat('ar-EG', { maximumFractionDigits: 6 }).format(v);
function Comparison({ job }: { job: TrainingJob }) {
  if (!job.baselineMetrics || !job.candidateMetrics) return null;
  return (
    <>
      <p>{`الاختبار يشمل ${number(job.eligibility.holdoutCount)} ملاحظة من ${number(job.eligibility.holdoutCards)} بطاقة، من الجزء الأحدث زمنيًا الذي يقارب ٢٠٪ من البيانات. لا يُستخدم تقييم الملاحظة لتوقع النتيجة نفسها.`}</p>
      <table>
        <caption>مقارنة الأداء على بيانات الاختبار</caption>
        <thead>
          <tr>
            <th scope="col">المقياس (الأقل أفضل)</th>
            <th scope="col">النموذج الحالي عند البدء</th>
            <th scope="col">النموذج المرشح</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <th scope="row">Log loss</th>
            <td>{metric(job.baselineMetrics.logLoss)}</td>
            <td>{metric(job.candidateMetrics.logLoss)}</td>
          </tr>
          <tr>
            <th scope="row">RMSE</th>
            <td>{metric(job.baselineMetrics.rmse)}</td>
            <td>{metric(job.candidateMetrics.rmse)}</td>
          </tr>
        </tbody>
      </table>
      {job.perDeck.length > 0 && (
        <details>
          <summary>أداء المجموعات في الاختبار</summary>
          <table>
            <caption>أداء كل مجموعة في الاختبار</caption>
            <thead>
              <tr>
                <th scope="col">المجموعة</th>
                <th scope="col">ملاحظات الاختبار</th>
                <th scope="col">Log loss قبل</th>
                <th scope="col">Log loss بعد</th>
                <th scope="col">RMSE قبل</th>
                <th scope="col">RMSE بعد</th>
              </tr>
            </thead>
            <tbody>
              {job.perDeck.map((d) => (
                <tr key={d.deckId}>
                  <th scope="row">
                    <bdi>
                      {job.eligibility.contributions.find((c) => c.deckId === d.deckId)?.deckName ??
                        d.deckId}
                    </bdi>
                  </th>
                  <td>{number(d.observations)}</td>
                  <td>{d.baseline ? metric(d.baseline.logLoss) : 'لا يوجد قياس'}</td>
                  <td>{d.candidate ? metric(d.candidate.logLoss) : 'لا يوجد قياس'}</td>
                  <td>{d.baseline ? metric(d.baseline.rmse) : 'لا يوجد قياس'}</td>
                  <td>{d.candidate ? metric(d.candidate.rmse) : 'لا يوجد قياس'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </details>
      )}
    </>
  );
}
export function TrainingPanel({
  profileId,
  api,
  onApplied,
  configurationDirty = false,
}: {
  profileId: string;
  api: Api;
  onApplied?: (profile: ProfileDetail) => void | Promise<void>;
  configurationDirty?: boolean;
}) {
  const [eligibility, setEligibility] = useState<TrainingEligibility | null>(null);
  const [job, setJob] = useState<TrainingJob | null>(null);
  const [history, setHistory] = useState<TrainingJob[]>([]);
  const [profile, setProfile] = useState<ProfileDetail | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [confirm, setConfirm] = useState(false);
  const sequence = useRef(0);
  const load = useCallback(async () => {
    const version = ++sequence.current;
    try {
      const [eligibility, runs, profile] = await Promise.all([
        api.call('getTrainingEligibility', { profileId }),
        api.call('listTrainingRuns', { profileId }),
        api.call('getProfile', { profileId }),
      ]);
      if (version !== sequence.current) return;
      setEligibility(eligibility);
      const latest =
        runs.slice().sort((a, b) => Date.parse(b.startedAt) - Date.parse(a.startedAt))[0] ?? null;
      const refreshed =
        latest?.status === 'ready'
          ? await api.call('getTrainingJob', { jobId: latest.id })
          : latest;
      if (version !== sequence.current) return;
      setProfile(profile);
      setHistory(runs);
      setJob(refreshed);
    } catch (error) {
      if (version === sequence.current) throw error;
    }
  }, [api, profileId]);
  useEffect(() => {
    let mounted = true;
    void load().catch((e) => {
      if (mounted) setError(e);
    });
    return () => {
      mounted = false;
      sequence.current++;
    };
  }, [load]);
  const jobId = job?.id;
  const jobStatus = job?.status;
  useEffect(() => {
    if (!jobId || !['queued', 'running'].includes(jobStatus ?? '')) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      const version = sequence.current;
      try {
        const result = await api.call('getTrainingJob', { jobId });
        if (!cancelled && version === sequence.current) setJob(result);
        if (!cancelled && active(result)) timer = setTimeout(() => void poll(), 1000);
      } catch (e) {
        if (!cancelled) {
          setError(e);
          timer = setTimeout(() => void poll(), 3000);
        }
      }
    };
    timer = setTimeout(() => void poll(), 1000);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [api, jobId, jobStatus]);
  async function start() {
    setBusy(true);
    setError(null);
    sequence.current++;
    try {
      setJob(await api.call('startTraining', { profileId }));
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function cancel() {
    if (!job) return;
    setBusy(true);
    setError(null);
    sequence.current++;
    try {
      setJob(await api.call('cancelTraining', { jobId: job.id }));
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function requestApply() {
    setBusy(true);
    setError(null);
    try {
      setProfile(await api.call('getProfile', { profileId }));
      setConfirm(true);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function apply() {
    if (!job || !profile) return;
    setBusy(true);
    setError(null);
    sequence.current++;
    try {
      const updated = await api.call('applyTraining', {
        jobId: job.id,
        expectedProfileRevision: profile.revision,
      });
      setProfile(updated);
      setJob({ ...job, status: 'applied' });
      setConfirm(false);
      await onApplied?.(updated);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <h3>التدريب على سجل المجموعات المرتبطة</h3>
      {configurationDirty && (
        <p>احفظ التعديلات غير المحفوظة أو تجاهلها قبل بدء تدريب أو تطبيق نموذج.</p>
      )}
      <FormErrors error={error} focus={busy || confirm} />
      {!eligibility ? (
        <p role="status">جارٍ فحص البيانات…</p>
      ) : (
        <>
          <EligibilityTable eligibility={eligibility} />
          <p>
            التدريب محلي وقد يستغرق بعض الوقت. أنهِ أي جلسة مراجعة غير مكتملة للمجموعات المرتبطة قبل
            البدء أو تطبيق النتيجة.
          </p>
          <div className="actions">
            <button
              className="primary"
              disabled={busy || configurationDirty || !eligibility.eligible || active(job)}
              onClick={() => void start()}
            >
              بدء التحسين
            </button>
            <button disabled={busy || active(job)} onClick={() => void load().catch(setError)}>
              تحديث أهلية التدريب
            </button>
            {active(job) && (
              <button disabled={busy || job?.cancelRequested} onClick={() => void cancel()}>
                إلغاء التدريب
              </button>
            )}
          </div>
        </>
      )}
      <StatusRegion
        message={
          job
            ? job.cancelRequested && active(job)
              ? 'جارٍ إلغاء التدريب…'
              : (phases[job.status] ?? job.status)
            : ''
        }
      />
      {job && (
        <section aria-label="نتيجة التدريب">
          {active(job) && (
            <>
              <progress aria-label="تقدم التدريب" value={job.progress.fraction} max={1} />
              <p>
                {number(job.progress.current)} من {number(job.progress.total)} خطوة تدريب مسجلة
              </p>
            </>
          )}
          {job.error && <FormErrors error={job.error} focus={false} />}
          <Comparison job={job} />
          {job.status === 'noImprovement' && (
            <p>
              لم تتحسن النتائج بما يكفي، أو أعاد المحرك أوزانه الافتراضية دون تدريب فعلي. لم تتغير
              الأوزان الحالية أو مواعيد البطاقات.
            </p>
          )}
          {job.status === 'ready' && (
            <button
              className="primary"
              disabled={busy || configurationDirty}
              onClick={() => void requestApply()}
            >
              تطبيق النموذج المحسن
            </button>
          )}
          {job.status === 'applied' && (
            <p>حُفظ النموذج الجديد وأُعيد بناء حالات الذاكرة. مواعيد الاستحقاق الحالية محفوظة.</p>
          )}
        </section>
      )}
      {history.length > 0 && (
        <details>
          <summary>سجل عمليات التحسين</summary>
          <ul>
            {history
              .slice()
              .sort((a, b) => Date.parse(b.startedAt) - Date.parse(a.startedAt))
              .map((run) => (
                <li key={run.id}>
                  {dateTime(run.startedAt)}:{' '}
                  {phases[run.id === job?.id ? job.status : run.status] ?? run.status}
                </li>
              ))}
          </ul>
        </details>
      )}
      {confirm && profile && (
        <Dialog
          title="تطبيق نموذج التحسين"
          onDismiss={() => {
            if (!busy) setConfirm(false);
          }}
        >
          <p>
            سيتغير نموذج كل المجموعات المرتبطة:{' '}
            {profile.memberDeckIds
              .map((id) => eligibility?.contributions.find((c) => c.deckId === id)?.deckName ?? id)
              .join('، ')}
            . ستُعاد حسابات الذاكرة بالسجل نفسه، مع الاحتفاظ بمواعيد الاستحقاق الحالية. لن تتغير
            النماذج المستقلة.
          </p>
          <FormErrors error={error} />
          <div className="actions">
            <button data-autofocus disabled={busy} onClick={() => setConfirm(false)}>
              إلغاء
            </button>
            <button className="primary" disabled={busy} onClick={() => void apply()}>
              تأكيد تطبيق النموذج
            </button>
          </div>
        </Dialog>
      )}
    </>
  );
}
