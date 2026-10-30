import type { SpellingConfig } from '../shared/generated/SpellingConfig';
import type { SpellingDeckResult } from '../shared/generated/SpellingDeckResult';
import { deckFixture } from './mockApi';
import { cardFixture } from './cardFixtures';
import { revealedFixture } from './reviewFixtures';
import type { ReviewView } from '../shared/generated/ReviewView';
import type { SpellingSourceView } from '../shared/generated/SpellingSourceView';
export const spellingDeck = {
  ...deckFixture,
  id: 'spelling-1',
  kind: 'spelling' as const,
  name: 'مجموعة التدرب على الكتابة',
};
export const spellingConfig: SpellingConfig = {
  deckId: spellingDeck.id,
  sourceDeckId: deckFixture.id,
  sourceDeckName: deckFixture.name,
  extractionPolicyVersion: 1,
  revision: 'spelling-config-1',
};
export const spellingResult: SpellingDeckResult = {
  deck: spellingDeck,
  config: spellingConfig,
  addedWords: 2,
};
export const spellingCard = {
  ...cardFixture,
  deckId: spellingDeck.id,
  forms: [
    { ...cardFixture.forms[0], front: 'English', back: 'English', notes: '', language: 'en' },
  ],
};
export const spellingQuestion: Extract<ReviewView, { state: 'question' }> = {
  state: 'question',
  payload: {
    ...revealedFixture.payload,
    mode: 'spelling',
    canRate: true,
    answerRevealed: false,
    card: spellingCard,
    form: spellingCard.forms[0],
  },
};
export const sourceContext: SpellingSourceView = {
  availability: 'deleted',
  snapshot: {
    id: 'snapshot-1',
    spellingDeckId: spellingDeck.id,
    sourceDeckId: deckFixture.id,
    sourceDeckName: deckFixture.name,
    sourceCardId: 'source-card',
    sourceFormId: 'source-form',
    sourceFormIndex: 0,
    sourceCardRevision: 'source-revision',
    extractionPolicyVersion: 1,
    capturedAt: '2026-10-24T12:00:00Z',
    front: '<b>English source</b>',
    back: 'معنى محفوظ',
    notes: 'سياق قديم',
    questionTitle: 'ترجم',
    language: 'en',
    contentSha256: 'synthetic',
  },
};
