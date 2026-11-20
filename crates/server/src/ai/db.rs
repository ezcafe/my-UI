use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiDbError {
    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[derive(Debug)]
struct SqliteCustomizer;

impl r2d2::CustomizeConnection<rusqlite::Connection, rusqlite::Error> for SqliteCustomizer {
    fn on_acquire(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA synchronous = NORMAL;",
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct AiDb {
    pool: Pool<SqliteConnectionManager>,
}

impl AiDb {
    pub fn new(path_str: &str) -> Result<Self, AiDbError> {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(8)
            .connection_customizer(Box::new(SqliteCustomizer))
            .build(manager)?;

        let db = Self { pool };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), AiDbError> {
        let conn = self.pool.get()?;
        let version: i32 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;
        if version < 1 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS article_ai_summaries (
                   entry_id INTEGER PRIMARY KEY,
                   summary TEXT NOT NULL,
                   model TEXT NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 );
                 CREATE INDEX IF NOT EXISTS idx_article_ai_summaries_created_at ON article_ai_summaries(created_at);
                 PRAGMA user_version = 1;",
            )?;
        }
        Ok(())
    }

    pub fn get_user_version(&self) -> Result<i32, AiDbError> {
        let conn = self.pool.get()?;
        let version: i32 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;
        Ok(version)
    }

    pub async fn get_summary(&self, entry_id: i64) -> Option<String> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().ok()?;
            conn.query_row(
                "SELECT summary FROM article_ai_summaries WHERE entry_id = ?1",
                params![entry_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
        .await
        .ok()
        .flatten()
    }

    pub async fn save_summary(&self, entry_id: i64, summary: &str, model: &str) -> Result<(), AiDbError> {
        let pool = self.pool.clone();
        let summary = summary.to_string();
        let model = model.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), AiDbError> {
            let conn = pool.get()?;
            conn.execute(
                "INSERT INTO article_ai_summaries (entry_id, summary, model, created_at)
                 VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(entry_id) DO UPDATE SET
                   summary = excluded.summary,
                   model = excluded.model,
                   created_at = excluded.created_at;",
                params![entry_id, summary, model],
            )?;
            Ok(())
        })
        .await??;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_ai_db_initializes_schema_and_user_version() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        let version = db.get_user_version().expect("get user version");
        assert_eq!(version, 1);
    }

    #[tokio::test]
    async fn test_ai_db_concurrent_pool_reads_under_wal() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = Arc::new(AiDb::new(path).expect("AiDb init failed"));

        db.save_summary(101, "Summary 101", "gpt-4o-mini").await.expect("save summary");

        let mut handles = Vec::new();
        for _ in 0..10 {
            let db_clone = db.clone();
            handles.push(tokio::spawn(async move {
                let res = db_clone.get_summary(101).await;
                assert_eq!(res.as_deref(), Some("Summary 101"));
            }));
        }

        for h in handles {
            h.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_ai_db_upsert_on_conflict_updates_summary() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        db.save_summary(202, "Initial Summary", "gpt-4o-mini").await.expect("save 1");
        assert_eq!(db.get_summary(202).await.as_deref(), Some("Initial Summary"));

        db.save_summary(202, "Updated Summary", "claude-3-5").await.expect("save 2");
        assert_eq!(db.get_summary(202).await.as_deref(), Some("Updated Summary"));
    }
}
