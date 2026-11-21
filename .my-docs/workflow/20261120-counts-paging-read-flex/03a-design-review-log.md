Result: clean

# Design Review Log: Counts, Paging, Auto-Mark Read & Fluid Spatial Container

- **Stage:** `design-verify-phase`
- **Mode:** simple | **Profile:** lite
- **Surfaces:** UI (yes), API (yes), DB (no), auth/PII/schema (no)
- **Status:** Paid (`clean`)

---

## 1. Review Summary

The design artifacts (`01-idea.md`, `02-analysis.md`, `02b-grill.md`, `03-design.md`, and `04-tasks.md`) have been evaluated against architecture principles, user experience guidelines, interface contracts, and TDD specifications.

- **API Contract Review (`03a-api-contract-review.md`)**: Paid (`clean`). `entriesResult` and `todayCounter` queries cleanly extend GraphQL capabilities with minimal network overhead. HTMX routes preserve filter states and deliver seamless out-of-band updates.
- **Database Surface**: Skipped (`Has DB = no`). The app integrates directly with the Miniflux API; no local database migrations are required.
- **UI & Layout Surface**: Verified compliant with Spatial glassmorphism design. Adheres strictly to the user rule forbidding hardcoded breakpoints by using modern CSS container queries (`container-type: inline-size` and `max-width: min(100cqi, 1440px)`).
- **Grill Stress-Test**: Settled (`frontier-empty` in `02b-grill.md`). All user decisions (row retention, bottom scrollable pagination bar, inline auto-mark read) are honored.
- **Critical / Major Findings**: 0.

---

## 2. Detailed Surface Reviews

### 2.1 UI & Interaction Architecture
- **Fluid Spatial Container**:
  - Replaces rigid `max-width: 720px` with fluid `max-width: min(100cqi, 1440px)`.
  - Uses native CSS container query units (`cqi`) on `.spatial-window` inside `.spatial-shell`, satisfying responsiveness rules without hardcoded media queries.
- **Dynamic Filter Counts**:
  - Header and filter pills render live article counts: `Unread (<count>)` and `Today (<count>)`.
  - Out-of-band updates keep counts synchronized whenever an article is marked read.
- **Scrollable List-Bottom Pagination Bar**:
  - Attached directly to the bottom of `#entries-list`, scrolling naturally with list items per Grill Q2 decision.
  - Automatically hidden when total items fit within a single page (`total_pages <= 1`).
  - Clear visual affordances (`cursor-not-allowed` on disabled buttons).
- **Auto-Mark Read Without List Jumping**:
  - Opening an article triggers backend mark-read and clears the blue indicator dot via an out-of-band swap (`#entry-{id}-dot`).
  - The list row itself stays mounted, preventing unexpected viewport or scroll jumps per Grill Q1 decision.

### 2.2 Security, Robustness & Performance
- **Minimal Network Round-Trips**:
  - `today_counter` queries Miniflux with `limit: 1`, extracting the exact matching `total` from the response without pulling unnecessary article bodies.
  - `entries_result` retrieves page items and total count in a single query.
  - Auto-mark read executes inline during detail fetch, avoiding extra round-trip HTTP requests.
- **Input Escaping & Sanitization**:
  - Search terms are JSON-escaped before insertion into GraphQL filter parameters.
  - HTML content in article details continues to be sanitized via `sanitize_html`.
- **Idempotency**:
  - Multiple reads of the same article are safe and produce consistent out-of-band updates.

### 2.3 Test Plan Review (`04-tasks.md`)
The TDD plan defines 6 targeted tests:
1. `test_entries_result_returns_total_and_entries`: Verifies total count and entry items returned by `entriesResult`.
2. `test_today_counter_returns_correct_count`: Verifies UTC midnight calculation and Miniflux `published_after` query filter.
3. `test_entry_list_renders_pagination_when_multiple_pages`: Verifies pagination bar rendering when entries exceed 1 page.
4. `test_entry_list_hides_pagination_when_single_page`: Verifies pagination bar omission when items fit on 1 page.
5. `test_get_entry_detail_auto_marks_as_read_and_returns_oob_swaps`: Verifies inline status update and OOB swaps for unread counts and entry dot.
6. `test_spatial_window_fluid_width_container_query`: Verifies CSS container query declaration `min(100cqi, 1440px)`.

---

## 3. Findings & Notes

| ID | Severity | Surface | Finding | Resolution |
|---|---|---|---|---|
| D-01 | Minor | API | Avoid redundant mark-read call if article is already read | Only trigger `update_entry_status` if entry `status` is `"unread"`. |
| D-02 | Minor | UI / Route | Preserve filter & search state in pagination URLs | Pass `status` and `search` to `EntryListTemplate` so paging buttons preserve active filters. |

---

## 4. Gate Sign-Off

- **Overall Result:** `clean`
- **Isolated API Result:** `clean` (`03a-api-contract-review.md`)
- **Isolated DB Result:** `skipped` (`Has DB = no`)
- **Ready for Gate B:** Yes
