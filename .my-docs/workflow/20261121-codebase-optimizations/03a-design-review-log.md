Result: clean

# General Design Review: Codebase Performance, Concurrency, and Quality Optimizations

- **Stage:** `design-review` (Mode: full)
- **Status:** Complete (`clean`)
- **Isolated Reviews Paid:**
  - API Contract Review: [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/03a-api-contract-review.md) (`clean`)
  - DB Design Review: [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/03a-db-design-review.md) (`clean`)
- **Review Targets:**
  - [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/03-design.md)
  - [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/04-tasks.md)

---

## 1. Executive Summary

This general design review evaluates the architecture, cross-cutting concerns, system reliability, and overall feasibility of the proposed optimizations in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/03-design.md).

- **Zero Critical / Major Issues**: The architecture is sound, safe, and robust.
- **Resource Bounding & Memory Safety**: Memory limits are properly enforced in `MinifluxCache` (1,000 entry cap) and `RateLimiter` (stale bucket pruning for idle clients > 10 min), preventing unbounded heap growth.
- **Execution Efficiency**: Precompilation of regexes and CSS selectors via `std::sync::LazyLock` eliminates repeated compilation in hot request paths.
- **Connection Efficiency**: Moving `PRAGMA journal_mode = WAL;` to schema initialization stops wasteful repetitive execution on every pool connection checkout.
- **DOM Rendering Performance**: Scoping Lucide icon rendering to swapped targets on HTMX swap events avoids re-scanning the entire page.
- **Zero Breaking Changes**: All public crate interfaces, error contracts, and HTTP endpoints remain 100% backward-compatible.

---

## 2. Multi-Axis Design Evaluation

### 2.1 Concurrency & Contention Safety
- **Cache Eviction**: Inline eviction occurs under the existing write lock only when `store.len() >= MAX_CACHE_CAPACITY`. Read requests use `RwLock::read`, maintaining high-throughput concurrency.
- **Rate Limiter Map Pruning**: Pruning inactive buckets (> 10 minutes) occurs under the existing mutex only when `buckets.len() > 100`. The retain scan takes microseconds and avoids background task churn.
- **Database Thread Isolation**: SQLite operations remain safely wrapped in `tokio::task::spawn_blocking`, ensuring synchronous disk I/O does not block Tokio worker threads.

### 2.2 Security & Robustness
- **Input Validation**: `RateLimiter::new` validates input numbers, preventing division by zero or `NaN` token refill rates.
- **HTML Sanitization & Summary**: `strip_html_tags` gracefully handles empty, unclosed, or malformed tags without panics, and does not bypass or degrade the primary `ammonia` sanitizer in `crates/ui-components/src/sanitizer.rs`.
- **Secrets & Data Protection**: No credentials, tokens, or environment secrets are exposed, logged, or altered by these optimizations.

### 2.3 Simplicity & Maintainability
- **Standard Library Primitives**: Using `std::sync::LazyLock` and lightweight tag stripping avoids adding heavy new dependencies (e.g. `scraper` or `ammonia` in `miniflux-client`).
- **Clippy Cleanliness**: Addressing the semaphore acquisition lint and type complexity in `cache.rs` improves code cleanliness and eliminates developer distraction.
- **No Background Janitor Overhead**: Choosing synchronous, in-line pruning over background tasks avoids lifecycle teardown complexity, timer management, and test flakiness.

### 2.4 Testability & Verification Plan
- **Granular Coverage**: Tasks in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/04-tasks.md) include 8 dedicated unit test cases covering cache capacity bounding, rate limiter bucket eviction, invalid parameter defaults, lazy regex/selector execution, HTML tag stripping, and SQLite WAL configuration.
- **Automated Verification**: Build and test gates will run `cargo clippy --workspace --all-targets` and `cargo test --workspace` to ensure zero regressions across all workspace crates.

---

## 3. Findings & Recommendations

- **Critical Issues:** 0
- **Major Issues:** 0
- **Minor Observations / Suggestions:**
  - In `crates/miniflux/src/summary.rs`, ensure `strip_html_tags` replaces block-level tag closures with a space so adjacent text blocks do not concatenate words together (e.g. `<p>First</p><p>Second</p>` -> `First Second`). The design snippet in `03-design.md` already pushes a space on `>` which handles this cleanly.

---

## 4. Conclusion & Next Stage

The design meets all quality, performance, and stability criteria. Both isolated reviews (`api-contract-review` and `db-design-review`) have passed clean, and the general design review is clean.

The design phase is complete. The workflow is ready to proceed to parent Gate B derivation.

**Review Result:** `clean`
