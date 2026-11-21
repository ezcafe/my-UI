# Analysis: Authentication Toggle, Client-Side State, Caching, and DDoS Protection

**Result:** done

## Context
The application is a spatial news reader connecting to Miniflux. To support public deployments, kiosk displays, or frictionless self-hosted usage, users need to be able to disable authentication completely via configuration (`AUTH_ENABLED=false`).

When authentication is enabled (`AUTH_ENABLED=true`):
- OIDC authentication and encrypted session cookies guard all endpoints.
- Read statuses and bookmark states synchronize directly to upstream Miniflux.
- Login and logout buttons are visible in the navigation header and auth screens.

When authentication is disabled (`AUTH_ENABLED=false`):
- Anyone can access the reader without logging in; login and logout buttons are removed.
- Read status is saved client-side using browser IndexedDB with an in-memory `Set` cache.
- Bookmarks are handled client-side using the Web Share API (`navigator.share`) or keyboard shortcut hint.
- Server mutations against Miniflux are blocked or no-oped to prevent guest users from mutating shared Miniflux state.
- Upstream Miniflux queries are cached in-memory on the server to prevent upstream flooding.
- In-flight DDoS rate-limiting and request capping protect public endpoints.

---

## Technical & Architectural Risks

### 1. Accidental Upstream State Corruption in Guest Mode
- **Risk**: If the server continues executing Miniflux mutations (`updateEntriesStatus`, `toggleBookmark`) when `AUTH_ENABLED=false`, a single guest user could mark all articles read for everyone or exhaust Miniflux API quotas.
- **Impact**: Loss of unread tracking for all users sharing the Miniflux API key.
- **Mitigation**: When `AUTH_ENABLED=false`, guard all mutation handlers (`/hx/entries/{id}/read`, `/hx/entries/{id}/bookmark`, `/hx/mark-page-read`, and detail view automatic read triggers) to skip upstream Miniflux calls and return clean UI responses immediately.

### 2. Client-Side Read State Latency & DOM Glitches
- **Risk**: Asynchronous IndexedDB queries on every rendered article card could cause layout shifts or visible latency before unread dots/bars disappear.
- **Impact**: Poor reading experience and jarring UI flicker.
- **Mitigation**: Hydrate an in-memory `Set<string|number>` from IndexedDB on initial page load. All subsequent lookups (`isRead(id)`) and DOM updates run synchronously in memory, while persisting back to IndexedDB asynchronously in the background.

### 3. Upstream Miniflux Cache Stampede
- **Risk**: High-frequency requests to `/` or `/hx/entries` could overload upstream Miniflux when unauthenticated access is enabled.
- **Impact**: Miniflux connection timeouts, HTTP 504 Gateway Timeouts, or rate-limit lockouts.
- **Mitigation**: Implement a server-side in-memory TTL cache (e.g., 30s TTL for feed/entry lists, 120s TTL for entry details) that serves repeated requests directly from memory.

### 4. Public Endpoint Denial of Service (DDoS)
- **Risk**: Open access allows bots and malicious crawlers to spam expensive endpoints (such as search, AI summaries, or GraphQL queries).
- **Impact**: Server CPU/memory exhaustion and degraded service for legitimate readers.
- **Mitigation**: Introduce an Axum/Tower rate-limiting middleware that tracks client IPs with a token bucket (e.g., 60 requests/min, burst 20) and returns HTTP 429 Too Many Requests when exceeded, accompanied by payload size limits.

### 5. Filter Tab Consistency with Local Read State
- **Risk**: Miniflux returns entries according to its own database status. In guest mode, Miniflux may consider an entry unread while the browser has already recorded it as read locally.
- **Impact**: "Unread" tab could show articles that the user already read locally.
- **Mitigation**: In guest mode, the client-side script monitors DOM insertions (or HTMX `htmx:afterSwap` events) and updates the unread indicator or hides read rows when the active filter is "unread".

---

## Open Decisions (Frontier for Grill)
1. **Mutation Route Response in Guest Mode**: Should `/hx/entries/{id}/bookmark` and `/hx/entries/{id}/read` return 200 OK with local visual updates, or should HTMX prevent network calls altogether via client-side event handlers?
2. **Server-Side Cache Strategy**: Lightweight in-memory store (`Arc<RwLock<HashMap<Key, CachedResponse>>>>`) vs third-party cache crate (`moka`).
3. **Rate Limiting Engine**: Tower-based token bucket per IP vs Axum middleware with in-memory sliding window.
4. **Configuration Validation**: When `AUTH_ENABLED=false`, how should missing OIDC environment variables be handled (optional with empty defaults vs fallback values)?
