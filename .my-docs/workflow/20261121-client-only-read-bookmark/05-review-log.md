# Code review log

**Overall Result:** clean

---

## Adversarial

**Result:** clean

### 1. Key Takeaways & Security Boundaries
- **Strict Server Mutation Shielding**: Verified that all mutation endpoints (`/hx/entries/{id}/read`, `/hx/entries/{id}/bookmark`, `/hx/mark-page-read`, `/hx/feeds/{id}/mark-all-read`, `/hx/mark-all-read`) strictly return `HTTP 403 Forbidden` (`Mutations require authentication`) when `auth_enabled=false`. Unauthenticated clients cannot alter upstream Miniflux state under any circumstances.
- **Client-Side Guest Isolation**: In guest mode (`auth_enabled=false`), read status is persisted locally in client IndexedDB (`my_news_guest_db`) without issuing unauthorized network requests to the backend.
- **Bulk Operation Resilience**: The `#mark-page-read-btn` cleanly branches between server HTMX submission when authenticated and local `markVisiblePageLocallyRead()` when unauthenticated, avoiding server-side error cascades.

### 2. Edge Case Hunting & Adversarial Findings
- **Server Mutation 403 Guardrails**:
  - *Observation*: Previously, `toggle_bookmark_htmx` returned an inlined HTML button with status 200 OK even when `auth_enabled=false`.
  - *Resolution*: Updated to return `(StatusCode::FORBIDDEN, "Mutations require authentication")`. All 5 mutation endpoints are now unified behind early HTTP 403 guards.
- **Local Unread Counter Sync**:
  - *Observation*: When entries are read or unread locally in guest mode, `#unread-pill` and `#filter-unread-count` in the DOM must reflect the updated client count.
  - *Resolution*: `syncDomReadStates()` iterates over `.annuaire-row`, evaluates against `readSet`, and dynamically updates both `#unread-pill` text (`"{unreadCount} unread"`) and `#filter-unread-count` text (`"({unreadCount})"`).
- **Native Bookmark Sharing Fallback**:
  - *Observation*: In guest mode, bookmarking triggers `triggerNativeBookmark(title, url)`. If `navigator.share` is unavailable or rejected, it gracefully copies the URL to the clipboard via `navigator.clipboard.writeText` and displays a floating toast notification. If clipboard access is blocked, it displays a Cmd+D / Ctrl+D fallback tip.
  - *Minor Hardening Note*: In [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L54), passing arguments via inline `onclick="triggerNativeBookmark('{{ entry.title|escape }}', '{{ entry.url|escape }}')"` works for standard titles; as a future non-blocking polish, using data attributes (`data-title="{{ entry.title }}" data-url="{{ entry.url }}" onclick="triggerNativeBookmark(this.dataset.title, this.dataset.url)"`) can provide extra resilience against unusual edge cases where article titles contain unescaped single quotes in custom RSS feeds.

---

## Quality

**Result:** clean

### 1. Five-Axis Quality Assessment
- **Correctness**: Fully compliant with [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-client-only-read-bookmark/03-design.md) specifications. All unit, integration, and E2E tests pass. Route tests in [routes/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L863-L888) and E2E tests in [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs#L287-L364) explicitly verify HTTP 403 rejection on unauthenticated mutations.
- **Readability**: Code structure follows repository conventions. Rust handlers in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) use concise early returns. Template conditionals (`{% if auth_enabled %}`) in [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html) and [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) clearly separate authenticated HTMX behavior from guest JavaScript handlers.
- **Architecture**: Zero added external dependencies. Clean separation between data model, presentation templates, and server routes. Preserves zero-build frontend principles using vanilla browser APIs (IndexedDB, Web Share API, Clipboard API).
- **Security**: Upstream Miniflux instance is completely insulated from guest mutation traffic. No sensitive session data or internal server exceptions are leaked in 403 responses.
- **Performance**: Guest actions require zero network round-trips. Local IndexedDB writes and DOM element updates are instantaneous.

### 2. Code Health & Simplification
- No dead code or unused imports introduced.
- `cargo check --all-targets` passes with zero warnings.
- Clean and consistent UI styling using Annuaire design tokens (`border-border`, `bg-card`, `hover:bg-muted`).

---

## Merged

**Result:** clean
