# Grill: Stress-Test & Decision Frontier

Result: frontier-empty

## Decision Tree & Settled Decisions

All architectural decisions and trade-offs have been explored and settled with confirmed user choices:

---

### 1. In-Memory Cache Eviction & Capacity Bounds (Settled)
- **Question**: How should `MinifluxCache` bound storage and handle stale entries without leaking memory or complicating background runtime lifecycles?
- **Settled Decision**: **Option A (Inline Eviction on Insert/Get with Max Capacity)**.
  - When items are inserted, if `store.len() >= MAX_CAPACITY` (1,000 entries), purge expired entries (`store.retain(|_, entry| !entry.is_expired())`).
  - If the store is still at capacity, evict the oldest entry by `inserted_at`.
  - Encapsulate the value in a named `CacheEntry` struct (`inserted_at`, `ttl`, `val`) to resolve `clippy::type_complexity`.
- **Status**: **Settled** (Confirmed user choice in `01-idea.md`).

---

### 2. HTML Tag Stripping in Summary Extractor (Settled)
- **Question**: How should `crates/miniflux/src/summary.rs` strip HTML tags from feeds prior to sentence segmentation without adding heavy parser dependencies to `miniflux-client`?
- **Settled Decision**: **Option A (Fast In-Crate Tag Stripping)**.
  - Implement a dedicated tag-stripping helper in `summary.rs` that strips `<...>` tags and unescapes basic HTML entities before sentence segmentation.
  - Keeps `miniflux-client` lightweight with zero new external crate dependencies.
- **Status**: **Settled** (Confirmed user choice in `01-idea.md`).

---

### 3. Rate Limiter Stale Bucket Pruning & Parameter Validation (Settled)
- **Question**: How should `RateLimiter` prevent memory growth from ephemeral client IPs and protect against invalid configurations?
- **Settled Decision**:
  - Validate parameters on instantiation (`capacity > 0.0` and `refill_per_sec > 0.0`).
  - Prune client buckets with `last_refill.elapsed() > Duration::from_secs(600)` (10 minutes) during rate limit checks.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

---

### 4. Regex & CSS Selector Precompilation (Settled)
- **Question**: Which mechanism should be used to precompile regular expressions and HTML selectors in `sanitizer.rs` and `extractor.rs`?
- **Settled Decision**:
  - Use `std::sync::LazyLock` for static initialization.
  - Precompile anchor and target regexes in `crates/ui-components/src/sanitizer.rs`.
  - Precompile `article` and `body` selectors in `crates/server/src/ai/extractor.rs`.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

---

### 5. SQLite PRAGMA Optimization in `AiDb` (Settled)
- **Question**: Where should SQLite pragmas be configured to avoid redundant overhead on connection checkout?
- **Settled Decision**:
  - Execute `PRAGMA journal_mode = WAL;` once in `init_schema()`.
  - Retain per-connection pragmas (`PRAGMA busy_timeout = 5000; PRAGMA synchronous = NORMAL;`) in `SqliteCustomizer::on_acquire`.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

---

### 6. Frontend DOM Scoping for Lucide Icons (Settled)
- **Question**: How should Lucide icon rendering be optimized during HTMX swaps?
- **Settled Decision**:
  - Scope `lucide.createIcons({ root: container })` to updated elements or `evt.detail.target` during swap events instead of rescanning the entire `document.body`.
- **Status**: **Settled** (Confirmed in `01-idea.md`).
