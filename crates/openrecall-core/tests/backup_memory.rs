mod common;
use common::*;
use openrecall_core::backup::format::{BackupEnvelope, digest};
use serde_json::json;

#[test]
fn restored_memory_must_satisfy_the_scheduler_invariant_before_replacing_the_library() {
    let mut f = Fixture::new();
    let deck = f.deck("memory bounds");
    let card = f.card(&deck, "question", "answer");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = &start["payload"]["sessionId"];
    let revealed = f.call("revealAnswer", json!({"sessionId":sid}));
    let frame = &revealed["payload"];
    f.call("rateCard", json!({"sessionId":sid,"presentationId":frame["presentationId"],"expectedCardRevision":frame["cardRevision"],"rating":"easy","actionId":uuid::Uuid::new_v4()}));
    let original = f.dir.path().join("memory.json");
    f.app.export_snapshot(&original).unwrap();
    let valid: BackupEnvelope = serde_json::from_slice(&std::fs::read(&original).unwrap()).unwrap();
    let unchanged = f.call("getCard", json!({"cardId":card}));
    for (mode, value) in [(0, 0.0), (0, 36501.0), (1, 36501.0)] {
        let mut bad = valid.clone();
        let memory = bad.data.cards[0].memory.as_mut().unwrap();
        if mode == 0 {
            memory.stability_fast = value;
        } else {
            memory.stability = value;
        }
        bad.payload_sha256 = digest(&bad.data).unwrap();
        let path = f.dir.path().join(format!("invalid-{mode}-{value}.json"));
        std::fs::write(
            &path,
            serde_json::to_vec(&serde_json::to_value(bad).unwrap()).unwrap(),
        )
        .unwrap();
        let result = f.app.preview_restore(&path);
        assert!(
            result.is_err(),
            "preview must reject scheduler-incompatible memory: mode={mode}, value={value}"
        );
        assert_eq!(result.unwrap_err().code, "invalidBackup");
        assert_eq!(f.call("getCard", json!({"cardId":card})), unchanged);
    }
    f.app.preview_restore(&original).unwrap();
}
