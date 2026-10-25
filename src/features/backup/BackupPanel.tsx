import { useContext, useState } from 'react';
import type { NativeApi } from '../../shared/api';
import type { ExportReport } from '../../shared/generated/ExportReport';
import type { RestorePreview } from '../../shared/generated/RestorePreview';
import { UnsavedContext } from '../../app/unsaved';
import { Disclosure } from '../../components/Disclosure';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { RestorePreviewDialog } from './RestorePreviewDialog';
export function BackupPanel({
  native,
  onRestored,
}: {
  native: NativeApi;
  onRestored: () => void | Promise<void>;
}) {
  const { dirty } = useContext(UnsavedContext);
  const [busy, setBusy] = useState(false),
    [error, setError] = useState<unknown>(null),
    [message, setMessage] = useState('');
  const [exported, setExported] = useState<ExportReport | null>(null),
    [preview, setPreview] = useState<RestorePreview | null>(null);
  async function run(action: () => Promise<void>) {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage('');
    try {
      await action();
    } catch (e) {
      if (!(typeof e === 'object' && e !== null && 'code' in e && e.code === 'cancelled'))
        setError(e);
    } finally {
      setBusy(false);
    }
  }
  function choose() {
    void run(async () => {
      const next = await native.pickBackupForPreview();
      if (next) setPreview(next);
    });
  }
  return (
    <Disclosure title="النسخ الاحتياطي والاستعادة">
      <p>
        احتفظ بنسخة محلية تشمل الإعدادات والبطاقات والصيغ والتقدم وسجلات المراجعة ونماذج التحسين.
        يُحفظ التصدير في التنزيلات، أو تختار مكانًا إذا تعذر الحفظ فيها.
      </p>
      {dirty && <p>احفظ التعديلات أو تجاهلها قبل التصدير أو الاستعادة.</p>}
      {!preview && <FormErrors error={error} />}
      <StatusRegion message={message} />
      <div className="actions">
        <button
          disabled={busy || dirty}
          onClick={() =>
            void run(async () => {
              const report = await native.exportBackup();
              setExported(report);
              setMessage('اكتمل تصدير النسخة الاحتياطية.');
            })
          }
        >
          تصدير جميع البيانات
        </button>
        <button disabled={busy || dirty} onClick={choose}>
          اختيار نسخة للاستعادة
        </button>
      </div>
      {busy && !preview && <p role="status">جارٍ تنفيذ عملية النسخ الاحتياطي…</p>}
      {exported && (
        <div className="result">
          <p dir="auto">{exported.path}</p>
          <button
            disabled={busy}
            onClick={() => void run(() => native.revealExport({ exportId: exported.exportId }))}
          >
            إظهار ملف التصدير
          </button>
        </div>
      )}
      {preview && (
        <RestorePreviewDialog
          key={preview.id}
          preview={preview}
          busy={busy}
          error={error}
          onDismiss={() => {
            setPreview(null);
            setError(null);
          }}
          onChoose={choose}
          onConfirm={() =>
            void run(async () => {
              const report = await native.restoreBackup({
                previewId: preview.id,
                expectedLibraryRevision: preview.expectedLibraryRevision,
              });
              setExported(report.safetyBackup);
              setPreview(null);
              setMessage('تمت استعادة النسخة الاحتياطية.');
              await onRestored();
            })
          }
        />
      )}
    </Disclosure>
  );
}
