use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSummaryRecord {
    pub entry_id: i64,
    pub summary: String,
    pub suggested_title: Option<String>,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleSummaryInput {
    pub entry_id: i64,
    pub summary: String,
    pub suggested_title: Option<String>,
    pub model: String,
}

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
            "PRAGMA busy_timeout = 5000;
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
        conn.execute_batch("PRAGMA busy_timeout = 5000; PRAGMA journal_mode = WAL;")?;
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
        if version < 2 {
            let mut has_suggested_title = false;
            {
                let mut stmt = conn.prepare("PRAGMA table_info(article_ai_summaries);")?;
                let mut rows = stmt.query([])?;
                while let Some(row) = rows.next()? {
                    let col_name: String = row.get(1)?;
                    if col_name == "suggested_title" {
                        has_suggested_title = true;
                        break;
                    }
                }
            }
            if !has_suggested_title {
                let _ = conn.execute_batch("ALTER TABLE article_ai_summaries ADD COLUMN suggested_title TEXT;");
            }

            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS page_executive_briefs (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   brief TEXT NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 );
                 CREATE INDEX IF NOT EXISTS idx_page_executive_briefs_created_at ON page_executive_briefs(created_at);
                 PRAGMA user_version = 2;",
            )?;
        }
        if version < 3 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   page_key TEXT NOT NULL UNIQUE,
                   brief TEXT NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 );
                 CREATE INDEX IF NOT EXISTS idx_page_briefs_v2_created_at ON page_executive_briefs_v2(created_at);
                 PRAGMA user_version = 3;",
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
        self.get_summary_and_title(entry_id)
            .await
            .map(|r| r.summary)
    }

    pub async fn get_summary_and_title(&self, entry_id: i64) -> Option<AiSummaryRecord> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().ok()?;
            conn.query_row(
                "SELECT entry_id, summary, suggested_title, model, created_at FROM article_ai_summaries WHERE entry_id = ?1",
                params![entry_id],
                |row| {
                    Ok(AiSummaryRecord {
                        entry_id: row.get(0)?,
                        summary: row.get(1)?,
                        suggested_title: row.get(2)?,
                        model: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )
            .ok()
        })
        .await
        .ok()
        .flatten()
    }

    pub async fn get_batch_summaries(&self, entry_ids: &[i64]) -> HashMap<i64, AiSummaryRecord> {
        if entry_ids.is_empty() {
            return HashMap::new();
        }
        let pool = self.pool.clone();
        let ids = entry_ids.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().ok()?;
            let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{}", i)).collect();
            let sql = format!(
                "SELECT entry_id, summary, suggested_title, model, created_at FROM article_ai_summaries WHERE entry_id IN ({})",
                placeholders.join(",")
            );
            let mut stmt = conn.prepare(&sql).ok()?;
            let rusqlite_params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
            let rows = stmt.query_map(rusqlite_params.as_slice(), |row| {
                Ok(AiSummaryRecord {
                    entry_id: row.get(0)?,
                    summary: row.get(1)?,
                    suggested_title: row.get(2)?,
                    model: row.get(3)?,
                    created_at: row.get(4)?,
                })
            }).ok()?;

            let mut map = HashMap::new();
            for r in rows.flatten() {
                map.insert(r.entry_id, r);
            }
            Some(map)
        })
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
    }

    pub async fn save_summary(&self, entry_id: i64, summary: &str, model: &str) -> Result<(), AiDbError> {
        self.save_summary_and_title(entry_id, None, summary, model).await
    }

    pub async fn save_summary_and_title(
        &self,
        entry_id: i64,
        suggested_title: Option<&str>,
        summary: &str,
        model: &str,
    ) -> Result<(), AiDbError> {
        let pool = self.pool.clone();
        let suggested_title = suggested_title.map(|s| s.to_string());
        let summary = summary.to_string();
        let model = model.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), AiDbError> {
            let conn = pool.get()?;
            conn.execute(
                "INSERT INTO article_ai_summaries (entry_id, summary, suggested_title, model, created_at)
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(entry_id) DO UPDATE SET
                   summary = excluded.summary,
                   suggested_title = excluded.suggested_title,
                   model = excluded.model,
                   created_at = excluded.created_at;",
                params![entry_id, summary, suggested_title, model],
            )?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn save_page_brief(&self, brief: &str) -> Result<(), AiDbError> {
        let pool = self.pool.clone();
        let brief = brief.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), AiDbError> {
            let conn = pool.get()?;
            conn.execute(
                "INSERT INTO page_executive_briefs (brief, created_at)
                 VALUES (?1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));",
                params![brief],
            )?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn get_latest_page_brief(&self) -> Option<String> {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().ok()?;
            conn.query_row(
                "SELECT brief FROM page_executive_briefs ORDER BY id DESC LIMIT 1;",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
        .await
        .ok()
        .flatten()
    }

    pub async fn get_page_brief_by_key(&self, page_key: &str) -> Option<String> {
        let pool = self.pool.clone();
        let key = page_key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().ok()?;
            conn.query_row(
                "SELECT brief FROM page_executive_briefs_v2 WHERE page_key = ?1;",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
        .await
        .ok()
        .flatten()
    }

    pub async fn save_page_brief_by_key(&self, page_key: &str, brief: &str) -> Result<(), AiDbError> {
        let pool = self.pool.clone();
        let key = page_key.to_string();
        let brief = brief.to_string();
        tokio::task::spawn_blocking(move || -> Result<(), AiDbError> {
            let conn = pool.get()?;
            conn.execute(
                "INSERT INTO page_executive_briefs_v2 (page_key, brief, created_at)
                 VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 ON CONFLICT(page_key) DO UPDATE SET
                   brief = excluded.brief,
                   created_at = excluded.created_at;",
                params![key, brief],
            )?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn save_page_brief_with_key(&self, page_key: &str, brief: &str) -> Result<(), AiDbError> {
        self.save_page_brief_by_key(page_key, brief).await
    }

    pub async fn get_unprocessed_entry_ids(&self, entry_ids: &[i64]) -> Vec<i64> {
        if entry_ids.is_empty() {
            return Vec::new();
        }
        let pool = self.pool.clone();
        let ids = entry_ids.to_vec();
        tokio::task::spawn_blocking(move || {
            let conn = pool.get().ok()?;
            let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{}", i)).collect();
            let sql = format!(
                "SELECT entry_id, model FROM article_ai_summaries WHERE entry_id IN ({})",
                placeholders.join(",")
            );
            let mut stmt = conn.prepare(&sql).ok()?;
            let rusqlite_params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
            let mut rows = stmt.query(rusqlite_params.as_slice()).ok()?;

            let mut processed_real_ids = std::collections::HashSet::new();
            while let Some(row) = rows.next().ok().flatten() {
                let entry_id: i64 = row.get(0).ok()?;
                let model: String = row.get(1).ok()?;
                if model != "extractive-fallback" {
                    processed_real_ids.insert(entry_id);
                }
            }

            let unprocessed: Vec<i64> = ids.into_iter().filter(|id| !processed_real_ids.contains(id)).collect();
            Some(unprocessed)
        })
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
    }

    pub async fn save_batch_results(
        &self,
        page_key: Option<(&str, &str)>,
        items: &[ArticleSummaryInput],
    ) -> Result<(), AiDbError> {
        let pool = self.pool.clone();
        let page_key_owned = page_key.map(|(k, b)| (k.to_string(), b.to_string()));
        let items_owned = items.to_vec();
        tokio::task::spawn_blocking(move || -> Result<(), AiDbError> {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            for item in &items_owned {
                tx.execute(
                    "INSERT INTO article_ai_summaries (entry_id, summary, suggested_title, model, created_at)
                     VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                     ON CONFLICT(entry_id) DO UPDATE SET
                       summary = excluded.summary,
                       suggested_title = excluded.suggested_title,
                       model = excluded.model,
                       created_at = excluded.created_at;",
                    params![item.entry_id, item.summary, item.suggested_title, item.model],
                )?;
            }

            if let Some((key, brief)) = &page_key_owned {
                tx.execute(
                    "INSERT INTO page_executive_briefs_v2 (page_key, brief, created_at)
                     VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                     ON CONFLICT(page_key) DO UPDATE SET
                       brief = excluded.brief,
                       created_at = excluded.created_at;",
                    params![key, brief],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn prune_page_briefs(&self, retention_days: i64) -> Result<usize, AiDbError> {
        let pool = self.pool.clone();
        let deleted = tokio::task::spawn_blocking(move || -> Result<usize, AiDbError> {
            let conn = pool.get()?;
            let deleted = conn.execute(
                "DELETE FROM page_executive_briefs_v2
                 WHERE created_at < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', printf('-%d days', ?1));",
                params![retention_days],
            )?;
            Ok(deleted)
        })
        .await??;
        Ok(deleted)
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
        assert_eq!(version, 3);
    }

    #[tokio::test]
    async fn test_migration_v1_to_v2() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();

        // Step 1: Initialize an older v1 database manually
        {
            let conn = rusqlite::Connection::open(path).expect("open raw sqlite");
            conn.execute_batch(
                "CREATE TABLE article_ai_summaries (
                   entry_id INTEGER PRIMARY KEY,
                   summary TEXT NOT NULL,
                   model TEXT NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 );
                 CREATE INDEX idx_article_ai_summaries_created_at ON article_ai_summaries(created_at);
                 INSERT INTO article_ai_summaries (entry_id, summary, model) VALUES (42, 'Old v1 summary', 'test-model');
                 PRAGMA user_version = 1;",
            )
            .expect("init v1 db");
        }

        // Step 2: Open with AiDb::new which should migrate to v3
        let db = AiDb::new(path).expect("AiDb init with migration failed");
        assert_eq!(db.get_user_version().expect("get user version"), 3);

        // Verify existing summary is preserved and suggested_title is None
        let record = db.get_summary_and_title(42).await.expect("record found");
        assert_eq!(record.entry_id, 42);
        assert_eq!(record.summary, "Old v1 summary");
        assert_eq!(record.suggested_title, None);
        assert_eq!(record.model, "test-model");
    }

    #[tokio::test]
    async fn test_batch_summaries_and_page_brief() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        // Save multiple items with and without suggested titles
        db.save_summary_and_title(1, Some("Title 1"), "Summary 1", "model-a")
            .await
            .expect("save 1");
        db.save_summary_and_title(2, None, "Summary 2", "model-a")
            .await
            .expect("save 2");

        // Single get
        let item1 = db.get_summary_and_title(1).await.expect("item 1 found");
        assert_eq!(item1.suggested_title.as_deref(), Some("Title 1"));
        assert_eq!(item1.summary, "Summary 1");

        // Batch get
        let batch = db.get_batch_summaries(&[1, 2, 999]).await;
        assert_eq!(batch.len(), 2);
        assert_eq!(batch.get(&1).unwrap().suggested_title.as_deref(), Some("Title 1"));
        assert_eq!(batch.get(&2).unwrap().suggested_title, None);
        assert!(!batch.contains_key(&999));

        // Empty batch get
        let empty_batch = db.get_batch_summaries(&[]).await;
        assert!(empty_batch.is_empty());

        // Page brief: initially none
        assert_eq!(db.get_latest_page_brief().await, None);

        // Save page briefs
        db.save_page_brief("First executive brief").await.expect("save brief 1");
        assert_eq!(
            db.get_latest_page_brief().await.as_deref(),
            Some("First executive brief")
        );

        db.save_page_brief("Second executive brief").await.expect("save brief 2");
        assert_eq!(
            db.get_latest_page_brief().await.as_deref(),
            Some("Second executive brief")
        );
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

    #[tokio::test]
    async fn test_sqlite_wal_mode_active() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        let conn = db.pool.get().expect("get connection");
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
            .expect("query journal_mode");
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }

    #[tokio::test]
    async fn test_migration_v2_to_v3() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();

        // Step 1: Initialize v2 database manually
        {
            let conn = rusqlite::Connection::open(path).expect("open raw sqlite");
            conn.execute_batch(
                "CREATE TABLE article_ai_summaries (
                   entry_id INTEGER PRIMARY KEY,
                   summary TEXT NOT NULL,
                   suggested_title TEXT,
                   model TEXT NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 );
                 CREATE INDEX idx_article_ai_summaries_created_at ON article_ai_summaries(created_at);
                 CREATE TABLE page_executive_briefs (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   brief TEXT NOT NULL,
                   created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                 );
                 CREATE INDEX idx_page_executive_briefs_created_at ON page_executive_briefs(created_at);
                 PRAGMA user_version = 2;",
            )
            .expect("init v2 db");
        }

        // Step 2: Open with AiDb::new which should migrate to v3
        let db = AiDb::new(path).expect("AiDb init with migration failed");
        assert_eq!(db.get_user_version().expect("get user version"), 3);

        // Validate page_executive_briefs_v2 table exists and index idx_page_briefs_v2_created_at exists
        let conn = db.pool.get().unwrap();
        let table_count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='page_executive_briefs_v2';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);

        let created_at_index_count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='index' AND name='idx_page_briefs_v2_created_at';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(created_at_index_count, 1);

        // Verify redundant idx_page_briefs_v2_key is absent (UNIQUE column has auto-index)
        let redundant_index_count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='index' AND name='idx_page_briefs_v2_key';",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(redundant_index_count, 0);
    }

    #[tokio::test]
    async fn test_page_brief_by_key_upsert() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        let key = "unread:1a2b3c4d5e6f7890";
        assert_eq!(db.get_page_brief_by_key(key).await, None);

        db.save_page_brief_by_key(key, "First brief content")
            .await
            .expect("save page brief");
        assert_eq!(
            db.get_page_brief_by_key(key).await.as_deref(),
            Some("First brief content")
        );

        // Re-saving with same key should update without SQLITE_CONSTRAINT_UNIQUE error
        db.save_page_brief_by_key(key, "Updated brief content")
            .await
            .expect("update page brief on conflict");
        assert_eq!(
            db.get_page_brief_by_key(key).await.as_deref(),
            Some("Updated brief content")
        );
    }

    #[tokio::test]
    async fn test_get_unprocessed_entry_ids() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        // Initially all are unprocessed
        let unprocessed = db.get_unprocessed_entry_ids(&[10, 20, 30]).await;
        assert_eq!(unprocessed, vec![10, 20, 30]);

        // Process 10 with real model
        db.save_summary(10, "Summary 10", "gpt-4o-mini").await.unwrap();
        // Process 20 with extractive-fallback
        db.save_summary(20, "Teaser 20", "extractive-fallback").await.unwrap();

        let unprocessed_after = db.get_unprocessed_entry_ids(&[10, 20, 30]).await;
        // 10 is processed, 20 (extractive fallback) and 30 (missing) are unprocessed
        assert_eq!(unprocessed_after, vec![20, 30]);
    }

    #[tokio::test]
    async fn test_save_batch_results_transaction() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        let items = vec![
            ArticleSummaryInput {
                entry_id: 101,
                summary: "Summary 101".into(),
                suggested_title: Some("Title 101".into()),
                model: "gpt-4o-mini".into(),
            },
            ArticleSummaryInput {
                entry_id: 102,
                summary: "Summary 102".into(),
                suggested_title: None,
                model: "gpt-4o-mini".into(),
            },
        ];
        let key = "unread:batchtest1234";

        db.save_batch_results(Some((key, "Batch top brief")), &items)
            .await
            .expect("save batch results");

        assert_eq!(
            db.get_page_brief_by_key(key).await.as_deref(),
            Some("Batch top brief")
        );

        let rec1 = db.get_summary_and_title(101).await.expect("rec 101");
        assert_eq!(rec1.summary, "Summary 101");
        assert_eq!(rec1.suggested_title.as_deref(), Some("Title 101"));

        let rec2 = db.get_summary_and_title(102).await.expect("rec 102");
        assert_eq!(rec2.summary, "Summary 102");
        assert_eq!(rec2.suggested_title, None);
    }

    #[tokio::test]
    async fn test_prune_page_briefs() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = AiDb::new(path).expect("AiDb init failed");

        // Insert an old brief directly with past timestamp and a recent one
        {
            let conn = db.pool.get().unwrap();
            conn.execute(
                "INSERT INTO page_executive_briefs_v2 (page_key, brief, created_at)
                 VALUES ('old_key', 'Old brief', '2020-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO page_executive_briefs_v2 (page_key, brief, created_at)
                 VALUES ('new_key', 'New brief', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))",
                [],
            )
            .unwrap();
        }

        let pruned_count = db.prune_page_briefs(14).await.expect("prune");
        assert_eq!(pruned_count, 1);
        assert_eq!(db.get_page_brief_by_key("old_key").await, None);
        assert_eq!(
            db.get_page_brief_by_key("new_key").await.as_deref(),
            Some("New brief")
        );
    }
}
