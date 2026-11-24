use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSummaryRecord {
    pub entry_id: i64,
    pub summary: String,
    pub suggested_title: Option<String>,
    pub rewrite: Option<String>,
    pub impact: Option<String>,
    pub fallacies: Option<String>,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleSummaryInput {
    pub entry_id: i64,
    pub summary: String,
    pub suggested_title: Option<String>,
    pub rewrite: Option<String>,
    pub impact: Option<String>,
    pub fallacies: Option<String>,
    pub model: String,
}

#[derive(Debug, Error)]
pub enum AiDbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub const INIT_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS article_ai_summaries (
    entry_id BIGINT PRIMARY KEY,
    summary TEXT NOT NULL,
    suggested_title TEXT,
    rewrite TEXT,
    impact TEXT,
    fallacies TEXT,
    model TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    search_vector tsvector GENERATED ALWAYS AS (
        to_tsvector('english', coalesce(suggested_title, '') || ' ' || coalesce(summary, '') || ' ' || coalesce(rewrite, ''))
    ) STORED
);
ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS impact TEXT;
ALTER TABLE article_ai_summaries ADD COLUMN IF NOT EXISTS fallacies TEXT;
CREATE INDEX IF NOT EXISTS idx_article_summaries_created_at ON article_ai_summaries(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_article_summaries_search_gin ON article_ai_summaries USING GIN(search_vector);

CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
    page_key TEXT PRIMARY KEY,
    brief TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_page_briefs_created_at ON page_executive_briefs_v2(created_at DESC);
"#;

#[derive(sqlx::FromRow)]
struct ArticleSummaryRow {
    entry_id: i64,
    summary: String,
    suggested_title: Option<String>,
    rewrite: Option<String>,
    impact: Option<String>,
    fallacies: Option<String>,
    model: String,
    created_at: DateTime<Utc>,
}

#[derive(Default)]
struct MockData {
    summaries: HashMap<i64, (AiSummaryRecord, DateTime<Utc>)>,
    briefs: HashMap<String, (String, DateTime<Utc>)>,
}

#[derive(Clone)]
enum Backend {
    Pg(sqlx::PgPool),
    Mock(Arc<RwLock<MockData>>),
}

static MOCK_REGISTRY: std::sync::LazyLock<
    std::sync::RwLock<std::collections::HashMap<String, Arc<RwLock<MockData>>>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

#[derive(Clone)]
pub struct AiDb {
    inner: Backend,
}

impl AiDb {
    pub async fn new(database_url: &str) -> Result<Self, AiDbError> {
        if database_url == ":memory:" {
            return Ok(Self::new_mock());
        }
        if database_url.starts_with("mock")
            || database_url.ends_with(".db")
            || database_url.ends_with(".sqlite")
        {
            let mut reg = MOCK_REGISTRY.write().unwrap();
            let mock = reg
                .entry(database_url.to_string())
                .or_insert_with(|| Arc::new(RwLock::new(MockData::default())))
                .clone();
            return Ok(Self {
                inner: Backend::Mock(mock),
            });
        }

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .idle_timeout(std::time::Duration::from_secs(300))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(database_url)
            .await?;
        let db = Self {
            inner: Backend::Pg(pool),
        };
        db.init_schema().await?;
        Ok(db)
    }

    pub fn new_mock() -> Self {
        Self {
            inner: Backend::Mock(Arc::new(RwLock::new(MockData::default()))),
        }
    }

    pub fn from_pool(pool: sqlx::PgPool) -> Self {
        Self {
            inner: Backend::Pg(pool),
        }
    }

    pub fn pool(&self) -> Option<&sqlx::PgPool> {
        match &self.inner {
            Backend::Pg(p) => Some(p),
            Backend::Mock(_) => None,
        }
    }

    pub async fn init_schema(&self) -> Result<(), AiDbError> {
        match &self.inner {
            Backend::Pg(pool) => {
                sqlx::raw_sql(INIT_SCHEMA_SQL).execute(pool).await?;
                Ok(())
            }
            Backend::Mock(_) => Ok(()),
        }
    }

    pub async fn get_summary(&self, entry_id: i64) -> Option<String> {
        match &self.inner {
            Backend::Pg(pool) => {
                let row: Option<(String,)> =
                    sqlx::query_as("SELECT summary FROM article_ai_summaries WHERE entry_id = $1")
                        .bind(entry_id)
                        .fetch_optional(pool)
                        .await
                        .ok()
                        .flatten();
                row.map(|(s,)| s)
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                guard
                    .summaries
                    .get(&entry_id)
                    .map(|(r, _)| r.summary.clone())
            }
        }
    }

    pub async fn get_summary_and_title(&self, entry_id: i64) -> Option<AiSummaryRecord> {
        match &self.inner {
            Backend::Pg(pool) => {
                let row: Option<ArticleSummaryRow> = sqlx::query_as(
                    r#"
                    SELECT entry_id, summary, suggested_title, rewrite, impact, fallacies, model, created_at
                    FROM article_ai_summaries
                    WHERE entry_id = $1
                    "#,
                )
                .bind(entry_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

                row.map(|r| AiSummaryRecord {
                    entry_id: r.entry_id,
                    summary: r.summary,
                    suggested_title: r.suggested_title,
                    rewrite: r.rewrite,
                    impact: r.impact,
                    fallacies: r.fallacies,
                    model: r.model,
                    created_at: r.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                })
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                guard.summaries.get(&entry_id).map(|(r, _)| r.clone())
            }
        }
    }

    pub async fn get_batch_summaries(&self, entry_ids: &[i64]) -> HashMap<i64, AiSummaryRecord> {
        if entry_ids.is_empty() {
            return HashMap::new();
        }
        match &self.inner {
            Backend::Pg(pool) => {
                let rows: Result<Vec<ArticleSummaryRow>, _> = sqlx::query_as(
                    r#"
                    SELECT entry_id, summary, suggested_title, rewrite, impact, fallacies, model, created_at
                    FROM article_ai_summaries
                    WHERE entry_id = ANY($1)
                    "#,
                )
                .bind(entry_ids)
                .fetch_all(pool)
                .await;

                let mut map = HashMap::new();
                if let Ok(rows) = rows {
                    for r in rows {
                        map.insert(
                            r.entry_id,
                            AiSummaryRecord {
                                entry_id: r.entry_id,
                                summary: r.summary,
                                suggested_title: r.suggested_title,
                                rewrite: r.rewrite,
                                impact: r.impact,
                                fallacies: r.fallacies,
                                model: r.model,
                                created_at: r.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            },
                        );
                    }
                }
                map
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                let mut map = HashMap::new();
                for id in entry_ids {
                    if let Some((r, _)) = guard.summaries.get(id) {
                        map.insert(*id, r.clone());
                    }
                }
                map
            }
        }
    }

    pub async fn save_summary(
        &self,
        entry_id: i64,
        summary: &str,
        model: &str,
    ) -> Result<(), AiDbError> {
        self.save_summary_and_title(entry_id, None, summary, model)
            .await
    }

    pub async fn save_summary_and_title(
        &self,
        entry_id: i64,
        suggested_title: Option<&str>,
        summary: &str,
        model: &str,
    ) -> Result<(), AiDbError> {
        self.save_summary_and_title_with_rewrite(entry_id, suggested_title, summary, None, model)
            .await
    }

    pub async fn save_summary_and_title_with_rewrite(
        &self,
        entry_id: i64,
        suggested_title: Option<&str>,
        summary: &str,
        rewrite: Option<&str>,
        model: &str,
    ) -> Result<(), AiDbError> {
        self.save_summary_and_title_with_analysis(
            entry_id,
            suggested_title,
            summary,
            rewrite,
            None,
            None,
            model,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save_summary_and_title_with_analysis(
        &self,
        entry_id: i64,
        suggested_title: Option<&str>,
        summary: &str,
        rewrite: Option<&str>,
        impact: Option<&str>,
        fallacies: Option<&str>,
        model: &str,
    ) -> Result<(), AiDbError> {
        let now = Utc::now();
        match &self.inner {
            Backend::Pg(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO article_ai_summaries (entry_id, summary, suggested_title, rewrite, impact, fallacies, model, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                    ON CONFLICT (entry_id) DO UPDATE SET
                        summary = EXCLUDED.summary,
                        suggested_title = COALESCE(EXCLUDED.suggested_title, article_ai_summaries.suggested_title),
                        rewrite = COALESCE(EXCLUDED.rewrite, article_ai_summaries.rewrite),
                        impact = COALESCE(EXCLUDED.impact, article_ai_summaries.impact),
                        fallacies = COALESCE(EXCLUDED.fallacies, article_ai_summaries.fallacies),
                        model = EXCLUDED.model,
                        created_at = EXCLUDED.created_at
                    "#,
                )
                .bind(entry_id)
                .bind(summary)
                .bind(suggested_title)
                .bind(rewrite)
                .bind(impact)
                .bind(fallacies)
                .bind(model)
                .execute(pool)
                .await?;
                Ok(())
            }
            Backend::Mock(data) => {
                let mut guard = data.write().await;
                let existing = guard.summaries.get(&entry_id).map(|(r, _)| r.clone());
                let final_suggested_title = suggested_title
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().and_then(|r| r.suggested_title.clone()));
                let final_rewrite = rewrite
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().and_then(|r| r.rewrite.clone()));
                let final_impact = impact
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().and_then(|r| r.impact.clone()));
                let final_fallacies = fallacies
                    .map(|s| s.to_string())
                    .or_else(|| existing.as_ref().and_then(|r| r.fallacies.clone()));
                guard.summaries.insert(
                    entry_id,
                    (
                        AiSummaryRecord {
                            entry_id,
                            summary: summary.to_string(),
                            suggested_title: final_suggested_title,
                            rewrite: final_rewrite,
                            impact: final_impact,
                            fallacies: final_fallacies,
                            model: model.to_string(),
                            created_at: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        },
                        now,
                    ),
                );
                Ok(())
            }
        }
    }

    pub async fn save_analysis(
        &self,
        entry_id: i64,
        impact: &str,
        fallacies: &str,
        model: &str,
    ) -> Result<(), AiDbError> {
        match &self.inner {
            Backend::Pg(pool) => {
                sqlx::query(
                    r#"
                    UPDATE article_ai_summaries
                    SET impact = $2, fallacies = $3, model = $4
                    WHERE entry_id = $1
                    "#,
                )
                .bind(entry_id)
                .bind(impact)
                .bind(fallacies)
                .bind(model)
                .execute(pool)
                .await?;
                Ok(())
            }
            Backend::Mock(data) => {
                let mut guard = data.write().await;
                if let Some((rec, _)) = guard.summaries.get_mut(&entry_id) {
                    rec.impact = Some(impact.to_string());
                    rec.fallacies = Some(fallacies.to_string());
                    rec.model = model.to_string();
                }
                Ok(())
            }
        }
    }

    pub async fn save_batch_results(
        &self,
        page_key: Option<(&str, &str)>,
        items: &[ArticleSummaryInput],
    ) -> Result<(), AiDbError> {
        let now = Utc::now();
        match &self.inner {
            Backend::Pg(pool) => {
                let mut tx = pool.begin().await?;

                for item in items {
                    sqlx::query(
                        r#"
                        INSERT INTO article_ai_summaries (entry_id, summary, suggested_title, rewrite, impact, fallacies, model, created_at)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                        ON CONFLICT (entry_id) DO UPDATE SET
                            summary = EXCLUDED.summary,
                            suggested_title = COALESCE(EXCLUDED.suggested_title, article_ai_summaries.suggested_title),
                            rewrite = COALESCE(EXCLUDED.rewrite, article_ai_summaries.rewrite),
                            impact = COALESCE(EXCLUDED.impact, article_ai_summaries.impact),
                            fallacies = COALESCE(EXCLUDED.fallacies, article_ai_summaries.fallacies),
                            model = EXCLUDED.model,
                            created_at = EXCLUDED.created_at
                        "#,
                    )
                    .bind(item.entry_id)
                    .bind(&item.summary)
                    .bind(item.suggested_title.as_deref())
                    .bind(item.rewrite.as_deref())
                    .bind(item.impact.as_deref())
                    .bind(item.fallacies.as_deref())
                    .bind(&item.model)
                    .execute(&mut *tx)
                    .await?;
                }

                if let Some((key, brief)) = page_key {
                    sqlx::query(
                        r#"
                        INSERT INTO page_executive_briefs_v2 (page_key, brief, created_at)
                        VALUES ($1, $2, NOW())
                        ON CONFLICT (page_key) DO UPDATE SET
                            brief = EXCLUDED.brief,
                            created_at = EXCLUDED.created_at
                        "#,
                    )
                    .bind(key)
                    .bind(brief)
                    .execute(&mut *tx)
                    .await?;
                }

                tx.commit().await?;
                Ok(())
            }
            Backend::Mock(data) => {
                let mut guard = data.write().await;
                for item in items {
                    let existing = guard.summaries.get(&item.entry_id).map(|(r, _)| r.clone());
                    let final_suggested_title = item
                        .suggested_title
                        .clone()
                        .or_else(|| existing.as_ref().and_then(|r| r.suggested_title.clone()));
                    let final_rewrite = item
                        .rewrite
                        .clone()
                        .or_else(|| existing.as_ref().and_then(|r| r.rewrite.clone()));
                    let final_impact = item
                        .impact
                        .clone()
                        .or_else(|| existing.as_ref().and_then(|r| r.impact.clone()));
                    let final_fallacies = item
                        .fallacies
                        .clone()
                        .or_else(|| existing.as_ref().and_then(|r| r.fallacies.clone()));
                    guard.summaries.insert(
                        item.entry_id,
                        (
                            AiSummaryRecord {
                                entry_id: item.entry_id,
                                summary: item.summary.clone(),
                                suggested_title: final_suggested_title,
                                rewrite: final_rewrite,
                                impact: final_impact,
                                fallacies: final_fallacies,
                                model: item.model.clone(),
                                created_at: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                            },
                            now,
                        ),
                    );
                }
                if let Some((key, brief)) = page_key {
                    guard
                        .briefs
                        .insert(key.to_string(), (brief.to_string(), now));
                }
                Ok(())
            }
        }
    }

    pub async fn get_unprocessed_entry_ids(&self, entry_ids: &[i64]) -> Vec<i64> {
        if entry_ids.is_empty() {
            return Vec::new();
        }
        match &self.inner {
            Backend::Pg(pool) => {
                let rows: Result<Vec<(i64, String)>, _> = sqlx::query_as(
                    "SELECT entry_id, model FROM article_ai_summaries WHERE entry_id = ANY($1)",
                )
                .bind(entry_ids)
                .fetch_all(pool)
                .await;

                let mut processed = std::collections::HashSet::new();
                if let Ok(rows) = rows {
                    for (entry_id, model) in rows {
                        if model != "extractive-fallback" {
                            processed.insert(entry_id);
                        }
                    }
                }
                entry_ids
                    .iter()
                    .copied()
                    .filter(|id| !processed.contains(id))
                    .collect()
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                let mut processed = std::collections::HashSet::new();
                for id in entry_ids {
                    if let Some((r, _)) = guard.summaries.get(id) {
                        if r.model != "extractive-fallback" {
                            processed.insert(*id);
                        }
                    }
                }
                entry_ids
                    .iter()
                    .copied()
                    .filter(|id| !processed.contains(id))
                    .collect()
            }
        }
    }

    pub async fn search_articles(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<AiSummaryRecord>, AiDbError> {
        let trimmed = query.trim();
        if trimmed.is_empty() || limit <= 0 {
            return Ok(Vec::new());
        }
        let limit = limit.min(1000);
        match &self.inner {
            Backend::Pg(pool) => {
                let rows: Vec<ArticleSummaryRow> = sqlx::query_as(
                    r#"
                    SELECT entry_id, summary, suggested_title, rewrite, impact, fallacies, model, created_at
                    FROM article_ai_summaries
                    WHERE search_vector @@ plainto_tsquery('english', $1)
                    ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC
                    LIMIT $2
                    "#,
                )
                .bind(trimmed)
                .bind(limit)
                .fetch_all(pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|r| AiSummaryRecord {
                        entry_id: r.entry_id,
                        summary: r.summary,
                        suggested_title: r.suggested_title,
                        rewrite: r.rewrite,
                        impact: r.impact,
                        fallacies: r.fallacies,
                        model: r.model,
                        created_at: r.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    })
                    .collect())
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                let terms: Vec<String> = trimmed
                    .to_lowercase()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                let mut matches: Vec<(i64, AiSummaryRecord)> = Vec::new();
                for (r, _) in guard.summaries.values() {
                    let mut score = 0;
                    let text = format!(
                        "{} {} {}",
                        r.suggested_title.as_deref().unwrap_or(""),
                        r.summary,
                        r.rewrite.as_deref().unwrap_or("")
                    )
                    .to_lowercase();
                    for t in &terms {
                        if text.contains(t) {
                            score += 1;
                        }
                    }
                    if score > 0 {
                        matches.push((score, r.clone()));
                    }
                }
                matches.sort_by_key(|a| std::cmp::Reverse(a.0));
                let results = matches
                    .into_iter()
                    .take(limit as usize)
                    .map(|(_, r)| r)
                    .collect();
                Ok(results)
            }
        }
    }

    pub async fn get_recent_summaries(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<AiSummaryRecord>, AiDbError> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        let limit = limit.min(1000);
        match &self.inner {
            Backend::Pg(pool) => {
                let rows: Vec<ArticleSummaryRow> = sqlx::query_as(
                    r#"
                    SELECT entry_id, summary, suggested_title, rewrite, impact, fallacies, model, created_at
                    FROM article_ai_summaries
                    WHERE created_at >= $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(since)
                .bind(limit)
                .fetch_all(pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|r| AiSummaryRecord {
                        entry_id: r.entry_id,
                        summary: r.summary,
                        suggested_title: r.suggested_title,
                        rewrite: r.rewrite,
                        impact: r.impact,
                        fallacies: r.fallacies,
                        model: r.model,
                        created_at: r.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    })
                    .collect())
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                let mut items: Vec<(DateTime<Utc>, AiSummaryRecord)> = guard
                    .summaries
                    .values()
                    .filter(|(_, dt)| *dt >= since)
                    .map(|(r, dt)| (*dt, r.clone()))
                    .collect();
                items.sort_by_key(|a| std::cmp::Reverse(a.0));
                let results = items
                    .into_iter()
                    .take(limit as usize)
                    .map(|(_, r)| r)
                    .collect();
                Ok(results)
            }
        }
    }

    pub async fn get_latest_page_brief(&self) -> Option<String> {
        match &self.inner {
            Backend::Pg(pool) => {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT brief FROM page_executive_briefs_v2 ORDER BY created_at DESC LIMIT 1",
                )
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();
                row.map(|(b,)| b)
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                let mut list: Vec<(DateTime<Utc>, String)> = guard
                    .briefs
                    .values()
                    .map(|(b, dt)| (*dt, b.clone()))
                    .collect();
                list.sort_by_key(|a| std::cmp::Reverse(a.0));
                list.first().map(|(_, b)| b.clone())
            }
        }
    }

    pub async fn get_page_brief_by_key(&self, page_key: &str) -> Option<String> {
        match &self.inner {
            Backend::Pg(pool) => {
                let row: Option<(String,)> = sqlx::query_as(
                    "SELECT brief FROM page_executive_briefs_v2 WHERE page_key = $1",
                )
                .bind(page_key)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();
                row.map(|(b,)| b)
            }
            Backend::Mock(data) => {
                let guard = data.read().await;
                guard.briefs.get(page_key).map(|(b, _)| b.clone())
            }
        }
    }

    pub async fn save_page_brief(&self, brief: &str) -> Result<(), AiDbError> {
        self.save_page_brief_by_key("__latest__", brief).await
    }

    pub async fn save_page_brief_by_key(
        &self,
        page_key: &str,
        brief: &str,
    ) -> Result<(), AiDbError> {
        let now = Utc::now();
        match &self.inner {
            Backend::Pg(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO page_executive_briefs_v2 (page_key, brief, created_at)
                    VALUES ($1, $2, NOW())
                    ON CONFLICT (page_key) DO UPDATE SET
                        brief = EXCLUDED.brief,
                        created_at = EXCLUDED.created_at
                    "#,
                )
                .bind(page_key)
                .bind(brief)
                .execute(pool)
                .await?;
                Ok(())
            }
            Backend::Mock(data) => {
                let mut guard = data.write().await;
                guard
                    .briefs
                    .insert(page_key.to_string(), (brief.to_string(), now));
                Ok(())
            }
        }
    }

    pub async fn save_page_brief_with_key(
        &self,
        page_key: &str,
        brief: &str,
    ) -> Result<(), AiDbError> {
        self.save_page_brief_by_key(page_key, brief).await
    }

    pub async fn prune_page_briefs(&self, retention_days: i64) -> Result<usize, AiDbError> {
        if retention_days < 0 {
            return Ok(0);
        }
        match &self.inner {
            Backend::Pg(pool) => {
                let res = sqlx::query(
                    r#"
                    DELETE FROM page_executive_briefs_v2
                    WHERE created_at < NOW() - ($1 * INTERVAL '1 day')
                    "#,
                )
                .bind(retention_days as f64)
                .execute(pool)
                .await?;
                Ok(res.rows_affected() as usize)
            }
            Backend::Mock(data) => {
                let mut guard = data.write().await;
                let cutoff = Utc::now() - chrono::Duration::days(retention_days);
                let initial_len = guard.briefs.len();
                guard.briefs.retain(|_, (_, dt)| *dt >= cutoff);
                Ok(initial_len - guard.briefs.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_db_connection_and_schema_init() {
        assert!(INIT_SCHEMA_SQL.contains("article_ai_summaries"));
        assert!(INIT_SCHEMA_SQL.contains("search_vector tsvector GENERATED ALWAYS AS"));
        assert!(INIT_SCHEMA_SQL.contains("USING GIN(search_vector)"));
        assert!(INIT_SCHEMA_SQL.contains("idx_article_summaries_created_at"));
        assert!(INIT_SCHEMA_SQL.contains("page_executive_briefs_v2"));
        assert!(INIT_SCHEMA_SQL.contains("idx_page_briefs_created_at"));

        let db = AiDb::new_mock();
        assert!(db.init_schema().await.is_ok());
    }

    #[tokio::test]
    async fn test_ai_db_summary_save_and_retrieve() {
        let db = AiDb::new_mock();

        // 1. Save and retrieve
        db.save_summary_and_title_with_rewrite(
            100,
            Some("Title 100"),
            "Summary 100",
            Some("Rewrite 100"),
            "test-model",
        )
        .await
        .expect("save summary");

        let rec = db.get_summary_and_title(100).await.expect("must exist");
        assert_eq!(rec.entry_id, 100);
        assert_eq!(rec.suggested_title.as_deref(), Some("Title 100"));
        assert_eq!(rec.summary, "Summary 100");
        assert_eq!(rec.rewrite.as_deref(), Some("Rewrite 100"));
        assert_eq!(rec.model, "test-model");

        // 2. Upsert updates summary without losing rewrite if not provided
        db.save_summary_and_title(100, Some("Updated Title"), "Updated Summary", "new-model")
            .await
            .expect("upsert summary");

        let updated = db.get_summary_and_title(100).await.expect("must exist");
        assert_eq!(updated.summary, "Updated Summary");
        assert_eq!(updated.suggested_title.as_deref(), Some("Updated Title"));
        assert_eq!(updated.rewrite.as_deref(), Some("Rewrite 100"));
        assert_eq!(updated.model, "new-model");

        // 3. get_summary
        assert_eq!(
            db.get_summary(100).await,
            Some("Updated Summary".to_string())
        );
        assert_eq!(db.get_summary(999).await, None);
    }

    #[tokio::test]
    async fn test_ai_db_save_and_retrieve_analysis() {
        let db = AiDb::new_mock();
        db.save_summary_and_title_with_analysis(
            200,
            Some("Title 200"),
            "Summary 200",
            Some("Rewrite 200"),
            Some("Impact 200"),
            Some("Fallacy 200"),
            "test-model",
        )
        .await
        .expect("save analysis");

        let rec = db.get_summary_and_title(200).await.expect("must exist");
        assert_eq!(rec.entry_id, 200);
        assert_eq!(rec.suggested_title.as_deref(), Some("Title 200"));
        assert_eq!(rec.summary, "Summary 200");
        assert_eq!(rec.rewrite.as_deref(), Some("Rewrite 200"));
        assert_eq!(rec.impact.as_deref(), Some("Impact 200"));
        assert_eq!(rec.fallacies.as_deref(), Some("Fallacy 200"));
        assert_eq!(rec.model, "test-model");
    }

    #[tokio::test]
    async fn test_ai_db_save_analysis_updates_existing_record() {
        let db = AiDb::new_mock();
        db.save_summary_and_title_with_rewrite(
            201,
            Some("Title 201"),
            "Summary 201",
            Some("Rewrite 201"),
            "batch-model",
        )
        .await
        .expect("save batch rewrite");

        db.save_analysis(
            201,
            "Real-world impact 201",
            "No fallacies detected",
            "analysis-model",
        )
        .await
        .expect("save analysis");

        let rec = db.get_summary_and_title(201).await.expect("must exist");
        assert_eq!(rec.entry_id, 201);
        assert_eq!(rec.summary, "Summary 201");
        assert_eq!(rec.rewrite.as_deref(), Some("Rewrite 201"));
        assert_eq!(rec.impact.as_deref(), Some("Real-world impact 201"));
        assert_eq!(rec.fallacies.as_deref(), Some("No fallacies detected"));
        assert_eq!(rec.model, "analysis-model");
    }

    #[tokio::test]
    async fn test_ai_db_batch_summaries_and_unprocessed() {
        let db = AiDb::new_mock();

        let items = vec![
            ArticleSummaryInput {
                entry_id: 1,
                summary: "Batch summary 1".to_string(),
                suggested_title: Some("Title 1".to_string()),
                rewrite: None,
                impact: None,
                fallacies: None,
                model: "gpt-4o-mini".to_string(),
            },
            ArticleSummaryInput {
                entry_id: 2,
                summary: "Batch summary 2 fallback".to_string(),
                suggested_title: None,
                rewrite: None,
                impact: None,
                fallacies: None,
                model: "extractive-fallback".to_string(),
            },
        ];

        db.save_batch_results(Some(("page-unread-1", "Executive Brief text")), &items)
            .await
            .expect("save batch");

        // Batch summaries retrieval
        let map = db.get_batch_summaries(&[1, 2, 3]).await;
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&1).unwrap().summary, "Batch summary 1");
        assert_eq!(map.get(&2).unwrap().model, "extractive-fallback");
        assert!(!map.contains_key(&3));

        // Page brief saved in transaction
        let brief = db.get_page_brief_by_key("page-unread-1").await;
        assert_eq!(brief.as_deref(), Some("Executive Brief text"));

        // Unprocessed filter: id 1 has real model, id 2 has extractive-fallback, id 3 is missing
        let unprocessed = db.get_unprocessed_entry_ids(&[1, 2, 3]).await;
        assert_eq!(unprocessed, vec![2, 3]);
    }

    #[tokio::test]
    async fn test_ai_db_fts_tsvector_search() {
        let db = AiDb::new_mock();

        db.save_summary_and_title_with_rewrite(
            10,
            Some("Quantum Computing Breakthrough"),
            "Researchers develop a new superconducting qubit.",
            None,
            "test-model",
        )
        .await
        .unwrap();

        db.save_summary_and_title_with_rewrite(
            20,
            Some("Global Economic Trends"),
            "Markets react to inflation and interest rate cuts.",
            None,
            "test-model",
        )
        .await
        .unwrap();

        let results = db.search_articles("quantum qubit", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry_id, 10);

        let econ_results = db.search_articles("inflation markets", 10).await.unwrap();
        assert_eq!(econ_results.len(), 1);
        assert_eq!(econ_results[0].entry_id, 20);
    }

    #[tokio::test]
    async fn test_ai_db_time_window_recent_summaries() {
        let db = AiDb::new_mock();
        let start = Utc::now() - chrono::Duration::hours(1);

        db.save_summary_and_title(50, Some("Recent Article"), "Recent Summary", "model")
            .await
            .unwrap();

        let recent = db.get_recent_summaries(start, 5).await.unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].entry_id, 50);

        let future = Utc::now() + chrono::Duration::hours(1);
        let none = db.get_recent_summaries(future, 5).await.unwrap();
        assert!(none.is_empty());
    }

    #[tokio::test]
    async fn test_ai_db_page_briefs_and_pruning() {
        let db = AiDb::new_mock();

        db.save_page_brief_by_key("key-1", "Brief for key 1")
            .await
            .unwrap();
        db.save_page_brief("Latest Brief").await.unwrap();

        assert_eq!(
            db.get_page_brief_by_key("key-1").await,
            Some("Brief for key 1".to_string())
        );
        assert_eq!(
            db.get_page_brief_by_key("__latest__").await,
            Some("Latest Brief".to_string())
        );
        assert_eq!(
            db.get_latest_page_brief().await,
            Some("Latest Brief".to_string())
        );

        // Pruning with retention 1 day should keep items created now
        let pruned = db.prune_page_briefs(1).await.unwrap();
        assert_eq!(pruned, 0);

        // Pruning with negative retention should safely return 0 without deleting
        let negative_pruned = db.prune_page_briefs(-1).await.unwrap();
        assert_eq!(negative_pruned, 0);
    }

    #[tokio::test]
    async fn test_ai_db_edge_cases_empty_query_and_nonpositive_limits() {
        let db = AiDb::new_mock();

        db.save_summary_and_title(100, Some("Edge Article"), "Content here", "model")
            .await
            .unwrap();

        // Empty and whitespace queries return empty results without error
        let empty_res = db.search_articles("", 10).await.unwrap();
        assert!(empty_res.is_empty());
        let ws_res = db.search_articles("   ", 10).await.unwrap();
        assert!(ws_res.is_empty());

        // Zero and negative limits return empty results without panicking
        let zero_limit_res = db.search_articles("Edge", 0).await.unwrap();
        assert!(zero_limit_res.is_empty());
        let neg_limit_res = db.search_articles("Edge", -5).await.unwrap();
        assert!(neg_limit_res.is_empty());

        // get_recent_summaries with nonpositive limits
        let start = Utc::now() - chrono::Duration::hours(1);
        let zero_recent = db.get_recent_summaries(start, 0).await.unwrap();
        assert!(zero_recent.is_empty());
        let neg_recent = db.get_recent_summaries(start, -1).await.unwrap();
        assert!(neg_recent.is_empty());
    }
}
