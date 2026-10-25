import { invoke } from '@tauri-apps/api/core';
import type { Request } from './generated/Request';
import type { Response } from './generated/Response';
import type { SystemInfo } from './generated/SystemInfo';
import type { PickedImportFile } from './generated/PickedImportFile';
import type { ExportReport } from './generated/ExportReport';
import type { RestorePreview } from './generated/RestorePreview';
import type { RestoreReport } from './generated/RestoreReport';
export type Kind = Request['kind'];
export type Input<K extends Kind> = Extract<Request, { kind: K }>['payload'];
export type Output<K extends Kind> = Extract<Response, { kind: K }>['payload'];
export interface Api {
  call<K extends Kind>(kind: K, payload: Input<K>): Promise<Output<K>>;
}
export function createApi(transport: (request: Request) => Promise<unknown>): Api {
  return {
    async call<K extends Kind>(kind: K, payload: Input<K>): Promise<Output<K>> {
      const response = await transport({ kind, payload } as Request);
      if (
        typeof response !== 'object' ||
        response === null ||
        !('kind' in response) ||
        response.kind !== kind ||
        !('payload' in response)
      )
        throw { code: 'responseMismatch', fields: [], context: null };
      return response.payload as Output<K>;
    },
  };
}
export function createTauriApi(): Api {
  return createApi((request) => invoke('execute', { request }));
}
export interface NativeApi {
  setEditState(input: { unsavedChanges: boolean }): Promise<void>;
  exportBackup(): Promise<ExportReport>;
  pickBackupForPreview(): Promise<RestorePreview | null>;
  restoreBackup(input: {
    previewId: string;
    expectedLibraryRevision: string;
  }): Promise<RestoreReport>;
  revealExport(input: { exportId: string }): Promise<void>;
  pickImportFile(): Promise<PickedImportFile | null>;
  getSystemInfo(): Promise<SystemInfo>;
}
export function createNativeApi(): NativeApi {
  return {
    setEditState: (input) => invoke('set_edit_state', { input }),
    exportBackup: () => invoke('export_backup'),
    pickBackupForPreview: () => invoke('pick_backup_for_preview'),
    restoreBackup: (input) => invoke('restore_backup', { input }),
    revealExport: (input) => invoke('reveal_export', { input }),
    getSystemInfo: () => invoke('get_system_info'),
    pickImportFile: () => invoke('pick_import_file'),
  };
}
