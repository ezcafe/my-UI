# Technical Analysis: PostgreSQL Database Migration

- **Stage:** analyze
- **Status:** completed
- **Result:** `done`

---

## 1. Background and Workload Context

`my-ui` currently utilizes SQLite (`rusqlite` + `r2d2`) for persisting AI-generated article summaries and page executive briefs. Because SQLite is a file-based embedded engine with synchronous APIs, all operations in `crates/server/src/ai/db.rs` are executed using `tokio::task::spawn_blocking`.

The user has requested migrating to **PostgreSQL 19** (`postgres:latest` / `postgres:19`) with a clean slate (discarding old SQLite files) and specifically optimizing the database for news reader workloads:
1. **Full-text search (FTS)** across article summaries and titles.
2. **Time-window aggregations** (e.g. top daily news, summaries in the last 24h/7d).
3. **Strictly fast key-value lookups** by article ID (`entry_id`) and page key (`page_key`).

---

## 2. Technical Evaluation of News Reader Workloads

### A. Strictly Fast Key-Value Lookups (Primary Access Path)
- **Pattern:** When rendering article lists or viewing an entry, the server queries summaries by `entry_id` (single) or `entry_ids` (batch of up to 50–100 items).
- **Postgres Optimization:**
  - `entry_id BIGINT PRIMARY KEY`: B-Tree indexed clustered lookup gives $O(\log N)$ point reads (<1ms).
  - Batch queries: `SELECT ... FROM article_ai_summaries WHERE entry_id = ANY($1::bigint[])` instead of generating variable-length `IN (?, ?, ?)` clauses. This enables query plan caching and optimal parameter binding in PostgreSQL.

### B. Time-Window Aggregations & Feed Ordering
- **Pattern:** Querying summaries created within time ranges (e.g., today's digests, past 24 hours, prune older than 30 days).
- **Postgres Optimization:**
  - Use native `TIMESTAMPTZ` with `DEFAULT NOW()` rather than string formatting (`strftime`).
  - Indexing: `CREATE INDEX idx_article_summaries_created_at_desc ON article_ai_summaries (created_at DESC);`
  - Enables index-only scans and fast window filtering without sequential table scans.

### C. Full-Text Search (FTS) Across Summaries and Titles
- **Pattern:** Searching across generated headlines (`suggested_title`), summary paragraphs (`summary`), and full rewrites (`rewrite`).
- **Postgres Optimization Options:**
  1. **Option A: Native `tsvector` with GIN Index:**
     - Stored generated column: `search_vector tsvector GENERATED ALWAYS AS (to_tsvector('english', coalesce(suggested_title, '') || ' ' || coalesce(summary, '') || ' ' || coalesce(rewrite, ''))) STORED;`
     - Index: `CREATE INDEX idx_article_summaries_search_gin ON article_ai_summaries USING GIN(search_vector);`
     - Pros: Native language stemming (English dictionary), stop-word filtering, extremely fast ranked search (`ts_rank`).
     - Cons: Requires word boundary matching; doesn't match arbitrary partial substrings.
  2. **Option B: `pg_trgm` (Trigram) Extension with GIN Index:**
     - Extension: `CREATE EXTENSION IF NOT EXISTS pg_trgm;`
     - GIN trigram index on concatenated text.
     - Pros: Matches arbitrary substrings and fuzzy typos (`ILIKE %term%`).
     - Cons: Larger index footprint, slower writes than `tsvector`.
  3. **Option C: Hybrid (tsvector for stemmed relevance search + trigram readiness):**
     - Primary search powered by `tsvector` with ranking; extension enabled for future fuzzy matching.

---

## 3. Database Driver & Concurrency Architecture

- **Async Driver (`sqlx`):**
  - Native asynchronous PostgreSQL driver using Tokio.
  - Built-in connection pool (`sqlx::PgPool` / `sqlx::postgres::PgPoolOptions`).
  - Completely eliminates `tokio::task::spawn_blocking` and thread pool starvation under high concurrent requests.
- **Connection Pool Sizing:**
  - Standard containerized pool: `max_connections = 10` (configurable via `DATABASE_MAX_CONNECTIONS`).
  - `acquire_timeout = 3 seconds`, `idle_timeout = 10 minutes`.

---

## 4. Risks & Mitigations

1. **Risk:** Local development setup without Docker could fail if Postgres is required.
   - **Mitigation:** Provide clear `docker compose up -d db` instructions and sensible `DATABASE_URL` defaults (`postgres://postgres:postgres@localhost:5432/my_ui`) in `.env.example`.
2. **Risk:** Unit and integration tests requiring live PostgreSQL instance.
   - **Mitigation:** In integration tests, read `DATABASE_URL` from environment; if unavailable, skip or gracefully degrade, or run tests against an ephemeral local Postgres test database.
3. **Risk:** Container startup race condition between `my-ui` and PostgreSQL.
   - **Mitigation:** Configure a robust `healthcheck` in `docker-compose.yml` (`pg_isready -U postgres`) and set `depends_on: db: condition: service_healthy` on `my-ui`.

---

## 5. Next Stage

- Proceed to `grill` to structure decisions and verify frontier requirements.
