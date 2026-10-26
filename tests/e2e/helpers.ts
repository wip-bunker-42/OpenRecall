import { test as base, expect, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { writeFileSync } from 'node:fs';
export { expect };
export const test = base.extend<{ app: Page }>({
  app: async ({ page }, use) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await page.goto('/tests/e2e/harness.html');
    await expect(page.getByRole('heading', { name: 'مجموعاتي' })).toBeVisible();
    await use(page);
    expect(errors).toEqual([]);
  },
});
export async function audit(page: Page, name: string) {
  const result = await new AxeBuilder({ page }).analyze();
  expect(result.passes.length, `${name}: axe must actually run rules`).toBeGreaterThan(0);
  expect(
    result.violations,
    `${name}: ${JSON.stringify(result.violations.map((v) => ({ id: v.id, nodes: v.nodes.map((n) => n.target) })))}`,
  ).toEqual([]);
  const path = test.info().outputPath(`axe-${name}.json`);
  writeFileSync(
    path,
    JSON.stringify({
      passes: result.passes.length,
      violations: result.violations,
      incomplete: result.incomplete.map((v) => v.id),
    }),
  );
  await test.info().attach(`axe-${name}`, { path, contentType: 'application/json' });
}
export async function deck(page: Page) {
  await page.getByRole('button', { name: 'طب', exact: true }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'طب' })).toBeVisible();
}
