# Design: Codebase Performance, Concurrency, and Quality Optimizations

Result: done

## Executive Summary

This design specifies performance, concurrency safety, resource bounding, and code hygiene improvements across 6 key components of the codebase:
1. **Bounded Cache & Eviction**: Introduce `CacheEntry` struct and inline eviction (capacity cap of 1,000 entries) in `MinifluxCache` to prevent unbounded memory growth and eliminate Clippy type complexity warnings.
2. **Rate Limiter Hardening**: Add input parameter validation and automatic pruning of stale client buckets (idle > 10 minutes) in `RateLimiter`.
3. **Precompiled Regex & Selectors**: Use `std::sync::LazyLock` to statically precompile regexes in `sanitizer.rs` and CSS selectors in `extractor.rs`.
4. **Clippy Cleanup**: Replace manual `match` with `.ok()` on semaphore permit acquisition in `llm.rs`.
5. **SQLite WAL Optimization**: Move `PRAGMA journal_mode = WAL;` to `init_schema()` so it runs once during database setup, while keeping per-connection pragmas in `on_acquire`.
6. **HTML Sentence Extraction**: Strip HTML tags and decode common entities before unicode sentence segmentation in `summary.rs`.
7. **Frontend DOM Efficiency**: Scope `lucide.createIcons({ root: container })` to updated DOM nodes on HTMX swaps.

---

## Architectural Options

### Option A: In-Line Bounded Eviction & LazyLock Static Initialization (Recommended)

- **Explanation**:
  - Keep cache and rate-limiter pruning synchronous and in-line during mutations or lookups. No background tasks, interval timers, or shutdown lifecycle coordination required.
  - Use `std::sync::LazyLock` from the Rust standard library for static precompilation of regular expressions and CSS selectors.
  - Implement a fast tag-stripping routine inside `miniflux-client` without introducing new external parsing dependencies.
  - Adjust SQLite connection pragmas to run persistent settings once at database initialization and connection-scoped settings on checkout.

- **Concrete Examples**:

  ```rust
  // 1. CacheEntry and bounded eviction in crates/server/src/cache.rs
  pub const MAX_CACHE_CAPACITY: usize = 1000;

  #[derive(Clone)]
  struct CacheEntry {
      inserted_at: Instant,
      ttl: Duration,
      val: serde_json::Value,
  }

  impl CacheEntry {
      fn is_expired(&self) -> bool {
          self.inserted_at.elapsed() >= self.ttl
      }
  }

  pub async fn insert_with_ttl(&self, key: String, val: serde_json::Value, ttl: Duration) {
      let mut store = self.store.write().await;
      if store.len() >= MAX_CACHE_CAPACITY {
          store.retain(|_, entry| !entry.is_expired());
          if store.len() >= MAX_CACHE_CAPACITY {
              if let Some(oldest_key) = store
                  .iter()
                  .min_by_key(|(_, entry)| entry.inserted_at)
                  .map(|(k, _)| k.clone())
              {
                  store.remove(&oldest_key);
              }
          }
      }
      store.insert(key, CacheEntry { inserted_at: Instant::now(), ttl, val });
  }
  ```

  ```rust
  // 2. LazyLock static precompilation in crates/ui-components/src/sanitizer.rs
  use std::sync::LazyLock;
  use regex::Regex;

  static RE_ANCHOR: LazyLock<Regex> = LazyLock::new(|| {
      Regex::new(r#"<a(\s+[^>]*)?>"#).expect("Valid anchor regex")
  });
  static RE_TARGET: LazyLock<Regex> = LazyLock::new(|| {
      Regex::new(r#"target="[^"]*""#).expect("Valid target regex")
  });
  ```

  ```rust
  // 3. SQLite PRAGMA separation in crates/server/src/ai/db.rs
  // In init_schema():
  conn.execute_batch("PRAGMA journal_mode = WAL;")?;

  // In SqliteCustomizer::on_acquire():
  conn.execute_batch(
      "PRAGMA busy_timeout = 5000;
       PRAGMA synchronous = NORMAL;",
  )?;
  ```

  ```rust
  // 4. Fast tag-stripping in crates/miniflux/src/summary.rs
  pub fn strip_html_tags(content: &str) -> String {
      let mut in_tag = false;
      let mut output = String::with_capacity(content.len());
      for c in content.chars() {
          match c {
              '<' => in_tag = true,
              '>' => {
                  in_tag = false;
                  output.push(' ');
              }
              _ if !in_tag => output.push(c),
              _ => {}
          }
      }
      output
  }
  ```

- **Pros**:
  - Deterministic memory bounds with zero background task overhead.
  - Significant performance boost on hot rendering and extraction paths.
  - Zero new external dependencies.
  - Fully backward-compatible public APIs.

- **Cons**:
  - Insert operations on full cache perform minor cleanup before insertion.

---

### Option B: Background Janitor Tasks & Full DOM Parser Dependencies

- **Explanation**:
  - Spawn background `tokio::spawn` loops that run periodic timers (e.g. every 60 seconds) to sweep the cache and rate-limiter maps.
  - Add `scraper` or `ammonia` dependency to `crates/miniflux` to parse HTML into a full DOM tree for tag removal.

- **Concrete Examples**:
  ```rust
  // Background interval in cache.rs
  pub fn start_janitor(store: Arc<RwLock<HashMap<String, CacheEntry>>>) {
      tokio::spawn(async move {
          let mut interval = tokio::time::interval(Duration::from_secs(60));
          loop {
              interval.tick().await;
              let mut lock = store.write().await;
              lock.retain(|_, entry| !entry.is_expired());
          }
      });
  }
  ```

- **Pros**:
  - Write operations do not perform any eviction work.
  - Complete DOM entity parsing for complex HTML documents.

- **Cons**:
  - Increases complexity with background task lifecycles, cancellation tokens, and testing teardowns.
  - Bloats `miniflux-client` crate with large external DOM parser dependencies.
  - Wastes CPU waking up background tasks even when the application is idle.

---

## Detailed Specifications by Component

### 1. `crates/server/src/cache.rs`
- Define `const MAX_CACHE_CAPACITY: usize = 1000`.
- Replace the raw tuple `(Instant, Duration, serde_json::Value)` with:
  ```rust
  #[derive(Clone)]
  pub struct CacheEntry {
      pub inserted_at: Instant,
      pub ttl: Duration,
      pub val: serde_json::Value,
  }
  ```
- In `get(&self, key: &str) -> Option<serde_json::Value>`:
  - If entry exists and `!entry.is_expired()`, return `Some(entry.val.clone())`.
- In `insert_with_ttl(&self, key: String, val: serde_json::Value, ttl: Duration)`:
  - Check capacity: if `store.len() >= MAX_CACHE_CAPACITY`, run `store.retain(|_, entry| !entry.is_expired())`.
  - If still at or over capacity, remove the oldest entry.
  - Insert the new `CacheEntry`.

### 2. `crates/server/src/rate_limit.rs`
- In `RateLimiter::new(capacity: f64, refill_per_sec: f64) -> Self`:
  - Validate parameters: if `capacity <= 0.0` or `refill_per_sec <= 0.0`, fallback to `(60.0, 5.0)`.
- In `RateLimiter::check(&self, ip: &str)`:
  - If `buckets.len() > 100`, prune inactive buckets:
    ```rust
    let prune_threshold = Duration::from_secs(600); // 10 minutes
    buckets.retain(|_, b| now.duration_since(b.last_refill) < prune_threshold);
    ```

### 3. `crates/ui-components/src/sanitizer.rs`
- Define static lazy regexes using `std::sync::LazyLock`:
  ```rust
  static RE_ANCHOR: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
      regex::Regex::new(r#"<a(\s+[^>]*)?>"#).unwrap()
  });
  static RE_TARGET: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
      regex::Regex::new(r#"target="[^"]*""#).unwrap()
  });
  ```
- Use `RE_ANCHOR.replace_all(...)` and `RE_TARGET.replace(...)` in `sanitize_html`.

### 4. `crates/server/src/ai/extractor.rs`
- Define static lazy selectors using `std::sync::LazyLock`:
  ```rust
  static ARTICLE_SELECTOR: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(|| {
      scraper::Selector::parse("article").unwrap()
  });
  static BODY_SELECTOR: std::sync::LazyLock<scraper::Selector> = std::sync::LazyLock::new(|| {
      scraper::Selector::parse("body").unwrap()
  });
  ```
- Replace `Selector::parse(...)` calls with references to `ARTICLE_SELECTOR` and `BODY_SELECTOR`.

### 5. `crates/server/src/ai/llm.rs`
- Line 41:
  ```rust
  let _permit = self.semaphore.acquire().await.ok();
  ```

### 6. `crates/server/src/ai/db.rs`
- Move `PRAGMA journal_mode = WAL;` from `SqliteCustomizer::on_acquire` into `init_schema()`.
- Keep `PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;` in `SqliteCustomizer::on_acquire`.

### 7. `crates/miniflux/src/summary.rs`
- Add `strip_html_tags(content: &str) -> String` to strip markup and normalize whitespace.
- Call `strip_html_tags` inside `extract_summary` prior to `.unicode_sentences()`.

### 8. `crates/ui-components/templates`
- In `crates/ui-components/templates/base.html` and `pages/index.html`:
  - When HTMX swaps elements (`htmx:afterSwap`), scope icon rendering:
    ```javascript
    document.body.addEventListener('htmx:afterSwap', function(evt) {
      if (window.lucide && typeof lucide.createIcons === 'function') {
        lucide.createIcons({ root: evt.detail.target });
      }
    });
    ```
  - When calling after modal or toast updates, pass `{ root: document.getElementById('...') }` where appropriate.
- **Hide Paging When No News**:
  - In `crates/ui-components/templates/partials/entry_list.html`, ensure pagination `<nav aria-label="Pagination">` is not rendered when `entries.is_empty()`.
  - In `crates/ui-components/templates/pages/index.html`, in `filterArticlesClientSide(query)`, hide the pagination element whenever visible items count is 0 or when search filter is active.
- **Remove Counting Next to App Title**:
  - In `crates/ui-components/templates/pages/index.html`, remove `#unread-pill` next to `<span class="font-display ...">My News</span>` in the top header.
- **Fix Loading Indicator Visibility**:
  - In `crates/ui-components/templates/base.html`, update `.htmx-indicator` CSS:
    ```css
    .htmx-indicator {
      display: none;
      opacity: 0;
      transition: opacity 150ms ease-in-out;
      pointer-events: none;
    }
    .htmx-request.htmx-indicator,
    .htmx-request .htmx-indicator {
      display: block;
      opacity: 1;
    }
    ```
    This prevents Tailwind's `animate-pulse` keyframe from overriding idle opacity when no request is running.

---

## Verification & Test Plan

1. **Unit Tests**:
   - `test_miniflux_cache_bounded_capacity`: Verify that cache size does not exceed `MAX_CACHE_CAPACITY` and evicts expired or oldest entries.
   - `test_rate_limiter_stale_bucket_pruning`: Verify that buckets idle for > 10 minutes are pruned.
   - `test_rate_limiter_invalid_params_fallback`: Verify fallback to valid positive parameters when non-positive inputs are passed.
   - `test_sanitizer_preserves_links_with_lazy_regex`: Verify anchor target injection works identically with `LazyLock`.
   - `test_extractor_with_lazy_selectors`: Verify article and body parsing works identically with `LazyLock`.
   - `test_summary_extractor_strips_html_tags`: Verify HTML tags are removed from sentences without tag residue.
   - `test_sqlite_pool_wal_mode`: Verify `PRAGMA journal_mode` reports `wal` on pool connections.
2. **Clippy & Linter Check**:
   - Run `cargo clippy --workspace --all-targets` and verify 0 warnings.
3. **Full Test Suite**:
   - Run `cargo test --workspace` and verify all tests pass.

---

## References

- [Rust standard library `std::sync::LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)
- [SQLite WAL documentation](https://www.sqlite.org/wal.html)
- [Lucide Icons `createIcons` API](https://lucide.dev/guide/packages/lucide#createicons)
- [Ammonia HTML Sanitizer](https://docs.rs/ammonia/latest/ammonia/)
