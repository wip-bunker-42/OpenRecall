# OpenRecall

An Arabic, dark, keyboard-accessible desktop flashcard app for personal spaced
repetition. Built with Tauri 2, React/TypeScript, SQLite and a native FSRS-7 engine.
Windows 11 x64 is the primary target; Windows 10 x64, macOS ARM64/Intel and Linux
x64 are secondary targets.

[Download releases](https://github.com/wip-bunker-42/OpenRecall/releases)
· [Arabic usage guide](docs/user-guide.ar.md)
· [Architecture](docs/development/architecture.md)
· [Contributing](docs/development/contributing.md)

## Study workflow

- Create, rename and delete decks. Each deck shows logical card counts, due/new
  cards, review history, forecasts and scheduling settings.
- Create/edit/delete cards, including any number of alternative wordings. All
  forms belong to one logical card and share its memory state and due date.
- Forms are shuffled without replacement through a persistent bag. With more
  than one form, the boundary between cycles avoids an immediate repeat. Revealing
  or switching filters does not consume a form; grading does.
- Start with all due cards, including new and due learning cards, ordered by due
  time. Filter to never-reviewed or previously-reviewed cards. Cards becoming due
  during an active session join its queue without interrupting the current question.
- Reveal the answer, then choose Again/Hard/Good/Easy using Arabic labels. Each
  rating's accessible name includes its approximate interval; the committed result
  shows the actual next due time. Undo restores the previous question, answer,
  learning state, form bag, queue and session counters.
- Session summaries distinguish attempts, unique cards, ratings and active study
  time. Review headings are the labels `السؤال` (H1), `الإجابة` (H2) and
  `الملاحظات` (H3 when notes exist). Card content remains ordinary text.

Statistics, card lists and advanced controls start collapsed. The interface uses
Arabic RTL navigation and `dir="auto"` for multilingual card content. Noto Sans
Arabic is bundled locally. Text size, reduced motion and review shortcuts are
configurable.

## Native scheduling and optimization

The engine is pinned to upstream commit
[`0fd59949e9168af7194418c2dba99836b5548e87`](https://github.com/open-spaced-repetition/fsrs-rs/tree/0fd59949e9168af7194418c2dba99836b5548e87).
That revision contains the 34-parameter FSRS-7 model, including `stability_fast`,
although its crate package version still says 6.6.2. The published registry crate
is not substituted based on a version number alone. Numerical scheduling goldens,
intraday memory and the real optimizer are tested in Rust.

Named optimization profiles can serve one or several selected decks. Detaching a
deck preserves its current model and creates independent future training. Training
uses only the explicit membership of that profile. Schedule penalties default on.
Minimum data requirements include 400 usable nonfirst observations, 50 cards,
seven study days, both recall outcomes, and a chronological held-out set of at
least 80 observations from 10 cards. Candidates require improved held-out log loss
with bounded RMSE regression and explicit application. Changed history, membership,
configuration or baseline invalidates stale candidates.

Defaults: Cairo timezone, study day beginning at 01:00 local time, retention 90%,
minimum interval 60 seconds and maximum interval 36,500 days. Advanced fields and
per-deck overrides have validated bounds. Changing a model or policy preserves
already saved due dates; subsequent reviews use the new model/policy.

## JSON import and complete backups

Card import accepts UTF-8 JSON, including a BOM, up to 50 MiB:

```json
[
  {
    "front": "ما معنى الاسترجاع النشط؟",
    "back": "محاولة استحضار المعلومة قبل رؤية الإجابة.",
    "notes": "أجب أولًا ثم اعرض الإجابة.",
    "variants": [
      {
        "front": "اشرح الاسترجاع النشط بكلماتك.",
        "back": "استحضار المعلومة من الذاكرة قبل الاطلاع عليها.",
        "notes": ""
      }
    ]
  }
]
```

`variants` can be absent or empty. Preview shows invalid rows/forms and duplicate
matches before any library write. Select valid cards and explicitly exclude bad
forms. Duplicate identity uses NFC-normalized, whitespace-collapsed question and
answer across the library; case, punctuation and diacritics remain significant.
Notes do not create a distinct card identity. Import commits atomically against
the preview's library revision.

Complete backups are a separate versioned UTF-8 JSON format, currently version 1
with a shared 256 MiB export/restore limit. They include settings, decks, forms,
progress, review/action/undo history, optimization profiles, model provenance and
training reports. They are plaintext, with a canonical payload digest for
corruption detection. Export finalizes a collision-safe file in Downloads, or
opens a native save dialog if that location is unavailable. Restore previews its
counts, requires full-replacement confirmation and writes a safety backup first.
Invalid, corrupted or future-format data does not replace the current library.

## Installation, offline use and updates

The normal Windows NSIS installer uses an existing WebView2 runtime or downloads
it during initial setup. The larger `offline-setup.exe` embeds the Evergreen
WebView2 installer for initial setup without an Internet connection. SQLite,
fonts and the frontend are bundled. End users do not install Node, Rust or a
database server. Studying, editing, importing, backups, statistics and training
run locally without network access.

Updates are requested in Settings: check, download, then explicitly confirm
installation. The official Tauri updater verifies package bytes and their signed
version. Installation waits for saved edits, completed/ended review and durable
training state. See [signing and release operations](docs/releases/signing.md).
Updater signing is separate from OS publisher certificates: Windows packages
have no Authenticode publisher identity and macOS packages are ad-hoc signed
unless additional organization credentials are supplied.

Use the Linux AppImage for in-app updates; `.deb` is additional installation
media managed through the system package installer. Linux builds use Ubuntu
22.04 and WebKitGTK 4.1. Secondary-platform builds and actual runtime testing are
reported separately in [verification results](docs/testing/automated-results.md).

## Development and verification

Use Node 24.21, pnpm 12.15 and Rust 1.99 with MSVC Build Tools on Windows or the
[Tauri native prerequisites](https://v2.tauri.app/start/prerequisites/) on macOS/Linux.
See [toolchain decisions](docs/development/toolchain.md).

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
pnpm verify
```

`pnpm verify` runs Rust formatting, strict Clippy, workspace tests, generated DTO
drift, TypeScript, ESLint, frontend formatting/tests, maintenance policy tests,
the frontend production build and workflow linting. Browser/native QA commands
and their environment are documented with the verification results. A checksum-
verified actionlint binary is installed under ignored `target/tools` if needed.

GitHub requires exact-revision quality/native/platform checks. Dependabot opens
dependency PRs; only verified same-repository development patch/minor changes
within the tested policy can merge automatically. Runtime, Rust, Tauri, FSRS,
workflow and security changes require review. A quiet, deduplicated FSRS watcher
tests a newly published crate against the actual core scheduling/optimizer tests
on an isolated runner before opening a migration issue. Release jobs compile
without signing credentials, sign existing bundles, verify all artifacts and
publish one complete updater manifest.

Agent accessibility testing uses automated DOM/keyboard/focus/axe checks.
**NVDA testing belongs exclusively to the human partner.** The agent must never
launch, control or configure NVDA, Narrator or another screen reader. The manual
checklist is delivered independently of source/installer completion.

## License

OpenRecall code is [MIT licensed](LICENSE). Bundled libraries/fonts retain their
own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md) and the complete
license texts under `licenses/`, also included in installed packages.
