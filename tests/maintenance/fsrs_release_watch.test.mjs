import { test } from 'node:test';
import assert from 'node:assert/strict';
import { inspectFsrsRelease, parseEnginePin } from '../../scripts/check-fsrs-release.mjs';
const source =
  'pub stability_fast: f32; fn next_states_with_elapsed_days() {} fn compute_parameters() {} enable_short_term schedule_penalties';
const goldens = ['fsrs7State', 'intraday', 'optimizer'].map((name) => ({ name, passed: true }));
const metadata = { version: '7.0.0', currentVersion: '6.6.2', yanked: false, seenVersions: [] };
test('watch baseline follows Cargo.lock and becomes quiet after a registry migration', () => {
  const lock =
    '[[package]]\nname = "other"\nversion = "1.0.0"\n[[package]]\nname = "fsrs"\nversion = "6.6.2"\nsource = "git+https://example.invalid"';
  assert.deepEqual(parseEnginePin('fsrs = { git = "https://example.invalid", rev = "a" }', lock), {
    currentVersion: '6.6.2',
    gitPinned: true,
  });
  assert.deepEqual(parseEnginePin('fsrs = "7.0.0"', lock.replace('6.6.2', '7.0.0')), {
    currentVersion: '7.0.0',
    gitPinned: false,
  });
  assert.throws(() => parseEnginePin('fsrs = "7.0.0"', ''));
});
test('a numerically higher FSRS-6-only crate or absent/failing goldens is not a migration signal', () => {
  assert.equal(
    inspectFsrsRelease({ ...metadata, version: '9.0.0' }, 'pub stability: f32;', goldens)
      .actionable,
    false,
  );
  for (const results of [
    [],
    goldens.slice(0, 2),
    goldens.map((r, i) => (i ? r : { ...r, passed: false })),
  ])
    assert.equal(inspectFsrsRelease(metadata, source, results).actionable, false);
});
test('required FSRS-7 state, intraday API and real optimizer goldens produce a deduplicated signal', () => {
  const first = inspectFsrsRelease(metadata, source, goldens);
  assert.equal(first.actionable, true);
  assert.equal(first.version, '7.0.0');
  assert.match(first.marker, /openrecall-fsrs-release:7\.0\.0/);
  assert.equal(
    inspectFsrsRelease({ ...metadata, seenVersions: ['7.0.0'] }, source, goldens).actionable,
    false,
  );
  for (const altered of [
    { version: '6.6.2' },
    { version: '5.0.0' },
    { version: '7.0.0-rc.1' },
    { yanked: true },
  ])
    assert.equal(
      inspectFsrsRelease({ ...metadata, ...altered }, source, goldens).actionable,
      false,
    );
});
