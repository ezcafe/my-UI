# Review Log

- **Result:** clean
- **Profile:** lite (bundled Adversarial + Quality)
- **Date:** 2026-11-19

## Summary

Verified code implementation across `crates/miniflux`, `crates/server`, and `crates/ui-components` for the Title-Only List Row, External Link Button, Today Filter, and Search/Filter integration. 

During the review, two edge cases were discovered and immediately resolved:
1. `crates/server/src/routes/pages.rs`: Initial `index_page` GraphQL query was missing `url` and `status` fields in its selection set, causing initial page render entries to have empty `url` links. Resolved by adding `url` and `status` and specifying `status: UNREAD`.
2. `crates/server/src/routes/entries.rs`: Search query parameter serialization was hardened with `serde_json::to_string` to guard against multiline strings, newlines, and control characters in GraphQL queries.

---

## Adversarial

- **Result:** clean

### Edge Cases & Security Checks
1. **External Link Security (Tab-Nabbing)**:
   - External link button in both `entry_item.html` and `entry_list.html` includes `target="_blank"` and `rel="noopener noreferrer"`.
   - Tested and verified: prevents reverse tab-hijacking and opener access.
2. **Event Propagation**:
   - External link button contains `onclick="event.stopPropagation()"`.
   - Prevents clicking the external link button from triggering the parent row's reader modal click handler (`hx-get="/hx/entries/{id}"`).
3. **GraphQL & Search Injection / Control Characters**:
   - `search` parameter in `build_graphql_filter` trims whitespace and serializes the search term using `serde_json::to_string`, securely escaping double quotes, backslashes, newlines (`\n`), carriage returns (`\r`), and control characters.
   - Whitespace-only search strings are safely ignored and omitted from the filter.
4. **Boundary & Timezone Handling**:
   - `status=today` computes UTC midnight timestamp (`00:00:00 UTC`) using `chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()`.
   - Faithfully follows settled user decision 1B (all articles published today, both read and unread).
5. **Initial State Synchronization**:
   - Initial page load (`index_page`) now fetches both `url` and `status` with default `status: UNREAD`, ensuring the default active filter ("Unread") matches the displayed dataset and external links are immediately functional.

---

## Quality

- **Result:** clean

### 5-Axis Quality Pass
1. **Correctness**:
   - Miniflux client correctly transmits `published_after` query parameter to upstream Miniflux API.
   - GraphQL schema resolves `publishedAfter` into domain filter.
   - Server filter builder properly generates GraphQL filter fragments for `all`, `unread`, `today`, `starred`, and `search`.
   - UI templates reflect unread status dots and correct active filter pill styling.
2. **Readability & Simplicity**:
   - Clean, minimal Askama templates.
   - Javascript helper `setActiveFilter` is concise and maintains single source of truth for `#active-status`.
   - No unnecessary dependencies or complex state machines.
3. **Architecture & Separation of Concerns**:
   - Strict separation between Miniflux client layer, GraphQL schema layer, Axum HTTP route handlers, and UI presentation templates.
   - View models (`EntryViewModel`) isolate template layer from internal GraphQL types.
4. **Adherence to Design & Prototypes (`ar-list.html`)**:
   - Strict adherence to `ar-list.html` spatial row structure.
   - Uses `ar-list-item`, `rounded-2xl`, gaze-friendly spacing, and title-only layout with `line-clamp-2`.
   - Completely omits redundant secondary metadata (summary, reading time, author, feed title) per user decision 2A.
5. **Performance**:
   - Server-side filter construction is lightweight with zero heavy allocations.
   - Client-side search input includes 300ms debounce delay (`delay:300ms`) via HTMX trigger to avoid hammering the server.
