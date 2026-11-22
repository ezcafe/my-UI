# Code review log

**Overall Result:** clean  

---

## Adversarial

**Result:** clean  

### Scope & Methodology
Hunted for edge cases, failure states, prompt injection risks, concurrency deadlocks, silent data truncation, race conditions, and unhandled errors across:
- `crates/server/src/routes/entries.rs`: Lazy executive brief endpoint `/hx/executive-brief`, OOB title rendering, in-flight deduplication, error collapse contract, and detail drawer loading.
- `crates/server/src/ai/llm.rs`: Multilingual prompt isolation, LLM CLI runner, regex and fenced code block fallback parsing, timeout protection.
- `crates/server/src/ai/db.rs`: SQLite migration v3, batched `IN (...)` queries, transaction wrapping, concurrent connection handling.
- `crates/server/src/ai/prefetch.rs`: Page prefetch thresholding, duplicate prevention, and async job execution.
- `crates/server/src/config.rs`: Decoupled `AI_ENABLED` flag parsing and matrix permutations.
- `crates/ui-components/templates/`: Annuaire pulsing skeleton, executive brief card, entry item OOB title spans, and container queries.

### Findings & Mitigations
1. **Prompt Injection & Untrusted LLM Output Handling**:
   - In `LlmCliRunner::format_batch_prompt`, untrusted feed content is formatted with explicit article boundaries (`[ID {}] Title: ... Content: ...`).
   - In `executive_brief_htmx`, OOB title updates strictly filter against verified `entries` queried from the authenticated/trusted user's active feed view. An attacker injecting fabricated IDs or extra keys into LLM output cannot inject arbitrary rows or hijack unvisited articles.
   - All dynamic titles in Out-Of-Band spans pass through `escape_html()`, escaping `&`, `<`, `>`, `"`, and `'`. Brief cards rendered via Askama templates automatically escape HTML content, preventing stored XSS.
2. **Concurrency & Deadlock Safety**:
   - `executive_brief_htmx` implements thundering herd protection via `ai_service.in_flight.lock().await`. The lock is held only long enough to check and register the `page_key`, and is explicitly dropped before awaiting the in-flight synthesis.
   - Resource cleanup is guaranteed by `InFlightGuard::drop`, which removes the key from the set even if the handler panics, returns early, or hits an unexpected branch.
   - SQLite connections use WAL mode with `PRAGMA busy_timeout = 5000` and connection pooling via `r2d2`. All blocking SQLite operations run inside `tokio::task::spawn_blocking`, ensuring Tokio async runtime worker threads are never blocked.
   - Subprocess execution is throttled via `Semaphore::new(2)` and protected by `tokio::time::timeout(Duration::from_secs(90))` with `kill_on_drop(true)` to prevent zombie child processes.
3. **Graceful Failure Contract**:
   - If the LLM CLI process crashes, exits non-zero, times out, or returns invalid JSON, `/hx/executive-brief` returns an HTTP 200 OK with `<div id="page-executive-brief" class="hidden"></div>` and zero OOB spans.
   - This ensures HTMX cleanly swaps out the Annuaire pulsing skeleton and collapses the placeholder without pulsing indefinitely, disrupting layout, or firing error toasts.
4. **Empty Entries & Ineligible View Guards**:
   - If `entries.is_empty()` (e.g. empty feed or caught up), `/hx/executive-brief` returns an immediate empty response without invoking hashing or LLM processes.
   - `is_brief_eligible_filter` strictly rejects `today`, `read`, `starred`, `all`, and active search queries, ensuring briefs are only generated where contextually intended.
5. **Data Truncation & Upsert Safety**:
   - `AiDb::save_page_brief_by_key` and `AiDb::save_batch_results` utilize SQLite `ON CONFLICT(page_key) DO UPDATE` and `ON CONFLICT(entry_id) DO UPDATE`. Concurrent writes for identical page keys or entry summaries update cleanly without throwing unique constraint violations.

---

## Quality

**Result:** clean  

### Five-Axis Review

1. **Correctness**:
   - Complete parity with `03-design.md` specifications.
   - Decoupled `AI_ENABLED` allows AI digest features in guest mode (`AUTH_ENABLED=false`) while respecting mutation restrictions.
   - Deterministic page fingerprint calculation (`compute_page_key`) ensures cache validity across varying feed parameters and article lists.
   - 161 automated tests pass across unit, integration, and e2e test suites (`miniflux-client`, `server`, `ui-components`).
   - Clean `cargo clippy --workspace --all-targets -- -D warnings` with zero warnings.
2. **Readability & Simplicity**:
   - Clear and concise functions: `compute_page_key`, `render_title_oob_span`, `is_brief_eligible_filter`.
   - Annuaire design system tokens and typography classes (`font-sans`, `line-clamp-2`, `text-brand`, `p-3.5`) are adhered to consistently across templates.
3. **Architecture & Layering**:
   - Clean separation of concerns:
     - `crates/server/src/ai/db.rs`: Database migrations, transactions, and batch queries.
     - `crates/server/src/ai/llm.rs`: Prompt composition, subprocess execution, semaphore throttling, and parsing.
     - `crates/server/src/ai/prefetch.rs`: Thresholded background prefetching.
     - `crates/server/src/routes/entries.rs`: HTTP route endpoints, HTMX response composition.
     - `crates/ui-components/templates/`: Pure presentation templates with zero embedded business logic.
4. **Security**:
   - SQL queries are strictly parameterized with dynamic binds; no raw SQL string formatting.
   - HTML attributes and titles are sanitized and escaped against XSS.
   - Authentication guards are enforced on all mutating actions; unauthenticated mutations return 403 Forbidden.
5. **Performance (<50ms Page Rendering)**:
   - Initial page loads render the pulsing skeleton instantly (<50ms).
   - Detail drawer reads strictly from cache or extractive teaser without launching CLI processes.
   - Uncached entries query uses a single batched `WHERE entry_id IN (...)` statement, avoiding N+1 connection acquisitions.
   - Multi-item summary updates execute within a single transaction in `AiDb::save_batch_results`.

---

## Merged

**Result:** clean  

### Lens Summary
- **Lens `api`**: Clean (`.my-docs/workflow/20261122-batch-page-ai-brief/05-lens-api.md`). Public endpoints follow typed input parsing, backward compatibility, stable error collapse contracts, and XSS-safe Out-Of-Band title updates.
- **Lens `db`**: Clean (`.my-docs/workflow/20261122-batch-page-ai-brief/05-lens-db.md`). Migration v3 verified, bounded batch queries, transactional atomic persistence, upsert conflict handling, and non-blocking WAL mode.
- **Action Items**: None. All adversarial checks, quality axes, and required lenses passed clean.
