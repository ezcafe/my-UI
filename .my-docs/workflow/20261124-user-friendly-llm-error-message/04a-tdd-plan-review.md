# TDD Plan Review

**Result:** clean

## Key Conclusions
- **All Core Failure Modes Covered**: The planned tests verify all 3 on-demand LLM endpoints (`get_entry_rewrite_htmx`, `get_entry_analysis_htmx`, `render_executive_brief_htmx`), the server-side toast helper/constant (`error.rs`), and the client-side lifecycle script (`base.html`).
- **Strict TDD Compliance**: Every planned test asserts the failure condition (Red) before implementation code is added (Green).
- **No Leaked Errors**: Tests explicitly assert that technical errors (stack traces, raw status codes, connection panics) are replaced by graceful fallback content and the exact user-facing notice string.

---

## Planned Test Cases Evaluation

### 1. Constant and Toast Rendering Helper
- **Test name**: `test_llm_error_toast_constant_and_helper`
- **Target file**: [`crates/server/src/error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs)
- **Scope**: Unit test
- **Verification goals**:
  - Constant `LLM_HIGH_DEMAND_ERROR_MESSAGE` matches: `"Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later."`
  - `render_llm_error_toast()` produces valid HTML containing `id="toast-container"`, `hx-swap-oob="beforeend"`, and `data-lucide="alert-circle"`.
- **Verdict**: **Approved**. Fast, deterministic unit test with zero external dependencies.

### 2. Entry Rewrite Fallback and OOB Toast
- **Test name**: `test_get_entry_rewrite_htmx_toast_on_llm_failure`
- **Target file**: [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L3177)
- **Scope**: Integration test
- **Verification goals**:
  - Request to `GET /hx/entries/{id}/rewrite` when LLM fails returns HTTP 200 OK.
  - Primary target `#ai-rewrite-box` contains graceful fallback text (cached summary or teaser).
  - Out-of-band `#toast-container` is appended with `hx-swap-oob="beforeend"`.
  - Toast body contains `LLM_HIGH_DEMAND_ERROR_MESSAGE`.
- **Verdict**: **Approved**. Expands existing test `test_get_entry_rewrite_htmx_fallback_on_llm_failure` to check OOB toast payload.

### 3. Entry Analysis Fallback and OOB Toast
- **Test name**: `test_get_entry_analysis_htmx_toast_on_llm_failure`
- **Target file**: [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1517)
- **Scope**: Integration test
- **Verification goals**:
  - Request to `GET /hx/entries/{id}/analysis` when LLM analysis fails returns HTTP 200 OK.
  - Primary target returns `<div id="ai-analysis-box" class="hidden"></div>` to cleanly remove the loading skeleton.
  - Appends OOB `#toast-container` with the user-friendly notification.
- **Verdict**: **Approved**. Ensures failure leaves no orphaned skeleton state.

### 4. Executive Brief Fallback and OOB Toast
- **Test name**: `test_render_executive_brief_htmx_toast_on_failure`
- **Target file**: [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L867)
- **Scope**: Integration test
- **Verification goals**:
  - Request to `GET /hx/entries/executive-brief` returns HTTP 200 OK when LLM synthesis yields empty items.
  - Returns `<div id="page-executive-brief" class="hidden"></div>` for the primary element.
  - Appends OOB `#toast-container` containing the user-friendly notification.
- **Verdict**: **Approved**. Covers multi-item page synthesis failure contract.

### 5. Client-Side Lifecycle & Auto-Dismiss Script
- **Test name**: `test_base_template_toast_auto_dismiss_and_icon_init`
- **Target file**: [`crates/ui-components/src/templates.rs`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
- **Scope**: Unit test
- **Verification goals**:
  - Rendering `IndexTemplate` (inheriting `base.html`) contains the JavaScript event listener for `htmx:oobAfterSwap`.
  - Listener checks for `#toast-container` swap, initializes Lucide icons (`createIcons`), and schedules auto-dismiss after 5000ms.
- **Verdict**: **Approved**. Validates that client-side script is present and structured without requiring a full headless browser.

---

## Verifiability & Reference Sources
- **Task Plan**: [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/04-tasks.md)
- **Design Specification**: [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261124-user-friendly-llm-error-message/03-design.md)
- **HTMX OOB Documentation**: [htmx.org/attributes/hx-swap-oob/](https://htmx.org/attributes/hx-swap-oob/)
- **Askama Toast Template**: [`ToastTemplate`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L124-L129)
