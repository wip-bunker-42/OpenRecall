# OpenRecall Desktop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver an offline Arabic, accessible Tauri desktop spaced repetition app with genuine FSRS-7, randomized alternative forms, filtered reviews, reusable optimization profiles, full backups, signed updates, and a public GitHub repository.

**Architecture:** A Rust workspace contains a SQLite-backed domain core and a thin Tauri shell. React and TypeScript render semantic HTML and invoke typed Rust commands; the frontend never implements a second scheduler. Rust DTOs generate TypeScript declarations, and deterministic clocks/randomness support meaningful behavioral tests.

**Tech Stack:** Tauri 2, Rust, rusqlite with bundled SQLite, the pinned upstream fsrs crate, chrono/chrono-tz, serde, ts-rs, React, TypeScript, Vite, pnpm, Vitest/Testing Library, Playwright/axe, WebdriverIO for native automation, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-24-openrecall-design.md` (approved 2026-10-24).

## Global Constraints

- Product language: Arabic. Development language: English.
- Windows 11 is the primary platform; Windows 10, macOS, and Linux are secondary.
- Intended public repository: `wip-bunker-42/OpenRecall`.
- Pin FSRS to `0fd59949e9168af7194418c2dba99836b5548e87` until a separately verified published release replaces it.
- Preserve `stability`, `stability_fast`, and `difficulty`; use `next_states_with_elapsed_days` with UTC elapsed seconds / 86400.
- Default desired retention is 0.90; editable 0.70-0.99. Default minimum interval is 60 seconds (editable 10-3600); default maximum is 36500 days (editable 1-36500).
- Study timezone: Africa/Cairo. Start of study day: 01:00 local time; editable 00:00-23:59.
- Set `lang="ar"` and `dir="rtl"`; card content uses `dir="auto"` and plain text.
- Review headings are the labels `السؤال` (H1), `الإجابة` (H2), and `الملاحظات` (H3 when notes exist).
- Ratings: `نسيت`, `افتكرتها بصعوبة`, `افتكرتها بشكل طبيعي`, `افتكرتها بسهولة`; names include approximate next intervals.
- Every new session starts on all; filters are all/new/previously-reviewed and use live committed rating state.
- Forms use a persistent shuffle bag without replacement per cycle; avoid immediate cycle-boundary repeats with two or more forms.
- Training is scoped to explicit optimization-profile membership, never automatically to the entire user library. Schedule penalties default to on.
- Gate training on 400 eligible nonfirst observations, 50 cards, seven study-date keys, five Again and five recalled outcomes, 64 training prefixes, and 80 held-out observations from 10 cards.
- Accept a candidate only when held-out log loss improves by at least 0.0001 and RMSE worsens by no more than 0.0001.
- Card JSON import limit starts at 50 MiB. Full backup is a separate, versioned format.
- Agent accessibility testing is automated only; do not launch, control, reconfigure, or use NVDA, Narrator, or another screen reader. Human NVDA testing is not a prerequisite to source/installer delivery.
- Ordinary study, CRUD, import, export, statistics, and training work offline. Only explicit app-update operations use network access at runtime.
- Commit lockfiles, keep signing credentials outside Git, and report actual verification evidence without claiming unperformed human or platform testing.

## Review Focus

1. A rating request is retried after a timeout or after undo: never double-rate or resurrect the undone rating (Task 8).
2. Wall clock moves backward or the computer sleeps: never produce negative FSRS deltas or count sleep as active study time (Tasks 2 and 7).
3. Form edits and filter switches occur at a shuffle-cycle boundary: retain the reserved form and prevent accidental bag consumption (Tasks 6 and 17).
4. Shared-profile membership changes after training: reject a stale candidate and preserve unaffected decks and due times (Tasks 10 and 12).
5. Disk failure or future-format backup during restore: retain the original library and avoid a false success result (Tasks 14 and 19).

## Execution and Review Contract

This is one integrated application, implemented as independently testable increments. Follow tasks in order; backend interfaces precede frontend consumers. Before product code, invoke `superpowers:using-git-worktrees` and use its isolation procedure. The approved design and this plan must be available in that checkout. Do not create a new user chat for subtasks.

Each task follows RED -> confirm the intended failure -> minimal implementation -> GREEN -> review -> commit. Write behavioral tests before feature bodies. A missing test runner is setup work, not proof of an intended failing behavior. A task's listed tests are a minimum, not permission to omit an acceptance criterion. If a real failure appears, invoke `superpowers:systematic-debugging`. Re-read the relevant spec section before implementing each task.

All `Create`/`Modify`/`Test` paths below are repository-relative. Runtime data and credentials must live outside the checkout. No product code or dependencies are installed during planning. At execution time check current compatible stable releases, their engine/peer constraints and release notes, then record resolved versions in `docs/development/toolchain.md`. Research on 2026-10-24 found Node 24 compatible with current Vite/Vitest, Tauri CLI/API 2.12.4, React 19.3.1, Vite 8.3.5, Vitest 5.0.5, TypeScript 7.0.2, pnpm 12.15.0, and ts-rs 12.0.1; these are candidates to verify together, not instructions to ignore a conflict. Rust/Cargo 1.99.0 is installed without `rustup` on PATH. Install/update suitable tooling where needed instead of excluding a language or framework.

## File Structure and Boundaries

```text
Cargo.toml, Cargo.lock                       Rust workspace
crates/openrecall-core/src/
  lib.rs, app.rs, error.rs, runtime.rs        facade, transactions, errors, clock/RNG
  contracts/{mod,settings,library,cards,
    review,import,profiles,training,
    statistics,backup,system,updates}.rs     serializable command/native DTOs
  domain/{settings,time,card,profile,
    session}.rs                             validated domain rules
  storage/{mod,migrations,library,cards,
    sessions,profiles,statistics,backup}.rs  focused SQL access
  storage/migrations/*.sql                  incremental schema migrations
  scheduling/{fsrs,forms,queue}.rs           real engine, shuffle bags, eligibility
  services/{settings,library,cards,sessions,
    import,profiles,training,statistics,
    backup}.rs                              transactional operations
  training/{dataset,eligibility,evaluate,
    runner}.rs                              offline optimizer pipeline
  backup/{format,validation,files}.rs        portable snapshot and atomic I/O
crates/openrecall-core/examples/
  export_contracts.rs                        deterministic ts-rs export
crates/openrecall-core/tests/
  common/mod.rs, fixtures/*                  temporary DB, fixed clock, seeded RNG
  <task-specific integration tests>.rs      observable behavior
src-tauri/{Cargo.toml,build.rs,tauri.conf.json}
src-tauri/src/{lib,main,commands,paths,
  jobs,files,updates}.rs                     narrow native integration
src-tauri/capabilities/default.json          minimal permissions
src/{main.tsx,app/App.tsx,app/navigation.ts}
src/shared/{api,errors,format,focus}.ts       typed adapter and presentation helpers
src/shared/generated/*.ts                   Rust-generated DTO declarations
src/components/{Dialog,Disclosure,
  FormErrors,StatusRegion}.tsx              semantic shared components
src/features/{library,cards,import,review,
  settings,profiles,statistics,backup,
  updates}/*                                focused feature UI and tests
src/styles/{tokens,global}.css               dark RTL, focus, zoom and motion
tests/e2e/*.spec.ts                          browser keyboard and axe checks
tests/native/*.spec.ts                      real Tauri persistence and focus
scripts/{verify,release,check-fsrs-release}.* repeatable checks and maintenance
.github/workflows/*.yml                     quality, release, dependency automation
docs/{development,testing,releases}/*        verified instructions and limitations
README.md, LICENSE, THIRD_PARTY_NOTICES.md
```

### Shared interfaces (define these in Task 1; extend feature DTOs in their task)

- `Id = String` containing a validated UUID; DTO timestamps are RFC3339 UTC strings. Revisions/action/presentation IDs are strings, never unsafe JavaScript 64-bit numbers.
- `type AppResult<T> = Result<T, AppError>`; `AppError { code: ErrorCode, fields: Vec<FieldError>, context: Option<String> }`. Error codes are stable English identifiers; frontend maps them to Arabic. Never return raw SQL or filesystem diagnostics containing credentials.
- `trait Clock: Send + Sync { fn now(&self) -> DateTime<Utc>; fn monotonic_ms(&self) -> u64; }`.
- `trait RandomSource: Send { fn shuffle(&mut self, ids: &mut [Id]); }`.
- `App::open(path: &Path, clock: Arc<dyn Clock>, random: Box<dyn RandomSource>) -> AppResult<App>`; serialized operations use `&mut self`.
- `App::execute(&mut self, request: Request) -> AppResult<Response>`. `Request` and `Response` use `#[serde(tag="kind", content="payload", rename_all="camelCase")]`; every matching operation has the same `kind` on input and output. Each task owns its new variants and payload DTOs. Empty inputs are `{}`.
- `contracts::export(output: &Path) -> AppResult<()>` exports Request, Response, AppError and all dependencies using ts-rs. Frontend imports generated types, not copies of Rust interfaces.
- Core domain commands use Request/Response. Native-only file, system-info and update commands use NativeApi and exported DTOs without adding platform-dependent operations to App::execute.
- DTO structs also use `#[serde(rename_all="camelCase")]`; the enum attribute alone does not rename nested fields. Wire revisions are opaque strings. Rust settings fields are `timezone`, `day_start_minute`, `desired_retention`, `min_interval_seconds`, `max_interval_days`, `shortcuts_enabled`, `text_scale`, `reduced_motion`, `page_size`, and `revision`. Settings defaults are Cairo/60/0.90/60/36500/true/1.0/followSystem/20; `day_start_minute = 60` means 01:00. Text scale is validated 0.8-2.0, and page size is 10/20/50/100.
- Configure ts-rs integer exports as numbers only for validated safe counters/durations, never for IDs/revisions/timestamps. Their DTO numeric values must stay within JavaScript's safe integer range. Assertion blocks below are excerpts inside the named tests; their local setup and observations use the interfaces specified in that task.
- Frontend `Api.call<K extends Request['kind']>(kind: K, payload: PayloadOf<K>): Promise<ResultOf<K>>`, where payload/result types are extracted by that same `kind`.
- Test-only `Fixture::new() -> Fixture` owns a temporary database, fixed UTC/monotonic clock, seeded RNG, and `app: App`; helpers `deck(name) -> Id`, `card(deck_id, front, back) -> Id`, `advance(seconds)`, `reopen()` create real data through implemented operations. Add helpers only when their owner operation exists. Feature test modules define local setup where necessary.

Create frontend script names in Task 15 so every later verification command is unambiguous: `contracts` runs the Rust export example with `src/shared/generated` as destination; `typecheck` runs `tsc --noEmit`; `lint` runs ESLint; `test` runs `vitest run`; `build` typechecks and invokes `vite build`; `tauri` invokes the local Tauri CLI. Add `test:e2e` = `playwright test`, `test:native` = `wdio run wdio.conf.ts`, and `verify` = `node scripts/verify.mjs` when their owning tasks arrive. `openrecall-desktop` is the Tauri Cargo package name; the initial version is 0.1.0, product name OpenRecall, application identifier `org.wipbunker42.openrecall`. Resolve its database as `openrecall.sqlite3` inside the OS app-data directory, never the repository.

---

### Task 1: Transactional storage and bootstrapped settings

**Files:** Create `Cargo.toml`, `.cargo/config.toml`, `crates/openrecall-core/Cargo.toml`, `src/{lib,app,error,runtime}.rs` under that crate, `src/contracts/{mod,settings}.rs`, `src/domain/{mod,settings}.rs`, `src/storage/{mod,migrations}.rs`, `src/storage/migrations/001_settings.sql`, `examples/export_contracts.rs`, `.gitignore`, `.gitattributes`, `docs/development/toolchain.md`; Test `tests/{bootstrap,contracts}.rs`, `tests/common/mod.rs` under that crate.

**Interfaces:** Produces the shared interfaces above; `BootstrapInput {}` -> `Bootstrap { settings: Settings, schema_version: u32 }`. Later tasks extend Bootstrap with library/profile/resume summaries. `Settings::default()` returns the approved defaults.

- [ ] **Step 1:** Verify compiler/linker, rustfmt/clippy availability and dependency compatibility; create only the workspace/test-host manifests needed to run the following tests. Pin compatible versions and document any tooling installation.
- [ ] **Step 2:** Write `bootstrap_persists_defaults_and_rejects_future_schema` and `contracts_round_trip_tagged_messages`: assert Cairo/01:00/0.90/60/36500 defaults, persisted reopen equality, foreign keys on, transaction rollback, and a structured error for a future schema. Assert Bootstrap JSON has `kind: "bootstrap"`, generated DTOs use strings for IDs/timestamps, and invalid wire inputs fail.
```rust
// bootstrap_persists_defaults_and_rejects_future_schema
assert_eq!(boot.settings.timezone, "Africa/Cairo");
assert_eq!(boot.settings.day_start_minute, 60);
assert_eq!(after_failed_transaction, before_failed_transaction);
```

- [ ] **Step 3:** Run `cargo test -p openrecall-core --test bootstrap --test contracts`; confirm failure in the unimplemented bootstrap/validation, not a missing build tool.
- [ ] **Step 4:** Implement App opening/migration/transaction/error/export contracts, with atomic schema version changes, WAL and busy timeout. Re-run the command; expect both test binaries PASS. Run `cargo fmt --all -- --check`.
- [ ] **Step 5:** Commit the tested storage/contract foundation: `feat: initialize transactional core and typed contracts`.

### Task 2: Validated settings and Cairo study dates

**Files:** Create `crates/openrecall-core/src/domain/time.rs`, `src/services/{mod,settings}.rs`; Modify domain/settings, settings contracts, App dispatch and settings storage; Test `crates/openrecall-core/tests/settings_time.rs`.

**Interfaces:** `SaveSettings { settings: Settings, expected_revision: String }` -> `Settings`; `study_date(at: DateTime<Utc>, timezone: Tz, start_minute: u16) -> NaiveDate`; `elapsed_days(previous: DateTime<Utc>, now: DateTime<Utc>) -> f32`. Include all normal/advanced validation ranges from spec section 9; profile-specific fields are introduced in Task 3.

- [ ] **Step 1:** Write tests asserting 00:59 and 01:00 Cairo instants map to different study days; use actual chrono-tz transition data to cover both DST changes. Assert global limits, timezone validation, settings conflict handling, and that backward clock movement gives a zero rather than negative elapsed delta. Record a clock-anomaly diagnostic without corrupting data.
```rust
// cairo_day_boundary_and_backward_clock
assert_ne!(before_01_study_date, at_01_study_date);
assert_eq!(elapsed_days(later_instant, earlier_instant), 0.0);
assert_eq!(due_before_timezone_change, due_after_timezone_change);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test settings_time`; expect intended validation/date failures.
- [ ] **Step 3:** Implement the functions and save command; timezone/day-start changes never rewrite stored UTC due instants. Store configurable presentation/shortcut settings too; no hard-coded Cairo offset.
- [ ] **Step 4:** Re-run the test binary; expect all assertions PASS, including unchanged due instants after timezone changes when card storage becomes available in Task 4.
- [ ] **Step 5:** Commit: `feat: validate settings and timezone-aware study days`.

### Task 3: Deck lifecycle with independent default optimization profiles

**Files:** Create `crates/openrecall-core/src/contracts/{library,profiles}.rs`, `src/domain/profile.rs`, `src/storage/{library,profiles}.rs`, `src/services/library.rs`, migration `002_library.sql`; Modify core Cargo manifest/lockfile to add the pinned FSRS dependency for official default parameters; Test `crates/openrecall-core/tests/library.rs`.

**Interfaces:** `CreateDeck { name: String, profile_id: Option<Id> }` -> `DeckDetail`; `RenameDeck { deck_id, name, expected_revision }` -> `DeckDetail`; `DeleteDeck { deck_id, expected_revision }` -> `DeleteReport`; `ListDecks {}` -> `Vec<DeckSummary>`; `GetDeck { deck_id }` -> `DeckDetail`. DeckSummary carries id/name/logical_cards/due_now/new_cards/profile_id/revision. ProfileSummary carries id/name/active_model_revision/member_deck_ids/config/revision. `TrainingConfig { num_epochs: Option<u16>, batch_size: u16, learning_rate: Option<f64>, seed: u32, max_seq_len: u16, gamma: f64, schedule_penalties: bool }` defaults to None/512/None/2023/256/1.0/true with spec ranges; None maps to the upstream automatic zero value. Extend Bootstrap with decks/profiles.

- [ ] **Step 1:** Write tests: create/rename/delete/reopen Unicode deck names; empty names rejected; independent default profile per new deck; selecting an existing profile works; `schedule_penalties == true`; deletion does not delete a shared profile. Check profile creation and deck insertion roll back together on failure.
```rust
// decks_start_independent_with_penalties_enabled
assert_ne!(first_deck.profile_id, second_deck.profile_id);
assert!(first_profile.config.schedule_penalties);
assert_eq!(shared_profile_after_delete.member_deck_ids, vec![surviving_deck_id]);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test library`; expect missing/incorrect lifecycle behavior.
- [ ] **Step 3:** Implement transactional deck operations and immutable default-model records. Retain unassigned profiles; validate provided profile IDs. Serialize real counts (zero before cards exist), never sample data.
- [ ] **Step 4:** Re-run the test binary and bootstrap/contract tests; expect PASS.
- [ ] **Step 5:** Commit: `feat: manage decks and default optimization profiles`.

### Task 4: Logical cards and alternative-form editing

**Files:** Create `crates/openrecall-core/src/domain/card.rs`, `src/contracts/cards.rs`, `src/storage/cards.rs`, `src/services/cards.rs`, migration `003_cards.sql`; Test `crates/openrecall-core/tests/cards.rs`.

**Interfaces:** `CardDraft { forms: Vec<FormDraft> }`; `FormDraft { id: Option<Id>, front, back, notes: String, language: Option<String> }`; `CreateCard { deck_id, draft }` -> `CardDetail`; `UpdateCard { card_id, draft, expected_revision }` -> `CardDetail`; `DeleteCard { card_id, expected_revision }` -> `DeleteReport`; `ListCards { deck_id, query, page: u32, page_size: u16 }` -> `Page<CardSummary>`; `GetCard { card_id }` -> `CardDetail`. CardDetail includes phase, memory, due_at, reviewed_at, counts, forms, revision; `MemoryState` has all three FSRS-7 fields. New cards are due at commit time with no memory.

- [ ] **Step 1:** Write tests for required nonblank front/back, optional empty notes, many variants, mixed Arabic/English/other Unicode, stable form IDs and order, pagination/search, cascade deletion, and unchanged learning progress after wording edits. Search treats quotes, percent signs and underscores as literal input safely. Assert logical-card count differs from form count. Include the deferred Task 2 due-instant invariant.
```rust
// alternative_forms_share_one_card
assert_eq!(deck_summary.logical_cards, 1);
assert_eq!(card.forms.len(), 3);
assert_eq!(edited_card.memory, original_card.memory);
assert_eq!(edited_card.due_at, original_card.due_at);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test cards`; expect intended CRUD/validation failures.
- [ ] **Step 3:** Implement card ownership, validation and persistence. Reject removing every form or referencing another card's form ID. Preserve all text exactly and invalidate relevant data revisions.
- [ ] **Step 4:** Re-run cards, library and settings_time tests; expect PASS.
- [ ] **Step 5:** Commit: `feat: persist logical cards and editable alternative forms`.

### Task 5: Genuine FSRS-7 adapter and rating previews

**Files:** Create `crates/openrecall-core/src/scheduling/{mod,fsrs}.rs`; verify the exact Git dependency already added for defaults in Task 3; Test `crates/openrecall-core/tests/fsrs_adapter.rs`, fixtures `tests/fixtures/fsrs7_golden.json` with documented upstream provenance.

**Interfaces:** `FsrsScheduler::new(parameters: &[f32]) -> AppResult<FsrsScheduler>`; `preview(&self, memory: Option<MemoryState>, elapsed_days: f32, policy: SchedulePolicy) -> AppResult<RatingPreviews>`; `replay(&self, history: &[ReviewObservation]) -> AppResult<Option<MemoryState>>`. `SchedulePolicy { desired_retention, min_interval_seconds, max_interval_days }`; RatingPreviews has Again/Hard/Good/Easy outcomes `{ memory, interval_seconds }`; Rating serializes as again/hard/good/easy and maps to 1/2/3/4.

- [ ] **Step 1:** Write golden behavioral tests for first review, fractional elapsed days, all ratings, stability_fast preservation, interval clamps, finite/length validation, and replay. Use independent pinned-upstream expectations, not self-generated snapshots accepted without review.
```rust
// intraday_previews_match_pinned_upstream_golden_values
assert!((preview.good.interval_seconds - golden.good_seconds).abs() < 0.01);
assert_eq!(replayed.stability_fast, golden.stability_fast);
assert!(preview.again.interval_seconds >= 60.0);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test fsrs_adapter`; expect an unimplemented adapter to fail.
- [ ] **Step 3:** Implement the thin upstream adapter using `next_states_with_elapsed_days`, explicitly verify FSRS-7 model identity, preserve its memory, and clamp only scheduling intervals. Do not introduce learning-step tables or interval fuzz.
- [ ] **Step 4:** Re-run the test binary; expect numerical agreement within documented float tolerance. Confirm Cargo.lock pins the approved Git SHA.
- [ ] **Step 5:** Commit: `feat: integrate pinned upstream FSRS-7 scheduling`.

### Task 6: Persistent randomized form selection

**Files:** Create `crates/openrecall-core/src/scheduling/forms.rs`; Modify cards storage/domain and migration `004_form_bags.sql`; Test `crates/openrecall-core/tests/form_bags.rs`.

**Interfaces:** `FormBag { remaining: Vec<Id>, consumed: Vec<Id>, last_consumed: Option<Id>, reserved: Option<Id>, cycle: u32 }`; `reserve_form(bag: &mut FormBag, form_ids: &[Id], rng: &mut dyn RandomSource) -> AppResult<Id>`; `consume_form(bag: &mut FormBag, reserved_id: &Id) -> AppResult<()>`; `reconcile_forms(bag: &mut FormBag, form_ids: &[Id], rng: &mut dyn RandomSource) -> AppResult<()>`. Consume is called only inside Task 8's rating transaction.

- [ ] **Step 1:** Write property-style tests over multiple seeds: each ID once per cycle, runtime orders can differ, no immediate boundary repeat with >=2 forms, one-form reuse, reserved choice stable across reopen, edits/new IDs/deletions reconcile, and restoring before-state gives exactly the same choice after boundary exhaustion.
```rust
// bag_cycle_contains_every_form_once
assert_eq!(cycle_ids.iter().collect::<std::collections::HashSet<_>>().len(), form_ids.len());
assert_ne!(previous_cycle_last, next_cycle_first);
assert_eq!(after_undo_bag, before_rating_bag);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test form_bags`; expect missing/nonrandom or duplicate-selection failures.
- [ ] **Step 3:** Implement shuffled bags using the injected random source; persist reservations without treating them as graded events. Never reshuffle on render/reveal/filter switch. Reconcile saved card edits atomically with bag changes.
- [ ] **Step 4:** Re-run form_bags and cards tests; expect PASS without probabilistic/flaky assertions about one specific random order.
- [ ] **Step 5:** Commit: `feat: randomize card forms without replacement`.

### Task 7: Resumable filtered review queues and timing

**Files:** Create `crates/openrecall-core/src/contracts/review.rs`, `src/domain/session.rs`, `src/scheduling/queue.rs`, `src/storage/sessions.rs`, `src/services/sessions.rs`, migration `005_sessions.sql`; Test `crates/openrecall-core/tests/session_queue.rs`.

**Interfaces:** `ReviewFilter = All | New | PreviouslyReviewed`; `ReviewView = Question(ReviewFrame) | EmptyFilter(EmptyFilterView) | Summary(SessionSummary)`; every variant carries session_id/filter. ReviewFrame includes presentation_id/card/form/answer_revealed/previews/progress/next_due_at/card_revision. `StartSession { deck_id }`, `ReadSession { session_id }`, `ResumeSession { session_id }`, `SetReviewFilter { session_id, filter }`, `RevealAnswer { session_id }`, `EndSession { session_id }` -> `ReviewView`; `SetSessionActivity { session_id, active: bool }` -> `SessionTiming`. `QueueEntry { card_id, due_at, created_at, review_count }`; `select_due(entries: &[QueueEntry], filter: ReviewFilter, now: DateTime<Utc>) -> Vec<QueueEntry>` owns pure ordering/filter rules. Define `SessionSummary { session_id, filter, filter_changes, started_at, ended_at, wall_ms, active_ms, attempts, unique_cards, new_cards, ratings, mean_response_ms, matching_due_remaining, all_due_remaining, next_due_at, can_undo }`. Extend Bootstrap with resumable session summaries.

- [ ] **Step 1:** Write tests for all-due default, live new/previously-reviewed eligibility, earliest-due order/ties, no future cards, newly due insertion during a displayed question, empty-filter state, parked form/reveal preservation, natural completion, zero-review summary, crash/resume, and exclusion of sleep/closed-app time from active duration. Assert filtering never consumes a bag. Use pure QueueEntry fixtures for reviewed-state unit cases until Task 8 adds integration through actual committed ratings.
```rust
// queue_filters_preserve_parked_presentations
assert_eq!(started_filter, ReviewFilter::All);
assert_eq!(ordered_due_ids, vec![oldest_due_id, newer_due_id]);
assert_eq!(restored_form_id, parked_form_id);
assert_eq!(active_ms_after_sleep, active_ms_before_sleep);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test session_queue`; expect intended queue/persistence/timing failures.
- [ ] **Step 3:** Implement snapshot-stable current presentation and due queries filtered in SQL; return the earliest matching future due instant for native/frontend wake scheduling. Recheck due cards atomically before completion. Persist active/reveal/parked states, session counters and monotonic timing checkpoints; exclude restart gaps.
- [ ] **Step 4:** Re-run session_queue and form_bags tests; expect PASS. Exercise wall-clock rollback separately from monotonic time advancement.
- [ ] **Step 5:** Commit: `feat: add resumable filtered review sessions`.

### Task 8: Atomic ratings, idempotency, undo, and summary

**Files:** Modify review contracts/session service/storage; add migration `006_reviews_undo.sql`; Test `crates/openrecall-core/tests/review_transactions.rs`.

**Interfaces:** `RateCard { session_id, presentation_id, rating: Rating, action_id: Id, expected_card_revision: String }` -> `ReviewView`; `UndoReview { session_id }` -> `ReviewView`. Define persisted ReviewLog/UndoRecord and populate Task 7's SessionSummary from durable rating events.

- [ ] **Step 1:** Write tests asserting fixed-clock preview/commit agreement; one atomic log/state/bag/session change; stale-card rejection; duplicate action retry changes nothing; retrying an undone action cannot apply it again; full rollback on injected write failure; all-due reappearance only at new due time; accurate attempt/unique counters; and undo from summary restoring memory, due, bag, prior filter, parked presentations, reveal, logs and counters.
```rust
// retried_and_undone_actions_are_not_double_rated
assert_eq!(logs_after_retry.len(), 1);
assert_eq!(logs_after_undo.len(), 0);
assert_eq!(after_undo_card, before_rating_card);
assert_eq!(logs_after_retry_of_undone_action.len(), 0);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test review_transactions`; expect transaction/undo behavior to fail before implementation.
- [ ] **Step 3:** Implement the rating at actual commit time through FsrsScheduler, consuming the reserved form in the same transaction. Reject a commit timestamp earlier than the card's last committed rating with a recoverable `ClockMovedBackward` error; retain the displayed question and all data. Persist before-images and action-token status independently so undo cannot resurrect a retry. Support at least one-level undo; invalidate it only on defined session/deletion/restore transitions. Implement phase transitions and summary statistics exactly as in spec sections 6-7.
- [ ] **Step 4:** Re-run review_transactions, session_queue and fsrs_adapter tests; expect PASS, including a learning card leaving new-only after its first rating and reentering new-only on undo.
- [ ] **Step 5:** Commit: `feat: commit reviews atomically with complete undo`.

### Task 9: Read-only JSON preview and duplicate-safe import

**Files:** Create `crates/openrecall-core/src/contracts/import.rs`, `src/services/import.rs`; Test `crates/openrecall-core/tests/json_import.rs`, fixtures `tests/fixtures/import_*.json`.

**Interfaces:** `PreviewImport { deck_id, text: String }` -> `ImportPreview { preview_id, library_revision, cards, counts, warnings }`; each preview card/form has a stable selection key, validation issues and duplicate references. `CommitImport { preview_id, library_revision, selected_card_keys: Vec<String>, excluded_form_keys: Vec<String> }` -> `ImportReport { inserted_cards, inserted_forms, excluded_cards, excluded_forms }`. Preview selections never accept invalid primaries/forms. `fingerprint(front: &str, back: &str) -> String` uses NFC, canonical line endings, outer trimming and whitespace collapse with a length-delimited pair; preserve case, diacritics and punctuation.

- [ ] **Step 1:** Write `preview_is_read_only_and_commit_is_atomic`, `duplicates_cover_primary_variants_other_decks_and_file_rows`, and `invalid_alternatives_can_be_excluded`: assert missing notes/variants defaults, exact invalid field paths, nested-variant rejection, unknown-key warnings, Unicode preservation, notes-independent matching, allowed partial variant exclusion, selectable valid rows, conflict refresh after library mutation, no partial write on injected failure, and the 50 MiB bound. Include UTF-8 BOM file handling, same-question/different-answer as distinct and same-text/different-case as distinct.
```rust
// preview_is_read_only_and_commit_is_atomic
assert_eq!(database_after_preview, database_before_preview);
assert_eq!(report.inserted_cards, 1);
assert_eq!(report.inserted_forms, 2);
assert_eq!(database_after_failed_commit, database_before_commit);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test json_import`; expect intended parser/duplicate/commit failures.
- [ ] **Step 3:** Implement two-phase import with in-memory preview plans and revision tokens. Scan existing primary and alternative forms across all decks; recheck before one commit transaction. Do not silently merge unique alternatives into an existing duplicated primary. Newly imported logical cards share one due instant at import commit and independent form bags.
- [ ] **Step 4:** Re-run json_import and cards tests; expect PASS, including zero database writes from a successful or failed preview.
- [ ] **Step 5:** Commit: `feat: preview and validate duplicate-safe JSON imports`.

### Task 10: Shared optimization profiles, attach, detach, and reset

**Files:** Create `crates/openrecall-core/src/services/profiles.rs`; Modify profile contracts/storage, add migration `007_model_revisions.sql`; Test `crates/openrecall-core/tests/profiles.rs`.

**Interfaces:** `ListProfiles {}` -> `Vec<ProfileSummary>`; `GetProfile { profile_id }` -> `ProfileDetail` (members, configuration, immutable model metadata, provenance); `CreateProfile { name }`, `RenameProfile { profile_id, name, expected_revision }`, `SaveTrainingConfig { profile_id, config, expected_revision }`, `AssignProfile { deck_id, profile_id, expected_deck_revision, expected_profile_revision }`, `DetachProfile { deck_id, new_name, expected_deck_revision }`, `ResetProfile { profile_id, expected_revision }` -> `ProfileDetail`; `DeleteProfile { profile_id, expected_revision }` -> `DeleteReport`. Profile-only operations compare the profile revision; assignment also compares the selected model/profile revision shown during confirmation.

- [ ] **Step 1:** Write tests asserting explicit member sets, default-on penalties and persisted off override, independent scheduling policies despite shared weights, memory replay with unchanged due times on attach/reset, immutable revision provenance, detach copying weights/config but isolating subsequent changes, blocked deletion while assigned, safe deck deletion, revision conflicts, and rejection of reassignment/reset with an involved resumable session.
```rust
// detach_isolates_future_training_without_resetting_progress
assert_ne!(detached_profile.id, shared_profile.id);
assert_eq!(detached_profile.config, shared_profile.config);
assert_eq!(due_after_detach, due_before_detach);
assert_eq!(unrelated_profile_after_reset, unrelated_profile_before_reset);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test profiles`; expect missing membership/replay/isolation behaviors.
- [ ] **Step 3:** Implement profile transactions using FsrsScheduler replay on each affected card's own history. Increment membership/config/data revisions and invalidate pending candidates. Keep prior model revisions referenced by history. Never pool unrelated decks implicitly or delete their content through a profile operation.
- [ ] **Step 4:** Re-run profiles, library, cards and review_transactions tests; expect PASS. Assert detaching does not change any shuffle bag or due timestamp.
- [ ] **Step 5:** Commit: `feat: share and separate optimization profiles safely`.

### Task 11: Leakage-free training datasets and data sufficiency

**Files:** Create `crates/openrecall-core/src/contracts/training.rs`, `src/training/{mod,dataset,eligibility}.rs`; Modify profile queries; Test `crates/openrecall-core/tests/training_data.rs`, fixture `tests/fixtures/training_histories.json` with synthetic-data provenance.

**Interfaces:** `GetTrainingEligibility { profile_id }` -> `TrainingEligibility { eligible, usable_observations, distinct_cards, study_days, again_count, recalled_count, train_count, holdout_count, holdout_cards, unmet_conditions, contributions }`; `build_dataset(history: &[TrainingObservation], config: &TrainingConfig, membership: &[Id]) -> AppResult<TrainingDataset>`; `TrainingDataset { train_items, train_card_ids, holdout_targets, baseline_parameters, revision_token, contributions }`. TrainingObservation includes globally unique card/deck IDs, UTC time, rating and undone/deleted eligibility. Each holdout target stores pre-target history, target rating and elapsed time, never target-inclusive memory.

- [ ] **Step 1:** Write tests with exact boundary counts: 399/400 observations, 49/50 cards, six/seven study dates, four/five Again and recalled outcomes, 63/64 training prefixes, 79/80 holdout observations and nine/10 holdout cards. Assert first ratings do not count; max-sequence filtering is reflected in usable counts; variants share card IDs; unrelated profiles and undone/deleted reviews are excluded; fractional deltas survive; no prefix crosses the temporal cutoff or includes its target label during prediction.
```rust
// eligibility_counts_usable_nonfirst_observations
assert_eq!(ineligible.usable_observations, 399);
assert!(!ineligible.eligible);
assert_eq!(sufficient.usable_observations, 400);
assert!(sufficient.eligible);
assert!(train_target_times.iter().all(|at| *at < holdout_cutoff));
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test training_data`; expect intended eligibility/leakage failures.
- [ ] **Step 3:** Implement deterministic chronological splitting (oldest approximately 80%, newest approximately 20%), stable aligned integer IDs only inside the FSRS dataset, and the complete conservative gate. Enforce chronological grouping on equal timestamps so training sees no later observation. Profile membership is explicit; show contributions without treating pooled counts as a single-deck statistic.
- [ ] **Step 4:** Re-run training_data and profiles tests; expect PASS. Inspect the sufficient fixture's counts and cutoff in test output so eligibility is demonstrated, not assumed.
- [ ] **Step 5:** Commit: `feat: gate profile training with chronological datasets`.

### Task 12: Real local training, validation, cancellation, and activation

**Files:** Create `crates/openrecall-core/src/training/{runner,evaluate}.rs`, `src/services/training.rs`, `src/jobs.rs`, migration `008_training_runs.sql`; Test `crates/openrecall-core/tests/training_jobs.rs`.

**Interfaces:** `StartTraining { profile_id }` -> `TrainingJob { id, profile_id, status, progress }`; `GetTrainingJob { job_id }` and `CancelTraining { job_id }` -> `TrainingJob`; `ApplyTraining { job_id, expected_profile_revision }` -> `ProfileDetail`. `train_snapshot(dataset: TrainingDataset, config: TrainingConfig, progress: Arc<Mutex<fsrs::CombinedProgressState>>) -> AppResult<TrainingCandidate>`; `score_targets(parameters: &[f32], targets: &[HeldOutTarget]) -> AppResult<EvaluationMetrics>`; `candidate_acceptable(baseline: EvaluationMetrics, candidate: EvaluationMetrics) -> bool`. Metrics are unweighted target log_loss/RMSE and optional per-deck breakdowns. TrainingJob status is queued/running/cancelled/interrupted/failed/noImprovement/ready/applied.

- [ ] **Step 1:** Write a real-upstream integration test on the sufficient deterministic fixture asserting a finite, valid FSRS-7 vector, explicit model version/short-term/penalty configuration, genuine progress, and reproducible metrics. Add focused tests for tolerance boundaries (0.0001), unchanged/default result not labeled improved, insufficient-data rejection in Rust, cancellation via `want_abort`, unchanged active model on failure, stale membership/data/config/baseline rejection, atomic activation across all linked decks, and no modification to other profiles. Use barriers/injected job timing for cancellation races rather than flaky sleeps.
```rust
// real_training_is_scoped_and_stale_candidates_cannot_apply
assert!(candidate.parameters.iter().all(|value| value.is_finite()));
assert!(recorded_run.config.schedule_penalties);
assert_eq!(active_revision_after_stale_apply, active_revision_before_apply);
assert_eq!(other_profile_after_training, other_profile_before_training);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test training_jobs`; expect intended optimizer/job/activation failures.
- [ ] **Step 3:** Implement jobs on worker threads using immutable snapshots; never hold the database/UI lock during optimization. Call upstream `compute_parameters` with `ComputeParametersVersion::Fsrs7`, `enable_short_term: true`, explicit `enable_sched_penalties`, aligned card IDs and validated hyperparameters. Propagate the library's real progress/abort state. Persist completed reports; mark abandoned running jobs interrupted after restart.
- [ ] **Step 4:** Implement held-out evaluation before optional activation; preserve baseline on no improvement. Apply one immutable model revision and replay all linked histories in one transaction while preserving due instants and guarding active/resumable sessions. Run `cargo test -p openrecall-core --test training_jobs --test training_data --test profiles`; expect PASS, and record the actual upstream fixture run result.
- [ ] **Step 5:** Commit: `feat: train and validate reusable FSRS-7 profiles offline`.

### Task 13: Defined deck statistics and forecasts

**Files:** Create `crates/openrecall-core/src/contracts/statistics.rs`, `src/storage/statistics.rs`, `src/services/statistics.rs`; Test `crates/openrecall-core/tests/statistics.rs`.

**Interfaces:** `GetDeckStatistics { deck_id }` -> `DeckStatistics { cards, forms, phases, due_now, due_later_today, next_due_at, forecast_7_days, forecast_30_days, today, last_7_days, last_30_days, all_time, daily_history, profile_status }`. `PeriodStatistics` includes attempts/unique_cards/active_ms/mean_response_ms/rating_counts/observed_recall_rate; absent denominators produce null, not NaN or invented zero accuracy.

- [ ] **Step 1:** Write exact-fixture tests for logical-card versus form counts, each phase, due forecast boundaries around Cairo rollover/DST, today/seven/thirty/all-time windows, all rating counts, 2-4 recall definition, mean active response, zero-denominator states, shared-profile contribution labels, and recomputation after rating/undo/card or deck deletion. Include completed session count refresh after history deletion.
```rust
// statistics_count_logical_cards_and_surviving_reviews
assert_eq!(stats.cards, 1);
assert_eq!(stats.forms, 3);
assert_eq!(stats.all_time.attempts, 4);
assert_eq!(stats.all_time.observed_recall_rate, Some(0.75));
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test statistics`; expect missing or incorrect aggregation failures.
- [ ] **Step 3:** Implement indexed SQL aggregation and timezone-aware day bucketing using stored UTC instants. Derive counts from authoritative surviving history, with transactional cache invalidation if caching is introduced. Keep per-deck metrics separate from pooled profile eligibility.
- [ ] **Step 4:** Re-run statistics, review_transactions and settings_time tests; expect PASS against manually calculated fixture values.
- [ ] **Step 5:** Commit: `feat: report deck progress and study-day forecasts`.

### Task 14: Complete portable backup format and safe restore core

**Files:** Create `crates/openrecall-core/src/contracts/backup.rs`, `src/backup/{mod,format,validation,files}.rs`, `src/storage/backup.rs`, `src/services/backup.rs`; Test `crates/openrecall-core/tests/backups.rs`.

**Interfaces:** `BackupEnvelope { format_version, schema_version, app_version, exported_at, payload_sha256, data: BackupData }`; BackupData contains every persisted user entity, bags, profiles/assignments/model revisions, histories, training reports, and resumable session/filter/undo state. `export_snapshot(&mut self, destination: &Path) -> AppResult<ExportReport>`; `preview_restore(&self, source: &Path) -> AppResult<RestorePreview>`; `restore_snapshot(&mut self, preview_id: Id, expected_library_revision: String, safety_destination: &Path) -> AppResult<RestoreReport>`. Preview/restore paths stay in native/core integration, not user-supplied IPC arbitrary-path commands.

- [ ] **Step 1:** Write round-trip tests containing a shared trained profile, detached copy, several form-bag cycles, a parked revealed question, active filter, undone-action token and complete settings. Assert field equality after restore, no machine path leakage, corrupted digest/future format/dangling references/invalid model rejection, older-format migration when supported, revision conflict rejection, collision-safe filenames, and original DB unchanged on safety-backup or transaction write failure. Inject small test resource limits to prove export never reports creating a backup that its same-version restore rejects merely because their limits differ.
```rust
// restore_preserves_full_user_state_and_fails_atomically
assert_eq!(restored_backup_data, exported_backup_data);
assert_eq!(after_safety_backup_failure, original_database);
assert_eq!(after_future_format_rejection, original_database);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-core --test backups`; expect intended snapshot/validation/rollback failures.
- [ ] **Step 3:** Implement consistent SQLite snapshot reads, canonical payload digest, bounded parsing with explicit resource errors, relationship/model validation and full replacement. Write temp/flush/atomic-finalize files; create a verified safety backup before one replacement transaction. Retain resumable valid undo, mark in-flight training jobs interrupted, and resolve runtime paths afresh.
- [ ] **Step 4:** Re-run backups and review_transactions/profile tests; expect PASS, including a rating and undo after restore. Confirm no preview operation writes to the current library.
- [ ] **Step 5:** Commit: `feat: export and safely restore complete offline backups`.

### Task 15: Native shell and accessible Arabic library UI

**Files:** Create root `package.json`, `pnpm-lock.yaml`, `tsconfig*.json`, `vite.config.ts`, `vitest.config.ts`, `eslint.config.*`, `index.html`, `src-tauri/{Cargo.toml,build.rs,tauri.conf.json}`, `src-tauri/src/{lib,main,commands,paths,jobs}.rs`, `src-tauri/capabilities/default.json`, `crates/openrecall-core/src/contracts/system.rs`, `src/{main.tsx,app/App.tsx,app/navigation.ts}`, `src/shared/{api,errors,format,focus}.ts`, `src/components/{Dialog,Disclosure,FormErrors,StatusRegion}.tsx`, `src/styles/{tokens,global}.css`, `src/features/library/*`, `src/test/{setup,mockApi}.ts`; Modify workspace membership and DTO export configuration; Test `src/features/library/LibraryPage.test.tsx`, `src/shared/api.test.ts`, `src-tauri/tests/paths.rs`.

**Interfaces:** Native `#[tauri::command] async fn execute(request: Request, state: State<'_, AppState>) -> Result<Response, AppError>` forwards to core App on a blocking worker without blocking the UI. AppState owns serialized core access; NativePaths resolves data and Downloads directories. `createTauriApi() -> Api`; `App({ api, native }: { api: Api; native: NativeApi })`; NativeApi is a separate typed wrapper for Task 19/20's file/update commands and `getSystemInfo(): Promise<SystemInfo>` (read-only app/engine/schema versions and data location). Export native DTO roots too. Navigation state is a discriminated library/deck/review/settings/profiles route, with explicit unsaved-edit protection. MockApi returns contracted fixture responses only; it is not a second scheduler or production data store.

- [ ] **Step 1:** Verify package peer/engine compatibility, install the chosen stable pnpm and dependencies, create minimal frontend/native test hosts, and generate DTOs with `pnpm contracts`. Write tests for Arabic/RTL document attributes, one H1, visible create-deck action, collapsed disclosure behavior, keyboard-only dialog focus/return, create/rename/delete invocation, real error rendering, and `Api.call` payload/result-kind verification. Native path tests use temporary directories.
```tsx
// library_has_labeled_keyboard_accessible_primary_actions
expect(document.documentElement).toHaveAttribute('lang', 'ar');
expect(document.documentElement).toHaveAttribute('dir', 'rtl');
expect(screen.getByRole('button', { name: 'إنشاء مجموعة' })).toBeVisible();
expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1);
```

- [ ] **Step 2:** Run `pnpm test -- src/features/library/LibraryPage.test.tsx src/shared/api.test.ts` and `cargo test -p openrecall-desktop --test paths`; expect intended UI/native-adapter failures.
- [ ] **Step 3:** Implement the shell, system-path DB initialization, single-instance behavior, typed adapter and semantic dark Arabic library. Follow the frontend-design skill when implementation begins. Keep custom command permissions/CSP restricted; no remote assets, arbitrary filesystem commands or embedded test server in production. Export all DTO roots deterministically into `src/shared/generated/`.
- [ ] **Step 4:** Re-run the tests and `pnpm typecheck`, `pnpm lint`, `pnpm build`, `cargo check -p openrecall-desktop`; expect PASS. Launch the desktop app against test data and verify a real create/restart/rename/delete path without a screen reader.
- [ ] **Step 5:** Commit: `feat: launch accessible Arabic Tauri library`.

### Task 16: Card editor and selectable JSON import preview UI

**Files:** Create `src/features/cards/{DeckPage,CardEditor,CardList}.tsx`, `src/features/import/{ImportDialog,ImportPreview}.tsx`, `src-tauri/src/files.rs` and associated hooks; Modify navigation and native file wrapper; Test `src/features/cards/CardEditor.test.tsx`, `src/features/import/ImportPreview.test.tsx`.

**Interfaces:** `DeckPage({ deckId, api, native, navigate })`; `CardEditor({ card: CardDetail | null, deckId, api, onSaved, onCancel })`; `ImportDialog({ deckId, api, native, onComplete, onCancel })`. Consume Task 4/9's generated contracts. Add `NativeApi.pickImportFile(): Promise<{ name: string; text: string } | null>` with a native dialog and backend-enforced UTF-8/50 MiB validation; cancel is not an error.

- [ ] **Step 1:** Write keyboard-focused tests for optional/unbounded variant rows, stable IDs on edit, focus after add/remove, required-field error summary, discard confirmation, search/pagination, collapsed card list, multilingual `dir="auto"` rendering, and full import selection flow. Include HTML-looking question text rendered literally without creating headings or executing markup. Assert invalid primary rows cannot be selected, invalid/duplicate variants can be excluded explicitly, warning/count changes are announced, and preview/cancel calls never submit CommitImport.
```tsx
// preview_requires_explicit_valid_selection_before_commit
expect(commitCalls).toHaveLength(0);
expect(invalidPrimaryCheckbox).toBeDisabled();
expect(validPrimaryCheckbox).toBeChecked();
expect(renderedQuestion).toHaveAttribute('dir', 'auto');
```

- [ ] **Step 2:** Run `pnpm test -- src/features/cards/CardEditor.test.tsx src/features/import/ImportPreview.test.tsx`; expect intended UI behavior failures.
- [ ] **Step 3:** Implement editor and paginated preview with labels/checkbox grouping, select-all-valid/deselect-all, row/field reasons, pasted JSON and native file selection. Render text nodes, never dangerouslySetInnerHTML. Show actual inserted logical-card/form counts and a useful revision-conflict refresh path.
- [ ] **Step 4:** Re-run tests, typecheck and lint; expect PASS. Native smoke uses an imported JSON fixture and reads back real cards through the core.
- [ ] **Step 5:** Commit: `feat: edit card forms and preview JSON imports accessibly`.

### Task 17: Accessible live review, filters, previews, undo and summary UI

**Files:** Create `src/features/review/{ReviewPage,ReviewQuestion,RatingButtons,ReviewSummary,useReviewSession}.tsx` (hook may use `.ts`); Test `src/features/review/ReviewPage.test.tsx`, `src/features/review/useReviewSession.test.ts`.

**Interfaces:** `ReviewPage({ sessionId, api, navigate })`; `useReviewSession(api: Api, sessionId: Id)` returns the latest ReviewView plus reveal/rate/undo/setFilter/end actions. Use Task 7/8's responses as authoritative. Schedule a refresh for next eligible due time and on focus/resume, pausing active timing while inactive. Use browser monotonic observations only to request checkpoints; never compute schedules in JavaScript.

- [ ] **Step 1:** Write assertions for label-only H1/H2/H3, answer absence before reveal, retained question after reveal, H1/H2 focus transitions, four accessible names including intervals, duplicate-click suppression, action-token retry, default-all and filter switching, empty-filter recovery, live progress and new arrivals, exact undo response rendering, error recovery, summary and undo-from-summary. Verify start-review actions from both a library deck tile and inside its deck create the same default-all flow. Include a switch at a form-cycle boundary that must preserve the reserved form/reveal state. Assert shortcuts do not fire in text inputs and can be disabled. Verify distant due times cannot overflow JavaScript's timer range and cause a tight loop; bound wake intervals to at most 60 seconds while a session is open.
```tsx
// reveal_keeps_question_and_focuses_label_only_answer_heading
expect(screen.getByRole('heading', { level: 1, name: 'السؤال' })).toBeVisible();
expect(screen.getByRole('heading', { level: 2, name: 'الإجابة' })).toHaveFocus();
expect(screen.getByRole('heading', { level: 3, name: 'الملاحظات' })).toBeVisible();
expect(screen.getByRole('button', { name: /افتكرتها بسهولة.*30 يوم/ })).toBeEnabled();
```

- [ ] **Step 2:** Run `pnpm test -- src/features/review`; expect intended interaction/focus/timer failures.
- [ ] **Step 3:** Implement a response-driven review state machine with no optimistic advancement before durable rating success. Provide documented buttons and optional Space/reveal, 1-4/rating, Ctrl+Z/undo shortcuts with text-input guards. Speak concise status changes through a polite region; render actual local due dates. Handle inactive/resumed windows and cancel obsolete timers on session/filter changes.
- [ ] **Step 4:** Re-run review tests with fake timers, typecheck and lint; expect PASS. Confirm the frontend's only form selection comes from the backend, including after retry and undo.
- [ ] **Step 5:** Commit: `feat: provide accessible filtered reviews and full undo`.

### Task 18: Settings, reusable profile training and deck statistics UI

**Files:** Create `src/features/settings/{SettingsPage,SchedulingSettings}.tsx`, `src/features/profiles/{ProfilesPage,ProfileEditor,TrainingPanel}.tsx`, `src/features/statistics/DeckStatistics.tsx`; Modify DeckPage/navigation; Test corresponding feature `*.test.tsx` files.

**Interfaces:** Consume Tasks 2/10/11/12/13. `ProfilesPage({ api, navigate })`; `TrainingPanel({ profileId, api })`; `DeckStatistics({ deckId, api })`; `SettingsPage({ api, native, navigate })`. Progress polling uses GetTrainingJob; activation requires a user action and the current revision token. DTO-generated fields/ranges are the source of truth for typed forms.

- [ ] **Step 1:** Write tests for every editable spec field/range/default, read-only model/version/data-location explanations, reset defaults, per-deck retention overrides, explicit default-on penalty checkbox, profile membership selection, shared-effects confirmation, detach preservation explanation, blocked profile deletion, unmet eligibility details, genuine job progress/cancel/failure/no-improvement/ready states, old/new metrics and held-out scope, and all statistics as accessible text/tables with empty states. Assert all long sections begin collapsed.
```tsx
// profile_defaults_and_training_eligibility_are_accessible
expect(screen.getByRole('checkbox', { name: /Schedule Penalties/ })).toBeChecked();
expect(screen.getByRole('button', { name: 'بدء التحسين' })).toBeDisabled();
expect(screen.getByText(/399.*400/)).toBeVisible();
```

- [ ] **Step 2:** Run `pnpm test -- src/features/settings src/features/profiles src/features/statistics`; expect intended forms/profile/training presentation failures.
- [ ] **Step 3:** Implement validated forms with Arabic help and field errors, profile management, manual training/apply actions, and defined metrics. Show pooled eligibility/contributions separately from deck-only statistics. Avoid frequent live-region chatter while polling; announce meaningful job phase/result changes. Keep UI responsive during native optimization.
- [ ] **Step 4:** Re-run tests, typecheck and lint; expect PASS. Verify the no-improvement path leaves every active profile/model ID unchanged.
- [ ] **Step 5:** Commit: `feat: manage settings, optimization profiles and study statistics`.

### Task 19: Native Downloads export and confirmed full restore UI

**Files:** Create `src/features/backup/{BackupPanel,RestorePreviewDialog}.tsx`; Modify `src-tauri/src/files.rs`, NativePaths/NativeApi/settings page and DTO exports; Test `src-tauri/tests/backup_commands.rs`, `src/features/backup/BackupPanel.test.tsx`.

**Interfaces:** NativeApi `exportBackup(): Promise<ExportReport>`, `pickBackupForPreview(): Promise<RestorePreview | null>`, `restoreBackup(input: { previewId: Id; expectedLibraryRevision: string }): Promise<RestoreReport>`, `revealExport(input: { exportId: Id }): Promise<void>`. Native commands resolve paths or use user-selected handles; the frontend cannot request arbitrary reads/writes. Native state binds preview/export IDs to the chosen files and current library revision.

- [ ] **Step 1:** Write tests proving default Downloads save, collision avoidance, native save fallback, cancellation, actual finalized file path, and file-manager reveal of only known exports. Assert full-restore preview counts, explicit replacement confirmation, safety backup first, revision-conflict refresh and preserved current DB/UI on injected disk failure or unsupported future backup. No success message before finalize.
```tsx
// failed_restore_keeps_library_and_does_not_announce_success
expect(restoreCallsBeforeConfirmation).toHaveLength(0);
expect(screen.queryByText('تمت استعادة النسخة الاحتياطية')).not.toBeInTheDocument();
expect(currentDeckNames).toEqual(originalDeckNames);
```

- [ ] **Step 2:** Run `cargo test -p openrecall-desktop --test backup_commands` and `pnpm test -- src/features/backup`; expect intended native/UI failures.
- [ ] **Step 3:** Connect native dialog/path handling to Task 14, implement Arabic confirmations/progress/results, and reload Bootstrap/navigation after successful replacement. Prevent restore during active writes/review without finishing the flow; preserve cancelled or failed restore inputs for retry. Add file operation permissions only as narrowly required.
- [ ] **Step 4:** Re-run those tests and backups core tests; expect PASS. Export a real native study library to a temporary Downloads test directory, restore it, and resume a question with intact form/filter/undo state.
- [ ] **Step 5:** Commit: `feat: export to Downloads and restore complete libraries`.

### Task 20: Signed updater and installable desktop packages

**Files:** Create `src-tauri/src/updates.rs`, `crates/openrecall-core/src/contracts/updates.rs`, `src/features/updates/UpdatePanel.tsx`, `src-tauri/tauri.offline.conf.json`, local icons, `scripts/release.mjs`, `docs/releases/signing.md`; Modify Tauri config/capabilities, native wrapper and DTO exports; Test `src-tauri/tests/update_policy.rs`, `src/features/updates/UpdatePanel.test.tsx`, `tests/maintenance/update_manifest.test.mjs`.

**Interfaces:** NativeApi `checkForUpdate(): Promise<UpdateStatus>`, `downloadUpdate(): Promise<UpdateStatus>`, `installUpdate(): Promise<void>`; status includes phase/current_version/available_version/notes/downloaded_bytes/total_bytes/error_code. Native state retains the verified updater object; update progress is exposed through a typed native event. Guard installation in Rust when a review, resumable session requiring completion, unsaved edit or write transaction is active. Unsaved-edit state is registered through a narrow native lifecycle call and cleared on navigation completion.

- [ ] **Step 1:** Write tests for no network check at startup, explicit offline failure, no-update/available/progress/retry-after-failure states, invalid signature/manifest rejection, blocked install during a guarded flow, durable state before restart, and a valid complete updater manifest. Assert update failures do not stop offline study. Use an injected updater transport for policy tests plus official verifier checks on signed fixtures.
```tsx
// offline_startup_does_not_check_updates_or_block_study
expect(updateChecksAfterStartup).toHaveLength(0);
expect(installCallsDuringReview).toHaveLength(0);
expect(screen.getByRole('button', { name: 'بدء المراجعة' })).toBeEnabled();
```

- [ ] **Step 2:** Run `cargo test -p openrecall-desktop --test update_policy`, `pnpm test -- src/features/updates`, and `node --test tests/maintenance/update_manifest.test.mjs`; expect intended integration/policy failures.
- [ ] **Step 3:** Integrate the official Tauri updater/restart APIs, generate signing keys outside the worktree after checking current official CLI syntax, and embed only the public key. Keep private keys out of logs/Git and later store them in GitHub secrets. Implement exact user-triggered update behavior and separate Windows offline-WebView2 installer configuration. Configure x64 NSIS, macOS arm64/x64 and Linux x64 AppImage/deb builds; document ad-hoc/unsigned OS publisher status accurately.
- [ ] **Step 4:** Re-run tests and build `pnpm tauri build --bundles nsis`; expect a real Windows installer. Build the offline variant with `pnpm tauri build --config src-tauri/tauri.offline.conf.json --bundles nsis`. Validate signed artifacts/manifest with the real public key; secondary platforms build in Task 21's CI.
- [ ] **Step 5:** Commit: `feat: package OpenRecall with verified in-app updates`.

### Task 21: GitHub source, quality gates and maintenance automation

**Files:** Create `.github/workflows/{quality,release,dependency-automerge,fsrs-release-watch}.yml`, `.github/dependabot.yml`, `.github/{ISSUE_TEMPLATE/*,pull_request_template.md}`, `scripts/{verify,check-fsrs-release}.mjs`, `scripts/dependency-policy.mjs`, `tests/maintenance/{dependency_policy,fsrs_release_watch}.test.mjs`, `README.md`, `LICENSE`, `THIRD_PARTY_NOTICES.md`, `docs/development/{contributing,architecture}.md`.

**Interfaces:** `dependencyChangeAllowed(input: PullRequestFacts): boolean` allows only verified bot-authored same-repository dependency-only patch/minor changes, excludes workflow/security-sensitive paths, major versions and FSRS source/model changes, and never overrides required checks. `inspectFsrsRelease(metadata, source, goldenResults) -> { actionable, version, reason }` verifies required FSRS-7 APIs/model format instead of comparing version numbers alone; the watcher deduplicates its actionable issue/PR by version and marker. `pnpm verify` runs the documented local quality suite.

- [ ] **Step 1:** Write policy tests for spoofed bot names, fork PRs, workflow/path changes, major updates, altered FSRS Git revision, failing/pending checks, allowed patch/minor dependency updates and unchanged upstream watch results. Write watcher tests for a higher FSRS-6-only version being rejected and a compatible FSRS-7 release producing one actionable deduplicated result.
```js
// dependency_policy_blocks_untrusted_or_model_changing_updates
assert.equal(dependencyChangeAllowed(forkBotPr), false);
assert.equal(dependencyChangeAllowed(fsrsRevisionPr), false);
assert.equal(dependencyChangeAllowed(passingPatchOnlyPr), true);
assert.equal(inspectFsrsRelease(fsrs6Metadata, fsrs6Source, []).actionable, false);
assert.equal(inspectFsrsRelease(fsrs7Metadata, fsrs7Source, passingGoldenResults).actionable, true);
```

- [ ] **Step 2:** Run `node --test tests/maintenance/dependency_policy.test.mjs tests/maintenance/fsrs_release_watch.test.mjs`; expect intended policy failures. Before any publication, run the full local verification suite and inspect tracked files for private keys/user data.
- [ ] **Step 3:** Implement least-privilege workflows: formatting/lint/types/generated-contract drift/core and frontend tests, Windows native tests, secondary platform builds, and release matrix. Configure Dependabot, guarded auto-merge and a quiet deduplicating FSRS watcher. Use SHA-pinned actions where practical; never run untrusted PR code with release secrets. Fail releases visibly when signing configuration is absent. Write real setup/offline/backup/release/support documentation and MIT/upstream license notices.
- [ ] **Step 4:** Run `pnpm verify` and validate workflow syntax with actionlint; expect all applicable local checks PASS before publishing tracked source. Keep release signing secrets out of this command's output.
- [ ] **Step 5:** Commit automation/docs as `ci: add verified builds and guarded GitHub maintenance`.
- [ ] **Step 6:** Check `wip-bunker-42/OpenRecall` is available and create it public with the connected account; an existing conflicting repository is a stop-and-report condition, never an overwrite.
- [ ] **Step 7:** Push approved documentation main plus the committed implementation branch, open the implementation PR, and immediately attach its URL using `mcp__codex_app__attach_artifact`. Store signing credentials as repository secrets without printing them and configure required status checks/auto-merge where permitted.
- [ ] **Step 8:** Wait for actual CI results and repair failures through the debugging workflow, committing and pushing repairs. Confirm Windows, macOS arm64/x64 and Linux artifacts build before claiming build verification. Retain the PR for Task 22's whole-branch review and final gates.

### Task 22: Automated accessibility/native QA, final review and release

**Files:** Create `playwright.config.ts`, `wdio.conf.ts`, `tests/e2e/{library,import,review,profiles,backup,accessibility}.spec.ts`, `tests/native/{persistence,review,offline,focus}.spec.ts`, `docs/testing/{automated-results,human-nvda-checklist}.md`, `docs/releases/0.1.0.md`; Modify verification/release workflows only as required by actual results.

**Interfaces:** Browser tests use a test-only contracted mock adapter to verify frontend interaction/accessibility, not to establish backend correctness. Native tests use the real compiled Tauri backend with an isolated temporary data directory and test-only WebDriver support excluded from production builds. Use the current documented WebdriverIO Tauri service/driver appropriate to the target; never introduce a production test endpoint or operate a screen reader. `pnpm test:e2e` runs Playwright; `pnpm test:native` runs the real desktop suite; `pnpm verify` runs the full nonpackaging check suite.

- [ ] **Step 1:** Write observable end-to-end cases: create/restart/edit/delete; valid/invalid/duplicate JSON selection; first/all/new/previously-reviewed filtered reviews, due arrival, answer headings, accessible interval names and undo-from-summary; profile attach/detach/insufficient-data guard; full export/restore; mixed-direction content; keyboard navigation and dialog focus; no automatically detected axe violations on each important expanded/collapsed/revealed/error state. Native tests block network at the OS/test transport boundary while exercising offline operations and inspect Windows WebView2 focus after Alt+Tab using automation only.
```ts
// accessibility_runs_without_launching_a_screen_reader
const audit = await new AxeBuilder({ page }).analyze();
expect(audit.violations).toEqual([]);
await expect(page.getByRole('heading', { level: 1, name: 'السؤال' })).toBeFocused();
```

- [ ] **Step 2:** Run `pnpm test:e2e` and `pnpm test:native`; confirm intended failures for any still-unimplemented behavior. Fix through systematic debugging and retain each meaningful regression case. Record any genuinely unavailable native test mechanism precisely; do not substitute a mock for a claimed native pass.
- [ ] **Step 3:** Run `pnpm contracts` then `git diff --exit-code -- src/shared/generated` to confirm no generated drift, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm typecheck`, `pnpm lint`, `pnpm test`, `pnpm test:e2e`, `pnpm test:native`, and both Windows production installer builds. Confirm offline runtime behavior, backup portability and signed updater manifest/artifacts. If a disposable update environment is available, exercise a real version-to-version update; otherwise report manifest/signature/integration checks and the unperformed full update separately.
- [ ] **Step 4:** Invoke `superpowers:requesting-code-review` for a fresh whole-branch review, implement verified findings through `receiving-code-review`, and rerun only affected checks plus required final gates. Follow `verification-before-completion`; record the actual test results and any genuine environment limits in the verification/release documents.
- [ ] **Step 5:** Commit verification/docs as `test: verify desktop study flows and release readiness`, push the final reviewed head, and confirm required CI checks apply to that exact revision.
- [ ] **Step 6:** Follow `finishing-a-development-branch`; user authorization covers public source publication and the intended PR/release workflow. Merge the implementation PR only after required checks and review pass. Attach every newly created PR to this chat.
- [ ] **Step 7:** Publish v0.1.0 and its verified installers/updater metadata through the release workflow, inspect its final artifact/manifest/signature results, and verify the public repository default branch contains the actual app.
- [ ] **Step 8:** Deliver clickable repository/release/PR links, a real Windows installer path/link, precise test evidence and platform limits, and the human-only NVDA checklist. Do not wait for or claim human NVDA results. Remove unused worktrees through the native archival workflow after preserving deliverables.

## Plan Self-Review and Coverage

- Intent/scope/selected stack: Tasks 1, 5, 15, 20-22.
- Navigation, Arabic/dark semantic UI, focus, collapsed sections and multilingual content: Tasks 15-18, 22.
- Logical model, transactions, deletion, migrations and durable settings: Tasks 1-4, 7-8, 10, 14.
- FSRS-7, sub-day scheduling, phase transitions, exact form-bag restoration: Tasks 5-8.
- Live filters, due-order merging, timing, resumability, complete undo and summary: Tasks 7-8, 17, 22.
- JSON validation/selection/duplicate matching and no-preview writes: Tasks 9, 16.
- Advanced settings and Cairo/DST semantics: Tasks 2, 10-12, 18.
- Profile lifecycle, pooled isolated training, eligibility, holdout/cancellation/activation: Tasks 10-12, 18.
- Statistics, deleted/undone history and forecasts: Task 13 with UI in Task 18.
- Full backup and safe replacement, Downloads and native dialogs: Tasks 14, 19.
- Offline behavior, signed updater, installer and cross-platform builds: Tasks 20-22.
- Public source, PR/check/merge/release/dependency maintenance and FSRS migration notice: Tasks 21-22.
- Five Review Focus cases have explicit owning tests above; human-only NVDA boundaries are repeated in the automated QA task.

Self-review completed on 2026-10-24: checked spec coverage, actionable steps/assertions, interface/type consistency, five Review Focus cases, and proportion to the design. Corrected early dependency ordering, summary DTO ownership, native/domain command separation, generated numeric types, profile revision checks, and commit-before-publication order. No unresolved design gaps remain in this review. The plan contains 22 sequential independently testable tasks and awaits user review.

Execution method has not yet been selected. Present this plan for review and ask the user to choose subagent-driven or native execution as required by the writing-plans skill. Product implementation starts after that user review.
