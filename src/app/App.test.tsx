import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from './App';
import { fixtureApi, nativeFixture } from '../test/mockApi';

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
