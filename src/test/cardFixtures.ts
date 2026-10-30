import type { CardDetail } from '../shared/generated/CardDetail';
import type { ImportPreview } from '../shared/generated/ImportPreview';
export const cardFixture: CardDetail = {
  id: 'card-1',
  deckId: 'deck-1',
  forms: [
    {
      id: 'form-1',
      front: '<h1>مرحبا</h1>',
      back: 'English',
      notes: 'ملاحظة',
      language: null,
      questionTitle: null,
    },
    { id: 'form-2', front: '日本語', back: 'عربي', notes: '', language: 'ja', questionTitle: null },
  ],
  phase: 'new',
  memory: null,
  dueAt: '2026-10-24T12:00:00Z',
  createdAt: '2026-10-24T12:00:00Z',
  reviewedAt: null,
  reviewCount: 0,
  lapseCount: 0,
  hasGraduated: false,
  revision: 'card-revision',
  bag: { remaining: [], consumed: [], lastConsumed: null, reserved: null, cycle: 0 },
};
const primary = {
  questionTitle: null,
  key: 'p1',
  front: 'سؤال عربي English',
  back: 'answer',
  notes: '',
  valid: true,
  duplicate: false,
  duplicateOf: null,
  issues: [],
  warnings: [],
};
export const importFixture: ImportPreview = {
  previewId: 'preview-1',
  deckId: 'deck-1',
  libraryRevision: 'library-revision',
  cards: [
    {
      key: 'card-0',
      valid: true,
      primary,
      issues: [],
      variants: [
        {
          ...primary,
          key: 'bad-variant',
          valid: false,
          front: '',
          issues: [{ field: '$[0].variants[0].front', code: 'required' }],
        },
        {
          ...primary,
          key: 'duplicate-variant',
          duplicate: true,
          duplicateOf: {
            scope: 'library',
            cardId: 'existing',
            deckId: 'other',
            formId: 'f',
            sourceKey: null,
          },
        },
      ],
    },
    {
      key: 'card-1',
      valid: false,
      primary: {
        ...primary,
        key: 'p2',
        valid: false,
        front: '',
        issues: [{ field: '$[1].front', code: 'required' }],
      },
      variants: [],
      issues: [],
    },
  ],
  counts: {
    validCards: 1,
    invalidCards: 1,
    duplicateCards: 0,
    selectedCards: 1,
    validForms: 1,
    invalidForms: 2,
    duplicateForms: 1,
    warnings: 0,
  },
  warnings: [],
};
