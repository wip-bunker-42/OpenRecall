import { test } from 'node:test';
import assert from 'node:assert/strict';
import { dependencyChangeAllowed } from '../../scripts/dependency-policy.mjs';
const sha = 'a'.repeat(40);
function facts() {
  return {
    repository: 'wip-bunker-42/OpenRecall',
    headRepository: 'wip-bunker-42/OpenRecall',
    baseRef: 'main',
    headSha: sha,
    author: { login: 'dependabot[bot]', id: 49699333, type: 'Bot' },
    files: ['package.json', 'pnpm-lock.yaml'],
    requiredChecks: [
      'quality',
      'native-windows',
      'bundle-windows',
      'bundle-linux',
      'bundle-macos-arm64',
      'bundle-macos-x64',
    ],
    checks: [
      'quality',
      'native-windows',
      'bundle-windows',
      'bundle-linux',
      'bundle-macos-arm64',
      'bundle-macos-x64',
    ].map((name) => ({ name, headSha: sha, status: 'completed', conclusion: 'success' })),
    before: {
      name: 'openrecall',
      dependencies: { react: '19.3.1' },
      devDependencies: { vitest: '5.0.5', typescript: '6.0.3' },
    },
    after: {
      name: 'openrecall',
      dependencies: { react: '19.3.1' },
      devDependencies: { vitest: '5.1.0', typescript: '6.0.4' },
    },
  };
}
test('verified same-repository development patch/minor PR with exact-head green checks can merge', () =>
  assert.equal(dependencyChangeAllowed(facts()), true));
test('spoofed bots, forks, stale heads, failed/pending/missing checks and non-main targets cannot merge', () => {
  const cases = [
    (f) => (f.author.id = 42),
    (f) => (f.author.type = 'User'),
    (f) => (f.author.login = 'Dependabot'),
    (f) => (f.headRepository = 'attacker/OpenRecall'),
    (f) => (f.repository = 'another/repo'),
    (f) => (f.baseRef = 'release'),
    (f) => (f.headSha = 'bad'),
    (f) => (f.checks[0].headSha = 'b'.repeat(40)),
    (f) => (f.checks[0].conclusion = 'failure'),
    (f) => (f.checks[0].status = 'in_progress'),
    (f) => f.checks.pop(),
    (f) => (f.requiredChecks = []),
    (f) => f.checks.push({ ...f.checks[0], conclusion: 'failure' }),
  ];
  for (const change of cases) {
    const input = facts();
    change(input);
    assert.equal(dependencyChangeAllowed(input), false, change.toString());
  }
});
test('workflow, security, Rust/FSRS, runtime, script and major/prerelease changes require review', () => {
  const cases = [
    (f) => f.files.push('.github/workflows/quality.yml'),
    (f) => f.files.push('src-tauri/tauri.conf.json'),
    (f) => f.files.push('Cargo.lock'),
    (f) => f.files.push('crates/openrecall-core/Cargo.toml'),
    (f) => (f.after.dependencies.react = '19.4.0'),
    (f) => (f.after.scripts = { postinstall: 'node code.mjs' }),
    (f) => (f.after.devDependencies.vitest = '6.0.0'),
    (f) => (f.after.devDependencies.vitest = '5.1.0-beta.1'),
    (f) => (f.after.devDependencies.vitest = '^5.1.0'),
    (f) => (f.after.devDependencies['@tauri-apps/cli'] = '2.13.0'),
    (f) => delete f.after.devDependencies.typescript,
    (f) => (f.after.devDependencies.vitest = '4.9.0'),
    (f) => (f.after = structuredClone(f.before)),
  ];
  for (const change of cases) {
    const input = facts();
    change(input);
    assert.equal(dependencyChangeAllowed(input), false, change.toString());
  }
});
