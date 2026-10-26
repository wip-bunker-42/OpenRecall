import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ImportDialog } from './ImportDialog';
import { fixtureApi, nativeFixture } from '../../test/mockApi';
import { importFixture } from '../../test/cardFixtures';
import { ImportPreview } from './ImportPreview';
it('resets selection to the fresh preview after a conflict', async () => {
  let inspections = 0;
  const updated = {
    ...importFixture,
    previewId: 'preview-2',
    cards: [{ ...importFixture.cards[0], valid: false, variants: [] }],
  };
  const api = fixtureApi((r) => {
    if (r.kind === 'previewImport')
      return {
        kind: r.kind,
        payload:
          inspections++ === 0
            ? { ...importFixture, cards: [{ ...importFixture.cards[0], variants: [] }] }
            : updated,
      };
    if (r.kind === 'commitImport') throw { code: 'conflict' };
  });
  const user = userEvent.setup();
  render(
    <ImportDialog
      deckId="deck-1"
      api={api}
      native={nativeFixture}
      onComplete={vi.fn()}
      onCancel={vi.fn()}
    />,
  );
  await user.click(screen.getByLabelText('محتوى JSON'));
  await user.paste('[]');
  await user.click(screen.getByRole('button', { name: 'فحص JSON' }));
  await user.click(screen.getByRole('button', { name: 'استيراد البطاقات المحددة' }));
  await user.click(screen.getByRole('button', { name: 'تحديث المعاينة وإعادة تحديد البطاقات' }));
  expect(screen.getByLabelText('استيراد البطاقة ١')).not.toBeChecked();
  expect(screen.getByRole('button', { name: 'استيراد البطاقات المحددة' })).toBeDisabled();
});
it('preserves manual valid-form exclusions when excluding invalid forms in bulk, and pages rows', async () => {
  const first = importFixture.cards[0];
  const valid = { ...first.primary, key: 'valid-alternative', front: 'a different question' };
  const preview = {
    ...importFixture,
    cards: [
      { ...first, variants: [...first.variants, valid] },
      ...Array.from({ length: 20 }, (_, i) => ({
        key: `row-${i}`,
        valid: true,
        primary: { ...first.primary, key: `primary-${i}`, front: `question ${i}` },
        variants: [],
        issues: [],
      })),
    ],
  };
  const commit = vi.fn();
  const user = userEvent.setup();
  render(
    <ImportPreview
      preview={preview}
      onCommit={commit}
      onBack={vi.fn()}
      onCancel={vi.fn()}
      busy={false}
    />,
  );
  expect(screen.queryByText('question 19')).not.toBeInTheDocument();
  await user.click(screen.getByLabelText('استبعاد الصيغة البديلة ٣ للبطاقة ١'));
  await user.click(screen.getByRole('button', { name: 'استبعاد كل الصيغ غير الصالحة والمكررة' }));
  expect(screen.getByLabelText('استبعاد الصيغة البديلة ٣ للبطاقة ١')).toBeChecked();
  await user.click(screen.getByRole('button', { name: 'التالي في المعاينة' }));
  expect(screen.getByText('question 19')).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'استيراد البطاقات المحددة' }));
  expect(commit).toHaveBeenCalledWith(
    expect.objectContaining({
      excludedFormKeys: expect.arrayContaining([
        'valid-alternative',
        'bad-variant',
        'duplicate-variant',
      ]),
    }),
  );
});
it('previews without writes and requires explicit exclusion of invalid and duplicate alternatives', async () => {
  const calls: unknown[] = [];
  const api = fixtureApi((r) => {
    calls.push(r);
    if (r.kind === 'previewImport') return { kind: r.kind, payload: importFixture };
    if (r.kind === 'commitImport')
      return {
        kind: r.kind,
        payload: { insertedCards: 1, insertedForms: 1, excludedCards: 1, excludedForms: 2 },
      };
  });
  const complete = vi.fn();
  const user = userEvent.setup();
  render(
    <ImportDialog
      deckId="deck-1"
      api={api}
      native={nativeFixture}
      onComplete={complete}
      onCancel={vi.fn()}
    />,
  );
  await user.click(screen.getByLabelText('محتوى JSON'));
  await user.paste('[]');
  await user.click(screen.getByRole('button', { name: 'فحص JSON' }));
  expect(screen.getByLabelText('استيراد البطاقة ١')).toBeChecked();
  expect(screen.getByLabelText('استيراد البطاقة ٢')).toBeDisabled();
  for (const text of screen.getAllByText('سؤال عربي English'))
    expect(text).toHaveAttribute('dir', 'auto');
  expect(screen.getByRole('button', { name: 'استيراد البطاقات المحددة' })).toBeDisabled();
  expect(calls).not.toContainEqual(expect.objectContaining({ kind: 'commitImport' }));
  await user.click(screen.getByLabelText('استبعاد الصيغة البديلة ١ للبطاقة ١'));
  await user.click(screen.getByLabelText('استبعاد الصيغة البديلة ٢ للبطاقة ١'));
  await user.click(screen.getByRole('button', { name: 'استيراد البطاقات المحددة' }));
  expect(calls).toContainEqual({
    kind: 'commitImport',
    payload: {
      previewId: 'preview-1',
      libraryRevision: 'library-revision',
      selectedCardKeys: ['card-0'],
      excludedFormKeys: ['bad-variant', 'duplicate-variant'],
    },
  });
  expect(complete).toHaveBeenCalledWith({
    insertedCards: 1,
    insertedForms: 1,
    excludedCards: 1,
    excludedForms: 2,
  });
});
it('allows cancellation and cancelled file picking without committing', async () => {
  const calls: unknown[] = [];
  const cancel = vi.fn();
  const native = { ...nativeFixture, pickImportFile: vi.fn().mockResolvedValue(null) };
  const user = userEvent.setup();
  render(
    <ImportDialog
      deckId="deck-1"
      api={fixtureApi((r) => {
        calls.push(r);
        return undefined;
      })}
      native={native}
      onComplete={vi.fn()}
      onCancel={cancel}
    />,
  );
  await user.click(screen.getByRole('button', { name: 'اختيار ملف JSON' }));
  expect(screen.getByLabelText('محتوى JSON')).toHaveValue('');
  await user.click(screen.getByRole('button', { name: 'إلغاء' }));
  expect(cancel).toHaveBeenCalled();
  expect(calls).toHaveLength(0);
});
