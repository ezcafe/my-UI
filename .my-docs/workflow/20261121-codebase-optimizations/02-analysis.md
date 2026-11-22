# Analysis: Codebase Performance, Concurrency, and Quality Optimizations

Result: done

## Context & Objectives

The goal is to apply targeted performance, concurrency safety, and code quality optimizations across backend services, database configuration, web scraping, and frontend templates. The codebase is already functional and tested; these changes harden the system against memory exhaustion, eliminate redundant computation in hot paths, and clean up code hygiene issues identified by Clippy.

---

## Detailed Analysis by Area

### 1. In-Memory Bounds & Concurrency Safety (`crates/server/src/cache.rs` and `rate_limit.rs`)

- **Miniflux Cache (`crates/server/src/cache.rs`)**:
  - *Context*: Used to cache upstream Miniflux API responses (feeds, entries, counters) to minimize latency and avoid overwhelming upstream services.
  - *Current Problem*: The cache uses an unbounded `HashMap`. Entries are only checked for expiration when accessed via `get()`. Stale or abandoned keys remain in memory indefinitely.
  - *Solution*: 
    - Introduce a named `CacheEntry` struct:
      ```rust
      struct CacheEntry {
          inserted_at: Instant,
          ttl: Duration,
          val: serde_json::Value,
      }
      ```
    - Provide a helper `is_expired(&self) -> bool`.
    - Enforce a maximum capacity limit (e.g. 1,000 entries).
    - When `store.len() >= MAX_CAPACITY` during an insertion, first retain only non-expired entries (`store.retain(|_, entry| !entry.is_expired())`).
    - If the store is still at capacity, evict the oldest entry by `inserted_at`.
  - *Risks & Mitigations*: The pruning logic executes inside the write lock. Because `MAX_CAPACITY` is bounded at 1,000 entries, a single linear pass takes mere microseconds, introducing negligible contention.

- **Rate Limiter (`crates/server/src/rate_limit.rs`)**:
  - *Context*: Protects public endpoints from flooding using a token bucket algorithm per IP address.
  - *Current Problem*: `new(capacity, refill_per_sec)` does not guard against zero or negative values. Furthermore, `ClientBucket` instances are inserted into `HashMap<String, ClientBucket>` on every unique IP request and never removed.
  - *Solution*:
    - Validate parameters: ensure `capacity > 0.0` and `refill_per_sec > 0.0`, falling back to safe defaults (e.g., 60.0 and 5.0) or asserting validity.
    - Evict stale buckets: prune client buckets whose `last_refill.elapsed() > Duration::from_secs(600)` (10 minutes) during `check()`, or when bucket count exceeds a threshold (e.g., 5,000 entries).
  - *Risks & Mitigations*: Pruning while holding the mutex lock is fast because stale entries are checked with a simple elapsed time comparison.

---

### 2. Precompiled Regex & Scraper Selectors (`LazyLock`)

- **HTML Sanitizer (`crates/ui-components/src/sanitizer.rs`)**:
  - *Current Problem*: Every call to `sanitize_html()` compiles two regular expressions:
    - `Regex::new(r#"<a(\s+[^>]*)?>"#)`
    - `Regex::new(r#"target="[^"]*""#)`
  - *Solution*: Precompile both regular expressions using `std::sync::LazyLock`:
    ```rust
    static RE_ANCHOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<a(\s+[^>]*)?>"#).unwrap());
    static RE_TARGET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"target="[^"]*""#).unwrap());
    ```
  - *Benefits*: Zero regex parsing overhead during rendering; 100% thread-safe; built-in to Rust standard library without external dependencies.

- **Article Extractor (`crates/server/src/ai/extractor.rs`)**:
  - *Current Problem*: Calls `Selector::parse("article")` and `Selector::parse("body")` on every extraction request.
  - *Solution*: Precompile selectors using `std::sync::LazyLock`:
    ```rust
    static ARTICLE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("article").unwrap());
    static BODY_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("body").unwrap());
    ```
  - *Benefits*: Pre-parsed CSS selectors eliminate AST parsing on incoming HTML strings.

---

### 3. Clippy Warnings & Code Hygiene

- **`crates/server/src/ai/llm.rs`**:
  - `clippy::manual_ok_err`: Replace manual match with `self.semaphore.acquire().await.ok()`.
- **`crates/server/src/cache.rs`**:
  - `clippy::type_complexity`: Encapsulate `(Instant, Duration, serde_json::Value)` in `CacheEntry`.

---

### 4. SQLite Connection Pool PRAGMA Optimization (`crates/server/src/ai/db.rs`)

- *Current Problem*: `SqliteCustomizer::on_acquire` runs `PRAGMA journal_mode = WAL;` on every connection checkout from the `r2d2` pool.
- *Analysis*: In SQLite, `journal_mode = WAL` is a database file-level property written into the database header. It persists across connection openings and closes. Executing `PRAGMA journal_mode = WAL;` repeatedly causes needless SQLite internal lock checks on every checkout.
- *Solution*:
  - Execute `PRAGMA journal_mode = WAL;` once inside `init_schema(&self)`.
  - In `SqliteCustomizer::on_acquire`, retain per-connection pragmas:
    - `PRAGMA busy_timeout = 5000;`
    - `PRAGMA synchronous = NORMAL;`

---

### 5. HTML Sentence Extraction Cleanup (`crates/miniflux/src/summary.rs`)

- *Current Problem*: Feeds often contain inline HTML tags (`<p>`, `<em>`, `<a>`). Direct unicode sentence segmentation treats tags as literal text or produces awkward sentence boundaries.
- *Solution*:
  - Implement a fast tag-stripping function `strip_html_tags(content: &str) -> String`.
  - It removes `<...>` tags and decodes common HTML entities (e.g. `&nbsp;`, `&amp;`, `&quot;`, `&#39;`, `&lt;`, `&gt;`), replacing block-level breaks with spaces.
  - Whitespace is normalized before running `unicode_sentences()`.
  - Unit tests verify that HTML tags are cleanly removed and summaries are pure readable text.

---

### 6. Frontend Template Lucide Icon Scoping (`crates/ui-components/templates`)

- *Current Problem*: HTMX swaps call `lucide.createIcons()` without arguments, scanning the whole `document.body` and replacing all SVG icons again.
- *Solution*:
  - Pass the container element: `lucide.createIcons({ root: targetElement })` where target element is `evt.detail.target` in HTMX swap listeners, or specific elements like `document.getElementById('detail-panel')`.
  - This prevents DOM thrashing, avoids layout recalculations for unchanged page sections, and improves UI responsiveness.

---

## Risk Assessment & Invariants

1. **API & Contract Invariance**: All public function signatures (`MinifluxCache::get`, `MinifluxCache::insert`, `RateLimiter::check`, `sanitize_html`, `extract_summary`) remain identical or backward-compatible.
2. **Safety & Concurrency**: All shared state continues using standard Rust concurrency primitives (`tokio::sync::RwLock`, `tokio::sync::Mutex`, `std::sync::LazyLock`).
3. **Zero New External Dependencies**: Uses existing standard library features (`LazyLock`) and existing crate dependencies.
