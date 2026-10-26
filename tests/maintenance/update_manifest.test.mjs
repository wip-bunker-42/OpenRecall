import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createManifest, validateManifest } from '../../scripts/release.mjs';
const platforms = ['windows-x86_64', 'darwin-aarch64', 'darwin-x86_64', 'linux-x86_64'];
const artifacts = platforms.map((platform) => ({
  platform,
  url: `https://github.com/wip-bunker-42/OpenRecall/releases/download/v0.1.0/${platform}.tar.gz`,
  signature: Buffer.from('signed fixture signature').toString('base64'),
}));
test('complete updater manifest uses all verified platforms and exact release URLs', () => {
  const manifest = createManifest({
    version: '0.1.0',
    notes: 'Initial release',
    publishedAt: '2026-10-25T00:00:00Z',
    artifacts,
  });
  assert.equal(validateManifest(manifest), true);
  assert.deepEqual(Object.keys(manifest.platforms), platforms);
});
test('incomplete, insecure, unsigned, duplicate and mismatched release artifacts are rejected', () => {
  for (const altered of [
    artifacts.slice(0, 3),
    [...artifacts, artifacts[0]],
    artifacts.map((a, i) => (i ? a : { ...a, signature: '' })),
    artifacts.map((a, i) => (i ? a : { ...a, url: a.url.replace('https:', 'http:') })),
    artifacts.map((a, i) => (i ? a : { ...a, url: a.url.replace('v0.1.0', 'v0.2.0') })),
  ])
    assert.throws(() =>
      createManifest({
        version: '0.1.0',
        notes: '',
        publishedAt: '2026-10-25T00:00:00Z',
        artifacts: altered,
      }),
    );
  assert.throws(() => validateManifest({ version: 'banana', platforms: {} }));
});
