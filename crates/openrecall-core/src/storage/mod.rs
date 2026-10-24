use crate::{AppError, AppResult, domain::settings::Settings};
use rusqlite::Connection;
use std::{path::Path, time::Duration};
pub(crate) const SCHEMA_VERSION: u32 = 1;
pub(crate) struct Store {
    pub conn: Connection,
}
impl Store {
    pub fn open(path: &Path) -> AppResult<Self> {
        let mut conn = Connection::open(path)?;
        let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(AppError::new("unsupportedSchema"));
        }
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")?;
        if version == 0 {
            let tx = conn.transaction()?;
            tx.execute_batch(include_str!("migrations/001_settings.sql"))?;
            tx.execute(
                "INSERT INTO settings(id,payload) VALUES(1,?1)",
                [serde_json::to_string(&Settings::default())?],
            )?;
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            tx.commit()?;
        }
        Ok(Self { conn })
    }
    pub fn settings(&self) -> AppResult<Settings> {
        let json: String =
            self.conn
                .query_row("SELECT payload FROM settings WHERE id=1", [], |r| r.get(0))?;
        Ok(serde_json::from_str(&json)?)
    }
    pub fn begin(&self) -> AppResult<()> {
        self.conn.execute_batch("SAVEPOINT app_command")?;
        Ok(())
    }
    pub fn finish<T>(&self, result: AppResult<T>) -> AppResult<T> {
        match result {
            Ok(value) => {
                self.conn.execute_batch("RELEASE app_command")?;
                Ok(value)
            }
            Err(error) => {
                self.conn
                    .execute_batch("ROLLBACK TO app_command; RELEASE app_command")?;
                Err(error)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_commands_roll_back_and_foreign_keys_are_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("store.sqlite3")).unwrap();
        let before = store.settings().unwrap();
        store.begin().unwrap();
        store
            .conn
            .execute("UPDATE settings SET payload='broken'", [])
            .unwrap();
        let _: AppResult<()> = store.finish(Err(AppError::new("injectedFailure")));
        assert_eq!(store.settings().unwrap(), before);
        assert_eq!(
            store
                .conn
                .pragma_query_value(None, "foreign_keys", |r| r.get::<_, u32>(0))
                .unwrap(),
            1
        );
    }
}
