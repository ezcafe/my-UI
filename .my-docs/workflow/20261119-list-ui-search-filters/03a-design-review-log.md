Result: clean

# Design Review Log: Title-Only List, External Link, Today Filter & Search/Filter Fixes

- **Stage:** `design-verify-phase`
- **Mode:** simple | **Profile:** lite
- **Surfaces:** UI (yes), API (yes), DB (no), auth/PII/schema (no)
- **Status:** Paid (`clean`)

---

## 1. Review Summary

The design artifacts (`01-idea.md`, `02-analysis.md`, `02b-grill.md`, `03-design.md`, and `04-tasks.md`) have been reviewed against project architecture standards, UI spatial styling requirements (`ar-list.html`), interface contracts, and security best practices.

- **API Contract Review (`03a-api-contract-review.md`)**: Paid (`clean`). Miniflux client `published_after` query parameter complies with Miniflux REST API; GraphQL `EntryFilterInput` and resolver changes are backwards-compatible; search query escaping prevents syntax injection.
- **Database Surface**: Skipped (`Has DB = no`).
- **UI Surface**: Verified compliant with `ar-list.html` spatial glassmorphism style, proper unread indicator dot, external link action isolation, and instant active filter pill feedback.
- **Critical / Major Findings**: 0.

---

## 2. Detailed Surface Reviews

### 2.1 UI & Interaction Architecture
- **Title-Only Layout**: Row layout in `entry_item.html` and `entry_list.html` replaces multi-line summary cards with concise title rows (`line-clamp-2`, `text-sm font-medium`, `text-slate-100`), aligning directly with `ar-list.html`.
- **External Link Behavior**: Anchor element includes `target="_blank" rel="noopener noreferrer"` and `onclick="event.stopPropagation()"`, preventing accidental opening of the reader modal when clicking the external link.
- **Filter Toolbar**: Ordered as `All Titles`, `Unread`, `Today`, `Starred`. Active pill script (`setActiveFilter`) toggles Tailwind classes immediately without server round-trip latency.
- **Search Integration**: Search input includes `hx-get="/hx/entries"`, debounced at 300ms, and updates `#entries-list`.

### 2.2 API & Data Flow
- **Miniflux Upstream**: `published_after` accurately leverages the Miniflux REST API integer timestamp parameter to filter today's articles.
- **GraphQL Resolver**: Schema additions to `EntryFilterInput` seamlessly forward to `EntriesFilter` and `MinifluxClient`.
- **Server Route Logic**: Search keyword escaping and status parameter mapping resolve existing bugs where search was dropped and filters defaulted to `UNREAD`.

### 2.3 Security & Robustness
- **Tabnabbing Prevention**: All external links enforce `rel="noopener noreferrer"`.
- **Injection Safety**: GraphQL query builder sanitizes double quotes and backslashes in user search strings.
- **Event Isolation**: `event.stopPropagation()` on the external link button ensures distinct click target behavior.

### 2.4 Test Plan Review (`04-tasks.md`)
- Planned unit tests cover:
  1. `test_render_title_only_entry_item`: Verifies HTML structure, external link attributes, unread dot indicator.
  2. `test_render_entry_list_with_title_only_items`: Verifies list rendering.
  3. `test_render_index_filter_bar`: Verifies filter buttons ordering and HTMX URLs.
  4. `test_miniflux_client_get_entries_published_after`: Verifies client query parameters.
  5. `test_graphql_query_entries_published_after_and_search`: Verifies GraphQL inputs and resolver propagation.
  6. `test_entries_filter_query_mapping`: Verifies route query parsing and GraphQL string generation.

---

## 3. Findings & Notes

| ID | Severity | Surface | Finding | Resolution |
|---|---|---|---|---|
| D-01 | Minor | API / Route | `EntryViewModel.is_read` needs dynamic `status` evaluation from GraphQL rather than query string heuristic | Detailed in `03a-api-contract-review.md`; request `status` in GraphQL query in `list_entries_htmx` |
| D-02 | Minor | UI | Ensure search input retains active filter context (`hx-include`) | Detailed in `03-design.md` Task 3 |

---

## 4. Gate Sign-Off

- **Overall Result:** `clean`
- **Isolated API Result:** `clean` (`03a-api-contract-review.md`)
- **Isolated DB Result:** `skipped` (`Has DB = no`)
- **Ready for TDD Build:** Yes
