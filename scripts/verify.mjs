import { spawnSync } from 'node:child_process';
import { readFileSync, existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { pnpmSync } from './tool-process.mjs';
const root = JSON.parse(readFileSync('package.json', 'utf8'));
function run(command, args) {
  const result =
    command === 'pnpm'
      ? pnpmSync(args, { stdio: 'inherit' })
      : spawnSync(command, args, { stdio: 'inherit' });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
run('cargo', ['fmt', '--all', '--', '--check']);
run('cargo', ['clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings']);
run('cargo', ['test', '--workspace', '--locked']);
run('cargo', [
  'test',
  '--locked',
  '-p',
  'openrecall-core',
  '--features',
  'serde_json/preserve_order',
  '--test',
  'backups',
]);
run('pnpm', ['contracts']);
run('git', ['diff', '--exit-code', '--', 'src/shared/generated']);
run('pnpm', ['typecheck']);
run('pnpm', ['lint']);
run('pnpm', ['format:check']);
run('pnpm', ['test']);
const maintenance = readdirSync('tests/maintenance')
  .filter((file) => file.endsWith('.test.mjs'))
  .map((file) => join('tests/maintenance', file));
run(process.execPath, ['--test', ...maintenance]);
run(process.execPath, ['scripts/check-notices.mjs']);
run('pnpm', ['build']);
if (root.scripts['test:e2e']) run('pnpm', ['test:e2e']);
if (process.env.OPENRECALL_VERIFY_NATIVE === '1' && root.scripts['test:native'])
  run('pnpm', ['test:native']);
const lint =
  process.platform === 'win32'
    ? 'target/tools/actionlint/actionlint.exe'
    : 'target/tools/actionlint/actionlint';
if (!existsSync(lint)) run(process.execPath, ['scripts/install-actionlint.mjs']);
run(lint, []);
console.log('OpenRecall verification passed.');
