Result: clean

# TDD Plan Review

This document evaluates the planned unit and integration tests defined in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/04-tasks.md) against the design specifications in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-auth-toggle-caching-ddos/03-design.md).

---

## 1. Test Coverage Analysis

### Requirement 1: Configuration & Environment Support
- **Planned tests**:
  - `test_config_parses_auth_enabled_and_defaults` in [config.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/config.rs)
  - `test_config_allows_missing_oidc_when_auth_disabled` in [config.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/config.rs)
- **Target verification**:
  - Validates parsing `AUTH_ENABLED` boolean flag with `true` as default.
  - Validates that when `AUTH_ENABLED=false`, missing OIDC variables and session secrets do not fail configuration parsing.
- **Verdict**: **Clean** — directly addresses configuration requirements and guest-mode fallback defaults.

---

### Requirement 2: Authentication Extractor Bypass & Route Handlers
- **Planned tests**:
  - `test_authenticated_user_bypasses_when_auth_disabled` in [middleware.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/auth/middleware.rs)
  - `test_auth_routes_redirect_to_root_when_auth_disabled` in [handlers.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/auth/handlers.rs)
- **Target verification**:
  - Validates `AuthenticatedUser` extractor immediately returns a guest user (`user_id: "guest"`, `username: "Guest"`) when auth is disabled.
  - Validates auth endpoints (`/login`, `/auth/login`, `/auth/callback`, `/auth/logout`) redirect to `/` without invoking external OIDC flows when auth is disabled.
- **Verdict**: **Clean** — validates identity extractor bypass and safe auth route redirection.

---

### Requirement 3: In-Memory Upstream Miniflux Caching
- **Planned tests**:
  - `test_miniflux_cache_get_and_insert_with_ttl` in [cache.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/cache.rs)
  - `test_list_entries_serves_from_cache_when_cached` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
- **Target verification**:
  - Validates TTL expiration and insertion/retrieval in thread-safe `MinifluxCache`.
  - Validates entry list queries serve cached responses without invoking upstream Miniflux GraphQL client.
- **Verdict**: **Clean** — verifies caching core mechanics and query route shielding.

---

### Requirement 4: Upstream Mutation Short-Circuiting
- **Planned tests**:
  - `test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - `test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - `test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
- **Target verification**:
  - Validates article detail view does not trigger `updateEntriesStatus` mutation when auth is disabled.
  - Validates bookmark toggle and mark-read endpoints return successful UI responses without calling Miniflux mutations in guest mode.
- **Verdict**: **Clean** — verifies that no unauthorized mutations are sent to Miniflux.

---

### Requirement 5: DDoS Protection & Rate Limiting
- **Planned tests**:
  - `test_rate_limiter_allows_burst_and_throttles_excess_requests` in [rate_limit.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/rate_limit.rs)
  - `test_ddos_body_limit_rejects_payloads_exceeding_cap` in [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs)
- **Target verification**:
  - Validates per-IP token bucket allows burst up to 20 requests and returns HTTP 429 when rate is exceeded.
  - Validates payloads exceeding 64KB cap are rejected with HTTP 413.
- **Verdict**: **Clean** — verifies rate limiting and request body protection against DDoS.

---

### Requirement 6: UI Adaptation & Client-Side State Tracking
- **Planned tests**:
  - `test_index_template_hides_logout_button_when_auth_disabled` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - `test_index_template_renders_guest_state_script_when_auth_disabled` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
- **Target verification**:
  - Validates `IndexTemplate` conditionally hides the logout icon button when `auth_enabled == false`.
  - Validates `IndexTemplate` renders guest-mode IndexedDB initialization and client-side state handling script when `auth_enabled == false`.
- **Verdict**: **Clean** — verifies template conditional rendering for guest mode.

---

## 2. Review Conclusion

All functional requirements identified during Ideation and Architecture Design are covered by explicit, named automated test cases.
The plan follows the TDD iron law:
- Tests will be written and verified failing (Red) before writing any production code.
- Minimal production code will be implemented to turn tests green (Green).
- Refactoring will proceed under green test coverage.

Result: **clean** — ready for Gate B.
