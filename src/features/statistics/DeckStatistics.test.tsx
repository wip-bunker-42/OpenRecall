import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DeckStatistics } from './DeckStatistics';
import { fixtureApi } from '../../test/mockApi';
import { statisticsFixture } from '../../test/profileFixtures';
it('exposes logical-card/form counts and empty recall metrics in accessible tables', async () => {
  const api = fixtureApi((r) =>
    r.kind === 'getDeckStatistics' ? { kind: r.kind, payload: statisticsFixture } : undefined,
  );
  const user = userEvent.setup();
  render(<DeckStatistics deckId="deck-1" api={api} />);
  expect(await screen.findByText('عدد البطاقات')).toBeVisible();
  expect(screen.getByText('عدد الصيغ')).toBeVisible();
  expect(screen.getByRole('table', { name: 'المراجعة عبر الفترات' })).toHaveTextContent(
    'لا توجد نتائج',
  );
  expect(document.body).not.toHaveTextContent('NaN');
  expect(
    screen.queryByRole('table', { name: 'الاستحقاقات خلال ٣٠ يومًا' }),
  ).not.toBeInTheDocument();
  await user.click(screen.getByText('توقعات الاستحقاق'));
  expect(screen.getByRole('table', { name: 'الاستحقاقات خلال ٣٠ يومًا' })).toBeVisible();
  expect(screen.getByText('إحصاءات هذه المجموعة')).toBeVisible();
  expect(screen.getByText('أهلية النموذج المشترك')).toBeVisible();
});
it('labels spelling units and exposes native completion counts for each period and day', async () => {
  const user = userEvent.setup();
  const period = { ...statisticsFixture.today, completedForDay: 3 };
  const api = fixtureApi((r) =>
    r.kind === 'getDeckStatistics'
      ? {
          kind: r.kind,
          payload: {
            ...statisticsFixture,
            today: period,
            dailyHistory: [{ date: '2026-10-24', statistics: period }],
          },
        }
      : undefined,
  );
  render(<DeckStatistics deckId="spelling-1" deckKind="spelling" api={api} />);
  expect(await screen.findByText('عدد كلمات الإملاء')).toBeVisible();
  expect(screen.getByText('انتهت مراجعاتها اليوم').nextElementSibling).toHaveTextContent('٣');
  await user.click(screen.getByText('سجل المراجعات اليومي'));
  expect(screen.getByRole('table', { name: 'المراجعات حسب يوم المذاكرة' })).toHaveTextContent(
    'انتهت مراجعاتها في اليوم',
  );
  expect(screen.getByText(/كلمات الإملاء مهام مستقلة/)).toBeVisible();
});
