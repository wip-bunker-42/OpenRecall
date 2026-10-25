import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';
import { github, pages, REPOSITORY } from './github.mjs';
import { validateManifest } from './release.mjs';
const [folderInput, version] = process.argv.slice(2);
assert.ok(folderInput && /^\d+\.\d+\.\d+$/.test(version));
const folder = resolve(folderInput),
  tag = `v${version}`,
  sha = process.env.RELEASE_SHA;
assert.match(sha ?? '', /^[a-f0-9]{40}$/);
const manifest = JSON.parse(await readFile(join(folder, 'latest.json'), 'utf8'));
validateManifest(manifest);
assert.equal(manifest.version, version);
const marker = `<!-- openrecall-release:${tag}:${sha} -->`;
let release = await github(`repos/${REPOSITORY}/releases/tags/${tag}`, { allowMissing: true });
if (release) {
  assert.equal(release.draft, true, 'Published releases are immutable; refuse to overwrite');
  assert.ok(release.body?.includes(marker), 'Refuse to replace an unrelated draft');
} else
  release = await github(`repos/${REPOSITORY}/releases`, {
    method: 'POST',
    body: {
      tag_name: tag,
      target_commitish: sha,
      name: `OpenRecall ${version}`,
      body: `${marker}\n\n${await readFile(`docs/releases/${version}.md`, 'utf8')}`,
      draft: true,
      prerelease: false,
    },
  });
const names = (await readdir(folder)).filter((name) => !name.startsWith('artifact-')).sort();
let existing = await pages(`repos/${REPOSITORY}/releases/${release.id}/assets`);
assert.ok(
  existing.every((asset) => names.includes(asset.name)),
  'Draft contains unexpected assets; inspect before publication',
);
const expected = new Map();
for (const name of names) {
  const bytes = await readFile(join(folder, name));
  const digest = `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
  expected.set(name, digest);
  const prior = existing.find((asset) => asset.name === name);
  if (prior?.digest === digest) continue;
  if (prior) await github(`repos/${REPOSITORY}/releases/assets/${prior.id}`, { method: 'DELETE' });
  const uploaded = await fetch(
    `https://uploads.github.com/repos/${REPOSITORY}/releases/${release.id}/assets?name=${encodeURIComponent(name)}`,
    {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${process.env.GH_TOKEN}`,
        Accept: 'application/vnd.github+json',
        'Content-Type': 'application/octet-stream',
        'X-GitHub-Api-Version': '2022-11-28',
      },
      body: bytes,
    },
  );
  if (!uploaded.ok) throw new Error(`Release upload failed (${uploaded.status}): ${name}`);
  const asset = await uploaded.json();
  assert.equal(asset.digest, digest, `GitHub upload digest mismatch: ${name}`);
  console.log(`Uploaded and verified: ${name}`);
}
existing = await pages(`repos/${REPOSITORY}/releases/${release.id}/assets`);
assert.equal(existing.length, names.length);
for (const asset of existing) assert.equal(asset.digest, expected.get(asset.name));
const published = await github(`repos/${REPOSITORY}/releases/${release.id}`, {
  method: 'PATCH',
  body: { draft: false, make_latest: 'true' },
});
assert.equal(published.draft, false);
console.log(`Published complete verified release: ${published.html_url}`);
