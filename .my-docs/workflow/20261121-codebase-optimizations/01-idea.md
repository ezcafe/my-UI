# 01-idea: Codebase Performance, Concurrency, and Quality Optimizations

## Intent

Optimize the codebase across backend services, database connections, web scraping, and frontend templates to improve performance, concurrency safety, resource bounds, and code hygiene.

---

## 1. Summary of 6 Optimization Areas

### 1. In-Memory Cache & Rate Limiter Bounds & Safety
- **MinifluxCache (`crates/server/src/cache.rs`)**:
  - Bound storage to a maximum capacity (e.g. 1,000 entries) to prevent unbounded memory growth.
  - Proactively evict expired entries upon insertion or when capacity limit is reached.
  - Encapsulate the raw tuple into a named `CacheEntry` struct with helper methods (`is_expired()`).
- **RateLimiter (`crates/server/src/rate_limit.rs`)**:
  - Validate parameters on instantiation (`capacity > 0.0` and `refill_per_sec > 0.0`).
  - Periodically prune inactive/stale client buckets (idle for > 10 minutes) during rate checks to prevent memory leakage from ephemeral IPs.

### 2. Precompiled Regex & Scraper Selectors (`std::sync::LazyLock`)
- **Sanitizer (`crates/ui-components/src/sanitizer.rs`)**:
  - Precompile anchor tag regex (`<a(\s+[^>]*)?>`) and target regex (`target="[^"]*"`) using `std::sync::LazyLock` rather than compiling them on every `sanitize_html()` call.
- **Extractor (`crates/server/src/ai/extractor.rs`)**:
  - Precompile `Selector::parse("article")` and `Selector::parse("body")` statically with `std::sync::LazyLock` instead of parsing selectors on every HTML extraction call.

### 3. Clippy Warnings Clean Up
- **`crates/server/src/ai/llm.rs`**:
  - Replace manual match pattern `match self.semaphore.acquire().await { Ok(p) => Some(p), Err(_) => None }` with idiomatic `self.semaphore.acquire().await.ok()` to resolve `clippy::manual_ok_err`.
- **`crates/server/src/cache.rs`**:
  - Replace `Arc<RwLock<HashMap<String, (Instant, Duration, serde_json::Value)>>>` with a named struct `CacheEntry` to resolve `clippy::type_complexity`.

### 4. SQLite Connection PRAGMA Optimization in `AiDb`
- **AiDb Pool Customizer (`crates/server/src/ai/db.rs`)**:
  - `PRAGMA journal_mode = WAL;` is a persistent database-level configuration. Remove it from `on_acquire` (which executes on every single checkout from the pool) and run it once during database initialization (`init_schema`).
  - Retain per-connection performance pragmas (`PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;`) in `on_acquire`.

### 5. HTML Sentence Extraction Cleanup
- **Summary Extractor (`crates/miniflux/src/summary.rs`)**:
  - Strip HTML tags before running unicode sentence segmentation so summaries generated from raw feed HTML never include raw markup or broken tag fragments.

### 6. Frontend & Template Optimizations
- **Icon Rendering & DOM Efficiency (`crates/ui-components/templates`)**:
  - Scope `lucide.createIcons()` invocations to specific updated DOM containers (e.g., `#entries-list`, `#detail-panel`, `#toast-container`) instead of rescanning the entire `document.body` on every HTMX swap.
  - Verify clean layout and avoid unnecessary DOM repaints during filter and pagination updates.

---

## 2. Key Technical Decisions & Trade-Offs

### Decision 1: In-Memory Cache Eviction Strategy
- **Option A (Recommended)**: In-line eviction on insert/get with capacity bound (max 1,000 entries). When full or on insert, purge expired items; if still full, evict oldest entry.
  - **Explanation**: Simple, self-contained within `MinifluxCache`, requires no extra background tasks or timers.
  - **Concrete Example**:
    ```rust
    if store.len() >= MAX_CAPACITY {
        store.retain(|_, entry| !entry.is_expired());
    }
    ```
  - **Pros**: Zero overhead when idle; deterministic bounds.
  - **Cons**: Write path performs minor housekeeping when capacity is reached.
- **Option B**: Background Tokio interval task.
  - **Explanation**: Spawns a background loop that wakes every 60 seconds to prune expired entries.
  - **Pros**: Offloads pruning from request paths.
  - **Cons**: Adds background task management and shutdown lifecycle complexity.

### Decision 2: HTML Tag Stripping in `miniflux/src/summary.rs`
- **Option A (Recommended)**: Fast regex/state-machine text extractor in `summary.rs` to strip tags prior to unicode sentence segmentation.
  - **Explanation**: Keeps `miniflux-client` crate lightweight without pulling in heavyweight HTML parser dependencies.
  - **Concrete Example**:
    ```rust
    let clean_text = strip_html_tags(content);
    clean_text.unicode_sentences().take(sentences_count)...
    ```
  - **Pros**: Blazing fast, zero new external dependencies in `miniflux-client`.
  - **Cons**: Basic tag stripping handles standard RSS markup well, but not exotic nested malformed HTML.
- **Option B**: Add `scraper` or `ammonia` dependency to `miniflux-client`.
  - **Explanation**: Uses a full DOM parser for tag stripping.
  - **Pros**: Complete DOM fidelity.
  - **Cons**: Adds compile time and dependencies to the client library crate.

---

## 3. Scope & Invariants

- **Has UI (tentative)**: yes
- **Has API**: yes
- **Has DB**: yes
- **Has auth/PII/schema**: yes
- **Non-Goals**:
  - Introducing distributed caching (e.g. Redis/Memcached).
  - Modifying database schemas or external Miniflux API contracts.
- **Anti-Goals**:
  - No breaking changes to existing endpoints or template layouts.
  - No blocking operations in async request paths.

---

## 4. Open Question for Confirmation (HITL)

Would you like to proceed with the recommended options above (Option A for in-memory cache inline eviction and Option A for fast lightweight HTML tag stripping in summary extraction), or do you want to say "just start" to skip straight to execution?
