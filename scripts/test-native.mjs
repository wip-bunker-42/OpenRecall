import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { pnpmInvocation } from './tool-process.mjs';
import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:http';
if (process.platform !== 'win32')
  throw new Error('The current native runtime suite targets Windows WebView2.');
const parent = resolve('target/native-tests');
mkdirSync(parent, { recursive: true });
const root = mkdtempSync(join(parent, 'run-'));
writeFileSync(
  join(root, 'cards.json'),
  JSON.stringify([
    {
      front: 'Imported English سؤال',
      back: '日本語 answer',
      notes: 'ملاحظة',
      variants: [{ front: '別の質問', back: 'بديل', notes: '' }],
    },
  ]),
);
console.log(`Isolated native QA artifacts: ${root}`);
const blocked = [];
const proxy = createServer((request, response) => {
  blocked.push({ method: request.method, target: request.url });
  response.writeHead(503);
  response.end('Offline native test boundary');
});
proxy.on('connect', (request, socket) => {
  blocked.push({ method: request.method, target: request.url });
  socket.end('HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\n\r\n');
});
await new Promise((resolve, reject) => {
  proxy.once('error', reject);
  proxy.listen(4459, '127.0.0.1', resolve);
});
async function invoke(args) {
  const { command, prefix } = pnpmInvocation();
  const status = await new Promise((resolve, reject) => {
    const child = spawn(command, [...prefix, ...args], {
      stdio: 'inherit',
      shell: false,
      env: { ...process.env, OPENRECALL_NATIVE_ROOT: root },
    });
    child.once('error', reject);
    child.once('exit', resolve);
  });
  if (status !== 0) throw new Error(`Native suite exited with ${status}`);
}
async function run(args) {
  await invoke(['exec', 'wdio', 'run', 'wdio.conf.ts', ...args]);
}
try {
  if (process.argv.length > 2) await run(process.argv.slice(2));
  else {
    await run(['--spec', 'tests/native/persistence.spec.ts']);
    const stopped = spawnSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-NonInteractive',
        '-Command',
        '$count=@(Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -eq $env:OPENRECALL_TEST_BINARY }).Count; if ($count -ne 0) { exit 1 }',
      ],
      {
        env: {
          ...process.env,
          OPENRECALL_TEST_BINARY: resolve('target/native-automation/OpenRecallAutomation.exe'),
        },
        encoding: 'utf8',
      },
    );
    if (stopped.status !== 0)
      throw new Error(
        'The first native app process did not exit; restart persistence is unverified.',
      );
    console.log(
      'Confirmed first app process exited. Restarting against the same isolated SQLite library.',
    );
    await run([
      '--spec',
      'tests/native/review.spec.ts',
      '--spec',
      'tests/native/offline.spec.ts',
      '--spec',
      'tests/native/focus.spec.ts',
    ]);
    if (
      !blocked.some(
        (request) => request.method === 'CONNECT' && request.target === 'github.com:443',
      )
    )
      throw new Error('No real native updater connection reached the rejecting proxy.');
    await invoke([
      'exec',
      process.execPath,
      'scripts/test-startup.mjs',
      join(root, 'data/openrecall.sqlite3'),
    ]);
  }
} finally {
  writeFileSync(
    join(root, 'network-boundary.json'),
    JSON.stringify({ policy: 'reject every request; never forward', blocked }, null, 2),
  );
  proxy.close();
  proxy.closeAllConnections();
}
