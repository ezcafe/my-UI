# Tasks (TDD-ready)

- **Stage:** design
- **Status:** completed
- **Result:** `done`

---

## 1. Planned Tests Table

| Case | Failing Test Name | Covers |
|:-----|:------------------|:-------|
| 1. Config Parsing | `test_config_database_url_from_env_or_default` | Verifies `Config` reads `DATABASE_URL` with fallback to `postgres://my_ui:my_ui_password@localhost:5432/my_ui`. |
| 2. Connection & Schema Init | `test_ai_db_connection_and_schema_init` | Verifies `AiDb::new` establishes connection and idempotently creates tables and indexes. |
| 3. Single Record CRUD | `test_ai_db_summary_save_and_retrieve` | Verifies `save_summary_and_title_with_rewrite` and `get_summary_and_title`, including upsert on conflict. |
| 4. Batch Query & Deduplication | `test_ai_db_batch_summaries_and_unprocessed` | Verifies `save_batch_results` (including optional `page_key` brief atomic transaction), `get_batch_summaries` using PostgreSQL `ANY`, and `get_unprocessed_entry_ids`. |
| 5. Full-Text Search (FTS) | `test_ai_db_fts_tsvector_search` | Verifies `search_articles` uses `to_tsvector` and `plainto_tsquery` with English stemming and relevance ranking. |
| 6. Time-Window Queries | `test_ai_db_time_window_recent_summaries` | Verifies `get_recent_summaries` filters records by `created_at` timestamp. |
| 7. Page Briefs & Retention | `test_ai_db_page_briefs_and_pruning` | Verifies saving, retrieving by page key, and pruning page briefs older than retention days. |

---

## 2. Implementation Tasks

### Task 1: Update Server Cargo Dependencies and Configuration

- **Failing test name:** `test_config_database_url_from_env_or_default`
- **Behavior:**
  - Remove `rusqlite`, `r2d2`, and `r2d2_sqlite` from `crates/server/Cargo.toml`.
  - Add `sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio", "tls-rustls", "postgres", "chrono", "json"] }`.
  - Replace `ai_db_path` in `crates/server/src/config.rs` with `database_url: String`, reading `DATABASE_URL` or falling back to default.
  - Update `.env.example` with `DATABASE_URL`.
- **Files allowed:**
  - `crates/server/Cargo.toml`
  - `crates/server/src/config.rs`
  - `.env.example`
- **Done evidence:**
  - `cargo check -p server` validates dependencies and config test passes.

---

### Task 2: Implement Core PostgreSQL `AiDb` Layer with Idempotent DDL

- **Failing test name:** `test_ai_db_connection_and_schema_init`
- **Behavior:**
  - Rewrite `crates/server/src/ai/db.rs` to use `sqlx::PgPool`.
  - Define explicit `AiDbError` enum (`Sqlx(#[from] sqlx::Error)`, `Io(#[from] std::io::Error)`).
  - Implement `pub async fn new(database_url: &str) -> Result<Self, AiDbError>`.
  - Implement `init_schema` executing idempotent DDL for `article_ai_summaries` (with `search_vector tsvector` generated column, B-Tree and GIN indexes) and `page_executive_briefs_v2`.
- **Files allowed:**
  - `crates/server/src/ai/db.rs`
- **Done evidence:**
  - `test_ai_db_connection_and_schema_init` runs and asserts tables exist in PostgreSQL.

---

### Task 3: Implement Point & Batch Summary Operations

- **Failing test name:** `test_ai_db_summary_save_and_retrieve`, `test_ai_db_batch_summaries_and_unprocessed`
- **Behavior:**
  - Implement `save_summary`, `save_summary_and_title`, `save_summary_and_title_with_rewrite` with `ON CONFLICT (entry_id) DO UPDATE`.
  - Implement `get_summary`, `get_summary_and_title`.
  - Implement `get_batch_summaries` using `WHERE entry_id = ANY($1::bigint[])`.
  - Implement `pub async fn save_batch_results(&self, page_key: Option<(&str, &str)>, items: &[ArticleSummaryInput]) -> Result<(), AiDbError>` supporting atomic brief and summary persistence in a single transaction.
  - Implement `get_unprocessed_entry_ids`.
- **Files allowed:**
  - `crates/server/src/ai/db.rs`
- **Done evidence:**
  - CRUD and batch tests pass against PostgreSQL.

---

### Task 4: Implement News FTS & Time-Window Aggregations

- **Failing test name:** `test_ai_db_fts_tsvector_search`, `test_ai_db_time_window_recent_summaries`
- **Behavior:**
  - Implement `search_articles(&self, query: &str, limit: i64) -> Result<Vec<AiSummaryRecord>, AiDbError>` using `search_vector @@ plainto_tsquery('english', $1)` ordered by `ts_rank(search_vector, plainto_tsquery('english', $1)) DESC`.
  - Implement `get_recent_summaries(&self, since: DateTime<Utc>, limit: i64) -> Result<Vec<AiSummaryRecord>, AiDbError>` ordered by `created_at DESC`.
  - Note: These are internal service methods with no new public HTTP route changes in this phase.
- **Files allowed:**
  - `crates/server/src/ai/db.rs`
- **Done evidence:**
  - FTS and time-window tests pass with verified ranking and timestamp filtering.

---

### Task 5: Implement Page Briefs & Retention Pruning

- **Failing test name:** `test_ai_db_page_briefs_and_pruning`
- **Behavior:**
  - Implement `save_page_brief`, `save_page_brief_by_key`, `get_page_brief_by_key`, `get_latest_page_brief`.
  - Implement `prune_page_briefs` deleting records older than `NOW() - INTERVAL '... days'`.
- **Files allowed:**
  - `crates/server/src/ai/db.rs`
- **Done evidence:**
  - Page brief storage and retention pruning tests pass.

---

### Task 6: Wire `AiDb` into App State and Call Sites

- **Failing test name:** `test_app_state_ai_db_wiring`
- **Behavior:**
  - Make `AiService::new(config: &Config).await` asynchronous, awaiting `AiDb::new`.
  - Make `pub async fn build_app(config: Arc<Config>, schema: AppSchema) -> Router` asynchronous and await it in `crates/server/src/main.rs`.
  - Completely remove the legacy SQLite `/tmp/ai_summaries_<pid>.db` fallback mechanism in `crates/server/src/app.rs`.
  - Update `crates/server/src/ai/mod.rs`, `crates/server/src/app.rs`, `crates/server/src/routes/entries.rs`, and test helpers to initialize `AiDb` with `database_url`.
  - Remove all `tokio::task::spawn_blocking` calls wrapping `AiDb` calls.
- **Files allowed:**
  - `crates/server/src/main.rs`
  - `crates/server/src/ai/mod.rs`
  - `crates/server/src/app.rs`
  - `crates/server/src/routes/entries.rs`
  - `crates/server/src/routes/mod.rs`
- **Done evidence:**
  - Workspace compiles with zero blocking database wrappers, server startup cleanly awaits PostgreSQL initialization without SQLite temp fallbacks, and tests pass.

---

### Task 7: Update Docker Compose and Remove SQLite Storage

- **Failing test name:** N/A (Integration / Configuration verification)
- **Behavior:**
  - Update `docker-compose.yml` and `docker-compose.local.yml` with official `postgres:latest` service `db`, volume `my_ui_postgres_data`, and healthcheck.
  - Set `DATABASE_URL` in `my-ui` environment.
  - Remove obsolete `data/summaries.db` references and volume mappings.
- **Files allowed:**
  - `docker-compose.yml`
  - `docker-compose.local.yml`
- **Done evidence:**
  - `docker compose config` validates without errors.

---

### Task 8: Update Documentation (README & Backup Guide)

- **Failing test name:** N/A (Documentation verification)
- **Behavior:**
  - Update `README.md` to document PostgreSQL 19 architecture, environment variables (`DATABASE_URL`), AI summary and executive brief caching, and provide a clear step-by-step PostgreSQL backup and restore guide (`pg_dump` and `psql` commands via Docker Compose).
- **Files allowed:**
  - `README.md`
- **Done evidence:**
  - `README.md` accurately describes new database service, AI features, and backup procedures.
