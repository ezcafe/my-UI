# TDD Plan Review: Contextual AI Error Display & Inline Recovery

## Result: clean

## Executive Summary
This review evaluates the Test-Driven Development (TDD) plan defined in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-contextual-ai-error-display/04-tasks.md) for contextual AI error display and inline recovery. The planned suite includes 1 unit test and 4 integration/route tests covering template rendering, failure responses across all three hypermedia endpoints, query parameter preservation, and toast removal. All planned test cases are sound, verifiable, and adhere strictly to TDD red-to-green flow. Result is marked **clean**.

## Key Conclusions
- **Comprehensive Failure Path Coverage**: All three HTMX endpoints (`/hx/executive-brief`, `/hx/entries/{id}/analysis`, `/hx/entries/{id}/rewrite`) have dedicated failure-mode tests asserting HTTP 200, container IDs, retry hypermedia attributes, and toast suppression.
- **Strict TDD Scaffolding**: Phase 1 enforces red tests before implementing `AiErrorCardTemplate` or modifying route handlers.
- **Defensive State Preservation**: Dedicated unit/integration tests verify that complex query filter combinations (`status`, `feed_id`, `category_id`) are preserved intact in the retry URL.
- **Zero Ambiguity in Assertions**: Tests assert both positive expectations (presence of `.is-error`, `hx-get`, `hx-target`, `hx-swap`, retry buttons) and negative invariants (absence of `#toast-container` OOB markup).

---

## Evaluation of Planned Test Cases

### 1. `test_ai_error_card_template_rendering`
- **Target File**: `crates/ui-components/src/templates.rs`
- **Scope**: Unit test
- **Verification Criteria**:
  - Validates `AiErrorCardTemplate` Askama rendering.
  - Verifies presence of header alert icon (`alert-circle`), error message text, and retry button (`refresh-cw`).
  - Verifies HTMX attributes: `hx-get`, `hx-target`, `hx-swap`, `hx-disabled-elt="this"`.
  - Verifies container ID logic: when `container_id` is non-empty (`"page-executive-brief"`), output includes `id="page-executive-brief"`; when `container_id` is empty (`""`), no malformed empty `id=""` attribute is emitted.
- **Assessment**: **Pass (Sound)**. Covers both component variations (standalone card vs embedded card).

### 2. `test_executive_brief_htmx_contextual_error_card_on_failure`
- **Target File**: `crates/server/src/routes/entries.rs`
- **Scope**: Route integration test via `tower::ServiceExt::oneshot`
- **Verification Criteria**:
  - Simulates batch AI synthesis failure (empty batch items/top).
  - Asserts HTTP 200 OK (enabling HTMX to swap DOM content).
  - Asserts response body contains `<div id="page-executive-brief" class="... is-error ...">`.
  - Asserts response body contains retry button pointing to `/hx/executive-brief?...`.
  - Asserts response body does **not** contain `<div id="toast-container" hx-swap-oob="beforeend">`.
- **Assessment**: **Pass (Sound)**. Guarantees contextual error replacement and toast elimination.

### 3. `test_get_entry_analysis_htmx_contextual_error_card_on_failure`
- **Target File**: `crates/server/src/routes/entries.rs`
- **Scope**: Route integration test via `tower::ServiceExt::oneshot`
- **Verification Criteria**:
  - Simulates `analyze_entry_impact_and_fallacies` returning an error.
  - Asserts HTTP 200 OK.
  - Asserts response body contains `id="ai-analysis-box"`, `.is-error`, `hx-get="/hx/entries/{id}/analysis"`, and `hx-target="#ai-analysis-box"`.
  - Asserts absence of `#toast-container`.
- **Assessment**: **Pass (Sound)**. Validates article analysis error handling.

### 4. `test_get_entry_rewrite_htmx_contextual_error_card_on_failure`
- **Target File**: `crates/server/src/routes/entries.rs`
- **Scope**: Route integration test via `tower::ServiceExt::oneshot`
- **Verification Criteria**:
  - Simulates `generate_rewrite_and_analysis` returning an error.
  - Asserts HTTP 200 OK.
  - Asserts response contains `id="ai-rewrite-box"` containing the error card with retry button (`hx-get="/hx/entries/{id}/rewrite"`).
  - Asserts response preserves readable fallback article text in `prose` styling so user reading is uninterrupted.
  - Asserts absence of `#toast-container`.
- **Assessment**: **Pass (Sound)**. Validates dual requirement: error visibility + reading continuity.

### 5. `test_executive_brief_retry_url_preserves_query_filters`
- **Target File**: `crates/server/src/routes/entries.rs`
- **Scope**: Unit / Route test
- **Verification Criteria**:
  - Sends request to `/hx/executive-brief?offset=20&limit=10&status=today&feed_id=42&category_id=7`.
  - Asserts retry URL in response body retains `offset=20`, `limit=10`, `status=today`, `feed_id=42`, and `category_id=7`.
- **Assessment**: **Pass (Sound)**. Prevents regression where retrying resets view to default page 1 unread items.

---

## TDD Phase Alignment

| Phase | Tasks | TDD State | Verification Target |
|-------|-------|-----------|---------------------|
| **Phase 1** | Tasks 1.1 – 1.5 | **RED** | Tests written and fail cleanly against current codebase (or fail compilation until template/route signatures exist). |
| **Phase 2** | Tasks 2.1 – 2.2 | **GREEN** (Unit) | `AiErrorCardTemplate` implemented; Task 1.1 passes with `cargo test -p ui-components`. |
| **Phase 3** | Tasks 3.1 – 3.3 | **GREEN** (Integration) | Handlers return error cards; Tasks 1.2 – 1.5 pass with `cargo test -p server`. |
| **Phase 4** | Task 4.1 | **GREEN** (E2E / Client) | Client-side dual-view sync script handles error state without continuous skeleton pulsing. |
| **Phase 5** | Tasks 5.1 – 5.3 | **VERIFY** | Full test suite clean, zero Clippy warnings, zero formatting violations. |

---

## Edge Cases and Defensive Considerations
- **Status Code 200 for HTMX Swaps**: By default, HTMX does not swap responses with HTTP 4xx/5xx status codes unless `hx-ext="response-targets"` is active. Returning HTTP 200 with the error card ensures reliable outerHTML DOM replacement.
- **OOB Toast Isolation**: In tests, asserting `!body.contains("toast-container")` ensures no zombie toasts are accidentally dispatched alongside inline error cards.
- **Double-Submit Prevention**: Asserting `hx-disabled-elt="this"` ensures the retry button is disabled during active in-flight requests, preventing concurrent duplicate LLM calls.

---

## Verifiability & Reference Sources
- **Askama Unit Testing Docs**: [docs.rs/askama/latest/askama/#unit-testing](https://docs.rs/askama)
- **Axum Integration Testing with Tower**: [docs.rs/axum/latest/axum/#testing](https://docs.rs/axum)
- **HTMX Swapping Semantics**: [htmx.org/docs/#swapping](https://htmx.org/docs/#swapping)
- **Primary Design Reference**: [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-contextual-ai-error-display/03-design.md)
- **Task List & Test Plan**: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-contextual-ai-error-display/04-tasks.md)
