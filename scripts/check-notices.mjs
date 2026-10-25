import { readFile } from 'node:fs/promises';
import { pnpmSync } from './tool-process.mjs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
const manifest = JSON.parse(await readFile('licenses/manifest.json', 'utf8'));
assert.equal(
  createHash('sha256')
    .update(await readFile('Cargo.lock'))
    .digest('hex'),
  manifest.cargoLockSha256,
  'Cargo graph changed: regenerate bundled notices',
);
const listed = pnpmSync(['licenses', 'list', '--prod', '--json'], { encoding: 'utf8' });
assert.equal(listed.status, 0);
const packages = Object.values(JSON.parse(listed.stdout))
  .flat()
  .sort((a, b) => (a.name < b.name ? -1 : 1))
  .map((pkg) => ({ name: pkg.name, versions: pkg.versions, license: pkg.license }));
assert.deepEqual(
  packages,
  manifest.frontendPackages,
  'Production npm dependencies changed: regenerate bundled notices',
);
for (const entry of manifest.sourceArchives)
  assert.equal(
    createHash('sha256')
      .update(await readFile(`licenses/sources/${entry.file}`))
      .digest('hex'),
    entry.sha256,
  );
for (const name of [
  'LICENSE',
  'THIRD_PARTY_NOTICES.md',
  'licenses/RUST.txt',
  'licenses/FRONTEND.txt',
  'licenses/SOURCES.md',
])
  assert.ok((await readFile(name)).length > 0);
console.log('Bundled third-party notices match the production dependency graph.');
