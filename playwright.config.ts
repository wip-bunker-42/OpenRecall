import { defineConfig, devices } from '@playwright/test';
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,
  workers: 2,
  retries: 0,
  timeout: 30_000,
  outputDir: 'target/playwright-results',
  reporter: [['list'], ['html', { outputFolder: 'target/playwright-report', open: 'never' }]],
  use: { baseURL: 'http://127.0.0.1:1421', trace: 'retain-on-failure' },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: 'node scripts/serve-e2e.mjs',
    url: 'http://127.0.0.1:1421/tests/e2e/harness.html',
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
