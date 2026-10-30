import { browser, $, $$, expect } from '@wdio/globals';
import axe from 'axe-core';
import type { AxeResults } from 'axe-core';
import { writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import type { Kind, Input, Output } from '../../src/shared/api';
import type { Request } from '../../src/shared/generated/Request';
import type { Response } from '../../src/shared/generated/Response';
export { browser, $, $$, expect };
export const root = process.env.OPENRECALL_NATIVE_ROOT!;
/** Typed real IPC used only against the runner's isolated synthetic library. */
export async function command<K extends Kind>(kind: K, payload: Input<K>): Promise<Output<K>> {
  const result = (await browser.executeAsync<unknown, [Request]>(
    (request, done) => {
      const native = window as unknown as {
        __TAURI_INTERNALS__: { invoke: (name: string, input: unknown) => Promise<Response> };
      };
      native.__TAURI_INTERNALS__
        .invoke('execute', { request })
        .then(done)
        .catch((error) => done({ error }));
    },
    { kind, payload } as Request,
  )) as Response | { error: unknown };
  if ('error' in result)
    throw new Error(`Synthetic native ${kind}: ${JSON.stringify(result.error)}`);
  expect(result.kind).toBe(kind);
  return result.payload as Output<K>;
}
export async function click(text: string) {
  const e = await $(`button=${text}`);
  await e.waitForEnabled();
  await e.click();
}
export async function heading(text: string) {
  await expect($(`h1=${text}`)).toBeDisplayed();
}
export async function containing(selector: string, text: string) {
  for (const element of await $$(selector))
    if ((await element.getText()).includes(text)) return element;
  throw new Error(`No ${selector} contains ${text}`);
}
export async function audit(name: string) {
  await browser.execute(axe.source);
  const result = (await browser.executeAsync((done) => {
    (window as unknown as { axe: { run: () => Promise<AxeResults> } }).axe.run().then(done);
  })) as AxeResults;
  expect(result.passes.length).toBeGreaterThan(0);
  expect(
    result.violations.map((v) => ({ id: v.id, targets: v.nodes.map((n) => n.target) })),
  ).toEqual([]);
  const folder = resolve(root, 'audits');
  mkdirSync(folder, { recursive: true });
  writeFileSync(
    resolve(folder, `${name}.json`),
    JSON.stringify({
      passes: result.passes.length,
      violations: result.violations,
      incomplete: result.incomplete.map((v) => v.id),
    }),
  );
}
