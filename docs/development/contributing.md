# Contributing

Development code, identifiers and technical documentation use English. Product
controls, feedback and validation messages use Arabic. Preserve RTL navigation,
multilingual card content and the keyboard/screen-reader semantic contract.

Install the pinned versions in `rust-toolchain.toml`, `package.json` and the
lockfiles, along with the native Tauri prerequisites. `pnpm tauri dev` runs the
real desktop backend. `pnpm dev` alone serves frontend assets and requires a test
adapter for browser-only interaction. It is not an alternate production app.

Use a focused branch and meaningful behavioral tests before implementation.
Run `pnpm verify` and the affected browser/native QA before requesting review.
Regenerate contracts with `pnpm contracts`; commit generated declarations only
when the Rust contract intentionally changes. Do not write a JavaScript scheduler
or substitute a different model because a published crate number looks newer.

Scheduling/profile changes require numeric FSRS-7 goldens, real optimizer tests,
stale-candidate/revision cases, intraday behavior and due-time preservation checks.
Import/backup changes require invalid, duplicate, stale, future-version and disk-
failure cases that preserve the original library. Update changes require official
signature/version verification and native lifecycle guards.

Tests must use temporary libraries. Keep study records, SQLite files, credentials,
keys, build outputs and screenshots containing personal data out of Git. Signing
keys belong outside the checkout and in repository secrets, never fixtures.
Public test signatures contain only inert text and a public key.

The agent must never operate a screen reader. Use DOM, keyboard, native focus and
axe automation; the human partner owns NVDA testing. State which OS actually ran
the app, which platforms only built, and whether a real update was performed.

Dependabot development-tool patch/minor PRs can merge only through the checked
policy and exact-head required checks. Runtime/Rust/Tauri/FSRS/workflow changes
receive human review. FSRS release monitoring produces a migration issue only
after real compatibility tests; it does not change the source pin itself.

Release from an existing tag whose commit is on main and has every required check
green. Keep Cargo, npm and Tauri app versions synchronized. The release pipeline
requires signing secrets and all four updater targets plus offline Windows media.
It publishes a complete verified draft and refuses to overwrite a public release.
See `docs/releases/signing.md` for credential custody and key rotation.
