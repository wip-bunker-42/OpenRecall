# Automated verification

The release gate is `pnpm verify`, followed by `pnpm build:native-test` and `pnpm test:native` on Windows. The final GitHub Quality run must pass on the exact reviewed revision. Release preflight additionally requires all six checks on the tagged main commit.

The final review fix suite includes 71 Rust workspace tests, the additional seven backup feature-compatibility cases, 53 frontend unit tests, 16 maintenance tests, seven browser scenarios, five native study-flow cases and two native startup-recovery probes. The browser and native study suites audit 25 and 11 UI states respectively and require zero detected axe violations. Every audit requires a nonzero number of passing rules.

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

The final independent GPT-6.1 Sol review identified six Important findings. The retained regression suite covers deleted form references in ended/parked sessions and undo before-images; drafts surviving cross-profile membership changes; late review-start responses preserving later drafts; old training loads preserving the newer job and its cancellation control; backup memory satisfying the same FSRS invariant used during review; and Arabic native recovery when a future-schema or inaccessible library cannot open. The startup probes inspect only the test process's own native window captions, verify a retry control and retained file digest, and never operate a screen reader. They do not initialize a substitute empty study library.

Two Minor review items remain deferred: adding start/end timestamps, final filter and an explicit observed-recall definition to the session summary; and expanding Arabic Log loss/RMSE and pooled-versus-per-deck improvement explanations. Existing durations/counts/ratings and per-deck metric tables remain available.

The live release API exposed a further integration case after merge: the release-by-tag endpoint returned 404 for an owned draft visible in the authenticated release list. Publisher transport regressions were observed RED then GREEN for draft recovery and refusing ambiguous drafts without mutation. A manual retry uses publisher policy from its trusted main workflow revision while build jobs retain the original verified tag. No application code or released tag is changed by that policy repair.

## Limits

No screen reader was launched, operated or configured. [Human NVDA checks](human-nvda-checklist.md) remain for the owner. The native focus test hides, shows and refocuses only its own test window and verifies real blur/focus events and retained DOM focus. Global Alt+Tab was not sent to unrelated user applications; the Alt+Tab-specific check remains in the human checklist.

Windows 10 and macOS/Linux runtime behavior have not been exercised on those operating systems here. CI builds Windows x64, Linux x64 and both macOS architectures. Those are build results, not runtime certifications.

No disposable older-version installation was available for a complete version-to-version update or OS installer execution. The updater integration is tested with injected failure/success transports, official version-bound signature verification, guarded restart/persistence, complete multi-platform manifests and release artifact digests. An updater signature does not supply a Windows publisher certificate or macOS notarization; macOS packages use ad-hoc signing.

Linux retains the documented upstream [GLib iterator advisory](../releases/linux-glib-advisory.md). Windows and macOS target graphs omit the affected crate; the dependency alert remains open.
