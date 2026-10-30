import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ReviewPage } from '../review/ReviewPage';
import { fixtureApi, nativeFixture, bootstrapFixture } from '../../test/mockApi';
import { spellingDeck, spellingQuestion, sourceContext } from '../../test/spellingFixtures';
import { completedFixture } from '../../test/reviewFixtures';
import type { Request } from '../../shared/generated/Request';
import type { Response } from '../../shared/generated/Response';
import type { ReviewView } from '../../shared/generated/ReviewView';
import { App } from '../../app/App';
const marker = 'private-attempt-marker';
function reviewApi(
  handler?: (r: Request) => Response | undefined | Promise<Response | undefined>,
  calls: Request[] = [],
) {
  return fixtureApi(async (r) => {
    calls.push(r);
    const response = await handler?.(r);
    if (response) return response;
    if (r.kind === 'readSession' || r.kind === 'startSession')
      return { kind: r.kind, payload: spellingQuestion };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
    if (r.kind === 'listSpellingSources')
      return { kind: r.kind, payload: { items: [sourceContext], total: 1, page: 1, pageSize: 20 } };
  });
}
it('shows a manually readable whole word, focused label, plain field and direct intervals', async () => {
  const calls: Request[] = [];
  const user = userEvent.setup();
  const api = reviewApi(
    (r) => (r.kind === 'rateCard' ? { kind: r.kind, payload: completedFixture } : undefined),
    calls,
  );
  render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
  const field = await screen.findByRole('textbox', { name: 'مساحة الكتابة' });
  expect(screen.getByRole('heading', { level: 1, name: 'الكلمة' })).toHaveFocus();
  const word = screen.getByText('English', { exact: true });
  expect(word.childNodes).toHaveLength(1);
  expect(word.firstChild?.nodeType).toBe(Node.TEXT_NODE);
  expect(word).toHaveAttribute('lang', 'en');
  expect(word).toHaveAttribute('dir', 'ltr');
  expect(word.closest('[aria-live]')).toBeNull();
  for (const [key, value] of [
    ['spellcheck', 'false'],
    ['autocorrect', 'off'],
    ['autocomplete', 'off'],
    ['autocapitalize', 'off'],
  ])
    expect(field).toHaveAttribute(key, value);
  expect(screen.getByText(/تختفي الكتابة بمجرد مغادرة البطاقة/)).toBeVisible();
  expect(screen.queryByRole('button', { name: 'عرض الإجابة' })).not.toBeInTheDocument();
  expect(screen.getByText('عرض البطاقة الأصلية').closest('details')).not.toHaveAttribute('open');
  expect(calls.some((r) => r.kind === 'listSpellingSources')).toBe(false);
  expect(screen.getByRole('button', { name: /افتكرتها بسهولة.*بعد نحو ٣٠ يوم/ })).toBeEnabled();
  await user.click(screen.getByRole('button', { name: /افتكرتها بسهولة/ }));
  await screen.findByRole('heading', { name: 'ملخص الجلسة' });
  expect(calls.some((r) => r.kind === 'rateCard')).toBe(true);
});

it('retains only the same DOM field across blur, context, polling and pending or failed grades', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  let reject!: (error: unknown) => void;
  const api = reviewApi(
    (r) =>
      r.kind === 'rateCard'
        ? new Promise<Response>((_, fail) => {
            reject = fail;
          })
        : undefined,
    calls,
  );
  const local = vi.spyOn(Storage.prototype, 'setItem');
  try {
    render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
    const field = await screen.findByRole('textbox', { name: 'مساحة الكتابة' });
    await user.type(field, marker);
    await user.click(screen.getByText('عرض البطاقة الأصلية'));
    await screen.findByText('معنى محفوظ');
    expect(field).toHaveValue(marker);
    await act(async () => {
      window.dispatchEvent(new Event('focus'));
    });
    expect(screen.getByRole('textbox', { name: 'مساحة الكتابة' })).toBe(field);
    expect(field).toHaveValue(marker);
    await user.click(screen.getByRole('button', { name: /افتكرتها بسهولة/ }));
    expect(field).toHaveValue(marker);
    await act(async () => reject({ code: 'storage' }));
    await screen.findByRole('alert');
    expect(screen.getByRole('textbox', { name: 'مساحة الكتابة' })).toBe(field);
    expect(field).toHaveValue(marker);
    expect(JSON.stringify(calls)).not.toContain(marker);
    expect(local.mock.calls).toHaveLength(0);
  } finally {
    local.mockRestore();
  }
});

it.each(['advance', 'summary'])(
  'destroys attempts after %s and restores an empty field on undo',
  async (next) => {
    const user = userEvent.setup();
    const calls: Request[] = [];
    const api = reviewApi((r) => {
      if (r.kind === 'rateCard')
        return {
          kind: r.kind,
          payload:
            next === 'summary'
              ? completedFixture
              : {
                  ...spellingQuestion,
                  payload: {
                    ...spellingQuestion.payload,
                    presentationId: 'presentation-2',
                    canUndo: true,
                  },
                },
        };
      if (r.kind === 'undoReview') return { kind: r.kind, payload: spellingQuestion };
    }, calls);
    render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
    const first = await screen.findByRole('textbox', { name: 'مساحة الكتابة' });
    await user.type(first, marker);
    await user.click(screen.getByRole('button', { name: /افتكرتها بسهولة/ }));
    if (next === 'summary') await screen.findByRole('heading', { name: 'ملخص الجلسة' });
    else {
      await waitFor(() =>
        expect(screen.getByRole('textbox', { name: 'مساحة الكتابة' })).not.toBe(first),
      );
      expect(screen.getByRole('textbox', { name: 'مساحة الكتابة' })).toHaveValue('');
    }
    await user.click(screen.getByRole('button', { name: 'التراجع عن آخر تقييم' }));
    expect(await screen.findByRole('textbox', { name: 'مساحة الكتابة' })).toHaveValue('');
    expect(JSON.stringify(calls)).not.toContain(marker);
  },
);

it('preserves input for a same-card filter but destroys it when parked and returned', async () => {
  const user = userEvent.setup();
  const api = reviewApi((r) => {
    if (r.kind === 'setReviewFilter') {
      const payload = { ...spellingQuestion.payload, filter: r.payload.filter };
      const view: ReviewView =
        r.payload.filter === 'previouslyReviewed'
          ? {
              state: 'emptyFilter',
              payload: {
                sessionId: 'session-1',
                filter: r.payload.filter,
                nextDueAt: null,
                lastReview: null,
                progress: payload.progress,
                canUndo: false,
              },
            }
          : { state: 'question', payload };
      return { kind: r.kind, payload: view };
    }
  });
  render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
  const field = await screen.findByRole('textbox', { name: 'مساحة الكتابة' });
  await user.type(field, marker);
  await user.click(screen.getByLabelText('البطاقات الجديدة فقط'));
  expect(screen.getByRole('textbox', { name: 'مساحة الكتابة' })).toBe(field);
  expect(field).toHaveValue(marker);
  await user.click(screen.getByLabelText('بطاقات سبق تقييمها'));
  await screen.findByRole('heading', { name: 'لا توجد بطاقات مطابقة الآن' });
  expect(screen.queryByRole('textbox', { name: 'مساحة الكتابة' })).not.toBeInTheDocument();
  await user.click(screen.getByLabelText('كل البطاقات المستحقة'));
  expect(await screen.findByRole('textbox', { name: 'مساحة الكتابة' })).toHaveValue('');
});

it('leaves textarea editing shortcuts intact and rates outside it without revealing', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  const api = reviewApi(
    (r) => (r.kind === 'rateCard' ? { kind: r.kind, payload: completedFixture } : undefined),
    calls,
  );
  render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
  const field = await screen.findByRole('textbox', { name: 'مساحة الكتابة' });
  await user.type(field, '1234');
  await user.keyboard('{Control>}z{/Control}');
  expect(
    calls.some(
      (r) => r.kind === 'rateCard' || r.kind === 'undoReview' || r.kind === 'revealAnswer',
    ),
  ).toBe(false);
  screen.getByRole('heading', { name: 'الكلمة' }).focus();
  fireEvent.keyDown(screen.getByRole('heading', { name: 'الكلمة' }), { key: ' ', code: 'Space' });
  expect(calls.some((r) => r.kind === 'revealAnswer')).toBe(false);
  fireEvent.keyDown(screen.getByRole('heading', { name: 'الكلمة' }), { key: '4' });
  await screen.findByRole('heading', { name: 'ملخص الجلسة' });
  expect(calls.filter((r) => r.kind === 'rateCard')).toHaveLength(1);
});

it('discards practice text on navigation and resume without treating it as unsaved data', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  const api = reviewApi((r) => {
    if (r.kind === 'bootstrap')
      return { kind: r.kind, payload: { ...bootstrapFixture, decks: [spellingDeck] } };
    if (r.kind === 'closeSession') return { kind: r.kind, payload: { id: 'session-1' } };
  }, calls);
  const page = render(<App api={api} native={nativeFixture} />);
  await user.click(
    await screen.findByRole('button', { name: `بدء المراجعة، ${spellingDeck.name}` }),
  );
  await user.type(await screen.findByRole('textbox', { name: 'مساحة الكتابة' }), marker);
  await user.click(screen.getByRole('button', { name: 'المجموعات' }));
  await screen.findByRole('heading', { name: 'مجموعاتي' });
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: `بدء المراجعة، ${spellingDeck.name}` }));
  expect(await screen.findByRole('textbox', { name: 'مساحة الكتابة' })).toHaveValue('');
  await user.type(screen.getByRole('textbox', { name: 'مساحة الكتابة' }), marker);
  page.unmount();
  render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
  expect(await screen.findByRole('textbox', { name: 'مساحة الكتابة' })).toHaveValue('');
  expect(JSON.stringify(calls)).not.toContain(marker);
});
