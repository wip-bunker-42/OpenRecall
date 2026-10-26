import { readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname, basename } from 'node:path';
import { pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';
const platforms = ['windows-x86_64', 'darwin-aarch64', 'darwin-x86_64', 'linux-x86_64'];
const semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?$/;
export function validateManifest(manifest) {
  assert.ok(manifest && semver.test(manifest.version), 'A release must have a semantic version');
  assert.equal(typeof manifest.notes, 'string');
  assert.ok(Number.isFinite(Date.parse(manifest.pub_date)), 'A publish timestamp is required');
  assert.deepEqual(
    Object.keys(manifest.platforms).sort(),
    [...platforms].sort(),
    'All supported updater platforms are required',
  );
  for (const item of Object.values(manifest.platforms)) {
    const url = new URL(item.url);
    assert.equal(url.protocol, 'https:');
    assert.equal(url.hostname, 'github.com');
    assert.ok(
      url.pathname.startsWith(`/wip-bunker-42/OpenRecall/releases/download/v${manifest.version}/`),
    );
    assert.ok(!url.username && !url.password && !url.search && !url.hash);
    assert.ok(
      /^[A-Za-z0-9+/]+={0,2}$/.test(item.signature) &&
        Buffer.from(item.signature, 'base64').length > 0,
      'An artifact signature is required',
    );
  }
  return true;
}
export function createManifest({ version, notes, publishedAt, artifacts }) {
  const manifest = { version, notes, pub_date: publishedAt, platforms: {} };
  for (const { platform, url, signature } of artifacts) {
    assert.ok(platforms.includes(platform), 'Unknown platform');
    assert.ok(!manifest.platforms[platform], 'Duplicate platform');
    manifest.platforms[platform] = { url, signature: signature.trim() };
  }
  validateManifest(manifest);
  return manifest;
}
async function main() {
  const [metadataPath, outputPath] = process.argv.slice(2);
  assert.ok(
    metadataPath && outputPath,
    'Usage: node scripts/release.mjs release-input.json latest.json',
  );
  const input = JSON.parse(await readFile(metadataPath, 'utf8'));
  const artifacts = [];
  for (const artifact of input.artifacts) {
    const path = resolve(dirname(metadataPath), artifact.file);
    artifacts.push({
      platform: artifact.platform,
      url: `https://github.com/wip-bunker-42/OpenRecall/releases/download/v${input.version}/${encodeURIComponent(basename(path))}`,
      signature: await readFile(path + '.sig', 'utf8'),
    });
  }
  const manifest = createManifest({ ...input, artifacts });
  await writeFile(outputPath, JSON.stringify(manifest, null, 2) + '\n');
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  await main();
