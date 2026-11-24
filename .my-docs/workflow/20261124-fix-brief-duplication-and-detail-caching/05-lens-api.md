# Lens: api

**Result:** clean

---

## 1. Executive Summary

This review assesses public and internal API contracts modified for the workflow `20261124-fix-brief-duplication-and-detail-caching`. All API modifications match [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/03-design.md) and adhere to REST and HTMX contract best practices:

- **Clean wire payloads**: Removed duplicate Out-of-Band (OOB) container `#list-executive-brief` from `executive_brief_card.html`, avoiding duplicate DOM IDs. Client updates `#list-executive-brief` via `htmx:afterSwap`.
- **Zero-overhead database caching**: `GET /hx/entries/{id}/analysis` and `AiService` serve stored `impact` and `fallacies` directly from SQLite/PostgreSQL, eliminating redundant upstream Miniflux queries and LLM invocations.
- **Stable status codes and error shapes**: HTMX partials return standard status codes (`200 OK`) with graceful hidden fallback fragments on failure, preventing UI layout breaks or leakage of internal errors.
- **Prompt contract compliance**: Updated LLM prompts enforce inverted pyramid structure (<110 words, 30–60 words sweet spot, 2–3 bullets) with unit test verification.

---

## 2. API Contract Checklist

| Category | Requirement | Status | Evidence / Verification |
| :--- | :--- | :--- | :--- |
| **Typing** | Typed input and output | Pass | `Path<i64>` for entry ID, `Query<EntriesFilterParams>` for query parameters; strongly-typed handlers returning `Html<String>` / `Response`. |
| **Error Handling** | Stable error shape; correct status codes | Pass | Safe hidden fallback fragments (`<div id="ai-analysis-box" class="hidden"></div>`) on error; no leaked internal traces or connection strings. |
| **Validation** | Validate at system edges only | Pass | System edge extractors validate parameters (e.g. numeric ID conversion); internal domain services receive strongly typed arguments. |
| **HTTP Semantics** | Proper methods & query filtering | Pass | `GET` endpoints for read operations; filters passed via query strings; read operations are strictly idempotent. |
| **Compatibility** | Additive fields; no silent breaks | Pass | Wire contract for `/hx/entries/{id}/analysis` remains identical; `/hx/executive-brief` payload cleanup coordinated with `index.html` listener. |
| **Naming** | Consistent naming conventions | Pass | Adheres to `/hx/...` prefix for HTMX partial routes and REST path conventions (`/hx/entries/{id}/analysis`). |
| **Idempotency** | Safe and idempotent GET requests | Pass | All touched endpoints are `GET` endpoints with no destructive mutations on repeat calls. |
| **Design Fidelity** | Code matches design contracts | Pass | Implementation faithfully mirrors [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/03-design.md) (Areas 1, 3, and 4). |
| **Security** | No internal error leaks | Pass | Upstream errors logged via `tracing::error!` / `tracing::warn!`; client receives sanitized UI fallback. |

---

## 3. Detailed Endpoint Reviews

### 3.1 `GET /hx/executive-brief`
- **Route**: `crates/server/src/routes/entries.rs:get_executive_brief_htmx`
- **Template**: `crates/ui-components/templates/partials/executive_brief_card.html`
- **Wire Contract**:
  - Previously returned `#page-executive-brief` and `#list-executive-brief` in the same response, causing duplicate DOM IDs in magazine and dual-column layouts.
  - Now returns a single `#page-executive-brief` card container.
  - The client `index.html` `htmx:afterSwap` event listener detects the swap, syncs content to `#list-executive-brief`, and removes the pulse loading skeleton.
- **Contract Assessment**: Clean. Payload size is reduced and DOM uniqueness is restored.

### 3.2 `GET /hx/entries/{id}/analysis`
- **Route**: `crates/server/src/routes/entries.rs:get_entry_analysis_htmx`
- **Wire Contract**:
  - Success returns `<div id="ai-analysis-box" class="space-y-6 mt-6">...</div>` with impact and fallacies cards.
  - Error or disabled state returns `<div id="ai-analysis-box" class="hidden"></div>`.
- **Database Reuse**:
  - Evaluates `ai_service.db.get_summary_and_title(id).await`.
  - If either `impact` or `fallacies` is present and non-empty, calls `render_analysis_cards_fragment(imp, fal)` and returns immediately.
  - Bypasses external network calls to Miniflux and local/remote LLM calls.
- **Contract Assessment**: Clean. Zero breaking changes to the wire response while optimizing throughput and response latency.

### 3.3 Internal LLM Prompt Contracts
- **Source**: `crates/server/src/ai/llm.rs`
- **Prompt Rules**:
  - Batch prompt: Enforces rewrite `r` strictly under 110 words (30–60 sweet spot), inverted pyramid structure, outcome in first 2 sentences, 2–3 bullet points, and no filler/boilerplate.
  - Single detail prompt: Matches the same constraints within JSON response schema.
  - Unit tests verify prompt structure: `test_batch_prompt_rewrite_rules_word_count_and_inverted_pyramid` and `test_rewrite_and_analyze_prompt_rules`.
- **Contract Assessment**: Clean. Validated by automated unit tests.

---

## 4. References & Documentation Sources

- Axum Error Handling: <https://docs.rs/axum/latest/axum/response/trait.IntoResponse.html>
- HTMX Out-of-Band Swaps: <https://htmx.org/attributes/hx-swap-oob/>
- Technical Design Document: [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/03-design.md)
- API Contract Review Spec: [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-fix-brief-duplication-and-detail-caching/03a-api-contract-review.md)
- Route implementation: [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
