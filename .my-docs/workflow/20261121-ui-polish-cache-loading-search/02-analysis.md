# Analysis: UI Polish, Client-Side Filter Caching & Loading State

Result: done

## Context & Architecture

This milestone delivers four user interface refinements to the AR Spatial RSS reader:
1. **Centered Login Card**: Replaces the split dual-panel login screen in `login.html` with a centered, self-contained Swiss card.
2. **Seamless List Borders**: Removes the bottom border on the last item in `entry_list.html` and `entry_item.html` via `last:border-b-0`.
3. **Filter 1-Minute Client Caching**: Adds an in-memory client cache (`Map`) in `index.html` with a 60-second TTL to avoid redundant server requests when repeatedly clicking filter tabs.
4. **HTMX Loading State**: Adds a top progress/pulsing indicator bar during HTMX requests for entries.
5. **Explicit Non-Goal**: Search Clear (x) button is skipped per user direction.

---

## Detailed Codebase Analysis

### 1. Centered Login Screen (`crates/ui-components/templates/pages/login.html`)
- **Current State**:
  - Uses `min-h-screen grid grid-cols-1 md:grid-cols-2 bg-background`.
  - Left panel: `bg-[var(--brand-surface)] text-[var(--brand-surface-foreground)] p-8 md:p-16` with wordmark, badge, description, and footer.
  - Right panel: `bg-card text-foreground` containing the login card with "Sign In" header and Pocket-ID button.
- **Target State**:
  - Remove the left brand panel entirely.
  - Center a single card on `min-h-screen flex items-center justify-center p-6 bg-background`.
  - The card features:
    - Editorial badge: `Editorial Directory` or `Edition 2026`.
    - Main title: `MY NEWS` in `font-display` with `text-brand` or `text-foreground`.
    - Subtitle: concise description of the reading environment.
    - Primary action: Pocket-ID ink pill button (`bg-primary text-primary-foreground font-bold rounded-full`).
- **Test Impact**:
  - `crates/ui-components/src/templates.rs::test_login_page_renders_dual_panel_swiss_brand` currently tests for `grid-cols-1 md:grid-cols-2` and `bg-[var(--brand-surface)]`.
  - This test must be updated to `test_login_page_renders_centered_card`, testing for the centered layout, wordmark, subtitle, and Pocket-ID ink pill.

### 2. Remove Last Item Border (`crates/ui-components/templates/partials/`)
- **Files**:
  - `crates/ui-components/templates/partials/entry_list.html`
  - `crates/ui-components/templates/partials/entry_item.html`
- **Current State**:
  - Container classes: `annuaire-row ar-list-item relative px-4 py-2.5 rounded-lg flex items-center justify-between gap-3 cursor-pointer select-none group hover:bg-muted/60 transition-colors border-b border-border/60`.
- **Target State**:
  - Append `last:border-b-0` to the class list:
    `... border-b border-border/60 last:border-b-0`.
  - Ensures the final article in `#entries-list` does not render an unnecessary trailing divider line above pagination.

### 3. Filter 1-Minute Client Caching (`crates/ui-components/templates/pages/index.html`)
- **Current State**:
  - Every filter pill (`#filter-unread`, `#filter-today`, `#filter-starred`, `#filter-all`, `#filter-ai`) issues an HTMX `GET /hx/entries?status=...`.
  - Re-clicking the same filter or switching back within seconds generates unnecessary network overhead and re-renders.
- **Target State**:
  - In `index.html`, declare an in-memory cache:
    `const filterCache = new Map();` storing `{ html: string, timestamp: number }`.
  - Intercept filter clicks:
    - If search input has text, bypass the status cache (or key with search).
    - If cached entry exists and `Date.now() - timestamp < 60_000`, swap `#entries-list` HTML directly, update active pill styling, re-initialize Lucide icons, and prevent the network request.
    - If cache miss or expired, let HTMX perform the request; on `htmx:afterSwap` for `/hx/entries`, store target HTML in cache with current timestamp.
  - Invalidation triggers:
    - Marking page as read (`/hx/mark-page-read`).
    - Marking individual entry read/unread (`/hx/entries/:id/read`).
    - Toggling bookmark (`/hx/entries/:id/bookmark`).
    - Local guest state changes in `readSet`.
    - On any invalidation trigger, call `filterCache.clear()`.

### 4. HTMX Loading State Indicator (`crates/ui-components/templates/pages/index.html`, `base.html`)
- **Current State**:
  - When fetching entries via HTMX, there is no immediate progress bar indicating network activity.
- **Target State**:
  - Add an indicator element above `#entries-list`:
    `<div id="entries-loading-bar" class="htmx-indicator h-0.5 bg-brand animate-pulse w-full"></div>`.
  - Add `hx-indicator="#entries-loading-bar"` to the filter bar / search triggers.
  - Define `.htmx-indicator` styling in `base.html` so it is cleanly hidden when idle and visible when `.htmx-request` is active.

---

## Risks & Mitigations

1. **Stale Cache Risk**: User marks an article as read or bookmarks it, but switching filters shows outdated item state.
   - *Mitigation*: Invalidate (`filterCache.clear()`) on all mutation events (`htmx:afterSwap` on write endpoints, and local guest DB updates).
2. **Search Conflict Risk**: User searches for text while filter status is cached.
   - *Mitigation*: Bypass filter status cache whenever `search-input` value is non-empty.
3. **Template Test Regressions**: Existing tests expect the dual-panel login screen.
   - *Mitigation*: Update unit test assertions in `crates/ui-components/src/templates.rs` to validate the new centered card structure.
