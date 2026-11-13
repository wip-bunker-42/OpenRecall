# Essay and multiple-choice questions

Status: approved by the owner on 2026-11-13. One independent
GPT-6.1 Sol / Max builder check found no remaining design questions.
Date: 2026-10-30. Baseline: published v0.2.0, commit
`ffd7a80bc58fea2465a533e38558741d670bba7c`.

## Intent and agreed behavior

Add question-only essay prompts for assessment by an external AI that the owner
operates independently, plus native single-answer and multiple-answer questions.
Keep Arabic dark RTL UI, offline study, accessible native controls, the existing
FSRS-7 engine and one shared learning state for every logical card's alternatives.

The owner approved the proposed behavior with one explicit amendment: an incorrect
multiple-choice answer must receive an automatic Again (Arabic: «نسيت»), with no
opportunity to substitute another grade. A correct answer permits manual grading.

Essay prompts expose the four grades immediately and have a one-action copy button.
Multiple choice has explicit checking, textual feedback, optional explanation,
default-on optional choice shuffling, and a reveal-for-unknown action. Import
preview, full backup, search and retained spelling contexts must understand the
new content. New words from incorrect distractors are excluded from spelling
collection; the stem, correct choices and notes remain eligible.

Builder calls proposed for approval: unknown/reveal receives automatic Again;
correct MCQ retains all four manual grades (including Again for a guessed answer);
wrong/unknown feedback remains visible after committing the grade until Continue;
unsubmitted choice selection is presentation-local, while submitted judgments and
display order persist. These are explicit interpretations, not additional owner
statements. No product implementation has started.

## Approaches and size

Considered (1) local choice checking followed by manual FSRS grading, and (2)
display/copy-only choices assessed externally like essays. The owner selected (1)
and changed its incorrect-answer branch to forced automatic Again. It gives
immediate correction offline and preserves user assessment after a correct answer.
External AI remains the owner's separate workflow for essays.

This is one coordinated feature project: question contracts, review state,
transactions, import, provenance, backups and UI change together. Keep React/Tauri
and the authoritative Rust core; do not introduce a frontend scheduler, an AI
service, automatic essay grading, answer-entry storage, a question generator,
timed exams, partial-credit scheduling or separate schedules for alternatives.
The design does not include the three previously deferred release minors.

## Content model and validation

Question type belongs to each form, not the deck. A standard deck can contain and
mix `recall`, `essay` and `multipleChoice` forms, including different types within
one card's alternatives. Form-bag selection remains randomized and nonrepeating;
all types share that card's memory, history, due date and optimizer membership.

- Recall: nonblank front and back, with the existing reveal-then-rate flow.
- Essay: nonblank front; back absent/empty. The editor omits the answer field.
  Optional notes are collapsed. No internal response-writing field is added.
- MCQ: nonblank front, at least two nonblank distinct choices, a valid answer key,
  `selectionMode` (`single` by default or `multiple`), and `shuffleChoices`
  (true by default). Single mode requires exactly one correct choice. Multiple
  mode permits one or more correct choices, including all choices. Correctness
  is exact set equality; an omitted correct choice or an extra choice is wrong.

Choices have stable native IDs owned by their form, independent of position.
Creation/import allocates IDs in Rust. Editing preserves IDs for surviving
choices and rejects foreign/duplicate IDs. Choice text comparisons for duplicate
validation use NFC and collapsed whitespace, preserving case: `US` and `us` may
be intentionally different answers. Question/choice/notes content is plain text.
Reject nonempty back on essay/MCQ and inappropriate MCQ fields on other types;
never silently discard supplied answer content. Existing language and custom
question-title behavior remains, including the 160-character title limit.

Resource ceilings for choice count/text/request size are the implementer's call,
must be generous, documented and consistently enforced in Rust/editor/import,
and must not impose a fixed four-choice quiz format. Existing variant-count
behavior is preserved. No unrelated dependency upgrades or engine/model changes.

## Editor, JSON and duplicate identity

Each form editor has an explicit Arabic question-type selector. It shows the
appropriate fields and exposes accessible add/remove choice controls, correct-key
selection, selection mode and shuffle preference. Switching type with populated
incompatible fields requires an explicit local discard confirmation; cancelling
preserves every draft. Native validation remains authoritative. Existing global
unsaved-change and source/policy action guards continue to apply.

Old JSON without `questionType` remains recall and retains its validation. An
omitted type on a variant inherits its primary form's type. Only type and the
existing question-title rule inherit; choices, answer key, selection mode and
shuffle preference do not inherit. Every MCQ form supplies its own complete key.
New optional MCQ settings default to single/true. Omitted/empty essay back is
valid only with explicit/inherited essay type. Unknown types are invalid.

Portable choices use local string keys, not native ownership IDs. Keys are unique
within a form; `correctChoices` refers to those keys. The importer validates the
complete structure and maps keys to fresh native choice IDs at commit. Example:

```json
[
  {
    "questionType": "essay",
    "front": "Explain how spaced repetition differs from rereading.",
    "notes": ""
  },
  {
    "questionType": "multipleChoice",
    "front": "Which city is the capital of France?",
    "choices": [
      { "key": "paris", "text": "Paris" },
      { "key": "london", "text": "London" }
    ],
    "correctChoices": ["paris"],
    "selectionMode": "single",
    "shuffleChoices": true,
    "notes": "Paris is the capital of France.",
    "variants": []
  }
]
```

Preview displays type, choices, key and field-specific errors so the owner can
exclude invalid/duplicate forms before commit. Existing library-revision guards
and transactional import/collection remain. Search includes authored choices.

Preserve legacy recall fingerprints byte-for-byte. Essay identity includes type
and normalized front. MCQ identity includes type, normalized stem, selection mode
and an order-independent set of (normalized choice text, is-correct) pairs;
ignore local/native choice IDs, display order, shuffle preference, title and notes.
Use case-preserving normalization for MCQ choices. A reordered import of the same
MCQ is duplicate; a different answer key or type is not the same question.
Managed spelling cards stay recall-shaped and cannot acquire manual question types.

## Review states and accessibility

Keep deck review mode (`standard`/`spelling`) separate from the selected form's
question type. Rust decides capabilities; the frontend does not infer permission
to grade from empty strings. Existing ordinary/spelling review remains unchanged.

Essay focuses the label-only H1 (custom title or «السؤال»), followed by the prompt
as normal mixed-language text, copy action, optional collapsed notes and the four
rating buttons with native approximate interval names. No empty reveal/answer UI.

MCQ focuses the label-only H1. An accessible fieldset/legend uses native radios
for single selection and checkboxes for multiple selection, with «اختر إجابة
واحدة» / «اختر جميع الإجابات الصحيحة». Check is enabled for a nonempty selection;
«لا أعرف، اعرض الإجابة» is available without guessing. Selection is freely editable
before confirmation. Number-grade hotkeys must not steal radio/checkbox keyboard
interaction; while an unconfirmed MCQ is shown, grading is unavailable in Rust.

Correct confirmation locks the submitted selection, focuses a label-only H2
«النتيجة», announces success in text, exposes the correct choices and optional
H3 notes/explanation, and enables all four grades. No review is recorded until a
grade is chosen. There is no second attempt on the same confirmed presentation.

Incorrect/unknown confirmation commits Again immediately and then shows a durable
graded-feedback screen: the question, selected answers (or skipped state), correct
answers, textual error/missed-answer details, explanation, actual committed due,
Continue and the existing undo/end/navigation controls. It has no manual grade
buttons. Continue only acknowledges feedback and selects the next due work;
it does not grade again. The final incorrect question still shows its correction
before Continue produces the session summary.

Feedback stays visible through polling, blur, filter changes, navigation/resume
and process restart. Filter changes affect the next question, not the visible
graded feedback. Ending the session clears pending feedback and shows the summary;
summary undo still restores the pre-grade state. Session active time includes
feedback reading, while question response time stops at grading.

All result text is plain text and programmatically labeled; correctness does not
depend on color. Background updates do not steal focus. Existing Arabic dark RTL,
language metadata, bidi isolation and reduced-motion/text-scale settings apply.
No agent operates a screen reader; human NVDA acceptance remains separate.

## Order, persistence, transactions and undo

Rust creates and persists a permutation of the selected form's choice IDs when
reserving its presentation. Disabled shuffling preserves authored order. Avoid
an immediate repeat of the entire previous order for the same form when shuffling
is enabled and another order exists. Do not fabricate factorial-sized shuffle
cycles; the existing alternative-form bag remains the nonrepeating cycle mechanism.
Retain the current/previous per-form order information needed across restarts.
Reconcile edited/deleted choice IDs atomically; no stale permutation is accepted.

Unconfirmed selection is local UI draft state: preserve it for same-presentation
poll/blur/errors and filter parking within the open review view; returning after
unmount/restart may reset it. The reserved order persists. Confirmation sends
selected stable IDs, presentation ID, expected card revision and an action token.
Before-image selection is captured in Rust so undo can reproduce the actual UI
immediately before that confirmation.

Rust validates ownership, selection mode, exact key equality and revision before
any history change. Correct confirmations persist their judgment and submitted
selection; wrong/unknown confirmations call the existing authoritative rating
operation with forced Again in the SAME transaction. Never implement this as a
frontend check followed by a separate rate request. A native direct grade cannot
bypass checking, replace a wrong grade, or grade an already committed feedback.

Persist operation-aware idempotency signatures: token reuse with different
operation/selection/revision is a conflict; matching retries never add a review;
undone tokens cannot reapply. Failure rolls back selection judgment, form/order
consumption, memory, history, counters, feedback and undo together. Correct
confirmation itself is not a review and does not replace the last-grade undo.

After wrong/unknown grading, hold native graded feedback rather than reserving a
new question or completing the session. Continue is bound to a feedback ID; stale
or duplicate requests cannot dismiss newer feedback or skip a presentation. On
acknowledgment, query the live Rust due queue/filter so arrivals join normally.

One-step grade undo restores memory/due/history/actions/counters, exact form/order,
selection and UI state. After wrong confirmation it returns to unconfirmed selected
choices (or unselected for unknown); after manually grading a correct response it
returns to that confirmed correct result. A later grade replaces the earlier undo
as before. Choice-order history consumed by a grade is included in its before-image.

Editing a question clears its stale submitted judgment/order draft and invalidates
affected undo using existing revision guards; preserve scheduling progress. Clear
its graded feedback on edit or deletion, and clear owned feedback on deck deletion.
Historical action payloads may reference past choice/form IDs without requiring
those IDs to remain live. Active/parked/undo/feedback references must be reconciled
so legal edits/deletions remain exportable and restorable.

## Copy and native integration

Use the official Tauri clipboard-manager Rust plugin behind a narrow native copy
command. Its documented Rust API supports text writes and the existing Rust
minimum (1.90); Windows/Linux/macOS are supported. The implementer selects/pins a
compatible current version from official metadata and regenerates notices.
Source checked: https://v2.tauri.app/plugin/clipboard/ on 2026-10-30.

The command accepts session/presentation identity (or feedback identity), not
arbitrary file paths, commands, URLs or text. Rust formats the current authored
content. Essay copies front exactly. MCQ copies front and numbered choices in the
current persisted display order, omitting key/correctness/notes. No added AI prompt.
It performs an explicit text write only: no clipboard read/monitor/clear, network,
external-app launch or synthesized speech. No JavaScript clipboard-read capability
is granted. Copy does not change scheduling, undo or presentation focus.

Success is announced only after the native write succeeds; failure has an Arabic
retryable message and no false success. Check stale responses before announcing
them on another question. Native file/update lifecycle guards cover the command.
Use isolated synthetic clipboard providers for byte-level automated assertions,
plus a real Windows write-path smoke test without reading existing clipboard data;
document any remaining human paste verification accurately.

## Spelling collection and retained provenance

Recall projects front/back/notes as before. Essay projects front/notes. MCQ projects
front/correct-choice texts/notes, excluding distractor-only tokens. This rule applies
to backfill, create/edit/import, rescan, reallow and backup edge validation.
It does not erase an existing spelling word merely because a later source makes
it a distractor; the established additive retention/deletion/exclusion policy stays.

Retained original context includes the authored MCQ choices, key, mode and shuffle
setting so it remains understandable after deletion. This is authored content, not
the user's attempted selection. Capture complete form content, not display shuffle.
Old recall snapshot hashes retain the exact v1 tuple in collect.rs; keep using it
for unchanged recall snapshots so rescans remain idempotent. Introduce a versioned,
domain-separated snapshot payload/hash for essay/MCQ that includes their typed
content. Default old snapshots to recall/v1; validate hashes and eligible-word edge
membership using each version. Content-availability comparison treats legacy defaults
as equivalent instead of reporting unchanged sources as changed after upgrade.

## Migration and portable data

Use schema 10 for typed forms, persistent permutations/judgments/feedback and typed
action signatures. JSON-backed storage may need little SQL, but migration must
explicitly upgrade existing payload defaults without changing study progress,
legacy action signatures, form bags, optimizer data or immutable v1 snapshot hashes.

Keep the format-2 envelope; dispatch by the supported (format, schema) tuple:
released (1,8), released (2,9), and current (2,10). Verify source JSON checksum
BEFORE defaults/upgrades. Preserve the recursively frozen legacy_v1 decoder and
add recursively frozen v0.2.0/schema-9 wire DTOs/conversion, including nested undo,
spelling snapshots and actions. Reject unsupported tuples/future versions.
Older apps reject schema 10 instead of silently stripping the new content.

Before changing product contracts, capture a genuine synthetic v0.2.0 export with
custom titles, spelling contexts/exclusions, profiles/history and usable live undo;
freeze its bytes/digest. Keep the genuine v0.1.0 fixture unchanged. Both must restore
with progress and usable undo. New backups include typed forms, order state,
correct confirmations, graded feedback, all existing data and undo before-images.

Semantic restore validation checks type invariants, choice ownership/permutations,
selection/key judgments, forced-Again feedback and its matching review/action,
active/parked/undo references, provenance hashes and eligible-word membership.
Recomputed-checksum corruptions must be rejected without writes. Correct-only
confirmation actions are valid without a review log; their historical IDs are not
new live foreign-key requirements. Safety-copy/preview/atomic replacement remains.

## Verification, delivery and remaining builder choices

Use behavioral RED-to-GREEN tests with isolated libraries, then full `pnpm verify`,
generated contracts, relevant browser/native suites, preserved FSRS inference and
optimizer goldens, both released backup fixtures and preserve_order cases. Include:

- Essay front-only editor/import/review/copy/direct grade/undo and legacy rejection
  of accidentally empty recall answers.
- Single and multiple exact-set checks, duplicate/foreign IDs, unknown/reveal,
  default/disabled shuffle and authored-order-sensitive questions.
- Wrong/unknown forced Again against direct IPC bypass, matching/conflicting retry,
  lost responses, rollback faults, restart, filter changes, final feedback, Continue
  duplication, due arrivals and complete undo; no double rating.
- Correct confirmation has no review until manual grade; correct manual undo restores
  the locked result; mixed-type alternatives retain one card learning state.
- Same-presentation keyboard/focus/draft retention, checkbox/radio arrow and space
  behavior, stale async errors/copy notices, notes/result headings and nonzero axe.
- Typed-content duplicate preview and search, migration/restore across all three
  tuples, genuine legacy undo, corruption classes, source edit/delete/rebind and
  distractor-excluded additive spelling collection/retained context.

The implementer chooses internal module boundaries, DTO/error names, generous
resource ceilings, supported clipboard version and exact SQL/index details. They
may refine state representation but must preserve the behavior/invariants above.
Do not introduce correctness analytics or optimizer inputs beyond existing rating
history. Release through the existing protected PR and signed pipeline, with exact
head/main checks and public asset/signature/version verification. Keep credentials
outside Git, production free of automation overrides, and report build/runtime,
human NVDA and installer/update limits accurately. Implementation planning follows
only after the owner approves this written design.
