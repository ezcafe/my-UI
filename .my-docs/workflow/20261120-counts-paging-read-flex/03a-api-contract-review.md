Result: clean

# API Contract Review: Counts, Paging, Auto-Mark Read & Fluid Spatial Container

- **Stage:** `api-contract-review` (Mode simple / `design-verify-phase`)
- **Status:** Paid (`clean`)
- **Review Target:** GraphQL Schema, Miniflux Client, and Axum Route Contracts

---

## 1. Executive Summary

This API contract review verifies the interface specifications across the application's three architectural layers:
1. **Miniflux Client Layer (`crates/miniflux/src/client.rs`)**: Verifies data models and API interactions for entry counting, paging metadata, and status updates.
2. **GraphQL Schema Layer (`crates/miniflux/src/schema/`)**: Defines the new `entriesResult` query and `todayCounter` query, ensuring consistency with existing resolvers and error handling.
3. **Axum HTTP & HTMX Layer (`crates/server/src/routes/`)**: Specifies route contracts for `GET /hx/entries` (pagination and filter preservation) and `GET /hx/entries/{id}` (inline auto-mark read with out-of-band swaps).

No Critical or Major defects were identified. All interface contracts are backwards-compatible and adhere to project standards.

---

## 2. Interface Contract Checklist

### 2.1 Miniflux Client Contract (`crates/miniflux/src/client.rs`)

- [x] **Pagination Support**: `MinifluxClient::get_entries(&self, filter: &EntriesFilter)` already maps upstream Miniflux response to `EntryResultSet { total: i64, entries: Vec<Entry> }`. The `total` count is populated directly by Miniflux.
- [x] **Today Count Optimization**: Calling `get_entries()` with `EntriesFilter { published_after: Some(utc_midnight), limit: Some(1), ..Default::default() }` retrieves the exact `total` matching articles published today in a single lightweight round-trip without transferring unnecessary entry payloads.
- [x] **Status Update Contract**: `MinifluxClient::update_entry_status(&self, entry_ids: Vec<i64>, status: &str)` supports `PUT /v1/entries` to mark entries as `"read"`. Idempotent operation: marking an already-read article as read has no adverse side effects.

### 2.2 GraphQL Schema Contract (`crates/miniflux/src/schema/`)

- [x] **`EntryResultSetGql` Output Object (`crates/miniflux/src/schema/types.rs`)**:
  ```rust
  #[derive(SimpleObject, Clone, Debug)]
  pub struct EntryResultSetGql {
      pub total: i64,
      pub entries: Vec<EntryGql>,
  }
  ```
- [x] **`Query::entries_result` (`crates/miniflux/src/schema/query.rs`)**:
  ```rust
  pub async fn entries_result(
      &self,
      ctx: &Context<'_>,
      filter: Option<EntryFilterInput>,
  ) -> Result<EntryResultSetGql>
  ```
  - Reuses existing `EntryFilterInput` validation (rejects negative `limit` and `offset` with clear errors).
  - Backwards-compatible: existing `Query::entries` remains functional without breaking changes.
- [x] **`Query::today_counter` (`crates/miniflux/src/schema/query.rs`)**:
  ```rust
  pub async fn today_counter(&self, ctx: &Context<'_>) -> Result<i64>
  ```
  - Calculates UTC midnight (`chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()`).
  - Queries `client.get_entries()` with `published_after` and `limit: 1`, returning `total`.
- [x] **Status Update Mutation**: `Mutation::update_entry_status(id: ID, status: EntryStatus)` and `update_entries_status` are already implemented and tested in `crates/miniflux/src/schema/mutation.rs`.

### 2.3 Server Route & HTMX Contracts (`crates/server/src/routes/`)

- [x] **`GET /hx/entries` (Entries List & Pagination)**:
  - **Query Parameters**:
    - `status`: Filter token (`"unread"`, `"today"`, `"starred"`, `"all"`). Defaults to `"unread"`.
    - `search`: Search query string, safely JSON-escaped before GraphQL inclusion.
    - `limit`: Number of items per page (default 50).
    - `offset`: Pagination offset (default 0).
  - **Pagination Calculations**:
    - `current_page = (offset / limit) + 1`
    - `total_pages = ((total + limit - 1) / limit).max(1)`
    - `has_prev = offset > 0`
    - `has_next = offset + limit < total`
    - `prev_offset = (offset - limit).max(0)`
    - `next_offset = offset + limit`
  - **HTMX Swaps**:
    - Primary target: `#entries-list` (replaces inner HTML with list items and list-bottom pagination bar).
    - Out-of-band: `#unread-pill` and `#filter-unread-count` updated with current unread count.
- [x] **`GET /hx/entries/{id}` (Detail View & Auto-Mark Read)**:
  - **Path Parameter**: `id: i64`
  - **Behavior**:
    1. Fetches entry by ID.
    2. If entry status is unread, invokes status update to `"read"`.
    3. Fetches updated unread count.
  - **Response & Out-of-Band Swaps**:
    - Primary target: `#detail-panel` receiving rendered `EntryDetailTemplate`.
    - OOB Swap 1: `<span id="unread-pill" hx-swap-oob="true">...</span>`
    - OOB Swap 2: `<span id="filter-unread-count" hx-swap-oob="true">...</span>`
    - OOB Swap 3: `<span id="entry-{id}-dot" hx-swap-oob="true"></span>` (removes blue dot while leaving article row intact in the list).

---

## 3. Findings & Implementation Directives

### Finding API-01 (Minor / Directive): Guard Status Update on Already-Read Items
- **Description**: If a user clicks an article that is already read (e.g. from "All Titles" or "Today"), invoking an upstream `update_entry_status` is redundant.
- **Directive**: Check the entry's status first. Only invoke the Miniflux status update mutation when `status == "unread"`.

### Finding API-02 (Minor / Directive): Include `status` and `search` in Template Pagination URLs
- **Description**: Pagination buttons must retain active filter state across page navigation.
- **Directive**: Pass `status` and `search` into `EntryListTemplate` so the pagination buttons can generate complete query strings (`/hx/entries?status={{ status }}&search={{ search }}&offset={{ next_offset }}&limit={{ limit }}`).

---

## 4. Contract Sign-Off

- **Critical Findings:** 0
- **Major Findings:** 0
- **Minor Directives:** 2 (addressed in directives above)
- **Result:** `clean`
