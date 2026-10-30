import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ReviewPage } from '../review/ReviewPage';
import { fixtureApi } from '../../test/mockApi';
import { spellingQuestion, sourceContext } from '../../test/spellingFixtures';
import type { Response } from '../../shared/generated/Response';
it('loads bounded retained contexts on request and paginates plain saved versions', async () => {
  const user = userEvent.setup();
  const pages: number[] = [];
  const api = fixtureApi((r) => {
    if (r.kind === 'readSession') return { kind: r.kind, payload: spellingQuestion };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
    if (r.kind === 'listSpellingSources') {
      pages.push(r.payload.page);
      return {
        kind: r.kind,
        payload: {
          items: [
            r.payload.page === 1
              ? sourceContext
              : {
                  ...sourceContext,
                  availability: 'changed',
                  snapshot: { ...sourceContext.snapshot, id: 'snapshot-2', back: 'السياق الثاني' },
                },
          ],
          total: 21,
          page: r.payload.page,
          pageSize: 20,
        },
      };
    }
  });
  render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
  await user.click(await screen.findByText('عرض البطاقة الأصلية'));
  expect(await screen.findByText('<b>English source</b>')).toBeVisible();
  expect(screen.getByText('<b>English source</b>').querySelector('b')).toBeNull();
  expect(screen.getByText(/حُذفت البطاقة الأصلية/)).toBeVisible();
  expect(screen.getByText(/نسخة محفوظة/)).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'السياقات التالية' }));
  expect(await screen.findByText('السياق الثاني')).toBeVisible();
  expect(screen.getByText(/تغيرت البطاقة الأصلية/)).toBeVisible();
  expect(pages).toEqual([1, 2]);
});

it('never shows a previous word source when its deferred response resolves after advancement', async () => {
  const user = userEvent.setup();
  let resolve!: (r: Response) => void;
  const api = fixtureApi((r) => {
    if (r.kind === 'readSession') return { kind: r.kind, payload: spellingQuestion };
    if (r.kind === 'setSessionActivity')
      return { kind: r.kind, payload: { activeMs: 0, questionMs: 0 } };
    if (r.kind === 'listSpellingSources')
      return new Promise<Response>((done) => {
        resolve = done;
      });
    if (r.kind === 'rateCard')
      return {
        kind: r.kind,
        payload: {
          ...spellingQuestion,
          payload: {
            ...spellingQuestion.payload,
            presentationId: 'next-word',
            card: { ...spellingQuestion.payload.card, id: 'word-2' },
            form: { ...spellingQuestion.payload.form, front: 'Next' },
          },
        },
      };
  });
  render(<ReviewPage sessionId="session-1" api={api} navigate={() => {}} />);
  await user.click(await screen.findByText('عرض البطاقة الأصلية'));
  await screen.findByText('جارٍ تحميل السياقات…');
  await user.click(screen.getByRole('button', { name: /افتكرتها بسهولة/ }));
  await screen.findByText('Next');
  await act(async () =>
    resolve({
      kind: 'listSpellingSources',
      payload: { items: [sourceContext], total: 1, page: 1, pageSize: 20 },
    }),
  );
  expect(screen.queryByText('معنى محفوظ')).not.toBeInTheDocument();
  expect(screen.getByText('عرض البطاقة الأصلية').closest('details')).not.toHaveAttribute('open');
});
