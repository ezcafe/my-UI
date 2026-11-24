Result: clean

# General Design Review: Executive Brief Duplication, Detail Caching, DB Reuse, and Prompt Rules

- **Workflow Slug:** `20261124-fix-brief-duplication-and-detail-caching`
- **Stage Id:** `design-review`
- **Mode:** full
- **Review Profile:** full
- **Isolated Reviews:**
  - `api-contract-review`: clean (`03a-api-contract-review.md`)
  - `db-design-review`: clean (`03a-db-design-review.md`)
- **Verdict:** `clean` (Zero Critical / Zero Major issues)

---

## 1. Executive Summary

This general design review evaluates the complete architectural proposal documented in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/03-design.md) and task breakdown in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/04-tasks.md).

The design covers four interconnected areas:
1. **Executive Brief Markup Single-Sourcing**: Eliminates duplicated out-of-band wire markup in `ExecutiveBriefCardTemplate`, using client-side `htmx:afterSwap` event synchronization to update list views.
2. **Instant Detail Title & Client Caching**: Resolves a fatal cache-invalidation string check in IndexedDB, and renders article titles and briefs instantly from the DOM/cache before network fetches begin.
3. **Database Reuse for Entry Analysis**: Short-circuits `/hx/entries/{id}/analysis` and `AiService` when pre-existing `impact` or `fallacies` records exist in the database, reducing latency from seconds to <1ms and avoiding unnecessary LLM costs.
4. **Prompt Optimization**: Enforces inverted pyramid structure, strict word counts (<110 words, 30–60 sweet spot), 2–3 bullets, and removes conversational opening/closing boilerplate.

---

## 2. Review Dimensions & Cross-Cutting Analysis

### 2.1 Architecture and Locality of Behavior
- **Decoupling Wire Payload from Dual-Column UI**: Returning only `#page-executive-brief` restores HTMX best practices by preventing duplicate DOM IDs and redundant HTML across the wire.
- **Client Synchronization**: Updating `#list-executive-brief` via standard DOM events inside `htmx:afterSwap` ensures both views stay synchronized without polluting server-side response partials.
- **Client Cache Lifecycle**: Fixing `cached && cached.html` restores the intended 30-day IndexedDB cache without requiring structural changes to the client storage schema.

### 2.2 Security, Data Privacy & Injection Defense
- **Client-Side Rendering Safety**: Titles extracted from the DOM during skeleton expansion must pass through `escapeHtml()` prior to template insertion to prevent XSS.
- **Server Database Parameterization**: Read-before-write checks utilize SQLx parameterized queries (`WHERE entry_id = $1`), preventing SQL injection.
- **Auth & PII**: No authentication, PII, or security schemas are modified.

### 2.3 Performance & User Experience
- **Elimination of UI Flicker**: Instant title rendering inside `openDetailWithSkeleton` provides immediate visual feedback, eliminating layout shifts and blank skeleton states.
- **Zero LLM Latency on Cached Reads**: Checking `ai_service.db.get_summary_and_title(id)` before dispatching background tasks or LLM calls reduces repeated view latencies by orders of magnitude.
- **Reduced Payload Size**: Eliminating redundant OOB containers trims wire payloads on paginated feed navigations.

### 2.4 Testability and Implementation Readiness
- **Comprehensive Test Coverage**: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/04-tasks.md) specifies unit and integration tests across both `crates/ui-components` and `crates/server`.
- **TDD Sequencing**: All tasks are structured to write failing unit tests before implementing changes in templates, scripts, routes, and prompt builders.

---

## 3. Findings & Observations

| Severity | Area | Description | Resolution / Status |
|---|---|---|---|
| Info | Frontend | Ensure `escapeHtml()` is used when injecting extracted title into detail skeleton HTML | Documented in design and handled in `openDetailWithSkeleton` |
| Info | Route Handler | Support partial analysis records (where only `impact` or only `fallacies` is non-empty) | Verified in API and DB reviews: `render_analysis_cards_fragment` handles both |

**Summary**: Zero Critical, zero Major, and zero blocking findings.

---

## 4. Final Verdict

- **Result:** `clean`
- **Next Stage:** `tdd-review` (or parent Gate B / tdd-review per workflow rules)
- **Gate B Readiness:** All design artifacts (`03-design.md`, `03a-api-contract-review.md`, `03a-db-design-review.md`, `03a-design-review-log.md`, `04-tasks.md`) are complete, validated, and clean.