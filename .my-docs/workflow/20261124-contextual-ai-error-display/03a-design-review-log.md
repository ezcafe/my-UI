# Design Review Log: Contextual AI Error Display & Inline Recovery

## Result: clean

## Executive Summary
This design review verifies the complete architectural specification for contextual AI error display and inline recovery across `03-design.md`, `04-tasks.md`, and `03a-api-contract-review.md`. The design cleanly decouples UI representation into Askama templates (`crates/ui-components`), simplifies route error handling in `crates/server`, and synchronizes client views via HTMX lifecycle events. All required isolated reviews (`03a-api-contract-review.md`) have passed. Zero Critical or Major architecture flaws exist.

## Key Conclusions
- **Clean Architectural Separation**: `AiErrorCardTemplate` is encapsulated in `crates/ui-components`, ensuring presentation logic remains compile-time verified and DRY across all three AI endpoints.
- **Robust Failure Resilience**: Instead of unhandled UI states or detached toast notifications that fail silently, errors render contextually in the DOM with a clear call-to-action (`Retry`), preventing layout shifting and infinite skeleton pulsing.
- **Strict TDD Pipeline**: The 5-stage task plan defines failing test cases (unit and integration) before any template or handler changes are introduced.
- **Reading Continuity Guaranteed**: Article reading is never interrupted when rewrite generation fails; users can continue reading the fallback article summary immediately while deciding whether to retry the rewrite.

---

## Architecture Review Across Core Dimensions

### 1. Component Boundaries & Modularity
- **Assessment**: Pass.
- `crates/ui-components`: Houses `AiErrorCardTemplate` and `partials/ai_error_card.html`. It has zero dependencies on `crates/server` or database models.
- `crates/server`: Invokes the template in `routes/entries.rs`. Replaces ad-hoc string formatting with strongly-typed Askama structs.
- `templates/pages/index.html`: Manages client-side dual-view synchronization within the `htmx:afterSwap` event listener, keeping server-side templates decoupled from dual-view DOM nuances.

### 2. State & Recovery Semantics
- **Assessment**: Pass.
- **Query Parameter Continuity**: The executive brief retry link dynamically builds the full filter query string (`status`, `limit`, `offset`, `feed_id`, `category_id`), ensuring a retry respects active user filters.
- **Idempotency**: All retry triggers perform HTTP GET requests without mutating server state or creating duplicate records.
- **Debounce / Spam Prevention**: The Retry button applies `hx-disabled-elt="this"` to prevent race conditions from repeated clicking during in-flight requests.

### 3. Error Handling & Security Boundary
- **Assessment**: Pass.
- **Zero Technical Leakage**: Internal error details (such as OpenAI/Ollama HTTP 5xx responses, CLI timeout exits, or network dropouts) are intercepted and translated into the user-friendly constant `LLM_HIGH_DEMAND_ERROR_MESSAGE`.
- **Auto-Escaped Markup**: Askama automatically escapes dynamic text fields (`title`, `message`, `retry_url`), eliminating potential XSS vectors.

### 4. Client-Side Lifecycle & Visual UX
- **Assessment**: Pass.
- Dual-view synchronization prevents `#list-executive-brief` from hanging with an infinite pulse animation on failure.
- Container queries and responsive styling (`rounded-lg p-5 bg-muted/40 border border-border`) align with the existing Tailwind theme tokens.
- Icon re-hydration (`lucide.createIcons`) is explicitly handled in the client swap callback.

---

## Review Checklist & Gate Status

| Review Axis | Findings | Status |
|---|---|---|
| **API Contract** | Verified in `03a-api-contract-review.md`. Zero Critical/Major issues. | **Pass** |
| **Database Design** | Has DB = no (no schema migrations or DB changes needed). | **Skipped** |
| **Cross-Cutting Architecture** | Clear separation between template rendering, route logic, and client DOM sync. | **Pass** |
| **TDD Task Breakdown** | 5 comprehensive unit/integration test cases sequenced in Phase 1 before implementation. | **Pass** |
| **User Global Rules** | Plain language, eye-flow structure, verifiability sources provided, no `.env` access. | **Pass** |

---

## Verifiability & Reference Sources
- **HTMX Documentation**: [htmx.org/docs/](https://htmx.org/docs/) · [htmx.org/api/#process](https://htmx.org/api/#process)
- **Askama Documentation**: [docs.rs/askama](https://docs.rs/askama)
- **Design Artifacts**:
  - [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-contextual-ai-error-display/03-design.md)
  - [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-contextual-ai-error-display/04-tasks.md)
  - [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-contextual-ai-error-display/03a-api-contract-review.md)
