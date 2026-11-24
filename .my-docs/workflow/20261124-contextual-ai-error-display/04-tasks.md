# Tasks: Contextual AI Error Display & Inline Recovery

## Planned tests

1. **`test_ai_error_card_template_rendering`**
   - **File**: `crates/ui-components/src/templates.rs`
   - **Type**: Unit test
   - **Description**: Verifies that `AiErrorCardTemplate` renders with the specified `container_id`, `title`, `message`, `retry_url`, `target`, `swap`, and includes the retry button with `hx-get`, `hx-target`, `hx-swap`, and `data-lucide="refresh-cw"`.

2. **`test_executive_brief_htmx_contextual_error_card_on_failure`**
   - **File**: `crates/server/src/routes/entries.rs`
   - **Type**: Integration test
   - **Description**: Verifies that when batch synthesis returns empty results, `executive_brief_htmx` returns HTTP 200 with an HTML body containing `id="page-executive-brief"`, class `is-error`, `hx-get="/hx/executive-brief?..."`, a Retry button, and NO `#toast-container` out-of-band element.

3. **`test_get_entry_analysis_htmx_contextual_error_card_on_failure`**
   - **File**: `crates/server/src/routes/entries.rs`
   - **Type**: Integration test
   - **Description**: Verifies that when `analyze_entry_impact_and_fallacies` returns an error, `get_entry_analysis_htmx` returns HTTP 200 with an HTML body containing `id="ai-analysis-box"`, class `is-error`, `hx-get="/hx/entries/{id}/analysis"`, a Retry button, and NO `#toast-container` out-of-band element.

4. **`test_get_entry_rewrite_htmx_contextual_error_card_on_failure`**
   - **File**: `crates/server/src/routes/entries.rs`
   - **Type**: Integration test
   - **Description**: Verifies that when `generate_rewrite_and_analysis` returns an error, `get_entry_rewrite_htmx` returns HTTP 200 containing `id="ai-rewrite-box"`, an error card with Retry button (`hx-get="/hx/entries/{id}/rewrite"`), the readable fallback article text, and NO `#toast-container` element.

5. **`test_executive_brief_retry_url_preserves_query_filters`**
   - **File**: `crates/server/src/routes/entries.rs`
   - **Type**: Unit / Integration test
   - **Description**: Verifies that when `executive_brief_htmx` is called with custom filters (e.g. `status=unread`, `feed_id=42`, `category_id=7`), the returned error card's retry button contains the exact query string `offset=20&limit=10&status=unread&feed_id=42&category_id=7`.

---

## Implementation Tasks

### Phase 1: Test Scaffolding (TDD Red)
- [x] **Task 1.1**: Add `test_ai_error_card_template_rendering` in `crates/ui-components/src/templates.rs`.
- [x] **Task 1.2**: Update / add `test_executive_brief_htmx_contextual_error_card_on_failure` in `crates/server/src/routes/entries.rs`.
- [x] **Task 1.3**: Update / add `test_get_entry_analysis_htmx_contextual_error_card_on_failure` in `crates/server/src/routes/entries.rs`.
- [x] **Task 1.4**: Update / add `test_get_entry_rewrite_htmx_contextual_error_card_on_failure` in `crates/server/src/routes/entries.rs`.
- [x] **Task 1.5**: Add `test_executive_brief_retry_url_preserves_query_filters` in `crates/server/src/routes/entries.rs`.

### Phase 2: Template Implementation (TDD Green)
- [x] **Task 2.1**: Create `crates/ui-components/templates/partials/ai_error_card.html` with responsive container styling, alert badge, message, and HTMX retry button.
- [x] **Task 2.2**: Define `AiErrorCardTemplate` struct in `crates/ui-components/src/templates.rs` and export it in `crates/ui-components/src/lib.rs`.

### Phase 3: Route Handlers Implementation
- [x] **Task 3.1**: Update `executive_brief_htmx` in `crates/server/src/routes/entries.rs` to render `AiErrorCardTemplate` with preserved query parameters on batch failure, removing `render_llm_error_toast()`.
- [x] **Task 3.2**: Update `get_entry_analysis_htmx` in `crates/server/src/routes/entries.rs` to render `AiErrorCardTemplate` for `#ai-analysis-box` on failure, removing `render_llm_error_toast()`.
- [x] **Task 3.3**: Update `get_entry_rewrite_htmx` in `crates/server/src/routes/entries.rs` to render `AiErrorCardTemplate` inside `#ai-rewrite-box` followed by fallback article content, removing `render_llm_error_toast()`.

### Phase 4: Client-Side Dual-View Synchronization
- [x] **Task 4.1**: Update `crates/ui-components/templates/pages/index.html` `htmx:afterSwap` brief sync handler to detect error card state on `#page-executive-brief`, remove `animate-pulse`, render matching compact error card with retry button into `#list-executive-brief`, and invoke `htmx.process` and `lucide.createIcons`.

### Phase 5: Verification & Regression Testing
- [x] **Task 5.1**: Run `cargo test -p ui-components` to verify all template rendering tests pass.
- [x] **Task 5.2**: Run `cargo test -p server` to verify all route handler failure and success tests pass.
- [x] **Task 5.3**: Run `cargo clippy` and `cargo fmt --check` to ensure zero warnings or formatting issues.
