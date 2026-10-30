import type { ProfileDetail } from '../shared/generated/ProfileDetail';
import type { TrainingEligibility } from '../shared/generated/TrainingEligibility';
import type { TrainingJob } from '../shared/generated/TrainingJob';
import type { DeckStatistics } from '../shared/generated/DeckStatistics';
import { deckFixture } from './mockApi';
// Presentation fixtures only. The UI never executes a model or trainer.
export const profileFixture: ProfileDetail = {
  id: 'profile-1',
  name: 'نموذج مشترك',
  config: {
    numEpochs: null,
    batchSize: 512,
    learningRate: null,
    seed: 2023,
    maxSeqLen: 256,
    gamma: 1,
    schedulePenalties: true,
  },
  parameters: Array(34).fill(1),
  activeModelRevision: 'model-1',
  modelOrigin: 'default',
  memberDeckIds: ['deck-1', 'deck-2'],
  revision: 'profile-revision',
  modelRevision: null,
};
export const eligibilityFixture: TrainingEligibility = {
  eligible: false,
  usableObservations: 399,
  distinctCards: 50,
  studyDays: 7,
  againCount: 5,
  recalledCount: 394,
  trainCount: 319,
  holdoutCount: 80,
  holdoutCards: 10,
  unmetConditions: ['observations'],
  contributions: [
    { deckId: 'deck-1', deckName: 'طب', observations: 200, cards: 25 },
    { deckId: 'deck-2', deckName: 'لغة', observations: 199, cards: 25 },
  ],
};
export const linkedDecks = [
  deckFixture,
  { ...deckFixture, id: 'deck-2', name: 'لغة', revision: 'deck-2-revision' },
];
export const readyJobFixture: TrainingJob = {
  id: 'job-1',
  profileId: 'profile-1',
  status: 'ready',
  progress: { current: 9, total: 9, fraction: 1 },
  eligibility: {
    ...eligibilityFixture,
    eligible: true,
    usableObservations: 400,
    unmetConditions: [],
  },
  config: profileFixture.config,
  sourceRevision: 'source-1',
  baselineModelRevision: 'model-1',
  memberDeckIds: ['deck-1', 'deck-2'],
  startedAt: '2026-10-25T12:00:00Z',
  finishedAt: '2026-10-25T12:01:00Z',
  error: null,
  reason: null,
  cancelRequested: false,
  baselineMetrics: { logLoss: 0.7, rmse: 0.4 },
  candidateMetrics: { logLoss: 0.6, rmse: 0.39 },
  perDeck: [
    {
      deckId: 'deck-1',
      observations: 40,
      baseline: { logLoss: 0.7, rmse: 0.4 },
      candidate: { logLoss: 0.6, rmse: 0.39 },
    },
  ],
  candidateParameters: Array(34).fill(0.9),
};
const empty = {
  completedForDay: 0,
  attempts: 0,
  uniqueCards: 0,
  activeMs: 0,
  meanResponseMs: null,
  ratingCounts: [0, 0, 0, 0] as [number, number, number, number],
  observedRecallRate: null,
};
export const statisticsFixture: DeckStatistics = {
  cards: 1,
  forms: 3,
  phases: { new: 0, learning: 1, relearning: 0, review: 0 },
  dueNow: 1,
  dueLaterToday: 0,
  nextDueAt: null,
  forecast7Days: [{ date: '2026-10-25', cards: 1 }],
  forecast30Days: [{ date: '2026-10-25', cards: 1 }],
  today: empty,
  last7Days: empty,
  last30Days: empty,
  allTime: empty,
  dailyHistory: [],
  profileStatus: {
    id: profileFixture.id,
    name: profileFixture.name,
    modelOrigin: 'default',
    modelRevision: 'model-1',
    eligibility: eligibilityFixture,
  },
};
