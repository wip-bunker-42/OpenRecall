# Essay and Multiple-Choice Questions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add question-only essays with copy and immediate manual grading, and locally checked MCQ with automatic Again for wrong or unknown answers.

**Architecture:** Each form carries its question type while alternatives retain one card's learning state. Rust validates choices, owns the presentation order, and uses the existing grade transaction for MCQ failures. Persist committed feedback until Continue, with revision checks and before-images for undo.

**Tech Stack:** Rust 1.99 toolchain, Rust 1.90 minimum, SQLite, pinned FSRS-7, Tauri 2, React, TypeScript, pnpm, Vitest, Playwright, WebdriverIO.

**Spec:** `docs/superpowers/specs/2026-10-30-question-types-design.md`, approved 2026-11-13. Implementation base `1e1639d3900e764e33d5299137bcbdc491508b1a`; released product baseline `ffd7a80bc58fea2465a533e38558741d670bba7c`.

## Global Constraints

- Keep Arabic dark RTL UI, offline study, accessible native controls, the existing FSRS-7 engine and one shared learning state for every logical card's alternatives.
- Rust is authoritative for scheduling, form selection, validation, grading, training and transactions. Preserve FSRS revision `0fd59949e9168af7194418c2dba99836b5548e87` and all three memory values.
- Grade atomically with action-token retry safety and complete one-step undo. Correct MCQ confirmation does not grade. Incorrect and unknown confirmation each commit Again immediately.
- Use schema 10 and supported backup tuples `(1,8)`, `(2,9)`, `(2,10)`. Verify the original checksum before upgrading defaults. Freeze both released fixtures.
- Imported and copied content is plain text. Keep the 160-character title limit, 50 MiB import limit, existing alternative-form limits and explicit signed update policy.
- Use isolated synthetic libraries. Never launch, operate or configure NVDA, Narrator or another screen reader.
- Generate TypeScript contracts with `pnpm contracts`. Keep product changes focused and run `pnpm verify` and Windows native acceptance.
- Keep credentials outside Git and the checkout. Production excludes automation and native test overrides. Distinguish runtime, build and human acceptance evidence.
- Continue the previously selected Native execution method. The isolated `abdo/essay-multiple-choice` worktree is ready. Verify the baseline before product edits. Preserve the existing protected merge and signed release pipeline.

## Review Focus

1. Leaving a wrong-answer correction through the actual navigation route should preserve its review, correction and linked undo on resume (Task 6).
2. A more urgent card can arrive during correction. Continue should reserve that card and undo should restore its actual pre-reservation bag (Task 6).
3. Editing the same form ID to another question type while a check is in flight must reject the old check without history writes (Task 6).
4. `US` and `us` can be meaningful distinct choices. Reordering and renaming import keys must still detect the same question as a duplicate (Task 3).
5. A word formerly correct and now only a distractor should keep its last eligible context and progress, while a new distractor-only token stays out of spelling collection (Task 7).

---

## File and contract boundaries

- `crates/openrecall-core/src/domain/questions.rs` owns typed question content and normalization. `contracts/cards.rs` owns editor drafts. `services/cards.rs` resolves ownership and saves forms. `services/import.rs` parses portable keys, deduplicates and feeds the same draft validation.
- `domain/session.rs` stores choice reservations and feedback. `services/sessions.rs` owns live selection and resume. `services/choice_reviews.rs` validates submission and continues feedback. `services/reviews.rs` shares the existing grade transaction and undo. `domain/review.rs` owns persisted action metadata.
- `backup/{decode,legacy_v2,validation}.rs` handles released shapes and current invariants. `spelling/{collect,validate}.rs` shares typed source projection and snapshot identity. `services/spelling.rs` compares live content to retained snapshots.
- `services/question_copy.rs` resolves the current prompt. `src-tauri/src/question_copy.rs` writes its text through Tauri. `src/shared/api.ts` exposes only the native copy action.
- `CardEditor.tsx` and `ChoiceEditor.tsx` edit forms. `ImportPreview.tsx`, `CardList.tsx` and `SourceContexts.tsx` render authored content. `ReviewPage.tsx` selects the presentation component. `useReviewSession.ts` serializes foreground, retry and polling requests.

### Task 1: Freeze and read the released v0.2.0 backup

**Files:** `crates/openrecall-core/tests/fixtures/backups/{v0.2.0.json,README.md}`, `crates/openrecall-core/tests/legacy_backup_v2.rs`, `crates/openrecall-core/src/backup/{mod,decode,legacy_v2}.rs`.

**Interfaces:** Use `Fixture::{new,call}` and `App::export_snapshot`. Produce `legacy_v2::upgrade(Value) -> AppResult<BackupData>` for `(2,9)`. Reuse immutable wire types from `legacy_v1` only where their complete shape is identical.

- [ ] **Baseline:** In the isolated branch, check `git diff ffd7a80 -- crates src src-tauri tests Cargo.lock pnpm-lock.yaml` is empty, then run `pnpm verify`. Expect `OpenRecall verification passed.` Resolve baseline problems before changing production files.
- [ ] **Capture:** With released product code, use a temporary integration test and `Fixture` to create a titled source with alternatives, collect spelling words, exclude a word, and grade another word while keeping its direct-grade undo. Export `(2,9)` with the released writer. Record fixture provenance and SHA-256. Remove the temporary capture test and preserve the v0.1.0 fixture byte-for-byte.
- [ ] **Write test:** `released_v2_preserves_spelling_and_undo` checks the original digest, exact source fields, exclusions, `stabilityFast` and restored `undoReview` returning to the original spelling presentation. `released_v2_rejects_rehashed_new_nested_fields` injects `questionType: "essay"` in an ordinary nested form and `choiceFeedback` in an undo session. Canonicalize and recompute the checksum for each. Assert restore preview rejects and the target digest is unchanged.
- [ ] **RED:** `cargo test -p openrecall-core --test legacy_backup_v2`. Expect acceptance of a rehashed unknown nested field in the released reader.
- [ ] **Implement:** Freeze all `(2,9)` nested wire shapes, including spelling registry and actions. Validate their shape before converting to current DTOs. Authenticate raw source JSON before dispatch. Keep `(1,8)` routing and reject unsupported tuples.
- [ ] **GREEN:** `cargo test -p openrecall-core --test legacy_backup --test legacy_backup_v2 --test spelling_backup`. Repeat with `--features serde_json/preserve_order`. All pass and both fixture hashes match.
- [ ] **Commit:** `test: freeze released schema-nine backup compatibility`.

### Task 2: Validate typed forms and upgrade the database

**Files:** Create `crates/openrecall-core/src/domain/questions.rs`, `crates/openrecall-core/tests/typed_questions.rs`. Modify `domain/{mod,card,spelling}.rs`, `contracts/cards.rs`, `services/{cards,import,spelling}.rs`, `storage/mod.rs`, `backup/{decode,validation}.rs`, `spelling/{collect,validate}.rs` under `crates/openrecall-core/src`. Update `crates/openrecall-core/tests/{spelling_schema,legacy_backup}.rs`, generated contracts and affected `src/test`/`tests/e2e/harness.tsx` fixture literals.

**Interfaces:** `QuestionType::{Recall,Essay,MultipleChoice}`, `SelectionMode::{Single,Multiple}`, `Choice { id, text }`, `ChoiceSet { choices, correct_choice_ids, selection_mode, shuffle_choices }`. `CardForm` adds `question_type: QuestionType` and `multiple_choice: Option<ChoiceSet>`. `FormDraft` adds the same kind and `multiple_choice: Option<ChoiceSetDraft>`. The choice draft set has `choices: Vec<ChoiceDraft>`, `correct_choice_keys: Vec<String>`, `selection_mode: SelectionMode` and `shuffle_choices: bool`. `ChoiceDraft { id: Option<String>, key: String, text: String }` maps portable keys to native IDs. `SourceSnapshot` adds `content_version: u16` (default 1) and typed authored content. `eligible_texts(&CardForm) -> Vec<&str>` and `snapshot_eligible_texts(&SourceSnapshot) -> Vec<&str>` share projection.

- [ ] **Write test:** Through `Fixture::call`, assert an essay with `front:"Explain"` and no `back` round-trips with empty back. A legacy form with no back, an essay with `back:"hidden"`, and recall with choice fields each fail. Assert `multipleChoice` with `US/us` creates distinct UUIDs, editing preserves an owned UUID, and a foreign UUID, identical normalized options, empty/duplicate key, invalid key reference or two correct choices in single mode fails. Assert managed spelling rejects manual writes and typed variants retain one card ID.
- [ ] **Write migration test:** Reconstruct a schema-9 SQLite library from the frozen v0.2.0 payload and released SQL. Open it and assert `user_version == 10`, byte-equal existing card/session/action payloads before normal writes, recall defaults on read, unchanged due/memory/bags and snapshot hashes, and genuine undo. Update the schema-8 migration expectation to 10 with the same byte-preservation assertions.
- [ ] **Write collection test:** In an ordinary deck already bound to spelling, create an essay and an MCQ. Assert both can export and restore, `green` in a correct choice collects, and `wrongword` found only in a distractor is absent. Assert a frozen recall snapshot still verifies with its original hash.

  ```rust
  assert_eq!(essay["forms"][0]["questionType"], "essay");
  assert_eq!(essay["forms"][0]["back"], "");
  assert_eq!(schema_after_open, 10);
  assert!(collected_words.contains("green"));
  assert!(!collected_words.contains("wrongword"));
  ```

- [ ] **RED:** `cargo test -p openrecall-core --test typed_questions --test spelling_schema`. Expect essay rejection and a schema-9 result.
- [ ] **Implement:** In `questions.rs`, add `validate_choice_draft(&ChoiceSetDraft) -> AppResult<()>`, `resolve_choices(ChoiceSetDraft, Option<&ChoiceSet>) -> AppResult<ChoiceSet>` and `validate_form(&CardForm) -> AppResult<()>`. Enforce NFC/whitespace comparison with case preserved, 2–128 choices, at most 16 KiB UTF-8 per choice and 128 bytes per portable key. Keep recalled back required and reject incompatible fields. Use lazy serde defaults for existing JSON rows, gate schema 10 transactionally and decode `(2,10)` as current. Initialize managed spelling forms as recall and validate that invariant. Keep legacy exports readable.
- [ ] **Connect typed collection in this commit:** Keep exact v1 recall snapshot hash and default shape. Use v2 domain-separated hashes for essay/MCQ with authored choices and key. Apply the same typed projection in collection and restore, and compare normalized typed content when reporting source availability. This keeps a bound spelling deck exportable as soon as typed creation becomes available. Update legacy equality assertions only for explicit newly serialized defaults.
- [ ] **GREEN:** `cargo test -p openrecall-core --test typed_questions --test spelling_schema --test legacy_backup_v2`; `pnpm contracts`; `pnpm typecheck`. All pass.
- [ ] **Commit:** `feat: add typed question forms and schema ten`.

### Task 3: Import, deduplicate and search typed questions

**Files:** `crates/openrecall-core/src/{contracts/import.rs,services/import.rs,services/cards.rs}`, `crates/openrecall-core/tests/typed_import.rs`, generated contracts and affected preview fixtures.

**Interfaces:** `ImportForm` carries `question_type` and optional `ChoiceSetDraft`. `form_fingerprint(&CardForm) -> String` and the preview equivalent share type-specific identity. Legacy `fingerprint(front,back)` keeps its bytes.

- [ ] **Write test:** `typed_preview_commit_resolves_keys` imports the essay and Paris/London examples from the spec, with an inherited MCQ variant and an explicitly essay variant. Assert preview type/key/errors, exclusion of a malformed variant, stored native choice ownership, and search by `London`. Omitted legacy type plus empty back remains invalid. A variant missing its own choices/key fails even with a valid primary.
- [ ] **Write Review Focus 4 test:** Store `US/us` with `US` correct. Preview reordered options with renamed local keys and changed notes/shuffle. Assert duplicate. Change correct choice to `us` and assert distinct. Assert two NFC/whitespace-equivalent options in a single form are invalid.

  ```rust
  assert_eq!(reordered["cards"][0]["primary"]["duplicate"], true);
  assert_eq!(changed_key["cards"][0]["primary"]["duplicate"], false);
  assert_eq!(missing_variant_key["cards"][0]["variants"][0]["valid"], false);
  ```

- [ ] **RED:** `cargo test -p openrecall-core --test typed_import`. Expect unknown typed fields, rejected essay or missing duplicate recognition.
- [ ] **Implement:** Parse the spec's `questionType`, `choices`, `correctChoices`, `selectionMode` and `shuffleChoices`. Inherit only type and existing title behavior. Use Rust validation and order-independent choice/correctness pairs. Keep preview revision/selection guards and native transaction, and match authored choice text in `listCards`.
- [ ] **GREEN:** `cargo test -p openrecall-core --test typed_import --test json_import --test question_titles`; `pnpm contracts`; `pnpm typecheck`. All pass.
- [ ] **Commit:** `feat: import and search typed questions`.

### Task 4: Enable direct essay grading in Rust

**Files:** `crates/openrecall-core/src/services/{sessions,reviews}.rs`, `crates/openrecall-core/src/backup/validation.rs`, `crates/openrecall-core/tests/essay_review.rs`.

**Interfaces:** `manual_grade_allowed(DeckKind, &CardForm, &Presentation) -> bool` is shared by view capability, rate validation and undo validation. At this stage essay/spelling allow direct grading, recall requires reveal and unconfirmed MCQ denies grading.

- [ ] **Write test:** With an essay frame, assert `canRate == true`, `answerRevealed == false`, four previews and native reveal rejection. Apply `good` twice with the same token, assert one review, then export/restore and undo. Assert the original essay, due and presentation return. Assert the same unrevealed `rateCard` on recall and MCQ is rejected.
- [ ] **RED:** `cargo test -p openrecall-core --test essay_review`. Expect `canRate == false` or hidden-answer rejection.
- [ ] **Implement:** Resolve the selected form and current card revision in Rust for reveal and rating eligibility. Apply the same predicate to saved undo before-images. Keep the existing scheduler and rate input.
- [ ] **GREEN:** `cargo test -p openrecall-core --test essay_review --test review_transactions --test spelling_review --test legacy_backup_v2`. All pass.
- [ ] **Commit:** `feat: rate question-only essays directly`.

### Task 5: Reserve durable choice order per presentation

**Files:** `crates/openrecall-core/src/{domain/card.rs,domain/session.rs,contracts/review.rs,services/sessions.rs,services/reviews.rs,services/cards.rs,backup/validation.rs,spelling/collect.rs}`, `crates/openrecall-core/tests/choice_reservations.rs`, generated contracts and fixtures.

**Interfaces:** `CardDetail.previous_choice_orders: BTreeMap<String, Vec<String>>` keyed by form. `Presentation.choices: Option<ChoicePresentation>` stores `order`, `selected_ids` and `confirmed_correct`; the last two default to empty/false. `ReviewFrame.choices` exposes that reservation. `reserve_choice_order(&ChoiceSet, Option<&[String]>, &mut dyn RandomSource) -> Vec<String>` uses injected randomness.

- [ ] **Write test:** A deterministic random source returning the previous permutation must still give a different valid permutation on a new graded presentation. With shuffle off, assert authored order exactly. Assert stable IDs/order after repeated reads, filter parking and process reopen. After edits to choice IDs or type, assert the old order is reconciled and selection/reveal cleared. Assert v0.2.0 restored forms have no choice state.
- [ ] **RED:** `cargo test -p openrecall-core --test choice_reservations`. Expect missing order and no durable choice state.
- [ ] **Implement:** Reserve once after form/revision resolution. Shuffle once, rotating one position only if it repeats a valid previous order. Keep previous-order history as part of card state written only on grade. Discard stale orders on content edits. Allow older rows to deserialize with empty state. Validate valid permutations and choice ownership in active, parked and undo snapshots.
- [ ] **GREEN:** `cargo test -p openrecall-core --test choice_reservations --test form_bags --test form_edit_backups`; `pnpm contracts`; `pnpm typecheck`. All pass.
- [ ] **Commit:** `feat: persist multiple-choice presentation order`.

### Task 6: Confirm choices and commit failure feedback atomically

**Files:** Create `crates/openrecall-core/src/services/choice_reviews.rs`, `crates/openrecall-core/tests/{choice_review,choice_backup}.rs`. Modify core `app.rs`, `contracts/{mod,review}.rs`, `domain/{session,review}.rs`, `services/{mod,sessions,reviews,cards}.rs`, `backup/validation.rs`, generated contracts and exhaustive view fixture consumers.

**Interfaces:** `ChoiceSubmission::{Selected { choice_ids }, Unknown}`. `ConfirmChoicesInput { session_id, presentation_id, expected_card_revision, action_id, submission }`. `confirm_choices(&mut self, ConfirmChoicesInput) -> AppResult<ReviewView>` and `continue_choice_feedback(&mut self, session_id: String, feedback_id: String) -> AppResult<ReviewView>`. Add `ReviewView::ChoiceFeedback(Box<ChoiceFeedbackFrame>)` with common filter/progress/next-due/last-review/undo fields and the persisted result, authored form/order and selected/missed/incorrect IDs. `Session.choice_feedback` stores a matching review ID and action ID. Shared `apply_rating(&mut self, Session, CardDetail, Rating, ActionRecord, GradeDestination) -> AppResult<ReviewView>` accepts `Advance` or `Feedback(ChoiceSubmission)`. Keep legacy rate signature bytes. New confirmation signatures identify their operation and canonical input, with stored `ChoiceOutcome` metadata (`correct`, `incorrect`, `unknown`). `UndoRecord.deferred_card_befores: Vec<CardDetail>` defaults empty and captures additional reservation mutations after a feedback grade.

- [ ] **Write test:** Exact multiple set `[A,C]` confirms correct in either order with `reviewCount == 0` and four previews. `[A]`, `[A,B,C]`, a wrong single option and `Unknown` each commit one `again`; duplicate/foreign IDs and empty selected lists reject. Confirm the wrong view has actual due and no current question. Assert `rateCard(good)` before check or on committed feedback and `revealAnswer` on MCQ reject. After correct confirmation, manual grade undo restores the locked correct result.

  ```rust
  assert_eq!(correct_view["payload"]["canRate"], true);
  assert_eq!(card_after_correct["reviewCount"], 0);
  assert_eq!(wrong_view["state"], "choiceFeedback");
  assert_eq!(wrong_view["payload"]["lastReview"]["rating"], "again");
  assert_eq!(card_after_wrong["reviewCount"], 1);
  assert_eq!(wrong_retry["payload"]["feedbackId"], wrong_view["payload"]["feedbackId"]);
  ```

- [ ] **Write retry/rollback test:** Lose a wrong confirmation response, repeat its token, and compare one history entry and the same feedback ID. Reject changed selection, cross-operation reuse and the undone token. Inject a SQLite failure on undo insertion after the grade starts. Assert card, actions, history, feedback and session digest roll back together. Undo of wrong restores unconfirmed selected IDs and exact order; undo of unknown restores an empty selection.
- [ ] **Write Review Focus 1–3 tests:** Wrong → `closeSession` → reopen → `resumeSession` keeps correction and usable undo. During feedback change filter and create a more urgent matching card. Continue must select it, and undo must restore that card's pre-reservation bag and the prior MCQ. Submit an old check after changing the same form ID from MCQ to essay and assert revision conflict with zero new reviews. Duplicate/stale Continue cannot dismiss later feedback. Test last-card feedback, summary undo, deletion of a graded card and end-with-feedback.
- [ ] **Write backup test:** Round-trip active wrong feedback and confirmed correct state with undo. Recompute digest after altering permutation, answer key/judgment, review rating, feedback link or operation signature, and assert rejection without a write. Accept a correct-confirmation action without a grade log, and edited/deleted historical option IDs.
- [ ] **RED:** `cargo test -p openrecall-core --test choice_review --test choice_backup`. Expect unsupported commands and unavailable feedback.
- [ ] **Implement:** Keep confirmation and grade in one `App::execute` transaction. Validate revision and unique IDs before writes. Correct saves only selection/judgment/receipt. Wrong and unknown share `apply_rating` with forced Again and capture the unconfirmed selection in the before-image. Defer reservation until a valid Continue. Capture each affected card once before its first deferred reservation, including later polling/filter changes while the feedback grade is still undoable. Restore those before-images on undo and invalidate them on relevant edits/deletes. Preserve linked undo when closing active feedback. End and affected edits/deletes clear feedback appropriately. Apply mode/operation/judgment invariants to live and backup state and all before-images. Add the new view variant to exhaustive consumers with safe existing handling until Task 10.
- [ ] **GREEN:** `cargo test -p openrecall-core --test choice_review --test choice_backup --test review_transactions --test restart --test completion_statistics --test legacy_backup_v2`. Repeat `choice_backup` with `--features serde_json/preserve_order`. Run `pnpm contracts` and `pnpm typecheck`. All pass.
- [ ] **Commit:** `feat: commit incorrect choice grades with durable feedback`.

### Task 7: Render typed retained contexts and verify source lifecycle

**Files:** `crates/openrecall-core/src/{domain/spelling.rs,spelling/collect.rs,spelling/validate.rs,services/spelling.rs}`, `crates/openrecall-core/tests/typed_spelling.rs`, `src/features/spelling/SourceContexts.tsx` and tests, generated contracts/fixtures.

**Interfaces:** Consume Task 2's `eligible_texts`, `snapshot_eligible_texts` and typed `SourceSnapshot`. Preserve v1 hash and v2 content rules while exercising transitions and rendering the retained authored choices.

- [ ] **Write test:** An essay prompt adds its front/notes words. MCQ adds stem/correct/notes words and excludes distractor-only tokens across create/import/edit/rescan/reallow. **Review Focus 5:** Change formerly correct `blue` to a distractor, set `green` correct and add `wrongword` distractor. Assert `blue` retains prior context/progress, `green` collects, `wrongword` stays absent. Delete the source and assert the complete original choices/key remain. Rehashed corruption of key or origin membership rejects restore. Unchanged legacy rescan remains idempotent.
- [ ] **Write UI test:** A retained MCQ shows authored choices and correct answers after source deletion; an essay shows its question/notes with no empty answer section. Plain HTML-looking text stays literal.
- [ ] **RED:** `pnpm test -- src/features/spelling/SourceContexts.test.tsx`. Expect missing typed context. Run `cargo test -p openrecall-core --test typed_spelling` as integration coverage of Task 2/3 hooks. Repair any observed lifecycle failure before broadening tests.
- [ ] **Implement:** Render retained choices and key in authored order using plain mixed-language text. Render only applicable answer sections. Keep existing lazy disclosure and pagination. Reuse the same snapshot version and source-availability comparisons across live and deleted contexts.
- [ ] **GREEN:** `cargo test -p openrecall-core --test typed_spelling --test spelling_collection --test spelling_lifecycle --test spelling_backup --test legacy_backup_v2`; `pnpm contracts`; `pnpm test -- src/features/spelling/SourceContexts.test.tsx`; `pnpm typecheck`. All pass.
- [ ] **Commit:** `feat: retain typed source context for spelling practice`.

### Task 8: Copy the reserved question through a native write

**Files:** Create core `services/question_copy.rs`, `tests/question_copy.rs`, desktop `src-tauri/src/question_copy.rs`, `src-tauri/tests/question_copy.rs`. Modify `crates/openrecall-core/src/{services/mod.rs,contracts/review.rs}`, `src-tauri/{Cargo.toml,src/lib.rs,permissions/study.toml,capabilities/default.json}`, `Cargo.lock`, `src/shared/api.ts`, native fixture, generated contracts and generated notices.

**Interfaces:** `CopyQuestionInput { session_id, display_id, expected_card_revision }`. `App::question_copy_text(&self, &CopyQuestionInput) -> AppResult<String>` resolves current presentation or matching feedback ID without checkpointing or reserving. `NativeApi.copyQuestion(input: CopyQuestionInput): Promise<void>` invokes `copy_question` only in the main window. The desktop adapter writes the resolved string via the official Tauri clipboard-manager Rust API.

- [ ] **Write test:** Essay bytes equal `"<b>Explain</b>\nSecond line"` exactly. MCQ includes the stem and numbered choices in reserved order and omits key/notes/feedback markers. Wrong feedback copies that same question. Stale ID/revision or invalid type fails without write. Assert a fake writer receives exactly one expected string and propagates a write error without touching scheduling/undo.
- [ ] **RED:** `cargo test -p openrecall-core --test question_copy` and `cargo test -p openrecall-desktop --test question_copy`. Expect missing adapter/command.
- [ ] **Implement:** Resolve a supported current display under the existing lifecycle/core lock, then write via the clipboard plugin. Choose the compatible pinned version from official Tauri/crates.io metadata at execution time. Register the narrow command with no raw text or plugin read/clear permissions. Reuse the license generator and regenerate locked notices. Source is the [Tauri clipboard plugin documentation](https://v2.tauri.app/plugin/clipboard/).
- [ ] **GREEN:** Both copy test targets, `pnpm contracts`, `pnpm typecheck` and `node scripts/check-notices.mjs`. All pass.
- [ ] **Commit:** `feat: copy reserved questions to the system clipboard`.

### Task 9: Edit and preview typed questions in Arabic

**Files:** `src/features/cards/{CardEditor.tsx,CardEditor.test.tsx,ChoiceEditor.tsx,CardList.tsx}`, `src/features/import/{ImportPreview.tsx,ImportPreview.test.tsx}`, `src/shared/errors.ts`, `src/styles/global.css`, typed fixtures in `src/test`.

**Interfaces:** `ChoiceEditor({value,onChange,disabled})` edits `ChoiceSetDraft`. Keep `CardEditor` save command and unsaved context. `ImportPreview` consumes generated `ImportForm`.

- [ ] **Write test:** Choose essay, type front and save with no answer field and an essay draft. Switching an occupied recall form to essay requires confirmation; cancel retains answer. MCQ add/remove preserves labels/focus and stable keys, supports single/multiple, toggles shuffle, removes deleted answer keys and sends an owned choice ID on edit. Preview exposes kind/choices/issues and keeps invalid forms excluded. Card list shows actual kind and no blank answer placeholder.
- [ ] **RED:** `pnpm test -- src/features/cards/CardEditor.test.tsx src/features/import/ImportPreview.test.tsx`. Expect missing kind selector and options.
- [ ] **Implement:** Arabic type selector per form, native labeled fields, local discard dialog, and meaningful focus on add/remove. Preserve existing dirty/busy guards and collapsed list behavior. Show inline Rust field errors and the chosen limits. Use current dark RTL tokens and mixed-language isolation.
- [ ] **GREEN:** Repeat the targeted tests, then `pnpm typecheck` and `pnpm lint`. All pass.
- [ ] **Commit:** `feat: edit and preview essay and choice cards`.

### Task 10: Review essays and MCQ through the existing session UI

**Files:** `src/features/review/{ReviewPage.tsx,EssayQuestion.tsx,MultipleChoiceQuestion.tsx,ChoiceFeedback.tsx,useReviewSession.ts,useReviewSession.test.ts,ReviewPage.test.tsx}`, `src/app/App.tsx`, `src/shared/errors.ts`, `src/styles/global.css`, `src/test/reviewFixtures.ts`.

**Interfaces:** Extend `useReviewSession` with `confirmChoices(ChoiceSubmission)`, `continueFeedback()`, and shared retry/recover for pending manual grade or choice confirmation. Use `NativeApi.copyQuestion` for copy. Presentation key is session, presentation and card revision. Feedback has its own identity key.

- [ ] **Write test:** Essay heading focuses with direct interval-labelled grades and collapsed notes. Copy uses current IDs, retains button focus, announces only after success and ignores late response on a newer question. MCQ radios/checkboxes obey native keyboard behavior. Number shortcuts cannot grade unconfirmed or committed wrong feedback. Keep unconfirmed drafts through poll, blur, failure and filter parking in the mounted view. After a correct native response, show text result and four grades. After wrong/unknown response, show due/correction/Continue/undo with zero frontend `rateCard` calls. Double click and lost response retry use one canonical token and never replace newer state.
- [ ] **RED:** `pnpm test -- src/features/review/ReviewPage.test.tsx src/features/review/useReviewSession.test.ts`. Expect absent type UI and confirmation/retry behavior.
- [ ] **Implement:** Reuse serialized session commands. Keep pending requests bound to the validated display. Preserve draft selection per live presentation within the mounted route, initialize restored undo selection from Rust and discard stale revisions. Render H1 prompt, H2 result, H3 explanation as specified. Focus result once, present misses/extras in text, preserve help and approximate intervals, and bind Continue to feedback ID. Extend native context at the existing App entry point.
- [ ] **GREEN:** Targeted tests and `pnpm test`, `pnpm typecheck`, `pnpm lint`. All pass including existing spelling and recall.
- [ ] **Commit:** `feat: review essays and multiple-choice questions accessibly`.

### Task 11: Verify complete flows and prepare 0.3.0

**Files:** `tests/e2e/{harness.tsx,question-types.spec.ts}`, `tests/native/question-types.spec.ts`, `scripts/test-native.mjs`, `docs/{user-guide.ar.md,testing/automated-results.md,testing/human-nvda-checklist.md,releases/0.3.0.md,releases/0.3.0.ar.md}`, `Cargo.toml`, `Cargo.lock`, `package.json`, `src-tauri/tauri.conf.json`, generated notices.

**Interfaces:** Browser harness mocks only native transport. Native test runner uses its existing automation binary, rejecting proxy and isolated SQLite library, with two real process launches for feedback resume.

- [ ] **Write acceptance cases:** Browser authors/imports both types and exercises accessible essay copy result, single/multiple selection, wrong feedback and correct manual rating. Run named axe states with nonzero passes and no violations. Native flow creates/imports with real IPC, tests write-only clipboard success on a synthetic prompt, grades wrong then closes/resumes through navigation and process restart, checks undo and Continue with a live due arrival, and round-trips format 2 schema 10 and both released fixtures. Record actual timing and audit counts. Assert rejected direct rating and no duplicate history.
- [ ] **Run acceptance:** `pnpm test:e2e`, `pnpm build:native-test`, `pnpm test:native`. Repair concrete regressions with observed failing tests before changes. Keep agent work limited to automated accessibility checks.
- [ ] **Prepare version:** Set workspace, package and Tauri version to `0.3.0`, update own lock records and notices, document JSON examples, grades, shuffle, compatibility and human NVDA/paste checks. Keep existing toolchain and dependencies except the approved clipboard addition.
- [ ] **Verify:** `pnpm verify` and relevant preserve-order suites. Confirm FSRS goldens, no generated drift, both immutable fixture hashes and no private credentials/real libraries tracked. Confirm production builds exclude automation/native overrides. Report the real Windows environment and other platform build evidence accurately.
- [ ] **Commit:** `chore: prepare OpenRecall 0.3.0 question types`.

### Task 12: Review, merge and verify the signed release

**Files:** Only verified review or CI corrections, `docs/testing/automated-results.md`, release notes and execution evidence as needed.

**Interfaces:** Repository `wip-bunker-42/OpenRecall`, protected `main`, tag `v0.3.0`. The six required checks are `quality`, `native-windows`, `bundle-windows`, `bundle-linux`, `bundle-macos-arm64`, `bundle-macos-x64`.

- [ ] **Review:** Request one fresh whole-branch GPT-6.1 Sol / Max review using the approved spec, this plan, both revisions, the five Review Focus lines and actual evidence. Verify findings by user effect. Fix Critical/Important via behavioral RED-to-GREEN, document Minor deferrals and rerun full/native gates after relevant repairs.
- [ ] **Integrate:** Push feature branch, create and attach its PR. Wait for all six actual required checks on that head. Use protected squash merge with matching head and no bypass. Verify merged tree and all six checks on the actual release commit.
- [ ] **Publish:** Verify unused `v0.3.0`, push its annotated tag at the checked commit, follow signed Release through completion. Repair real failures with the existing owned-draft recovery policy, retaining immutable published tags/assets.
- [ ] **Verify public delivery:** Match all 15 asset digests to local files and `SHA256SUMS.txt`. Confirm four updater targets, five official Tauri signed-version verifications, anonymous access to all assets and public latest manifest. Preserve both final Windows installers and evidence under the primary checkout's `target/delivery/v0.3.0`. Keep prior delivery files.
- [ ] **Close:** Run the final `pnpm verify` gate. Report PR, release, downloads, relevant decisions, testing limits and human NVDA checklist. Preserve the host-managed worktree and archive the plan's evidence without overriding platform cleanup restrictions.

## Self-review and handoff

Content and portable keys are covered by Tasks 1–3. Review gates, order and complete undo are covered by Tasks 4–6. Copy and typed provenance are covered by Tasks 7–8. Tasks 9–11 connect actual entry points and acceptance. Task 12 closes review and release. Each Review Focus item has its owning test. The plan sets `0.3.0`, 128 choices and 16 KiB per choice as implementation decisions. The product baseline remains the reviewed v0.2.0 tree until plan approval.
