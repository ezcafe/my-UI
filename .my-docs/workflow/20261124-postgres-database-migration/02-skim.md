# Codebase Skim: PostgreSQL Database Migration

- **Stage:** skim
- **Status:** completed
- **Result:** `done`

---

## 1. Key Codebase Findings

### Storage & Database Layer (`crates/server/src/ai/db.rs`)
- Currently uses `rusqlite = "0.32"` with `"bundled"` feature and `r2d2_sqlite = "0.25"`.
- Uses `r2d2::Pool<SqliteConnectionManager>` wrapped inside `AiDb`.
- All database queries run in `tokio::task::spawn_blocking` blocks because SQLite is synchronous.
- Schema consists of two primary tables:
  1. `article_ai_summaries(entry_id INTEGER PRIMARY KEY, summary TEXT NOT NULL, suggested_title TEXT, rewrite TEXT, model TEXT NOT NULL, created_at TEXT NOT NULL)` with an index on `created_at`.
  2. `page_executive_briefs_v2(id SERIAL/INTEGER PRIMARY KEY, page_key TEXT NOT NULL UNIQUE, brief TEXT NOT NULL, created_at TEXT NOT NULL)` with indexes on `created_at` and `page_key`.
- Key queries performed:
  - `get_summary_and_title(entry_id: i64)`: Point lookup by primary key.
  - `get_batch_summaries(entry_ids: &[i64])`: Batch lookup using `IN (...)`.
  - `save_summary_and_title_with_rewrite`: Upsert using `ON CONFLICT(entry_id) DO UPDATE`.
  - `get_page_brief_by_key(page_key: &str)`: Point lookup by `page_key`.
  - `save_batch_results`: Transaction writing multiple summaries and optional page brief.
  - `prune_page_briefs(retention_days: i64)`: Time-window deletion.

### Configuration & Environment (`crates/server/src/config.rs` & `.env.example`)
- Configuration currently has `ai_db_path: String` pointing to `data/summaries.db` by default.
- Needs to support `DATABASE_URL` (e.g. `postgres://my_ui_user:my_ui_password@localhost:5432/my_ui_db`) with fallback or explicit configuration.
- Needs `sqlx` (or `deadpool-postgres` / `tokio-postgres`) in `crates/server/Cargo.toml`.

### Deployment & Containers (`docker-compose.yml`, `docker-compose.local.yml`, `Dockerfile`)
- `docker-compose.yml` and `docker-compose.local.yml` only run `my-ui` right now.
- Need to add a `db` service running PostgreSQL 19 (`postgres:latest` or `postgres:19`), with named volume persistence, health checks (`pg_isready`), and environment variables (`POSTGRES_DB`, `POSTGRES_USER`, `POSTGRES_PASSWORD`).
- `my-ui` service should declare `depends_on: db: condition: service_healthy`.
- `Dockerfile` can remove build dependencies for bundled SQLite (`sqlite3`) and replace with libpq / postgres client libraries if dynamically linked, or pure Rust TLS (`rustls`).

---

## 2. Next Stage

- Proceed to `analyze`.
