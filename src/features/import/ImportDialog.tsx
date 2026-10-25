import { useEffect, useRef, useState } from 'react';
import type { Api, NativeApi } from '../../shared/api';
import type { ImportPreview as Preview } from '../../shared/generated/ImportPreview';
import type { CommitImportInput } from '../../shared/generated/CommitImportInput';
import type { ImportReport } from '../../shared/generated/ImportReport';
import { Dialog } from '../../components/Dialog';
import { FormErrors } from '../../components/FormErrors';
import { ImportPreview } from './ImportPreview';
export function ImportDialog({
  deckId,
  api,
  native,
  onComplete,
  onCancel,
}: {
  deckId: string;
  api: Api;
  native: NativeApi;
  onComplete: (report: ImportReport) => void;
  onCancel: () => void;
}) {
  const [text, setText] = useState('');
  const [filename, setFilename] = useState('');
  const [preview, setPreview] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const title = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    if (preview) title.current?.focus();
  }, [preview]);
  async function pick() {
    setBusy(true);
    setError(null);
    try {
      const picked = await native.pickImportFile();
      if (picked) {
        setText(picked.text);
        setFilename(picked.name);
      }
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function inspect() {
    setBusy(true);
    setError(null);
    try {
      setPreview(await api.call('previewImport', { deckId, text }));
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function commit(input: CommitImportInput) {
    setBusy(true);
    setError(null);
    try {
      onComplete(await api.call('commitImport', input));
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog
      title="استيراد بطاقات JSON"
      onDismiss={() => {
        if (!busy) onCancel();
      }}
    >
      <FormErrors error={error} />
      {preview ? (
        <>
          <h3 ref={title} tabIndex={-1}>
            معاينة البطاقات
          </h3>
          <ImportPreview
            key={preview.previewId}
            preview={preview}
            busy={busy}
            onCommit={(input) => void commit(input)}
            onBack={() => {
              setPreview(null);
              setError(null);
            }}
            onCancel={onCancel}
          />
          {error && (
            <button disabled={busy} onClick={() => void inspect()}>
              تحديث المعاينة وإعادة تحديد البطاقات
            </button>
          )}
        </>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void inspect();
          }}
        >
          <p>
            اختر ملفًا أو الصق مصفوفة JSON. السؤال والإجابة مطلوبان؛ الملاحظات والصيغ البديلة
            اختياريان. الحد الأقصى ٥٠ ميبيبايت.
          </p>
          <button type="button" disabled={busy} onClick={() => void pick()}>
            اختيار ملف JSON
          </button>
          {filename && (
            <p>
              الملف: <bdi>{filename}</bdi>
            </p>
          )}
          <label>
            محتوى JSON
            <textarea
              className="json-input"
              value={text}
              onChange={(e) => setText(e.target.value)}
              dir="ltr"
              spellCheck={false}
            />
          </label>
          <div className="actions">
            <button className="primary" disabled={busy || !text.trim()} type="submit">
              فحص JSON
            </button>
            <button type="button" disabled={busy} onClick={onCancel}>
              إلغاء
            </button>
          </div>
        </form>
      )}
    </Dialog>
  );
}
