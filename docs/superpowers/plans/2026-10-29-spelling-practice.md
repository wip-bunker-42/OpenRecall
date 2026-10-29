# OpenRecall Spelling Practice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Release OpenRecall 0.2.0 with one additive spelling deck, durable original contexts, disposable accessible practice input, configurable question headings and daily completion statistics.

**Architecture:** Extend the existing authoritative Rust/SQLite core and typed React UI. Managed spelling cards reuse FSRS, sessions, profiles and undo; a normalized vocabulary/provenance registry collects source content atomically. Upgrade backup decoding before evolving payloads, then implement the shared schema, native operations, UI and one integrated release.

**Tech Stack:** Existing Tauri 2/Rust/rusqlite/React/TypeScript stack, unicode-segmentation, linkify, unicode-script, existing unicode-normalization, Vitest/Testing Library, Playwright/axe, WebdriverIO and GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-29-spelling-practice-design.md` (approved 2026-10-29 after clarifying that writing is discarded whenever the card is left).

## Global Constraints

- Product language: Arabic. Development language: English.
- At most one spelling deck in the local library; default name `مجموعة التدرب على الكتابة`.
- Include unknown names and specialist terms automatically; dictionary membership must not gate inclusion.
- Collection is additive. Source edits, form removal, card deletion, source-deck deletion and later source changes must not delete collected spelling words or reset their schedules, history or progress.
- Keep the last saved source context after the live source is deleted, so `عرض البطاقة الأصلية` still explains where the word came from.
- Practice input exists only inside the currently displayed card's text field. Leaving that card discards it immediately, including when returning through undo, filter changes or a resumed session. Never save the user's typing.
- Do not erase text on textarea blur. Pending/failed grading and polling keep the same displayed field intact.
- Ordinary study remains offline. Do not launch, operate or configure NVDA, Narrator or another screen reader.
- Preserve the pinned FSRS-7 engine and its `stability`, `stability_fast`, `difficulty`, default-on Schedule Penalties and training sufficiency gates.
- Normal review still requires answer reveal. Spelling allows direct self-rating with native interval previews; no text comparison or automatic speech.
- Source collection scans all front/back/notes in all forms, not titles, JSON keys or unrelated decks.
- Question titles are optional plain text, at most 160 characters, without control characters/newlines; missing/null/blank resolves to `السؤال`.
- Use configured timezone/day-start for completion; defaults remain Africa/Cairo and 01:00. Do not reinterpret the lifetime `has_graduated` flag.
- Schema 9 / backup format 2; accept real v0.1.0 format-1 backups before adding new defaults. Full backup limit remains 256 MiB; JSON card import limit remains 50 MiB.
- Persist `extractionPolicyVersion: 1`; unknown policy versions fail before restore writes.
- New release is v0.2.0. Published v0.1.0 tags/assets remain unchanged. Signing keys stay outside Git; production excludes automation.
- Windows 11 is primary; Windows 10/macOS/Linux are secondary. Report actual runtime, installer and update testing separately from builds.

## Review Focus

1. A source edit/deletion lands during spelling grading: keep the word's revision and learning state independent, preserve its current input and avoid a false stale-rating conflict (Tasks 7, 9, 12).
2. A shared context loses one word, then its source is unbound/deleted: retain that word's last matching snapshot while other words advance; garbage collection must not remove a referenced historical context (Tasks 7, 10).
3. A real format-1 file has nested forms in undo before-images: verify its original digest before injecting title/kind/spelling defaults and retain a usable undo after restore (Tasks 1, 10).
4. The same presentation returns through filter parking, navigation, summary undo or restart: its field must be empty, while blur/poll/source disclosure on the still-visible card must retain normal DOM editing (Tasks 12, 15).
5. A card is reviewed again tomorrow or near Cairo DST/day rollover: preserve yesterday's count, count intraday boundary crossings correctly and reverse only the relevant undone rating (Tasks 13, 14).

## Execution Contract and File Map

Preserve the previously selected **Native** execution method: one implementer in this session and one fresh whole-branch Sol review at the end. The design builder check is already complete; do not repeat it as an implementation review. The user must review this plan before implementation. Public push, PR merge and the new signed release are authorized after the required tests/review/checks.

This is one integrated release: headings appear in retained contexts; deck kind, provenance, backups and statistics share domain contracts. Backend/UI boundaries below are independently testable tasks, rather than independent deployments with conflicting intermediate backup formats.

At execution start invoke using-git-worktrees, reuse the clean attached isolated `feat/spelling-practice` checkout when suitable, and establish a fresh baseline. Do not remove protected primary worktrees. Never use real study data. Run each intended RED before implementation, GREEN afterward and the appropriate complete project suite before its commit. Maintain the executing-plans ledger; do not treat a typo, missing toolchain or unexpected setup failure as the intended RED.

```text
crates/openrecall-core/src/
  backup/{format,legacy_v1,decode,validation,files}.rs version dispatch, frozen legacy DTOs, validation
  domain/{card,spelling}.rs                         titles and vocabulary/context data
  spelling/{mod,extract,storage,collect,validate}.rs word policy, indexed storage and provenance
  statistics/{mod,completion}.rs                   day-completion calculation
  contracts/{cards,import,library,review,
    statistics,spelling,mod}.rs                     typed public operations
  storage/{mod,backup,migrations/009_spelling.sql} schema and full snapshot storage
  services/{spelling,cards,library,import,
    sessions,reviews,statistics,backup,mod}.rs       transactional integration
crates/openrecall-core/tests/
  {legacy_backup,question_titles,spelling_extract,
    spelling_schema,spelling_collection,
    spelling_lifecycle,spelling_exclusions,
    spelling_review,spelling_backup,
    completion_statistics}.rs                      behavioral integration tests
  fixtures/backups/{v0.1.0.json,README.md}           genuine synthetic legacy export and provenance
src/features/spelling/
  {SpellingSetup,SpellingSettings,ExcludedWords,
    SpellingQuestion,SourceContexts}.tsx            singleton settings, disposable field and contexts
src/features/{cards,import,review,statistics,
  library}/*                                       existing UI integration
src/test/{mockApi,cardFixtures,reviewFixtures,
  spellingFixtures}.ts                             typed test fixtures
tests/{e2e,native}/spelling.spec.ts                 browser/native study and privacy cases
docs/{testing,releases,development}/*               evidence, human checklist and release instructions
```

### Shared data and command decisions

All IDs/revisions are opaque `String`, timestamps RFC3339 UTC, Rust fields snake_case and wire fields camelCase. Use existing `AppResult`, `Page<T>`, `App::execute`, `Fixture::call`, `Fixture::advance/reopen` and generated TypeScript contracts. New enums serialize camelCase.

Assertion blocks below are excerpts within the named tests. Their setup variables
and local assertion helpers are test-local, not additional production APIs.

- `CardForm.question_title` and `FormDraft.question_title`: defaulted `Option<String>`; `ImportForm.question_title` carries the resolved value through preview/commit.
- `DeckKind::{Standard, Spelling}`; `DeckDetail.kind` defaults to Standard. SQL `decks.kind` is `standard` or `spelling` and agrees with payload.
- `SpellingConfig { deck_id, source_deck_id: Option<String>, source_deck_name: Option<String>, extraction_policy_version: u16, revision }`.
- `SpellingWord { card_id, deck_id, canonical, display }`.
- `SourceSnapshot { id, spelling_deck_id, source_deck_id, source_deck_name, source_card_id, source_form_id, source_form_index: u32, source_card_revision, extraction_policy_version: u16, captured_at, front, back, notes, question_title: Option<String>, language: Option<String>, content_sha256 }`; form index is zero-based.
- `WordOrigin { spelling_card_id, source_card_id, source_form_id, snapshot_id, first_seen_at }`.
- `SpellingExclusion { deck_id, canonical, excluded_at }`.
- `SpellingBackupData { configs: Vec<SpellingConfig>, words: Vec<SpellingWord>, snapshots: Vec<SourceSnapshot>, origins: Vec<WordOrigin>, exclusions: Vec<SpellingExclusion> }`; `BackupData.spelling` owns it. Legacy upgrade explicitly inserts its empty default.
- Snapshot fingerprint includes source identity/labels, form index and complete form content; it excludes capture time and the opaque source card revision, which can change on ratings without content changes. The backup digest still covers all serialized fields.
- `SourceAvailability::{Current, Changed, Deleted}` and `SpellingSourceView { snapshot: SourceSnapshot, availability: SourceAvailability }`.
- `SpellingDeckResult { deck: DeckDetail, config: SpellingConfig, added_words: u32 }`.
- `ReviewMode::{Standard, Spelling}`; add `ReviewFrame.mode` and `ReviewFrame.can_rate: bool`. RateInput stays unchanged and never contains practice text.
- `PeriodStatistics.completed_for_day: u32`; for multi-day periods it is the sum of daily completions. `LibraryStatistics { today: PeriodStatistics, daily_history: Vec<HistoryDay>, spelling_completed_today: u32 }`.

New Request/Response operations, defined in the task producing each:

| Kind | Payload | Result |
| --- | --- | --- |
| `createSpellingDeck` | `{sourceDeckId: String}` | `SpellingDeckResult` |
| `getSpellingConfig` | `{deckId: String}` | `SpellingConfig` |
| `setSpellingSource` | `{deckId, sourceDeckId, expectedRevision}` | `SpellingDeckResult` |
| `rescanSpellingSource` | `{deckId, expectedRevision}` | `SpellingDeckResult` |
| `listSpellingSources` | `{cardId, page: u32, pageSize: u16}` | `Page<SpellingSourceView>` |
| `listSpellingExclusions` | `{deckId, page, pageSize}` | `Page<SpellingExclusion>` |
| `allowSpellingWord` | `{deckId, canonical, expectedRevision}` | `SpellingDeckResult` |
| `getLibraryStatistics` | `{}` | `LibraryStatistics` |

Spelling mutation `expectedRevision` is the config revision. Advance it on binding/exclusion changes, not source-context refresh. Independently advance deck data revisions when vocabulary/history changes. Generic `deleteCard` still uses the card revision. Page sizes remain 10/20/50/100. Arabic errors are mapped in `src/shared/errors.ts`; no raw database/path diagnostics are exposed.

---

### Task 1: Version-dispatched backups and a real v0.1.0 fixture

**Files:** Create `crates/openrecall-core/src/backup/{legacy_v1,decode}.rs`, `crates/openrecall-core/tests/legacy_backup.rs`, `crates/openrecall-core/tests/fixtures/backups/{v0.1.0.json,README.md}`; Modify `crates/openrecall-core/src/backup/{format,files,mod}.rs`, `crates/openrecall-core/src/services/backup.rs`, `crates/openrecall-core/tests/backups.rs`, `src-tauri/tests/backup_commands.rs` where existing format expectations change.

**Interfaces:** Produce `backup::decode::decode(bytes: &[u8]) -> AppResult<ValidatedBackup>`. `ValidatedBackup` is non-serialized and carries original `format_version`, `schema_version`, `app_version`, `exported_at`, `payload_sha256`, plus upgraded `data: BackupData`. Change private `files::read(&Path, BackupLimits)` and `PreparedRestore` to use it. The writer still takes the current serialized `BackupEnvelope`; public App export/preview/restore signatures stay unchanged.

- [ ] **Step 1:** Before changing domain/backup code, capture a synthetic real format-1/schema-8 export using the current v0.1.0 core: Fixture, Unicode/multiple-form cards, real revealed ratings, a resumable session and undo. Use `App::export_snapshot(&f.dir.path().join("legacy.json"))`, copy its reported file to the fixture path and record baseline SHA, safe synthetic content and SHA256 in README. Write `legacy_checksum_precedes_defaults_and_nested_undo_survives` and `new_exports_are_format_two` using that frozen file.
  ```rust
  assert_eq!(legacy["formatVersion"], 1);
  assert_eq!(legacy["schemaVersion"], 8);
  assert_eq!(preview.format_version, 1);
  assert_eq!(new_export["formatVersion"], 2);
  assert_eq!(restored_card["dueAt"], legacy_card["dueAt"]);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test legacy_backup`; Expected: format-2 export assertion FAIL with current format 1, not a fixture-generation error.
- [ ] **Step 3:** Freeze the full legacy wire shape, including nested card/form undo before-images. Validate original sorted JSON/checksum/header before upgrading; do not interpret new fields smuggled into format 1. Set current `FORMAT_VERSION=2`, dispatch only supported versions and preserve source metadata in preview. Convert legacy data explicitly; later tasks extend this adapter with new defaults. Keep size caps and finalize verification.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `cargo test -p openrecall-core --features serde_json/preserve_order --test backups`, and `cargo test -p openrecall-desktop --test backup_commands`; Expected: PASS, including corrupt digest/future format/disk-failure cases. Run contracts/typecheck if native DTOs changed.
- [ ] **Step 5:** Commit as `feat: preserve legacy backups with versioned decoding`.

### Task 2: Native question titles and JSON compatibility

**Files:** Modify `crates/openrecall-core/src/domain/card.rs`, `crates/openrecall-core/src/contracts/{cards,import}.rs`, `crates/openrecall-core/src/services/{cards,import}.rs`, `crates/openrecall-core/src/backup/{legacy_v1,validation}.rs`, generated DTOs and typed fixtures; Create `crates/openrecall-core/tests/question_titles.rs`.

**Interfaces:** Produce `domain::card::normalize_question_title(value: Option<String>) -> AppResult<Option<String>>`; add the title fields listed above. Stored None resolves to `السؤال` in presentation. Import omitted variant title inherits the primary; explicit null/blank resets it.

- [ ] **Step 1:** Write `title_import_inherits_only_when_omitted`, `title_edit_retains_memory_and_form_identity`, and validation/round-trip cases. Include default old JSON, null, whitespace, 161-character title, wrong type, newline/control, plain HTML-looking text and title-only duplicates.
  ```rust
  assert_eq!(forms[1]["questionTitle"], "ترجم");
  assert!(forms[2]["questionTitle"].is_null());
  assert_eq!(edited["forms"][0]["id"], before["forms"][0]["id"]);
  assert_eq!(edited["memory"], before["memory"]);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test question_titles`; Expected: missing title/preview/validation behavior FAIL.
- [ ] **Step 3:** Normalize titles consistently in create/edit/import; allow the new JSON key, carry preview data into commit, preserve duplicate fingerprint semantics. Upgrade legacy forms and nested undo forms to None. Update all Rust constructors and TS fixture builders affected by generated required-nullable properties.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `pnpm contracts`, `pnpm typecheck`; Expected: PASS with regenerated DTOs and no legacy checksum regression.
- [ ] **Step 5:** Commit as `feat: support per-form question headings in native card data`.

### Task 3: Accessible title editing, preview and review headings

**Files:** Modify `src/features/cards/{CardEditor,CardEditor.test}.tsx`, `src/features/import/{ImportPreview,ImportPreview.test}.tsx`, `src/features/review/{ReviewQuestion,ReviewPage.test}.tsx`, `src/test/{cardFixtures,reviewFixtures}.ts`.

**Interfaces:** Consume `FormDraft.questionTitle`/`CardForm.questionTitle`; existing editor and ReviewQuestion signatures remain unchanged.

- [ ] **Step 1:** Write `edits_each_form_title_without_losing_other_drafts` and `review_uses_custom_h1_with_default_fallback`; verify preview reflects inherited/reset titles.
  ```tsx
  expect(screen.getByRole('heading', { level: 1, name: 'ترجم' })).toBeVisible();
  expect(screen.queryByRole('heading', { name: 'unpredictable' })).not.toBeInTheDocument();
  expect(screen.getByRole('heading', { level: 2, name: 'الإجابة' })).toBeVisible();
  ```
- [ ] **Step 2:** Run `pnpm test -- src/features/cards/CardEditor.test.tsx src/features/import/ImportPreview.test.tsx src/features/review/ReviewPage.test.tsx`; Expected: custom heading/editor assertions FAIL.
- [ ] **Step 3:** Add labelled per-form title inputs with default help, retain existing draft/focus logic and plain-text rendering, use auto direction for custom H1. Keep answer H2/notes H3 and ordinary reveal semantics.
- [ ] **Step 4:** Run `pnpm test`, `pnpm typecheck`, `pnpm lint`; Expected: PASS, including default heading and stale-navigation regressions.
- [ ] **Step 5:** Commit as `feat: expose accessible custom question titles`.

### Task 4: Versioned native word extraction and dependency notices

**Files:** Create `crates/openrecall-core/src/spelling/{mod,extract}.rs`, `crates/openrecall-core/tests/spelling_extract.rs`; Modify `crates/openrecall-core/src/lib.rs`, `crates/openrecall-core/Cargo.toml`, `Cargo.lock`, `licenses/{manifest.json,RUST.txt}`, `THIRD_PARTY_NOTICES.md`, `docs/development/toolchain.md` as required by resolved licenses.

**Interfaces:** Produce `ExtractedWord { canonical: String, display: String }`, `pub const EXTRACTION_POLICY_VERSION: u16 = 1`, `extract_words(text: &str) -> Vec<ExtractedWord>`, `canonical_word(text: &str) -> Option<String>` in `spelling::extract`. Output unique words in first-appearance order.

- [ ] **Step 1:** Write corpus cases named `unknown_words_contractions_and_compounds_are_whole`, `links_numbers_and_mixed_scripts_do_not_leak_fragments`, `canonical_keys_preserve_distinct_inflections` and `alphanumeric_and_symbol_terms_are_not_dictionary_gated`.
  ```rust
  assert_eq!(keys("don't don’t well-known OpenAI B2B GPT-4 C++ C# F#"),
      ["don't", "well-known", "openai", "b2b", "gpt-4", "c++", "c#", "f#"]);
  assert_eq!(keys("Learn learn learning"), ["learn", "learning"]);
  assert_eq!(keys("a I the is 2026 test@example.com https://example.com"), ["a", "i", "the", "is"]);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test spelling_extract`; Expected: missing extractor API/behavior FAIL. Verify compatible current stable library releases/MSRVs/licenses before adding them; keep unrelated toolchain/dependencies stable.
- [ ] **Step 3:** Implement the approved section-4 policy using Unicode boundary/script APIs, existing NFC normalization and linkify spans with bare URL detection enabled. Join internal apostrophe/hyphen and bounded symbol suffixes without building a replacement language engine. Reject excluded spans as wholes, preserve accents and first display spelling, normalize standalone I to I. Add direct native dependencies. The existing pinned license helper is `target/tools/cargo-about/cargo-about-0.9.2-x86_64-pc-windows-msvc/cargo-about.exe` (not currently on PATH); use `generate --locked --format json -o target/licenses/about.json`, then `node scripts/generate-notices.mjs target/licenses/about.json`. Create the ignored output directory first; if the helper is absent, install the generator-compatible version during execution, not planning.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `node scripts/check-notices.mjs`, `pnpm verify`; Expected: PASS with finite bounded work on long synthetic text and all fixed extractor cases.
- [ ] **Step 5:** Commit as `feat: extract spelling terms with versioned Unicode rules`.

### Task 5: Schema 9, typed spelling storage and backup collections

**Files:** Create `crates/openrecall-core/src/domain/spelling.rs`, `crates/openrecall-core/src/spelling/storage.rs`, `crates/openrecall-core/src/contracts/spelling.rs`, `crates/openrecall-core/src/storage/migrations/009_spelling.sql`, `crates/openrecall-core/tests/spelling_schema.rs`; Modify `crates/openrecall-core/src/domain/mod.rs`, `crates/openrecall-core/src/contracts/{library,mod}.rs`, `crates/openrecall-core/src/storage/{mod,backup}.rs`, `crates/openrecall-core/src/services/library.rs`, `crates/openrecall-core/src/backup/{format,legacy_v1,validation}.rs`, generated DTOs and affected typed fixtures.

**Interfaces:** Produce the Shared data types above. Add `BackupData.spelling: SpellingBackupData`; legacy adapter inserts empty collections. Private storage functions in `spelling::storage` take `&rusqlite::Connection`, return `AppResult`, and persist typed rows without touching FSRS state. Their names are used only by the collector/service tasks and may stay private.

- [ ] **Step 1:** Write `schema8_upgrade_preserves_existing_payloads`, `sqlite_prevents_a_second_spelling_kind`, and `format2_storage_round_trips_spelling_rows`. Build valid typed synthetic rows directly for storage tests; never use real data.
  ```rust
  assert_eq!(pragma_user_version(&db), 9);
  assert_eq!(existing_deck.kind, DeckKind::Standard);
  assert!(second_spelling_insert.is_err());
  assert_eq!(restored.spelling.snapshots, before.spelling.snapshots);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test spelling_schema`; Expected: schema/kind/collection behavior FAIL.
- [ ] **Step 3:** Add kind column/check and singleton partial unique index. Update both `save_deck` and backup replacement to write the kind column consistently with payload; do not leave spelling inserts using SQL's standard default. Add config with nullable live-source FK, word map, immutable snapshots, origins and exclusions. Use historical source IDs without cascading live-source FKs; all rows are owned by the spelling deck. Enforce unique canonical/origin keys and snapshot references. Extend snapshot/replace order and basic ownership validation; exclude no new collection from backup. Preserve existing schema-8 data and update legacy defaults before verifying converted data.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `pnpm contracts`, `pnpm typecheck`; Expected: PASS for schema migration and ordinary fixtures, preserve source format 1 in restore preview.
- [ ] **Step 5:** Commit as `feat: persist a singleton spelling registry and source snapshots`.

### Task 6: Singleton creation, source selection and atomic backfill

**Files:** Create `crates/openrecall-core/src/spelling/collect.rs`, `crates/openrecall-core/src/services/spelling.rs`, `crates/openrecall-core/tests/spelling_collection.rs`, `crates/openrecall-core/tests/common/spelling.rs`; Modify `crates/openrecall-core/src/services/mod.rs`, `crates/openrecall-core/src/app.rs`, `crates/openrecall-core/src/contracts/{spelling,mod}.rs`, `crates/openrecall-core/src/spelling/storage.rs`, `crates/openrecall-core/tests/common/mod.rs`.

**Interfaces:** Produce `App::create_spelling_deck(&mut self, source_deck_id: String) -> AppResult<SpellingDeckResult>`, `get_spelling_config(&self, deck_id: &str) -> AppResult<SpellingConfig>`, `set_spelling_source(&mut self, deck_id: String, source_deck_id: String, expected_revision: String) -> AppResult<SpellingDeckResult>`, `rescan_spelling_source(&mut self, deck_id: String, expected_revision: String) -> AppResult<SpellingDeckResult>`. Collector `App::collect_spelling_content(&mut self, source_card: &CardDetail, collect_new: bool) -> AppResult<u32>` operates inside the caller's command savepoint. It creates managed rows internally, never through guarded public createCard.

- [ ] **Step 1:** Write `singleton_backfill_collects_all_fields_and_forms`, `source_switch_accumulates_without_resetting_words`, `stale_binding_and_failed_backfill_roll_back` and idempotent rescan/default-profile assertions.
  ```rust
  assert_eq!(word_keys(&mut f, &spelling), ["apple", "blue", "openai"]);
  assert_eq!(after_existing_word["dueAt"], before_existing_word["dueAt"]);
  assert_ne!(spelling_deck["profileId"], source_deck["profileId"]);
  assert_eq!(spelling_deck["name"], "مجموعة التدرب على الكتابة");
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test spelling_collection`; Expected: missing native operations/backfill FAIL.
- [ ] **Step 3:** Implement the four operations and Request/Response wiring. Require an ordinary existing source, enforce policy 1, use a separate default profile, scan all forms and create one managed front/back word form with language en. Share immutable snapshots by content fingerprint, create unique word/source edges and touch affected decks once per batch. Roll back profile/deck/word/context rows together on any error. A repeat create returns structured `spellingDeckExists` with existing-deck context.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `pnpm contracts`, `pnpm typecheck`; Expected: PASS, including reopen, empty source, invalid source/self-link, two create requests and format-2 round-trip.
- [ ] **Step 5:** Commit as `feat: create and backfill the additive spelling deck`.

### Task 7: Incremental collection and durable last matching contexts

**Files:** Modify `crates/openrecall-core/src/services/{cards,import,library,spelling}.rs`, `crates/openrecall-core/src/spelling/{collect,storage}.rs`; Create `crates/openrecall-core/tests/spelling_lifecycle.rs`.

**Interfaces:** Consume `collect_spelling_content` and produce `App::sync_spelling_for_content_change(&mut self, card: &CardDetail) -> AppResult<()>` and `App::retain_spelling_before_source_delete(&mut self, card_ids: &[String]) -> AppResult<()>`. The active source admits new words/origins; formerly bound forms refresh only their existing edges. Raw save_card/review/undo/replay do not collect.

- [ ] **Step 1:** Write `red_blue_edit_freezes_red_and_updates_blue`, `deleted_sources_leave_words_contexts_and_history`, `former_binding_updates_context_without_collecting_new_words` and RF1/RF2 cases with shared snapshots and a reserved spelling presentation.
  ```rust
  assert!(red_context["front"].as_str().unwrap().contains("red blue"));
  assert_eq!(blue_context["front"], "blue green");
  assert_eq!(reserved_after["presentationId"], reserved_before["presentationId"]);
  assert_eq!(word_after["revision"], word_before["revision"]);
  assert_eq!(word_after["memory"], word_before["memory"]);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test spelling_lifecycle`; Expected: missing synchronization/retention behavior FAIL.
- [ ] **Step 3:** Hook successful ordinary create/edit and import commit into collection in the same transaction; preview stays readonly. Refresh tracked contexts before live source/form removal, never remove word learning rows. Clear deleted active binding to None, retain labels and snapshots, and update compatible library revisions only. Index lookups for previously observed forms; garbage-collect only unreferenced snapshots. New words update spelling data revision, context-only changes do not invalidate ratings or training histories.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`; Expected: PASS for atomic injected failure, imports/duplicate skips, last-form removal, source-deck deletion, past sources, snapshot sharing and unchanged existing word due/undo.
- [ ] **Step 5:** Commit as `feat: retain spelling progress and context across source changes`.

### Task 8: Explicit deletion, durable exclusions and managed-deck guards

**Files:** Modify `crates/openrecall-core/src/services/{spelling,cards,library,import}.rs`, `crates/openrecall-core/src/contracts/{spelling,mod}.rs`, `crates/openrecall-core/src/spelling/storage.rs`; Create `crates/openrecall-core/tests/spelling_exclusions.rs`.

**Interfaces:** Produce `list_spelling_exclusions(&self, deck_id: &str, page: u32, page_size: u16) -> AppResult<Page<SpellingExclusion>>`, `allow_spelling_word(&mut self, deck_id: String, canonical: String, expected_revision: String) -> AppResult<SpellingDeckResult>`. Existing deleteCard/deleteDeck signatures remain; backend semantics depend on persisted kind.

- [ ] **Step 1:** Write `manual_delete_prevents_immediate_recollection`, `allow_word_readds_only_from_current_source_with_new_progress`, `managed_deck_rejects_generic_card_writes` and `normal_import_does_not_deduplicate_against_spelling_task`.
  ```rust
  assert_eq!(after_rescan_words.len(), before_delete_words.len() - 1);
  assert_eq!(readded["reviewCount"], 0);
  assert_eq!(create_error.code, "managedSpellingDeck");
  assert_eq!(ordinary_import["counts"]["validCards"], 1);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test spelling_exclusions`; Expected: recollection/guard/dedup behavior FAIL.
- [ ] **Step 3:** Register an exclusion atomically before explicit word deletion, clear incompatible spelling undo and prune only unused contexts. Allowing again clears the key and scans the active source; absent words remain absent until future collection, without invented progress recovery. Reject manual create/update/import into spelling. Normal JSON duplicate searches cover ordinary tasks only. Whole spelling-deck deletion removes its registry/contexts/exclusions; normal source deletion does not. Keep rename support and singleton identity independent of name.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `pnpm contracts`, `pnpm typecheck`; Expected: PASS, including source import after generated vocabulary, concurrent/stale exclusions and delete/recreate singleton.
- [ ] **Step 5:** Commit as `feat: manage spelling exclusions without source deletion coupling`.

### Task 9: Native spelling review mode and paginated retained sources

**Files:** Modify `crates/openrecall-core/src/contracts/{review,spelling,mod}.rs`, `crates/openrecall-core/src/services/{sessions,reviews,spelling}.rs`, generated DTOs and `src/test/reviewFixtures.ts`; Create `crates/openrecall-core/tests/spelling_review.rs`.

**Interfaces:** Add ReviewMode/can_rate as shared above. Produce `list_spelling_sources(&self, card_id: &str, page: u32, page_size: u16) -> AppResult<Page<SpellingSourceView>>`. Existing RateInput remains session/presentation/rating/action/expected-card-revision only.

- [ ] **Step 1:** Write `spelling_rates_without_reveal_and_normal_still_requires_it`, `context_refresh_during_rating_does_not_stale_word`, `ordinary_source_rating_does_not_mark_unchanged_context_as_changed`, and `retained_sources_page_after_every_live_source_is_gone`.
  ```rust
  assert_eq!(frame["mode"], "spelling");
  assert_eq!(frame["canRate"], true);
  assert_eq!(frame["answerRevealed"], false);
  assert!(frame["previews"].is_object());
  assert_eq!(normal_unrevealed_error.code, "answerHidden");
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test spelling_review`; Expected: mode/direct rating/previews FAIL; the normal unrevealed-card error remains `answerHidden`.
- [ ] **Step 3:** Determine mode from deck kind, authorize normal reveal versus direct spelling grades natively, return previews/due dates immediately for spelling and reject irrelevant reveal requests. Reuse the unchanged FSRS/state/action/undo transaction. Page contexts by first_seen_at, source_card_id, source_form_id. A live context is current when the form's front/back/notes/title/language match its snapshot; opaque card revisions, capture time or a renamed source deck alone do not mean its text changed. Missing forms are changed; missing cards are deleted. Do not require historical sources to exist, expose paths or accept typed input.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `pnpm contracts`, `pnpm typecheck`; Expected: PASS for independent learning, retries, summary undo, filters/live arrivals, restart, source deletion during an active spelling session and normal reveal guard.
- [ ] **Step 5:** Commit as `feat: review spelling cards with direct native self-ratings`.

### Task 10: Complete format-2 semantic validation and migration regressions

**Files:** Create `crates/openrecall-core/src/spelling/validate.rs`, `crates/openrecall-core/tests/spelling_backup.rs`; Modify `crates/openrecall-core/src/backup/{validation,legacy_v1}.rs`, `crates/openrecall-core/src/storage/backup.rs`, `src-tauri/tests/backup_commands.rs` and fixture provenance if needed.

**Interfaces:** Produce `spelling::validate::validate(data: &BackupData) -> AppResult<()>`; call it from full validation before writes. Validate source-word membership with the stored policy version and snapshot fingerprint; missing live sources are allowed, missing retained snapshots are not.

- [ ] **Step 1:** Write `full_spelling_backup_preserves_deleted_sources_and_active_undo`, `legacy_defaults_do_not_change_original_digest` (RF3), and table-driven recomputed-digest corruptions for duplicate singleton/word keys, wrong owner/kind/form, invalid source snapshot/hash/policy, orphan origin and active/excluded collision.
  ```rust
  assert_eq!(restored_word["memory"], before_word["memory"]);
  assert_eq!(retained_context["front"], "red blue");
  assert_eq!(corrupt_preview_error.code, "invalidBackup");
  assert_eq!(library_digest_after_rejection, library_digest_before);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test spelling_backup`; Expected: semantic corruption is accepted or legacy/nested-undo regression FAIL; each case must fail for the intended invariant.
- [ ] **Step 3:** Enforce all registry/snapshot/exclusion ownership and kind invariants, policy-1 extraction proof and one managed form, while retaining existing FSRS/history/session checks. Validate indexed/payload agreement on import/migration. Repair replacement order and preserve source-format preview metadata, safety export, active-job interruption and revision checks. No scratchpad field exists in any backup type.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `cargo test -p openrecall-core --features serde_json/preserve_order --test backups`, `cargo test -p openrecall-core --features serde_json/preserve_order --test spelling_backup`, and `cargo test -p openrecall-desktop --test backup_commands`; Expected: PASS including disk failure and genuine legacy fixture.
- [ ] **Step 5:** Commit as `feat: validate portable spelling data and legacy upgrades`.

### Task 11: Singleton setup, source settings and generated word management UI

**Files:** Create `src/features/spelling/{SpellingSetup,SpellingSettings,ExcludedWords}.tsx`, `src/features/spelling/{SpellingSetup,SpellingSettings,ExcludedWords}.test.tsx`, `src/test/spellingFixtures.ts`; Modify `src/features/library/{LibraryPage,LibraryPage.test}.tsx`, `src/features/cards/{DeckPage,CardList}.tsx`, `src/test/mockApi.ts`, `src/shared/errors.ts`.

**Interfaces:** `SpellingSetup({api, sources: DeckDetail[], onCreated(result: SpellingDeckResult), onCancel})`; `SpellingSettings({api, deck: DeckDetail, onChanged(): Promise<void>})`; `ExcludedWords({api, config: SpellingConfig, onChanged(): Promise<void>})`. Consume the Task 6/8 commands; reuse existing Api and navigation types.

- [ ] **Step 1:** Test singleton create/open, no-source explanation, source rebinding/rescan counts, deleted-source status, explicit word/group deletion and allow-again. Test an unsaved source-selector choice surviving unrelated context/deck updates and a late create response respecting current navigation/drafts.
  ```tsx
  expect(screen.queryByRole('button', { name: 'إنشاء مجموعة التدرب على الكتابة' })).not.toBeInTheDocument();
  expect(screen.getByLabelText('المجموعة المصدر')).toHaveValue(chosenSourceId);
  expect(screen.queryByRole('button', { name: 'استيراد بطاقات JSON' })).not.toBeInTheDocument();
  ```
- [ ] **Step 2:** Run `pnpm test -- src/features/spelling src/features/library/LibraryPage.test.tsx`; Expected: new controls/guards FAIL.
- [ ] **Step 3:** Integrate the singleton action and typed setup dialog, badge, source settings and excluded list, all collapsed by default except primary actions. Use busy/dirty/freshness guards, preserve normal deck actions and show generated cards without manual front/back editing. Explain that source deletion keeps word/context copies and explicit spelling deletion removes that progress.
- [ ] **Step 4:** Run `pnpm test`, `pnpm typecheck`, `pnpm lint`; Expected: PASS for setup, focus/error handling, source-draft retention and ordinary decks.
- [ ] **Step 5:** Commit as `feat: add accessible spelling deck setup and source controls`.

### Task 12: Disposable spelling textarea, source disclosure and keyboard flow

**Files:** Create `src/features/spelling/{SpellingQuestion,SourceContexts}.tsx`, `src/features/spelling/{SpellingQuestion,SourceContexts}.test.tsx`; Modify `src/features/review/{ReviewPage,ReviewPage.test}.tsx`, `src/features/review/{useReviewSession,useReviewSession.test}.ts`, `src/test/spellingFixtures.ts`, `src/styles/global.css`.

**Interfaces:** `SpellingQuestion({api, frame: ReviewFrame, onRate(rating: Rating): void, disabled: boolean, timezone: string})`; `SourceContexts({api, cardId: string})`. ReviewPage mounts spelling question keyed by session/presentation; useReviewSession checks native canRate. No text value, onChange draft, input ref read or scratchpad cache is introduced.

- [ ] **Step 1:** Test an uncontrolled field labelled `مساحة الكتابة`, H1 `الكلمة`, full English text node, immediate interval-labelled grades and collapsed `عرض البطاقة الأصلية`. RF4 tests type a synthetic marker: blur/context/poll/failed grade retain the same field; advance, summary undo, filter parking/back, navigation/resume and remount return empty. Same-card filter changes that do not park it retain the field.
  ```tsx
  expect(screen.getByRole('textbox', { name: 'مساحة الكتابة' })).toHaveValue('private-attempt-marker');
  await user.click(screen.getByRole('button', { name: 'التراجع عن آخر تقييم' }));
  expect(screen.getByRole('textbox', { name: 'مساحة الكتابة' })).toHaveValue('');
  expect(JSON.stringify(capturedRequests)).not.toContain('private-attempt-marker');
  ```
- [ ] **Step 2:** Run `pnpm test -- src/features/spelling src/features/review`; Expected: direct spelling flow/disposable input behavior FAIL.
- [ ] **Step 3:** Render a native textarea with no application copies of text/selection and disable spellcheck/autocorrect/autocomplete/autocapitalization. Do not clear on blur or remount on poll. Focus label H1, never the word; keep the word out of live announcements. Rate without reveal, honor text-editing shortcuts inside input and grade shortcuts only outside editing controls. Load/paginate plain saved contexts on disclosure with stale-response guards; never let a previous word's pending source response populate the next word.
- [ ] **Step 4:** Run `pnpm test`, `pnpm typecheck`, `pnpm lint`; Expected: PASS for all transition/privacy cases and ordinary review. Confirm no practice text enters persistent/browser storage and no field-content-based grading branch exists.
- [ ] **Step 5:** Commit as `feat: provide disposable accessible spelling practice input`.

### Task 13: Calendar-based completion counts and library aggregation

**Files:** Create `crates/openrecall-core/src/statistics/{mod,completion}.rs`, `crates/openrecall-core/tests/completion_statistics.rs`; Modify `crates/openrecall-core/src/lib.rs`, `crates/openrecall-core/src/services/statistics.rs`, `crates/openrecall-core/src/contracts/{statistics,mod}.rs`, `crates/openrecall-core/src/app.rs`, generated contracts and affected statistics fixtures.

**Interfaces:** Produce `statistics::completion::daily_completions(logs: &[ReviewLog], zone: chrono_tz::Tz, start_minute: u16) -> AppResult<BTreeMap<chrono::NaiveDate, u32>>` and `App::library_statistics(&self) -> AppResult<LibraryStatistics>`. Logs arrive in database insertion order; choose the latest reviewed_at per card/day, breaking equal instants by later input order (not random UUID). Per-deck/global queries preserve `at_ms,rowid` ordering and surviving-history semantics.

- [ ] **Step 1:** Write `completion_is_last_effective_review_per_card_and_study_day`, `next_day_review_does_not_erase_yesterday` (RF5), `undo_and_deletion_recompute_only_surviving_history`, and global spelling-subset cases. Pure tests construct valid synthetic ReviewLog values, independent of persisted-card consistency.
  ```rust
  // Cairo winter: 00:59:30 -> 01:00:30 crosses the configured 01:00 boundary in one minute.
  assert_eq!(completion("2026-01-15T22:59:30Z", "2026-01-15T23:00:30Z"), 1);
  assert_eq!(completion("2026-01-15T22:58:00Z", "2026-01-15T22:59:00Z"), 0);
  assert_eq!(unreviewed_future_card_count, 0);
  assert_eq!(counts_after_tomorrows_review[&yesterday], 1);
  ```
- [ ] **Step 2:** Run `cargo test -p openrecall-core --test completion_statistics`; Expected: absent/wrong calendar count FAIL.
- [ ] **Step 3:** Compute counts from effective logs and `study_date` for both reviewed and next-due instants. Count each card/day once, including Again crossing the boundary; aggregate windows as sums of daily counts. Add completed_for_day to existing periods/history and the new library command with the spelling subset. Keep scheduling graduation and saved due instants unchanged. Use current settings consistently, including timezone rebucketing and DST.
- [ ] **Step 4:** Run `cargo test -p openrecall-core`, `pnpm contracts`, `pnpm typecheck`; Expected: PASS for 01:00 boundary, Cairo DST, equal timestamps, repeated forms, undone last review, historical counts and normal/spelling double-task totals.
- [ ] **Step 5:** Commit as `feat: calculate completed-for-today counts from review history`.

### Task 14: Per-deck and collapsed global completion statistics UI

**Files:** Modify `src/features/statistics/{DeckStatistics,DeckStatistics.test}.tsx`, `src/features/library/{LibraryPage,LibraryPage.test}.tsx`, `src/features/cards/DeckPage.tsx`; Create `src/features/statistics/{LibraryStatistics,LibraryStatistics.test}.tsx`; update typed mocks.

**Interfaces:** `LibraryStatistics({api, revision: string, timezone: string})`; consume Task 13's DTOs. Extend existing DeckStatistics props with `deckKind?: DeckKind` defaulting to standard and pass `deck.kind` from DeckPage. Label units from deck kind without extracting words in React.

- [ ] **Step 1:** Test lazy collapsed global loading, today/per-day totals, separate spelling-word subset, per-deck unit labels and a rollover refetch from native results rather than browser day calculations.
  ```tsx
  expect(screen.getByText('انتهت مراجعاتها اليوم')).toBeVisible();
  expect(screen.getByText('منها كلمات الإملاء')).toBeVisible();
  expect(globalStatsCallsBeforeExpansion).toBe(0);
  ```
- [ ] **Step 2:** Run `pnpm test -- src/features/statistics src/features/library/LibraryPage.test.tsx`; Expected: missing counter/history/global disclosure FAIL.
- [ ] **Step 3:** Add completion values to current deck statistics/daily table and the collapsed library section. Explain ordinary logical-card versus spelling-word units and that total counts separate tasks. Page daily history at 20 rows; fetch on expansion, explicit refresh, visible focus and once per minute while expanded, rejecting stale responses. Rust remains the day-boundary authority. Do not eagerly load every deck's training dataset for global counts.
- [ ] **Step 4:** Run `pnpm test`, `pnpm typecheck`, `pnpm lint`; Expected: PASS for empty/error states, pagination, day rollover and existing card/statistics disclosures.
- [ ] **Step 5:** Commit as `feat: show accessible daily completion totals`.

### Task 15: Browser/native privacy, migration and accessibility acceptance

**Files:** Create `tests/{e2e,native}/spelling.spec.ts`; Modify `tests/e2e/harness.tsx`, native fixtures/helpers/runner only as required, `docs/testing/{automated-results,human-nvda-checklist}.md`, `docs/user-guide.ar.md`, `README.md`.

**Interfaces:** Reuse the existing contracted browser adapter, genuine isolated native runner, rejecting proxy, feature-specific automation executable and axe assertions. Preserve any required v0.1.0 synthetic fixture before building over automation outputs; never launch a production binary against real user data.

- [ ] **Step 1:** Add browser and real-native scenarios: singleton setup/backfill; import old JSON/custom titles; direct typing/self-rating/filter/undo; delete every source then inspect saved contexts; source changes during a reserved question; explicit exclusion/reallow; global completion counters; format-2 export/delete/restore and legacy-format import. Use a unique typed marker and assert it is absent from native backup/exported review data. Restart must resume learning with empty input. Axe audits require nonzero passes and zero violations for each important new state.
- [ ] **Step 2:** Run `pnpm test:e2e`; Expected: all new scenarios exercise actual frontend behavior. Any exposed failure enters systematic-debugging and gets an observed RED-to-GREEN regression. Build/copy automation with `pnpm build:native-test`, then run `pnpm test:native`; preserve the feature-specific copy before workspace test commands can replace target/debug binaries.
- [ ] **Step 3:** Complete integration repairs and documentation, including exact extraction examples, source-copy retention, explicit deletion/exclusions, questionTitle schema and the disposable-input exception to UI undo. Human checklist covers manual reading, typing echo/caret, source disclosure and empty input on return; do not run NVDA. Preserve offline evidence and distinguish browser mocks from native core behavior.
- [ ] **Step 4:** Run `pnpm verify`, `pnpm build:native-test`, `pnpm test:native`; Expected: PASS with actual recorded counts and privacy/migration/context evidence, no production automation leak. Do not run native build and a Cargo verifier concurrently against shared outputs.
- [ ] **Step 5:** Commit as `test: cover spelling study privacy and retained contexts`.

### Task 16: Versioned production packaging and release readiness

**Files:** Modify `Cargo.toml`, `Cargo.lock`, `package.json`, `src-tauri/tauri.conf.json`, license manifest/notices as version changes require; Create `docs/releases/{0.2.0.md,0.2.0.ar.md}`; update verification/signing documentation. Modify a maintenance test only if an actual packaging regression requires it, not to mirror a version literal.

**Interfaces:** Reuse version-parameterized release/preflight/staging/manifest/publisher scripts. Keep the existing production identifier/public key, offline installer-only override and trusted workflow policy. No new signing secret is needed.

- [ ] **Step 1:** Run `node --test tests/maintenance/release_pipeline.test.mjs tests/maintenance/update_manifest.test.mjs tests/maintenance/package_config.test.mjs`; Expected: PASS for existing version-consistency, four-target and production-capability guards. This routine configuration/version bump does not need a new test that mirrors `0.2.0`. An actual packaging defect still requires an observed regression before repair.
- [ ] **Step 2:** Update all three product version sources/lockfile to `0.2.0`, release notes and license graph; check the three actual values agree. Record the existing Linux GLib advisory status without suppressing the alert or claiming a patch that is not present.
- [ ] **Step 3:** Compile production without signing credentials, then use official `tauri bundle` to sign normal/offline Windows installers from the same binary. Load credentials only from the existing protected external files for bundling and remove process env afterward. Preserve normal output before offline overwrites its basename.
- [ ] **Step 4:** Run `pnpm verify` and official `verify_update` for both local installers with version `0.2.0`; Expected: PASS. Verify production override-marker absence and preserve delivery output outside any checkout intended for cleanup. A build/signature result is not an OS installer run or real older-to-newer installation result.
- [ ] **Step 5:** Commit as `release: prepare verified OpenRecall 0.2.0 packages`.

### Task 17: Whole-branch review, protected merge and public signed release

**Files:** Change only code/tests/docs needed for verified findings or actual CI/release failures; record evidence in `docs/testing/automated-results.md`, release notes and the executing-plans ledger.

**Interfaces:** Use existing repository `wip-bunker-42/OpenRecall`, required checks `quality`, `native-windows`, `bundle-windows`, `bundle-linux`, `bundle-macos-arm64`, `bundle-macos-x64`, and release tag `v0.2.0`.

- [ ] **Step 1:** Invoke requesting-code-review with a fresh GPT-6.1 Sol whole-branch reviewer, spec, full plan, branch base, actual verification and every ruling. Include the five Review Focus lines verbatim. Regrade by user effect, apply Important/Critical findings through behavioral RED-to-GREEN in one fix pass and record deferred minors. No automatic re-review loop.
- [ ] **Step 2:** Run the required full local verification after fixes and appropriate native tests. Scan tracked changes for private keys and real user data. Expected: green suites, preserved FSRS goldens, complete format-1/2 compatibility and no leaked practice text. Commit fixes and push the branch. Create the feature PR with exact evidence/limits and immediately attach it to this chat.
- [ ] **Step 3:** Follow finishing-a-development-branch using the user's existing explicit authorization to push, merge and release. Wait for the six actual required exact-head checks; repair actual failures instead of bypassing them. Merge only after the review/fix gate and required checks pass. Verify actual app code on main and all six checks on the exact release commit.
- [ ] **Step 4:** Confirm `v0.2.0` is unused, create/push its annotated tag at the checked main commit and follow the signed Release workflow. Repair/retry publication through owned-draft recovery without rewriting published tags/assets. Verify 15 public assets, four updater targets, normal/offline Windows installers, official signatures/signed versions, server/local/SHA256SUMS digests and anonymous access to every asset plus latest.json. Record actual release/CI links and source/policy SHAs.
- [ ] **Step 5:** Complete the task's final `pnpm verify` gate, copy/hash-verify installers and evidence to the primary checkout's `target/delivery/v0.2.0`, retain earlier delivery files, and deliver repository/PR/release/installer/checklist links. Collect every ruling with its cost and all deferred minors before removing only this plan's scratch. Use native archival only when allowed; if the task checkout is protected, preserve it and report the limit rather than forcing removal.

## Plan Self-Review and Coverage

- Singleton/configuration/collection and unknown-word policy: Tasks 4-8 and 11.
- Independent FSRS, existing guards/undo, native sources and disposable input: Tasks 9, 12 and 15.
- Source switching, last matching context, historical IDs and shared snapshot retention: Tasks 5-7, 10 and 15.
- JSON/editor headings and backward compatibility: Tasks 1-3, 10 and 15.
- Per-deck/global calendar statistics, history and rollover: Tasks 13-15.
- Format-1 checksum-before-defaults, format-2 registry integrity and safety restore: Tasks 1, 5, 10 and 15.
- Accessible Arabic offline UI, automated tests and human-only reader boundary: Tasks 3, 11-15.
- Version/signing/production isolation/protected public release and delivery: Tasks 16-17.

Self-review completed on 2026-10-29 against the approved spec: checked every
section's task mapping, action/test/expected-result steps, shared types and
producer-before-consumer ordering, all five Review Focus regressions and document
proportion. Corrected the existing `answerHidden` error, native method receivers,
kind writes in save/restore, per-deck kind prop, exact source/test paths, hook
extension, installed license-tool path and the routine version-bump verification
boundary. No unresolved planning gaps remain. Product implementation is blocked
until the user reviews this saved plan. Native execution is the preserved method;
no new execution-method selection is required.
