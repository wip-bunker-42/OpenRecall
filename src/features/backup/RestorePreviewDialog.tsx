import { useState } from 'react';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import type { RestorePreview } from '../../shared/generated/RestorePreview';
export function RestorePreviewDialog({
  preview,
  busy,
  error,
  onDismiss,
  onChoose,
  onConfirm,
}: {
  preview: RestorePreview;
  busy: boolean;
  error: unknown;
  onDismiss: () => void;
  onChoose: () => void;
  onConfirm: () => void;
}) {
  const [confirmed, setConfirmed] = useState(false);
  return (
    <Dialog
      title="معاينة الاستعادة"
      onDismiss={() => {
        if (!busy) onDismiss();
      }}
    >
      <FormErrors error={error} />
      <p>
        ستُستبدل جميع البيانات الحالية بمحتويات هذه النسخة، بما فيها الإعدادات والتقدم ونماذج
        التحسين. تُحفظ نسخة أمان من المكتبة الحالية قبل الاستبدال.
      </p>
      <dl className="metrics">
        <dt>المجموعات</dt>
        <dd>{preview.decks}</dd>
        <dt>البطاقات</dt>
        <dd>{preview.cards}</dd>
        <dt>الصيغ</dt>
        <dd>{preview.forms}</dd>
        <dt>المراجعات المسجلة</dt>
        <dd>{preview.reviews}</dd>
        <dt>نماذج التحسين</dt>
        <dd>{preview.profiles}</dd>
        <dt>تاريخ التصدير</dt>
        <dd>
          <time dateTime={preview.exportedAt}>
            {new Intl.DateTimeFormat('ar-EG', { dateStyle: 'medium', timeStyle: 'short' }).format(
              new Date(preview.exportedAt),
            )}
          </time>
        </dd>
      </dl>
      <label>
        <input
          type="checkbox"
          checked={confirmed}
          disabled={busy}
          onChange={(e) => setConfirmed(e.target.checked)}
        />
        أوافق على استبدال جميع بيانات المكتبة الحالية
      </label>
      <div className="actions">
        <button data-autofocus disabled={busy} onClick={onDismiss}>
          إلغاء الاستعادة
        </button>
        <button disabled={busy} onClick={onChoose}>
          اختيار ملف آخر أو تحديث المعاينة
        </button>
        <button className="danger" disabled={busy || !confirmed} onClick={onConfirm}>
          استعادة النسخة واستبدال المكتبة
        </button>
      </div>
      {busy && <p role="status">جارٍ حفظ نسخة الأمان واستعادة المكتبة…</p>}
    </Dialog>
  );
}
