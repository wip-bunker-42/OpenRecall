# OpenRecall spelling practice, question titles and daily completion

Date: 2026-10-29 (Africa/Cairo)
Status: Approved by the user on 2026-10-29, after the disposable-input clarification.
Builder check: Independent GPT-6.1 Sol returned "nothing important left" on 2026-10-29.
Subsequent user clarification: practice input is discarded on leaving a card;
there is no typing retention for navigation, undo or session resume.
User amendment on 2026-10-30: prefill the target word on the textarea first line
and enter that field on its first line automatically. This replaces the earlier
heading-focus/initially-empty-field choice; user-entered attempts remain disposable.
Baseline: main `81a301aad06f5e9dbf04cc149dad67bd23ef2f08`, released application v0.1.0.
Product language: Arabic. Development language: English.
Intended release: v0.2.0, through the existing protected PR and signed release workflow.

## 1. Intent and confirmed requirements

Add a dedicated spelling workflow for a blind learner who enters a native text
field prefilled with an English word on the first line, reads it with their own
screen reader, writes in a
plain accessible text field, and self-rates recall. The application does not
pronounce the word, spell it aloud, compare the typed text, correct it, suggest
completions, or decide the rating.

The user explicitly requested:

- At most one spelling deck per account. OpenRecall currently has one local
  library and no account service: enforce at most one in that local library.
- The default deck name is `مجموعة التدرب على الكتابة`.
- Select one ordinary source deck. Automatically collect unique English words
  from front, back and notes, including existing cards and later additions.
- Include unknown names and specialist terms automatically; dictionary membership
  must not gate inclusion.
- Collection is additive. Source edits, form removal, card deletion, source-deck
  deletion and later source changes must not delete collected spelling words or
  reset their schedules, history or progress.
- Keep the last saved source context after the live source is deleted, so
  `عرض البطاقة الأصلية` still explains where the word came from.
- Practice input exists only inside the currently displayed card's text field.
  Leaving that card discards it immediately, including when returning through
  undo, filter changes or a resumed session. Never save the user's typing.
- Let ordinary cards customize the review question heading through JSON.
- Show daily completed-for-today counts per deck and for the whole library.
- Continue offline study, accessible Arabic UI, genuine FSRS-7 scheduling,
  transactional undo, full backups, and signed explicit updates.
- The agent performs automated accessibility checks only. The human partner
  performs NVDA testing. Never launch, operate or configure a screen reader.

### Failure conditions

Do not silently lose vocabulary when its source changes. Do not turn multiple
occurrences into independent spelling cards. Do not merge a spelling card's
learning state with an ordinary source card. Do not score or inspect practice
input to derive a grade. Do not announce words through a live region or
programmatically synthesize speech. Initial focus enters the native textarea
at the beginning of its prefilled first line, without selecting the word. Do not silently omit unknown dictionary words.
Do not break existing cards, JSON imports, v0.1.0 backups or normal answer reveal.

## 2. Approaches and selection

The following approaches were presented before writing this document:

1. **Selected by the user:** native Rust `unicode-segmentation`, with a small
   documented English-token policy, URL/email exclusion and deterministic
   duplicate keys. It fits this English-focused feature without a language model
   or dictionary gate.
2. ICU4X word segmentation. A suitable Unicode implementation with broader
   multilingual capabilities and additional data/features. Those capabilities
   are unnecessary for the present English-focused extraction policy.

Research also considered Hunspell-compatible dictionaries, Spellbook and ESDB,
and linguistic tokenizers such as nlprule. A dictionary would exclude legitimate
unknown names unless supplemented, while a grammar/lemma model adds behavior the
user does not want. Neither is required for this release. This is a decision
about the requested task, not a claim that Unicode segmentation identifies a
word's natural language or the author's intent.

Use the existing Rust core, SQLite, Tauri and React stack. Proposed small native
support libraries are `linkify` for URL/email spans, `unicode-script` for Latin
script identification, and the existing `unicode-normalization`. The builder
verifies current compatible stable versions, licenses and the frozen test corpus
before adding dependencies. Do not write a replacement Unicode boundary engine,
spellchecker, screen-reader driver, or NLP runtime.

## 3. Spelling deck lifecycle

### Creation and configuration

- The library offers `إنشاء مجموعة التدرب على الكتابة` while no spelling deck
  exists. Its dialog selects one existing ordinary source deck. If none exists,
  explain that an ordinary deck must be created first.
- Creation atomically creates a deck of immutable kind `spelling`, a separate
  default optimization profile, its source binding, initial vocabulary and source
  contexts. The existing profile settings and default-on Schedule Penalties apply.
- Repeated/stale creation requests cannot create another spelling deck. SQLite
  enforces the singleton, and the API returns a structured existing-deck error
  with a way to open it. Uniqueness is by kind, not by user-editable deck name.
- The default Arabic name may be renamed through the ordinary deck action. A
  spelling badge and source settings continue identifying its special kind.
- Source selection is available in a collapsed section in the spelling deck.
  Changing the source adds words from the newly selected deck; it retains words
  and contexts collected from previously selected sources.
- Only the currently selected deck contributes new words or new provenance.
  Already tracked source forms from past bindings may still refresh the last
  matching context of their existing word links when edited. They cannot add new
  words after unbinding. This keeps retained context current without silently
  continuing to collect vocabulary from several decks.
- A spelling deck cannot select itself or another non-ordinary deck. Ordinary
  decks cannot be converted to spelling by renaming or generic editing.

### Collection and deletion

- Scan all forms of each source logical card, across front/back/notes only.
  Do not scan JSON keys, instruction headings, deck names or unrelated decks.
- Backfill on creation or source selection and synchronize successful source
  card creation, content edits and committed imports. Import preview does no
  collection and makes no writes.
- A repeated occurrence only adds or refreshes a provenance link. It does not
  modify the spelling card's memory, due time, card revision or current display.
- Removing a word from a source freezes its previous matching context. Removing
  a form, card or deck retains existing spelling cards and their saved contexts.
- Deleting the selected source clears its live binding and explains that a new
  source can be selected. Existing spelling reviews remain available.
- Generated spelling cards have one managed form and cannot be manually edited
  into arbitrary ordinary cards. Ordinary card creation and JSON import into this
  special deck are rejected by Rust and are not offered by its UI.
- The user may explicitly delete a spelling word with the normal confirmation.
  This deletes that word's own history/progress and provenance and adds a durable
  exclusion key, preventing the next source edit from immediately recreating it.
  A collapsed excluded-words list can allow future collection again. This is
  labelled as allowing re-addition from the source, not restoring deleted progress.
- Explicitly deleting the whole spelling deck removes its vocabulary, exclusions,
  sessions and cached contexts after confirmation. Its ordinary source is not
  deleted. A later new spelling deck starts a new collection and progress.

## 4. Deterministic extraction policy

Unknown words are included without a dictionary. Consequently, the feature
extracts eligible Latin-script words/terms, not a statistically certified English
vocabulary. No tool can guarantee the intended meaning of an unfamiliar name.
Make this limitation understandable in collapsed help; provide explicit word
deletion/exclusion rather than silently changing source text.

The policy below is a builder proposal for user approval:

1. Use linkify to identify URL and email spans, including bare host links, before
   word collection. Do not collect substrings inside those spans. No networking
   occurs during link detection.
2. Use Unicode word boundaries and character script properties. Accept Latin
   letters with their combining marks and terms containing ASCII digits if they
   also contain at least one Latin letter. Reject pure numbers and mixed-script
   candidate tokens rather than extracting misleading Latin fragments from them.
3. Preserve internal straight/curly apostrophes and single internal hyphens as
   part of a token. `don't`, `John's` and `well-known` are single spelling targets.
   En/em dashes separate words. Wrapping quotation and sentence punctuation are
   excluded. Recognize the bounded terminal forms `C++`, `C#` and `F#` as intact
   terms; do not generate their punctuation as separate cards.
4. Numeric identifiers with no letters, underscore identifiers and dotted
   candidates that are recognized as links are excluded as whole spans, not
   broken into accidental vocabulary fragments. Dotted technical names are an
   acknowledged ambiguous case; URL recognition wins in this release.
5. The duplicate key uses NFC normalization, Unicode lowercase, curly-apostrophe
   equivalence and equivalent internal hyphen characters. Preserve accents; do
   not stem or lemmatize. Keep the first captured display spelling, with `I`
   displayed as `I`. Capitalization alone does not create another target.
6. `learn`, `learned` and `learning` remain distinct targets. All ordinary words,
   including `a`, `I`, `the` and `is`, are eligible. Do not remove stopwords.
7. One key identifies one spelling card throughout the singleton's lifetime,
   across all fields, alternative forms, cards and past source bindings.

Freeze a synthetic extraction corpus covering English/Arabic mixed prose,
Unicode normalization, punctuation, contractions, hyphenation, unknown names,
alphanumeric terms, URLs/emails, repeated fields/forms, and the exclusions above.
Changing library versions or token policy requires checking that corpus first.
Persist `extractionPolicyVersion: 1` with the spelling configuration and snapshots.
Validate stored provenance using that version, not whatever a newer library would
produce by default. A future policy change requires an explicit compatibility
decision/migration; unknown policy versions fail before restoring any data.

## 5. Retained source context

Each provenance entry identifies the original source deck/card/form and the last
content version of that form which actually contained this word. It stores the
complete front, back, notes, optional question title, language, source labels and
capture time. IDs of historical sources are historical identifiers, not foreign
keys requiring the live source to remain present.

Use immutable normalized context snapshots shared by all words referencing the
same source-form revision. Each word/source-form edge points to its latest
matching snapshot. For example, when a form changes from `red blue` to
`blue green`, red retains the old context, blue points to the updated context,
and green is newly collected. Do not replace red's context with text that no
longer explains red. Keep only referenced snapshots, not an unlimited edit log.

On source deletion, retained snapshots survive. There is no cascading foreign
key from a live source to spelling progress or cached context. Garbage collection
can remove only snapshots with no remaining provenance references, such as after
explicit deletion of the spelling word or deck.

`عرض البطاقة الأصلية` is a native disclosure collapsed on each new word. It
shows the retained source form as plain text, with a captured-version label and
capture date. Where multiple sources exist, show a paginated list, ordered by
first capture and stable source IDs, with a count and previous/next controls.
The user can inspect contexts even when all live sources are gone. A removed
source is labelled as removed; never substitute a different live card.

Load context details only on request through a bounded typed API. A source form
is sufficient context: do not duplicate unrelated alternative forms unless they
also contain the word and thus have their own provenance entry.

## 6. Review, editing and accessibility

### Spelling review

- Reuse native sessions, FSRS-7, queue order, timing, all/new/previous filters,
  live due arrivals, interval previews, action-token retries and atomic undo.
  Reviews remain per-deck as in the current app; no new cross-deck session is added.
- A spelling word is its own logical card, independent of all source cards.
  Its single managed form has front/back equal to the display word, empty notes
  and English language metadata. Back is internal compatibility data and is not
  displayed as an answer or used to compare input.
- The review view explicitly identifies `standard` or `spelling` mode. Rust
  determines the mode from persisted deck kind. Ordinary cards still require
  answer reveal; spelling cards permit direct self-rating and receive native
  rating previews immediately. There is no `عرض الإجابة` step for spelling.
  A reveal command for spelling returns a structured not-applicable error.
- Use H1 `الكلمة`, followed by the entire word in one normal text node with
  `lang="en"`, LTR direction and bidirectional isolation. Keep this normal text
  node and label heading available for navigation. On each presentation mount,
  initialize the textarea to the target plus a newline, focus it and place the
  collapsed caret at offset zero on the first line. Do not select the word.
  The second line starts blank; all text remains freely editable.
- Do not put the word or source content in aria-live announcements, synthesize
  speech, split it into letter nodes, or send commands to a screen reader.
  Actual pronunciation and typing echo remain controlled by the user's reader.
- Follow the word with a labelled native multiline textarea, then the existing
  four rating buttons and their accessible intervals. Disable spellcheck,
  autocorrect, autocomplete and autocapitalization. The initial value is only the
  known target plus a newline, never a saved attempt. No answer validation,
  correctness markers, text suggestions, automatic grade or disabled grade based
  on the text. Empty input may be self-rated too.
- Keep normal cursor movement, selection, clipboard and text undo. Review
  hotkeys do not run inside the textarea; outside editing controls they may rate
  a spelling card directly when shortcuts are enabled. Screen-reader navigation
  does not require switching to a custom editor.
- Put the original-card disclosure after the rating controls. Nested source
  headings stay below the review H1. Reset disclosure on a new word, and keep it
  stable on polling. Pending or failed grading cannot clear the typed attempt.

### Disposable practice input (explicit user requirement)

Use an ordinary uncontrolled textarea whose value lives only in the currently
mounted card's DOM. The application does not read or copy its text or selection
into React draft state, session/presentation caches, undo snapshots, browser
storage, native requests, logs, training, backups or analytics. Normal native
text editing, including text undo, works only while that field remains open.

Leaving the card destroys the field and its user-edited content. This includes advancing to
another presentation after a successful grade, going to the summary or another
page, parking a card through a filter change, ending the session, restoring the
library, or closing/reloading the application. Every later return starts with
only the target word and a newline, even if the word/presentation is the same
or native review undo returns to the previous card. Only the initial collapsed caret is set; the application never reads the text
or selection. Do not retain a draft or caret position for undo,
navigation or session resume. Native review undo still restores scheduling,
history and counters; the user's explicit disposable-input rule governs typing.

Moving focus to another control inside the same displayed card is not leaving
the card. Do not erase text on textarea blur. Opening/closing source context,
ordinary polling or rerendering, and a failed/pending grade leave the same DOM
field intact while the same card remains displayed. A changed presentation or
unmounted review view starts with only the target and a newline. Practice typing is not an unsaved-data source
and must not add a discard-draft navigation prompt. Field help states that the
text disappears as soon as the user leaves the card.

### Ordinary question titles

Add optional per-form `questionTitle` to Rust data, generated contracts, the card
editor and JSON import. It is a heading label, not the front text. Missing/null
or whitespace-only values resolve to `السؤال`. Nonblank titles are trimmed plain
Unicode text, limited to 160 characters, without control characters/newlines.
The heading remains H1; answer remains H2 and notes H3. Its text uses auto
direction and does not change source language or speech settings.

Example:

```json
[
  {
    "questionTitle": "ترجم",
    "front": "unpredictable",
    "back": "لا يمكن التنبؤ به",
    "notes": "",
    "variants": [
      {
        "questionTitle": "أكمل",
        "front": "The weather is ____.",
        "back": "unpredictable",
        "notes": ""
      }
    ]
  }
]
```

On import, an omitted variant title inherits the primary title; explicit null or
blank resets that variant to the default. Store resolved per-form values: this
is import convenience, not a permanent inheritance link in the editor. Bad title
types/lengths are shown in the existing readonly preview and invalid-form
exclusion flow. Unknown unrelated fields retain the existing warning policy.

Existing JSON still imports. Existing forms default to `السؤال`. A title does
not alter ordinary duplicate identity (front plus back), FSRS memory or form-bag
identity. Regular import duplicate checks exclude generated spelling cards,
because spelling and knowledge recall are different tasks. Retained contexts
include the title but vocabulary extraction never scans it.

## 7. Completed-for-today statistics

Add `completedForDay` to per-period/day statistics and display `انتهت مراجعاتها
اليوم` per deck. In the spelling deck its unit is words; in ordinary decks its
unit is logical cards, not all words appearing in a card. This unit choice is a
builder proposal for approval.

For study day D, a surviving logical card contributes once if:

1. It has at least one effective, non-undone rating in D.
2. Its last effective rating in D has `next_due_at` in a study day later than D.

Use the configured timezone and day-start minute for both instants, including
Cairo/DST. Do not use the scheduler's lifetime `has_graduated` flag or require a
24-hour interval. A five-minute interval crossing the study-day boundary can
qualify; a due time later within the same study day cannot. Ratings of any type
qualify if they meet this date rule. An untouched card already due next week does
not count as today's accomplishment.

Count a card once despite repeated ratings/forms. A later same-day rating or
undo recomputes its eligibility. Historical days use each day's last effective
rating, so tomorrow's review does not erase yesterday's completion. Changing
timezone/day-start re-buckets history consistently with existing statistics and
does not move saved due instants. Explicit card/deck/history deletion continues
the existing surviving-history semantics; deleting a source does not delete its
independent spelling history.

Add a collapsed `الإحصاءات الإجمالية` section on the library page with today's
count and paginated daily history. The total counts logical cards across ordinary
and spelling decks. Show the spelling-word subset separately; reviewing a word
for meaning and for spelling represents two independent cards, not one global
semantic-word identity. Period totals sum daily completions, while one day's
count remains distinct-card based. Existing attempts/time/ratings stay intact.

## 8. Native model, transactions and interfaces

Use a structural SQLite migration from schema 8 to 9:

- A normalized immutable deck-kind column, defaulting existing decks to ordinary,
  with a unique partial index allowing at most one spelling deck. Typed payload
  kind and the indexed column must agree on all writes/restores.
- A spelling source binding tied to the spelling deck; its live ordinary source
  foreign key may become null on source deletion. Cache a descriptive source name.
- A word/card map with a unique canonical key, pointing to its ordinary scheduler
  card row inside the spelling deck.
- Immutable shared source-form snapshots and word/source provenance edges.
  Historical source IDs are allowed to refer to deleted records.
- Durable excluded-word keys scoped to the spelling-deck lifetime.

Core invariants: every spelling card has exactly one word mapping and one managed
form; mappings belong to the singleton; provenance points to an existing snapshot
whose content contains the mapped canonical word under the versioned policy;
ignored keys cannot simultaneously have an active word card. A deleted live
source is not a dangling provenance error.

Keep create/bind/backfill and each source mutation plus derived inserts/context
changes in the existing native atomic command boundary. Do CPU work on native
blocking workers, not in React. A content edit/import reports success only when
its derived collection commits; any failure rolls both back. Large backfills use
bounded batches internally within the transaction and touch each affected deck's
revision once, without per-word full-deck queries. Show busy progress; do not
claim success early or expose a partially collected deck.

Do not attach collection to generic `save_card`, which is also used by ratings,
undo and replay. Hook content create/update/import and source-binding workflows.
Ordinary ratings do not re-extract content. Context-only refresh bumps the
library revision but not an existing spelling card's revision or learning-data
revision. New vocabulary changes the spelling deck's data revision, invalidating
stale training datasets normally. No source update preempts a reserved question
or clears the current uncontrolled attempt. New committed words are eligible new due cards.

Keep typed narrow commands for creating the singleton, reading/changing its
binding, rescanning the current source, paging retained contexts and managing
excluded keys. Reuse ordinary deck deletion, card deletion with kind-aware
semantics, review and training commands. Add library statistics and title fields
to generated DTOs. Exact command names and internal modules are left to the
builder; arbitrary file paths, scripts and update URLs remain forbidden IPC.

### Backup and migration compatibility

Export a format-2 backup containing kind, word maps, source binding, retained
contexts/provenance and exclusions in addition to all existing progress/settings.
Keep the shared 256 MiB limit and safety-export/atomic-replacement guarantees.
Shared snapshots prevent copying a long source paragraph once per extracted word.

Accept the real v0.1.0 format-1 backups through a version-dispatched legacy reader.
Verify the legacy checksum using the original serialized data/canonical rules
before adding default title/kind/new collections. Do not deserialize into new
DTOs and reserialize newly defaulted fields before checking the old digest.
Validate legacy relationships, then upgrade ordinary decks, absent titles and
empty spelling collections in memory. Preserve memory, due times, histories,
profiles and sessions. Never reset an existing library during schema upgrade.

Format-2 validation enforces singleton/mapping ownership, canonical uniqueness,
snapshot integrity, safe missing-live-source references, exclusions, all existing
FSRS invariants and session/undo before-images. Reject contradictory mappings,
forged snapshots or multiple spelling decks before writes. Old applications do
not import format 2; explain that the new version is required. Keep v0.1.0 and its
published artifacts/tags unchanged.

## 9. Verification and release gates

Plan behavioral RED-to-GREEN coverage for:

- Singleton enforcement, stale create/bind requests and atomic rollback.
- Backfill and incremental extraction across fields/forms/imports and unknown
  words; deterministic deduplication; source switching; durable exclusions.
- Source edit/removal/deletion and source-deck deletion retaining spelling state
  and the last matching context; multiple words sharing snapshots correctly.
- Spelling FSRS progress independent of sources; direct ratings/previews without
  normal reveal; unchanged normal reveal guard; retries, filters, due arrivals,
  restart and native one-step undo.
- Plain textarea behavior, stable DOM text/caret while the card stays displayed,
  keyboard editing and no erasure on blur/source disclosure/polling/failed grade.
  Advancing, leaving/resuming, filter parking, summary undo and restart must all
  return to a field containing only the target and a newline. Assert typed practice text never enters native
  requests, persisted state, undo records or backup data.
- Old JSON and title inheritance/default/error cases, editor round-trip, dynamic
  H1 headings and unaffected ordinary duplicate identity.
- Day-boundary/DST, several reviews/forms per card, future untouched cards,
  historical days, undo and global/subset aggregation.
- Real format-1 fixtures from v0.1.0, schema-8 upgrade, format-2 round-trip with
  deleted sources and undo, corruption rejection, safety backup and disk failure.

Run `pnpm verify`, contracts/drift, and relevant real Windows native tests using
temporary data. Audit spelling question/input/disclosure/error/summary, ordinary
custom title, settings, global statistics and collapsed/expanded states with axe
and keyboard automation. Native tests use real Rust/SQLite/FSRS. Agent tests never
use a screen reader or the user's study database. Update the human NVDA checklist
for manual word navigation, typing echo/caret, source disclosure and undo.

The implementation plan must include a fresh whole-branch code review, verified
fixes, the six required exact-revision GitHub checks, protected PR merge and a
new signed v0.2.0 release. Match Cargo/package/Tauri versions and release notes.
Check normal/offline Windows installers, all four updater targets, all signatures,
public downloads, server/local checksums and the live latest endpoint. Existing
signing credentials remain outside Git; production omits automation features.
Report actual runtime testing separately from builds, especially the unperformed
human NVDA, secondary OS runtime and full older-to-newer installer tests. Preserve
delivery artifacts before any allowed workspace cleanup; do not bypass protected
primary worktree restrictions. Reassess the existing Linux GLib disclosure rather
than silently suppressing its alert.

## 10. Builder decisions presented for approval

- One singleton per local library; no account/cloud system is introduced.
- Ordinary/spelling learning remains independent with a separate default profile.
- Source switching accumulates vocabulary; contexts retain the last version that
  actually contained each word, not later unrelated edits.
- Normalized shared snapshots, not one full source copy per word.
- Case/punctuation canonical keys and the explicit extraction policy in section 4;
  no dictionary, language-model or input-correction gate.
- Managed word forms, explicit deletion plus durable exclusions, and ordinary
  rename/delete behavior for the singleton.
- Per-form `questionTitle`, with import-time variant inheritance only.
- Logical-card counts in ordinary decks, word counts in spelling, and an explicit
  spelling subset in global totals.
- Schema 9 / backup format 2, with a real backward reader for format 1, and v0.2.0.

These builder proposals were presented with the document and are part of the
approved working design. They are not independently originated user requirements.

The disposable practice-input requirement in section 6 is an explicit user
decision, not a builder proposal: input must not survive leaving the card.

## Research references

- [unicode-segmentation and UAX #29 API](https://docs.rs/unicode-segmentation/latest/unicode_segmentation/trait.UnicodeSegmentation.html)
- [Unicode text segmentation standard](https://unicode.org/reports/tr29/)
- [ICU4X word segmentation](https://docs.rs/icu_segmenter/latest/icu_segmenter/struct.WordSegmenter.html)
- [linkify URL/email spans](https://docs.rs/linkify/latest/linkify/)
- [Unicode script properties](https://docs.rs/unicode-script/latest/unicode_script/)
- [English Speller Database](https://github.com/en-wl/wordlist)
- [Spellbook scope and maturity](https://docs.rs/crate/spellbook/latest)
- [nlprule](https://github.com/bminixhofer/nlprule)
