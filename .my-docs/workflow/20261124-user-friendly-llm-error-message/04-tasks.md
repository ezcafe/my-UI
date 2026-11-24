# Implementation Tasks: Non-Blocking LLM Error Toast Notification

## Planned Tests
1. **`test_llm_error_toast_constant_and_helper`**:
   - Location: `crates/server/src/error.rs`
   - Description: Verifies that `render_llm_error_toast()` returns an HTML fragment containing `id="toast-container"`, `hx-swap-oob="beforeend"`, `data-lucide="alert-circle"`, and the exact message string `"Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later."`.
2. **`test_get_entry_rewrite_htmx_toast_on_llm_failure`**:
   - Location: `crates/server/src/routes/entries.rs`
   - Description: Sends a `GET /hx/entries/{id}/rewrite` request when the LLM service fails. Asserts that the response status is 200 OK, the body contains the `#ai-rewrite-box` with graceful fallback content, and also contains the OOB `#toast-container` with the user-friendly high-demand warning.
3. **`test_get_entry_analysis_htmx_toast_on_llm_failure`**:
   - Location: `crates/server/src/routes/entries.rs`
   - Description: Sends a `GET /hx/entries/{id}/analysis` request when the LLM analysis fails. Asserts that the response status is 200 OK, the body contains `<div id="ai-analysis-box" class="hidden"></div>` (to remove loading skeleton), and also contains the OOB `#toast-container` with the user-friendly high-demand warning.
4. **`test_render_executive_brief_htmx_toast_on_failure`**:
   - Location: `crates/server/src/routes/entries.rs`
   - Description: Requests `GET /hx/entries/executive-brief` when batch LLM summarization fails. Asserts that the response status is 200 OK, the body contains `<div id="page-executive-brief" class="hidden"></div>`, and appends the OOB toast warning.
5. **`test_base_template_toast_auto_dismiss_and_icon_init`**:
   - Location: `crates/ui-components/src/templates.rs`
   - Description: Verifies that rendering `IndexTemplate` (which extends `base.html`) includes the client-side script listening to `htmx:oobAfterSwap` for `#toast-container` with 5000ms auto-dismiss and Lucide icon initialization.

---

## Task List

### Phase 1: Test Scaffolding (Red)
- [x] **Task 1.1**: Add unit test for `render_llm_error_toast` in `crates/server/src/error.rs`.
- [x] **Task 1.2**: Update and expand integration test `test_get_entry_rewrite_htmx_fallback_on_llm_failure` in `crates/server/src/routes/entries.rs` to verify OOB toast presence.
- [x] **Task 1.3**: Add integration test `test_get_entry_analysis_htmx_fallback_on_llm_failure` in `crates/server/src/routes/entries.rs` to assert fallback hidden box + OOB toast.
- [x] **Task 1.4**: Add test `test_render_executive_brief_htmx_toast_on_failure` in `crates/server/src/routes/entries.rs` for batch LLM failure handling.
- [x] **Task 1.5**: Add unit test `test_base_template_toast_auto_dismiss_and_icon_init` in `crates/ui-components/src/templates.rs`.

### Phase 2: Server-Side Implementation (Green)
- [x] **Task 2.1**: Define `LLM_HIGH_DEMAND_ERROR_MESSAGE` constant and `render_llm_error_toast()` helper in `crates/server/src/error.rs`.
- [x] **Task 2.2**: Update `get_entry_rewrite_htmx` in `crates/server/src/routes/entries.rs` to append `render_llm_error_toast()` when `generate_rewrite_and_analysis` returns `Err(_)`.
- [x] **Task 2.3**: Update `get_entry_analysis_htmx` in `crates/server/src/routes/entries.rs` to append `render_llm_error_toast()` when `analyze_entry_impact_and_fallacies` returns `Err(_)`.
- [x] **Task 2.4**: Update `render_executive_brief_htmx` in `crates/server/src/routes/entries.rs` to append `render_llm_error_toast()` when batch synthesis returns empty results on error.

### Phase 3: Client-Side Refinements & Testing (Refactor & Polish)
- [x] **Task 3.1**: In `crates/ui-components/templates/base.html`, add `htmx:oobAfterSwap` listener for `#toast-container` to initialize Lucide icons and attach a 5000ms auto-dismiss timeout.
- [x] **Task 3.2**: Execute cargo test suite (`cargo test --all`) to verify all unit and integration tests pass cleanly.
