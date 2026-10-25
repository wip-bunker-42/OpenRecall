import { spawnSync } from 'node:child_process';
import { existsSync, realpathSync } from 'node:fs';
import { join, basename, delimiter } from 'node:path';
function invocation() {
  const explicit = process.env.npm_execpath;
  if (explicit && /pnpm\.(?:c?js|mjs)$/.test(basename(explicit)) && existsSync(explicit))
    return { command: process.execPath, prefix: [explicit] };
  for (const folder of (process.env.PATH ?? '').split(delimiter).filter(Boolean)) {
    const exe = join(folder, 'pnpm.exe');
    if (process.platform === 'win32' && existsSync(exe)) return { command: exe, prefix: [] };
    const launcher = join(folder, 'pnpm');
    if (existsSync(launcher)) {
      const resolved = realpathSync(launcher);
      if (/\.(?:c?js|mjs)$/.test(resolved))
        return { command: process.execPath, prefix: [resolved] };
    }
    for (const path of [
      join(folder, 'node_modules/pnpm/bin/pnpm.cjs'),
      join(folder, '../pnpm/bin/pnpm.cjs'),
      join(folder, 'node_modules/corepack/dist/pnpm.js'),
    ])
      if (existsSync(path)) return { command: process.execPath, prefix: [realpathSync(path)] };
  }
  throw new Error(
    'Cannot locate a direct pnpm JavaScript/executable entry point. Install pinned pnpm 12.15.0.',
  );
}
export function pnpmSync(args, options = {}) {
  const { command, prefix } = invocation();
  return spawnSync(command, [...prefix, ...args], { ...options, shell: false });
}
