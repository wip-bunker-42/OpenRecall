import { useContext, useEffect, useState } from 'react';
import type { NativeApi } from '../../shared/api';
import type { UpdateStatus } from '../../shared/generated/UpdateStatus';
import { Disclosure } from '../../components/Disclosure';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { UnsavedContext } from '../../app/unsaved';
export function UpdatePanel({ native }: { native: NativeApi }) {
  const [status, setStatus] = useState<UpdateStatus | null>(null),
    [busy, setBusy] = useState(false),
    [confirm, setConfirm] = useState(false),
    [error, setError] = useState<unknown>(null);
  const { dirty } = useContext(UnsavedContext);
  useEffect(() => {
    let current = true,
      dispose: (() => void) | undefined,
      eventEpoch = 0;
    void native
      .onUpdateStatus((s) => {
        eventEpoch++;
        if (current) setStatus(s);
      })
      .then(async (unlisten) => {
        if (!current) {
          unlisten();
          return;
        }
        dispose = unlisten;
        const epoch = eventEpoch;
        const initial = await native.getUpdateStatus();
        if (current && eventEpoch === epoch) setStatus(initial);
      })
      .catch((e) => {
        if (current) setError(e);
      });
    return () => {
      current = false;
      dispose?.();
    };
  }, [native]);
  async function run(action: () => Promise<UpdateStatus | void>) {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      const next = await action();
      if (next) setStatus(next);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  const pending = busy || ['checking', 'downloading', 'installing'].includes(status?.phase ?? '');
  const messages: Record<string, string> = {
    checking: 'جارٍ التحقق من التحديثات…',
    upToDate: 'أنت تستخدم أحدث إصدار متاح.',
    available: 'يتوفر تحديث جديد.',
    downloading: 'جارٍ تنزيل التحديث والتحقق من توقيعه…',
    ready: 'اكتمل التنزيل والتحقق من التوقيع. التحديث جاهز للتثبيت.',
    installing: 'جارٍ تثبيت التحديث وإعادة تشغيل التطبيق…',
  };
  const problem = error ?? (status?.errorCode ? { code: status.errorCode } : null);
  return (
    <Disclosure title="تحديث التطبيق">
      <p>
        تعمل المذاكرة دون إنترنت. يتصل هذا القسم بـ GitHub عند طلب التحقق أو التنزيل فقط. احفظ
        تعديلاتك وأنهِ الجلسات والتدريب قبل تثبيت التحديث.
      </p>
      {!confirm && <FormErrors error={problem} />}
      <StatusRegion message={messages[status?.phase ?? ''] ?? ''} />
      {status && (
        <p>
          الإصدار الحالي: <bdi>{status.currentVersion}</bdi>
          {status.availableVersion && (
            <>
              {' '}
              · الإصدار المتاح: <bdi>{status.availableVersion}</bdi>
            </>
          )}
        </p>
      )}
      {status?.notes && (
        <div className="prose" dir="auto">
          {status.notes}
        </div>
      )}
      {status?.phase === 'downloading' && (
        <div
          role="progressbar"
          aria-label="تنزيل التحديث"
          aria-valuemin={0}
          aria-valuemax={status.totalBytes ?? undefined}
          aria-valuenow={status.totalBytes ? status.downloadedBytes : undefined}
          aria-valuetext={`${status.downloadedBytes} بايت${status.totalBytes ? ` من ${status.totalBytes}` : ''}`}
        >
          {status.downloadedBytes} بايت
        </div>
      )}
      <div className="actions">
        <button disabled={pending} onClick={() => void run(() => native.checkForUpdate())}>
          التحقق من وجود تحديث
        </button>
        {status?.availableVersion && status.phase !== 'ready' && status.phase !== 'installing' && (
          <button disabled={pending} onClick={() => void run(() => native.downloadUpdate())}>
            تنزيل التحديث
          </button>
        )}
        {status?.phase === 'ready' && (
          <button
            disabled={pending || dirty}
            onClick={() => {
              setError(null);
              setConfirm(true);
            }}
          >
            تثبيت التحديث وإعادة التشغيل
          </button>
        )}
      </div>
      {dirty && <p>احفظ التعديلات أو تجاهلها أولًا قبل التثبيت.</p>}
      {confirm && (
        <Dialog
          title="تثبيت التحديث"
          onDismiss={() => {
            if (!busy) setConfirm(false);
          }}
        >
          <FormErrors error={problem} />
          <p>
            سيُغلق التطبيق لتثبيت الإصدار الجديد ثم يُعاد تشغيله. بيانات المذاكرة محفوظة على جهازك.
          </p>
          <div className="actions">
            <button data-autofocus disabled={busy} onClick={() => setConfirm(false)}>
              إلغاء التثبيت
            </button>
            <button disabled={busy} onClick={() => void run(() => native.installUpdate())}>
              تثبيت الآن
            </button>
          </div>
          {busy && <p role="status">جارٍ التحقق من حفظ البيانات وتثبيت التحديث…</p>}
        </Dialog>
      )}
    </Disclosure>
  );
}
