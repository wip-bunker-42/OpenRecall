import { test } from 'node:test';
import assert from 'node:assert/strict';
import { pnpmSync, pnpmInvocation } from '../../scripts/tool-process.mjs';
import { mkdtempSync, writeFileSync, rmSync, realpathSync } from 'node:fs';
import { join, dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
test('resolves extensionless pnpm native executables installed by Actions on POSIX', () => {
  const folder = mkdtempSync(join(tmpdir(), 'openrecall-pnpm-'));
  try {
    const binary = join(folder, 'pnpm');
    writeFileSync(binary, 'synthetic native executable');
    assert.deepEqual(pnpmInvocation({ PATH: folder }, 'linux'), {
      command: realpathSync(binary),
      prefix: [],
    });
  } finally {
    assert.equal(dirname(resolve(folder)), resolve(tmpdir()));
    rmSync(folder, { recursive: true });
  }
});
test('pinned pnpm runs directly without cmd shell argument concatenation', () => {
  const version = pnpmSync(['--version'], { encoding: 'utf8' });
  assert.equal(version.status, 0);
  assert.equal(version.stdout.trim(), '12.15.0');
  assert.ok(!version.stderr.includes('DEP0190'));
  const literal = 'value with spaces; $() & literal';
  const result = pnpmSync(
    ['exec', process.execPath, '-e', 'process.stdout.write(process.argv[1])', '--', literal],
    { encoding: 'utf8' },
  );
  assert.equal(result.status, 0);
  assert.equal(result.stdout, literal);
});
