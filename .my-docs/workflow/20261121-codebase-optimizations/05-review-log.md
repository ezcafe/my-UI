# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

---

## Adversarial

**Result:** clean

### Adversarial Evaluation & Breakage Analysis

1. **Cache Capacity & Eviction Boundary Tests (`crates/server/src/cache.rs`)**:
   - *Attack / Stress vector:* Rapid burst insertions exceeding `MAX_CACHE_CAPACITY` (1,000 items) with mixed expiration timestamps.
   - *Findings:* Verified that `insert_with_ttl` first purges expired entries using `.retain(|_, entry| !entry.is_expired())`. If capacity remains full, it deterministically locates the oldest item via `entry.inserted_at` and evicts it. No unbounded memory growth or resource exhaustion is possible.
   - *Lock contention:* Write lock is held only during map pruning and insertion; read operations utilize `RwLock::read()` and bypass expired items without locking writers.

2. **Rate Limiter Degenerate Inputs (`crates/server/src/rate_limit.rs`)**:
   - *Attack / Stress vector:* Passing negative or zero capacity/refill rates (e.g. `RateLimiter::new(-1.0, 0.0)` or `(0.0, -5.0)`).
   - *Findings:* Input validation catches `capacity <= 0.0 || refill_per_sec <= 0.0` and substitutes defaults `(60.0, 5.0)`. This completely eliminates division by zero, `NaN` token calculations, and permanent client lockout.
   - *Memory leak attack:* Spoofing millions of unique IP addresses over time. Pruning logic triggers when bucket count exceeds 100, removing entries idle for > 10 minutes (`Duration::from_secs(600)`), bounding heap memory.

3. **HTML Sanitization & Tag Stripping Edge Cases (`crates/miniflux/src/summary.rs`)**:
   - *Attack / Stress vector:* Malformed HTML (unclosed `<p>` tags, mismatched `<>` brackets, dense entity sequences).
   - *Findings:* `strip_html_tags` consumes characters linearly, correctly tracking `in_tag` state and replacing closing `>` with space delimiters to avoid word concatenation. Unescaping replaces standard HTML entities and normalizes punctuation spacing. No panic or buffer overflow can occur.

4. **UI HTMX Indicator State Leaks (`crates/ui-components/templates/base.html`)**:
   - *Attack / Stress vector:* Interaction with Tailwind's `animate-pulse` keyframe animation which modifies opacity during idle state.
   - *Findings:* Explicit `display: none` on `.htmx-indicator` and `display: block` on `.htmx-request.htmx-indicator` prevents Tailwind CSS animation keyframes from bleeding opacity into idle indicators.

---

## Quality

**Result:** clean

### Five-Axis Quality Pass

1. **Correctness**:
   - All unit and integration tests compile cleanly.
   - Cache bounds memory to 1,000 items with two-tier eviction (expired first, then oldest).
   - Static regexes and selectors (`LazyLock`) maintain identical matching behavior while improving throughput.
   - SQLite WAL mode is activated once during schema initialization and verified via test.

2. **Readability & Maintainability**:
   - Replaced complex raw tuple `(Instant, Duration, serde_json::Value)` with dedicated `CacheEntry` struct containing clear helper method `is_expired()`.
   - Cleaned up manual Clippy match construct in `crates/server/src/ai/llm.rs` using `.ok()`.
   - Templating code in `index.html` and `base.html` uses clear, well-commented JS helpers for DOM-scoped Lucide icon creation.

3. **Architecture & Layering**:
   - Avoided introducing heavyweight external parsing dependencies (e.g. `scraper` / `ammonia`) into `miniflux-client`; implemented a lightweight, fast tag-stripping routine.
   - Database pragma separation respects SQLite architecture: connection-level pragmas (`busy_timeout`, `synchronous`) remain in `on_acquire`, while database-file-level pragmas (`journal_mode = WAL`) run during `init_schema()`.

4. **Security**:
   - Strict resource bounding on in-memory caches and rate limiter state prevents denial-of-service (DoS) via memory exhaustion.
   - Safe HTML link handling preserves `rel="noopener noreferrer"` and `target="_blank"`.
   - SQL queries remain 100% parameterized without raw string formatting.

5. **Performance**:
   - `std::sync::LazyLock` precompiles regular expressions and CSS selectors once at application startup, eliminating repeated parsing on hot request paths.
   - Removing `PRAGMA journal_mode = WAL;` from pooled connection checkouts avoids redundant disk checkpoints and lock acquisition.
   - Scoping `lucide.createIcons({ root: target })` restricts DOM traversals to updated subtrees, eliminating full-page icon re-renders on every HTMX swap.

---

## Merged

**Result:** clean

### Lens Synthesis & Final Sign-Off

- **API Lens ([05-lens-api.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/05-lens-api.md)):** `clean`
  - Zero breaking changes to public APIs.
  - Full backward compatibility for HTTP headers, status codes, and service structs.
- **DB Lens ([05-lens-db.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-codebase-optimizations/05-lens-db.md)):** `clean`
  - Pragma separation verified; WAL mode active in pool connections.
  - Zero migration or locking risks.
- **Adversarial & Quality Reviews:** Both `clean` across all five axes.
- **Final Verdict:** All review stages and conditional lenses are clean. No fixes required.
