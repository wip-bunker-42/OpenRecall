# OpenRecall Desktop Design

Date: 2026-10-24 (Africa/Cairo)
Status: Draft for user approval; architecture and repository destination approved.
Builder check: Independent review completed; no important planning questions left.
Product language: Arabic. Development language: English.
Intended public repository: `wip-bunker-42/OpenRecall`.

## 1. Intent

OpenRecall is a personal, offline spaced repetition desktop application. It helps
its owner practice knowledge rather than memorize a particular wording of a
question. Screen reader access is a primary product requirement, including the
review process, import previews, settings, statistics, dialogs, and updates.

Windows 11 is the primary platform; Windows 10, macOS, and Linux are secondary.
The initial product includes real persistence, genuine FSRS-7 scheduling and
per-deck optimization, backups, installers, and an in-app updater. It is not a
browser-only mockup or a service that needs an account or network connection to
study. There is no cloud backend, telemetry, or automatic upload of study data.

## 2. Requirements and boundaries

### Goals

- An Arabic RTL interface, dark by default, with legible text, restrained motion,
  strong focus indicators, and complete keyboard access.
- Create, rename, and delete decks; create, edit, and delete cards and variants.
- A visible create-deck action on the opening page, a compact summary per deck,
  and review actions both in the deck overview and inside a deck.
- Collapsed-by-default sections for card lists, statistics, advanced settings,
  and optimization details. Primary actions remain directly available.
- One learning state per logical card shared by all of its alternative forms.
- FSRS-7 scheduling using actual elapsed time, including intraday intervals.
- Review all currently due cards, including new cards; dynamically discover cards
  that become due while a session is running.
- Accessible interval previews for all four ratings and a complete undo of the
  most recent rating, including from the session summary.
- Validate and preview JSON imports, let the user exclude rows, and prevent exact
  duplicates of existing questions and answers from being imported again.
- Local per-deck training with a meaningful, enforced data sufficiency gate.
- Per-deck statistics and a useful session summary.
- Full, versioned backup export to Downloads and safe full restore.
- Public GitHub source, quality checks, dependency maintenance, release artifacts,
  and cryptographically verified updates initiated inside the app.

### Non-goals for the first version

Cloud sync, collaboration, mobile apps, AI-generated questions, automatic
paraphrasing, external content hosting, rich HTML execution, multimedia cards,
and Anki package import are outside this request. Text cards support Unicode,
newlines, arbitrary languages, and mixed-direction content. Card data is plain
text, not executable HTML or Markdown.

### Failure conditions to avoid

Variants must never acquire independent schedules. Incomplete training must not
be presented as successful personalization. Import preview must not modify the
library. A visually correct UI with broken heading navigation is unacceptable.
Backup restore must not silently discard the current database. Ordinary studying
must not require Internet access. Maintenance automation must not bypass failing
checks or silently change the FSRS model.

## 3. Approaches and the selected architecture

The approaches were presented to the user before this document was written:

1. **Selected:** Tauri 2 + a Rust application core + SQLite, with React,
   TypeScript, and Vite for the frontend. This combines native FSRS integration
   with a broad ecosystem for testing complicated accessible interactions.
2. Tauri 2 + the same Rust/SQLite core with Svelte and TypeScript. This reduces
   some frontend ceremony, with a smaller ecosystem of existing components and
   test integrations.

The user approved the first approach and the public repository destination.
Both approaches use a local Rust core because the requested FSRS-7 scheduler and
optimizer already exist together in the upstream Rust implementation.

### Responsibilities

- **Rust domain core:** model validation, FSRS adapter, scheduling, review queue,
  undo, statistics, import planning, training, backup validation, and persistence.
- **Tauri shell:** window lifecycle, single-instance enforcement, native file
  dialogs, user data and Downloads paths, and update integration.
- **React frontend:** navigation, accessible presentation, input collection, focus
  restoration, concise announcements, and review interaction. It must not have a
  separate scheduling implementation.
- **SQLite:** transactional authoritative data, schema migrations, and indexes.
  Enable foreign keys, a busy timeout, and WAL. Use an embedded SQLite build so
  the user does not need a separate database installation.

Keep the domain core independent of Tauri wherever practical so it can be tested
without a WebView. Use typed, narrow commands and structured results over IPC.
Long training, validation, and export operations run off the UI thread. A single
app instance owns the database. Rust validates every mutation even if the
frontend has already validated the input.

Use compatible stable toolchain and dependency versions available at
implementation time, inspect release notes and minimum versions, and commit
both package and Cargo lockfiles. Existing installed versions are not the reason
to select a dependency. Avoid a server framework, ORM, or global frontend state
library unless an actual implementation need justifies it.

## 4. Application navigation and accessibility

### Main areas

- **Library:** create a deck, list decks, compact counts (logical cards, due now,
  new), open a deck, review it, rename it, or delete it.
- **Deck:** name and compact status; start review; create a card; import JSON;
  collapsed cards, statistics, and optimization sections. The card section has
  search and pagination so a large deck does not produce an unbounded page.
- **Settings:** study preferences, appearance/accessibility preferences, advanced
  scheduling and training controls, full backups, and updates.
- **Review and summary:** a focused study flow entered for one deck at a time.

There is no separate account or sign-in screen. Empty states explain the next
action. Deletion dialogs identify the target and explain associated data removal.
Forms show an error summary and errors associated with the relevant controls.
Unsaved card edits receive a discard-or-continue prompt when leaving the form.

### Semantic structure

Set the application document to `lang="ar"` and `dir="rtl"`. Use native buttons,
links, inputs, labels, fieldsets, tables, dialogs, and disclosure controls. Use
ARIA only where it adds missing semantics. Disclosure controls expose their
expanded state and support Enter/Space. All non-review pages have one H1.

During review, the actual labels, and not the card content, are headings:

1. `<h1>السؤال</h1>`, followed by the front text as ordinary text.
2. After reveal, retain the question and add `<h2>الإجابة</h2>`, followed by the
   back text.
3. If notes exist, add `<h3>الملاحظات</h3>`, followed by the notes.

Before reveal the answer and notes are absent from the accessibility tree.
Content blocks use `dir="auto"`, Unicode isolation where appropriate, preserved
newlines, and sensible wrapping. Do not reverse content or force Arabic direction
on an English question. Do not guess a card's language for speech; allow optional
per-form language metadata in the editor while keeping the requested JSON schema
fully supported without it.

On entering a question focus its H1 (`tabindex="-1"`). Revealing the answer focuses
the answer heading. Advancing focuses the next question heading. Dialogs have
accessible names, contain focus while open, and return focus to the invoking
control. Undo restores the prior reveal state and a predictable focus target.

Rating controls are real buttons with these visible Arabic labels:

- `نسيت` -> Again (1).
- `افتكرتها بصعوبة` -> Hard (2).
- `افتكرتها بشكل طبيعي` -> Good (3).
- `افتكرتها بسهولة` -> Easy (4).

Each button visibly shows its approximate interval, and its accessible name
includes that interval (for example, `افتكرتها بسهولة، المراجعة بعد حوالي 30 يومًا`).
An associated description may provide the exact local date and time. Do not
depend on tooltips, hover, color, or a separate live announcement to communicate
the interval. Refresh stale previews on returning to a question and after
relevant data changes. The committed result announces the actual next due time.

Provide a skip-to-main link, logical tab order, visible focus, adequate contrast,
text zoom, reduced-motion support, and textual equivalents for every statistic.
Keep live regions polite and concise: announce a saved rating/undo, queue count
change, completed import, or error; do not continuously read clocks or the entire
card. Ordinary buttons suffice for all actions. Optional review shortcuts must
be documented, disabled while editing text, and never be the only way to act.

Use dark charcoal surfaces, a restrained teal accent, system Arabic-capable fonts,
and spacious readable text. Any bundled assets are local. Cosmetic details are
builder decisions subject to accessibility checks.

## 5. Logical model and persistence

Use stable unique IDs and UTC timestamps with sub-day precision. The logical
entities are:

| Entity | Essential responsibility |
| --- | --- |
| App settings | Validated preferences, study timezone and day boundary |
| Deck | Name, creation order, per-deck scheduling overrides and active parameters |
| Card | Deck membership, one memory state, due time, review counts, lifecycle state |
| Card form | Parent card, front/back/notes, stable display order, optional language |
| Review log | Card, form shown, rating, timestamps, elapsed time, before/after scheduler state, effective settings/model identity, response time |
| Session | Deck, start/end/status, persisted in-progress presentation and counters |
| Undo record | Before-image and session/presentation state for the latest rating |
| Training run | Deck, eligibility counts, configuration, dataset revision, progress, evaluation, candidate parameters and outcome |
| Schema metadata | Schema version and migration history |

Every card has one primary form at position zero and zero or more alternatives.
Forms never have a due date or independent FSRS state. The editor always leaves
at least one valid form. Original imported primary and variant order is retained.

Card and deck edits are transactional. Editing a wording preserves progress and
does not automatically reset the schedule. An explicit reset-progress action, if
provided, must be separately labeled and confirmed. Deleting a card removes its
forms, review history, and undo references; historical aggregates are recalculated
from remaining history. Deleting a deck removes its cards, histories, sessions,
and trained parameters. Deletion dialogs explain this. This is an engineering
choice for simple, predictable ownership and complete deletion.

Application startup runs migrations safely before accessing data. Back up before
a destructive schema migration. Do not operate on a newer unsupported schema.
Failures are surfaced in Arabic with a retry or recovery path.

## 6. FSRS-7 integration and scheduling

### Upstream dependency

At research time the official upstream `open-spaced-repetition/fsrs-rs` main
branch contains FSRS-7 after merged PR #426, but the latest published release is
still v6.6.2. Pin the Git dependency to this inspected revision:

`0fd59949e9168af7194418c2dba99836b5548e87`

The crate version alone does not identify its model. Check the model version
explicitly, preserve all three FSRS-7 memory fields (`stability`,
`stability_fast`, and `difficulty`), and record parameter/model provenance.
Wrap the crate behind a small adapter. Never implement a substitute algorithm
under the FSRS-7 name. Include the upstream license/attribution.

Use `next_states_with_elapsed_days` and fractional elapsed days, derived from
actual UTC elapsed seconds / 86400. Do not use the integer-day API for review or
truncate intraday histories. A first review has elapsed time zero. Map the four
ratings directly to the corresponding FSRS next state.

### Due dates and lifecycle

- New cards have no memory state and are due at creation/import commit time.
- Compute the interval from the upstream result at the effective desired
  retention. Retain intraday precision; do not round everything to whole days.
- Default desired retention is 0.90. Use FSRS-7 for learning and relearning too;
  do not add a second hard-coded learning-step scheduler.
- Apply a configurable lower interval bound (default 60 seconds) and maximum
  interval (default 36500 days). Clamp to a valid ordered range, preserving the
  chosen FSRS memory state. The bounds and the effective interval are visible.
- No random interval fuzz is required in the first release. Variant rotation
  addresses wording repetition without adding scheduling randomness.
- Store the resulting absolute UTC due time and the complete post-review memory
  state atomically with the review log.
- New means never rated. Learning means the first successful interval of at least
  one day has not yet occurred. After graduation, Again enters relearning until a
  successful interval is at least one day; other graded cards remain in review.
  These labels describe scheduling state; they do not substitute for FSRS.

Previews and commits use the same adapter and configuration. The backend
recomputes a rating at its actual commit timestamp rather than trusting state
from the frontend. Visible previews say approximately because time can elapse
while a user reads or thinks. With a fixed clock and unchanged inputs the preview
and committed interval must agree. A revision check prevents submitting a rating
against a card that has changed. Repeated submission of the same action token is
idempotent.

Do not promise that forgetting disappears or that a due card never repeats.
The enforceable repetition rule concerns forms, not suppressing necessary due
reviews. The app does not generate alternative questions on its own.

### Variant rotation

The first presentation uses the primary form. After a successfully committed
rating, advance to the next form in stable cyclic order. With alternatives, do
not show the same form again before the other current forms have been visited.
The next presentation can happen in the same session if FSRS makes it due.

Viewing a card, revealing its answer, opening the editor, or abandoning a review
does not advance rotation. Undo restores the form cursor exactly. After editing
forms, keep the next surviving stable form ID; if it was removed, choose the next
surviving form in order. With only one form, it is naturally reused. All forms
share ratings, history, memory state, and the next due date.

## 7. Review session and queue

Starting review selects one deck and all its nondeleted cards with `due_at <=
now`, including new, learning, relearning, and review cards. No default daily
limit or new-card cap hides eligible cards.

Sort by earliest due timestamp first, then creation time and stable ID for ties.
New cards use their creation/import timestamp as their initial due time. This
matches the requested oldest/most urgent due order without an undisclosed
priority algorithm.

The currently displayed card remains stable while the user reads it. Query and
merge newly due cards after every rating or undo, on app resume/focus, and when
a timer reaches the earliest upcoming due timestamp. Re-sort only the waiting
queue. Ensure at most one pending occurrence of a card, and do not insert the
displayed card again while it is being answered. A rated card can return only
when its new due time is reached.

Above the question show `البطاقة X من Y`. X counts committed review attempts in
the session plus the current presentation. Y is committed attempts plus the
current card and currently due waiting cards. Announce that the total can grow
when additional cards become due. Keep distinct-card counts separately so the
summary does not confuse repeat attempts with unique cards.

The answer-revealed state displays all four interval previews. A successful
rating persists one atomic change, announces the actual due date, and moves to
the next available question. Disable duplicate submission while a command is in
flight and recover visibly from write failures without losing the displayed card.

Before completing an empty queue, query due cards again inside the completion
operation. If none are due now, finish and show the summary immediately. Do not
keep the session open waiting for future due cards; they have not become due
during this session. Show the next future due time in the summary. If no cards
were due when starting, show a useful zero-review summary and that next time.

The user can end early with a partial summary. Persist session state and active
presentation so a crash/restart offers to resume or finish the interrupted
session. A mere close or crash must not submit a rating. Pause response-time
measurement while the app is inactive; separately record wall duration and active
study duration. Use a monotonic timer for in-process durations, UTC for due times,
and exclude closed-app time on resume.

### Undo

Provide a clearly named undo action and optional Ctrl+Z outside text inputs.
The supported undo unit is the latest committed rating in the active or just
completed session; consecutive undos may be supported if the same transaction
model remains simple, but one-level undo is the required acceptance target.

Persist enough before-state to restore the card's complete memory, lifecycle,
due time, counts, last-review timestamp, form cursor, displayed form, reveal
state, session counts, and review log. The undone rating must not contribute to
statistics or training. Undo the last rating from a summary returns to that
question. Refresh time-dependent queue membership after restoring the prior
presentation; real elapsed time is not reversed. Irrelevant dialogs and render
events do not replace the undo record. Leaving that session, deleting its card,
or restoring a backup invalidates its undo record clearly.

### Summary

Include start/end time, wall and active duration, review attempts, unique logical
cards, new cards studied, distribution of the four ratings, mean active response
time, remaining due count if ended early, and earliest upcoming due time. State
that recall rate means ratings 2-4 divided by scored attempts, not a promise of
future retention. Provide return-to-deck and undo actions.

## 8. JSON import and duplicate handling

Accept UTF-8 JSON files and pasted JSON in exactly this shape:

```json
[
  {
    "front": "Question",
    "back": "Answer",
    "notes": "Optional notes",
    "variants": [
      { "front": "Alternative question", "back": "Answer", "notes": "" }
    ]
  }
]
```

`front` and `back` are required nonblank strings. `notes` may be omitted or empty
and defaults to an empty string. `variants` may be omitted or empty and defaults
to an empty array. Every variant follows the same front/back/notes rules but
cannot contain nested variants. Accept arbitrary Unicode text and any practical
number of alternatives; do not impose a small arbitrary variant count.

Report unknown keys as warnings and ignore them, rather than guessing their
meaning. Reject wrong types, null fields, malformed JSON, an invalid top-level
shape, or nested variants with exact row/field paths and Arabic explanations.
An invalid primary form prevents selecting that card. An invalid alternative can
be individually excluded, leaving a valid primary card. Never silently import a
partially invalid alternative. Preserve user text; normalization is only for
blank checks and duplicate matching.

### Duplicate identity

The proposed engineering policy is **exact normalized question-and-answer
matching across the entire library**, including all primary and alternative
forms. A match in another deck is still shown as already present. Notes do not
make an otherwise identical form new.

For matching, normalize Unicode to NFC, canonicalize line endings, trim outer
whitespace, and collapse whitespace runs. Preserve case, punctuation, digits,
diacritics, and language-specific distinctions. Hash a length-delimited pair of
normalized front/back strings and confirm equality when needed. This catches
identical content and whitespace differences; it is not semantic paraphrase
detection.

Preview reports duplicates against existing cards and within the incoming file.
A duplicated primary blocks that card by default; it does not overwrite, merge,
or reset the existing card. A duplicated alternative is excluded and explained,
and the user may import the valid primary with remaining distinct alternatives.
If a duplicated primary has unique alternatives, explain that they can be added
through the existing card's editor; do not silently lose or merge them.

### Two-phase flow

1. Choose a destination deck and a file or pasted text.
2. Parse and validate without modifying the database.
3. Show a summary of valid, invalid, duplicate, warning, and selected counts.
4. Show an accessible paginated preview with per-card and per-variant selections,
   reasons for exclusions, and select-all-valid / deselect-all actions.
5. Import only explicitly selected valid content in one transaction. Recheck
   duplicates and the preview's database revision at commit time. If the library
   changed, refresh the preview before committing.
6. Announce the actual inserted logical-card/form counts and exclusions.

Add defensive file/text-size validation (initial limit 50 MiB, documented as a
resource limit) and paginated/lazy previews. Exceeding a limit returns a useful
message without importing anything. No file is uploaded anywhere.

## 9. Settings and study-day boundaries

Defaults and constraints are engineering proposals, not prior user selections.

| Setting | Default | Editing rule |
| --- | --- | --- |
| Study timezone | Africa/Cairo | Any supported IANA timezone |
| Start of study day | 01:00 local time | 00:00-23:59 |
| Desired retention | 0.90 | 0.70-0.99, global default and per-deck override |
| Minimum interval | 60 seconds | 10-3600 seconds |
| Maximum interval | 36500 days | 1-36500 days; cannot be below the minimum |
| Review shortcuts | Enabled | Can be disabled; help lists actual bindings |
| Text size | System/default | Accessible adjustable text scale |
| Reduced motion | Follow OS | Follow OS or force reduction |
| Card list page size | 20 | Bounded selection, e.g. 10/20/50/100 |
| Training epochs | Upstream FSRS-7 default | Automatic or validated 1-100 |
| Training batch size | Upstream default 512 | Validated 16-4096 |
| Training learning rate | Upstream FSRS-7 default | Automatic or finite 0.00001-0.1 |
| Training seed | Upstream default | Unsigned bounded integer |
| Training maximum sequence length | 256 | Validated 32-4096 |
| Training regularization multiplier (gamma) | Upstream default 1.0 | Validated 0.1-5 |
| Schedule penalties | Upstream default off | Advanced per-deck toggle |

Use normal and collapsed advanced sections with help text, valid units, and
reset-to-default actions. Decks inherit global scheduling defaults unless they
have explicit overrides. Training controls are set and stored per deck; the
settings area can link to each deck's optimization panel.

Expose the active learned parameter vector, its origin, last training metrics,
engine/model versions, schema version, and data location as read-only information.
Offer an explicit restore-default-parameters action for a deck. Raw learned
weights, derived counts, database paths, and compatibility identifiers are
read-only because editing them without preserving model constraints or data
relationships would be unsafe. These do not restrict valid scheduling/training
preferences. Data sufficiency and validation safeguards cannot be bypassed in
advanced mode. Short-term FSRS-7 learning stays enabled because sub-day scheduling
is a core requirement.

Define the study-date key as the local calendar date after subtracting the
configured day-start offset from the timezone-converted instant. Use an IANA
timezone implementation with DST handling; do not hard-code Cairo's UTC offset.
Store original UTC review timestamps. Changing the timezone/day boundary changes
report grouping, not elapsed-time calculations or existing due instants.
Display exact due dates using the configured timezone and Arabic formatting.

Retention/interval setting changes and activation of trained/default parameters
affect future ratings. Rebuild affected cards' memory states from their actual
histories when the parameter vector changes, but preserve already committed due
times until the next review. Do not silently rewrite a deck's future calendar.
Apply model updates only outside an active session for that deck; do not allow a
training result to invalidate the question currently being answered.

## 10. Per-deck optimization

Training is manual, local, cancellable, and scoped to exactly one deck. It never
borrows another deck's history. Multiple forms contribute to their parent card's
single chronological history. Exclude undone reviews and deleted cards. Preserve
fractional-day deltas and use zero for each first review.

### Data eligibility policy

The following conservative product gate is an engineering choice. It is separate
from the library's much lower implementation minimum and is not a statistical
guarantee of improvement. Require:

- At least 400 eligible scored recall observations after a card's first rating.
- At least 50 distinct cards contributing such observations.
- Observations spanning at least seven study-date keys.
- At least five Again outcomes and five recalled outcomes (ratings 2-4).
- A chronological training partition with at least 64 usable prefix observations
  and a held-out partition with at least 80 observations from at least 10 cards.

Count actual usable observations, not just raw button presses or all first
ratings. Show current counts and each unmet condition. Disable training and also
reject the command in Rust while ineligible. Explain when defaults are being
used and why collecting more history is necessary.

### Training and evaluation

Take an immutable, revision-tagged snapshot. Construct chronological per-card
histories and prefix items with stable aligned card IDs for the upstream trainer.
Split by target observation timestamp, approximately oldest 80% for training and
newest 20% held out. Training prefixes must contain no events beyond the cutoff.
Held-out predictions may use genuinely earlier observations of the same card,
but never the target rating or later observations. Compare the candidate against
the deck's existing parameters on precisely the same held-out targets using
unweighted log loss and root mean squared probability error. Explain both metrics
in Arabic in expandable details.

Use the real upstream `compute_parameters` for `Fsrs7`, with short-term support,
and report genuine progress. Validate parameter length, finite numbers, and
upstream model constraints before constructing a candidate. Do not call returning
unchanged default parameters an optimized result.

An acceptable candidate improves held-out log loss by at least 0.0001 and does
not worsen RMSE by more than 0.0001. These numeric tolerances are implementation
policy and tested explicitly. Show counts, old/new metrics, and outcome. If it
does not pass, keep the current model. If it passes, allow the user to apply it
or keep the current model; applying rebuilds memory states atomically and
preserves existing due dates. Do not perform a second fit on the held-out data
and then misrepresent its validation score as an independent test.

Allow cancellation, keep UI responsive, and leave active parameters intact on
failure or cancellation. If the deck's data/settings revision changes during a
run, do not activate a stale candidate; require a fresh run. Persist the training
report and model provenance. Do not train during a review session of that deck.

## 11. Statistics

The compact overview provides total logical cards, due now, and new cards. The
collapsed per-deck statistics section includes:

- Total cards and total forms separately; counts by new/learning/relearning/review.
- Due now, due later today, earliest next due time, and forecast counts over the
  next seven and thirty study days.
- Review attempts, distinct reviewed cards, active study time, and mean response
  time over today, the last seven days, the last thirty days, and all time.
- Four-rating distribution and observed recall rate with the stated definition.
- Daily history grouped by configured study date.
- Trained/default parameter status and optimization eligibility progress.

All metrics have explicit definitions and empty states; do not display NaN,
divide by zero, or invent sample progress. Tables and text are authoritative;
optional simple charts must have equivalent accessible data. Card counts count
logical cards, not forms. Session summary counts and stored logs use consistent
definitions after undo. Editing/deleting data invalidates derived caches.

## 12. Backups, export, and restore

Full export is distinct from importing a deck's card JSON. Create a UTF-8,
versioned `OpenRecall-backup-<local-timestamp>.json` file in the OS-resolved
Downloads directory, avoiding filename collisions. Export a consistent database
snapshot including settings, decks, all forms, memory states, absolute due dates,
review logs, sessions (including resumable state and valid undo records), trained parameters, model
provenance, training reports, and other user state necessary to resume studying.
Include format/schema/app versions and export time. Exclude machine-bound paths
and development signing credentials; resolve platform paths on restore.

Write to a temporary adjacent file, flush it, then finalize atomically. Report
the actual saved path and offer to reveal it in the file manager. If Downloads
cannot be resolved or written, show the reason and a native save-dialog fallback.
Never report success before the final file exists.

Restore is a full replacement, not an ambiguous merge. First parse, validate
version and all relationships, validate model data and timestamps, and show a
preview with settings/deck/card/review counts and export date. Confirm the
replacement explicitly in-app. Before replacement create a safety backup of
current data; abort if that cannot be saved. Apply the validated snapshot in an
atomic transaction, keeping existing data intact on error. Backups from an older
supported schema migrate; reject unsupported future formats with an explanation.
Reject duplicate IDs, dangling references, invalid schedules, incomplete FSRS-7
memory states, and unreasonable resource sizes. Restore works offline and can
move data between supported operating systems. Preserve due instants, settings,
and resume state while clearing any machine-specific stale handles.

## 13. Updates, packaging, and offline behavior

Bundle the frontend, fonts/assets if any, domain core, database engine, and FSRS
implementation. Study, edit, import, export, statistics, and training work without
network access. Network access occurs only when the user requests checking for
or downloading an app update. Offline update errors do not interfere with study.

Use the official Tauri updater with the public GitHub Releases endpoint for this
repository. Settings contains check-for-update, version/release details, download
progress, and install/restart controls. Verify signed update artifacts with the
embedded public key. Persist all data and finish pending transactions before
restarting. Do not install an update while a review or unsaved edit is active;
guide the user to finish or leave that flow first.

Generate the updater signing key locally outside the tracked repository, store
its private material only in the local credential location and GitHub Actions
secrets, and document recovery/rotation. Never print or commit private keys.
Updater signing is distinct from operating-system publisher certificates. Produce
Windows installers without claiming a verified publisher certificate when none
is available; macOS uses ad-hoc signing if no Apple credentials exist, with that
distribution status documented accurately.

Primary release target is Windows x64 with an NSIS installer compatible with
Windows 10/11. Provide a Windows offline installation variant embedding the
WebView2 offline installer so initial setup can also work without Internet.
Secondary CI targets are macOS Apple Silicon and Intel, and Linux x64 (AppImage
and a suitable Debian package). Publish only successfully built artifacts. Native
testing availability and any platform limitations must be reported honestly.

Tauri capabilities are limited to required commands, dialogs, paths, and updater
operations. Card content is rendered as text. Do not expose arbitrary filesystem
or shell commands to imported content. Keep a restrictive CSP and do not load
remote UI scripts.

## 14. GitHub repository and maintenance

Create the public repository under the approved organization using the connected
account. Check that the destination is available before creation; a conflicting
existing repository must not be overwritten. Use an English README with setup,
architecture, testing, supported platforms, privacy/offline behavior, backup
format, release/update instructions, and contributor guidance. Include a
permissive project license (MIT proposed) and required third-party notices.

Set up GitHub Actions for formatting, lint/type checks, frontend interaction and
accessibility tests, Rust unit/integration tests, and platform build checks.
Protect main with required status checks where repository capabilities permit.
Use PR-based changes and retain a traceable history. Source publication is
authorized; no separate approval is needed to push the completed source.

Use Dependabot for supported ecosystems and a narrowly scoped workflow to enable
auto-merge of bot-authored dependency-only patch/minor PRs after required checks.
Require human review for major versions, workflow/security-sensitive changes,
FSRS revision/model migrations, and changes that fail checks. Do not assume that
GitHub supplies a general autonomous repair agent: automate the concrete checks,
updates, PR creation, and permitted merges we can actually configure.

Add a scheduled upstream FSRS release check that opens or updates a migration
issue/PR when a published crate demonstrably contains the required FSRS-7 APIs.
Do not switch based only on a higher version number. Verify model identity,
parameter format, intraday scheduling, optimizer compatibility, and golden test
results before changing the dependency source. Do not auto-merge that migration.

Use issue/PR updates and GitHub's normal subscriptions for actionable failures
and maintenance notices; do not send messages externally. Avoid creating a new
issue on every unchanged scheduled run. Release automation builds and verifies
packages, signs updater artifacts, and publishes a complete updater manifest;
missing required signing configuration must fail visibly rather than ship
unverifiable updates.

## 15. Verification and acceptance criteria

Use Superpowers' test-first and review workflows in the implementation plan.
Inject clocks and temporary databases for domain tests. Assertions must verify
behavior rather than mirror implementation. The following are required:

1. **Persistence:** create/edit/delete a deck and card, restart, and verify real
   persisted content and state. Database errors do not produce false success.
2. **FSRS:** golden cases against the pinned real engine for all ratings, first
   reviews, intraday learning, long intervals, full three-field memory state,
   parameter validation, and fixed-clock preview/commit agreement.
3. **Forms:** primary -> variant 1 -> variant 2 -> primary, shared schedule and
   counts, no advancement on reveal/abandon, correct undo and form-edit behavior.
4. **Queue:** includes all due lifecycle states, excludes future cards, orders
   oldest due first, discovers a card becoming due during another question,
   avoids duplicates, and rechecks before session completion.
5. **Undo:** restores schedule, memory, form, answer visibility, logs, statistics,
   counters, and summary transition; repeated rating submission is idempotent.
6. **Import:** malformed/type-invalid/nested/empty/multilingual inputs, optional
   fields, partial variant exclusion, existing and in-file duplicates, zero
   writes before commit, revision conflict, and transaction rollback on failure.
7. **Training:** prevents insufficient or single-deck-crossed data, uses genuine
   upstream training on a sufficient deterministic fixture, avoids temporal
   leakage, handles cancellation/stale data, retains baseline on no improvement,
   and isolates applied parameters to the correct deck.
8. **Time:** Cairo DST transitions, configurable day rollover, unchanged due
   instants after timezone changes, fractional deltas, and app resume behavior.
9. **Backup:** export/restore round trip preserves all study state, cross-platform
   paths are resolved, invalid/future/corrupted snapshots leave data intact, and
   safety backup failure prevents replacement.
10. **Frontend:** keyboard-only creation/import/review/undo; correct H1/H2/H3
    labels and hidden answers; accessible interval names; disclosure states;
    focus management; Arabic/English mixed content; error and empty states.
11. **Native accessibility:** a Windows WebView2 smoke test plus manual assistive
    technology checks where available, including focus after Alt+Tab. Explicitly
    record what was tested with NVDA/Narrator and what still needs user testing;
    automated axe results alone do not establish screen reader compatibility.
12. **Release:** production frontend and native Windows build, installer artifact,
    CI builds for secondary platforms, updater manifest/signature validation,
    offline use smoke test, and source publication at the approved GitHub URL.

Do not claim native installation, screen reader validation, secondary-platform
runtime support, or an end-to-end update was tested unless it was actually
exercised. If a paid OS signing identity or physical secondary-platform test
environment is unavailable, deliver the rest and describe the precise remaining
distribution/testing limitation.

## 16. Decisions left to the builder

The builder may choose compatible stable dependency patch versions, internal
module/file boundaries, the exact typed IPC transport types, SQLite migration
mechanics, test-runner versions, icon assets, restrained styling details,
pagination mechanics, and CI cache strategy. They may tune nonfunctional resource
limits with documented evidence, but may not reduce the requested feature set or
replace the FSRS-7 model, accessibility contract, training isolation, or offline
data guarantees.

The chosen duplicate scope, training gate, default interval bounds, MIT license,
plain-text-only content, complete-delete policy, and full-replacement restore
are explicit engineering proposals for the user to review with this document.

## 17. Primary research references

- FSRS-7 merge: https://github.com/open-spaced-repetition/fsrs-rs/pull/426
- Inspected engine revision: https://github.com/open-spaced-repetition/fsrs-rs/tree/0fd59949e9168af7194418c2dba99836b5548e87
- Published release at research time: https://github.com/open-spaced-repetition/fsrs-rs/releases/tag/v6.6.2
- Fractional scheduling and memory fields: https://github.com/open-spaced-repetition/fsrs-rs/blob/0fd59949e9168af7194418c2dba99836b5548e87/src/inference.rs
- Training inputs and native minimums: https://github.com/open-spaced-repetition/fsrs-rs/blob/0fd59949e9168af7194418c2dba99836b5548e87/src/training.rs
- Tauri frontend guidance: https://v2.tauri.app/start/frontend/
- Tauri updater: https://v2.tauri.app/plugin/updater/
- Tauri GitHub builds: https://v2.tauri.app/distribute/pipelines/github/
- Tauri capabilities: https://v2.tauri.app/security/capabilities/
- Tauri WebView versions: https://v2.tauri.app/reference/webview-versions/
- Windows accessibility regression/fix context: https://github.com/tauri-apps/tauri/issues/15624
- WAI disclosure pattern: https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/
