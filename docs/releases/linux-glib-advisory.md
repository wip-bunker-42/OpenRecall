# Linux GLib dependency assessment

Dependabot reports medium-severity GHSA-wrw7-89jp-8q8g / RUSTSEC-2024-0429 in glib 0.18.5. The locked Linux target requires it through GTK3, WebKitGTK and Tauri. Windows and macOS dependency graphs do not include this crate.

The affected API is `VariantStrIter`, created by `Variant::array_iter_str`. Inspection of the application and downloaded locked Linux dependency sources found references only in GLib's own definitions, examples and tests. No production call site was identified. This is a reachability assessment, not proof that every possible execution is unaffected, and the dependency is not patched.

The upstream fix is available in glib 0.20 and newer. GTK3 requires the 0.18 dependency line, so a direct version replacement is incompatible. Tauri tracks this as an upstream Linux dependency limitation. This release retains the compatible dependency and discloses the residual Linux risk rather than introducing a separately maintained GTK fork. A missed or future call to the affected iterator could cause undefined behavior or a crash. Any relevant dependency/API change must reassess this conclusion and the availability of a compatible fix.

The Dependabot alert remains open and security updates stay enabled. Linux is a secondary target whose packages are built in CI; Linux runtime testing has not been performed here.

Sources: [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), [upstream fix](https://github.com/gtk-rs/gtk-rs-core/pull/1343), [Tauri dependency issue](https://github.com/tauri-apps/tauri/issues/12048).
