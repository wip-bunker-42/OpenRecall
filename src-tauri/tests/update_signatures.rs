#[path = "../test-support/mod.rs"]
mod support;
#[test]
fn official_updater_verifies_real_signed_fixture_and_rejects_tampering_and_version_substitution() {
    let key = include_str!("../../tests/fixtures/updates/public-key.txt").trim();
    let signature = include_str!("../../tests/fixtures/updates/payload.txt.sig").trim();
    let bytes = include_bytes!("../../tests/fixtures/updates/payload.txt");
    let updater = support::offline_verifier(key, "fixture");
    let mut manifest = serde_json::json!({"version":"0.0.0","platforms":{"fixture":{"url":"https://example.invalid/fixture","signature":signature}}});
    let update = updater.restore_update(manifest.clone()).unwrap().unwrap();
    update.verify(bytes).unwrap();
    assert!(update.verify(b"modified payload").is_err());
    manifest["version"] = serde_json::json!("99.0.0");
    assert!(
        updater
            .restore_update(manifest.clone())
            .unwrap()
            .unwrap()
            .verify(bytes)
            .is_err()
    );
    manifest["platforms"]["fixture"]["signature"] = serde_json::json!("invalid");
    assert!(
        updater
            .restore_update(manifest)
            .unwrap()
            .unwrap()
            .verify(bytes)
            .is_err()
    );
    assert!(
        updater
            .restore_update(serde_json::json!({"version":"bad","platforms":{}}))
            .is_err()
    );
}
