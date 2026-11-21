# Framing & Intent: Authentication Toggle, Client-Side State, Caching, and DDoS Protection

- **Work size:** Project with written design
- **Has UI (tentative):** yes

## Takeaway & Intent Summary
We want to allow running the application either with authentication enabled or completely disabled. When authentication is on, all read status updates and bookmarking actions sync to upstream Miniflux as they do today. When authentication is off, the app runs without any login requirements: login and logout buttons are removed from the interface, read statuses are stored locally in the browser (`localStorage`), bookmarks are handled locally in the browser, and the server protects upstream Miniflux with in-memory caching and DDoS rate limiting.

## Framing

### Goals
- **Auth Toggle**: Add a configuration option (such as `AUTH_ENABLED=true|false`, defaulting to `true`) allowing the server to run in authenticated or open guest mode.
- **UI Adaptation**: When authentication is disabled, hide all login and logout buttons across header and pages.
- **Client-Side Read State**: When auth is off, save read entries in browser IndexedDB with an in-memory `Set` cache (Option 1). Visual indicators (unread dot, 3px red bar, counter) reflect local state, and detail opens automatically mark entries as read asynchronously without blocking UI or filling `localStorage`.
- **Browser Bookmarking**: When auth is off, handle bookmarks natively on the client: invoke Web Share API (`navigator.share`) on supporting platforms, and fall back to a clean toast/hint prompt ("Press ⌘+D / Ctrl+D to bookmark") on desktop browsers without calling Miniflux mutation endpoints.
- **Upstream Caching**: Implement a server-side cache (e.g., in-memory cache for feed lists and entry summaries) so repeated requests do not flood upstream Miniflux.
- **DDoS Prevention**: Add server rate-limiting and request-capping middleware (e.g., connection limits, request size limits, and per-IP rate limiting) to protect public endpoints.

### Non-Goals
- Changing the database schema in Miniflux itself.
- Building a multi-user account sync engine for no-auth mode (guest mode is scoped to that single browser).
- Disabling Miniflux API reads entirely (public feeds still fetch through Miniflux backend client, protected by cache).

### Anti-Goals
- Requiring dummy user credentials or hardcoded passwords when auth is disabled.
- Breaking existing authenticated OIDC login flow when `AUTH_ENABLED=true`.
- Complex distributed caching infrastructure (like Redis) when in-process memory caching satisfies requirements cleanly.

### Constraints
- Zero-build frontend architecture: keep vanilla JavaScript and HTMX patterns without adding node/npm toolchains.
- Respect existing Annuaire theme design tokens and styling rules.
- Maintain high test coverage across server routes and UI components.

---

## Shared Understanding
- [x] User confirmed (brainstorm hard-gate): Option 1 (IndexedDB + in-memory Set cache for local read statuses; native Web Share / shortcut prompt for bookmarks).
