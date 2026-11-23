Result: clean

# TDD Plan Review: Unread Count, Brief Infinite Loading, Title Sentence Restriction, and Brief Throttle Fixes

## Key Takeaways
- **Test coverage is complete**: Planned tests cover all core logic changes across server routing, AI prompt generation, and template rendering.
- **Fail-first verified**: Each planned test targets specific functional logic that does not yet exist or fails under the current implementation.
- **TDD compliance**: Tests will be committed in red phase before implementation in the Build stage.

---

## Planned Tests Review

### 1. AI Prompt Sentence Constraint
- **Target test**: `test_format_batch_prompt_suggested_title_one_sentence` in `crates/server/src/ai/llm.rs`
- **Scope**: Asserts that `format_batch_prompt_structured` contains `"strictly in one sentence"` for suggested titles and eliminates `"<=10 words"`.
- **Verdict**: **Clean**. Direct string assertion that enforces the exact prompt requirement without needing network or LLM execution.

### 2. Query Deserialization for Brief Throttling
- **Target test**: `test_entries_filter_query_request_brief_parsing` in `crates/server/src/routes/entries.rs`
- **Scope**: Tests query string parsing of `request_brief` for `None`, `Some(true)`, and `Some(false)`.
- **Verdict**: **Clean**. Covers all edge cases for query parameter handling in Axum.

### 3. Unread Count Fallback
- **Target test**: `test_unread_count_fallback_to_total_in_entries` in `crates/server/src/routes/entries.rs`
- **Scope**: Verifies that when unread counter is 0 or null and `total > 0`, the handler falls back to `total` as `unread_count`.
- **Verdict**: **Clean**. Addresses the root cause of the unread count mismatch bug.

### 4. Template Rendering for Static Top Brief
- **Target test**: `test_render_entry_list_with_static_top_executive_brief` in `crates/ui-components/src/templates.rs`
- **Scope**: Verifies that `EntryListTemplate` renders the static brief card without the loading skeleton or `hx-trigger="load"` when `is_brief_eligible` is false and `top_executive_brief` is present.
- **Verdict**: **Clean**. Ensures client-side throttle logic receives the proper HTML structure without triggering an infinite brief load loop.

---

## Observations & Recommendations for Build
- **Client-side JS verification**: Client logic in `index.html` (HTMX `configRequest` interceptor, `htmx.process(entriesList)`, and `readSinceLastBriefSet`) is verified via template syntax and integration tests.
- **Cargo test execution**: All 4 tests are unit tests and can run quickly in the sandbox via standard `cargo test`.
