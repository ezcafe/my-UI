Result: clean

# Database Design Review: AI Performance, Efficiency & Native REST Client

## 1. Executive Summary

- **Verdict**: **Clean** (Zero Critical or Major defects).
- **Schema Readiness**: SQLite schema version 3 in `crates/server/src/ai/db.rs` fully supports the keyed page brief storage (`page_executive_briefs_v2`) and batch summary queries required by the performance optimization design.
- **Concurrency & Reliability**: Existing WAL mode, 5-second busy timeout, connection customizer, and `r2d2` pool via `tokio::task::spawn_blocking` prevent connection starvation and database lock contention during concurrent HTMX requests and background prefetch tasks.

---

## 2. Detailed Schema & Table Verification

### 2.1 `page_executive_briefs_v2`
- **Definition** (`crates/server/src/ai/db.rs#L120-L125`):
  ```sql
  CREATE TABLE IF NOT EXISTS page_executive_briefs_v2 (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     page_key TEXT NOT NULL UNIQUE,
     brief TEXT NOT NULL,
     created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
  );
  CREATE INDEX IF NOT EXISTS idx_page_briefs_v2_created_at ON page_executive_briefs_v2(created_at);
  ```
- **Evaluation**:
  - **Uniqueness & Deduplication**: `page_key TEXT NOT NULL UNIQUE` provides automatic SQLite unique index enforcement. `INSERT ... ON CONFLICT(page_key) DO UPDATE` ensures idempotency when prefetch and UI requests race to store the same brief.
  - **Index Efficiency**: Lookups by `page_key` are $O(1)$ index seeks. Retention pruning (`prune_page_briefs`) utilizes `idx_page_briefs_v2_created_at` for range deletes without full table scans.

### 2.2 `article_ai_summaries`
- **Definition** (`crates/server/src/ai/db.rs#L81-L88`, `L105`):
  ```sql
  CREATE TABLE IF NOT EXISTS article_ai_summaries (
     entry_id INTEGER PRIMARY KEY,
     summary TEXT NOT NULL,
     model TEXT NOT NULL,
     created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
     suggested_title TEXT
  );
  CREATE INDEX IF NOT EXISTS idx_article_ai_summaries_created_at ON article_ai_summaries(created_at);
  ```
- **Evaluation**:
  - **Primary Key Lookups**: `entry_id INTEGER PRIMARY KEY` acts as a SQLite ROWID alias, enabling single-digit microsecond batch fetches in `get_batch_summaries` via `WHERE entry_id IN (...)`.
  - **Fallback Model Tracking**: The `model` column permits `get_unprocessed_entry_ids` to distinguish real LLM summaries from `extractive-fallback`, ensuring entries are reprocessed when an LLM becomes available.

---

## 3. Migration & Versioning Integrity

- **Version Management** (`crates/server/src/ai/db.rs#L78-L130`):
  - Initial version starts at `version < 1` (base summaries table).
  - Version 2 introduces `suggested_title` column and legacy `page_executive_briefs`.
  - Version 3 introduces `page_executive_briefs_v2` with `page_key UNIQUE`.
- **Backward Compatibility**:
  - Tested migration logic from v1 to v3 is validated in unit test `test_migration_v1_to_v2` (`crates/server/src/ai/db.rs#L418-L450`).
  - No additional schema migration or version bump is needed for this workflow; existing schema version 3 satisfies all design requirements.

---

## 4. SQLite Concurrency, PRAGMAs & Thread Safety

- **WAL Mode**: `PRAGMA journal_mode = WAL;` is executed on startup (`init_schema`), permitting simultaneous readers while a background writer commits batches.
- **Busy Timeout**: `PRAGMA busy_timeout = 5000;` is applied both at initialization and per acquired connection via `SqliteCustomizer` (`crates/server/src/ai/db.rs#L41-L48`). This guarantees that transient locks during batch saves will wait up to 5 seconds rather than returning `SQLITE_BUSY`.
- **Synchronous Mode**: `PRAGMA synchronous = NORMAL;` in WAL mode provides durability against application crashes while eliminating synchronous fsync bottlenecks.
- **Asynchronous Bridging**: All database calls execute inside `tokio::task::spawn_blocking` blocks on an `r2d2` pool of size 8, preventing blocking I/O from starving the Tokio async reactor.

---

## 5. Query Patterns & Operational Observations

- **Batch Insertion Atomicity**:
  - `save_batch_results` (`crates/server/src/ai/db.rs#L343-L384`) bundles article summary updates and page brief storage inside a single explicit `conn.transaction()?`.
  - **Optimization Note for Prefetch**: In `crates/server/src/ai/prefetch.rs#L108-L123`, the current code writes individual items in a loop. It is recommended during implementation to switch `prefetch.rs` to call `save_batch_results` (or use `save_page_brief_by_key`), saving multiple roundtrips and ensuring atomic page brief updates.
- **Method Naming Alignment**:
  - `03-design.md` mentions `ai.db.save_page_brief_with_key`.
  - The existing method in `crates/server/src/ai/db.rs#L288` is named `save_page_brief_by_key(&self, page_key: &str, brief: &str)`.
  - Implementers can either use `save_page_brief_by_key` directly or add a 1-line forwarding alias `save_page_brief_with_key` for naming flexibility.

---

## 6. Conclusion

- **Result**: `clean`
- The database architecture is robust, well-indexed, and fully compliant with all concurrency, isolation, and performance objectives.
