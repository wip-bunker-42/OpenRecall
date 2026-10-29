import type { ReviewView } from '../shared/generated/ReviewView';
import type { ReviewFrame } from '../shared/generated/ReviewFrame';
import type { SessionSummary } from '../shared/generated/SessionSummary';
import { cardFixture } from './cardFixtures';
const memory = { stability: 1, stabilityFast: 0, difficulty: 5 };
export const frameFixture: ReviewFrame = {
  mode: 'standard',
  canRate: false,
  lastReview: null,
  sessionId: 'session-1',
  filter: 'all',
  presentationId: 'presentation-1',
  card: cardFixture,
  form: cardFixture.forms[0],
  answerRevealed: false,
  previews: null,
  progress: { current: 1, total: 2, completed: 0 },
  nextDueAt: '2027-10-24T12:00:00Z',
  cardRevision: 'card-revision',
  canUndo: false,
  previewDueAt: null,
};
export const questionFixture: Extract<ReviewView, { state: 'question' }> = {
  state: 'question',
  payload: frameFixture,
};
export const revealedFixture: Extract<ReviewView, { state: 'question' }> = {
  state: 'question',
  payload: {
    ...frameFixture,
    canRate: true,
    answerRevealed: true,
    previews: {
      again: { memory, intervalSeconds: 60 },
      hard: { memory, intervalSeconds: 3600 },
      good: { memory, intervalSeconds: 86400 },
      easy: { memory, intervalSeconds: 2592000 },
    },
    previewDueAt: {
      again: '2026-10-24T12:01:00Z',
      hard: '2026-10-24T13:00:00Z',
      good: '2026-10-25T12:00:00Z',
      easy: '2026-11-23T12:00:00Z',
    },
  },
};
export const summaryFixture: SessionSummary = {
  lastReview: {
    cardId: 'card-1',
    rating: 'easy',
    intervalSeconds: 2592000,
    dueAt: '2026-11-23T12:00:00Z',
  },
  sessionId: 'session-1',
  filter: 'all',
  filterChanges: [],
  startedAt: '2026-10-24T12:00:00Z',
  endedAt: '2026-10-24T12:00:10Z',
  wallMs: 10000,
  activeMs: 8000,
  attempts: 1,
  uniqueCards: 1,
  newCards: 1,
  ratings: [0, 0, 0, 1],
  meanResponseMs: 8000,
  matchingDueRemaining: 0,
  allDueRemaining: 0,
  nextDueAt: '2026-11-23T12:00:00Z',
  canUndo: true,
};
export const completedFixture: Extract<ReviewView, { state: 'summary' }> = {
  state: 'summary',
  payload: summaryFixture,
};
