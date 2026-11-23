# Test Log: Unread Filter, Brief Infinite Loading, Title Sentence Restriction, and Brief Throttle Fixes

- **Run slug:** `20261123-unread-filter-brief-title-fixes`
- **Updated:** 2026-11-23T08:59:30+07:00
- **Test path:** lite
- **Result:** success
- **Smoke source:** Smoke-Build

---

## 1. Test Suite Verification Summary

| Target / Crate | Tests Run | Passed | Failed | Status |
|----------------|-----------|--------|--------|--------|
| `miniflux-client` (unit) | 23 | 23 | 0 | PASSED |
| `server` (unit) | 104 | 104 | 0 | PASSED |
| `server` (integration / e2e) | 6 | 6 | 0 | PASSED |
| `ui-components` (unit & templates) | 52 | 52 | 0 | PASSED |
| **Total** | **185** | **185** | **0** | **ALL PASSED** |

---

## 2. Static Analysis & Lints

- **Command:** `cargo clippy --workspace --all-targets`
- **Exit Code:** 0
- **Warnings:** 0
- **Status:** PASSED (0 warnings)

---

## 3. Milestone Test Details

### Milestone 1: Title Sentence Restriction in LLM Batch Prompt
- Verified batch prompt directive explicitly instructs `strictly in one sentence` instead of `<=10 words`.
- Verified batch prompt rejects truncation of descriptive sentence clauses across multiple languages.
- Tests:
  - `ai::llm::tests::test_format_batch_prompt_suggested_title_one_sentence`

### Milestone 2: Unread Filter Header Count Reconciliation & Client Sync
- Verified server sets unread counter to total count when upstream reports 0 but entries exist.
- Verified query deserialization for `request_brief` query parameter (`true`, `false`, and default `None`).
- Verified client script in `index.html` computes `effectiveUnread = visibleUnreadRowCount` when server count is 0 and unread filter is active.
- Tests:
  - `routes::entries::tests::test_unread_counter_fallback_to_total_when_zero`
  - `routes::entries::tests::test_entries_filter_query_request_brief_parsing`
  - `templates::tests::test_index_template_brief_caching_and_throttling_scripts`

### Milestone 3: Executive Brief Throttling & Pre-existing Brief Rendering
- Verified server serves existing brief (`top_executive_brief`) when `request_brief` is false instead of regenerating.
- Verified client script defines `readSinceLastBriefSet`, sets `request_brief` based on threshold (>= 10 read articles), and clears set after refresh.
- Verified `recordReadArticle` updates tracking on article read events.
- Tests:
  - `routes::entries::tests::test_render_entries_list_throttled_brief_serves_existing`
  - `templates::tests::test_index_template_brief_caching_and_throttling_scripts`

### Milestone 4: Skeleton Re-render & HTMX Processing Fixes
- Verified brief swaps update the filter cache with the full HTML representation.
- Verified cached list restoration calls `htmx.process(entriesList)` so subsequent swaps and triggers work reliably.
- Verified e2e integration flow tests pass without regression.
- Tests:
  - `templates::tests::test_index_template_brief_caching_and_throttling_scripts`
  - `tests/e2e_tests.rs::test_executive_brief_cache_hit_and_oob_swaps`
  - `tests/e2e_tests.rs::test_e2e_authenticated_session_flow`
  - `tests/e2e_tests.rs::test_e2e_guest_mode_flow`
