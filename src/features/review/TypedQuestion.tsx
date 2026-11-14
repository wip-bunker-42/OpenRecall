import { useEffect, useId, useRef, useState } from 'react';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import type { NativeApi } from '../../shared/api';
import type { CardForm } from '../../shared/generated/CardForm';
import type { CopyQuestionInput } from '../../shared/generated/CopyQuestionInput';

export function TypedPrompt({ form, focus }: { form: CardForm; focus: boolean }) {
  const heading = useRef<HTMLHeadingElement>(null);
  const id = useId();
  useEffect(() => {
    if (focus) heading.current?.focus();
  }, [focus]);
  return (
    <section aria-labelledby={id}>
      <h1 id={id} ref={heading} tabIndex={-1} dir="auto">
        {form.questionTitle ?? 'السؤال'}
      </h1>
      <p className="study-text question-text" dir="auto" lang={form.language ?? undefined}>
        {form.front}
      </p>
    </section>
  );
}

export function CopyQuestion({
  copy,
  input,
  disabled,
}: {
  copy: NativeApi['copyQuestion'];
  input: CopyQuestionInput;
  disabled: boolean;
}) {
  const active = useRef(false);
  const inFlight = useRef(false);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<unknown>(null);
  useEffect(() => {
    active.current = true;
    return () => {
      active.current = false;
    };
  }, []);
  async function onCopy() {
    if (disabled || inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setCopied(false);
    setError(null);
    try {
      await copy(input);
      if (active.current) setCopied(true);
    } catch (e) {
      if (active.current) setError(e);
    } finally {
      inFlight.current = false;
      if (active.current) setBusy(false);
    }
  }
  return (
    <>
      <button type="button" aria-disabled={disabled || busy} onClick={() => void onCopy()}>
        نسخ السؤال
      </button>
      <StatusRegion message={copied ? 'نُسخ السؤال إلى الحافظة.' : ''} />
      <FormErrors error={error} focus={false} />
    </>
  );
}
