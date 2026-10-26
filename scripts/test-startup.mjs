import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, copyFileSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { DatabaseSync } from 'node:sqlite';
const parent = resolve('target/startup-tests');
mkdirSync(parent, { recursive: true });
const source = process.argv[2];
assert.ok(source, 'Usage: test-startup.mjs EXISTING_SYNTHETIC_DATABASE');
for (const failure of ['future-schema', 'storage']) {
  const folder = mkdtempSync(join(parent, 'run-'));
  const data = join(folder, 'data');
  const database = failure === 'future-schema' ? join(data, 'openrecall.sqlite3') : data;
  if (failure === 'future-schema') {
    mkdirSync(data);
    copyFileSync(source, database);
    const connection = new DatabaseSync(database);
    connection.exec('PRAGMA user_version=999');
    connection.close();
  } else {
    writeFileSync(data, 'This file blocks creation of the synthetic data directory.');
  }
  const digest = () => createHash('sha256').update(readFileSync(database)).digest('hex');
  const before = digest();
  const child = spawn(resolve('target/native-automation/OpenRecallAutomation.exe'), [], {
    windowsHide: true,
    stdio: 'ignore',
    env: {
      ...process.env,
      OPENRECALL_TEST_DATA: data,
      OPENRECALL_TEST_DOWNLOADS: folder,
      HTTP_PROXY: 'http://127.0.0.1:9',
      HTTPS_PROXY: 'http://127.0.0.1:9',
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:
        '--proxy-server=http://127.0.0.1:9 --proxy-bypass-list=localhost;127.0.0.1;[::1];<local>',
    },
  });
  let exitCode = null;
  const exited = new Promise((resolve) =>
    child.once('exit', (code) => {
      exitCode = code;
      resolve();
    }),
  );
  let texts = [];
  try {
    for (let attempt = 0; attempt < 12; attempt++) {
      await new Promise((resolve) => setTimeout(resolve, 500));
      assert.equal(
        exitCode,
        null,
        `Startup must show recoverable Arabic feedback instead of exiting: ${exitCode}`,
      );
      const result = spawnSync(
        'powershell.exe',
        [
          '-NoProfile',
          '-NonInteractive',
          '-File',
          'tests/native/read-owned-window-text.ps1',
          '-AppProcessId',
          String(child.pid),
        ],
        { encoding: 'utf8' },
      );
      assert.equal(result.status, 0, result.stderr);
      texts = JSON.parse(result.stdout || '[]');
      if (
        texts.some((text) => text.includes('تعذّر فتح مكتبة OpenRecall')) &&
        texts.some((text) => text.includes('إعادة المحاولة'))
      )
        break;
    }
    assert.ok(
      texts.some((text) => text.includes('تعذّر فتح مكتبة OpenRecall')),
      'A real native Arabic startup recovery dialog must be visible',
    );
    assert.ok(
      texts.some((text) => text.includes('إعادة المحاولة')),
      'Recovery must offer retry',
    );
    assert.equal(digest(), before, 'A future-format library must remain untouched');
    writeFileSync(
      join(folder, 'result.json'),
      JSON.stringify(
        {
          failure,
          nativeArabicRecovery: true,
          retryOffered: true,
          databaseUnchanged: true,
          screenReaderUsed: false,
        },
        null,
        2,
      ),
    );
    console.log(`Native Arabic startup recovery and retained database verified: ${folder}`);
  } finally {
    child.kill();
    await exited;
  }
}
