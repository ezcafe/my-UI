# Skim: Codebase Inspection

**Result:** done

## Key Takeaways
- **Config & Auth**: `Config` currently enforces `SESSION_SECRET` and `OIDC_*` variables unconditionally. Introducing `AUTH_ENABLED: bool` (defaulting to `true`) allows skipping or relaxing strict OIDC validation in no-auth mode. `AuthenticatedUser` extractor can seamlessly bypass session cookie checks and provide a guest principal when `AUTH_ENABLED=false`.
- **UI & Templates**: `IndexTemplate` and `base.html` render the user header with a direct logout link (`/auth/logout`), while `login.html` presents OIDC login. In guest mode (`AUTH_ENABLED=false`), logout/login buttons must be hidden.
- **Client-Side Read State & Bookmarks**: Entries currently use server-driven status toggling via `/hx/entries/{id}/read` and `/hx/entries/{id}/bookmark`. When auth is disabled, the UI will manage read states using browser `IndexedDB` with an in-memory `Set` cache (updating unread dots, 3px red bars, and counter locally), and handle bookmarks client-side via `navigator.share` / keyboard shortcut prompts without server mutation calls.
- **Upstream Caching & DDoS Protection**: Upstream Miniflux queries (entry lists, counters, entry details) currently hit Miniflux directly without intermediate caching. An in-memory TTL cache (e.g. `moka` or `std::sync::Arc<tokio::sync::RwLock<HashMap<...>>>`) will shield Miniflux from read floods. Axum/Tower middleware (or governor/token bucket) will provide per-IP rate limiting and payload limits for public guest access.

---

## Detailed Findings

### 1. Configuration & Authentication Architecture (`crates/server/src/config.rs`, `crates/server/src/auth/`)
- `Config::from_map`:
  - `session_secret`, `oidc_issuer_url`, `oidc_client_id`, `oidc_client_secret`, and `oidc_redirect_url` are presently fetched via `get_req(...)`.
  - Adding `AUTH_ENABLED` (boolean, default `true`): when `false`, `Config` can assign dummy/default values or treat OIDC variables as optional.
- `AuthenticatedUser` (`crates/server/src/auth/middleware.rs`):
  - Extracted from request parts. Checks session cookie and returns redirection to `/login` if missing or expired.
  - When `AUTH_ENABLED` is false, `from_request_parts` should instantly return an `AuthenticatedUser` with `user_id: "guest"`, `username: "Guest"`, `access_token: ""`.
- Auth Routes (`crates/server/src/auth/handlers.rs` & `app.rs`):
  - `/login`, `/auth/login`, `/auth/callback`, `/auth/logout` can redirect directly to `/` or return 404/disabled status when `AUTH_ENABLED=false`.

### 2. Server Routing & State (`crates/server/src/app.rs`, `crates/server/src/routes/`)
- `AppState` (`crates/server/src/app.rs`):
  - Holds `Arc<Config>`, `AppSchema`, and `Arc<AiService>`.
  - Cache service and rate limiter state can be held in `AppState` or registered as Tower layers.
- `list_entries_htmx` & `get_entry_detail_htmx` (`crates/server/src/routes/entries.rs`):
  - In `get_entry_detail_htmx`, reading an unread item currently fires a Miniflux GraphQL mutation `updateEntriesStatus(entryIds: ["{id}"], status: READ)`.
  - In guest mode, mutating Miniflux read status should be bypassed on the server, leaving status tracking purely client-side.
- Caching:
  - Repeated calls to fetch entry lists and entry details can be cached by query key with a short TTL (e.g., 30–60s) to absorb spikes.

### 3. UI Templates & Frontend State (`crates/ui-components/`)
- **Header & Navigation (`crates/ui-components/templates/pages/index.html`)**:
  - The logout button (`<a href="/auth/logout">`) is conditionally rendered only when `auth_enabled` is true.
  - Page mark-read button (`#mark-page-read-btn`) will invoke client-side mark-read in guest mode.
- **Entry Rows & Indicators (`crates/ui-components/templates/partials/entry_item.html`)**:
  - Contains `#entry-{{ entry.id }}-bar` (3px red positional bar) and `#entry-{{ entry.id }}-dot` (unread dot).
  - In guest mode, a small inline script or event handler checks the client-side `Set`/IndexedDB and hides the red bar and dot for locally read items.
- **Entry Detail & Bookmarks (`crates/ui-components/templates/partials/entry_detail.html`)**:
  - Unread button (`/hx/entries/{{ entry.id }}/read?status=unread`) and bookmark button (`/hx/entries/{{ entry.id }}/bookmark`).
  - In guest mode, clicking bookmark invokes Web Share API (`navigator.share`) or shows a non-intrusive toast ("Press ⌘+D / Ctrl+D to bookmark") instead of calling the server.

### 4. DDoS & Rate Limiting Protection
- Public endpoints need protection against request flooding and large payloads.
- Tower middleware can limit concurrent requests, enforce request timeouts, and throttle requests per IP address.
