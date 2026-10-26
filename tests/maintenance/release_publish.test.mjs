import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join, sep } from 'node:path';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { createManifest } from '../../scripts/release.mjs';

async function runPublisher(t, ambiguous = false) {
  const folder = await mkdtemp(join(tmpdir(), 'openrecall-publish-test-'));
  t.after(async () => {
    assert.ok(resolve(folder).startsWith(resolve(tmpdir()) + sep));
    await rm(folder, { recursive: true, force: true });
  });
  const assets = join(folder, 'assets');
  await mkdir(assets);
  const sha = 'a'.repeat(40);
  const draft = {
    id: 42,
    tag_name: 'v0.1.0',
    draft: true,
    body: `<!-- openrecall-release:v0.1.0:${sha} -->`,
  };
  const manifest = createManifest({
    version: '0.1.0',
    notes: 'Synthetic publisher transport test; assembly verifies real signatures separately.',
    publishedAt: '2026-10-26T00:00:00Z',
    artifacts: ['windows-x86_64', 'linux-x86_64', 'darwin-aarch64', 'darwin-x86_64'].map(
      (platform) => ({
        platform,
        url: `https://github.com/wip-bunker-42/OpenRecall/releases/download/v0.1.0/${platform}`,
        signature: Buffer.from('synthetic transport fixture').toString('base64'),
      }),
    ),
  });
  await writeFile(join(assets, 'latest.json'), JSON.stringify(manifest));
  await writeFile(join(assets, 'package.bin'), 'synthetic package bytes');
  const callsPath = join(folder, 'calls.json');
  const hook = join(folder, 'transport.mjs');
  await writeFile(
    hook,
    `import {writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const draft=${JSON.stringify(draft)}, drafts=${JSON.stringify(ambiguous ? [draft, { ...draft, id: 43 }] : [draft])};
const calls=[], assets=[];
globalThis.fetch=async (input, options={})=>{
  const url=new URL(input), method=options.method??'GET';
  calls.push({method,path:url.pathname});
  writeFileSync(${JSON.stringify(callsPath)},JSON.stringify(calls));
  const response=(data,status=200)=>new Response(JSON.stringify(data),{status});
  if(method==='GET'&&url.pathname.endsWith('/releases/tags/v0.1.0')) return response({},404);
  if(method==='GET'&&url.pathname.endsWith('/releases')) return response(drafts);
  if(method==='GET'&&url.pathname.endsWith('/releases/42/assets')) return response(assets);
  if(method==='POST'&&url.hostname==='uploads.github.com'&&url.pathname.endsWith('/releases/42/assets')){
    const asset={id:assets.length+1,name:url.searchParams.get('name'),digest:'sha256:'+createHash('sha256').update(options.body).digest('hex')};
    assets.push(asset);return response(asset,201);
  }
  if(method==='PATCH'&&url.pathname.endsWith('/releases/42')) return response({...draft,draft:false,html_url:'https://github.com/wip-bunker-42/OpenRecall/releases/tag/v0.1.0'});
  return response({message:'Unexpected or unsafe mutation'},409);
};`,
  );
  const result = spawnSync(
    process.execPath,
    ['--import', pathToFileURL(hook).href, resolve('scripts/publish-release.mjs'), assets, '0.1.0'],
    { encoding: 'utf8', env: { ...process.env, GH_TOKEN: 'synthetic-token', RELEASE_SHA: sha } },
  );
  return { result, calls: JSON.parse(await readFile(callsPath, 'utf8')) };
}

test('publisher resumes its owned draft when the tag endpoint only exposes published releases', async (t) => {
  const { result, calls } = await runPublisher(t);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Published complete verified release/);
  assert.equal(
    calls.some((call) => call.method === 'POST' && call.path.endsWith('/releases')),
    false,
  );
  assert.ok(calls.some((call) => call.method === 'PATCH' && call.path.endsWith('/releases/42')));
});

test('publisher refuses ambiguous drafts before uploading, deleting or publishing anything', async (t) => {
  const { result, calls } = await runPublisher(t, true);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /ambiguous/i);
  assert.equal(
    calls.some((call) => call.method !== 'GET'),
    false,
  );
});

test('release dispatch runs publisher policy from the trusted workflow revision', async () => {
  const workflow = await readFile('.github/workflows/release.yml', 'utf8');
  const publisher = workflow.split('\n  publish:\n')[1].split('\n  notify-failure:')[0];
  assert.match(publisher, /ref: \$\{\{ github\.sha \}\}/);
});
