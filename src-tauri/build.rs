fn main() {
    tauri_build::build();
    // Tauri embeds the app manifest for its main binary. Cargo's standalone
    // integration tests/examples also link the updater's native dialog code.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        for target in ["tests", "examples"] {
            println!("cargo:rustc-link-arg-{target}=/MANIFEST:EMBED");
            println!(
                "cargo:rustc-link-arg-{target}=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
            );
        }
    }
}
