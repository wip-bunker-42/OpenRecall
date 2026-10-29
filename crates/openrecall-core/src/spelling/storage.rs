use crate::{AppResult, domain::spelling::*};
use rusqlite::{Connection, params};

fn query<T: serde::de::DeserializeOwned>(db: &Connection, sql: &str) -> AppResult<Vec<T>> {
    db.prepare(sql)?
        .query_map([], |row| row.get::<_, String>(0))?
        .map(|row| Ok(serde_json::from_str(&row?)?))
        .collect()
}

pub(crate) fn load(db: &Connection) -> AppResult<SpellingBackupData> {
    Ok(SpellingBackupData {
        configs: query(
            db,
            "SELECT json_object('deckId',deck_id,'sourceDeckId',source_deck_id,'sourceDeckName',source_deck_name,'extractionPolicyVersion',extraction_policy_version,'revision',revision) FROM spelling_configs ORDER BY rowid",
        )?,
        words: query(
            db,
            "SELECT json_object('cardId',card_id,'deckId',deck_id,'canonical',canonical,'display',display) FROM spelling_words ORDER BY rowid",
        )?,
        snapshots: query(db, "SELECT payload FROM spelling_snapshots ORDER BY rowid")?,
        origins: query(
            db,
            "SELECT json_object('spellingCardId',spelling_card_id,'sourceCardId',source_card_id,'sourceFormId',source_form_id,'snapshotId',snapshot_id,'firstSeenAt',first_seen_at) FROM spelling_origins ORDER BY rowid",
        )?,
        exclusions: query(
            db,
            "SELECT json_object('deckId',deck_id,'canonical',canonical,'excludedAt',excluded_at) FROM spelling_exclusions ORDER BY rowid",
        )?,
    })
}
pub(crate) fn save_config(db: &Connection, x: &SpellingConfig) -> AppResult<()> {
    db.execute("INSERT INTO spelling_configs(deck_id,source_deck_id,source_deck_name,extraction_policy_version,revision) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(deck_id) DO UPDATE SET source_deck_id=excluded.source_deck_id,source_deck_name=excluded.source_deck_name,extraction_policy_version=excluded.extraction_policy_version,revision=excluded.revision",params![x.deck_id,x.source_deck_id,x.source_deck_name,x.extraction_policy_version,x.revision])?;
    Ok(())
}
pub(crate) fn save_word(db: &Connection, x: &SpellingWord) -> AppResult<()> {
    db.execute(
        "INSERT INTO spelling_words(card_id,deck_id,canonical,display) VALUES(?1,?2,?3,?4)",
        params![x.card_id, x.deck_id, x.canonical, x.display],
    )?;
    Ok(())
}
pub(crate) fn save_snapshot(db: &Connection, x: &SourceSnapshot) -> AppResult<()> {
    db.execute("INSERT INTO spelling_snapshots(id,spelling_deck_id,content_sha256,payload) VALUES(?1,?2,?3,?4)",params![x.id,x.spelling_deck_id,x.content_sha256,serde_json::to_string(x)?])?;
    Ok(())
}
pub(crate) fn save_origin(db: &Connection, x: &WordOrigin) -> AppResult<()> {
    db.execute("INSERT INTO spelling_origins(spelling_card_id,source_card_id,source_form_id,snapshot_id,first_seen_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(spelling_card_id,source_card_id,source_form_id) DO UPDATE SET snapshot_id=excluded.snapshot_id",params![x.spelling_card_id,x.source_card_id,x.source_form_id,x.snapshot_id,x.first_seen_at])?;
    Ok(())
}
pub(crate) fn save_exclusion(db: &Connection, x: &SpellingExclusion) -> AppResult<()> {
    db.execute("INSERT INTO spelling_exclusions(deck_id,canonical,excluded_at) VALUES(?1,?2,?3) ON CONFLICT(deck_id,canonical) DO NOTHING",params![x.deck_id,x.canonical,x.excluded_at])?;
    Ok(())
}
pub(crate) fn insert_all(db: &Connection, data: &SpellingBackupData) -> AppResult<()> {
    for x in &data.configs {
        save_config(db, x)?;
    }
    for x in &data.words {
        save_word(db, x)?;
    }
    for x in &data.snapshots {
        save_snapshot(db, x)?;
    }
    for x in &data.origins {
        save_origin(db, x)?;
    }
    for x in &data.exclusions {
        save_exclusion(db, x)?;
    }
    Ok(())
}
