import { test } from 'node:test';
import assert from 'node:assert/strict';
import { pnpmSync } from '../../scripts/tool-process.mjs';
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
