import { createApi, type NativeApi } from '../shared/api';
import type { Request } from '../shared/generated/Request';
import type { Response } from '../shared/generated/Response';
import type { Bootstrap } from '../shared/generated/Bootstrap';
import type { DeckDetail } from '../shared/generated/DeckDetail';
export const deckFixture: DeckDetail = {
  kind: 'standard',
  id: 'deck-1',
  name: 'طب',
  profileId: 'profile-1',
  createdAt: '2026-10-24T12:00:00Z',
  revision: 'deck-revision',
  dataRevision: 'deck-data',
  logicalCards: 3,
  dueNow: 2,
  newCards: 1,
  desiredRetention: null,
  minIntervalSeconds: null,
  maxIntervalDays: null,
};
export const bootstrapFixture: Bootstrap = {
  settings: {
    timezone: 'Africa/Cairo',
    dayStartMinute: 60,
    desiredRetention: 0.9,
    minIntervalSeconds: 60,
    maxIntervalDays: 36500,
    shortcutsEnabled: true,
    textScale: 1,
    reducedMotion: 'followSystem',
    pageSize: 20,
    revision: 'settings-revision',
  },
  schemaVersion: 9,
  resumableSessions: [],
  decks: [deckFixture],
  profiles: [],
  libraryRevision: 'library-revision',
};
export function fixtureApi(
  handler?: (r: Request) => Response | undefined | Promise<Response | undefined>,
) {
  return createApi(async (r) => {
    const response = await handler?.(r);
    if (response) return response;
    if (r.kind === 'bootstrap') return { kind: r.kind, payload: bootstrapFixture };
    if (r.kind === 'getDefaultSettings')
      return { kind: r.kind, payload: bootstrapFixture.settings };
    if (r.kind === 'listDecks') return { kind: r.kind, payload: [deckFixture] };
    throw new Error(`Missing fixture ${r.kind}`);
  });
}
export const nativeFixture: NativeApi = {
  async getUpdateStatus() {
    return {
      phase: 'idle',
      currentVersion: '0.1.0',
      availableVersion: null,
      notes: null,
      downloadedBytes: 0,
      totalBytes: null,
      errorCode: null,
    };
  },
  async onUpdateStatus() {
    return () => {};
  },
  async checkForUpdate() {
    return { ...(await this.getUpdateStatus()), phase: 'upToDate' };
  },
  async downloadUpdate() {
    throw { code: 'updateNotReady' };
  },
  async installUpdate() {
    throw { code: 'updateNotReady' };
  },
  async setEditState() {},
  async exportBackup() {
    throw { code: 'fileAccess' };
  },
  async pickBackupForPreview() {
    return null;
  },
  async restoreBackup() {
    throw { code: 'fileAccess' };
  },
  async revealExport() {},
  async pickImportFile() {
    return null;
  },
  async getSystemInfo() {
    return {
      appVersion: '0.1.0',
      engineVersion: 'FSRS-7',
      engineRevision: '0fd5994',
      schemaVersion: 9,
      dataDirectory: 'test-data',
    };
  },
};
