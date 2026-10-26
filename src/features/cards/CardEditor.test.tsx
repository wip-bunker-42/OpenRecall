import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { CardEditor } from './CardEditor';
import { DeckPage } from './DeckPage';
import { fixtureApi, nativeFixture, deckFixture } from '../../test/mockApi';
import { cardFixture } from '../../test/cardFixtures';
it('keeps stable form IDs, focuses added/removed alternatives and submits all forms', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'updateCard') return { kind: r.kind, payload: cardFixture };
  });
  const saved = vi.fn();
  const user = userEvent.setup();
  render(
    <CardEditor
      card={cardFixture}
      deckId={deckFixture.id}
      api={api}
      onSaved={saved}
      onCancel={vi.fn()}
    />,
  );
  expect(screen.getByLabelText('السؤال، الصيغة ١')).toHaveFocus();
  await user.click(screen.getByRole('button', { name: 'إضافة صيغة بديلة' }));
  expect(screen.getByLabelText('السؤال، الصيغة ٣')).toHaveFocus();
  await user.type(screen.getByLabelText('السؤال، الصيغة ٣'), '日本語');
  await user.type(screen.getByLabelText('الإجابة، الصيغة ٣'), 'English');
  await user.click(screen.getByRole('button', { name: 'إضافة صيغة بديلة' }));
  await user.click(screen.getByRole('button', { name: 'حذف الصيغة ٤' }));
  expect(screen.getByLabelText('السؤال، الصيغة ٣')).toHaveFocus();
  await user.click(screen.getByRole('button', { name: 'حفظ البطاقة' }));
  expect(saved).toHaveBeenCalled();
  expect(calls).toContainEqual(
    expect.objectContaining({
      kind: 'updateCard',
      payload: expect.objectContaining({
        draft: {
          forms: [
            expect.objectContaining({ id: 'form-1' }),
            expect.objectContaining({ id: 'form-2' }),
            expect.objectContaining({ id: null, front: '日本語', back: 'English' }),
          ],
        },
      }),
    }),
  );
});
it('announces required fields without submitting and confirms discarding edits', async () => {
  const calls: unknown[] = [];
  const cancel = vi.fn();
  const user = userEvent.setup();
  render(
    <CardEditor
      card={null}
      deckId="deck-1"
      api={fixtureApi((r) => {
        calls.push(r);
        return undefined;
      })}
      onSaved={vi.fn()}
      onCancel={cancel}
    />,
  );
  await user.click(screen.getByRole('button', { name: 'حفظ البطاقة' }));
  expect(screen.getByRole('alert')).toHaveTextContent('السؤال والإجابة');
  expect(screen.getByRole('alert')).toHaveFocus();
  expect(calls).toHaveLength(0);
  await user.type(screen.getByLabelText('السؤال، الصيغة ١'), 'new');
  await user.click(screen.getByRole('button', { name: 'إلغاء' }));
  expect(cancel).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'تجاهل التغييرات' }));
  expect(cancel).toHaveBeenCalled();
});
it('loads paginated literal multilingual cards only when the closed card section opens', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'getDeck') return { kind: r.kind, payload: deckFixture };
    if (r.kind === 'listCards')
      return {
        kind: r.kind,
        payload: { items: [cardFixture], total: 21, page: r.payload.page, pageSize: 20 },
      };
  });
  const user = userEvent.setup();
  render(<DeckPage deckId="deck-1" api={api} native={nativeFixture} navigate={vi.fn()} />);
  await screen.findByRole('heading', { name: 'طب', level: 1 });
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'listCards' }));
  await user.click(screen.getByText('البطاقات'));
  const content = await screen.findByText('<h1>مرحبا</h1>');
  expect(content).toHaveAttribute('dir', 'auto');
  expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
  await user.type(screen.getByLabelText('البحث في البطاقات'), '日本');
  await user.click(screen.getByRole('button', { name: 'بحث' }));
  await user.click(screen.getByRole('button', { name: 'الصفحة التالية' }));
  expect(calls).toContainEqual({
    kind: 'listCards',
    payload: { deckId: 'deck-1', query: '日本', page: 2, pageSize: 20 },
  });
  expect(
    within(content.closest('li')!).getByRole('button', { name: 'تعديل البطاقة ٢١' }),
  ).toBeVisible();
});
