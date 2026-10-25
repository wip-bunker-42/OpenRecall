import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { pnpmSync } from './tool-process.mjs';
import { createHash } from 'node:crypto';
import { join, basename } from 'node:path';
import assert from 'node:assert/strict';
const reportPath = process.argv[2];
assert.ok(reportPath, 'Usage: generate-notices.mjs CARGO_ABOUT_JSON');
const report = JSON.parse(await readFile(reportPath, 'utf8'));
const listed = pnpmSync(['licenses', 'list', '--prod', '--json'], { encoding: 'utf8' });
assert.equal(listed.status, 0);
const npm = Object.values(JSON.parse(listed.stdout)).flat();
await mkdir('licenses/sources', { recursive: true });
const crates = new Map(),
  mpl = new Map();
let rust =
  'OpenRecall Rust third-party license texts\nGenerated with cargo-about 0.9.2 for the four production targets; development/build-only dependencies excluded.\n\n';
function sourceUrl(crate) {
  return crate.source?.startsWith('git+')
    ? `https://github.com/open-spaced-repetition/fsrs-rs/tree/0fd59949e9168af7194418c2dba99836b5548e87`
    : `https://crates.io/api/v1/crates/${encodeURIComponent(crate.name)}/${encodeURIComponent(crate.version)}/download`;
}
for (const license of report.licenses) {
  const users = license.used_by
    .map((item) => item.crate)
    .filter((crate) => !crate.name.startsWith('openrecall-'));
  if (!users.length) continue;
  rust += `\n${'='.repeat(72)}\n${license.name} (${license.id})\nUsed by:\n`;
  for (const crate of users) {
    assert.match(crate.name, /^[A-Za-z0-9_-]+$/);
    const record = {
      name: crate.name,
      version: crate.version,
      license: crate.license,
      source: sourceUrl(crate),
    };
    crates.set(`${crate.name}@${crate.version}`, record);
    if (license.id === 'MPL-2.0') mpl.set(`${crate.name}@${crate.version}`, record);
    rust += `- ${crate.name} ${crate.version}: ${record.source}\n`;
  }
  rust += `\n${license.text}\n`;
}
await writeFile('licenses/RUST.txt', rust);
let frontend = 'OpenRecall frontend and font license texts\nProduction npm packages only.\n\n';
const frontendPackages = [];
for (const pkg of npm.sort((a, b) => (a.name < b.name ? -1 : 1))) {
  frontendPackages.push({ name: pkg.name, versions: pkg.versions, license: pkg.license });
  frontend += `\n${'='.repeat(72)}\n${pkg.name} ${pkg.versions.join(', ')}\nLicense: ${pkg.license}\n${pkg.homepage ?? ''}\n\n`;
  const folder = pkg.paths[0];
  const files = (await readdir(folder))
    .filter(
      (name) => /^(LICENSE|LICENCE|COPYING|OFL)(?:$|[-.])/i.test(name) && !name.endsWith('.spdx'),
    )
    .sort();
  assert.ok(files.length, `No license text found for ${pkg.name}`);
  for (const file of files) frontend += `${file}\n${await readFile(join(folder, file), 'utf8')}\n`;
}
await writeFile('licenses/FRONTEND.txt', frontend);
let sources =
  '# Third-party source locations\n\nThese are the unmodified upstream sources corresponding to the licensed Rust\npackages. MPL-2.0 crate archives are also included under `sources/`.\n\n';
const sorted = [...crates.values()].sort((a, b) =>
  `${a.name}@${a.version}` < `${b.name}@${b.version}` ? -1 : 1,
);
for (const crate of sorted)
  sources += `- [${crate.name} ${crate.version}](${crate.source}) — ${crate.license}\n`;
await writeFile('licenses/SOURCES.md', sources);
const sourceArchives = [];
for (const crate of mpl.values()) {
  const name = `${crate.name}-${crate.version}.crate`;
  assert.equal(basename(name), name);
  const response = await fetch(crate.source, {
    headers: { 'User-Agent': 'OpenRecall third-party source packaging' },
  });
  assert.ok(response.ok, `Cannot fetch MPL source ${crate.name}`);
  const bytes = Buffer.from(await response.arrayBuffer());
  assert.ok(bytes.length < 10 * 1024 * 1024);
  await writeFile(join('licenses/sources', name), bytes);
  sourceArchives.push({ file: name, sha256: createHash('sha256').update(bytes).digest('hex') });
}
const manifest = {
  generator: 'cargo-about 0.9.2; pnpm licenses --prod',
  cargoLockSha256: createHash('sha256')
    .update(await readFile('Cargo.lock'))
    .digest('hex'),
  rustCrates: sorted,
  frontendPackages,
  sourceArchives,
};
await writeFile('licenses/manifest.json', JSON.stringify(manifest, null, 2) + '\n');
console.log(
  `Packaged notices: ${sorted.length} Rust crates, ${frontendPackages.length} npm packages, ${sourceArchives.length} MPL source archives.`,
);
