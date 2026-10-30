import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../../app/App';
import { bootstrapFixture, deckFixture, fixtureApi, nativeFixture } from '../../test/mockApi';
import {
  spellingCard,
  spellingConfig,
  spellingDeck,
  spellingQuestion,
  spellingResult,
} from '../../test/spellingFixtures';
import type { Request } from '../../shared/generated/Request';
import type { Response } from '../../shared/generated/Response';
import { profileFixture } from '../../test/profileFixtures';
const second = { ...deckFixture, id: 'source-2', name: 'مصدر ثان' };
const decks = [deckFixture, second, spellingDeck];
function apiFor(
  calls: Request[],
  extra?: (r: Request) => Response | undefined | Promise<Response | undefined>,
) {
  return fixtureApi(async (r) => {
    calls.push(r);
    const result = await extra?.(r);
    if (result) return result;
    if (r.kind === 'bootstrap') return { kind: r.kind, payload: { ...bootstrapFixture, decks } };
    if (r.kind === 'getDeck') return { kind: r.kind, payload: spellingDeck };
    if (r.kind === 'listDecks') return { kind: r.kind, payload: decks };
    if (r.kind === 'listProfiles') return { kind: r.kind, payload: [profileFixture] };
    if (r.kind === 'getSpellingConfig') return { kind: r.kind, payload: spellingConfig };
    if (r.kind === 'listCards')
      return { kind: r.kind, payload: { items: [spellingCard], total: 1, page: 1, pageSize: 20 } };
    if (r.kind === 'startSession' || r.kind === 'readSession')
      return { kind: r.kind, payload: spellingQuestion };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
  });
}
async function openSource(user: ReturnType<typeof userEvent.setup>) {
  await user.click(await screen.findByRole('button', { name: 'فتح مجموعة التدرب على الكتابة' }));
  await user.click(await screen.findByText('مصدر كلمات الكتابة'));
  return screen.findByLabelText('المجموعة المصدر');
}
async function openPolicy(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByText('الجدولة والتحسين'));
  return screen.findByRole('spinbutton', { name: 'أقل فاصل لهذه المجموعة (ثانية)' });
}
const exclusion = {
  deckId: spellingDeck.id,
  canonical: 'english',
  excludedAt: '2026-10-24T12:00:00Z',
};
it('prevents native session creation for an unsaved source and preserves the draft after cancelled navigation', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  render(<App api={apiFor(calls)} native={nativeFixture} />);
  const select = await openSource(user);
  await user.selectOptions(select, second.id);
  const start = screen.getByRole('button', { name: 'بدء المراجعة' });
  expect(start).toBeDisabled();
  await user.click(start);
  expect(calls.some((r) => r.kind === 'startSession')).toBe(false);
  await user.click(screen.getByRole('button', { name: 'المجموعات' }));
  await screen.findByRole('dialog', { name: 'تغييرات غير محفوظة' });
  await user.click(screen.getByRole('button', { name: 'متابعة التعديل' }));
  expect(select).toHaveValue(second.id);
  expect(start).toBeDisabled();
  expect(calls.some((r) => r.kind === 'startSession')).toBe(false);
  await user.click(screen.getByRole('button', { name: 'تجاهل الاختيار وتحديث المصدر' }));
  await waitFor(() => expect(start).toBeEnabled());
  await user.click(start);
  await screen.findByRole('heading', { level: 1, name: 'الكلمة' });
  expect(calls.filter((r) => r.kind === 'startSession')).toHaveLength(1);
});
it('keeps review blocked until source saving and its dependent refresh have both completed', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  let reads = 0;
  let save!: (r: Response) => void;
  let refresh!: (r: Response) => void;
  let config = spellingConfig;
  const api = apiFor(calls, (r) => {
    if (r.kind === 'getDeck' && ++reads > 1)
      return new Promise<Response>((done) => {
        refresh = done;
      });
    if (r.kind === 'setSpellingSource')
      return new Promise<Response>((done) => {
        save = done;
      });
    if (r.kind === 'getSpellingConfig') return { kind: r.kind, payload: config };
  });
  render(<App api={api} native={nativeFixture} />);
  await user.selectOptions(await openSource(user), second.id);
  await user.click(screen.getByRole('button', { name: 'حفظ المصدر' }));
  const start = screen.getByRole('button', { name: 'بدء المراجعة' });
  expect(start).toBeDisabled();
  config = {
    ...config,
    sourceDeckId: second.id,
    sourceDeckName: second.name,
    revision: 'saved-config',
  };
  await act(async () =>
    save({ kind: 'setSpellingSource', payload: { ...spellingResult, config } }),
  );
  await waitFor(() => expect(reads).toBe(2));
  expect(start).toBeDisabled();
  expect(calls.some((r) => r.kind === 'startSession')).toBe(false);
  await act(async () =>
    refresh({ kind: 'getDeck', payload: { ...spellingDeck, dataRevision: 'after-source-save' } }),
  );
  await waitFor(() => expect(start).toBeEnabled());
});
it('blocks word/config mutations for a source draft and blocks review during pending reallow', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  let allow!: (r: Response) => void;
  let allowed = false;
  let config = spellingConfig;
  const api = apiFor(calls, (r) => {
    if (r.kind === 'getSpellingConfig') return { kind: r.kind, payload: config };
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
    if (r.kind === 'allowSpellingWord')
      return new Promise<Response>((done) => {
        allow = done;
      });
  });
  render(<App api={api} native={nativeFixture} />);
  const select = await openSource(user);
  await user.click(screen.getByText('الكلمات المستبعدة'));
  const button = await screen.findByRole('button', { name: 'السماح بجمع english مجددًا' });
  await waitFor(() => expect(button).toBeEnabled());
  await user.selectOptions(select, second.id);
  expect(button).toBeDisabled();
  await user.click(screen.getByText('البطاقات', { exact: true }));
  expect(await screen.findByRole('button', { name: 'حذف البطاقة ١' })).toBeDisabled();
  await user.click(screen.getByRole('button', { name: 'تجاهل الاختيار وتحديث المصدر' }));
  await waitFor(() => expect(button).toBeEnabled());
  await user.click(button);
  const start = screen.getByRole('button', { name: 'بدء المراجعة' });
  expect(start).toBeDisabled();
  expect(select).toBeDisabled();
  expect(calls.some((r) => r.kind === 'startSession')).toBe(false);
  allowed = true;
  config = { ...config, revision: 'after-reallow' };
  await act(async () =>
    allow({ kind: 'allowSpellingWord', payload: { ...spellingResult, config } }),
  );
  await screen.findByText('لا توجد كلمات مستبعدة.');
  await waitFor(() => expect(start).toBeEnabled());
});
it('protects an existing scheduling draft from source, rescan and reallow mutations', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  const api = apiFor(calls, (r) => {
    if (r.kind === 'listSpellingExclusions')
      return {
        kind: r.kind,
        payload: { items: [exclusion], total: 1, page: 1, pageSize: 20 },
      };
  });
  render(<App api={api} native={nativeFixture} />);
  const source = await openSource(user);
  await user.click(screen.getByText('الكلمات المستبعدة'));
  const allow = await screen.findByRole('button', { name: 'السماح بجمع english مجددًا' });
  await waitFor(() => expect(allow).toBeEnabled());
  const policy = await openPolicy(user);
  await user.type(policy, '120');
  expect(policy).toHaveValue(120);
  expect(source).toBeDisabled();
  const rescan = screen.getByRole('button', { name: 'فحص المصدر مجددًا' });
  expect(rescan).toBeDisabled();
  expect(allow).toBeDisabled();
  await user.selectOptions(source, second.id);
  await user.click(rescan);
  await user.click(allow);
  expect(
    calls.some((r) =>
      ['setSpellingSource', 'rescanSpellingSource', 'allowSpellingWord'].includes(r.kind),
    ),
  ).toBe(false);
  expect(policy).toHaveValue(120);
  expect(screen.getByRole('button', { name: 'بدء المراجعة' })).toBeDisabled();
  await user.click(screen.getByRole('button', { name: 'المجموعات' }));
  await screen.findByRole('dialog', { name: 'تغييرات غير محفوظة' });
  await user.click(screen.getByRole('button', { name: 'متابعة التعديل' }));
  expect(policy).toHaveValue(120);
  await user.click(screen.getByRole('button', { name: 'تجاهل تعديلات المجموعة' }));
  await waitFor(() => expect(source).toBeEnabled());
  expect(rescan).toBeEnabled();
  expect(allow).toBeEnabled();
});
it.each(['setSpellingSource', 'rescanSpellingSource', 'allowSpellingWord'] as const)(
  'prevents a new scheduling draft during pending %s and permits editing after vocabulary refresh',
  async (kind) => {
    const user = userEvent.setup();
    const calls: Request[] = [];
    let finish!: (r: Response) => void;
    let deck = spellingDeck;
    let config = spellingConfig;
    const api = apiFor(calls, (r) => {
      if (r.kind === 'getDeck') return { kind: r.kind, payload: deck };
      if (r.kind === 'getSpellingConfig') return { kind: r.kind, payload: config };
      if (r.kind === 'listSpellingExclusions')
        return {
          kind: r.kind,
          payload: { items: [exclusion], total: 1, page: 1, pageSize: 20 },
        };
      if (r.kind === kind)
        return new Promise<Response>((done) => {
          finish = done;
        });
    });
    render(<App api={api} native={nativeFixture} />);
    const source = await openSource(user);
    if (kind === 'setSpellingSource') {
      await user.selectOptions(source, second.id);
      await user.click(screen.getByRole('button', { name: 'حفظ المصدر' }));
    } else if (kind === 'rescanSpellingSource') {
      await user.click(screen.getByRole('button', { name: 'فحص المصدر مجددًا' }));
    } else {
      await user.click(screen.getByText('الكلمات المستبعدة'));
      const allow = await screen.findByRole('button', { name: 'السماح بجمع english مجددًا' });
      await waitFor(() => expect(allow).toBeEnabled());
      await user.click(allow);
    }
    const policy = await openPolicy(user);
    expect(policy).toBeDisabled();
    await user.type(policy, '120');
    expect(policy).toHaveValue(null);
    expect(screen.getByRole('button', { name: 'حفظ إعدادات المجموعة' })).toBeDisabled();
    expect(calls.some((r) => r.kind === 'saveDeckPolicy')).toBe(false);
    deck = { ...deck, revision: 'collected-word', dataRevision: 'collected-word-data' };
    config = { ...config, revision: 'collected-word-config' };
    await act(async () =>
      finish({ kind, payload: { ...spellingResult, deck, config, addedWords: 1 } }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole('spinbutton', { name: 'أقل فاصل لهذه المجموعة (ثانية)' }),
      ).toBeEnabled(),
    );
    const refreshedPolicy = screen.getByRole('spinbutton', {
      name: 'أقل فاصل لهذه المجموعة (ثانية)',
    });
    await user.type(refreshedPolicy, '120');
    expect(refreshedPolicy).toHaveValue(120);
    expect(source).toBeDisabled();
  },
);
