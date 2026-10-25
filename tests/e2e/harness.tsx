// Browser-only presentation adapter. Native tests establish real persistence/scheduling.
// This HTML entry is absent from the production Vite build.
import { createRoot } from 'react-dom/client';
import '@fontsource-variable/noto-sans-arabic';
import '../../src/styles/global.css';
import { App } from '../../src/app/App';
import { createApi } from '../../src/shared/api';
import { bootstrapFixture, deckFixture, nativeFixture } from '../../src/test/mockApi';
import { cardFixture, importFixture } from '../../src/test/cardFixtures';
import { profileFixture, eligibilityFixture, statisticsFixture } from '../../src/test/profileFixtures';
import { questionFixture, revealedFixture, completedFixture } from '../../src/test/reviewFixtures';
import type { Request } from '../../src/shared/generated/Request';
import type { Response } from '../../src/shared/generated/Response';
import type { ReviewView } from '../../src/shared/generated/ReviewView';
import type { CardDetail } from '../../src/shared/generated/CardDetail';
const decks = structuredClone([deckFixture]);
let cards: CardDetail[] = structuredClone([cardFixture]);
let settings = structuredClone(bootstrapFixture.settings);
let review: ReviewView = structuredClone(questionFixture);
const profiles = structuredClone([profileFixture]);
const calls: Request[] = [];
Object.assign(window, { presentationCalls: calls });
const api = createApi(async (r) => {
  calls.push(r);
  let payload: unknown;
  switch (r.kind) {
    case 'bootstrap': payload = { ...bootstrapFixture, decks, settings }; break;
    case 'listDecks': payload = decks; break;
    case 'getDeck': payload = decks.find(d => d.id === r.payload.deckId); break;
    case 'createDeck': { const deck = { ...deckFixture, id: crypto.randomUUID(), name: r.payload.name }; decks.push(deck); payload = deck; break; }
    case 'renameDeck': { const deck = decks.find(d => d.id === r.payload.deckId)!; deck.name = r.payload.name; payload = deck; break; }
    case 'deleteDeck': decks.splice(decks.findIndex(d => d.id === r.payload.deckId), 1); payload = { id: r.payload.deckId }; break;
    case 'listCards': payload = { items: cards, total: cards.length, page: 1, pageSize: 20 }; break;
    case 'getCard': payload = cards.find(c => c.id === r.payload.cardId); break;
    case 'createCard': case 'updateCard': {
      const card = { ...cardFixture, id: r.kind === 'updateCard' ? r.payload.cardId : crypto.randomUUID(), forms: r.payload.draft.forms.map(f => ({ ...f, id: f.id ?? crypto.randomUUID() })) };
      cards = [...cards.filter(c => c.id !== card.id), card]; payload = card; break;
    }
    case 'deleteCard': cards = cards.filter(c => c.id !== r.payload.cardId); payload = { id: r.payload.cardId }; break;
    case 'previewImport': if (r.payload.text === 'invalid') throw { code: 'invalidJson' }; payload = importFixture; break;
    case 'commitImport': payload = { insertedCards: 1, insertedForms: 1, excludedCards: 1, excludedForms: 2 }; break;
    case 'getDefaultSettings': payload = bootstrapFixture.settings; break;
    case 'saveSettings': settings = r.payload.settings; payload = settings; break;
    case 'getDeckStatistics': payload = statisticsFixture; break;
    case 'listProfiles': payload = profiles; break;
    case 'getProfile': payload = profiles[0]; break;
    case 'getTrainingEligibility': payload = eligibilityFixture; break;
    case 'listTrainingRuns': payload = []; break;
    case 'createProfile': { const profile = { ...profileFixture, id: crypto.randomUUID(), name: r.payload.name, memberDeckIds: [] }; profiles.push(profile); payload = profile; break; }
    case 'startSession': review = structuredClone(questionFixture); payload = review; break;
    case 'readSession': case 'resumeSession': payload = review; break;
    case 'revealAnswer': review = structuredClone(revealedFixture); payload = review; break;
    case 'rateCard': case 'endSession': review = structuredClone(completedFixture); payload = review; break;
    case 'undoReview': review = structuredClone(revealedFixture); payload = review; break;
    case 'setReviewFilter': review = r.payload.filter === 'previouslyReviewed' ? { state: 'emptyFilter', payload: { sessionId: 'session-1', filter: r.payload.filter, nextDueAt: null, progress: { current: 0, total: 0, completed: 0 }, canUndo: false, lastReview: null } } : { state: 'question', payload: { ...revealedFixture.payload, filter: r.payload.filter } }; payload = review; break;
    case 'setSessionActivity': payload = { activeMs: 0, questionMs: 0 }; break;
    case 'closeSession': payload = { id: r.payload.sessionId }; break;
    default: throw new Error(`Uncontracted presentation request: ${r.kind}`);
  }
  return { kind: r.kind, payload } as Response;
});
const report = { exportId: 'export', path: 'Downloads/OpenRecall.json', bytes: 1000, payloadSha256: 'digest' };
const native = {
  ...nativeFixture,
  async exportBackup() { return report; },
  async pickBackupForPreview() { return { id: 'preview', expectedLibraryRevision: 'rev', exportedAt: '2026-10-25T00:00:00Z', cards: 1, forms: 2, decks: 1, reviews: 0, profiles: 1, formatVersion: 1 }; },
  async restoreBackup() { return { safetyBackup: report, cards: 1, decks: 1 }; },
};
createRoot(document.getElementById('root')!).render(<App api={api} native={native} />);
