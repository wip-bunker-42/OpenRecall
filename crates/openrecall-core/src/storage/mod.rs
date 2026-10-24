use crate::{AppError, AppResult, domain::settings::Settings};
use rusqlite::Connection;
use std::{path::Path, time::Duration};
pub(crate) const SCHEMA_VERSION: u32 = 2;
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
        if version < SCHEMA_VERSION {
            let tx = conn.transaction()?;
            if version < 1 {
                tx.execute_batch(include_str!("migrations/001_settings.sql"))?;
                tx.execute(
                    "INSERT INTO settings(id,payload) VALUES(1,?1)",
                    [serde_json::to_string(&Settings::default())?],
                )?;
            }
            if version < 2 {
                tx.execute_batch(include_str!("migrations/002_library.sql"))?;
            }
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
    pub fn get<T: serde::de::DeserializeOwned>(&self, table: &str, id: &str) -> AppResult<T> {
        use rusqlite::OptionalExtension;
        let payload: Option<String> = self
            .conn
            .query_row(
                &format!("SELECT payload FROM {table} WHERE id=?1"),
                [id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(serde_json::from_str(
            &payload.ok_or_else(|| AppError::new("notFound"))?,
        )?)
    }
    pub fn all<T: serde::de::DeserializeOwned>(&self, table: &str) -> AppResult<Vec<T>> {
        let mut statement = self
            .conn
            .prepare(&format!("SELECT payload FROM {table} ORDER BY rowid"))?;
        let rows = statement.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }
    pub fn put<T: serde::Serialize>(&self, table: &str, id: &str, value: &T) -> AppResult<()> {
        self.conn.execute(&format!("INSERT INTO {table}(id,payload) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload"), rusqlite::params![id,serde_json::to_string(value)?])?;
        Ok(())
    }
    pub fn touch(&self) -> AppResult<()> {
        self.conn.execute("INSERT INTO metadata(key,value) VALUES('libraryRevision',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [uuid::Uuid::new_v4().to_string()])?;
        Ok(())
    }
    pub fn revision(&self) -> AppResult<String> {
        use rusqlite::OptionalExtension;
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='libraryRevision'",
                [],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(self.settings()?.revision))
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
