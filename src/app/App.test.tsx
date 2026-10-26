import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from './App';
import { fixtureApi, nativeFixture, deckFixture } from '../test/mockApi';
import { questionFixture } from '../test/reviewFixtures';

it('keeps a new collection draft from opening while review startup is pending', async () => {
  let finish!: (value: typeof questionFixture) => void;
  const pending = new Promise<typeof questionFixture>((resolve) => {
    finish = resolve;
  });
  const api = fixtureApi(async (r) => {
    if (r.kind === 'startSession') return { kind: r.kind, payload: await pending };
    if (r.kind === 'readSession') return { kind: r.kind, payload: questionFixture };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
  });
  const user = userEvent.setup();
  render(<App api={api} native={nativeFixture} />);
  await user.click(await screen.findByRole('button', { name: 'بدء المراجعة، طب' }));
  expect(screen.getByRole('button', { name: 'إنشاء مجموعة' })).toBeDisabled();
  await act(async () => finish(questionFixture));
});

it.each(['library', 'deck'])(
  'ignores a late review start from the obsolete %s page and preserves a settings draft',
  async (origin) => {
    let finish!: (value: typeof questionFixture) => void;
    const pending = new Promise<typeof questionFixture>((resolve) => {
      finish = resolve;
    });
    const api = fixtureApi(async (r) => {
      if (r.kind === 'getDeck') return { kind: r.kind, payload: deckFixture };
      if (r.kind === 'startSession') return { kind: r.kind, payload: await pending };
      if (r.kind === 'readSession') return { kind: r.kind, payload: questionFixture };
      if (r.kind === 'setSessionActivity')
        return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
    });
    const user = userEvent.setup();
    render(<App api={api} native={nativeFixture} />);
    await screen.findByRole('button', { name: 'إنشاء مجموعة' });
    if (origin === 'deck') await user.click(screen.getByRole('button', { name: 'طب' }));
    await user.click(
      await screen.findByRole('button', {
        name: origin === 'deck' ? 'بدء المراجعة' : 'بدء المراجعة، طب',
      }),
    );
    await user.click(screen.getByRole('button', { name: 'الإعدادات' }));
    const zone = await screen.findByLabelText('المنطقة الزمنية');
    await user.clear(zone);
    await user.type(zone, 'Europe/Paris');
    await act(async () => finish(questionFixture));
    expect(screen.getByLabelText('المنطقة الزمنية')).toHaveValue('Europe/Paris');
    expect(screen.queryByRole('heading', { name: 'السؤال' })).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'المجموعات' }));
    expect(screen.getByRole('dialog', { name: 'تغييرات غير محفوظة' })).toBeVisible();
  },
);

it('honors a newly dirty card editor when a review start from the same page finishes', async () => {
  let finish!: (value: typeof questionFixture) => void;
  const pending = new Promise<typeof questionFixture>((resolve) => {
    finish = resolve;
  });
  const api = fixtureApi(async (r) => {
    if (r.kind === 'getDeck') return { kind: r.kind, payload: deckFixture };
    if (r.kind === 'startSession') return { kind: r.kind, payload: await pending };
    if (r.kind === 'readSession') return { kind: r.kind, payload: questionFixture };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
  });
  const user = userEvent.setup();
  render(<App api={api} native={nativeFixture} />);
  await user.click(await screen.findByRole('button', { name: 'طب' }));
  await user.click(await screen.findByRole('button', { name: 'بدء المراجعة' }));
  await user.click(screen.getByRole('button', { name: 'إنشاء بطاقة' }));
  await user.type(
    within(screen.getByRole('dialog', { name: 'إنشاء بطاقة' })).getAllByRole('textbox')[0],
    'مسودة سؤال',
  );
  await act(async () => finish(questionFixture));
  const guard = screen.getByRole('dialog', { name: 'تغييرات غير محفوظة' });
  await user.click(within(guard).getByRole('button', { name: 'متابعة التعديل' }));
  expect(
    within(screen.getByRole('dialog', { name: 'إنشاء بطاقة' })).getAllByRole('textbox')[0],
  ).toHaveValue('مسودة سؤال');
});

it('keeps restore completion and the safety export available after returning to the library', async () => {
  const safetyBackup = {
    exportId: 'safety-id',
    path: 'Downloads/safety.json',
    bytes: 12,
    payloadSha256: 'sha',
  };
  const native = {
    ...nativeFixture,
    pickBackupForPreview: vi.fn().mockResolvedValue({
      id: 'preview',
      expectedLibraryRevision: 'revision',
      exportedAt: '2026-10-25T00:00:00Z',
      cards: 1,
      forms: 2,
      decks: 1,
      reviews: 0,
      profiles: 1,
      formatVersion: 1,
    }),
    restoreBackup: vi.fn().mockResolvedValue({ safetyBackup, cards: 1, decks: 1 }),
    revealExport: vi.fn().mockResolvedValue(undefined),
  };
  const user = userEvent.setup();
  render(<App api={fixtureApi()} native={native} />);
  await screen.findByRole('button', { name: 'إنشاء مجموعة' });
  await user.click(screen.getByRole('button', { name: 'الإعدادات' }));
  await user.click(await screen.findByText('النسخ الاحتياطي والاستعادة'));
  await user.click(screen.getByRole('button', { name: 'اختيار نسخة للاستعادة' }));
  const dialog = await screen.findByRole('dialog', { name: 'معاينة الاستعادة' });
  await user.click(within(dialog).getByLabelText('أوافق على استبدال جميع بيانات المكتبة الحالية'));
  await user.click(within(dialog).getByRole('button', { name: 'استعادة النسخة واستبدال المكتبة' }));
  await screen.findByRole('button', { name: 'إنشاء مجموعة' });
  expect(screen.getByText('تمت استعادة النسخة الاحتياطية.')).toHaveAttribute('role', 'status');
  expect(screen.getByText(safetyBackup.path)).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'إظهار نسخة الأمان السابقة للاستعادة' }));
  expect(native.revealExport).toHaveBeenCalledWith({ exportId: 'safety-id' });
});
