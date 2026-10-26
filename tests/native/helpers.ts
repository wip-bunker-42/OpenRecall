import { browser, $, $$, expect } from '@wdio/globals';
import axe from 'axe-core';
import type { AxeResults } from 'axe-core';
import { writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
export { browser, $, $$, expect };
export const root = process.env.OPENRECALL_NATIVE_ROOT!;
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
