use tauri_plugin_updater::{Updater, UpdaterExt};
pub fn offline_verifier(public_key: &str, target: &str) -> Updater {
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert(
        "updater".into(),
        serde_json::json!({"pubkey":public_key,"requireSignedVersion":true}),
    );
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)
        .unwrap();
    app.updater_builder()
        .target(target)
        .version_comparator(|_, _| true)
        .endpoints(vec!["https://example.invalid/no-network".parse().unwrap()])
        .unwrap()
        .configure_client(|_| panic!("offline verification must not access the network"))
        .build()
        .unwrap()
}
