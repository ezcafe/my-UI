Result: clean

# API Contract Review: my-ui wrapper (Re-review)

**Review Date:** 2026-11-19  
**Review Profile:** full  
**Status:** All Critical and Major findings verified and resolved  
**Reviewed Documents:**
- Design Document: [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md)
- Tasks Document: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md)
- Miniflux OpenAPI Specification: [miniflux-openapi.yaml](file:///Users/ptquang86/ws/my-ui/miniflux-openapi.yaml)
- Pocket-ID API Reference: [Pocket-ID Endpoints](https://pocket-id.org/docs/api/endpoints)

---

## Executive Summary

A comprehensive re-review of the API contracts in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md) confirms that **all 2 Critical issues and 3 Major issues previously identified have been cleanly resolved**. The contracts now strictly adhere to Miniflux v2.2.0 API specifications, Pocket-ID OIDC requirements, Axum/HTMX best practices, and clean Rust crate architecture.

No blocking issues remain. The API contracts are ready for implementation.

---

## Verification of Previous Findings

### 1. [Critical] Stateless PKCE and CSRF State Storage
- **Previous Finding:** `03-design.md` specified PKCE for Pocket-ID OIDC but omitted how the transient code verifier and CSRF state are stored across redirects without a database or server memory.
- **Resolution Verification:**
  - **Contract Definition:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L387-L401) defines an ephemeral encrypted cookie named `__oidc_temp`.
  - **Cookie Specs:** Encrypted via AES-256-GCM private jar, `HttpOnly = true`, `Secure = true` (in production), `SameSite = Lax`, `Path = /auth/callback`, and `Max-Age = 300` (5 minutes).
  - **Payload & Lifecycle:** Stores JSON `{"pkce_verifier": "...", "csrf_state": "..."}`. On `GET /auth/callback`, the server decrypts the cookie, validates state, sends the verifier to `/token`, and clears `__oidc_temp` (`Max-Age = 0`).
  - **Task Alignment:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md#L106-L124) Task 5 includes `temp_cookie.rs`, roundtrip encryption unit tests, and callback cookie clearing tests.
- **Verdict:** **Resolved (Clean)**.

### 2. [Critical] Miniflux `markAllAsRead` User ID Contract
- **Previous Finding:** Miniflux requires `userID: int64` for `PUT /v1/users/{userID}/mark-all-as-read`, but the design defined a parameterless mutation without resolving user ID.
- **Resolution Verification:**
  - **Contract Definition:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L111-L115) and [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L285-L288) specify that `MinifluxClient` calls `GET /v1/me` upon initialization or first request to cache `user_id: i64`.
  - **Resolver Mapping:** `markAllAsRead` mutation resolver invokes `PUT /v1/users/{cached_user_id}/mark-all-as-read`.
  - **Task Alignment:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md#L47-L60) Task 2 adds client methods `get_me` and `mark_all_as_read` with WireMock tests (`test_get_me_caches_user_id` and `test_mark_all_as_read_uses_cached_user_id`).
- **Verdict:** **Resolved (Clean)**.

### 3. [Major] HTMX Counter Synchronization & Error Protocols
- **Previous Finding:** Mutating entry read status left header unread pills and counters stale, and upstream errors during HTMX swaps risked overwriting target DOM elements with raw error text.
- **Resolution Verification:**
  - **Counter Synchronization:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L316-L331) specifies out-of-band updates using `hx-swap-oob="true"` on `#unread-pill` returned alongside row partials on all status-changing endpoints.
  - **Error Handling:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L350-L364) specifies that for HTMX requests, server errors return HTTP 200 with `HX-Reswap: none` and an out-of-band toast container append (`#toast-container` with `hx-swap-oob="beforeend"`), preventing DOM destruction.
  - **Task Alignment:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md#L84-L103) Task 4 delivers `toast.html` with OOB support; Task 6 includes `error.rs` and tests `test_hx_read_returns_oob_unread_pill` and `test_htmx_error_returns_oob_toast_without_reswapping_target`.
- **Verdict:** **Resolved (Clean)**.

### 4. [Major] Miniflux `Feed.unreadCount` Aggregation
- **Previous Finding:** Upstream Miniflux `GET /v1/feeds` does not include unread counts; they are provided via `GET /v1/feeds/counters`. The resolver lacked explicit merging instructions.
- **Resolution Verification:**
  - **Contract Definition:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L114-L115) and [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L281-L284) detail concurrent calls via `tokio::join!` fetching `/v1/feeds` and `/v1/feeds/counters`, matching `counters.unreads[feed.id]` to populate `Feed.unreadCount`.
  - **Task Alignment:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md#L50) Task 2 adds concurrent fetching in `get_feeds` and test `test_get_feeds_merges_unread_counters`; Task 3 tests GraphQL feed query unread count resolution.
- **Verdict:** **Resolved (Clean)**.

### 5. [Major] `Entry.summary` Crate Dependency Conflict
- **Previous Finding:** `summary.rs` was located in `crates/ui-components`, but GraphQL `Entry.summary` in `crates/miniflux` required it without a crate dependency.
- **Resolution Verification:**
  - **Architecture Adjustment:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L116-L117) and [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L291-L294) moved `summary.rs` into `crates/miniflux/src/summary.rs`.
  - **Clean Boundary:** `crates/miniflux` has zero dependencies on `crates/ui-components`, allowing the GraphQL schema to compute extractive summaries directly.
  - **Task Alignment:** [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/04-tasks.md#L46) Task 2 places `summary.rs` in `crates/miniflux`, tested by `test_extractive_summary_returns_first_two_sentences`.
- **Verdict:** **Resolved (Clean)**.

---

## Verification of Minor Findings

- **Session Cookie Budget (Issue 1.2):** Explicitly budgeted via `SessionData` struct ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L407-L416)); bulky `id_token` is discarded immediately after extracting claims.
- **Logout Behavior (Issue 1.3):** Documented as local session termination ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L417)).
- **Read Mutation Parameter Contract (Issue 2.2):** Defined with optional query/form `status=read|unread`, defaulting to toggle when omitted ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L347-L349)).
- **Query Parameter Contract (Issue 2.4):** Explicitly defined via `EntriesFilterQuery` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L334-L345)).
- **Miniflux Status Batch Payload (Issue 3.4):** Clarified to use batch endpoint `PUT /v1/entries` (`{"entry_ids": [id], "status": "read"}`) ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261118-my-ui-wrapper/03-design.md#L115)).

---

## Conclusion

The API contracts in `03-design.md` and `04-tasks.md` are comprehensive, fully aligned with external specifications, and free of architectural or protocol discrepancies.

**Result: clean**
