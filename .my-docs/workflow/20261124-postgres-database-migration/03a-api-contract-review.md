Result: clean

# API Contract Review: PostgreSQL Migration

- **Stage:** api-contract-review
- **Reviewed Document:** `.my-docs/workflow/20261124-postgres-database-migration/03-design.md`
- **Result:** `clean`
- **Review Date:** 2026-11-24

---

## 1. Executive Summary

The updated `03-design.md` resolves all contract gaps previously identified. The Rust API surface and server initialization flow are verified to be fully backward-compatible with existing callers and cleanly integrated with the Tokio asynchronous runtime:

1. **`save_batch_results` parameter restored:** Section 4.2 retains `page_key: Option<(&str, &str)>`, ensuring atomic batch updates for `executive_brief_htmx` in `crates/server/src/routes/entries.rs`.
2. **Asynchronous initialization contract established:** Section 4.3 specifies `pub async fn new(config: &Config) -> Result<Self, AiDbError>` for `AiService` and `pub async fn build_app(...) -> Router` for application assembly, eliminating the fragile SQLite `/tmp` fallback in favor of fail-fast startup logging.
3. **Explicit `AiDbError` enum added:** Section 4.1 defines the error variants wrapping `sqlx::Error` and `std::io::Error`.
4. **Internal scope clarified:** FTS and time-window query methods (`search_articles`, `get_recent_summaries`) are designated as internal library methods on `AiDb`/`AiService` with no route additions required during this phase.

There are zero Critical or Major contract issues remaining.

---

## 2. Verification of Prior Findings

### Finding 1: `save_batch_results` signature
- **Status:** **Resolved**
- **Verification:** Section 4.2 lines 189–193 defines:
  ```rust
  pub async fn save_batch_results(&self, page_key: Option<(&str, &str)>, items: &[ArticleSummaryInput]) -> Result<(), AiDbError>;
  ```
  This preserves compatibility with `crates/server/src/routes/entries.rs:783` while retaining transactional consistency across page briefs and article summaries.

### Finding 2: `AiService` and `build_app` async initialization
- **Status:** **Resolved**
- **Verification:** Section 4.3 explicitly transitions both `AiService::new` and `build_app` to `async`, aligned with `crates/server/src/main.rs`. The SQLite `/tmp` fallback is completely removed in favor of explicit startup error handling.

### Finding 3: `AiDbError` enum specification
- **Status:** **Resolved**
- **Verification:** Section 4.1 defines `AiDbError` containing `Sqlx(#[from] sqlx::Error)` and `Io(#[from] std::io::Error)` with `thiserror::Error` derive.

### Finding 4: Route exposure of FTS and time-window queries
- **Status:** **Resolved**
- **Verification:** Section 4.2 clarifies that `search_articles` and `get_recent_summaries` are internal service methods without introducing public HTTP route alterations in this milestone.

---

## 3. Conclusion

The API contracts in `03-design.md` are complete, sound, and ready for implementation.
Result token: `clean`.
