#[path = "../test-support/mod.rs"]
mod support;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 4 {
        return Err("Usage: verify_update PACKAGE SIGNATURE VERSION PLATFORM".into());
    }
    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))?;
    let key = config["plugins"]["updater"]["pubkey"]
        .as_str()
        .ok_or("missing public key")?;
    let signature = std::fs::read_to_string(&args[1])?;
    let updater = support::offline_verifier(key, &args[3]);
    let manifest = serde_json::json!({"version":args[2],"platforms":{&args[3]:{"url":"https://example.invalid/verified-artifact","signature":signature.trim()}}});
    let update = updater.restore_update(manifest)?.ok_or("no candidate")?;
    update.verify(&std::fs::read(&args[0])?)?;
    println!("Signature and signed version verified for {}", args[3]);
    Ok(())
}
