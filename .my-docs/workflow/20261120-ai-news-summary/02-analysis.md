# Analysis: AI News Batch Summary

**Result:** done

## Context
The application is a spatial RSS reader built with Rust, Axum, HTMX, and Miniflux. Users require an executive AI news digest of the latest 15 articles grouped by feed source. 

When the user activates the `AI` filter pill:
1. The server queries the latest 15 articles across feeds and organizes them under feed source headers.
2. The initial response renders lightweight card skeletons with `hx-trigger="revealed"`.
3. As cards enter the viewport, individual HTMX requests fetch the summary.
4. If cached in SQLite, the summary returns instantly.
5. If uncached, the server ensures full content (fetching and parsing the `<article>` tag if the feed only had a teaser), runs a fast LLM CLI with zero reasoning budget, caches the result in SQLite, and returns the 1–2 sentence summary.
6. If any step fails (network block, paywall, CLI timeout), the system seamlessly falls back to the extractive summary.

---

## Technical & Architectural Risks

### 1. External Scraper Stalls & Anti-Bot Challenges
- **Risk**: Remote article websites may hang, encounter Cloudflare Turnstile/CAPTCHA challenges, or return HTTP 403 Forbidden.
- **Impact**: Server threads could stall or users could see broken cards.
- **Mitigation**: Enforce a strict 3-second HTTP timeout with `reqwest`. Treat any non-200 response, Cloudflare barrier, or timeout as an immediate trigger to fall back to RSS title + teaser excerpt. Never retry indefinitely and never stall the batch.

### 2. LLM CLI Process Hanging & Resource Exhaustion
- **Risk**: Spawning CLI processes (`llm`, `gemini-cli`, etc.) without strict controls can cause orphan processes or exhausted file descriptors.
- **Impact**: Memory leaks, high CPU load, or unresponsive UI slots.
- **Mitigation**: Wrap CLI command execution with `tokio::time::timeout` (e.g., 5 seconds) and ensure `kill_on_drop(true)` on `tokio::process::Command`. If the process fails or times out, immediately fall back to `crates/miniflux/src/summary.rs`.

### 3. Rapid Viewport Scroll & Concurrent CLI Burst
- **Risk**: If a user quickly scrolls through all 15 articles, 15 HTMX `revealed` requests could trigger 15 simultaneous LLM CLI processes.
- **Impact**: High system load and degraded responsiveness.
- **Mitigation**: Use a Tokio semaphore (e.g., `Arc<Semaphore>` with 2 permits) to serialize/throttle concurrent LLM CLI invocations. Fast SQLite cache hits bypass the semaphore entirely.

### 4. SQLite Concurrency in Async Axum
- **Risk**: SQLite (`rusqlite`) calls are synchronous/blocking and can starve the Tokio runtime if executed on async worker threads.
- **Impact**: Increased request latency across all endpoints.
- **Mitigation**: Execute SQLite queries inside `tokio::task::spawn_blocking` with an `Arc<Mutex<rusqlite::Connection>>` or a lightweight connection pool.

### 5. Content Sanitization & Prompt Delimiting
- **Risk**: Untrusted web article text could contain prompt injections or malformed markup.
- **Impact**: Erroneous LLM output or XSS vulnerabilities.
- **Mitigation**: Clean extracted HTML to plain text, truncate to Title + 400–600 words, wrap inside clear prompt delimiters (e.g., `<article_body>...</article_body>`), and sanitize rendered HTML via `ammonia`.

---

## Open Decisions (Frontier for Grill)
1. **Article Extraction Method**: Use `scraper` crate (built on `html5ever`, which is already in the lockfile) to query the `<article>` DOM element, or regex-based extraction.
2. **SQLite Access Pattern**: `tokio::task::spawn_blocking` with `Arc<tokio::sync::Mutex<Connection>>` vs connection pool.
3. **LLM CLI Concurrency Throttling**: Tokio `Semaphore(2)` to cap simultaneous CLI processes.
4. **Cache Invalidation & Schema**: SQLite table schema `article_ai_summaries` keyed by `entry_id`.
