import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DeckPage } from '../cards/DeckPage';
import { SpellingSettings } from './SpellingSettings';
import type { Response } from '../../shared/generated/Response';
import { deckFixture, fixtureApi, nativeFixture } from '../../test/mockApi';
import {
  spellingCard,
  spellingConfig,
  spellingDeck,
  spellingResult,
} from '../../test/spellingFixtures';

it('guards generated words and supports source binding, rescan and explicit deletion', async () => {
  const user = userEvent.setup();
  const calls: unknown[] = [];
  const source2 = { ...deckFixture, id: 'source-2', name: 'English 2' };
  let config = spellingConfig;
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'getDeck') return { kind: r.kind, payload: spellingDeck };
    if (r.kind === 'listDecks')
      return { kind: r.kind, payload: [deckFixture, source2, spellingDeck] };
    if (r.kind === 'getSpellingConfig') return { kind: r.kind, payload: config };
    if (r.kind === 'listSpellingExclusions')
      return { kind: r.kind, payload: { items: [], total: 0, page: 1, pageSize: 20 } };
    if (r.kind === 'setSpellingSource') {
      config = {
        ...config,
        sourceDeckId: source2.id,
        sourceDeckName: source2.name,
        revision: 'next-config',
      };
      return { kind: r.kind, payload: { ...spellingResult, config, addedWords: 3 } };
    }
    if (r.kind === 'rescanSpellingSource')
      return { kind: r.kind, payload: { ...spellingResult, config, addedWords: 0 } };
    if (r.kind === 'listCards')
      return { kind: r.kind, payload: { items: [spellingCard], total: 1, page: 1, pageSize: 20 } };
    if (r.kind === 'deleteCard') return { kind: r.kind, payload: { id: spellingCard.id } };
  });
  const props = { deckId: spellingDeck.id, api, native: nativeFixture, navigate: () => {} };
  const view = render(<DeckPage {...props} />);
  await screen.findByRole('heading', { level: 1, name: spellingDeck.name });
  expect(screen.queryByRole('button', { name: 'إنشاء بطاقة' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'استيراد بطاقات JSON' })).not.toBeInTheDocument();
  expect(screen.queryByLabelText('المجموعة المصدر')).not.toBeInTheDocument();
  await user.click(screen.getByText('مصدر كلمات الكتابة'));
  await user.selectOptions(await screen.findByLabelText('المجموعة المصدر'), source2.id);
  view.rerender(<DeckPage {...props} timezone="UTC" />);
  expect(screen.getByLabelText('المجموعة المصدر')).toHaveValue(source2.id);
  await user.click(screen.getByRole('button', { name: 'حفظ المصدر' }));
  await screen.findByText(/أُضيفت ٣ كلمة/);
  await user.click(screen.getByRole('button', { name: 'فحص المصدر مجددًا' }));
  await screen.findByText(/أُضيفت ٠ كلمة/);
  await user.click(screen.getByText('البطاقات', { exact: true }));
  await screen.findAllByText('English');
  expect(screen.queryByRole('button', { name: /تعديل البطاقة/ })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'حذف البطاقة ١' }));
  expect(
    within(screen.getByRole('dialog')).getByText(/لن تُجمع هذه الكلمة مجددًا/),
  ).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'حذف البطاقة نهائيًا' }));
  await waitFor(() =>
    expect(calls).toContainEqual({
      kind: 'deleteCard',
      payload: { cardId: spellingCard.id, expectedRevision: spellingCard.revision },
    }),
  );
  expect(calls).toContainEqual({
    kind: 'rescanSpellingSource',
    payload: { deckId: spellingDeck.id, expectedRevision: 'next-config' },
  });
});

it('keeps last source context when its source has been deleted', async () => {
  const user = userEvent.setup();
  const api = fixtureApi((r) =>
    r.kind === 'getDeck'
      ? { kind: r.kind, payload: spellingDeck }
      : r.kind === 'getSpellingConfig'
        ? { kind: r.kind, payload: { ...spellingConfig, sourceDeckId: null } }
        : undefined,
  );
  render(
    <DeckPage deckId={spellingDeck.id} api={api} native={nativeFixture} navigate={() => {}} />,
  );
  await user.click(await screen.findByText('مصدر كلمات الكتابة'));
  expect(await screen.findByText(/حُذفت المجموعة المصدر/)).toHaveTextContent('طب');
  expect(screen.getByRole('button', { name: 'فحص المصدر مجددًا' })).toBeDisabled();
});
it('preserves a source draft through context updates and ignores an older read after saving', async () => {
  const user = userEvent.setup();
  const source2 = { ...deckFixture, id: 'source-2', name: 'English 2' };
  let reads = 0;
  let resolve!: (response: Response) => void;
  const nextConfig = {
    ...spellingConfig,
    sourceDeckId: source2.id,
    sourceDeckName: source2.name,
    revision: 'config-2',
  };
  const api = fixtureApi((r) => {
    if (r.kind === 'listDecks') return { kind: r.kind, payload: [deckFixture, source2] };
    if (r.kind === 'getSpellingConfig') {
      reads++;
      if (reads === 2)
        return new Promise<Response>((done) => {
          resolve = done;
        });
      return { kind: r.kind, payload: reads > 2 ? nextConfig : spellingConfig };
    }
    if (r.kind === 'setSpellingSource')
      return { kind: r.kind, payload: { ...spellingResult, config: nextConfig } };
  });
  const onChanged = async () => {};
  const view = render(<SpellingSettings api={api} deck={spellingDeck} onChanged={onChanged} />);
  await user.selectOptions(await screen.findByLabelText('المجموعة المصدر'), source2.id);
  view.rerender(
    <SpellingSettings
      api={api}
      deck={{ ...spellingDeck, dataRevision: 'context-update' }}
      onChanged={onChanged}
    />,
  );
  await waitFor(() => expect(reads).toBe(2));
  expect(screen.getByLabelText('المجموعة المصدر')).toHaveValue(source2.id);
  await user.click(screen.getByRole('button', { name: 'حفظ المصدر' }));
  await screen.findByText(/أُضيفت ٢ كلمة/);
  await act(async () => resolve({ kind: 'getSpellingConfig', payload: spellingConfig }));
  expect(screen.getByLabelText('المجموعة المصدر')).toHaveValue(source2.id);
});
it('can retry an initial source load failure', async () => {
  let failed = false;
  const user = userEvent.setup();
  const api = fixtureApi((r) => {
    if (r.kind === 'getSpellingConfig') {
      if (!failed) {
        failed = true;
        throw { code: 'storage' };
      }
      return { kind: r.kind, payload: spellingConfig };
    }
  });
  render(<SpellingSettings api={api} deck={spellingDeck} onChanged={async () => {}} />);
  await screen.findByRole('alert');
  await user.click(screen.getByRole('button', { name: 'تحديث إعدادات المصدر' }));
  expect(await screen.findByLabelText('المجموعة المصدر')).toHaveValue(deckFixture.id);
});
