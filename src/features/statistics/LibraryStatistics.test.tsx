import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../../app/App';
import { fixtureApi, nativeFixture } from '../../test/mockApi';
import { statisticsFixture } from '../../test/profileFixtures';
import { LibraryStatistics } from './LibraryStatistics';
import { DeckStatistics } from './DeckStatistics';
import type { Response } from '../../shared/generated/Response';
const stats = {
  today: { ...statisticsFixture.today, completedForDay: 2 },
  spellingCompletedToday: 1,
  dailyHistory: Array.from({ length: 21 }, (_, i) => ({
    date: `2026-10-${String(i + 1).padStart(2, '0')}`,
    statistics: { ...statisticsFixture.today, completedForDay: i + 1 },
  })),
};
it('loads collapsed library statistics lazily and pages daily completion counts', async () => {
  const user = userEvent.setup();
  let calls = 0;
  const api = fixtureApi((r) => {
    if (r.kind === 'getLibraryStatistics') {
      calls++;
      return { kind: r.kind, payload: stats };
    }
  });
  render(<App api={api} native={nativeFixture} />);
  await screen.findByRole('heading', { name: 'مجموعاتي' });
  expect(calls).toBe(0);
  await user.click(screen.getByText('إحصاءات المكتبة'));
  await screen.findByText('انتهت مراجعاتها اليوم');
  expect(screen.getByText('انتهت مراجعاتها اليوم').nextElementSibling).toHaveTextContent('٢');
  expect(screen.getByText('منها كلمات الإملاء').nextElementSibling).toHaveTextContent('١');
  expect(screen.getByText(/مهمتان مستقلتان/)).toBeVisible();
  expect(screen.queryByRole('table', { name: 'الإنجاز حسب يوم المذاكرة' })).not.toBeInTheDocument();
  await user.click(screen.getByText('الإنجاز في الأيام السابقة'));
  expect(
    within(screen.getByRole('table', { name: 'الإنجاز حسب يوم المذاكرة' })).getAllByRole('row'),
  ).toHaveLength(21);
  await user.click(screen.getByRole('button', { name: 'الأيام الأقدم' }));
  expect(
    within(screen.getByRole('table', { name: 'الإنجاز حسب يوم المذاكرة' })).getAllByRole('row'),
  ).toHaveLength(2);
  expect(calls).toBe(1);
});
it('refetches native day results on a minute and visible focus without calculating the day in the browser', async () => {
  vi.useFakeTimers();
  let calls = 0;
  const api = fixtureApi((r) =>
    r.kind === 'getLibraryStatistics'
      ? {
          kind: r.kind,
          payload: {
            ...stats,
            today: { ...stats.today, completedForDay: ++calls === 1 ? 2 : 0 },
            spellingCompletedToday: 0,
          },
        }
      : undefined,
  );
  try {
    const view = render(<LibraryStatistics api={api} revision="1" timezone="Africa/Cairo" />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(screen.getByText('انتهت مراجعاتها اليوم').nextElementSibling).toHaveTextContent('٢');
    await act(async () => {
      await vi.advanceTimersByTimeAsync(60000);
    });
    expect(screen.getByText('انتهت مراجعاتها اليوم').nextElementSibling).toHaveTextContent('٠');
    expect(calls).toBe(2);
    await act(async () => {
      window.dispatchEvent(new Event('focus'));
    });
    expect(calls).toBe(3);
    view.unmount();
    await vi.advanceTimersByTimeAsync(120000);
    expect(calls).toBe(3);
  } finally {
    vi.useRealTimers();
  }
});
it('ignores stale results, recovers errors and explains empty history', async () => {
  const user = userEvent.setup();
  let calls = 0;
  let resolve!: (response: Response) => void;
  const api = fixtureApi((r) => {
    if (r.kind === 'getLibraryStatistics') {
      calls++;
      if (calls === 1) throw { code: 'storage' };
      if (calls === 3)
        return new Promise<Response>((done) => {
          resolve = done;
        });
      return {
        kind: r.kind,
        payload: { ...stats, today: { ...stats.today, completedForDay: calls }, dailyHistory: [] },
      };
    }
  });
  render(<LibraryStatistics api={api} revision="1" timezone="Africa/Cairo" />);
  await screen.findByRole('alert');
  await user.click(screen.getByRole('button', { name: 'تحديث إحصاءات المكتبة' }));
  await screen.findByText('انتهت مراجعاتها اليوم');
  await user.click(screen.getByText('الإنجاز في الأيام السابقة'));
  expect(screen.getByText('لا توجد مراجعات محفوظة بعد.')).toBeVisible();
  fireEvent.focus(window);
  await waitFor(() => expect(calls).toBe(3));
  await user.click(screen.getByRole('button', { name: 'تحديث إحصاءات المكتبة' }));
  await waitFor(() =>
    expect(screen.getByText('انتهت مراجعاتها اليوم').nextElementSibling).toHaveTextContent('٤'),
  );
  await act(async () => resolve({ kind: 'getLibraryStatistics', payload: stats }));
  expect(screen.getByText('انتهت مراجعاتها اليوم').nextElementSibling).toHaveTextContent('٤');
});
it.each(['library', 'deck'])(
  'keeps a valid history page when native deletion shrinks %s history',
  async (kind) => {
    const user = userEvent.setup();
    let reads = 0;
    const api = fixtureApi((r) => {
      if (r.kind === 'getLibraryStatistics' || r.kind === 'getDeckStatistics') {
        reads++;
        const dailyHistory = reads === 1 ? stats.dailyHistory : stats.dailyHistory.slice(0, 1);
        return r.kind === 'getLibraryStatistics'
          ? { kind: r.kind, payload: { ...stats, dailyHistory } }
          : { kind: r.kind, payload: { ...statisticsFixture, dailyHistory } };
      }
    });
    render(
      kind === 'library' ? (
        <LibraryStatistics api={api} revision="1" timezone="Africa/Cairo" />
      ) : (
        <DeckStatistics api={api} deckId="deck-1" />
      ),
    );
    await screen.findByText('انتهت مراجعاتها اليوم');
    await user.click(
      screen.getByText(kind === 'library' ? 'الإنجاز في الأيام السابقة' : 'سجل المراجعات اليومي'),
    );
    await user.click(screen.getByRole('button', { name: 'الأيام الأقدم' }));
    await user.click(
      screen.getByRole('button', {
        name: kind === 'library' ? 'تحديث إحصاءات المكتبة' : 'تحديث الإحصاءات',
      }),
    );
    const table = screen.getByRole('table', {
      name: kind === 'library' ? 'الإنجاز حسب يوم المذاكرة' : 'المراجعات حسب يوم المذاكرة',
    });
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'الأيام الأقدم' })).toBeDisabled(),
    );
    expect(within(table).getAllByRole('row')).toHaveLength(2);
    expect(screen.getByRole('button', { name: 'الأيام الأحدث' })).toBeDisabled();
  },
);
