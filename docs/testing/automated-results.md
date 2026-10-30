# Automated verification

The release gate is `pnpm verify`, followed by `pnpm build:native-test` and `pnpm test:native` on Windows. The final GitHub Quality run must pass on the exact reviewed revision. Release preflight additionally requires all six checks on the tagged main commit.

The v0.1.0 release gate covered 71 Rust workspace tests, the additional seven backup feature-compatibility cases, 53 frontend unit tests, 16 maintenance tests, seven browser scenarios, five native study-flow cases and two native startup-recovery probes. Its browser and native study suites audited 25 and 11 UI states respectively and require zero detected axe violations. Every audit requires a nonzero number of passing rules.

## Environments and test boundaries

- Primary local runtime: Windows 11 Pro x64, WebView2 155, Rust 1.99, Node 24.21 and pnpm 12.15.
- Browser presentation tests: Playwright Chromium, a contracted mock API adapter, the real React components and bundled styles/fonts. The fixture has its own HTML build and static server; it is absent from the production entry. These tests establish UI interaction and automated accessibility, not native scheduling or persistence.
- Native runtime tests: WebdriverIO 9.32 with the compatible Tauri service/plugin 1.5.1 and embedded WebDriver. The optional `automation` Cargo feature, automation identifier, frontend mode, capabilities, test data directory and picker/Downloads overrides are excluded from production builds. Native commands still use the real Rust core, SQLite, FSRS and file validation. Only the OS file-choice boundary is selected by fixture environment variables.
- The runner starts a rejecting local HTTP/CONNECT proxy for the application and WebView2. It never forwards traffic. The complete suite requires a real `github.com:443` updater connection to reach that proxy; the request and rejection are recorded in `network-boundary.json`. Local driver/IPC traffic is allowed. No external library or study service is required.
- Persistence uses two separate desktop launches against the same temporary SQLite library. The runner verifies the first app process has exited before launching the review suite.

The automation executable is copied to its own path after a successful feature-specific build and a compiled-marker assertion. Workspace tests cannot replace it with an ordinary Cargo binary. The native provider's select-option default action is unavailable; that test dispatches a real DOM `change` event before the actual Rust profile operations. No backend command is mocked in the native suite.

## Coverage and evidence

The Rust suite covers domain validation, Cairo study-day/DST boundaries, SQLite migration/transactions, immutable profiles, pinned FSRS-7 inference/optimizer goldens, shuffled form cycles and reservations, live due order/filters, retry-safe atomic grading and undo, chronological training holdout and stale candidates, statistics, and complete backup validation/rollback. The seven backup cases also run with `serde_json/preserve_order` to verify portable canonical digests across dependency feature combinations.

Frontend unit tests cover editing, import exclusion, keyboard/focus, durable review recovery, advanced settings/profile guards, restore notices across navigation and updater event ordering. Maintenance tests cover verified bot identity, exact-head required checks, dependency scope restrictions, compatible FSRS registry detection, release manifests/assets/preflight, literal shell-free pnpm arguments and platform icon configuration.

Seven Playwright scenarios audit 25 named UI states: library/create/delete dialogs; collapsed deck/statistics/card/variant editors; invalid/duplicate import preview; question/revealed answer/empty filter/summary; collapsed/expanded/advanced profiles and insufficient-data gate; settings/export/restore preview/completion; system information/updater states/validation error. Every axe call asserts a nonzero number of rules passed and zero detected violations. Incomplete axe checks are recorded and do not count as human review.

The native suite exercises real create/import/duplicate exclusion, process restart, mixed-language cards, all/new/previous filters, due arrival while another question stays reserved, nonrepeating alternative forms, interval descriptions, summary undo, card edit/delete, profile assignment/detachment/training gate, portable export/delete/restore with safety backup, failed offline updater check, keyboard dialog return and Windows native blur/refocus. Native axe reports are retained under `target/native-tests/run-*/audits`.

Meaningful regressions were observed failing before repair: restore feedback lost on route unmount; accessible names differing from visible button text; stale updater status overwriting a newer event; feature-dependent backup digests; corrupt backup timestamps/undo snapshots; POSIX native pnpm detection; missing AppImage icon configuration.

The v0.1.0 independent GPT-6.1 Sol review identified six Important findings. The retained regression suite covers deleted form references in ended/parked sessions and undo before-images; drafts surviving cross-profile membership changes; late review-start responses preserving later drafts; old training loads preserving the newer job and its cancellation control; backup memory satisfying the same FSRS invariant used during review; and Arabic native recovery when a future-schema or inaccessible library cannot open. The startup probes inspect only the test process's own native window captions, verify a retry control and retained file digest, and never operate a screen reader. They do not initialize a substitute empty study library.

Two Minor review items remain deferred: adding start/end timestamps, final filter and an explicit observed-recall definition to the session summary; and expanding Arabic Log loss/RMSE and pooled-versus-per-deck improvement explanations. Existing durations/counts/ratings and per-deck metric tables remain available.

The live release API exposed a further integration case after merge: the release-by-tag endpoint returned 404 for an owned draft visible in the authenticated release list. Publisher transport regressions were observed RED then GREEN for draft recovery and refusing ambiguous drafts without mutation. A manual retry uses publisher policy from its trusted main workflow revision while build jobs retain the original verified tag. No application code or released tag is changed by that policy repair.

## Spelling feature acceptance on 2026-10-30

The feature branch passed 106 core tests, 82 frontend unit tests and nine browser
scenarios before release packaging. The full `pnpm verify` gate passed 115 Rust workspace tests, seven additional
backup preserve-order cases, 82 frontend unit tests, 16 maintenance tests and
nine browser scenarios. Its 36 browser axe audits have zero violations and
at least 14 passing rules each; duplicated attachment copies are not counted.
Rust formatting/Clippy, generated-contract drift, TypeScript, ESLint, Prettier,
production notices, frontend build and workflow lint all passed. The genuine v0.1.0 export fixture is
frozen with its original digest; compatibility tests verify that checksum before
adding current defaults, including nested undo before-images.

The real Windows suite passed seven study-flow cases and two native startup
recovery probes on Windows 11 Pro 10.0.26300 x64, WebView2 156 and Node 24.21.0.
The copied automation executable uses an isolated library and Downloads directory.
Its 20 axe reports have zero violations and at least 20 passing rules each. Native
artifacts for this acceptance run are under
`target/native-tests/run-JINEkt`; the startup probes are under
`target/startup-tests/run-iS8u7X` and `run-n0cQzu`.

Spelling acceptance exercises real JSON with custom headings and legacy omitted
headings, singleton backfill, an arriving word while another stays reserved,
source editing/deletion with retained context, direct grading, six daily
completions, summary undo, full format-2 export/delete/restore, genuine format-1
restore with usable undo, explicit exclusion/rescan/reallow and fresh progress.
The test closes the actual process, verifies it exited and restarts the same
SQLite library. The presentation survives; the textarea contains only its target
and newline, focused at collapsed caret zero. Distinct synthetic attempts are
absent from exported JSON and SQLite/main-WAL bytes. No real study data is used.

The additional expected-calendar-date and delay-caveat announcement was removed
from all rating buttons at the user's request. Approximate interval names and
actual committed due feedback remain. First-line prefill/focus and removal of the
extra accessible description each have an observed failing-then-passing regression.

Two independent GPT-6.1 Sol whole-branch reviews were completed, the second at
Max effort at the owner's explicit request. Neither found a Critical issue.
The first Important finding concerned review creation before resolving an unsaved
source selection; three App regressions were observed failing, then passing after
blocking conflicting native actions through source saving and dependent refreshes.
The second Important finding concerned spelling collection remounting scheduling
settings and discarding an unsaved policy. Four additional cases failed before
coordinating the two settings sections, then all seven action-guard cases passed.
They cover existing scheduling drafts and new policy editing during pending source
save, rescan and reallow. Source drafts also retain the navigation confirmation.

After both Important fixes, `pnpm build:native-test` and `pnpm test:native`
passed all seven native study cases and both startup-recovery probes again.
The fresh isolated root is `target/native-tests/run-E5qrZb`, with startup roots
`run-SmGhfC` and `run-oQrM2v`. All 22 native axe reports have zero violations and
at least 20 passing rules. The added real-native checks assert that a policy
draft blocks source mutations, a source draft blocks policy editing/review, and
cancelling navigation preserves the draft and native sessions/library revision.
The fresh post-fix `pnpm verify` run passed 115 Rust workspace tests, seven
additional preserve-order backup cases, 89 frontend tests, 16 maintenance cases
and nine browser scenarios. All 36 original browser axe reports have zero
violations and at least 14 passing rules. Formatting, Clippy, contracts, types,
lint, notices, production frontend build and workflow lint passed as well.

One new Minor remains deferred: retained original-card capture timestamps use
Cairo rather than the selected timezone. This affects explanatory metadata; the
native scheduler and completion calendar use the selected timezone. The two
v0.1.0 Minor items described above remain deferred as well.

A supplementary agent-browser presentation check reported 37 passing rules,
zero violations and one incomplete decorative-mark color-contrast check. The
versioned project acceptance uses axe 4.14 rather than that tool's bundled 4.12.1.
These are automated accessibility results; no screen reader was run.

## 0.2.0 production packaging preflight

The three product version sources and two workspace lockfile entries agree on
0.2.0. The third-party graph remains 430 Rust crates, five npm packages and five
MPL source archives; notices were regenerated for the new lockfile hash.
`pnpm verify` also passed after the version bump with the same 115 Rust workspace,
seven extra backup, 82 frontend, 16 maintenance and nine browser case counts.

The production Windows x64 binary was compiled without signing credentials or
the automation feature. All four native test-override strings are absent before
and after bundling. Official Tauri bundling produced a 5,431,735-byte normal NSIS
installer and a 223,217,644-byte WebView2-inclusive offline installer from that
existing production build. Both passed Tauri's official signature and signed
version verification for 0.2.0. Credentials were loaded only during bundling and
removed in `finally`; no new secret or key was created. Local candidates and
hash evidence are preserved outside this worktree under the primary checkout's
`target/delivery/v0.2.0/local-preflight` directory.

The local feature-flow acceptance above used the automation build before the
routine product-version bump. Production packaging was built and cryptographically
verified without launching it against a real study library. These preflight
artifacts are distinct from the final immutable-tag CI release artifacts.
The local preflight candidates also precede the review guard repairs and must
not be substituted for those final checked-tag artifacts.

## Limits

No screen reader was launched, operated or configured. [Human NVDA checks](human-nvda-checklist.md) remain for the owner. The native focus test hides, shows and refocuses only its own test window and verifies real blur/focus events and retained DOM focus. Global Alt+Tab was not sent to unrelated user applications; the Alt+Tab-specific check remains in the human checklist.

Windows 10 and macOS/Linux runtime behavior have not been exercised on those operating systems here. CI builds Windows x64, Linux x64 and both macOS architectures. Those are build results, not runtime certifications.

No disposable older-version installation was available for a complete version-to-version update or OS installer execution. The updater integration is tested with injected failure/success transports, official version-bound signature verification, guarded restart/persistence, complete multi-platform manifests and release artifact digests. An updater signature does not supply a Windows publisher certificate or macOS notarization; macOS packages use ad-hoc signing.

Linux retains the documented upstream [GLib iterator advisory](../releases/linux-glib-advisory.md). Windows and macOS target graphs omit the affected crate; the dependency alert remains open.
