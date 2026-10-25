import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../../app/App';
import { fixtureApi, nativeFixture, deckFixture } from '../../test/mockApi';
import { Disclosure } from '../../components/Disclosure';
it('keeps optional details collapsed with a focusable native summary', async () => {
  const user = userEvent.setup();
  render(
    <Disclosure title="الإحصاءات">
      <p>بيانات المراجعة</p>
    </Disclosure>,
  );
  expect(screen.getByText('بيانات المراجعة')).not.toBeVisible();
  await user.tab();
  expect(
    screen.getByRole('heading', { level: 2, name: 'الإحصاءات' }).closest('summary'),
  ).toHaveFocus();
  await user.click(screen.getByText('الإحصاءات'));
  expect(screen.getByText('بيانات المراجعة')).toBeVisible();
});
it('shows Arabic library with one heading and visible create, and returns focus after cancel', async () => {
  const user = userEvent.setup();
  render(<App api={fixtureApi()} native={nativeFixture} />);
  const button = await screen.findByRole('button', { name: 'إنشاء مجموعة' });
  expect(document.documentElement).toHaveAttribute('lang', 'ar');
  expect(document.documentElement).toHaveAttribute('dir', 'rtl');
  expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
  await user.click(button);
  const dialog = screen.getByRole('dialog');
  expect(within(dialog).getByLabelText('اسم المجموعة')).toHaveFocus();
  await user.keyboard('{Escape}');
  await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
  expect(button).toHaveFocus();
});
it('creates, renames and deletes through explicit dialogs and displays API errors', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'createDeck') return { kind: r.kind, payload: deckFixture };
    if (r.kind === 'renameDeck') return { kind: r.kind, payload: { ...deckFixture, name: 'جديد' } };
    if (r.kind === 'deleteDeck') throw { code: 'storage', fields: [], context: null };
  });
  const user = userEvent.setup();
  render(<App api={api} native={nativeFixture} />);
  await user.click(await screen.findByRole('button', { name: 'إنشاء مجموعة' }));
  await user.type(screen.getByLabelText('اسم المجموعة'), 'طب');
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'إنشاء' }));
  await screen.findByRole('button', { name: 'إعادة تسمية طب' });
  await user.click(screen.getByRole('button', { name: 'إعادة تسمية طب' }));
  await user.clear(screen.getByLabelText('اسم المجموعة'));
  await user.type(screen.getByLabelText('اسم المجموعة'), 'جديد');
  await user.click(screen.getByRole('button', { name: 'حفظ الاسم' }));
  await screen.findByRole('button', { name: 'حذف طب' });
  await user.click(screen.getByRole('button', { name: 'حذف طب' }));
  await user.click(
    within(screen.getByRole('dialog')).getByRole('button', { name: 'حذف المجموعة نهائيًا' }),
  );
  expect(await screen.findByRole('alert')).toHaveTextContent('تعذر حفظ البيانات');
  expect(calls).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ kind: 'createDeck' }),
      expect.objectContaining({ kind: 'renameDeck' }),
      expect.objectContaining({ kind: 'deleteDeck' }),
    ]),
  );
});
