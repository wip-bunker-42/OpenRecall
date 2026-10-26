import { mkdir, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
const key = `${process.platform}-${process.arch}`;
const archives = {
  'win32-x64': [
    'windows_amd64.zip',
    '6e7241b51e6817ea6a047693d8e6fed13b31819c9a0dd6c5a726e1592d22f6e9',
  ],
  'linux-x64': [
    'linux_amd64.tar.gz',
    '8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8',
  ],
  'darwin-x64': [
    'darwin_amd64.tar.gz',
    '5b44c3bc2255115c9b69e30efc0fecdf498fdb63c5d58e17084fd5f16324c644',
  ],
  'darwin-arm64': [
    'darwin_arm64.tar.gz',
    'aba9ced2dee8d27fecca3dc7feb1a7f9a52caefa1eb46f3271ea66b6e0e6953f',
  ],
};
if (!archives[key]) throw new Error(`Install actionlint 1.7.12 manually for ${key}`);
const [suffix, expected] = archives[key],
  name = `actionlint_1.7.12_${suffix}`,
  folder = resolve('target/tools/actionlint');
await mkdir(folder, { recursive: true });
const response = await fetch(
  `https://github.com/rhysd/actionlint/releases/download/v1.7.12/${name}`,
);
if (!response.ok) throw new Error(`Actionlint download failed: ${response.status}`);
const bytes = Buffer.from(await response.arrayBuffer());
if (createHash('sha256').update(bytes).digest('hex') !== expected)
  throw new Error('Actionlint archive checksum mismatch');
const archive = join(folder, name);
await writeFile(archive, bytes);
const extracted = spawnSync('tar', ['-xf', archive, '-C', folder], { stdio: 'inherit' });
if (extracted.status !== 0) throw new Error('Cannot extract verified actionlint');
const version = spawnSync(
  join(folder, process.platform === 'win32' ? 'actionlint.exe' : 'actionlint'),
  ['-version'],
  { encoding: 'utf8' },
);
if (version.status !== 0 || !version.stdout.includes('1.7.12'))
  throw new Error('Unexpected actionlint version');
console.log('Installed checksum-verified actionlint 1.7.12');
