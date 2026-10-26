import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { appendFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { pages, REPOSITORY } from './github.mjs';
import { REQUIRED_CHECKS } from './dependency-policy.mjs';
export function validateReleaseContext({ tag, sha, branchContainsTag, versions, checks }) {
  assert.match(tag, /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/);
  assert.match(sha, /^[a-f0-9]{40}$/);
  assert.equal(branchContainsTag, true, 'Release commit must already belong to main');
  const version = tag.slice(1);
  assert.equal(versions.length, 3);
  assert.ok(
    versions.every((v) => v === version),
    'Cargo, package and Tauri versions must match the tag',
  );
  for (const name of REQUIRED_CHECKS) {
    const matches = checks.filter((c) => c.name === name);
    assert.equal(matches.length, 1, `Required check missing or ambiguous: ${name}`);
    const check = matches[0];
    assert.ok(
      check.head_sha === sha && check.status === 'completed' && check.conclusion === 'success',
      `Required check is not green at release revision: ${name}`,
    );
  }
  return { version, sha };
}
function git(args) {
  const result = spawnSync('git', args, { encoding: 'utf8' });
  if (result.status !== 0) throw new Error('Cannot resolve reviewed release Git state');
  return result.stdout.trim();
}
async function main() {
  const tag = process.env.RELEASE_TAG;
  assert.match(tag ?? '', /^v\d+\.\d+\.\d+$/);
  const sha = git(['rev-parse', `refs/tags/${tag}^{commit}`]);
  const ancestor =
    spawnSync('git', ['merge-base', '--is-ancestor', sha, 'origin/main']).status === 0;
  const packageVersion = JSON.parse(git(['show', `${sha}:package.json`])).version;
  const tauriVersion = JSON.parse(git(['show', `${sha}:src-tauri/tauri.conf.json`])).version;
  const cargoVersion = git(['show', `${sha}:Cargo.toml`]).match(
    /\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/,
  )?.[1];
  const checks = await pages(
    `repos/${REPOSITORY}/commits/${sha}/check-runs?filter=latest`,
    'check_runs',
  );
  const verified = validateReleaseContext({
    tag,
    sha,
    branchContainsTag: ancestor,
    versions: [packageVersion, tauriVersion, cargoVersion],
    checks,
  });
  await appendFile(
    process.env.GITHUB_OUTPUT,
    `version=${verified.version}\nsha=${verified.sha}\ntag=${tag}\n`,
  );
  console.log(`Reviewed release context verified: ${tag} ${sha}`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  await main();
