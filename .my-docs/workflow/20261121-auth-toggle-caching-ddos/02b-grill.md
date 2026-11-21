# Grill Stress-Test & Decision Frontier

**Result:** frontier-empty

## Decision Tree & Settled Decisions

All core design and architectural decisions have been systematically mapped and settled through the confirmed framing in `01-idea.md`:

### 1. Authentication Toggle & Mode Control (Settled)
- **Decision**: How should authentication be toggled, and how should config handle missing OIDC credentials in guest mode?
- **Settled Decision**: Add `AUTH_ENABLED: bool` (defaulting to `true`). When `AUTH_ENABLED=false`:
  - `Config::from_map` treats `SESSION_SECRET` and `OIDC_*` variables as optional (providing sensible fallback/mock values if omitted), so the server can run without an identity provider.
  - `AuthenticatedUser` request extractor immediately succeeds with a guest principal (`user_id: "guest"`, `username: "Guest"`), eliminating redirect-to-login guards.
  - Authentication endpoints (`/login`, `/auth/login`, `/auth/callback`, `/auth/logout`) redirect to `/` or return an inactive response.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

### 2. UI Presentation & Navigation Adaptation (Settled)
- **Decision**: How should UI headers and pages adjust when auth is disabled?
- **Settled Decision**: In `crates/ui-components/templates/pages/index.html` and other templates, the header logout button (`<a href="/auth/logout">`) is conditionally rendered only when `auth_enabled` is true. In guest mode, all login/logout action controls are suppressed.
- **Status**: **Settled** (Explicit user requirement in `01-idea.md`).

### 3. Client-Side Read State Architecture (Settled)
- **Decision**: How should read state be persisted and rendered without server-side sync?
- **Settled Decision**: **Option 1 (IndexedDB + in-memory Set cache)**.
  - On page load, client-side script initializes an IndexedDB store (`my_news_read_entries`) and loads all read IDs into an in-memory JavaScript `Set`.
  - DOM rows (`#entry-{{ id }}-bar`, `#entry-{{ id }}-dot`) check the `Set` synchronously and hide unread indicators for read entries.
  - Opening an article or clicking mark-as-read immediately marks the ID in the in-memory `Set`, updates the local DOM indicators, and writes asynchronously to IndexedDB in the background.
  - "Mark This Page as Read" button marks all visible items in the `Set` and IndexedDB, updating the unread counter badge locally.
- **Status**: **Settled** (Confirmed user choice of Option 1 in `01-idea.md`).

### 4. Client-Side Bookmarking Flow (Settled)
- **Decision**: How should bookmarks work when auth is disabled?
- **Settled Decision**: Native browser bookmarking / sharing.
  - When clicking the bookmark button in `entry_detail.html`:
    - On mobile / supporting browsers: Triggers Web Share API (`navigator.share({ title, url })`).
    - On desktop / fallback browsers: Displays a lightweight toast notification ("Press ⌘+D / Ctrl+D to bookmark this article").
  - Does not invoke Miniflux bookmark mutation endpoints.
- **Status**: **Settled** (Confirmed user choice of Option 1 in `01-idea.md`).

### 5. Upstream Miniflux Caching (Settled)
- **Decision**: How should upstream queries be cached to avoid flooding Miniflux in open guest mode?
- **Settled Decision**: In-memory TTL cache on the server.
  - Caches entry list queries, unread counters, and entry detail responses.
  - TTL: 30 seconds for list/counter queries, 300 seconds for entry details.
  - When a cache entry is valid, requests are served instantly without issuing HTTP requests to Miniflux.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

### 6. DDoS Protection & Rate Limiting (Settled)
- **Decision**: What protections are needed for public endpoints?
- **Settled Decision**: Axum/Tower rate-limiting middleware.
  - Per-IP rate limiting (token bucket: 60 requests/minute with a burst capacity of 20).
  - Returns HTTP 429 Too Many Requests when limits are exceeded.
  - Request body size limits to prevent payload exhaustion attacks.
- **Status**: **Settled** (Confirmed in `01-idea.md`).

---

## Frontier Status
- **Open Questions**: None.
- **Frontier state**: **Empty**. All architectural decisions are aligned with codebase patterns and confirmed by the user.
