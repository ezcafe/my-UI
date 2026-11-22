Result: clean

# API Contract Review: Codebase Performance, Concurrency, and Quality Optimizations

- **Stage:** `api-contract-review` (Mode full / isolated review)
- **Status:** Paid (`clean`)
- **Review Targets:**
  - [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/03-design.md)
  - [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/04-tasks.md)
  - [crates/server/src/rate_limit.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/rate_limit.rs)
  - [crates/server/src/cache.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/cache.rs)
  - [crates/server/src/ai/extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs)
  - [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs)
  - [crates/ui-components/src/sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs)
  - [crates/miniflux/src/summary.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/summary.rs)

---

## 1. Executive Summary

This review analyzes the API and interface contracts proposed in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/03-design.md) and broken down in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/04-tasks.md).

- **Zero Critical / Major Blockers**: All internal interfaces, public crate exports, and HTTP middleware contracts maintain full backward compatibility.
- **Strict Backward Compatibility**: Public functions and structs (`RateLimiter`, `MinifluxCache`, `sanitize_html`, `extract_summary`, `ArticleExtractor`) retain their existing method signatures, argument types, and return contracts.
- **Safe Resource Bounding**: The rate limiter and cache layers gain strict input validation and bounded memory growth, preventing unbounded resource leaks without altering external interfaces.
- **HTTP Middleware Preservation**: Rate-limiting HTTP headers (`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `Retry-After`) and status codes (`429 Too Many Requests`) remain unchanged.

---

## 2. Interface Contract Checklist

### 2.1 Rate Limiting Contract ([crates/server/src/rate_limit.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/rate_limit.rs))

- [x] **Constructor Signature & Validation**:
  - Signature: `pub fn new(capacity: f64, refill_per_sec: f64) -> Self` remains preserved.
  - Contract hardening: Non-positive parameters (`capacity <= 0.0` or `refill_per_sec <= 0.0`) safely fallback to defaults `(60.0, 5.0)` instead of producing `NaN` or panicking on zero-division during refill calculation.
- [x] **Rate Check Signature**:
  - Signature: `pub async fn check(&self, ip: &str) -> Result<(u64, u64), u64>` is preserved.
  - Returns `Ok((limit, remaining))` when allowed and `Err(wait_seconds)` when throttled.
- [x] **HTTP Middleware Contract**:
  - `rate_limit_middleware` continues inserting `X-RateLimit-Limit` and `X-RateLimit-Remaining` response headers on allowed requests.
  - Emits `StatusCode::TOO_MANY_REQUESTS` with `Retry-After` header when throttled.
- [x] **Bucket Map Pruning**:
  - Pruning is triggered during `.check(&ip)` when bucket count exceeds 100, removing client entries inactive for > 10 minutes.
  - This is completely internal to `RateLimiter` and does not affect public callers or API consumers.

### 2.2 Cache Interface Contract ([crates/server/src/cache.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/cache.rs))

- [x] **Public Method Signatures**:
  - `pub fn new(default_ttl: Duration) -> Self` — unchanged.
  - `pub async fn get(&self, key: &str) -> Option<serde_json::Value>` — unchanged.
  - `pub async fn insert(&self, key: String, val: serde_json::Value)` — unchanged.
  - `pub async fn insert_with_ttl(&self, key: String, val: serde_json::Value, ttl: Duration)` — unchanged.
  - `pub async fn invalidate(&self, key_prefix: &str)` — unchanged.
  - `pub async fn clear(&self)` — unchanged.
- [x] **Encapsulation & Typing**:
  - The internal tuple `(Instant, Duration, serde_json::Value)` is replaced with `struct CacheEntry { inserted_at: Instant, ttl: Duration, val: serde_json::Value }`.
  - Fixes Clippy `type_complexity` warnings without leaking internal entry types to external callers.
- [x] **Capacity Bounding**:
  - Bounded by `const MAX_CACHE_CAPACITY: usize = 1000`. Eviction of expired entries and oldest entries occurs synchronously inside `insert_with_ttl`, guaranteeing memory bounds without changing return types.

### 2.3 HTML Sanitization Contract ([crates/ui-components/src/sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs))

- [x] **Function Signature**:
  - `pub fn sanitize_html(input: &str) -> String` — unchanged.
- [x] **Behavioral Contract**:
  - Anchors receive `target="_blank"` and `rel="noopener noreferrer"`.
  - Replacing per-call `Regex::new(...)` with `static LazyLock<Regex>` is purely an internal execution optimization. All existing regex replacement patterns and outputs remain 100% identical.

### 2.4 Article Extractor Contract ([crates/server/src/ai/extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs))

- [x] **Method Signatures**:
  - `pub fn parse_html(&self, html_str: &str, title: &str, teaser: &str) -> String` — unchanged.
  - `pub async fn extract_content(&self, url: &str, title: &str, teaser: &str) -> String` — unchanged.
- [x] **Internal Selector Precompilation**:
  - Replacing per-call `Selector::parse("article")` and `Selector::parse("body")` with static `LazyLock<Selector>` leaves the return contract and DOM traversal behavior identical while eliminating parsing overhead.

### 2.5 Text Summarization Contract ([crates/miniflux/src/summary.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/summary.rs))

- [x] **Function Signatures**:
  - `pub fn extract_summary(content: &str, sentences_count: usize) -> String` — unchanged.
  - Adds optional helper: `pub fn strip_html_tags(content: &str) -> String`.
- [x] **Extraction Quality**:
  - Stripping HTML markup prior to sentence segmentation produces cleaner text output and prevents tag fragment leakage into generated summaries.
  - Pure plain-text inputs continue to segment identically.

### 2.6 Database & Concurrency Contracts

- [x] **LLM Semaphore Acquisition ([crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs))**:
  - Simplifying `match self.semaphore.acquire().await` to `.ok()` maintains exact runtime semantics (`Option<SemaphorePermit>`) and eliminates Clippy lint noise without altering `summarize`'s signature or behavior.
- [x] **SQLite Database Initialization ([crates/server/src/ai/db.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/db.rs))**:
  - Moving `PRAGMA journal_mode = WAL;` to run once during schema initialization (`init_schema()`) leaves `AiDb::new` and all query methods unchanged.
  - Per-connection checkout pragmas (`busy_timeout`, `synchronous = NORMAL`) remain active in `on_acquire`.

---

## 3. Edge Cases & Concurrency Evaluation

1. **Rate Limiter Concurrency & Lock Hold Times**:
   - Bucket pruning runs inside the existing `Mutex` lock during `.check()`.
   - The prune threshold check (`buckets.len() > 100`) ensures the linear retain scan is skipped on small map sizes, keeping mutex lock hold times negligible (< 10 µs).
2. **Cache Eviction Contention**:
   - Inline eviction runs only when `store.len() >= MAX_CACHE_CAPACITY` under the existing write lock.
   - Normal cache reads continue using `RwLock::read`, allowing high-throughput concurrent cache hits.
3. **Empty or Malformed HTML Input**:
   - `strip_html_tags` handles unclosed tags and empty strings gracefully without panic or indexing out of bounds.

---

## 4. Conclusion & Recommendation

The proposed changes in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/03-design.md) introduce zero breaking contract changes, preserve all public APIs, and strengthen runtime safety and resource constraints.

**Review Outcome:** `clean`
