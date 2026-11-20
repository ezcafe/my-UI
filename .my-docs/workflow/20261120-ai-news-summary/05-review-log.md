# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

## Adversarial

**Result:** clean

### Review Scope & Threat Modeling
- **Subprocess Execution (`crates/server/src/ai/llm.rs`)**:
  - Validated that `tokio::process::Command::new(&self.cli_cmd)` executes binary directly via `execve` arguments, preventing shell injection vulnerabilities from untrusted article titles or contents.
  - Subprocess execution is wrapped in `tokio::time::timeout(Duration::from_secs(5))` and uses `kill_on_drop(true)` to prevent orphaned or hanging child processes.
  - Bounded concurrency via `Arc<Semaphore>` (2 permits) prevents process exhaustion / fork bombs under traffic spikes.
- **Content Extraction & SSRF Considerations (`crates/server/src/ai/extractor.rs`)**:
  - Web scraping uses a dedicated `reqwest::Client` with a strict 3-second timeout (`Duration::from_secs(3)`).
  - DOM parsing traverses `<article>` tag hierarchy, explicitly ignoring dangerous or boilerplate tags (`<script>`, `<style>`, `<nav>`, `<header>`, `<footer>`, `<aside>`, `<noscript>`, `<svg>`, `<form>`, `<button>`).
  - Graceful fallback on network errors, 403 Forbidden, 429 Rate Limited, or Cloudflare blocks immediately returns title and 2-sentence teaser without blocking or failing requests.
  - Content truncated to 500 words before feeding to prompt to prevent token exhaustion and slow inference.
  - *Defense-in-depth note*: URLs originate from user-configured RSS feed entries in Miniflux; for multi-tenant cloud hosting, optional egress restrictions on cloud metadata endpoints (e.g. `169.254.169.254`) are recommended.
- **Authentication & Authorization Guardrails (`crates/server/src/routes/entries.rs`)**:
  - `_user: AuthenticatedUser` extractor protects both `/hx/entries?status=ai` and `/hx/entries/{id}/ai-summary`.
  - Prior to reading the SQLite cache or spawning LLM tasks, `/hx/entries/{id}/ai-summary` validates that entry `{id}` exists and is accessible under the authenticated user's Miniflux session, preventing IDOR / cross-tenant summary leakage.
  - HTMX outerHTML swap response is guaranteed: returns HTTP 200 replacement card on missing entries, failures, or extractor fallbacks, ensuring HTMX loading skeletons never freeze or leave broken UI states.

---

## Quality

**Result:** clean

### Five-Axis Review
1. **Correctness**:
   - Accurately groups the latest 15 entries by feed source into `Vec<FeedSummaryGroupViewModel>`.
   - Correctly integrates HTMX lazy-loading via `hx-trigger="revealed"` targeting `/hx/entries/{id}/ai-summary`.
   - Preserves search query forwarding (`search`) into GraphQL filter criteria.
   - Comprehensive TDD suite: 71 tests passing across `miniflux-client`, `server`, and `ui-components`.
2. **Readability & Maintainability**:
   - Clean, modular code organization in `crates/server/src/ai/` (`db.rs`, `extractor.rs`, `llm.rs`, `mod.rs`).
   - Idiomatic Rust with clear error types (`AiDbError`, `thiserror`).
   - Well-structured Askama templates (`index.html`, `ai_grouped_list.html`, `ai_summary_card.html`).
3. **Architecture**:
   - Aligns with Design Option A (Cohesive AI service within `crates/server`).
   - Clean separation between HTTP routing, persistence, scraping, and process management.
   - No unnecessary traits or premature abstraction layers.
4. **Security**:
   - Zero shell command string construction.
   - Safe SQL parameter binding throughout.
   - Standard Askama HTML escaping on dynamic user and article fields.
   - External links specify `target="_blank"` and `rel="noopener noreferrer"`.
5. **Performance**:
   - SQLite WAL mode with 8-connection `r2d2` pool ensures non-blocking concurrent reads.
   - Viewport-triggered lazy loading avoids bulk LLM execution for off-screen cards.
   - Concurrency bounded to 2 simultaneous LLM executions.
   - 500-word truncation caps input tokens.

---

## Merged

**Result:** clean

All review axes and conditional lenses (`lens-api`, `lens-db`) have completed with zero blocking findings.

| Review Axis / Lens | Output File | Result | Summary |
|---|---|---|---|
| **Adversarial** | `05-review-log.md` | `clean` | Process isolation, timeouts, IDOR protection, and guaranteed 200 HTMX swaps verified. |
| **Quality** | `05-review-log.md` | `clean` | Five-axis pass (correctness, readability, architecture, security, performance) clean. |
| **API Lens (`lens-api`)** | `05-lens-api.md` | `clean` | Endpoints, query parameters, auth guards, and error responses conform to contracts. |
| **DB Lens (`lens-db`)** | `05-lens-db.md` | `clean` | SQLite schema, indexes, versioning, WAL pooling, and UPSERT concurrency verified. |
