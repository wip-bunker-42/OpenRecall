import assert from 'node:assert/strict';
import { mkdir, readdir, copyFile, access, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
async function one(folder, predicate) {
  const entries = await readdir(folder, { withFileTypes: true });
  const files = entries.filter((entry) => entry.isFile() && predicate(entry.name));
  assert.equal(files.length, 1, `Expected one matching artifact in ${folder}`);
  return join(folder, files[0].name);
}
export async function stageArtifacts({
  platform,
  version,
  bundleRoot,
  destination,
  offline = false,
}) {
  assert.match(version, /^\d+\.\d+\.\d+$/);
  assert.ok(
    ['windows-x86_64', 'linux-x86_64', 'darwin-aarch64', 'darwin-x86_64'].includes(platform),
  );
  assert.ok(!offline || platform === 'windows-x86_64');
  await mkdir(destination, { recursive: true });
  async function copy(source, name, signed) {
    await copyFile(source, join(destination, name));
    if (signed) {
      await access(source + '.sig');
      await copyFile(source + '.sig', join(destination, name + '.sig'));
    }
    return name;
  }
  let file;
  if (platform === 'windows-x86_64') {
    const source = await one(
      join(bundleRoot, 'nsis'),
      (name) => name.endsWith('.exe') && name.includes(`_${version}_`),
    );
    file = await copy(
      source,
      `OpenRecall_${version}_x64${offline ? '-offline' : ''}-setup.exe`,
      true,
    );
  } else if (platform === 'linux-x86_64') {
    const source = await one(
      join(bundleRoot, 'appimage'),
      (name) => name.endsWith('.AppImage') && name.includes(version),
    );
    file = await copy(source, `OpenRecall_${version}_linux_x86_64.AppImage`, true);
    await copy(
      await one(join(bundleRoot, 'deb'), (name) => name.endsWith('.deb') && name.includes(version)),
      `OpenRecall_${version}_amd64.deb`,
      false,
    );
  } else {
    const arch = platform === 'darwin-aarch64' ? 'aarch64' : 'x86_64';
    const source = await one(join(bundleRoot, 'macos'), (name) => name.endsWith('.app.tar.gz'));
    file = await copy(source, `OpenRecall_${version}_macos_${arch}.app.tar.gz`, true);
    await copy(
      await one(join(bundleRoot, 'dmg'), (name) => name.endsWith('.dmg') && name.includes(version)),
      `OpenRecall_${version}_macos_${arch}.dmg`,
      false,
    );
  }
  if (!offline)
    await writeFile(
      join(destination, `artifact-${platform}.json`),
      JSON.stringify({ platform, version, file }, null, 2) + '\n',
    );
  return file;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [platform, version, target, mode] = process.argv.slice(2);
  await stageArtifacts({
    platform,
    version,
    bundleRoot: resolve('target', target, 'release/bundle'),
    destination: resolve('release-assets', platform),
    offline: mode === 'offline',
  });
}
