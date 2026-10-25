import { useEffect, useState } from 'react';
import type { Api, NativeApi } from '../../shared/api';
import type { Navigate } from '../../app/navigation';
import type { Settings } from '../../shared/generated/Settings';
import type { SystemInfo } from '../../shared/generated/SystemInfo';
import { Field } from '../../components/Field';
import { Disclosure } from '../../components/Disclosure';
import { FormErrors } from '../../components/FormErrors';
import { StatusRegion } from '../../components/StatusRegion';
import { numeric, validation } from '../../shared/forms';
import { useUnsaved } from '../../app/unsaved';
import { BackupPanel } from '../backup/BackupPanel';
function SettingsForm({
  settings,
  api,
  onSaved,
}: {
  settings: Settings;
  api: Api;
  onSaved: (settings: Settings) => void;
}) {
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [initial, setInitial] = useState(settings);
  const [formKey, setFormKey] = useState(0);
  const { setDirty } = useUnsaved();
  useEffect(() => () => setDirty(false), [setDirty]);
  async function save(data: FormData) {
    setError(null);
    try {
      const timezone = String(data.get('timezone') ?? '').trim();
      if (!timezone) validation('timezone', 'invalidTimezone');
      const start = String(data.get('dayStart') ?? '');
      if (!/^([01]\d|2[0-3]):[0-5]\d$/.test(start)) validation('dayStartMinute');
      const [hour, minute] = start.split(':').map(Number);
      const draft: Settings = {
        ...settings,
        timezone,
        dayStartMinute: hour * 60 + minute,
        desiredRetention: numeric(data, 'desiredRetention', 70, 99) / 100,
        minIntervalSeconds: numeric(data, 'minIntervalSeconds', 10, 3600, true),
        maxIntervalDays: numeric(data, 'maxIntervalDays', 1, 36500, true),
        shortcutsEnabled: data.has('shortcutsEnabled'),
        textScale: numeric(data, 'textScale', 80, 200) / 100,
        reducedMotion: String(data.get('reducedMotion')),
        pageSize: Number(data.get('pageSize')),
      };
      setBusy(true);
      const saved = await api.call('saveSettings', {
        settings: draft,
        expectedRevision: settings.revision,
      });
      setDirty(false);
      onSaved(saved);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  async function defaults() {
    setBusy(true);
    setError(null);
    try {
      const defaults = await api.call('getDefaultSettings', {});
      setInitial({ ...defaults, revision: settings.revision });
      setFormKey((n) => n + 1);
      setDirty(true);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <FormErrors error={error} />
      <form
        key={formKey}
        noValidate
        onInput={() => setDirty(true)}
        onSubmit={(e) => {
          e.preventDefault();
          void save(new FormData(e.currentTarget));
        }}
      >
        <fieldset disabled={busy}>
          <legend>تفضيلات المذاكرة</legend>
          <Field
            name="timezone"
            label="المنطقة الزمنية"
            defaultValue={initial.timezone}
            dir="ltr"
            error={error}
            help="استخدم اسم منطقة IANA، مثل Africa/Cairo. يُطبّق التوقيت الصيفي تلقائيًا."
          />
          <Field
            name="dayStart"
            label="بداية يوم المذاكرة"
            type="time"
            defaultValue={`${String(Math.floor(initial.dayStartMinute / 60)).padStart(2, '0')}:${String(initial.dayStartMinute % 60).padStart(2, '0')}`}
            error={error}
            help="بداية يوم الإحصاءات. تغييرها لا ينقل مواعيد البطاقات المحفوظة."
          />
          <Field
            name="desiredRetention"
            label="الاحتفاظ المطلوب (٪)"
            type="number"
            min={70}
            max={99}
            step="any"
            defaultValue={initial.desiredRetention * 100}
            error={error}
            help="من ٧٠ إلى ٩٩٪. زيادة الاحتفاظ تقصّر الفواصل عادةً وتزيد عدد المراجعات."
          />
          <label>
            <input
              type="checkbox"
              name="shortcutsEnabled"
              defaultChecked={initial.shortcutsEnabled}
            />
            تفعيل اختصارات المراجعة
          </label>
          <Field
            name="textScale"
            label="حجم النص (٪)"
            type="number"
            min={80}
            max={200}
            step="any"
            defaultValue={initial.textScale * 100}
            error={error}
          />
          <label>
            الحركة
            <select name="reducedMotion" defaultValue={initial.reducedMotion}>
              <option value="followSystem">اتباع إعداد النظام</option>
              <option value="reduce">تقليل الحركة</option>
            </select>
          </label>
          <label>
            عدد البطاقات في الصفحة
            <select name="pageSize" defaultValue={initial.pageSize}>
              {[10, 20, 50, 100].map((n) => (
                <option value={n} key={n}>
                  {n}
                </option>
              ))}
            </select>
          </label>
        </fieldset>
        <Disclosure title="إعدادات متقدمة">
          <fieldset disabled={busy}>
            <legend>حدود الجدولة العامة</legend>
            <Field
              name="minIntervalSeconds"
              label="الحد الأدنى للفاصل (ثانية)"
              type="number"
              min={10}
              max={3600}
              step={1}
              defaultValue={initial.minIntervalSeconds}
              error={error}
              help="من ١٠ إلى ٣٦٠٠ ثانية. تُطبّق الحدود على اقتراح FSRS نفسه، دون جدول تعلّم منفصل."
            />
            <Field
              name="maxIntervalDays"
              label="الحد الأقصى للفاصل (يوم)"
              type="number"
              min={1}
              max={36500}
              step={1}
              defaultValue={initial.maxIntervalDays}
              error={error}
              help="من يوم واحد إلى ٣٦٥٠٠ يوم."
            />
          </fieldset>
        </Disclosure>
        <div className="actions">
          <button className="primary" type="submit" disabled={busy}>
            حفظ الإعدادات
          </button>
          <button type="button" disabled={busy} onClick={() => void defaults()}>
            استعادة القيم الافتراضية
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={() => {
              setInitial(settings);
              setFormKey((n) => n + 1);
              setError(null);
              setDirty(false);
            }}
          >
            التراجع إلى القيم المحفوظة
          </button>
        </div>
        <p className="muted">
          استعادة القيم الافتراضية تعدّل الحقول فقط. اضغط حفظ الإعدادات لتطبيقها.
        </p>
      </form>
    </>
  );
}
export function SettingsPage({
  api,
  native,
  navigate,
  onSaved,
  onRestored,
}: {
  api: Api;
  native: NativeApi;
  navigate: Navigate;
  onSaved?: () => void | Promise<void>;
  onRestored?: () => void | Promise<void>;
}) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [info, setInfo] = useState<SystemInfo | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [message, setMessage] = useState('');
  useEffect(() => {
    let current = true;
    void api
      .call('bootstrap', {})
      .then((b) => {
        if (current) setSettings(b.settings);
      })
      .catch((e) => {
        if (current) setError(e);
      });
    void native
      .getSystemInfo()
      .then((i) => {
        if (current) setInfo(i);
      })
      .catch((e) => {
        if (current) setError(e);
      });
    return () => {
      current = false;
    };
  }, [api, native]);
  void navigate;
  return (
    <>
      <h1 tabIndex={-1}>الإعدادات</h1>
      <StatusRegion message={message} />
      <FormErrors error={error} />
      {settings ? (
        <SettingsForm
          key={settings.revision}
          settings={settings}
          api={api}
          onSaved={(s) => {
            setSettings(s);
            setMessage('حُفظت الإعدادات.');
            void Promise.resolve(onSaved?.()).catch(setError);
          }}
        />
      ) : (
        <p role="status">جارٍ تحميل الإعدادات…</p>
      )}
      <BackupPanel
        native={native}
        onRestored={onRestored ?? (() => navigate({ page: 'library' }))}
      />
      <Disclosure title="معلومات التطبيق">
        {info ? (
          <>
            <Field
              name="appVersion"
              label="إصدار التطبيق"
              value={info.appVersion}
              readOnly
              help="يتغير عند تثبيت إصدار جديد من التطبيق."
            />
            <Field
              name="engineVersion"
              label="محرك الجدولة"
              value={info.engineVersion}
              readOnly
              help="المحرك مثبت على FSRS-7. تغييره يتطلب تحديثًا موثقًا واختبارات توافق."
            />
            <Field
              name="engineRevision"
              label="مراجعة مصدر المحرك"
              value={info.engineRevision}
              readOnly
              dir="ltr"
            />
            <Field
              name="schemaVersion"
              label="إصدار قاعدة البيانات"
              value={info.schemaVersion}
              readOnly
            />
            <Field
              name="dataDirectory"
              label="مجلد البيانات"
              value={info.dataDirectory}
              readOnly
              dir="ltr"
              help="المسار يحدده نظام التشغيل. استخدم التصدير والاستيراد لنقل مكتبتك بأمان."
            />
            <Field name="interfaceLanguage" label="لغة الواجهة" value="العربية" readOnly />
            <Field name="theme" label="المظهر" value="داكن" readOnly />
          </>
        ) : (
          <p>معلومات النسخة غير متاحة حاليًا.</p>
        )}
      </Disclosure>
    </>
  );
}
