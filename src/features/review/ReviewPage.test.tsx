import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ReviewPage } from './ReviewPage';
import { fixtureApi, nativeFixture, deckFixture } from '../../test/mockApi';
import { questionFixture, revealedFixture, completedFixture } from '../../test/reviewFixtures';
import type { Request } from '../../shared/generated/Request';
import type { Response } from '../../shared/generated/Response';
import { App } from '../../app/App';
import { ReviewQuestion } from './ReviewQuestion';
it('review_uses_custom_h1_with_default_fallback', async () => {
  const user = userEvent.setup();
  const form = { ...revealedFixture.payload.form, questionTitle: 'ترجم', front: 'unpredictable' };
  const props = {
    frame: { ...revealedFixture.payload, form },
    onReveal: vi.fn(),
    onRate: vi.fn(),
    disabled: false,
    timezone: 'Africa/Cairo',
  };
  const { rerender } = render(<ReviewQuestion {...props} />);
  expect(screen.getByRole('heading', { level: 1, name: 'ترجم' })).toHaveAttribute('dir', 'auto');
  expect(screen.queryByRole('heading', { name: 'unpredictable' })).not.toBeInTheDocument();
  expect(screen.getByText('unpredictable')).toBeVisible();
  expect(screen.getByRole('heading', { level: 2, name: 'الإجابة' })).toBeVisible();
  expect(screen.getByRole('heading', { level: 3, name: 'الملاحظات' })).toBeVisible();
  await user.click(screen.getByRole('button', { name: /افتكرتها بسهولة/ }));
  expect(props.onRate).toHaveBeenCalledWith('easy');
  rerender(
    <ReviewQuestion
      {...props}
      frame={{ ...props.frame, form: { ...form, questionTitle: null } }}
    />,
  );
  expect(screen.getByRole('heading', { level: 1, name: 'السؤال' })).toBeVisible();
  rerender(
    <ReviewQuestion
      {...props}
      frame={{ ...props.frame, form: { ...form, questionTitle: '<b>أكمل</b>' } }}
    />,
  );
  expect(screen.getByRole('heading', { level: 1, name: '<b>أكمل</b>' })).toHaveTextContent(
    '<b>أكمل</b>',
  );
  expect(screen.getByRole('heading', { level: 1 }).querySelector('b')).toBeNull();
});
it('locks review actions while navigation waits for the durable session close', async () => {
  let release!: () => void;
  const api = fixtureApi(async (r) => {
    if (r.kind === 'startSession' || r.kind === 'readSession')
      return { kind: r.kind, payload: questionFixture };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
    if (r.kind === 'closeSession')
      return new Promise<Response>((resolve) => {
        release = () => resolve({ kind: 'closeSession', payload: { id: 'session-1' } });
      });
  });
  const user = userEvent.setup();
  render(<App api={api} native={nativeFixture} />);
  await user.click(await screen.findByRole('button', { name: 'بدء المراجعة، طب' }));
  await screen.findByRole('heading', { name: 'السؤال' });
  await user.click(screen.getByRole('button', { name: 'المجموعات' }));
  expect(screen.getByRole('button', { name: 'عرض الإجابة' })).toBeDisabled();
  await act(async () => release());
  await screen.findByRole('heading', { name: 'مجموعاتي' });
});
it.each(['library', 'deck'])(
  'starts default-all reviews from %s and closes undo on explicit navigation',
  async (origin) => {
    const calls: Request[] = [];
    const api = fixtureApi((r) => {
      calls.push(r);
      if (r.kind === 'getDeck') return { kind: r.kind, payload: deckFixture };
      if (r.kind === 'startSession' || r.kind === 'readSession')
        return { kind: r.kind, payload: questionFixture };
      if (r.kind === 'setSessionActivity')
        return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
      if (r.kind === 'closeSession') return { kind: r.kind, payload: { id: 'session-1' } };
    });
    const user = userEvent.setup();
    render(<App api={api} native={nativeFixture} />);
    if (origin === 'deck') {
      await user.click(await screen.findByRole('button', { name: 'طب' }));
      await user.click(await screen.findByRole('button', { name: 'بدء المراجعة' }));
    } else await user.click(await screen.findByRole('button', { name: 'بدء المراجعة، طب' }));
    await screen.findByRole('heading', { name: 'السؤال' });
    expect(screen.getByLabelText('كل البطاقات المستحقة')).toBeChecked();
    await user.click(screen.getByRole('button', { name: 'المجموعات' }));
    await screen.findByRole('heading', { name: 'مجموعاتي' });
    expect(calls).toContainEqual({ kind: 'startSession', payload: { deckId: 'deck-1' } });
    expect(calls).toContainEqual({ kind: 'closeSession', payload: { sessionId: 'session-1' } });
  },
);
it('keeps the question until durable success and ignores a second pending click', async () => {
  const calls: Request[] = [];
  let release: (() => void) | null = null;
  const api = fixtureApi(async (r) => {
    calls.push(r);
    if (r.kind === 'readSession') return { kind: r.kind, payload: questionFixture };
    if (r.kind === 'revealAnswer') return { kind: r.kind, payload: revealedFixture };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
    if (r.kind === 'rateCard')
      return new Promise<Response>((resolve) => {
        release = () => resolve({ kind: 'rateCard', payload: completedFixture });
      });
  });
  const user = userEvent.setup();
  render(<ReviewPage sessionId="session-1" api={api} navigate={vi.fn()} />);
  await user.click(await screen.findByRole('button', { name: 'عرض الإجابة' }));
  await user.dblClick(screen.getByRole('button', { name: /افتكرتها بسهولة/ }));
  expect(screen.getByRole('heading', { name: 'السؤال' })).toBeVisible();
  expect(screen.getByRole('button', { name: /افتكرتها بسهولة/ })).toBeDisabled();
  expect(calls.filter((r) => r.kind === 'rateCard')).toHaveLength(1);
  await act(async () => {
    release!();
  });
  expect(await screen.findByRole('heading', { name: 'ملخص الجلسة' })).toBeVisible();
});
function reviewApi(calls: Request[] = []) {
  return fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'readSession' || r.kind === 'resumeSession')
      return { kind: r.kind, payload: questionFixture };
    if (r.kind === 'revealAnswer' || r.kind === 'undoReview')
      return { kind: r.kind, payload: revealedFixture };
    if (r.kind === 'rateCard' || r.kind === 'endSession')
      return { kind: r.kind, payload: completedFixture };
    if (r.kind === 'setReviewFilter')
      return {
        kind: r.kind,
        payload:
          r.payload.filter === 'previouslyReviewed'
            ? {
                state: 'emptyFilter',
                payload: {
                  sessionId: 'session-1',
                  filter: 'previouslyReviewed',
                  nextDueAt: null,
                  progress: { current: 0, total: 1, completed: 0 },
                  canUndo: false,
                  lastReview: null,
                },
              }
            : {
                ...revealedFixture,
                payload: { ...revealedFixture.payload, filter: r.payload.filter },
              },
      };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
  });
}
it('reveals label-only headings with correct focus, accessible intervals, summary and exact undo', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  render(<ReviewPage sessionId="session-1" api={reviewApi(calls)} navigate={vi.fn()} />);
  const question = await screen.findByRole('heading', { level: 1, name: 'السؤال' });
  expect(question).toHaveFocus();
  expect(screen.queryByText('English')).not.toBeInTheDocument();
  expect(screen.getByText('<h1>مرحبا</h1>')).toHaveAttribute('dir', 'auto');
  await user.click(screen.getByRole('button', { name: 'عرض الإجابة' }));
  expect(screen.getByRole('heading', { level: 2, name: 'الإجابة' })).toHaveFocus();
  expect(screen.getByRole('heading', { level: 3, name: 'الملاحظات' })).toBeVisible();
  expect(question).toBeVisible();
  expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
  for (const label of [
    /نسيت.*دقيقة/,
    /افتكرتها بصعوبة.*ساعة/,
    /افتكرتها بشكل طبيعي.*يوم/,
    /افتكرتها بسهولة.*٣٠ يوم/,
  ])
    expect(screen.getByRole('button', { name: label })).toHaveAccessibleDescription(
      /الموعد المتوقع/,
    );
  await user.click(screen.getByRole('button', { name: /افتكرتها بسهولة.*٣٠ يوم/ }));
  expect(await screen.findByRole('heading', { level: 1, name: 'ملخص الجلسة' })).toHaveFocus();
  expect(screen.getByText(/موعد المراجعة الجديد/)).toBeVisible();
  await user.keyboard('{Shift>}{Tab}{/Shift}');
  expect(screen.getByRole('button', { name: 'التراجع عن آخر تقييم' })).toHaveFocus();
  await user.keyboard('{Control>}z{/Control}');
  expect(await screen.findByRole('heading', { level: 2, name: 'الإجابة' })).toHaveFocus();
  expect(calls.filter((r) => r.kind === 'rateCard')).toHaveLength(1);
});
it('switches empty filters and restores the reserved revealed form at a cycle boundary', async () => {
  const user = userEvent.setup();
  render(<ReviewPage sessionId="session-1" api={reviewApi()} navigate={vi.fn()} />);
  await screen.findByRole('heading', { name: 'السؤال' });
  expect(screen.getByLabelText('كل البطاقات المستحقة')).toBeChecked();
  await user.click(screen.getByRole('button', { name: 'عرض الإجابة' }));
  await user.click(screen.getByLabelText('بطاقات سبق تقييمها'));
  expect(await screen.findByRole('heading', { name: 'لا توجد بطاقات مطابقة الآن' })).toBeVisible();
  await user.click(screen.getByLabelText('كل البطاقات المستحقة'));
  expect(await screen.findByRole('heading', { name: 'الإجابة' })).toBeVisible();
  expect(screen.getByText('<h1>مرحبا</h1>')).toBeVisible();
});
it('suppresses duplicate ratings and retries the same durable action token after an error', async () => {
  const calls: Request[] = [];
  let rates = 0;
  const base = reviewApi();
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'rateCard') {
      if (++rates === 1) throw { code: 'storage' };
      return { kind: r.kind, payload: completedFixture };
    }
    return undefined;
  });
  api.call = async (kind, payload) => {
    if (kind === 'rateCard')
      return fixtureApi((r) => {
        calls.push(r);
        if (r.kind === 'rateCard') {
          if (++rates === 1) throw { code: 'storage' };
          return { kind: r.kind, payload: completedFixture };
        }
      }).call(kind, payload);
    return base.call(kind, payload);
  };
  const user = userEvent.setup();
  render(<ReviewPage sessionId="session-1" api={api} navigate={vi.fn()} />);
  await user.click(await screen.findByRole('button', { name: 'عرض الإجابة' }));
  await user.dblClick(screen.getByRole('button', { name: /افتكرتها بسهولة/ }));
  await screen.findByRole('alert');
  await user.click(screen.getByRole('button', { name: 'إعادة محاولة التقييم' }));
  await screen.findByRole('heading', { name: 'ملخص الجلسة' });
  const rated = calls.filter((r) => r.kind === 'rateCard');
  expect(rated).toHaveLength(2);
  expect(rated[1]).toEqual(rated[0]);
});
it('guards text inputs and disabled shortcuts while enabling explicit review keys', async () => {
  const user = userEvent.setup();
  const calls: Request[] = [];
  const { rerender } = render(
    <>
      <input aria-label="خارج المراجعة" />
      <ReviewPage
        sessionId="session-1"
        api={reviewApi(calls)}
        navigate={vi.fn()}
        shortcutsEnabled={false}
      />
    </>,
  );
  await screen.findByRole('heading', { name: 'السؤال' });
  await user.keyboard(' ');
  expect(calls.some((r) => r.kind === 'revealAnswer')).toBe(false);
  const api = reviewApi(calls);
  rerender(
    <>
      <input aria-label="خارج المراجعة" />
      <ReviewPage sessionId="session-1" api={api} navigate={vi.fn()} shortcutsEnabled />
    </>,
  );
  await waitFor(() => expect(screen.getByRole('heading', { name: 'السؤال' })).toBeVisible());
  await user.click(screen.getByLabelText('خارج المراجعة'));
  await user.keyboard(' 1234');
  expect(calls.some((r) => r.kind === 'revealAnswer' || r.kind === 'rateCard')).toBe(false);
  await user.click(screen.getByRole('heading', { name: 'السؤال' }));
  await user.keyboard(' ');
  expect(await screen.findByRole('heading', { name: 'الإجابة' })).toBeVisible();
  await user.keyboard('4');
  expect(await screen.findByRole('heading', { name: 'ملخص الجلسة' })).toBeVisible();
});
it('reports background refresh failures without stealing keyboard focus', async () => {
  let reads = 0;
  const api = fixtureApi((r) => {
    if (r.kind === 'readSession') {
      if (++reads > 1) throw { code: 'storage' };
      return { kind: r.kind, payload: questionFixture };
    }
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
  });
  vi.useFakeTimers();
  const { unmount } = render(<ReviewPage sessionId="session-1" api={api} navigate={vi.fn()} />);
  try {
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    const filter = screen.getByLabelText('كل البطاقات المستحقة');
    filter.focus();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(15000);
    });
    expect(screen.getByRole('alert')).toBeVisible();
    expect(filter).toHaveFocus();
    unmount();
  } finally {
    vi.useRealTimers();
  }
});
