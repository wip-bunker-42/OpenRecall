import assert from 'node:assert/strict';
import { readFile, writeFile, readdir, lstat } from 'node:fs/promises';
import { resolve, join, basename } from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { createManifest } from './release.mjs';
const [folderInput, version] = process.argv.slice(2);
assert.ok(
  folderInput && /^\d+\.\d+\.\d+$/.test(version),
  'Usage: assemble-release.mjs ASSETS VERSION',
);
const folder = resolve(folderInput);
const names = await readdir(folder),
  artifacts = [];
const metadataNames = names.filter((name) => /^artifact-.*\.json$/.test(name));
assert.equal(metadataNames.length, 4, 'All four platform builds must supply metadata');
for (const metadataName of metadataNames) {
  const metadata = JSON.parse(await readFile(join(folder, metadataName), 'utf8'));
  assert.equal(metadata.version, version);
  assert.equal(basename(metadata.file), metadata.file);
  assert.ok((await lstat(join(folder, metadata.file))).isFile());
  artifacts.push({
    ...metadata,
    url: `https://github.com/wip-bunker-42/OpenRecall/releases/download/v${version}/${encodeURIComponent(metadata.file)}`,
    signature: (await readFile(join(folder, metadata.file + '.sig'), 'utf8')).trim(),
  });
}
// Verification receives no signing credential. It uses the official updater and
// the public key from this reviewed revision, including version binding.
for (const name of names.filter((name) => name.endsWith('.sig'))) {
  const file = name.slice(0, -4);
  assert.equal(basename(file), file);
  assert.ok((await lstat(join(folder, file))).isFile());
  const platform = artifacts.find((a) => a.file === file)?.platform ?? 'extra-artifact';
  const verified = spawnSync(
    'cargo',
    [
      'run',
      '--locked',
      '-p',
      'openrecall-desktop',
      '--example',
      'verify_update',
      '--',
      join(folder, file),
      join(folder, name),
      version,
      platform,
    ],
    { stdio: 'inherit' },
  );
  assert.equal(verified.status, 0, `Official signature verification failed: ${file}`);
}
assert.ok(
  names.includes(`OpenRecall_${version}_x64-offline-setup.exe`),
  'Offline installer is required',
);
assert.ok(names.includes(`OpenRecall_${version}_x64-offline-setup.exe.sig`));
const notes = await readFile(`docs/releases/${version}.ar.md`, 'utf8');
const manifest = createManifest({
  version,
  notes,
  publishedAt: new Date().toISOString(),
  artifacts,
});
await writeFile(join(folder, 'latest.json'), JSON.stringify(manifest, null, 2) + '\n');
const publicNames = (await readdir(folder))
  .filter((name) => !name.startsWith('artifact-') && name !== 'SHA256SUMS.txt')
  .sort();
const sums = [];
for (const name of publicNames) {
  assert.ok((await lstat(join(folder, name))).isFile());
  sums.push(
    `${createHash('sha256')
      .update(await readFile(join(folder, name)))
      .digest('hex')}  ${name}`,
  );
}
await writeFile(join(folder, 'SHA256SUMS.txt'), sums.join('\n') + '\n');
console.log(`Complete signed release assembled: ${publicNames.length} assets`);
