# Design: PostgreSQL Database Migration

- **Stage:** design
- **Status:** completed
- **Result:** `done`

---

## 1. Executive Summary

This design replaces the synchronous, embedded SQLite storage layer in `crates/server` with a production-grade, asynchronous PostgreSQL 19 backend using `sqlx`. The migration cleans out legacy SQLite files and tailors the schema and index structures specifically for news reader workloads:
1. **Instant Key-Value Lookups:** Clustered B-Tree primary keys for article IDs (`entry_id`) and page keys (`page_key`), with batch lookups using PostgreSQL `ANY($1::bigint[])`.
2. **Chronological Time Windows:** `created_at TIMESTAMPTZ` with descending B-Tree indexes for fast digest retrieval and data retention pruning.
3. **High-Performance Full-Text Search (FTS):** Stored generated `tsvector` columns with GIN indexing for English stemming and relevance ranking (`ts_rank`).
4. **Idempotent Lifecycle:** Automatic DDL migration during server initialization in `AiDb::new`.
5. **Container Orchestration:** Official PostgreSQL 19 container configured in Docker Compose with health checks and volume persistence.

---

## 2. Architecture & Approach Options

### Option A: Native Async `sqlx::PgPool` with Idempotent Startup DDL (Recommended)

- **Explanation:**
  Use `sqlx` with the Tokio runtime and Rustls TLS. Connections are managed via a connection pool (`sqlx::PgPool`). SQL queries are written directly using `sqlx::query` and `sqlx::query_as`, mapping records into domain structs. Database schema initialization runs on startup inside `AiDb::new` with idempotent `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` statements inside a transaction.
- **Concrete Example:**
  ```rust
  #[derive(Clone)]
  pub struct AiDb {
      pool: sqlx::PgPool,
  }

  impl AiDb {
      pub async fn new(database_url: &str) -> Result<Self, AiDbError> {
          let pool = sqlx::postgres::PgPoolOptions::new()
              .max_connections(10)
              .acquire_timeout(std::time::Duration::from_secs(3))
              .connect(database_url)
              .await?;
          let db = Self { pool };
          db.init_schema().await?;
          Ok(db)
      }

      async fn init_schema(&self) -> Result<(), AiDbError> {
          sqlx::query(
              r#"
              CREATE TABLE IF NOT EXISTS article_ai_summaries (
                  entry_id BIGINT PRIMARY KEY,
                  summary TEXT NOT NULL,
                  suggested_title TEXT,
                  rewrite TEXT,
                  model TEXT NOT NULL,
                  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                  search_vector tsvector GENERATED ALWAYS AS (
                      to_tsvector('english', coalesce(suggested_title, '') || ' ' || coalesce(summary, '') || ' ' || coalesce(rewrite, ''))
                  ) STORED
              );
              CREATE INDEX IF NOT EXISTS idx_article_summaries_created_at ON article_ai_summaries(created_at DESC);
              CREATE INDEX IF NOT EXISTS idx_article_summaries_search_gin ON article_ai_summaries USING GIN(search_vector);

              CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
                  page_key TEXT PRIMARY KEY,
                  brief TEXT NOT NULL,
                  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
              );
              CREATE INDEX IF NOT EXISTS idx_page_briefs_created_at ON page_executive_briefs_v2(created_at DESC);
              "#,
          )
          .execute(&self.pool)
          .await?;
          Ok(())
      }
  }
  ```
- **Pros:**
  - Native asynchronous integration with Tokio runtime without blocking worker threads.
  - Built-in connection pooling, statement caching, and automatic reconnection.
  - Zero external migration dependencies or packaging hurdles.
  - Native support for PostgreSQL types (`TIMESTAMPTZ`, `BIGINT`, arrays via `ANY`).
- **Cons:**
  - Startup DDL requires careful idempotent scripting without automatic down-migrations.

---

### Option B: `tokio-postgres` with `deadpool-postgres` and External Migration Runner

- **Explanation:**
  Use low-level `tokio-postgres` paired with `deadpool-postgres` for pool management, along with external migration files executed via `refinery` or a dedicated migration container.
- **Concrete Example:**
  ```rust
  let mut cfg = deadpool_postgres::Config::new();
  cfg.url = Some(database_url.to_string());
  let pool = cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls)?;
  ```
- **Pros:**
  - Minimal dependency tree without SQLx macros or query compilation baggage.
- **Cons:**
  - Verbose manual row parsing for complex query results.
  - Requires separate migration container or CLI runner to manage schema versions.
  - More complex error handling and array binding mechanics.

---

### Recommendation

**Option A (`sqlx::PgPool` with idempotent startup DDL)** is strongly recommended. It provides a cohesive, type-safe, non-blocking PostgreSQL client that cleanly matches the existing `my-ui` async architecture while eliminating `tokio::task::spawn_blocking` and keeping server deployment completely self-contained.

---

## 3. Detailed Data Models & Schema Design

### 3.1 `article_ai_summaries` Table

Stores AI-generated summaries, suggested headlines, rewrites, and full-text search vectors.

```sql
CREATE TABLE IF NOT EXISTS article_ai_summaries (
    entry_id BIGINT PRIMARY KEY,
    summary TEXT NOT NULL,
    suggested_title TEXT,
    rewrite TEXT,
    model TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    search_vector tsvector GENERATED ALWAYS AS (
        to_tsvector('english', coalesce(suggested_title, '') || ' ' || coalesce(summary, '') || ' ' || coalesce(rewrite, ''))
    ) STORED
);

-- Index for time-window queries and chronological digest ordering
CREATE INDEX IF NOT EXISTS idx_article_summaries_created_at 
    ON article_ai_summaries(created_at DESC);

-- GIN index for ranked full-text search
CREATE INDEX IF NOT EXISTS idx_article_summaries_search_gin 
    ON article_ai_summaries USING GIN(search_vector);
```

### 3.2 `page_executive_briefs_v2` Table

Stores pre-generated executive briefs for article feeds and collections.

```sql
CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
    page_key TEXT PRIMARY KEY,
    brief TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for pruning old briefs
CREATE INDEX IF NOT EXISTS idx_page_briefs_created_at 
    ON page_executive_briefs_v2(created_at DESC);
```

---

## 4. `AiDb` Rust API Specification

### 4.1 Error Handling (`AiDbError`)

With `sqlx` replacing `r2d2` and `rusqlite`, asynchronous database and IO errors map into a simplified error enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AiDbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 4.2 Query and Command Methods

All query methods are fully asynchronous and use `&self.pool`:

1. **Initialization:**
   ```rust
   pub async fn new(database_url: &str) -> Result<Self, AiDbError>;
   ```
2. **Article Summaries (Key-Value Lookups & Upserts):**
   ```rust
   pub async fn get_summary(&self, entry_id: i64) -> Option<String>;
   pub async fn get_summary_and_title(&self, entry_id: i64) -> Option<AiSummaryRecord>;
   pub async fn get_batch_summaries(&self, entry_ids: &[i64]) -> HashMap<i64, AiSummaryRecord>;
   pub async fn save_summary(&self, entry_id: i64, summary: &str, model: &str) -> Result<(), AiDbError>;
   pub async fn save_summary_and_title(&self, entry_id: i64, summary: &str, suggested_title: Option<&str>, model: &str) -> Result<(), AiDbError>;
   pub async fn save_summary_and_title_with_rewrite(&self, entry_id: i64, summary: &str, suggested_title: Option<&str>, rewrite: Option<&str>, model: &str) -> Result<(), AiDbError>;
   pub async fn save_batch_results(&self, page_key: Option<(&str, &str)>, items: &[ArticleSummaryInput]) -> Result<(), AiDbError>;
   pub async fn get_unprocessed_entry_ids(&self, entry_ids: &[i64]) -> Vec<i64>;
   ```
   *Note on `save_batch_results`:* Preserves `page_key: Option<(&str, &str)>` to support atomic batch saves matching callers in `entries.rs`. When `page_key` is provided (such as `Some((key, brief_text))`), both the executive brief and article summaries are committed inside a single database transaction.

3. **News Workload Extensions (FTS & Time Windows):**
   ```rust
   pub async fn search_articles(&self, query: &str, limit: i64) -> Result<Vec<AiSummaryRecord>, AiDbError>;
   pub async fn get_recent_summaries(&self, since: chrono::DateTime<chrono::Utc>, limit: i64) -> Result<Vec<AiSummaryRecord>, AiDbError>;
   ```
   *Internal Library Scope:* `search_articles` and `get_recent_summaries` are internal service methods on `AiDb` and `AiService` prepared for search handlers and background news aggregators. They do not introduce new public HTTP routes or alter existing Axum route tables in this migration phase.

4. **Page Briefs & Retention Pruning:**
   ```rust
   pub async fn get_latest_page_brief(&self) -> Option<String>;
   pub async fn get_page_brief_by_key(&self, page_key: &str) -> Option<String>;
   pub async fn save_page_brief(&self, brief: &str) -> Result<(), AiDbError>;
   pub async fn save_page_brief_by_key(&self, page_key: &str, brief: &str) -> Result<(), AiDbError>;
   pub async fn save_page_brief_with_key(&self, page_key: &str, brief: &str) -> Result<(), AiDbError>;
   pub async fn prune_page_briefs(&self, retention_days: i64) -> Result<usize, AiDbError>;
   ```

### 4.3 Async Initialization Contract for Service and Application

1. **`AiService::new`:**
   Made asynchronous to cleanly await connection pool creation and idempotent schema initialization:
   ```rust
   impl AiService {
       pub async fn new(config: &Config) -> Result<Self, AiDbError>;
   }
   ```
2. **`build_app`:**
   Made asynchronous and invoked as `build_app(config, schema).await` in `crates/server/src/main.rs`:
   ```rust
   pub async fn build_app(config: Arc<Config>, schema: AppSchema) -> Router;
   ```
3. **Removal of Legacy SQLite Fallback:**
   The legacy fallback that opened an SQLite database at `/tmp/ai_summaries_<pid>.db` is completely removed. If PostgreSQL is unreachable or fails to initialize during startup, the server logs an explicit error and fails fast rather than silently creating transient SQLite files on the host filesystem.

---

## 5. Configuration & Environment

- **Configuration Struct (`crates/server/src/config.rs`):**
  Replace `ai_db_path: String` with `database_url: String`.
  Default: `"postgres://my_ui:my_ui_password@localhost:5432/my_ui"`.
- **Environment Variable:**
  `DATABASE_URL` replaces `AI_DB_PATH`. Documented in `.env.example`.

---

## 6. Docker Compose Deployment Topology

Update `docker-compose.yml` and `docker-compose.local.yml`:

```yaml
services:
  db:
    image: postgres:latest
    container_name: my-ui-db
    restart: always
    environment:
      POSTGRES_USER: my_ui
      POSTGRES_PASSWORD: my_ui_password
      POSTGRES_DB: my_ui
    ports:
      - "5432:5432"
    volumes:
      - my_ui_postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U my_ui -d my_ui"]
      interval: 5s
      timeout: 5s
      retries: 5

  my-ui:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: my-ui
    restart: always
    depends_on:
      db:
        condition: service_healthy
    environment:
      DATABASE_URL: "postgres://my_ui:my_ui_password@db:5432/my_ui"
    ports:
      - "${PORT:-3000}:${PORT:-3000}"
    # Removed SQLite volume mounts

volumes:
  my_ui_postgres_data:
    driver: local
```

---

## 7. Proposed Has

- **Has UI:** `no`
- **Has API:** `yes`
- **Has DB:** `yes`
- **Has auth/PII/schema:** `yes`
