import type { SpellingConfig } from '../shared/generated/SpellingConfig';
import type { SpellingDeckResult } from '../shared/generated/SpellingDeckResult';
import { deckFixture } from './mockApi';
import { cardFixture } from './cardFixtures';
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
