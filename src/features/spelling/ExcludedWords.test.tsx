import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DeckPage } from '../cards/DeckPage';
import { fixtureApi, nativeFixture } from '../../test/mockApi';
import { spellingConfig, spellingDeck, spellingResult } from '../../test/spellingFixtures';
it('loads excluded words only when opened and explicitly allows collecting them again', async () => {
  const user = userEvent.setup();
  const calls: unknown[] = [];
  let allowed = false;
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'getDeck') return { kind: r.kind, payload: spellingDeck };
    if (r.kind === 'getSpellingConfig') return { kind: r.kind, payload: spellingConfig };
    if (r.kind === 'listSpellingExclusions')
      return {
        kind: r.kind,
        payload: {
          items: allowed
            ? []
            : [
                {
                  deckId: spellingDeck.id,
                  canonical: 'english',
                  excludedAt: '2026-10-24T12:00:00Z',
                },
              ],
          total: allowed ? 0 : 1,
          page: 1,
          pageSize: 20,
        },
      };
    if (r.kind === 'allowSpellingWord') {
      allowed = true;
      return { kind: r.kind, payload: spellingResult };
    }
  });
  render(
    <DeckPage deckId={spellingDeck.id} api={api} native={nativeFixture} navigate={() => {}} />,
  );
  await user.click(await screen.findByText('مصدر كلمات الكتابة'));
  await screen.findByLabelText('المجموعة المصدر');
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'listSpellingExclusions' }));
  await user.click(screen.getByText('الكلمات المستبعدة'));
  await user.click(await screen.findByRole('button', { name: 'السماح بجمع english مجددًا' }));
  await screen.findByText('لا توجد كلمات مستبعدة.');
  await waitFor(() =>
    expect(calls).toContainEqual({
      kind: 'allowSpellingWord',
      payload: {
        deckId: spellingDeck.id,
        canonical: 'english',
        expectedRevision: spellingConfig.revision,
      },
    }),
  );
});
