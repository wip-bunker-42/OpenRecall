import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SettingsPage } from './SettingsPage';
import { SchedulingSettings } from './SchedulingSettings';
import { fixtureApi, nativeFixture, bootstrapFixture, deckFixture } from '../../test/mockApi';
import { profileFixture } from '../../test/profileFixtures';
import { DeckPage } from '../cards/DeckPage';
it('preserves collapsed deck-policy drafts and blocks card mutations until save or discard', async () => {
  const api = fixtureApi((r) => {
    if (r.kind === 'getDeck') return { kind: r.kind, payload: deckFixture };
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [profileFixture] };
  });
  const user = userEvent.setup();
  render(<DeckPage deckId="deck-1" api={api} native={nativeFixture} navigate={vi.fn()} />);
  await screen.findByRole('heading', { name: 'طب' });
  await user.click(screen.getByText('الجدولة والتحسين'));
  await user.type(screen.getByLabelText('الاحتفاظ لهذه المجموعة (٪)'), '85');
  expect(screen.getByRole('button', { name: 'إنشاء بطاقة' })).toBeDisabled();
  await user.click(screen.getByText('الجدولة والتحسين'));
  expect(screen.getByRole('button', { name: 'بدء المراجعة' })).toBeDisabled();
  await user.click(screen.getByText('الجدولة والتحسين'));
  expect(screen.getByLabelText('الاحتفاظ لهذه المجموعة (٪)')).toHaveValue(85);
  await user.click(screen.getByRole('button', { name: 'تجاهل تعديلات المجموعة' }));
  expect(screen.getByRole('button', { name: 'إنشاء بطاقة' })).toBeEnabled();
});
it('exposes all settings with approved limits, native read-only information and explicit defaults', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'saveSettings')
      return { kind: r.kind, payload: { ...r.payload.settings, revision: 'saved' } };
  });
  const user = userEvent.setup();
  render(<SettingsPage api={api} native={nativeFixture} navigate={vi.fn()} />);
  expect(await screen.findByLabelText('المنطقة الزمنية')).toHaveValue('Africa/Cairo');
  expect(screen.getByLabelText('بداية يوم المذاكرة')).toHaveValue('01:00');
  expect(screen.getByLabelText('الاحتفاظ المطلوب (٪)')).toHaveValue(90);
  expect(screen.getByLabelText('الاحتفاظ المطلوب (٪)')).toHaveAttribute('min', '70');
  expect(screen.getByLabelText('الاحتفاظ المطلوب (٪)')).toHaveAttribute('max', '99');
  expect(screen.getByLabelText('الحد الأدنى للفاصل (ثانية)')).not.toBeVisible();
  await user.click(screen.getByText('إعدادات متقدمة'));
  expect(screen.getByLabelText('الحد الأدنى للفاصل (ثانية)')).toHaveAttribute('min', '10');
  expect(screen.getByLabelText('الحد الأدنى للفاصل (ثانية)')).toHaveAttribute('max', '3600');
  expect(screen.getByLabelText('الحد الأقصى للفاصل (يوم)')).toHaveValue(36500);
  expect(screen.getByLabelText('تفعيل اختصارات المراجعة')).toBeChecked();
  expect(screen.getByLabelText('حجم النص (٪)')).toHaveAttribute('min', '80');
  expect(screen.getByLabelText('حجم النص (٪)')).toHaveAttribute('max', '200');
  expect(screen.getByLabelText('عدد البطاقات في الصفحة')).toHaveValue('20');
  expect(screen.getByLabelText('الحركة')).toHaveValue('followSystem');
  await user.click(screen.getByText('معلومات التطبيق'));
  expect(await screen.findByLabelText('محرك الجدولة')).toHaveAttribute('readonly');
  expect(screen.getByLabelText('مجلد البيانات')).toHaveValue('test-data');
  await user.clear(screen.getByLabelText('الاحتفاظ المطلوب (٪)'));
  await user.type(screen.getByLabelText('الاحتفاظ المطلوب (٪)'), '69');
  await user.click(screen.getByRole('button', { name: 'حفظ الإعدادات' }));
  expect(screen.getByRole('alert')).toBeVisible();
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'saveSettings' }));
  await user.click(screen.getByRole('button', { name: 'استعادة القيم الافتراضية' }));
  expect(screen.getByLabelText('الاحتفاظ المطلوب (٪)')).toHaveValue(90);
  await user.click(screen.getByRole('button', { name: 'حفظ الإعدادات' }));
  expect(calls).toContainEqual({
    kind: 'saveSettings',
    payload: { settings: bootstrapFixture.settings, expectedRevision: 'settings-revision' },
  });
});
it('saves independent deck overrides and explains shared profile changes before assigning', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [profileFixture] };
    if (r.kind === 'getProfile') return { kind: r.kind, payload: profileFixture };
    if (r.kind === 'saveDeckPolicy') return { kind: r.kind, payload: deckFixture };
  });
  const user = userEvent.setup();
  render(<SchedulingSettings deck={deckFixture} api={api} onSaved={vi.fn()} navigate={vi.fn()} />);
  await screen.findByLabelText('الاحتفاظ لهذه المجموعة (٪)');
  await user.type(screen.getByLabelText('الاحتفاظ لهذه المجموعة (٪)'), '85');
  await user.click(screen.getByRole('button', { name: 'حفظ إعدادات المجموعة' }));
  expect(calls).toContainEqual({
    kind: 'saveDeckPolicy',
    payload: {
      deckId: 'deck-1',
      desiredRetention: 0.85,
      minIntervalSeconds: null,
      maxIntervalDays: null,
      expectedRevision: 'deck-revision',
    },
  });
  expect(screen.getByText(/اترك الحقول فارغة/)).toBeVisible();
});
