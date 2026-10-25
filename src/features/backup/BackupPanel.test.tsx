import { render, screen, within, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BackupPanel } from './BackupPanel';
import { nativeFixture } from '../../test/mockApi';
const preview = {
  id: 'preview',
  expectedLibraryRevision: 'rev',
  exportedAt: '2026-10-24T00:00:00Z',
  cards: 24,
  forms: 48,
  decks: 3,
  reviews: 100,
  profiles: 2,
  formatVersion: 1,
};
const report = {
  exportId: 'export',
  path: 'Downloads/OpenRecall.json',
  bytes: 1000,
  payloadSha256: 'digest',
};
it('announces export only after finalize and reveals only its export ID', async () => {
  let finish!: (value: typeof report) => void;
  const native = {
    ...nativeFixture,
    exportBackup: vi.fn(
      () =>
        new Promise<typeof report>((resolve) => {
          finish = resolve;
        }),
    ),
    revealExport: vi.fn(),
  };
  const user = userEvent.setup();
  render(<BackupPanel native={native} onRestored={vi.fn()} />);
  await user.click(screen.getByText('النسخ الاحتياطي والاستعادة'));
  await user.click(screen.getByRole('button', { name: 'تصدير جميع البيانات' }));
  expect(screen.queryByText('اكتمل تصدير النسخة الاحتياطية.')).not.toBeInTheDocument();
  finish(report);
  expect(await screen.findByText(report.path)).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'إظهار ملف التصدير' }));
  expect(native.revealExport).toHaveBeenCalledWith({ exportId: 'export' });
});
it('previews counts and requires replacement confirmation; disk failure keeps the preview and UI', async () => {
  const onRestored = vi.fn();
  const native = {
    ...nativeFixture,
    pickBackupForPreview: vi.fn().mockResolvedValue(preview),
    restoreBackup: vi.fn().mockRejectedValue({ code: 'fileAccess' }),
  };
  const user = userEvent.setup();
  render(<BackupPanel native={native} onRestored={onRestored} />);
  await user.click(screen.getByText('النسخ الاحتياطي والاستعادة'));
  await user.click(screen.getByRole('button', { name: 'اختيار نسخة للاستعادة' }));
  const dialog = await screen.findByRole('dialog', { name: 'معاينة الاستعادة' });
  expect(within(dialog).getByText('24')).toBeInTheDocument();
  expect(native.restoreBackup).not.toHaveBeenCalled();
  await user.click(within(dialog).getByLabelText('أوافق على استبدال جميع بيانات المكتبة الحالية'));
  await user.click(within(dialog).getByRole('button', { name: 'استعادة النسخة واستبدال المكتبة' }));
  expect(await screen.findByText(/تعذر فتح الملف أو حفظه/)).toBeInTheDocument();
  expect(onRestored).not.toHaveBeenCalled();
  expect(screen.queryByText('تمت استعادة النسخة الاحتياطية.')).not.toBeInTheDocument();
  expect(screen.getByRole('dialog')).toBeInTheDocument();
});
it('preserves preview on cancelled replacement choice, refreshes conflict, and reloads only after success', async () => {
  const onRestored = vi.fn();
  const native = {
    ...nativeFixture,
    pickBackupForPreview: vi
      .fn()
      .mockResolvedValueOnce(preview)
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce({ ...preview, id: 'fresh', expectedLibraryRevision: 'fresh-rev' }),
    restoreBackup: vi
      .fn()
      .mockRejectedValueOnce({ code: 'conflict' })
      .mockResolvedValueOnce({ cards: 24, decks: 3, safetyBackup: report }),
  };
  const user = userEvent.setup();
  render(<BackupPanel native={native} onRestored={onRestored} />);
  await user.click(screen.getByText('النسخ الاحتياطي والاستعادة'));
  await user.click(screen.getByRole('button', { name: 'اختيار نسخة للاستعادة' }));
  await user.click(screen.getByRole('button', { name: 'اختيار ملف آخر أو تحديث المعاينة' }));
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  await user.click(screen.getByLabelText('أوافق على استبدال جميع بيانات المكتبة الحالية'));
  await user.click(screen.getByRole('button', { name: 'استعادة النسخة واستبدال المكتبة' }));
  await screen.findByText(/تغيرت البيانات منذ/);
  expect(onRestored).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'اختيار ملف آخر أو تحديث المعاينة' }));
  await user.click(screen.getByLabelText('أوافق على استبدال جميع بيانات المكتبة الحالية'));
  await user.click(screen.getByRole('button', { name: 'استعادة النسخة واستبدال المكتبة' }));
  await waitFor(() => expect(onRestored).toHaveBeenCalledTimes(1));
  expect(native.restoreBackup).toHaveBeenLastCalledWith({
    previewId: 'fresh',
    expectedLibraryRevision: 'fresh-rev',
  });
});
