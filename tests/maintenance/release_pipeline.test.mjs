import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve, sep } from 'node:path';
import { validateReleaseContext } from '../../scripts/release-context.mjs';
import { stageArtifacts } from '../../scripts/stage-release.mjs';
import { REQUIRED_CHECKS } from '../../scripts/dependency-policy.mjs';
const sha = 'a'.repeat(40);
function context() {
  return {
    tag: 'v0.1.0',
    sha,
    branchContainsTag: true,
    versions: ['0.1.0', '0.1.0', '0.1.0'],
    checks: REQUIRED_CHECKS.map((name) => ({
      name,
      head_sha: sha,
      status: 'completed',
      conclusion: 'success',
    })),
  };
}
test('release secrets are gated by a reviewed main ancestor, matching versions and exact revision checks', () => {
  assert.deepEqual(validateReleaseContext(context()), { version: '0.1.0', sha });
  for (const change of [
    (f) => (f.branchContainsTag = false),
    (f) => (f.tag = 'v0.1.0;command'),
    (f) => (f.versions[1] = '0.2.0'),
    (f) => (f.checks[0].status = 'in_progress'),
    (f) => (f.checks[0].conclusion = 'failure'),
    (f) => (f.checks[0].head_sha = 'b'.repeat(40)),
    (f) => f.checks.pop(),
  ]) {
    const input = context();
    change(input);
    assert.throws(() => validateReleaseContext(input));
  }
});
test('normal and offline installer staging preserves both signed files and exposes one updater target', async (t) => {
  const folder = await mkdtemp(join(tmpdir(), 'openrecall-stage-test-'));
  t.after(async () => {
    assert.ok(resolve(folder).startsWith(resolve(tmpdir()) + sep));
    await rm(folder, { recursive: true, force: true });
  });
  const bundle = join(folder, 'bundle', 'nsis'),
    out = join(folder, 'out');
  await mkdir(bundle, { recursive: true });
  const source = join(bundle, 'OpenRecall_0.1.0_x64-setup.exe');
  await writeFile(source, 'normal');
  await writeFile(source + '.sig', 'normal-signature');
  await stageArtifacts({
    platform: 'windows-x86_64',
    version: '0.1.0',
    bundleRoot: join(folder, 'bundle'),
    destination: out,
  });
  await writeFile(source, 'offline');
  await writeFile(source + '.sig', 'offline-signature');
  await stageArtifacts({
    platform: 'windows-x86_64',
    version: '0.1.0',
    bundleRoot: join(folder, 'bundle'),
    destination: out,
    offline: true,
  });
  assert.equal(await readFile(join(out, 'OpenRecall_0.1.0_x64-setup.exe'), 'utf8'), 'normal');
  assert.equal(
    await readFile(join(out, 'OpenRecall_0.1.0_x64-offline-setup.exe'), 'utf8'),
    'offline',
  );
  const metadata = JSON.parse(await readFile(join(out, 'artifact-windows-x86_64.json'), 'utf8'));
  assert.equal(metadata.file, 'OpenRecall_0.1.0_x64-setup.exe');
  await assert.rejects(
    stageArtifacts({
      platform: 'windows-x86_64',
      version: '0.2.0',
      bundleRoot: join(folder, 'bundle'),
      destination: out,
    }),
  );
});
