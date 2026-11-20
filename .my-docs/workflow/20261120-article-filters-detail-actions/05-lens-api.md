# Lens: API Contract

**Lens:** api  
**Result:** clean  

## Summary

The API contract review evaluated the new and updated public endpoints and GraphQL schema mutations introduced for article filtering, hybrid search, page-level read marking, and detail view actions. All endpoints adhere to REST and GraphQL conventions, input handling is sanitized, error handling preserves HTMX UI integrity, and contracts match `03-design.md` specifications.

---

## API Checklist

| Check | Status | Evidence / Notes |
|---|---|---|
| **Typed input & output** | Pass | Form data in `MarkPageReadForm` and query params in `EntriesFilterQuery` are strongly typed in Axum. GraphQL `updateEntriesStatus` enforces typed `[ID!]!` and `EntryStatus!` enum. |
| **Stable error shape & status codes** | Pass | HTMX errors route through `handle_htmx_error`, returning rendered toast notifications with `HX-Reswap: none` to prevent breaking existing DOM state. Non-HTMX requests receive standard HTTP status codes. |
| **Edge validation** | Pass | Input validation occurs at HTTP and GraphQL entry points. Empty, whitespace-only, or invalid input is safely handled without panics or unnecessary upstream calls. |
| **Idempotency** | Pass | `POST /hx/mark-page-read` and `POST /hx/entries/{id}/read` are idempotent: marking articles as read or unread multiple times yields the same final state. |
| **GraphQL injection safety** | Pass | Search keyword in `build_graphql_filter` is sanitized via `serde_json::to_string`, securely escaping quotes, backslashes, and control characters before GraphQL query interpolation. |
| **Design contract alignment** | Pass | `/hx/entries`, `/hx/mark-page-read`, `/hx/entries/{id}/read`, and `updateEntriesStatus` match contracts in `03-design.md`. |
| **No internal leaks** | Pass | Errors do not expose internal paths, database connection strings, or sensitive API keys. Miniflux auth token remains securely stored within `MinifluxClient`. |
| **Backward compatibility** | Pass | Additive changes only. Previous routes (`/hx/mark-all-read`) and mutations remain intact and functional. |

---

## Endpoint Details

### 1. `POST /hx/mark-page-read`
- **Purpose**: Marks visible articles on the current page as read.
- **Payload**: Form-encoded `entry_ids` (supports JSON array string `["1","2"]` or comma-separated string `1,2`), `status`, `search`, `feed_id`, `category_id`.
- **Behavior**:
  - Handles empty ID lists as a no-op without error.
  - Calls `updateEntriesStatus` GraphQL mutation for non-empty lists.
  - Re-queries the entries list under current active filter criteria.
  - Returns updated `EntryListTemplate` with an out-of-band update for `#unread-pill`.
- **Verdict**: Clean.

### 2. `GET /hx/entries`
- **Purpose**: Retrieves filtered article list.
- **Parameters**: `status`, `search`, `feed_id`, `category_id`, `limit`, `offset`.
- **Behavior**:
  - Defaults to searching across all articles when `search` is provided without explicit `status`.
  - Escapes search strings via `serde_json::to_string`.
  - Supports pagination with `limit` and `offset`.
- **Verdict**: Clean.

### 3. `POST /hx/entries/{id}/read`
- **Purpose**: Marks a specific article as read or unread.
- **Parameters**: Query parameter `status=unread` or defaults to `read`.
- **Behavior**:
  - Calls `updateEntryStatus` GraphQL mutation.
  - Returns `EntryItemTemplate` with `hx-swap-oob="true"` updating the global unread count.
- **Verdict**: Clean.

### 4. GraphQL Mutation `updateEntriesStatus`
- **Signature**: `updateEntriesStatus(entryIds: [ID!]!, status: EntryStatus!): Boolean!`
- **Behavior**:
  - Parses each `ID` to `i64`. Returns early GraphQL error if parsing fails.
  - Calls Miniflux client `update_entry_status(ids, status)` in a single batch HTTP PUT request.
  - Maps upstream errors cleanly.
- **Verdict**: Clean.

---

## Findings

No Critical, Major, or Minor API contract defects found. All contracts are verified and clean.
