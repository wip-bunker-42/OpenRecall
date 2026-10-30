# Toolchain

The initial build uses stable Rust 1.99.0 (MSVC x64), edition 2024, with the installed Visual Studio 18 C++ Build Tools. Core requirements are Rust 1.90 or newer. Node 24 is the intended frontend runtime; pnpm and the frontend dependency set will be compatibility-checked together before installation.

Frontend compatibility check (2026-10-25): Node 24.21, pnpm 12.15, React/React DOM 19.3.1, Vite 8.3.5, plugin-react 6.1.2, Vitest 5.0.5 and JSDOM 30.1.3 have compatible engines/peers. TypeScript is deliberately pinned to 6.0.3: the current typescript-eslint 8.72 supports TypeScript below 6.1, so its parser does not yet support TypeScript 7. ESLint 10.13 is supported by that parser. Cargo and pnpm lockfiles pin the resolved dependency graph. Noto Sans Arabic is bundled locally; no font network request is needed.

Tauri JavaScript API/CLI 2.12.4, native Tauri 2.12.5 and tauri-build 2.7.4 are the compatible current stable releases queried from their official registries. The standard Windows host uses system WebView2 and NSIS. Cross-platform prerequisites are documented by [Tauri](https://v2.tauri.app/start/prerequisites/).

SQLite is embedded through rusqlite 0.41. The native FSRS dependency is pinned to the approved upstream commit. Rand 0.9 is selected to match that engine's supported dependency line. Cargo.lock records exact resolved versions. Users do not need a separately installed Rust, Node or database runtime to run release installers.

Packaging uses tauri-plugin-updater 2.14.0 with signed-version enforcement and
tauri-plugin-opener 2.7.2 for revealing only exports recorded by native state.
Restart uses Tauri's core AppHandle API, so no JavaScript process permission or
additional process plugin is needed. The desktop library emits an rlib used by
the executable; unused mobile static/dynamic library outputs are omitted.
Windows release builds explicitly retain Tauri's static Visual C++ runtime.
The offline NSIS variant embeds the WebView2 runtime installer.

Node declarations are pinned to @types/node 24.19.2 to match the Node 24 runtime.
GitHub setup actions use immutable commits verified from their current release
tags. Build jobs use standard hosted runners; macOS ARM64 limits Cargo parallelism
to fit its available memory. Release compilation receives no signing credential;
the separate `tauri bundle` step signs the already-built reviewed binary.

Spelling extraction policy 1 pins `unicode-segmentation` 1.13.3,
`unicode-script` 0.5.8 and `linkify` 0.11.0. Their current stable versions and
MIT OR Apache-2.0 licenses were checked against crates.io on 2026-10-29.
unicode-segmentation declares Rust 1.85; the other two do not declare an MSRV.
All three are compiled with the existing supported toolchain. These libraries
provide [Unicode word boundaries](https://docs.rs/unicode-segmentation/1.13.3/),
[script properties](https://docs.rs/unicode-script/0.5.8/) and
[local URL/email detection](https://docs.rs/linkify/0.11.0/). No dictionary,
language model or network lookup participates in extraction. Dependency or
policy changes must run the frozen `spelling_extract` corpus before changing
persisted provenance semantics; unrelated FSRS and toolchain versions stay fixed.
