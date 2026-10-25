# Architecture and invariants

The Rust core owns the learning state, scheduling, form selection, history,
training, validation and SQLite transactions. The Tauri shell owns native dialogs,
OS paths, single-instance behavior, lifecycle guards and signed updates. React
renders semantic Arabic HTML and invokes typed native operations. Browser test
fixtures never establish backend scheduling correctness.

## Data and transactions

`openrecall-core::App::execute(Request)` serializes validated commands through one
SQLite connection with foreign keys, WAL and a busy timeout. Normalized owner
tables and indexed due/deck columns contain typed JSON payloads. Each command is
an atomic savepoint; failures roll back related card, history, action and session
writes. Native commands run on blocking workers behind the database mutex.

Schema version 8 uses structural migrations 1, 2, 3, 5, 6 and 8. Defaulted JSON
fields do not require empty structural migrations. Future schemas are rejected.
Opaque revisions prevent stale edits and stale preview commits.

A card has stable form IDs, one shared FSRS memory state and a persistent shuffle
bag. Reading reserves a form without consuming it; grading consumes it. Undo
restores exact learning/presentation/bag state but refreshes opaque card revisions
to reject delayed requests. Content edits, relevant deletion, model changes and a
fresh deck session invalidate incompatible undo records. Action UUIDs are retained
as applied/undone records for retry safety.

Sessions preserve the current and parked presentations, filter, reveal state,
counters and timing. A live queue reads committed due state in urgency order,
without preempting an ongoing question. A monotonic heartbeat counts active time
and bounds suspension gaps. UTC due instants are independent of study-day buckets;
Cairo timezone/day-start settings determine local calendar statistics and DST.

## Scheduling and training

The pinned Rust FSRS adapter preserves difficulty, stability and stability_fast.
Fractional elapsed days come from UTC seconds; the app applies only validated
interval bounds to the engine's outcome. The frontend displays native previews
and absolute due dates and does not recompute a second scheduling model.

Immutable model revisions have provenance and explicit profile membership.
Training snapshots member histories and all relevant data/model/configuration
revisions. The chronological split excludes each held-out target from its own
history. The worker uses real FSRS optimization with cancellation/progress and
never writes the database directly. Polling persists the result, and applying a
candidate revalidates its complete source token and held-out acceptance metrics.
Replaying member histories under an activated model preserves their saved due
times. Other profiles remain unaffected.

## Files and native boundary

Card imports have a readonly, revision-bound preview and a single atomic commit.
Global duplicate identity is normalized question plus answer, independent of
notes. Invalid variants require explicit exclusion. All card content is plain
text with per-form language/direction support.

Full backups use a versioned typed envelope and canonical sorted JSON digest with
round-tripping numeric serialization. Validation checks ownership, models,
history aggregates, learning totals and undo before-images. Export flushes a
temporary file, verifies it and finalizes without overwriting an existing backup.
Restore binds a native preview ID to its parsed snapshot and the current library
revision, writes safety data first, then replaces the database transactionally.

IPC accepts no arbitrary filesystem paths or update URLs. Native pickers choose
input files, generated export IDs reveal only known exports, and the official
updater retains/verifies its own candidate and bytes. Capability grants are scoped
to the main window. A restrictive CSP and bundled assets keep study local.

## UI and accessibility

The app uses semantic navigation, native disclosure elements, labels, error
summaries, live status regions and focus-returning dialogs. Draft ownership is
aggregated per mounted component so saving one form cannot discard another form's
navigation guard. Collapsed policy/profile editors retain their draft state.
Review headings name the section rather than containing the question text.

Generated `ts-rs` DTOs keep native JSON numbers aligned with TypeScript numbers.
Tests use deterministic clocks/randomness and real temporary SQLite databases.
Only the explicit `automation` build feature can use isolated native test data;
production installers omit it. Automated accessibility evidence and human-only
NVDA verification are distinct deliverables.
