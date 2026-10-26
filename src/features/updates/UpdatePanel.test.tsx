import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { UpdatePanel } from './UpdatePanel';
import { nativeFixture } from '../../test/mockApi';
const available = {
  phase: 'available',
  currentVersion: '0.1.0',
  availableVersion: '0.2.0',
  notes: 'تحسين المراجعة',
  downloadedBytes: 0,
  totalBytes: null,
  errorCode: null,
};
it('keeps a newer progress event when the initial local status response arrives late', async () => {
  let listener!: (status: typeof available) => void;
  let finish!: (status: typeof available) => void;
  const native = {
    ...nativeFixture,
    onUpdateStatus: vi.fn(async (callback: typeof listener) => {
      listener = callback;
      return () => {};
    }),
    getUpdateStatus: vi.fn(
      () =>
        new Promise<typeof available>((resolve) => {
          finish = resolve;
        }),
    ),
  };
  const user = userEvent.setup();
  render(<UpdatePanel native={native} />);
  await waitFor(() => expect(native.getUpdateStatus).toHaveBeenCalled());
  await user.click(screen.getByText('تحديث التطبيق'));
  await act(async () => {
    listener({ ...available, phase: 'ready' });
    finish({ ...available, phase: 'downloading' });
  });
  expect(screen.getByRole('button', { name: 'تثبيت التحديث وإعادة التشغيل' })).toBeEnabled();
});
it('does not check at startup; offline failure permits retry and never installs automatically', async () => {
  const native = {
    ...nativeFixture,
    checkForUpdate: vi
      .fn()
      .mockRejectedValueOnce({ code: 'updateCheckFailed' })
      .mockResolvedValueOnce(available),
    downloadUpdate: vi
      .fn()
      .mockResolvedValue({ ...available, phase: 'ready', downloadedBytes: 4, totalBytes: 4 }),
    installUpdate: vi.fn(),
  };
  const user = userEvent.setup();
  render(<UpdatePanel native={native} />);
  expect(native.checkForUpdate).not.toHaveBeenCalled();
  await user.click(screen.getByText('تحديث التطبيق'));
  expect(native.checkForUpdate).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'التحقق من وجود تحديث' }));
  await screen.findByText(/تعذر التحقق من التحديثات/);
  expect(native.installUpdate).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'التحقق من وجود تحديث' }));
  await screen.findByText('تحسين المراجعة');
  await user.click(screen.getByRole('button', { name: 'تنزيل التحديث' }));
  await screen.findByRole('button', { name: 'تثبيت التحديث وإعادة التشغيل' });
  expect(native.installUpdate).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'تثبيت التحديث وإعادة التشغيل' }));
  const dialog = await screen.findByRole('dialog', { name: 'تثبيت التحديث' });
  expect(dialog).toBeInTheDocument();
  expect(native.installUpdate).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'تثبيت الآن' }));
  await waitFor(() => expect(native.installUpdate).toHaveBeenCalledTimes(1));
});
it('shows no-update status and explains native review guards without closing study data', async () => {
  const native = {
    ...nativeFixture,
    checkForUpdate: vi
      .fn()
      .mockResolvedValueOnce({ ...available, phase: 'upToDate', availableVersion: null })
      .mockResolvedValueOnce(available),
    downloadUpdate: vi.fn().mockResolvedValue({ ...available, phase: 'ready' }),
    installUpdate: vi.fn().mockRejectedValue({ code: 'reviewInProgress' }),
  };
  const user = userEvent.setup();
  render(<UpdatePanel native={native} />);
  await user.click(screen.getByText('تحديث التطبيق'));
  await user.click(screen.getByRole('button', { name: 'التحقق من وجود تحديث' }));
  await screen.findByText('أنت تستخدم أحدث إصدار متاح.');
  await user.click(screen.getByRole('button', { name: 'التحقق من وجود تحديث' }));
  await user.click(await screen.findByRole('button', { name: 'تنزيل التحديث' }));
  await user.click(await screen.findByRole('button', { name: 'تثبيت التحديث وإعادة التشغيل' }));
  await user.click(screen.getByRole('button', { name: 'تثبيت الآن' }));
  expect(await screen.findByText(/أكمل جلسة المراجعة أو أنهها أولًا/)).toBeInTheDocument();
});
