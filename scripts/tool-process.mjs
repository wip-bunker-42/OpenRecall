import { spawnSync } from 'node:child_process';
import { existsSync, realpathSync } from 'node:fs';
import { join, basename, delimiter } from 'node:path';
export function pnpmInvocation(env = process.env, platform = process.platform) {
  const explicit = env.npm_execpath;
  if (explicit && /pnpm\.(?:c?js|mjs)$/.test(basename(explicit)) && existsSync(explicit))
    return { command: process.execPath, prefix: [explicit] };
  for (const folder of (env.PATH ?? '').split(delimiter).filter(Boolean)) {
    const exe = join(folder, 'pnpm.exe');
    if (platform === 'win32' && existsSync(exe)) return { command: exe, prefix: [] };
    const launcher = join(folder, 'pnpm');
    if (existsSync(launcher)) {
      const resolved = realpathSync(launcher);
      if (/\.(?:c?js|mjs)$/.test(resolved))
        return { command: process.execPath, prefix: [resolved] };
      if (platform !== 'win32') return { command: resolved, prefix: [] };
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
  const { command, prefix } = pnpmInvocation();
  return spawnSync(command, [...prefix, ...args], { ...options, shell: false });
}
