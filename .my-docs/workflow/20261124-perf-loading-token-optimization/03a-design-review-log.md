Result: clean

# General Design Review: Performance, Loading, and Token Optimization

**Date:** 2026-11-24  
**Workflow Slug:** `20261124-perf-loading-token-optimization`  
**Review Stage:** `design-review`  
**Target Design:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03-design.md)  
**Supporting Reviews:**
- [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03a-api-contract-review.md) (`Result: clean`)
- [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03a-db-design-review.md) (`Result: clean`)

---

## 1. Executive Summary

This review assesses the comprehensive system design in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03-design.md) across five core engineering dimensions:
1. **Performance**: Token reduction, latency optimization, and fast-paths.
2. **Security**: Endpoint authentication, prompt grounding, and database safety.
3. **Architecture**: Clean modular boundaries and in-flight deduplication.
4. **UI/UX Consistency**: Instant drawer response, skeleton loaders, and zero FOUC.
5. **Backwards Compatibility**: Schema stability, route preservation, and fallback cascades.

- **Verdict:** `Result: clean`
- **Findings:** Zero Critical or Major issues identified. The design is robust, highly cohesive, directly satisfies all decisions from [02b-grill.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/02b-grill.md), and seamlessly integrates findings from both API and DB reviews.

---

## 2. Evaluation Across Review Dimensions

### 2.1 Performance
- **Token budget reduction (~80% savings):** The batch prompt generates an inverted-pyramid rewrite only for the top-1 lead article, deferring rewrites for the remaining 14 articles until opened. Combined with capping article prompt input at 130 words (down from 300 words), output tokens drop by 1,200–1,500 tokens per page view.
- **Sub-50ms drawer response:** Article drawers open instantly without waiting for upstream LLM round-trips. The 3-sentence executive brief renders on initial load, while uncached rewrites load asynchronously in the background.
- **Scraper fast-path & timeout:** Lowering the teaser fast-path threshold to 25 words eliminates network scrapes for more than 70% of RSS feeds. Reducing HTTP scraper timeout from 3.0s to 1.5s prevents slow upstream servers from stalling processing threads.
- **Zero layout shift & CSS delivery:** Precompiling CSS into `/static/annuaire-theme.css` eliminates runtime evaluation of `cdn.tailwindcss.com`, removing the 150–300ms flash of unstyled content (FOUC) on initial page load.
- **Canonical cache keys:** Standardizing `compute_page_key(status, entry_ids)` ensures prefetch jobs and foreground requests share the same cache entries and lock mutexes, eliminating orphaned prefetch records and redundant LLM executions.

### 2.2 Security & Protection
- **Authentication guard on on-demand rewrite:** In line with recommendations from [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03a-api-contract-review.md), the new `GET /hx/entries/{id}/rewrite` endpoint enforces user authentication when `config.auth_enabled` is active, preventing unauthorized LLM token depletion.
- **Strict prompt grounding:** Batch and on-demand prompts include mandatory `GROUNDING CONSTRAINT` instructions forbidding extrapolation and external assertions, preventing hallucinations.
- **SQL injection safety:** All database queries in `AiDb` utilize parameterized queries (`rusqlite::params!`) with zero string interpolation.
- **Static route isolation:** The `/static/annuaire-theme.css` route strictly returns the dedicated stylesheet with fixed content types and headers, avoiding arbitrary file disclosure risks.

### 2.3 Architecture & System Boundaries
- **Clear separation of concerns:**
  - `AiDb` ([crates/server/src/ai/db.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/db.rs)): Encapsulates SQLite connection pooling, index lookups, and upserts.
  - `AiService` ([crates/server/src/ai/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs)): Manages in-flight deduplication mutexes, background worker prefetching, and LLM orchestration.
  - Route handlers ([crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)): Parse HTTP requests, check permissions, and return Askama HTML templates.
  - UI templates ([crates/ui-components](file:///Users/ptquang86/ws/my-ui/crates/ui-components)): Manage visual presentation, skeleton loaders, and HTMX swap targets.
- **Concurrency & WAL mode:** SQLite WAL mode with 8 pooled connections and `PRAGMA busy_timeout = 5000` allows parallel reads during batch upserts without lock contention.
- **In-flight request deduplication:** The shared `ai_service.in_flight` mutex prevents duplicate concurrent requests for the same page key or single-entry rewrite.

### 2.4 UI & UX Consistency
- **Predictable layout replacement:** Both the skeleton loader and the delivered rewrite share the root `#ai-rewrite-box` container with identical prose styling classes, ensuring seamless 1:1 `outerHTML` replacement without visual jumping.
- **No raw content dump:** As decided in Grill 1A variant, raw article text and duplicate teasers are suppressed below the skeleton loader while the rewrite is pending.
- **Resilient error states:** If on-demand rewrite generation encounters a network error or timeout, the handler falls back to rendering the 3-sentence brief inside `#ai-rewrite-box`, ensuring the user never sees a broken drawer or infinite loading animation.
- **Offline stylesheet support:** Serving static CSS locally removes external runtime CDN dependencies, enabling full offline development and styling consistency.

### 2.5 Backwards Compatibility
- **Zero database migrations:** The existing database schema (`PRAGMA user_version = 4`) already contains `rewrite TEXT` in `article_ai_summaries` and `page_executive_briefs_v2`. No table alteration or schema migration scripts are required.
- **Non-destructive upserts:** As detailed in [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-perf-loading-token-optimization/03a-db-design-review.md), updates to article rewrites preserve existing summary text via `COALESCE`, preventing accidental data loss.
- **Graceful degradation when AI disabled:** If `config.ai_enabled` is false, endpoints and drawer templates fall back gracefully to displaying existing teasers and summaries without runtime exceptions.

---

## 3. Findings Summary

| Issue ID | Severity | Category | Description | Status |
| :--- | :--- | :--- | :--- | :--- |
| **None** | - | - | Zero Critical or Major issues identified | **PASS** |

### Minor Implementation Reminders (from Isolated Reviews)
1. **Enforce `_user: AuthenticatedUser` on rewrite endpoint:** Ensure `GET /hx/entries/{id}/rewrite` includes auth guards matching other detail endpoints.
2. **Dedicated rewrite saver in `AiDb`:** Provide a dedicated helper (`save_rewrite(entry_id, rewrite)`) to prevent accidental overwriting of executive briefs with empty strings.
3. **In-flight key for single rewrite:** Track `format!("rewrite:{}", id)` in `ai.in_flight` during on-demand rewrite generation to prevent duplicate LLM calls on double clicks.
4. **Cache-Control headers:** Set `Cache-Control: no-cache, no-store, must-revalidate` on dynamic HTMX fragment responses.

---

## 4. Stage Verdict & Transition

- **Stage Verdict:** `Result: clean`
- **Conclusion:** All architectural, performance, security, and UI/UX requirements are satisfied. The design is approved and ready for transition to the parent orchestrator for Gate B derivation.
