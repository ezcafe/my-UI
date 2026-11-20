# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

## Adversarial

**Result:** clean

### Findings Verification

- **ADV-1 [Resolved - GraphQL Authentication Bypass]**:
  - `_user: AuthenticatedUser` extractor added to `graphql_handler` (`crates/server/src/routes/graphql.rs:8`).
  - `AuthenticatedUser::from_request_parts` in `crates/server/src/auth/middleware.rs:51-66` enforces HTTP 401 Unauthorized for unauthenticated `/graphql` requests.
  - Verified by `test_unauthenticated_graphql_request_rejected_with_401`.

- **ADV-2 [Resolved - Missing Session Expiration Enforcement]**:
  - `AuthenticatedUser::from_request_parts` in `crates/server/src/auth/middleware.rs:82-85` checks `session.expires_at <= chrono::Utc::now().timestamp()`.
  - Expired sessions are rejected with 401 / redirect.
  - Verified by `test_expired_session_rejected`.

- **ADV-3 [Resolved - Insecure Cookie Configuration]**:
  - `Config::is_secure(&self)` in `crates/server/src/config.rs:77-80` checks whether `app_base_url` starts with `https://` or host is external (`!= 127.0.0.1 && != localhost`).
  - Cookie builders `build_temp_set_cookie_header`, `build_session_set_cookie_header`, and clear helpers pass `config.is_secure()` in `crates/server/src/auth/handlers.rs`.
  - Verified by `test_secure_cookie_flag_when_https`.

- **ADV-4 [Resolved - Link Sanitization]**:
  - Sanitizer in `crates/ui-components/src/sanitizer.rs` uses Ammonia's `link_rel(Some("noopener noreferrer"))` combined with regex-based replacement across all `<a>` tags to guarantee both `target="_blank"` and `rel="noopener noreferrer"`.
  - Tested with multiple links and edge cases in `test_sanitizer_multiple_links_all_receive_target_blank`.

---

## Quality

**Result:** clean

### Findings Verification

- **QUA-1 [Resolved - UI State Corruption on Mutation Swaps]**:
  - `mark_entry_read_htmx` in `crates/server/src/routes/entries.rs:216-313` queries updated entry details and the unread counter from GraphQL after mutating status, populating real title, summary, feed title, and status in `EntryItemTemplate`.
  - `toggle_bookmark_htmx` in `crates/server/src/routes/entries.rs:177-209` checks current bookmark state via GraphQL query and toggles between filled (`fill-amber-400 text-amber-400`) and outline bookmark icon.
  - Verified by `test_hx_bookmark_returns_updated_icon`, `test_hx_bookmark_toggle_to_unbookmarked`, and `test_hx_read_returns_oob_unread_pill`.

- **QUA-2 [Resolved - O(N) In-Memory Scan for Single Entry]**:
  - `MinifluxClient::get_entry` in `crates/miniflux/src/client.rs:275-295` queries `GET /v1/entries/{id}` directly.
  - GraphQL `Query::entry` in `crates/miniflux/src/schema/query.rs:101-146` invokes `client.get_entry(entry_id)`.
  - `get_entry_detail_htmx` in `crates/server/src/routes/entries.rs:100-140` resolves the single entry directly via GraphQL query `entry(id: "{id}")` instead of scanning the full list.
  - Verified by `test_get_entry_success` and `test_get_entry_detail_htmx_fetches_single_entry`.

- **QUA-3 [Resolved - Clippy Warnings]**:
  - Simplified `splitn(2, '=')` to `split_once('=')` in `crates/server/src/auth/middleware.rs:26`.
  - Fixed format strings in `crates/server/src/routes/entries.rs`.
  - `cargo clippy --workspace --all-targets -- -D warnings` runs with 0 warnings.

---

## Merged

**Result:** clean
- Single lens in plan (`api`): all findings verified clean.
