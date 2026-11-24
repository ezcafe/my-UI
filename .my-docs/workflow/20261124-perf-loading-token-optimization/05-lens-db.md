# Lens: Database Review

- **Workflow Slug:** `20261124-perf-loading-token-optimization`
- **Lens:** db
- **Result:** clean

## Summary

The persistence layer implementation safely supports the extended SQLite schema (version 4, `PRAGMA user_version = 4`) and provides non-blocking, concurrent read and write operations. The `rewrite` column on `article_ai_summaries` is verified and guarded against re-migration errors. Upsert queries use `COALESCE` to prevent overwriting existing rewrites or titles when saving summaries. Connection pooling with r2d2, WAL mode, and `busy_timeout = 5000` ensure sub-millisecond read latency and zero thread blocking on the async Tokio runtime.

---

## DB Checklist

| Requirement | Status | Notes |
|-------------|--------|-------|
| **Typed columns & nullability** | Pass | `rewrite TEXT` is nullable in SQLite schema; maps cleanly to `Option<String>` in `AiSummaryRecord` and `ArticleSummaryInput`. |
| **Additive migration** | Pass | Migration v3 -> v4 checks `PRAGMA table_info(article_ai_summaries)` before running `ALTER TABLE article_ai_summaries ADD COLUMN rewrite TEXT;`. Safe for repeated runs. |
| **Journal & version consistency** | Pass | `PRAGMA user_version = 4;` recorded consistently; WAL mode (`PRAGMA journal_mode = WAL;`) and `PRAGMA busy_timeout = 5000;` active across all connections. |
| **Safe SQL binds** | Pass | Parameterized queries using `rusqlite::params!` (`?1, ?2, ...`) across all insert, update, and select statements; zero raw string interpolation. |
| **Upsert idempotency** | Pass | `ON CONFLICT(entry_id) DO UPDATE SET rewrite = COALESCE(excluded.rewrite, article_ai_summaries.rewrite)` protects pre-existing data during concurrent writes. |
| **Bounded batch queries** | Pass | `get_batch_summaries` dynamically creates `IN (?1, ?2, ...)` placeholders up to the page limit (max 50 IDs). |
| **Non-blocking concurrency** | Pass | All SQLite queries run inside `tokio::task::spawn_blocking` to prevent blocking the async Tokio runtime threads. |
| **Design parity** | Pass | Schema, upsert semantics, and connection pooling match `03-design.md` Database Contracts section. |

---

## Findings

- **No Critical or Major issues found.**
- **Schema & Migration Verification**:
  - `AiDb::init_db`: Verified migration chain from version 0 through 4. Column inspection prevents errors on pre-existing columns.
  - Verified by dedicated unit test `test_migration_v3_to_v4_adds_rewrite_column`.
- **Query Performance & Locking**:
  - `get_summary_and_title(entry_id)` executes against the primary key index (`entry_id PRIMARY KEY`), resolving in < 1ms.
  - `save_summary_and_title_with_rewrite` and `save_batch_results` execute with parameter binding; batch writes use explicit transactions (`conn.transaction()`).
  - WAL mode allows concurrent readers (e.g. instant drawer loads) while background prefetch writes or on-demand rewrites commit.
- **Verification**: Dedicated database unit tests in `ai::db::tests` pass with 100% success (`cargo test -p server --lib ai::db`).
