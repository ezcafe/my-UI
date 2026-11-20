Result: clean

# DB Design Review

**Result:** clean

## Key Takeaways
- **Zero blocking issues**: All previous database review findings from [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03a-db-design-review.md) have been resolved in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03-design.md).
- **Safe connection pooling**: The updated design uses `r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>` (pool size 8) with queries executed via `tokio::task::spawn_blocking`, removing the async mutex anti-pattern.
- **Robust concurrency controls**: Every connection applies `PRAGMA journal_mode = WAL;`, `PRAGMA busy_timeout = 5000;`, and `PRAGMA synchronous = NORMAL;`, preventing `SQLITE_BUSY` errors when 15 HTMX card skeletons load at once.
- **Race-free inserts**: An explicit `ON CONFLICT(entry_id) DO UPDATE` upsert clause prevents unique constraint failures during concurrent requests for the same entry.
- **Migration & query optimization**: Schema creation uses `PRAGMA user_version = 1;` for versioned migrations and adds an index on `created_at` to support future TTL pruning.
- **Tenant isolation**: The canonical endpoint `GET /hx/entries/{id}/ai-summary` requires `AuthenticatedUser` and verifies Miniflux entry ownership before querying the local cache or executing LLM processes.

---

## Verification of Prior Findings

### 1. Connection Pooling & Async Threading
- **Prior Finding**: `tokio::sync::Mutex` inside `tokio::task::spawn_blocking` was an anti-pattern that serialized all cache operations and risked thread blocking.
- **Resolution in Design**: 
  - Adopted Option A using `r2d2` (`0.8`) and `r2d2_sqlite` (`0.25`) with `rusqlite` (`0.32`).
  - Configured a dedicated connection pool of 8 connections in `crates/server/src/ai/db.rs`.
  - Tasks check out pooled connections inside `tokio::task::spawn_blocking` independently, enabling up to 8 simultaneous cache reads.
- **Status**: **PASS** (Clean).

### 2. Concurrency Pragmas (WAL Mode & Busy Timeout)
- **Prior Finding**: Missing Write-Ahead Logging and busy timeouts caused readers and writers to lock each other, triggering `SQLITE_BUSY` errors.
- **Resolution in Design**:
  - Connection manager initializes every connection with:
    - `PRAGMA journal_mode = WAL;` (concurrent readers alongside a writer).
    - `PRAGMA busy_timeout = 5000;` (waits up to 5 seconds before failing on lock contention).
    - `PRAGMA synchronous = NORMAL;` (optimal performance and safety for WAL mode).
- **Reference**: [SQLite WAL Documentation](https://www.sqlite.org/wal.html), [SQLite PRAGMA busy_timeout](https://www.sqlite.org/pragma.html#pragma_busy_timeout).
- **Status**: **PASS** (Clean).

### 3. UPSERT / Duplicate Insert Handling
- **Prior Finding**: Plain `INSERT INTO article_ai_summaries` failed with `UNIQUE constraint failed: article_ai_summaries.entry_id` if duplicate requests fired concurrently.
- **Resolution in Design**:
  - Insert query now specifies:
    ```sql
    INSERT INTO article_ai_summaries (entry_id, summary, model, created_at)
    VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    ON CONFLICT(entry_id) DO UPDATE SET
      summary = excluded.summary,
      model = excluded.model,
      created_at = excluded.created_at;
    ```
- **Reference**: [SQLite UPSERT Documentation](https://www.sqlite.org/lang_UPSERT.html).
- **Status**: **PASS** (Clean).

### 4. Indexing for Query Performance & Retention
- **Prior Finding**: Missing index on `created_at` forced full table scans for future TTL expiration or cache pruning queries.
- **Resolution in Design**:
  - Schema initialization includes:
    ```sql
    CREATE INDEX IF NOT EXISTS idx_article_ai_summaries_created_at ON article_ai_summaries(created_at);
    ```
  - Primary key index on `entry_id` handles O(1) point lookups, while `idx_article_ai_summaries_created_at` optimizes time-based range queries.
- **Status**: **PASS** (Clean).

### 5. Migration Safety & Version Tracking
- **Prior Finding**: Relying solely on `CREATE TABLE IF NOT EXISTS` provided no mechanism to manage future column additions or schema migrations.
- **Resolution in Design**:
  - Initialization inspects `PRAGMA user_version;`.
  - When `version < 1`, it creates the table, creates the index, and sets `PRAGMA user_version = 1;`.
- **Reference**: [SQLite PRAGMA user_version](https://www.sqlite.org/pragma.html#pragma_user_version).
- **Status**: **PASS** (Clean).

### 6. Endpoint Security & Multi-User Cache Isolation
- **Prior Finding**: Cache queries by `entry_id` risked data leakage across users if entry ownership was not verified.
- **Resolution in Design**:
  - `GET /hx/entries/{id}/ai-summary` enforces `_user: AuthenticatedUser`.
  - Verifies entry `{id}` exists and belongs to the authenticated user via Miniflux API/GraphQL before reading from cache or running the LLM.
  - Returns a graceful replacement HTML card on unauthorized access or missing entry, preserving multi-tenant isolation.
- **Status**: **PASS** (Clean).

---

## Conclusion
The database schema, pooling configuration, concurrency pragmas, and access control model in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03-design.md) are sound, resilient, and ready for implementation.

**Final Assessment:** **Clean**. No further design updates required for the database layer.
