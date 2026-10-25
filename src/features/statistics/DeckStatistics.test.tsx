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
