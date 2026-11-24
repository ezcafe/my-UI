# Grill: PostgreSQL Database Migration

- **Stage:** grill
- **Status:** completed
- **Result:** `frontier-empty`

---

## 1. Design Tree

```
PostgreSQL Database Migration
├── Engine & Data Management
│   ├── Target Engine: PostgreSQL 19 (`postgres:latest` / `postgres:19`) [Settled by user]
│   └── Data Strategy: Discard SQLite data, clean start [Settled by user]
├── Storage Architecture & Indexing
│   ├── Primary Access: B-Tree primary key on entry_id & page_key [Settled by user]
│   ├── Time-Window Queries: B-Tree on created_at DESC [Settled by user]
│   └── Full-Text Search (FTS) Strategy: Native English tsvector + GIN index [Settled: Option A]
├── Schema Initialization & Evolution
│   └── Startup DDL vs sqlx::migrate: Idempotent startup DDL in AiDb::new [Settled: Option A]
└── Container & Deployment Topology
    └── Local Docker Compose: Bundled db service with pg_isready healthcheck [Settled: Option A]
```

---

## 2. Frontier Questions & User Decisions (Round 1)

### Q1 - Full-Text Search (FTS) Strategy
- **Decision:** **Option A (Native English `tsvector` generated column with GIN index)**.
- **Settled Details:**
  PostgreSQL automatically maintains stemmed lexemes for `suggested_title`, `summary`, and `rewrite` in a stored generated column. A GIN index enables sub-millisecond full-text queries and ranking via `ts_rank`.
  ```sql
  ALTER TABLE article_ai_summaries ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (
      to_tsvector('english', coalesce(suggested_title, '') || ' ' || coalesce(summary, '') || ' ' || coalesce(rewrite, ''))
    ) STORED;
  CREATE INDEX idx_article_summaries_search_gin ON article_ai_summaries USING GIN(search_vector);
  ```

---

### Q2 - Schema Initialization & DDL Execution Strategy
- **Decision:** **Option A (Idempotent startup DDL executed in `AiDb::new`)**.
- **Settled Details:**
  On server boot, `AiDb::new` connects to PostgreSQL and runs idempotent DDL (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`). This avoids external migration runners or packaging separate migration files, matching the clean-start architecture.
  ```rust
  // Executed on startup in AiDb::new
  sqlx::query("CREATE TABLE IF NOT EXISTS article_ai_summaries (...);").execute(&pool).await?;
  ```

---

### Q3 - Container & Local Development Topology
- **Decision:** **Option A (Bundled `db` service with healthcheck in Docker Compose)**.
- **Settled Details:**
  Add official PostgreSQL 19 service (`postgres:latest` / `postgres:19`) named `db` to `docker-compose.yml` and `docker-compose.local.yml`. `my-ui` depends on `db` being healthy (`condition: service_healthy` using `pg_isready -U postgres`). A named volume `my_ui_postgres_data` persists data across container restarts.

---

## 3. Frontier Status

- **Unsettled questions:** None.
- **Result:** `frontier-empty` (Grill closed).
- **Next stage:** Proceed to `design`.
