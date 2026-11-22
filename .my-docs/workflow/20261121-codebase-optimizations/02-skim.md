# Skim: Codebase Performance, Concurrency, and Quality Optimizations

Result: done

## Codebase Skim Summary

This skim surveys the 6 optimization areas across backend crates, database configuration, web scraping, and frontend templates.

---

### Area 1: In-Memory Cache & Rate Limiter Bounds (`crates/server/src/`)

- **Cache Implementation (`crates/server/src/cache.rs`)**:
  - `MinifluxCache` stores items in `Arc<RwLock<HashMap<String, (Instant, Duration, serde_json::Value)>>>`.
  - Keys are added without any upper bound. Expired items are only checked upon retrieval (`get`), meaning abandoned keys never get evicted.
  - Type complexity triggers Clippy warnings due to the nested 3-tuple.
- **Rate Limiter Implementation (`crates/server/src/rate_limit.rs`)**:
  - `RateLimiter` uses `Arc<Mutex<HashMap<String, ClientBucket>>>`.
  - Client buckets accumulate indefinitely without eviction of stale IP addresses.
  - Parameter initialization does not validate that `capacity` and `refill_per_sec` are positive numbers.

---

### Area 2: Precompiled Regex & Scraper Selectors (`LazyLock`)

- **HTML Sanitizer (`crates/ui-components/src/sanitizer.rs`)**:
  - Lines 36-37 instantiate `regex::Regex::new(r#"<a(\s+[^>]*)?>"#)` and `regex::Regex::new(r#"target="[^"]*""#)` on every execution of `sanitize_html()`.
  - Compiling regular expressions repeatedly introduces unnecessary CPU allocation in high-throughput rendering paths.
- **Article Extractor (`crates/server/src/ai/extractor.rs`)**:
  - Lines 24 and 32 parse CSS selectors (`Selector::parse("article")` and `Selector::parse("body")`) on each HTML document extraction.
  - Parsing selectors on every request adds CPU overhead during article summarization.

---

### Area 3: Clippy Warnings & Code Hygiene

- **LLM CLI Runner (`crates/server/src/ai/llm.rs`)**:
  - Line 41 uses manual `match self.semaphore.acquire().await { Ok(p) => Some(p), Err(_) => None }`.
  - Triggers `clippy::manual_ok_err`. Idiomatic Rust uses `.ok()`.
- **Cache Tuple (`crates/server/src/cache.rs`)**:
  - The type `(Instant, Duration, serde_json::Value)` triggers `clippy::type_complexity`.

---

### Area 4: SQLite PRAGMA Optimization in `AiDb` (`crates/server/src/ai/db.rs`)

- **Connection Pool Customizer**:
  - `SqliteCustomizer::on_acquire` runs `PRAGMA journal_mode = WAL;` on every single connection acquisition.
  - In SQLite, WAL mode is a persistent database setting. Executing it repeatedly on pool checkouts creates redundant lock transitions and query overhead.
  - Per-connection settings (`PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;`) are appropriate for `on_acquire`.

---

### Area 5: HTML Sentence Extraction Cleanup (`crates/miniflux/src/summary.rs`)

- **Summary Extractor**:
  - `extract_summary` runs unicode sentence segmentation directly on raw input strings.
  - If feeds supply HTML markup, raw tags (`<p>`, `<a>`, `<span>`) contaminate summary sentences.
  - Requires pre-stripping tags prior to running sentence boundary detection.

---

### Area 6: Frontend Template Lucide Icon Scoping (`crates/ui-components/templates`)

- **Icon Instantiation**:
  - `base.html` (lines 363, 368) and `pages/index.html` (lines 213, 459, 792, 871, 946) invoke `lucide.createIcons()` across the whole document.
  - HTMX swaps frequently update isolated panels (`#entries-list`, `#detail-panel`, `#toast-container`).
  - Scoping icon creation to the target container prevents full DOM recalculations and layout jitter.
