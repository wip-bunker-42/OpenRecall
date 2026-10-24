use super::Store;
use crate::{AppResult, backup::format::BackupData};
use rusqlite::params;
fn ms(s: &str) -> AppResult<i64> {
    Ok(chrono::DateTime::parse_from_rfc3339(s)
        .map_err(|_| crate::AppError::new("invalidBackup"))?
        .timestamp_millis())
}
impl Store {
    pub(crate) fn snapshot(&self) -> AppResult<BackupData> {
        Ok(BackupData {
            settings: self.settings()?,
            library_revision: self.revision()?,
            models: self.all("models")?,
            profiles: self.all("profiles")?,
            decks: self.all("decks")?,
            cards: self.all("cards")?,
            sessions: self.all("sessions")?,
            reviews: self.all("reviews")?,
            actions: self.all("actions")?,
            undo_records: self.all("undo_records")?,
            training_runs: self.all("training_runs")?,
        })
    }
    pub(crate) fn replace_snapshot(&self, d: &BackupData) -> AppResult<()> {
        self.conn.execute_batch("DELETE FROM undo_records;DELETE FROM actions;DELETE FROM reviews;DELETE FROM sessions;DELETE FROM cards;DELETE FROM decks;DELETE FROM training_runs;DELETE FROM profiles;DELETE FROM models;DELETE FROM metadata;")?;
        self.conn.execute(
            "UPDATE settings SET payload=?1 WHERE id=1",
            [serde_json::to_string(&d.settings)?],
        )?;
        self.conn.execute(
            "INSERT INTO metadata(key,value) VALUES('libraryRevision',?1)",
            [&d.library_revision],
        )?;
        for m in &d.models {
            self.put("models", &m.id, m)?;
        }
        for p in &d.profiles {
            self.put("profiles", &p.id, p)?;
        }
        for x in &d.decks {
            self.conn.execute(
                "INSERT INTO decks(id,profile_id,payload) VALUES(?1,?2,?3)",
                params![x.id, x.profile_id, serde_json::to_string(x)?],
            )?;
        }
        for x in &d.cards {
            self.conn.execute("INSERT INTO cards(id,deck_id,due_ms,created_ms,review_count,payload) VALUES(?1,?2,?3,?4,?5,?6)",params![x.id,x.deck_id,ms(&x.due_at)?,ms(&x.created_at)?,x.review_count,serde_json::to_string(x)?])?;
        }
        for x in &d.sessions {
            self.conn.execute(
                "INSERT INTO sessions(id,deck_id,status,payload) VALUES(?1,?2,?3,?4)",
                params![x.id, x.deck_id, x.status, serde_json::to_string(x)?],
            )?;
        }
        for x in &d.reviews {
            self.conn.execute("INSERT INTO reviews(id,card_id,deck_id,session_id,at_ms,model_revision,payload) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![x.id,x.card_id,x.deck_id,x.session_id,ms(&x.reviewed_at)?,x.model_revision,serde_json::to_string(x)?])?;
        }
        for x in &d.actions {
            self.conn.execute(
                "INSERT INTO actions(id,session_id,payload) VALUES(?1,?2,?3)",
                params![x.id, x.session_id, serde_json::to_string(x)?],
            )?;
        }
        for x in &d.undo_records {
            self.conn.execute(
                "INSERT INTO undo_records(id,session_id,payload) VALUES(?1,?2,?3)",
                params![x.id, x.session_id, serde_json::to_string(x)?],
            )?;
        }
        for x in &d.training_runs {
            self.conn.execute(
                "INSERT INTO training_runs(id,profile_id,payload) VALUES(?1,?2,?3)",
                params![x.id, x.profile_id, serde_json::to_string(x)?],
            )?;
        }
        Ok(())
    }
}
