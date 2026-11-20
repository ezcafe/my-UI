Result: clean

# Design Review Log: Article Filters, Hybrid Search, Page Mark-Read & Detail Actions

- **Stage:** `design-verify-phase`
- **Mode:** simple | **Profile:** lite
- **Surfaces:** UI (yes), API (yes), DB (no), auth/PII/schema (no)
- **Status:** Paid (`clean`)

---

## 1. Review Summary

The design artifacts (`01-idea.md`, `02-analysis.md`, `02b-grill.md`, `03-design.md`, and `04-tasks.md`) have been verified against architecture standards, UI glassmorphism design system (`ar-list.html` / `ar-detail.html`), interface contracts, and TDD specifications.

- **API Contract Review (`03a-api-contract-review.md`)**: Paid (`clean`). Batch entry status update aligns with Miniflux REST API `PUT /v1/entries`; GraphQL mutation `updateEntriesStatus` cleanly extends `Mutation`; Axum routes preserve status context and escape search strings safely.
- **Database Surface**: Skipped (`Has DB = no`).
- **UI Surface**: Verified compliant with Spatial glassmorphism styling, clean button hierarchy, instant client-side DOM search filter, and accurate out-of-band unread count badge synchronization.
- **Grill Stress-Test**: Settled (`frontier-empty` in `02b-grill.md`).
- **Critical / Major Findings**: 0.

---

## 2. Detailed Surface Reviews

### 2.1 UI & Interaction Architecture
- **Filter Pills Reordering**: Filter buttons reordered to `Unread` (active default), `Today`, `Starred`, `All titles`. `setActiveFilter` toggles active styles immediately in browser without wait.
- **Hybrid Search Box**:
  - Typing triggers `oninput="filterArticlesClientSide(this.value)"` for zero-latency client-side DOM filtering across loaded titles.
  - Submitting form (`Enter` or search icon click) fires `GET /hx/entries` targeting `#entries-list` with `status=all`, querying all articles across Miniflux.
- **Mark This Page as Read**:
  - Replaces global mark-all-read with page-scoped action.
  - Helper `getVisibleEntryIds()` gathers only entries currently visible (`display !== 'none'`), respecting active client-side searches.
  - Upstream request marks only those entries as read and returns fresh `#entries-list` with updated `#unread-pill`.
- **Detail Metadata Cleanup**:
  - Author line (`• By <Author>`) is removed from `entry_detail.html`, leaving a clean `{{ entry.feed_title }} • {{ entry.published_at }}` display.
- **Detail "Mark as Unread" Button**:
  - Placed before Bookmark button in `entry_detail.html` header.
  - POSTs to `/hx/entries/{{ entry.id }}/read?status=unread`.
  - Swaps `#entry-{{ entry.id }}` in the list with the unread indicator dot and updates `#unread-pill` out-of-band.

### 2.2 Security, Robustness & Performance
- **Client Latency Reduction**: Eliminates 300ms network debounce requests while typing; only submits network query on deliberate Enter/submit.
- **Input Sanitization**: Search query parameters continue to be escaped via `serde_json::to_string` before inclusion in GraphQL documents.
- **Empty Batch Handling**: Handler safely checks for empty `entry_ids` before executing GraphQL mutation, avoiding unnecessary upstream round-trips.

### 2.3 Test Plan Review (`04-tasks.md`)
The TDD plan specifies 6 focused tests:
1. `test_filter_bar_reordering`: Verifies `IndexTemplate` rendered pill order and default active styling on `Unread`.
2. `test_detail_template_omits_author`: Verifies author display removal from `EntryDetailTemplate`.
3. `test_detail_template_has_mark_unread_button`: Verifies presence and attributes of Mark as Unread button.
4. `test_build_graphql_filter_search_defaults_to_all`: Verifies status fallback when search query is present.
5. `test_graphql_mutation_update_entries_status`: Verifies GraphQL batch update mutation.
6. `test_hx_mark_page_read_endpoint`: Verifies end-to-end Axum route handler with wiremock.

---

## 3. Findings & Notes

| ID | Severity | Surface | Finding | Resolution |
|---|---|---|---|---|
| D-01 | Minor | API / Route | Handle empty `entry_ids` gracefully in `mark_page_read_htmx` | Clarified in `03a-api-contract-review.md`; if list is empty, skip mutation and re-render current view |
| D-02 | Minor | UI | Add `data-entry-id` attribute to list items in `entry_list.html` and `entry_item.html` | Included in `04-tasks.md` Task 1.2 to support robust ID extraction in `getVisibleEntryIds()` |

---

## 4. Gate Sign-Off

- **Overall Result:** `clean`
- **Isolated API Result:** `clean` (`03a-api-contract-review.md`)
- **Isolated DB Result:** `skipped` (`Has DB = no`)
- **Ready for Gate B:** Yes
