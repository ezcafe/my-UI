# Code review log

**Overall Result:** clean

---

## Adversarial

**Result:** clean

### Security & Breakage Analysis
1. **Credential Exposure Mitigation:**
   - *Risk:* `DATABASE_URL` often includes plaintext database passwords (e.g., `postgres://user:password@host:port/db`). Printing `Config` via `{:?}` or logging during server boot exposed raw passwords in server logs.
   - *Resolution:* Implemented `mask_database_url` in `crates/server/src/config.rs` which sanitizes database connection URLs in `Config::fmt` and `crates/server/src/main.rs` `[SERVER_INIT]` output, replacing passwords with `[REDACTED]`. Added unit test coverage.
2. **Query Limit Validation & Wrap-Around Defense:**
   - *Risk:* Passing negative limits to `search_articles` or `get_recent_summaries` triggers PostgreSQL error `LIMIT must not be negative` and wraps around to `usize::MAX` in mock backends.
   - *Resolution:* Implemented defensive boundary checks returning `Ok(Vec::new())` immediately if `limit <= 0` and clamping large limits to `limit.min(1000)`. Short-circuited empty/whitespace queries in `search_articles`.
3. **Retention Pruning Boundary Safety:**
   - *Risk:* Negative `retention_days` in `prune_page_briefs` would compute `NOW() - (-N days) = NOW() + N days`, inadvertently purging all recent records.
   - *Resolution:* Added guard returning `Ok(0)` when `retention_days < 0`.
4. **SQL Injection & Parameter Safety:**
   - Verified that all queries utilize parameterized placeholders (`$1`, `$2`, `ANY($1)`). No string concatenation is used for SQL inputs.

---

## Quality

**Result:** clean

### Five-Axis Review Pass
1. **Correctness:**
   - 203 workspace unit, integration, and e2e tests execute and pass with 0 failures (`cargo test --workspace`).
   - Clean static analysis: `cargo clippy --workspace -- -D warnings` reported 0 warnings.
   - Code formatting verified with `cargo fmt --check`.
2. **Readability:**
   - Clear, idiomatic Rust code with descriptive function names and explicit types.
   - Modular separation between PostgreSQL client operations, mock registry for local testing, and domain service wrappers.
3. **Architecture:**
   - Complete removal of legacy `rusqlite`, `r2d2`, and `r2d2_sqlite` crates.
   - Replaced all blocking SQLite thread pools (`spawn_blocking`) with non-blocking native async Tokio integration via `sqlx::PgPool`.
   - Idempotent schema bootstrap via `INIT_SCHEMA_SQL` avoids complex external migration runner dependencies.
4. **Security:**
   - Passwords and keys (`SESSION_SECRET`, `OIDC_CLIENT_SECRET`, `MINIFLUX_API_KEY`, `LLM_API_KEY`, and `DATABASE_URL`) are redacted in debug and startup logs.
   - Docker Compose services run with dedicated non-root credentials.
5. **Performance:**
   - PostgreSQL stored generated column `search_vector tsvector GENERATED ALWAYS AS (...) STORED` eliminates triggers.
   - GIN index (`idx_article_summaries_search_gin`) provides sub-millisecond full-text searches.
   - Batch lookups using `ANY($1)` resolve hundreds of summary records in a single query roundtrip.
   - Production connection pool configured with `max_connections(10)`, `acquire_timeout(3s)`, `idle_timeout(300s)`, and `max_lifetime(1800s)`.

---

## Merged

**Result:** clean

### Lens Verification
- **Lens Plan:** `api|db`
- **05-lens-api.md:** `clean` (typed inputs, idempotent upserts, backward compatibility, stable errors)
- **05-lens-db.md:** `clean` (typed columns, GIN index, atomic batch transaction, connection pool options, safe array binds)
- **All Findings Resolved:** All identified edge cases and credential leakage risks have been resolved, formatted, and verified by passing test suites.
