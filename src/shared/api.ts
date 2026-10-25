import { invoke } from '@tauri-apps/api/core';
import type { Request } from './generated/Request';
import type { Response } from './generated/Response';
import type { SystemInfo } from './generated/SystemInfo';
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
  getSystemInfo(): Promise<SystemInfo>;
}
export function createNativeApi(): NativeApi {
  return { getSystemInfo: () => invoke('get_system_info') };
}
