# Third-party notices

OpenRecall's own code and original icon are MIT licensed. Third-party software
and fonts retain their respective copyrights and licenses. Installed packages
include this notice and the full texts in the `licenses` resource directory.

- [Rust library license texts](licenses/RUST.txt) cover the four production
  targets and their transitive runtime dependencies. Development/build-only
  dependencies are excluded from this catalog.
- [Frontend and font license texts](licenses/FRONTEND.txt) include React,
  React DOM, Scheduler, Tauri's JavaScript API and Noto Sans Arabic.
- [Exact upstream source locations](licenses/SOURCES.md) and the
  [dependency catalog](licenses/manifest.json) identify the versions and sources.
  Unmodified MPL-2.0 crate source archives are additionally provided under
  `licenses/sources/` and bundled with the application.

FSRS comes from the pinned open-spaced-repetition/fsrs-rs Git revision documented
in README and Cargo.lock. Its package version alone does not identify the FSRS-7
model. The font is unmodified Noto Sans Arabic, copyright The Noto Project Authors,
distributed under SIL Open Font License 1.1. Its complete license accompanies the
bundled font files.

The offline Windows installer embeds Microsoft's unmodified Evergreen WebView2
Runtime installer. WebView2 is Microsoft software governed by its own terms;
see [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/).
Operating-system components and system WebKitGTK remain their vendors' software.

To refresh this catalog, use cargo-about 0.9.2 with `about.toml`:

```sh
cargo about generate --workspace --locked --fail --format json -o target/rust-licenses.json
node scripts/generate-notices.mjs target/rust-licenses.json
node scripts/check-notices.mjs
```

The checked-in catalog omits local machine paths. Production npm checks compare
the production package graph, so development-only dependency updates do not cause
unrelated license drift. Rust graph changes require catalog regeneration.
