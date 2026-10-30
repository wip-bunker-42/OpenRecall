import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../../app/App';
import { bootstrapFixture, deckFixture, fixtureApi, nativeFixture } from '../../test/mockApi';
import { spellingDeck, spellingResult } from '../../test/spellingFixtures';
import type { Response } from '../../shared/generated/Response';

it('creates from a selected source and subsequently opens the singleton', async () => {
  const user = userEvent.setup();
  let created = false;
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'bootstrap')
      return {
        kind: r.kind,
        payload: {
          ...bootstrapFixture,
          decks: created ? [deckFixture, spellingDeck] : [deckFixture],
        },
      };
    if (r.kind === 'createSpellingDeck') {
      created = true;
      return { kind: r.kind, payload: spellingResult };
    }
    if (r.kind === 'getDeck') return { kind: r.kind, payload: spellingDeck };
  });
  render(<App api={api} native={nativeFixture} />);
  await user.click(await screen.findByRole('button', { name: 'إنشاء مجموعة التدرب على الكتابة' }));
  const dialog = screen.getByRole('dialog');
  expect(within(dialog).getByLabelText('المجموعة المصدر')).toHaveFocus();
  await user.selectOptions(within(dialog).getByLabelText('المجموعة المصدر'), deckFixture.id);
  await user.click(within(dialog).getByRole('button', { name: 'إنشاء وربط المصدر' }));
  expect(
    await screen.findByRole('heading', { level: 1, name: spellingDeck.name }),
  ).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'العودة إلى المجموعات' }));
  expect(
    await screen.findByRole('button', { name: 'فتح مجموعة التدرب على الكتابة' }),
  ).toBeInTheDocument();
  expect(
    screen.queryByRole('button', { name: 'إنشاء مجموعة التدرب على الكتابة' }),
  ).not.toBeInTheDocument();
  expect(calls).toContainEqual({
    kind: 'createSpellingDeck',
    payload: { sourceDeckId: deckFixture.id },
  });
});

it('explains the ordinary source requirement without a usable create action', async () => {
  render(
    <App
      api={fixtureApi((r) =>
        r.kind === 'bootstrap'
          ? { kind: r.kind, payload: { ...bootstrapFixture, decks: [] } }
          : undefined,
      )}
      native={nativeFixture}
    />,
  );
  expect(
    await screen.findByRole('button', { name: 'إنشاء مجموعة التدرب على الكتابة' }),
  ).toBeDisabled();
  expect(screen.getByText(/أنشئ مجموعة عادية أولًا/)).toBeInTheDocument();
});

it('does not navigate away from a newer page when a pending create resolves', async () => {
  const user = userEvent.setup();
  let resolve!: (r: Response) => void;
  const api = fixtureApi((r) =>
    r.kind === 'createSpellingDeck'
      ? new Promise<Response>((done) => {
          resolve = done;
        })
      : undefined,
  );
  render(<App api={api} native={nativeFixture} />);
  await user.click(await screen.findByRole('button', { name: 'إنشاء مجموعة التدرب على الكتابة' }));
  await user.selectOptions(screen.getByLabelText('المجموعة المصدر'), deckFixture.id);
  await user.click(screen.getByRole('button', { name: 'إنشاء وربط المصدر' }));
  await user.click(screen.getByRole('button', { name: 'الإعدادات' }));
  const discard = screen.queryByRole('button', { name: /تجاهل/ });
  if (discard) await user.click(discard);
  await screen.findByRole('heading', { level: 1, name: 'الإعدادات' });
  await act(async () => resolve({ kind: 'createSpellingDeck', payload: spellingResult }));
  await waitFor(() =>
    expect(screen.getByRole('heading', { level: 1, name: 'الإعدادات' })).toBeInTheDocument(),
  );
});
