import { mkdirSync, copyFileSync, readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { pnpmSync } from './tool-process.mjs';
if (process.platform !== 'win32')
  throw new Error('The current native runtime suite targets Windows.');
const result = pnpmSync(
  [
    'tauri',
    'build',
    '--debug',
    '--no-bundle',
    '--features',
    'automation',
    '--config',
    'tests/native/tauri.automation.conf.json',
  ],
  { stdio: 'inherit' },
);
if (result.status !== 0) process.exit(result.status ?? 1);
// Cargo workspace tests can rebuild target/debug's ordinary binary. Never run it
// as the automation app; preserve the feature-specific executable separately.
const folder = resolve('target/native-automation');
mkdirSync(folder, { recursive: true });
assert.ok(
  readFileSync(resolve('target/debug/openrecall-desktop.exe')).includes(
    Buffer.from('OPENRECALL_TEST_DATA'),
  ),
  'Refusing to preserve a binary without the automation feature',
);
copyFileSync(
  resolve('target/debug/openrecall-desktop.exe'),
  resolve(folder, 'OpenRecallAutomation.exe'),
);
