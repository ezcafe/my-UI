# Tasks: Authentication Toggle, Client-Side State, Caching, and DDoS Protection (TDD-Ready)

## Key Takeaways
- Every implementation task begins with a failing unit or integration test before production code is written.
- Tasks span configuration parsing, authentication extractor bypass, upstream query caching, DDoS rate-limiting and body limits, mutation short-circuiting on the server, and UI adaptation with IndexedDB client-side state.
- Planned tests are fully specified with explicit failing test names and test coverage.

---

## Planned Tests

| Case | Failing test name | Covers |
|------|-------------------|--------|
| 1. Config auth_enabled parsing | `test_config_parses_auth_enabled_and_defaults` | Parsing `AUTH_ENABLED` (boolean, defaults to `true`) in `Config::from_map` |
| 2. Config OIDC relaxed validation | `test_config_allows_missing_oidc_when_auth_disabled` | Allowing missing `SESSION_SECRET` and `OIDC_*` variables when `AUTH_ENABLED=false` |
| 3. Auth extractor guest bypass | `test_authenticated_user_bypasses_when_auth_disabled` | `AuthenticatedUser` returning guest user without checking cookies when `auth_enabled == false` |
| 4. Auth handlers redirect | `test_auth_routes_redirect_to_root_when_auth_disabled` | `/login`, `/auth/login`, and `/auth/logout` redirecting to `/` in guest mode |
| 5. In-memory cache operations | `test_miniflux_cache_get_and_insert_with_ttl` | Storing and retrieving values in `MinifluxCache`, respecting TTL expiration |
| 6. Upstream cache integration | `test_list_entries_serves_from_cache_when_cached` | Serving GraphQL entry list queries directly from cache to prevent upstream Miniflux queries |
| 7. Detail read mutation bypass | `test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled` | Bypassing `updateEntriesStatus` mutation when loading article details in guest mode |
| 8. Bookmark mutation bypass | `test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled` | Short-circuiting bookmark toggle on server without calling Miniflux when auth is off |
| 9. Mark read mutation bypass | `test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled` | Short-circuiting mark-read endpoint on server without calling Miniflux when auth is off |
| 10. Rate limiter throttling | `test_rate_limiter_allows_burst_and_throttles_excess_requests` | Allowing valid burst traffic and returning HTTP 429 when IP rate limit is exceeded |
| 11. Request body size limit | `test_ddos_body_limit_rejects_payloads_exceeding_cap` | Rejecting oversized payloads (> 64KB) with HTTP 413 Payload Too Large |
| 12. UI logout button visibility | `test_index_template_hides_logout_button_when_auth_disabled` | Conditionally hiding logout button in `IndexTemplate` when `auth_enabled == false` |
| 13. UI client state script | `test_index_template_renders_guest_state_script_when_auth_disabled` | Rendering guest-mode IndexedDB and Web Share script in `IndexTemplate` when `auth_enabled == false` |

---

## Detailed Task Breakdown

### Task 1: Configuration & Environment Support (`crates/server/src/config.rs`)
- **Behavior**: Add `pub auth_enabled: bool` to `Config`. Parse `AUTH_ENABLED` environment variable (default `true`). When `AUTH_ENABLED=false`, provide internal fallback values for `SESSION_SECRET` and `OIDC_*` variables so the app initializes without identity provider configuration. Update `.env.example` with `AUTH_ENABLED=true`.
- **Failing test names**:
  - `test_config_parses_auth_enabled_and_defaults`
  - `test_config_allows_missing_oidc_when_auth_disabled`
- **Files allowed**:
  - `crates/server/src/config.rs`
  - `.env.example`
- **Done evidence**: `cargo test -p server test_config_parses_auth_enabled` and `cargo test -p server test_config_allows_missing_oidc` pass.

### Task 2: Auth Extractor Bypass & Handlers (`crates/server/src/auth/`)
- **Behavior**: In `AuthenticatedUser::from_request_parts`, check `config.auth_enabled`. If `false`, immediately return `AuthenticatedUser { user_id: "guest".to_string(), username: "Guest".to_string(), access_token: String::new() }`. In `crates/server/src/auth/handlers.rs`, redirect `/auth/login`, `/auth/callback`, and `/auth/logout` directly to `/` when `!config.auth_enabled`.
- **Failing test names**:
  - `test_authenticated_user_bypasses_when_auth_disabled`
  - `test_auth_routes_redirect_to_root_when_auth_disabled`
- **Files allowed**:
  - `crates/server/src/auth/middleware.rs`
  - `crates/server/src/auth/handlers.rs`
- **Done evidence**: `cargo test -p server test_authenticated_user_` and `cargo test -p server test_auth_routes_` pass.

### Task 3: In-Memory Upstream Miniflux Cache (`crates/server/src/cache.rs`)
- **Behavior**: Implement `MinifluxCache` wrapping a thread-safe `Arc<RwLock<HashMap<String, (Instant, serde_json::Value)>>>` with TTL. Add `pub cache: Arc<MinifluxCache>` to `AppState`. Cache query results in `list_entries_htmx`, `index_page`, and `get_entry_detail_htmx`.
- **Failing test names**:
  - `test_miniflux_cache_get_and_insert_with_ttl`
  - `test_list_entries_serves_from_cache_when_cached`
- **Files allowed**:
  - `crates/server/src/cache.rs`
  - `crates/server/src/lib.rs`
  - `crates/server/src/app.rs`
  - `crates/server/src/routes/entries.rs`
  - `crates/server/src/routes/pages.rs`
- **Done evidence**: `cargo test -p server test_miniflux_cache_` and `cargo test -p server test_list_entries_serves_from_cache` pass.

### Task 4: Upstream Mutation Short-Circuiting in Guest Mode (`crates/server/src/routes/entries.rs`)
- **Behavior**: When `!config.auth_enabled`, skip calling Miniflux mutations in `get_entry_detail_htmx`, `mark_entry_read_htmx`, `toggle_bookmark_htmx`, `mark_page_read_htmx`, `mark_feed_read_htmx`, and `mark_all_read_htmx`, returning successful UI responses without mutating upstream data.
- **Failing test names**:
  - `test_get_entry_detail_skips_miniflux_mutation_when_auth_disabled`
  - `test_toggle_bookmark_htmx_skips_miniflux_when_auth_disabled`
  - `test_mark_entry_read_htmx_skips_miniflux_when_auth_disabled`
- **Files allowed**:
  - `crates/server/src/routes/entries.rs`
- **Done evidence**: `cargo test -p server test_get_entry_detail_skips_miniflux_mutation` and `cargo test -p server test_toggle_bookmark_htmx_skips_miniflux` pass.

### Task 5: DDoS Protection & Rate Limiting (`crates/server/src/rate_limit.rs`)
- **Behavior**: Implement an Axum middleware layer `RateLimiter` managing per-IP token buckets (capacity 20, refill 1/sec, max 60/min). Return HTTP 429 when limit is exceeded. Add `DefaultBodyLimit::max(64 * 1024)` to `build_app`.
- **Failing test names**:
  - `test_rate_limiter_allows_burst_and_throttles_excess_requests`
  - `test_ddos_body_limit_rejects_payloads_exceeding_cap`
- **Files allowed**:
  - `crates/server/src/rate_limit.rs`
  - `crates/server/src/lib.rs`
  - `crates/server/src/app.rs`
- **Done evidence**: `cargo test -p server test_rate_limiter_` and `cargo test -p server test_ddos_body_limit_` pass.

### Task 6: UI Templates & Client-Side State (`crates/ui-components/`)
- **Behavior**: Add `pub auth_enabled: bool` to `IndexTemplate`. Conditionally render logout button in `templates/pages/index.html` only when `auth_enabled == true`. When `auth_enabled == false`, render client-side IndexedDB initialization and event listeners to track read entries in an in-memory `Set` and handle native bookmarking via Web Share / shortcut toast.
- **Failing test names**:
  - `test_index_template_hides_logout_button_when_auth_disabled`
  - `test_index_template_renders_guest_state_script_when_auth_disabled`
- **Files allowed**:
  - `crates/ui-components/src/templates.rs`
  - `crates/ui-components/templates/pages/index.html`
  - `crates/ui-components/templates/partials/entry_detail.html`
- **Done evidence**: `cargo test -p ui-components test_index_template_` passes.
