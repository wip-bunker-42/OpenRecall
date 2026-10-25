import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
test('desktop bundles explicitly include square PNG, Windows ICO and macOS ICNS icons', () => {
  const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  const icons = config.bundle.icon ?? [];
  assert.ok(icons.some(path => path.endsWith('.png')), 'AppImage requires an explicitly configured square PNG');
  assert.ok(icons.some(path => path.endsWith('.ico')));
  assert.ok(icons.some(path => path.endsWith('.icns')));
  for (const path of icons.filter(path => path.endsWith('.png'))) {
    const bytes = readFileSync(resolve('src-tauri', path));
    assert.equal(bytes.readUInt32BE(16), bytes.readUInt32BE(20));
  }
});
