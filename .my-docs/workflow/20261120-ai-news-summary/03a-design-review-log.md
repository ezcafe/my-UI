Result: clean

# General Design Review: AI News Batch Summary

- **Stage:** `design-review` (Mode full / cross-cutting review)
- **Status:** Complete (`clean`)
- **Review Targets:**
  - Design Document: [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03-design.md)
  - Tasks & TDD Plan: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/04-tasks.md)
  - API Contract Review: [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03a-api-contract-review.md)
  - Database Design Review: [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03a-db-design-review.md)

---

## 1. Executive Summary

This general design review evaluates the end-to-end architecture, cross-cutting system behavior, fault tolerance, and implementation readiness of the AI News Batch Summary feature.

Key conclusions:
- **Zero Critical / Major Blockers**: All system components integrate cleanly without architectural contradictions or dangling dependencies.
- **Isolated Reviews Paid**: Both isolated reviews ([03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03a-api-contract-review.md) and [03a-db-design-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/03a-db-design-review.md)) are fully resolved and stamped `clean`.
- **Guaranteed UI Unsticking**: The HTMX lifecycle guarantees that lazy-loaded summary cards always receive HTTP 200 with replacement HTML, preventing permanently frozen loading spinners.
- **Process & Resource Isolation**: Concurrency is safely throttled across both SQLite connection pools (`r2d2` pool size 8) and CLI subprocesses (`tokio::sync::Semaphore` with 2 permits), avoiding thread and CPU starvation.
- **TDD Readiness**: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/04-tasks.md) provides 16 explicit test cases and 6 sequenced tasks with clear completion criteria.

---

## 2. Isolated Review Consolidation

Both upstream isolated design reviews have passed with clean verdicts:

1. **API Contract Review (`03a-api-contract-review.md`)**:
   - Standardized the canonical lazy-load route as `GET /hx/entries/{id}/ai-summary`.
   - Enforced search parameter forwarding (`query.search`) for `GET /hx/entries?status=ai`.
   - Formalized HTTP 200 response contracts for all failure and fallback scenarios.
   - Status: **Clean** (0 Critical, 0 Major).

2. **Database Design Review (`03a-db-design-review.md`)**:
   - Replaced async mutex serialization with `r2d2` connection pooling (8 connections).
   - Applied Write-Ahead Logging (`PRAGMA journal_mode = WAL;`) and `PRAGMA busy_timeout = 5000;`.
   - Added `PRAGMA user_version = 1;` schema tracking and index on `created_at`.
   - Enforced `ON CONFLICT(entry_id) DO UPDATE` upserts for race-free cache insertion.
   - Status: **Clean** (0 Critical, 0 Major).

---

## 3. Cross-Cutting Architecture Evaluation

### 3.1 UI / HTMX Lifecycle & Viewport Staggering
- **Filter Bar Activation**:
  - The `[AI]` button in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) renders conditionally before `[Unread]` only when `config.ai_enabled` is true.
  - Clicking triggers `GET /hx/entries?status=ai`, passing active search query input via `hx-include="[name='search']"`.
- **Feed Grouping**:
  - The latest 15 articles are fetched and grouped by feed source, rendering clean glass headers with article counts.
- **Lazy Viewport Summarization**:
  - Each item card embeds an `.ai-summary-slot` with `hx-trigger="revealed"` targeting `/hx/entries/{id}/ai-summary`.
  - Network and CLI workloads are naturally staggered by user scrolling rather than bombarding the server with 15 concurrent subprocess requests upon page load.
- **Assessment**: **Pass**. Clean reading eye flow and predictable client-server interaction.

### 3.2 Content Extraction (`<article>` Tag Parsing & Fallback)
- **Extraction Flow**:
  1. Validates source URL; if missing or empty, returns article title and RSS teaser immediately.
  2. Issues HTTP GET using `reqwest::Client` with a strict 3-second timeout and browser user-agent.
  3. If the remote server responds with non-200 (403 Forbidden, Cloudflare challenge, paywall) or network times out: immediately falls back to `title` + `miniflux_client::summary::extract_summary(teaser, 2)`.
  4. On HTTP 200: parses HTML with `scraper::Html` to locate `<article>` tag.
  5. Strips boilerplate elements (`<script>`, `<style>`, `<nav>`, `<header>`, `<footer>`, `<aside>`).
  6. If `<article>` tag is absent or yields less than 50 characters, falls back to `<body>` text or `teaser`.
  7. Collapses redundant whitespace and truncates content to title + first 500 words.
- **Assessment**: **Pass**. Fulfills the updated requirement to parse `<article>` tags while defending against remote scraping failures and blocking.

### 3.3 LLM CLI Runner & Concurrency Isolation
- **Subprocess Management**:
  - CLI commands execute via `tokio::process::Command` wrapped with `kill_on_drop(true)` to prevent orphaned zombie processes.
  - Overall CLI execution is bounded by `tokio::time::timeout(Duration::from_secs(5))`.
- **Concurrency Throttling**:
  - Guarded by an `Arc<tokio::sync::Semaphore>` capped at 2 concurrent permits.
  - Prevents system load spikes and CPU starvation when multiple browser cards scroll into view simultaneously.
- **Fast-Path Cache Bypass**:
  - Cache hits return instantly from SQLite within 2ms, bypassing the semaphore and leaving execution permits available for uncached items.
- **Fallback Execution**:
  - On non-zero exit code, CLI missing, or timeout, the runner falls back immediately to extractive sentence segmentation and labels the model as `"extractive-fallback"`.
- **Assessment**: **Pass**. Excellent fault containment and process safety.

### 3.4 Persistent SQLite Caching
- **Storage Strategy**:
  - Local database file configured via `AI_DB_PATH` (default: `data/summaries.db`).
  - Connection pool with 8 pooled connections managed by `r2d2`.
  - All database operations execute inside `tokio::task::spawn_blocking`.
- **Concurrency & Locking**:
  - WAL mode permits concurrent reads alongside active writes without mutual exclusion.
  - 5000ms busy timeout allows short lock queues during rapid upserts without erroring out.
- **Assessment**: **Pass**. Scalable local caching with robust multi-threading guarantees.

### 3.5 Security, Authentication & Multi-Tenant Isolation
- **Endpoint Protection**:
  - Both `/hx/entries?status=ai` and `/hx/entries/{id}/ai-summary` enforce `AuthenticatedUser` middleware.
  - Unauthenticated access redirects cleanly to `/login`.
- **Ownership Verification (IDOR Defense)**:
  - Before reading cached summaries or invoking the LLM, the handler verifies that entry `{id}` belongs to the authenticated user via Miniflux API/GraphQL.
  - Prevents unauthorized users from harvesting cached summaries or draining server LLM quotas.
- **Assessment**: **Pass**. Thorough multi-tenant protection and secure access control.

### 3.6 Error Resilience & UI Unsticking Guarantee
- **HTMX OuterHTML Contract**:
  - HTMX `revealed` triggers replace DOM elements using `hx-swap="outerHTML"`.
  - If a server error returns an error partial with `HX-Reswap: none`, the initial skeleton element remains frozen indefinitely.
  - The design ensures `/hx/entries/{id}/ai-summary` always returns HTTP 200 with a valid replacement card (extractive summary or graceful "Summary unavailable" message), ensuring the swap always completes.
- **Assessment**: **Pass**. Guarantees a polished user experience under all edge conditions.

---

## 4. Task Breakdown & TDD Readiness Audit (`04-tasks.md`)

The task breakdown in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261120-ai-news-summary/04-tasks.md) was reviewed for TDD compliance:

- **Planned Tests**: 16 dedicated unit and integration tests covering every component:
  - Configuration parsing (Test 1)
  - SQLite schema, WAL pooling, and upsert logic (Tests 2–4)
  - `<article>` tag extraction, boilerplate stripping, and fallback behaviors (Tests 5–8)
  - LLM prompt formatting, CLI timeouts, fallbacks, and concurrency limits (Tests 9–11)
  - Askama template rendering and filter button placement (Tests 12–13)
  - Axum routing, search forwarding, auth verification, and 200 outerHTML swap guarantees (Tests 14–16)
- **Task Granularity**: 6 focused tasks with strict file isolation and verifiable completion criteria.
- **Sequencing**:
  1. Task 1: Configuration & Environment Setup
  2. Task 2: SQLite Persistence Layer
  3. Task 3: Article Content Extractor & Fallback Engine
  4. Task 4: LLM CLI Runner & Concurrency Guard
  5. Task 5: UI Templates & Filter Bar
  6. Task 6: Server Routing & Endpoints
- **Assessment**: **Pass**. Every implementation step starts with a failing test before production code.

---

## 5. Directives for Implementation Phase

1. **Directive REV-01: Extractor Timeout Granularity**
   - In `ArticleExtractor`, use `reqwest::ClientBuilder::new().timeout(Duration::from_secs(3)).build()` so network delays never block worker threads.
2. **Directive REV-02: Strict Semaphore Scope**
   - Acquire semaphore permit *only* after checking SQLite cache. Release permit as soon as LLM execution completes or falls back.
3. **Directive REV-03: Graceful Swap Response**
   - For missing or unauthorized entries, return `<div class="ai-summary-card text-xs text-slate-500 italic mt-3 pt-3 border-t border-white/5">Summary unavailable</div>` with HTTP 200 to guarantee DOM replacement.

---

## 6. Review Summary & Sign-Off

- **Overall Result:** `clean`
- **Critical Issues:** 0
- **Major Issues:** 0
- **Minor Observations:** Addressed via directives
- **Ready for Gate B:** Yes. Architecture is cohesive, resilient, and TDD-ready.
