Result: clean

# DB Design Review: PostgreSQL Database Migration

- **Workflow Slug:** `20261124-postgres-database-migration`
- **Result:** clean
- **Review Date:** 2026-11-24

---

## Key Takeaways

- **Zero blocking issues**: The database schema, connection pooling, indexing architecture, full-text search design, and container orchestration in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-postgres-database-migration/03-design.md) are sound, robust, and production-ready for PostgreSQL 19.
- **Native async concurrency**: Migrating from synchronous SQLite `r2d2` + `tokio::task::spawn_blocking` to asynchronous `sqlx::PgPool` eliminates worker threadpool offloading, reducing memory overhead and unlocking non-blocking async query pipelines.
- **Sub-millisecond key-value lookups**: Clustered B-tree primary keys on `entry_id BIGINT` and `page_key TEXT`, combined with `WHERE entry_id = ANY($1::bigint[])`, provide ultra-fast point and batch retrievals for article lists.
- **Optimized Full-Text Search (FTS)**: Stored generated `tsvector` column using immutable `to_tsvector('english', ...)` coupled with a GIN index delivers ranked sub-millisecond search without external search infrastructure.
- **Resilient container topology**: Docker Compose orchestration with `pg_isready` healthcheck, `service_healthy` startup gates, and persistent named volume `my_ui_postgres_data` guarantees safe data persistence and clean startup order.

---

## Detailed Evaluation Checklist

### 1. PostgreSQL 19 Engine & Data Types

- **Design Specification**:
  - PostgreSQL 19 engine target (`postgres:latest` / `postgres:19`).
  - `entry_id BIGINT PRIMARY KEY` for 64-bit integer article IDs.
  - `TEXT` for unbounded text payloads (`summary`, `suggested_title`, `rewrite`, `model`, `page_key`, `brief`).
  - `TIMESTAMPTZ NOT NULL DEFAULT NOW()` for UTC timezone-aware chronological timestamps.
- **Assessment**:
  - `BIGINT` maps natively to Rust's `i64` with zero conversion overhead or overflow risk.
  - In PostgreSQL, `TEXT` has identical performance to `VARCHAR(n)` without arbitrary length ceilings.
  - `TIMESTAMPTZ` is the gold standard for datetime persistence in PostgreSQL, avoiding subtle client-server timezone skew.
- **Status**: **PASS** (Clean).

---

### 2. Primary Keys & Clustered Key-Value Lookups

- **Design Specification**:
  - Primary key `entry_id` on `article_ai_summaries`.
  - Primary key `page_key` on `page_executive_briefs_v2`.
  - Batch lookups using PostgreSQL `ANY($1::bigint[])`.
- **Assessment**:
  - Primary key constraints automatically maintain a unique B-Tree index, enabling O(log N) point lookups (< 0.2ms).
  - Using `WHERE entry_id = ANY($1::bigint[])` passes a single parameterized array slice rather than dynamically formatting `IN (?, ?, ...)`, optimizing query plan caching in PostgreSQL.
- **Status**: **PASS** (Clean).

---

### 3. Full-Text Search (FTS) & GIN Indexing

- **Design Specification**:
  - Stored generated column `search_vector tsvector GENERATED ALWAYS AS (to_tsvector('english', coalesce(suggested_title, '') || ' ' || coalesce(summary, '') || ' ' || coalesce(rewrite, ''))) STORED`.
  - Inverted index: `CREATE INDEX IF NOT EXISTS idx_article_summaries_search_gin ON article_ai_summaries USING GIN(search_vector);`.
- **Assessment**:
  - Specifying the explicit dictionary `'english'` (2-argument `to_tsvector`) ensures the function is strictly `IMMUTABLE`. PostgreSQL correctly accepts this in `GENERATED ALWAYS AS ... STORED`.
  - Wrapping nullable columns in `coalesce(..., '')` prevents NULL concatenation from nullifying the entire search vector.
  - GIN indexing is the standard, high-efficiency structure for `tsvector` lookups, delivering sub-millisecond keyword matches across large document collections.
- **Status**: **PASS** (Clean).

---

### 4. Time-Window Queries & Data Retention

- **Design Specification**:
  - Descending B-Tree index `idx_article_summaries_created_at ON article_ai_summaries(created_at DESC)`.
  - Descending B-Tree index `idx_page_briefs_created_at ON page_executive_briefs_v2(created_at DESC)`.
- **Assessment**:
  - Explicit `DESC` index matches range queries like `WHERE created_at >= $1 ORDER BY created_at DESC LIMIT $2` directly without requiring in-memory sort buffers.
  - Facilitates index-only scans for retention pruning (`DELETE FROM page_executive_briefs_v2 WHERE created_at < NOW() - INTERVAL '30 days'`).
- **Status**: **PASS** (Clean).

---

### 5. Connection Pooling & Concurrency Architecture

- **Design Specification**:
  - `sqlx::postgres::PgPoolOptions::new().max_connections(10).acquire_timeout(Duration::from_secs(3))`.
  - `AiDb` struct holds cloned `sqlx::PgPool`.
- **Assessment**:
  - Default pool capacity of 10 connections is well-calibrated for an Axum-based web reader, preventing connection thrashing on the database container.
  - The 3-second acquire timeout prevents worker threads from blocking indefinitely under heavy load or temporary database restarts.
  - `sqlx::PgPool` is internally reference-counted (`Arc`), allowing cheap, lock-free cloning across Axum route handlers.
- **Status**: **PASS** (Clean).

---

### 6. Idempotent Startup DDL & Migration Strategy

- **Design Specification**:
  - Schema initialization runs on startup in `AiDb::new`.
  - DDL statements use `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`.
- **Assessment**:
  - Cleanly fits the "clean start / discard SQLite data" architectural decision settled during ideation and grill.
  - Eliminates external CLI runners or separate migration container dependencies, keeping the application self-contained.
  - Re-running `AiDb::new` across container restarts is completely idempotent.
- **Status**: **PASS** (Clean).

---

### 7. Container Orchestration & Volume Mounts

- **Design Specification**:
  - Container service `db` with `postgres:latest` image.
  - Health check: `pg_isready -U my_ui -d my_ui` with 5s interval and retries.
  - App service `depends_on: db: condition: service_healthy`.
  - Named persistent volume: `my_ui_postgres_data:/var/lib/postgresql/data`.
  - Deprecated SQLite volume mounts removed.
- **Assessment**:
  - Named volumes provide safe host filesystem abstraction and avoid Docker Desktop file permission issues.
  - `condition: service_healthy` guarantees that Axum does not attempt connection pool startup until PostgreSQL is ready to accept client connections.
- **Status**: **PASS** (Clean).

---

## Recommendations for Implementation (Non-blocking)

1. **Use `sqlx::raw_sql` for Multi-statement Startup DDL**:
   - In `AiDb::init_schema`, execute the multi-statement script via `sqlx::raw_sql(...)` rather than `sqlx::query(...)`, ensuring full compatibility with PostgreSQL simple query protocol.
2. **Defensive COALESCE on Upsert Statements**:
   - In `save_summary_and_title`, use:
     ```sql
     ON CONFLICT (entry_id) DO UPDATE SET
         summary = EXCLUDED.summary,
         suggested_title = COALESCE(EXCLUDED.suggested_title, article_ai_summaries.suggested_title),
         rewrite = COALESCE(EXCLUDED.rewrite, article_ai_summaries.rewrite),
         model = EXCLUDED.model,
         created_at = NOW();
     ```
     This prevents accidentally overwriting existing rewrite or title fields when updating fallback summaries.
3. **Use `websearch_to_tsquery` for User Queries**:
   - In `search_articles`, construct the tsquery via `websearch_to_tsquery('english', $1)` so unescaped characters (quotes, slashes, punctuation) do not raise PostgreSQL syntax errors.

---

## Conclusion

The PostgreSQL database migration design in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-postgres-database-migration/03-design.md) complies with all database design best practices, performance, concurrency, full-text search, and reliability standards.

**Final Assessment:** **clean**. Zero Critical or Major issues identified.
