Result: clean

# API Contract Review: Title-Only List, Today Filter & Search/Filter Fixes

- **Stage:** `api-contract-review` (Mode simple / design-verify-phase)
- **Status:** Paid (`clean`)
- **Review Target:** GraphQL Schema, Miniflux REST Client, and Axum Route Contracts

---

## 1. Executive Summary

This API contract review verifies the interface contracts across three integration layers:
1. **Miniflux REST API**: Query parameter contracts for `/v1/entries`.
2. **GraphQL Schema & Resolvers**: `EntryFilterInput`, `Query::entries`, and type field exposure.
3. **Axum HTTP/HTMX Routes**: `/hx/entries` query parsing, sanitization, and ViewModel mapping.

No Critical or Major defects were identified. The contracts are sound, backwards-compatible, and adhere to OpenAPI/Miniflux upstream specifications.

---

## 2. Interface Contract Checklist

### 2.1 Miniflux REST Client Contract (`crates/miniflux/src/client.rs`)

- [x] **Parameter Name Compliance**: Upstream Miniflux (since v2.0.49) exposes `published_after` (Unix timestamp in seconds) on `GET /v1/entries`. The client maps `EntriesFilter.published_after` to query parameter `"published_after"`.
- [x] **Type Alignment**: `published_after` is typed as `Option<i64>` in `EntriesFilter` and serialized to string integer for HTTP query formatting (`req.query(&[("published_after", published_after.to_string())])`).
- [x] **Backwards Compatibility**: Adding an `Option<i64>` field to `EntriesFilter` with `Default::default()` preserves all existing call sites without breaking changes.
- [x] **Error Handling**: Existing `req.send().await.map_err(MinifluxError::from_reqwest)` and HTTP status validation handles upstream 4xx/5xx responses safely.

### 2.2 GraphQL Schema & Resolver Contract (`crates/miniflux/src/schema/`)

- [x] **Input Object Definition**: `EntryFilterInput` adds `pub published_after: Option<i64>`. By `async-graphql` convention, Rust field `published_after` maps to GraphQL input field `publishedAfter: Int` (or `i64` scalar).
- [x] **Search Field Preservation**: `EntryFilterInput` already defines `pub search: Option<String>`, and `Query::entries` resolver already forwards `f.search` to `domain_filter.search`.
- [x] **Domain Mapping**: Resolver in `crates/miniflux/src/schema/query.rs` maps `f.published_after` to `domain_filter.published_after`.
- [x] **Status Exposure on Output**: `EntryGql` already exposes `pub status: EntryStatus` (`Read`, `Unread`, `Removed`). This contract supports accurate read status resolution in parent callers.

### 2.3 Server Route Interface Contract (`crates/server/src/routes/entries.rs`)

- [x] **Query Parameter Contract**: `EntriesFilterQuery` accepts `status`, `feed_id`, `category_id`, `search`, `limit`, `offset`.
- [x] **Status Parameter Mapping**:
  - `status=all` -> No status filter applied in GraphQL.
  - `status=unread` (default) -> `status: UNREAD`.
  - `status=starred` -> `starred: true`.
  - `status=today` -> `publishedAfter: <start_of_today_utc_timestamp>` (all published today per decision 1B).
  - `status=read` -> `status: READ`.
- [x] **Search Input Sanitization**: Search string is trimmed and escaped against backslashes (`\`) and double quotes (`"`), avoiding GraphQL injection or document syntax corruption.
- [x] **Pagination Defaults**: Explicit fallback defaults for `limit` (default 50) and `offset` (default 0) are enforced.

---

## 3. Findings & Implementation Directives

### Finding API-01 (Minor / Implementation Directive): Request `status` in GraphQL Selection Set for Accurate `is_read` Resolution
- **Description**: In `list_entries_htmx`, `EntryViewModel.is_read` was previously derived via `query.status.as_deref() == Some("read")`. For `status=today`, `status=all`, or `status=starred`, this heuristic fails because both read and unread items can be returned.
- **Directive**:
  1. Add `status` to the GraphQL selection set in `list_entries_htmx`:
     ```graphql
     query {
       entries(filter: { ... }) {
         id
         title
         status
         summary
         publishedAt
         readingTime
         isBookmarked
         url
       }
       unreadCounter
     }
     ```
  2. Map `is_read` dynamically from the item payload:
     ```rust
     let is_read = item.get("status").and_then(|v| v.as_str()) == Some("READ");
     ```
- **Severity**: Minor (low risk, clear directive for TDD build).

---

## 4. Contract Sign-Off

- **Critical Findings:** 0
- **Major Findings:** 0
- **Minor Directives:** 1 (addressed above)
- **Result:** `clean`
