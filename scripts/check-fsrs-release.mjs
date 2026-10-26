import { readFile, writeFile, mkdtemp, cp, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join, dirname, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';
import { notifyOnce } from './github.mjs';
const stable = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
export function parseEnginePin(manifest, lock) {
  const declaration = manifest.match(/^fsrs\s*=\s*(.+)$/m)?.[1];
  const blocks = lock.split('[[package]]').filter((block) => /^name\s*=\s*"fsrs"\s*$/m.test(block));
  const currentVersion = blocks[0]?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!declaration || blocks.length !== 1 || !stable.test(currentVersion ?? ''))
    throw new Error('Cannot resolve a unique stable FSRS lockfile baseline');
  return { currentVersion, gitPinned: /\bgit\s*=/.test(declaration) };
}
function newer(a, b) {
  const x = a.split('.').map(Number),
    y = b.split('.').map(Number);
  for (let i = 0; i < 3; i++) {
    if (x[i] !== y[i]) return x[i] > y[i];
  }
  return false;
}
export function inspectFsrsRelease(metadata, source, goldenResults) {
  const { version, currentVersion, seenVersions = [], yanked } = metadata;
  const marker = `<!-- openrecall-fsrs-release:${version} -->`;
  const result = (reason) => ({ actionable: reason === 'compatible', version, reason, marker });
  if (
    !stable.test(version) ||
    !stable.test(currentVersion) ||
    yanked ||
    !newer(version, currentVersion)
  )
    return result('notNewStableRelease');
  if (seenVersions.includes(version)) return result('alreadyNotified');
  if (
    ![
      'stability_fast',
      'next_states_with_elapsed_days',
      'compute_parameters',
      'enable_short_term',
      'schedule_penalties',
    ].every((api) => source.includes(api))
  )
    return result('missingFsrs7Api');
  if (
    !Array.isArray(goldenResults) ||
    goldenResults.some((test) => test.passed !== true) ||
    ['fsrs7State', 'intraday', 'optimizer'].some(
      (name) => goldenResults.filter((test) => test.name === name).length !== 1,
    )
  )
    return result('goldensNotPassed');
  return result('compatible');
}
async function sourceText(folder) {
  let text = '';
  for (const entry of await readdir(folder, { withFileTypes: true })) {
    const path = join(folder, entry.name);
    if (entry.isDirectory()) text += await sourceText(path);
    else if (entry.isFile() && entry.name.endsWith('.rs')) text += await readFile(path, 'utf8');
    if (text.length > 8 * 1024 * 1024) throw new Error('FSRS source exceeded inspection limit');
  }
  return text;
}
async function probe(output) {
  // Run only on an ephemeral read-only-permission CI job. Notification happens
  // on a separate clean runner so upstream build scripts never receive its token.
  if (process.env.GITHUB_ACTIONS !== 'true')
    throw new Error('Live upstream probes run only in the isolated GitHub probe job');
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  const pin = parseEnginePin(
    await readFile(join(root, 'crates/openrecall-core/Cargo.toml'), 'utf8'),
    await readFile(join(root, 'Cargo.lock'), 'utf8'),
  );
  if (!pin.gitPinned) {
    await writeFile(
      output,
      JSON.stringify({
        actionable: false,
        version: pin.currentVersion,
        reason: 'alreadyUsingRegistry',
      }),
    );
    return;
  }
  const response = await fetch('https://crates.io/api/v1/crates/fsrs', {
    headers: {
      'User-Agent': 'OpenRecall FSRS compatibility watcher (github.com/wip-bunker-42/OpenRecall)',
    },
  });
  if (!response.ok) throw new Error(`crates.io lookup failed: ${response.status}`);
  const info = await response.json();
  const release = info.versions.find((item) => !item.yanked && stable.test(item.num));
  if (!release) throw new Error('No stable fsrs release found');
  const metadata = {
    version: release.num,
    currentVersion: pin.currentVersion,
    yanked: release.yanked,
  };
  let result = inspectFsrsRelease(metadata, '', []);
  if (result.reason === 'notNewStableRelease') {
    await writeFile(output, JSON.stringify(result));
    return;
  }
  const folder = await mkdtemp(join(tmpdir(), 'openrecall-fsrs-probe-'));
  const cleanEnv = { ...process.env };
  for (const name of Object.keys(cleanEnv))
    if (/TOKEN|SECRET|PASSWORD|SIGNING_PRIVATE_KEY/i.test(name)) delete cleanEnv[name];
  function cargo(args) {
    return spawnSync('cargo', args, {
      cwd: folder,
      env: cleanEnv,
      encoding: 'utf8',
      timeout: 20 * 60 * 1000,
      maxBuffer: 16 * 1024 * 1024,
    });
  }
  try {
    await cp(join(root, 'crates/openrecall-core'), join(folder, 'crates/openrecall-core'), {
      recursive: true,
    });
    await writeFile(
      join(folder, 'Cargo.toml'),
      '[workspace]\nmembers=["crates/openrecall-core"]\nresolver="3"\n[workspace.package]\nversion="0.1.0"\nedition="2024"\nrust-version="1.90"\nlicense="MIT"\n',
    );
    const manifestPath = join(folder, 'crates/openrecall-core/Cargo.toml');
    const manifest = await readFile(manifestPath, 'utf8');
    const pattern = /fsrs\s*=\s*\{[^}]+\}/g;
    if ([...manifest.matchAll(pattern)].length !== 1)
      throw new Error('Pinned engine declaration changed; inspect watcher integration');
    await writeFile(
      manifestPath,
      manifest.replace(pattern, `fsrs = { version = "=${release.num}" }`),
    );
    const metadataRun = cargo(['metadata', '--format-version', '1']);
    if (metadataRun.status !== 0) throw new Error('Cannot resolve candidate crate metadata');
    const packages = JSON.parse(metadataRun.stdout).packages;
    const fsrs = packages.find((pkg) => pkg.name === 'fsrs' && pkg.version === release.num);
    if (!fsrs) throw new Error('Candidate crate metadata missing');
    const source = await sourceText(join(dirname(fsrs.manifest_path), 'src'));
    result = inspectFsrsRelease(metadata, source, []);
    if (result.reason !== 'missingFsrs7Api') {
      const tested = cargo([
        'test',
        '-p',
        'openrecall-core',
        '--test',
        'fsrs_adapter',
        '--test',
        'training_jobs',
      ]);
      console.log((tested.stderr ?? '').slice(-6000));
      result = inspectFsrsRelease(
        metadata,
        source,
        ['fsrs7State', 'intraday', 'optimizer'].map((name) => ({
          name,
          passed: tested.status === 0,
        })),
      );
    }
    await writeFile(output, JSON.stringify(result, null, 2) + '\n');
    console.log(`FSRS ${result.version}: ${result.reason}`);
  } finally {
    const absolute = resolve(folder);
    if (
      !absolute.startsWith(resolve(tmpdir()) + sep) ||
      !absolute.split(sep).at(-1).startsWith('openrecall-fsrs-probe-')
    )
      throw new Error('Refusing unsafe probe cleanup');
    await rm(absolute, { recursive: true, force: true });
  }
}
async function notify(input) {
  const result = JSON.parse(await readFile(input, 'utf8'));
  if (result.actionable !== true || result.reason !== 'compatible') return;
  if (!stable.test(result.version)) throw new Error('Invalid candidate version');
  await notifyOnce({
    marker: `<!-- openrecall-fsrs-release:${result.version} -->`,
    label: 'fsrs-release',
    title: `Published FSRS ${result.version} passed FSRS-7 compatibility checks`,
    body: `The published crate passed the current FSRS-7 state/intraday numerical goldens and real optimizer tests in the isolated probe job.\n\nOpen a separately reviewed migration PR replacing the pinned Git source with fsrs = "=${result.version}". Keep the 34-parameter model, stability_fast, intraday scheduling, training gates and numerical goldens. Re-run the full quality/native/release checks. This migration is deliberately excluded from automatic merging.\n\n[Published crate](https://crates.io/crates/fsrs/${result.version})`,
  });
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [mode, path] = process.argv.slice(2);
  if (!path) throw new Error('Usage: check-fsrs-release.mjs probe|notify RESULT.json');
  if (mode === 'probe') await probe(path);
  else if (mode === 'notify') await notify(path);
  else throw new Error('Unknown watcher mode');
}
