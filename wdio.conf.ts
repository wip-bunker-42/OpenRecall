import { resolve } from 'node:path';
import { mkdirSync } from 'node:fs';
const root = process.env.OPENRECALL_NATIVE_ROOT;
if (!root) throw new Error('Run pnpm test:native to create an isolated native library.');
mkdirSync(resolve(root, 'downloads'), { recursive: true });
const application = resolve('target/native-automation/OpenRecallAutomation.exe');
export const config = {
  runner: 'local',
  specs: [
    './tests/native/persistence.spec.ts',
    './tests/native/review.spec.ts',
    './tests/native/offline.spec.ts',
    './tests/native/focus.spec.ts',
  ],
  maxInstances: 1,
  logLevel: 'warn',
  outputDir: resolve(root, 'logs'),
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { timeout: 90_000 },
  waitforTimeout: 15_000,
  connectionRetryTimeout: 30_000,
  connectionRetryCount: 0,
  services: [
    [
      '@wdio/tauri-service',
      {
        appBinaryPath: application,
        driverProvider: 'embedded',
        embeddedPort: 4457,
        autoDownloadEdgeDriver: false,
        captureBackendLogs: false,
        captureFrontendLogs: false,
        startTimeout: 60_000,
        statusPollTimeout: 5_000,
        env: {
          OPENRECALL_TEST_DATA: resolve(root, 'data'),
          OPENRECALL_TEST_DOWNLOADS: resolve(root, 'downloads'),
          OPENRECALL_TEST_IMPORT_FILE: resolve(root, 'cards.json'),
          OPENRECALL_TEST_BACKUP_FILE: resolve(root, 'restore.json'),
          HTTP_PROXY: 'http://127.0.0.1:4459',
          HTTPS_PROXY: 'http://127.0.0.1:4459',
          ALL_PROXY: 'http://127.0.0.1:4459',
          NO_PROXY: 'localhost,127.0.0.1,::1',
          WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:
            '--proxy-server=http://127.0.0.1:4459 --proxy-bypass-list=localhost;127.0.0.1;[::1];<local>',
        },
      },
    ],
  ],
  capabilities: [{ browserName: 'tauri', 'tauri:options': { application } }],
};
