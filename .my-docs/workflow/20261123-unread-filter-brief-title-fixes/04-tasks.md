# Tasks: Unread Count, Brief Infinite Loading, Title Sentence Restriction, and Brief Throttle Fixes

## Planned Tests

- [x] `test_format_batch_prompt_suggested_title_one_sentence`: Verifies that `format_batch_prompt_structured` in `llm.rs` mandates "strictly in one sentence" for suggested titles (`"t"`) and does not contain `<=10 words`.
- [x] `test_entries_filter_query_request_brief_parsing`: Verifies that `EntriesFilterQuery` correctly deserializes `request_brief` from query strings as `Some(true)`, `Some(false)`, and `None`.
- [x] `test_unread_count_fallback_to_total_in_entries`: Verifies that when unread counter is 0 or null and `total > 0`, the route handler falls back to `total` as `unread_count`.
- [x] `test_render_entry_list_with_static_top_executive_brief`: Verifies that `EntryListTemplate` renders the static brief card without the loading skeleton or `hx-trigger="load"` when `is_brief_eligible` is false and `top_executive_brief` is present.

---

## Milestone 1: LLM Prompt Suggested Title Sentence Constraint

- [x] **Task 1.1: TDD test for title one-sentence constraint in prompt**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Add `test_format_batch_prompt_suggested_title_one_sentence` asserting the presence of `strictly in one sentence` and absence of `<=10 words`.
- [x] **Task 1.2: Update prompt directive in `format_batch_prompt_structured`**
  - Path: `crates/server/src/ai/llm.rs`
  - Action: Update line 78 to instruct: `provide a punchy "t" (suggested title, strictly in one sentence) and "b" (executive brief, strictly 3 sentences, what changed/why it matters).`
  - Verification: Run `cargo test --package server ai::llm`.

---

## Milestone 2: Server-Side Unread Count Fallback & Brief Request Throttling

- [x] **Task 2.1: TDD tests for query parsing and count fallback**
  - Path: `crates/server/src/routes/entries.rs`
  - Action: Add `test_entries_filter_query_request_brief_parsing` and `test_unread_count_fallback_to_total_in_entries`.
- [x] **Task 2.2: Add `request_brief` field to `EntriesFilterQuery` and update eligibility logic**
  - Path: `crates/server/src/routes/entries.rs`
  - Action:
    - Add `pub request_brief: Option<bool>` to `EntriesFilterQuery`.
    - In `render_entries_list`:
      - Fall back `unread_count` to `Some(total as i32)` if counter is 0/null and `total > 0` on `status=unread`.
      - Check `filter_eligible = is_brief_eligible_filter(...)`.
      - Check `request_brief = query.request_brief.unwrap_or(true)`.
      - `is_eligible = filter_eligible && request_brief`.
      - If `filter_eligible && !is_eligible`, load `top_executive_brief` from `ai.db.get_latest_page_brief().await`.
      - Pass `top_executive_brief` to `EntryListTemplate`.
  - Verification: Run `cargo test --package server routes::entries`.
- [x] **Task 2.3: Update unread count fallback in `render_index_page`**
  - Path: `crates/server/src/routes/pages.rs`
  - Action: In `render_index_page`, if `unread_count == 0` and `total > 0`, fall back to `total as i32`.
  - Verification: Run `cargo test --package server routes::pages`.

---

## Milestone 3: Template & Client-Side Cache, HTMX Processing & Counter Tracking

- [x] **Task 3.1: TDD test for template rendering with static top brief**
  - Path: `crates/ui-components/src/templates.rs`
  - Action: Add `test_render_entry_list_with_static_top_executive_brief` verifying static brief output when `is_brief_eligible` is false.
  - Verification: Run `cargo test --package ui-components`.
- [x] **Task 3.2: Update `index.html` client logic**
  - Path: `crates/ui-components/templates/pages/index.html`
  - Action:
    - In `syncDomReadStates()`, add fallback to `visibleUnreadRowCount` if `effectiveUnread === 0` but `visibleUnreadRowCount > 0` on Unread filter.
    - In `applyCachedList()`, add `if (window.htmx && typeof htmx.process === 'function') htmx.process(entriesList);`.
    - In `htmx:afterSwap`, intercept swaps on `#page-executive-brief` or descendants of `#entries-list`, and refresh `filterCache` for active filter.
    - Add `readSinceLastBriefSet = new Set()` and `recordReadArticle(id)` helper called on `markArticleRead`, `markLocallyRead`, and detail view swaps.
    - In `htmx:configRequest`, check clicks on `#filter-unread`: if `readSinceLastBriefSet.size >= 10`, send `request_brief=true` and clear set; else send `request_brief=false`.
  - Verification: Inspect template markup and ensure all JS syntax checks pass.

---

## Milestone 4: Workspace Verification

- [x] **Task 4.1: Run full test suite and clippy**
  - Action: Run `cargo test --workspace` and `cargo clippy --workspace --all-targets`.

## Result

Result: done
