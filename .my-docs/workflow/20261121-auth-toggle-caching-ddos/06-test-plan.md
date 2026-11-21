# Test Plan and Verification Report: Authentication Toggle, Client-Side State, Caching, and DDoS Protection

- **Result:** success
- **Test path:** full
- **Smoke source:** Smoke-Task
- **Date:** 2026-11-21
- **Stage:** test-full

---

## Smoke Section

### 1. Scope & Strategy
The smoke verification executes the critical guest-mode and security boundary flows end-to-end:
1. **Unauthenticated Public Access**: Boot server with `AUTH_ENABLED=false` without OIDC credentials. Ensure guest access to root `/` and `/hx/entries` succeeds with HTTP 200 without redirecting to `/login`.
2. **Auth Route Protection**: Ensure `/auth/login`, `/auth/callback`, `/auth/logout`, and `/login` gracefully redirect to `/` in guest mode.
3. **Upstream Miniflux Shielding**: Ensure GraphQL queries (`/hx/entries`, `/entries/{id}`) are served from in-memory cache on subsequent requests without querying upstream.
4. **Mutation Short-Circuiting**: Ensure bookmarking and read-state mutation requests return immediate UI responses without mutating upstream Miniflux.
5. **DDoS & Body Caps**: Ensure oversized payloads (> 64KB) return HTTP 413, and burst traffic beyond 20 requests throttles with HTTP 429.

### 2. Smoke Commands & Results
- **Command 1**: `cargo test -p server test_config_` — Passed (6 tests: default config, auth disabled fallbacks, secret lengths).
- **Command 2**: `cargo test -p server test_authenticated_user_` & `test_auth_routes_` — Passed (2 tests: guest bypass, root redirect).
- **Command 3**: `cargo test -p server test_miniflux_cache_` & `test_list_entries_serves_from_cache_` — Passed (2 tests: TTL caching, 1 upstream request for 2 calls).
- **Command 4**: `cargo test -p server test_get_entry_detail_skips_miniflux_mutation` & `test_toggle_bookmark_htmx_skips_miniflux` & `test_mark_entry_read_htmx_skips_miniflux` — Passed (3 tests: 0 mutation requests upstream).
- **Command 5**: `cargo test -p server test_rate_limiter_` & `test_ddos_body_limit_` — Passed (2 tests: burst & 429 throttling, 64KB cap & 413 rejection).
- **Command 6**: `cargo test -p ui-components test_index_template_` — Passed (6 tests: hidden logout button, guest IndexedDB script).
- **Command 7 (E2E Integration)**: `cargo test -p server --test e2e_tests` — Passed (3 tests: health check, authenticated session flow, guest mode end-to-end flow).
- **Command 8 (Live Binary Smoke)**: Booted server binary on `127.0.0.1:3345` with `AUTH_ENABLED=false` — Verified `/health` responds HTTP 200 `{"status":"ok"}`; `/` serves HTML with rate-limiting headers; `/login` returns HTTP 303 redirecting to `/`.

---

## Test Execution Summary

| Crate / Target | Total Tests | Passed | Failed | Status |
|---|---|---|---|---|
| `server` (lib unittests) | 50 | 50 | 0 | pass |
| `server` (e2e integration) | 3 | 3 | 0 | pass |
| `ui-components` (lib unittests) | 24 | 24 | 0 | pass |
| `miniflux-client` (lib unittests) | 22 | 22 | 0 | pass |
| **Workspace Total** | **99** | **99** | **0** | **smoke-pass** |

---

## TDD Implementation Breakdown (`04-tasks.md`)

All planned TDD test cases from `04-tasks.md` were implemented using red-green TDD:

1. **`test_config_parses_auth_enabled_and_defaults`** (`crates/server/src/config.rs`):
   - Verifies `auth_enabled` defaults to `true` when omitted.
   - Verifies `AUTH_ENABLED=false` parses accurately.
2. **`test_config_allows_missing_oidc_when_auth_disabled`** (`crates/server/src/config.rs`):
   - Verifies fallback credentials for `SESSION_SECRET` and `OIDC_*` when auth is disabled.
3. **`test_authenticated_user_bypasses_when_auth_disabled`** (`crates/server/src/auth/mod.rs`):
   - Verifies extractor yields `AuthenticatedUser { user_id: "guest", username: "Guest", .. }` without checking cookies.
4. **`test_auth_routes_redirect_to_root_when_auth_disabled`** (`crates/server/src/auth/mod.rs`):
   - Verifies `/auth/login`, `/auth/callback`, `/auth/logout`, and `/login` redirect to `/` when `auth_enabled == false`.
5. **`test_miniflux_cache_get_and_insert_with_ttl`** (`crates/server/src/cache.rs`):
   - Verifies storing and retrieving values in `MinifluxCache` and eviction after TTL.
6. **`test_list_entries_serves_from_cache_when_cached`** (`crates/server/src/routes/mod.rs`):
   - Verifies GraphQL entry list queries hit the in-memory cache and prevent redundant upstream requests.
7. **`test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled`** (`crates/server/src/routes/mod.rs`):
   - Verifies `updateEntriesStatus` mutation is skipped when opening entry details in guest mode.
8. **`test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled`** (`crates/server/src/routes/mod.rs`):
   - Verifies bookmark toggling is short-circuited on the server without calling Miniflux.
9. **`test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled`** (`crates/server/src/routes/mod.rs`):
   - Verifies mark-read endpoint is short-circuited without mutating upstream Miniflux.
10. **`test_rate_limiter_allows_burst_and_throttles_excess_requests`** (`crates/server/src/rate_limit.rs`):
    - Verifies token bucket burst capacity (20) and throttling of excess requests.
11. **`test_ddos_body_limit_rejects_payloads_exceeding_cap`** (`crates/server/src/rate_limit.rs`):
    - Verifies request payloads exceeding 64KB are rejected with HTTP 413 Payload Too Large.
12. **`test_index_template_hides_logout_button_when_auth_disabled`** (`crates/ui-components/src/templates.rs`):
    - Verifies conditional omission of the logout button when `auth_enabled == false`.
13. **`test_index_template_renders_guest_state_script_when_auth_disabled`** (`crates/ui-components/src/templates.rs`):
    - Verifies rendering of client-side IndexedDB initialization and event listeners in guest mode.

---

## Full Test Suite Execution (`test-full`)

- **Command**: `cargo test --workspace`
- **Execution Date**: 2026-11-21
- **Result**: `success`
- **Test path**: `full`

### Workspace Test Matrix

| Target / Crate | Test Kind | Passed | Failed | Ignored | Status |
|---|---|---|---|---|---|
| `miniflux-client` (`crates/miniflux`) | lib unittests | 22 | 0 | 0 | pass |
| `server` (`crates/server`) | lib unittests | 50 | 0 | 0 | pass |
| `server` (`crates/server`) | integration e2e (`tests/e2e_tests.rs`) | 3 | 0 | 0 | pass |
| `ui-components` (`crates/ui-components`) | lib unittests | 24 | 0 | 0 | pass |
| **Total Workspace Tests** | **All targets** | **99** | **0** | **0** | **success** |

### Test Coverage Summary
1. **Authentication & Guest Mode**:
   - Config parsing of `AUTH_ENABLED` (both `true` and `false`).
   - Session secret length validation and fallback handling.
   - Bypass of auth middleware in guest mode producing synthetic `guest` user.
   - Redirect of `/auth/*` and `/login` endpoints to root `/` in guest mode.
   - Session encryption, tampering rejection, and OIDC callback verification in authenticated mode.
2. **In-Memory Caching**:
   - TTL expiration and cache invalidation.
   - Entry list query caching ensuring single upstream query for repeated requests.
3. **Miniflux Shielding**:
   - Read detail, mark-read, mark-page-read, mark-feed-read, and bookmark short-circuiting when auth disabled.
   - Upstream write prevention with instant optimistic UI response.
4. **DDoS Protection & Rate Limiting**:
   - Axum `DefaultBodyLimit` rejecting payloads > 64KB with HTTP 413.
   - Token bucket rate limiter enforcing 20 burst capacity and throttling with HTTP 429 and `Retry-After`.
5. **UI & Template Rendering**:
   - Guest IndexedDB sync script injected when `AUTH_ENABLED=false`.
   - Logout button omitted when `AUTH_ENABLED=false`.
   - Container query layout, theme switching, and XSS sanitization intact.
6. **End-to-End Integration**:
   - Health check endpoint `/health` verification.
   - End-to-end authenticated session lifecycle.
   - End-to-end guest mode browsing and HTMX interaction.
