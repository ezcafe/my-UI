Result: clean

# API Contract Review: Article Filters, Hybrid Search, Page Mark-Read & Detail Actions

- **Stage:** `api-contract-review` (Mode simple / design-verify-phase)
- **Status:** Paid (`clean`)
- **Review Target:** GraphQL Schema, Miniflux Client, and Axum Route Contracts

---

## 1. Executive Summary

This API contract review verifies the interface contracts across three integration layers:
1. **Miniflux REST Client (`crates/miniflux/src/client.rs`)**: Batch status update contract for `PUT /v1/entries`.
2. **GraphQL Schema & Resolvers (`crates/miniflux/src/schema/`)**: New mutation `updateEntriesStatus` for batch entry status update.
3. **Axum HTTP/HTMX Routes (`crates/server/src/routes/entries.rs`, `crates/server/src/app.rs`)**:
   - `GET /hx/entries`: Search status default fallback and query parameter contracts.
   - `POST /hx/mark-page-read`: Payload parsing (`entry_ids`), upstream mutation invocation, and HTMX out-of-band response.
   - `POST /hx/entries/{id}/read?status=unread`: Existing contract validation for unread status toggling.

No Critical or Major defects were identified. All contracts are backwards-compatible and adhere to project standards.

---

## 2. Interface Contract Checklist

### 2.1 Miniflux REST Client Contract (`crates/miniflux/src/client.rs`)

- [x] **Batch Endpoint Availability**: Upstream Miniflux exposes `PUT /v1/entries` accepting `{"entry_ids": [...], "status": "..."}`.
- [x] **Client Implementation**: `MinifluxClient::update_entry_status(&self, entry_ids: Vec<i64>, status: &str)` is already implemented in `crates/miniflux/src/client.rs` (lines 186-211) with proper serialization and error propagation.
- [x] **Status Values**: Accepts `"read"` and `"unread"` status strings conforming to Miniflux v2 API specifications.

### 2.2 GraphQL Schema & Resolver Contract (`crates/miniflux/src/schema/`)

- [x] **Mutation Definition**: Add `update_entries_status` to `Mutation` in `crates/miniflux/src/schema/mutation.rs`:
  ```rust
  pub async fn update_entries_status(&self, ctx: &Context<'_>, entry_ids: Vec<ID>, status: EntryStatus) -> Result<bool>
  ```
- [x] **ID Parsing & Validation**: Parse each `ID` into `i64`, filtering invalid values or returning appropriate error.
- [x] **Delegation**: Forwards `entry_ids: Vec<i64>` and `status.as_str()` directly to `client.update_entry_status(ids, status.as_str())`.
- [x] **Backwards Compatibility**: Existing `update_entry_status(id: ID, status: EntryStatus)` remains intact for single-item updates.

### 2.3 Server Route Interface Contracts (`crates/server/src/routes/entries.rs`, `crates/server/src/app.rs`)

- [x] **`GET /hx/entries` (Search & Status Query Contract)**:
  - Supports `search: Option<String>` and `status: Option<String>`.
  - When `search` is provided and `status` is omitted or set to `all`, query falls back to no status constraint in Miniflux, allowing read articles to be returned.
  - Search query strings are escaped via `serde_json::to_string` before inclusion in GraphQL filter strings.
- [x] **`POST /hx/mark-page-read`**:
  - Accepts URL-encoded form data containing:
    - `entry_ids`: Comma-separated list or JSON array of entry IDs.
    - `status`: Optional current active status filter (`#active-status`) to preserve view context.
    - `search`: Optional current search query to preserve search results.
  - Graceful empty check: If `entry_ids` is empty, safely skips the mutation without upstream error.
  - Returns rendered `EntryListTemplate` with updated entries and out-of-band `#unread-pill` swap (`hx-swap-oob="true"`).
- [x] **`POST /hx/entries/{id}/read?status=unread`**:
  - Route already registered and handled by `mark_entry_read_htmx`.
  - Query parameter `status=unread` maps to GraphQL `status: UNREAD`.
  - Returns rendered `EntryItemTemplate` with unread dot indicator and out-of-band `#unread-pill` swap.

---

## 3. Findings & Implementation Directives

### Finding API-01 (Minor / Directive): Robust Parsing of `entry_ids` in `mark_page_read_htmx`
- **Description**: The client button will send `entry_ids` as a comma-separated string (`"101,102,103"`).
- **Directive**:
  Parse `entry_ids` in `mark_page_read_htmx` using:
  ```rust
  let ids: Vec<async_graphql::ID> = form
      .entry_ids
      .split(',')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .map(|s| async_graphql::ID(s.to_string()))
      .collect();
  ```
  If `ids.is_empty()`, skip executing the mutation and directly re-query/render the current list.

---

## 4. Contract Sign-Off

- **Critical Findings:** 0
- **Major Findings:** 0
- **Minor Directives:** 1 (addressed above)
- **Result:** `clean`
