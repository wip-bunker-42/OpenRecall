import { isDeepStrictEqual } from 'node:util';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { github, pages, REPOSITORY } from './github.mjs';
export const REQUIRED_CHECKS = [
  'quality',
  'native-windows',
  'bundle-windows',
  'bundle-linux',
  'bundle-macos-arm64',
  'bundle-macos-x64',
];
const allowed = new Set([
  '@eslint/js',
  'eslint',
  'globals',
  'prettier',
  'typescript',
  'typescript-eslint',
  'vite',
  '@vitejs/plugin-react',
  'vitest',
  'jsdom',
]);
const version = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
function developmentTool(name) {
  return (
    allowed.has(name) ||
    name.startsWith('@types/') ||
    name.startsWith('@testing-library/') ||
    name === '@playwright/test' ||
    name === '@axe-core/playwright'
  );
}
function patchOrMinor(before, after) {
  if (!version.test(before) || !version.test(after)) return false;
  const a = before.split('.').map(Number),
    b = after.split('.').map(Number);
  return a[0] === b[0] && (b[1] > a[1] || (b[1] === a[1] && b[2] > a[2]));
}
export function dependencyChangeAllowed(input) {
  try {
    if (
      input.repository !== REPOSITORY ||
      input.headRepository !== REPOSITORY ||
      input.baseRef !== 'main' ||
      !/^[0-9a-f]{40}$/.test(input.headSha)
    )
      return false;
    const author = input.author;
    if (author?.login !== 'dependabot[bot]' || author.id !== 49699333 || author.type !== 'Bot')
      return false;
    if (
      !input.files.length ||
      input.files.some((path) => !['package.json', 'pnpm-lock.yaml'].includes(path))
    )
      return false;
    if (!isDeepStrictEqual([...input.requiredChecks].sort(), [...REQUIRED_CHECKS].sort()))
      return false;
    if (
      !input.checks.length ||
      input.checks.some(
        (check) =>
          check.headSha !== input.headSha ||
          check.status !== 'completed' ||
          check.conclusion !== 'success',
      )
    )
      return false;
    if (
      REQUIRED_CHECKS.some(
        (name) => input.checks.filter((check) => check.name === name).length !== 1,
      )
    )
      return false;
    const { devDependencies: before, ...beforeRest } = input.before,
      { devDependencies: after, ...afterRest } = input.after;
    if (
      !isDeepStrictEqual(beforeRest, afterRest) ||
      !isDeepStrictEqual(Object.keys(before).sort(), Object.keys(after).sort())
    )
      return false;
    let changed = 0;
    for (const name of Object.keys(before)) {
      if (before[name] !== after[name]) {
        if (!developmentTool(name) || !patchOrMinor(before[name], after[name])) return false;
        changed++;
      }
    }
    return changed > 0;
  } catch {
    return false;
  }
}
async function packageAt(sha) {
  const file = await github(`repos/${REPOSITORY}/contents/package.json?ref=${sha}`);
  return JSON.parse(Buffer.from(file.content, 'base64').toString('utf8'));
}
async function main() {
  const event = JSON.parse(await readFile(process.env.GITHUB_EVENT_PATH, 'utf8'));
  if (event.repository?.full_name !== REPOSITORY) return;
  const sha = event.workflow_run?.head_sha;
  if (event.workflow_run?.conclusion !== 'success' || !/^[0-9a-f]{40}$/.test(sha ?? '')) return;
  const related = await pages(`repos/${REPOSITORY}/commits/${sha}/pulls`);
  for (const candidate of related.filter((pr) => pr.state === 'open')) {
    const pr = await github(`repos/${REPOSITORY}/pulls/${candidate.number}`);
    if (pr.head.sha !== sha) continue;
    const [files, runs, statuses, before, after] = await Promise.all([
      pages(`repos/${REPOSITORY}/pulls/${pr.number}/files`),
      pages(`repos/${REPOSITORY}/commits/${sha}/check-runs?filter=latest`, 'check_runs'),
      pages(`repos/${REPOSITORY}/commits/${sha}/statuses`),
      packageAt(pr.base.sha),
      packageAt(sha),
    ]);
    const checks = runs.map((run) => ({
      name: run.name,
      headSha: run.head_sha,
      status: run.status,
      conclusion: run.conclusion,
    }));
    for (const status of statuses) {
      if (checks.some((c) => c.name === status.context)) continue;
      checks.push({
        name: status.context,
        headSha: status.sha ?? sha,
        status: status.state === 'pending' ? 'pending' : 'completed',
        conclusion: status.state,
      });
    }
    const facts = {
      repository: REPOSITORY,
      headRepository: pr.head.repo?.full_name,
      baseRef: pr.base.ref,
      headSha: sha,
      author: pr.user,
      files: files.map((file) => file.filename),
      requiredChecks: REQUIRED_CHECKS,
      checks,
      before,
      after,
    };
    if (!dependencyChangeAllowed(facts)) {
      console.log(`PR #${pr.number} requires review; automatic merge policy did not pass.`);
      continue;
    }
    // The SHA precondition is atomic: a changed head cannot inherit an earlier approval.
    const result = await github(`repos/${REPOSITORY}/pulls/${pr.number}/merge`, {
      method: 'PUT',
      body: { sha, merge_method: 'squash' },
    });
    if (!result.merged) throw new Error(`GitHub did not merge PR #${pr.number}`);
    console.log(`Merged verified dependency PR #${pr.number}: ${sha}`);
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  await main();
