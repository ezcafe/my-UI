# Ideation Framing: UI Polish, Client-side Filter Caching & Loading State

## Goals
1. **Centered Login Card**: Remove the left brand red panel in `login.html`. Present a centered, clean, self-contained Swiss card containing the "My News" wordmark and Pocket-ID sign-in button.
2. **Seamless List Borders**: Remove the trailing bottom border of the last entry item in `entry_list.html` and `entry_item.html` using `last:border-b-0`.
3. **Filter Caching (1-Minute TTL)**: Clicking on filter tabs (e.g. Unread, Today, Starred, All Titles) within 60 seconds displays the cached HTML immediately without an unnecessary round-trip request to the server.
4. **HTMX Loading State**: Provide an immediate, elegant visual indicator (Annuaire-compliant top indicator bar / pulsing state) when data requests are in flight.

## Non-goals & Anti-goals
- **No Search Clear (x) Button**: Explicitly skipped per user instruction.
- **No backend database or schema modifications**: All behavior changes are strictly frontend template / client-side enhancements.
- **No third-party heavy JavaScript libraries**: Use vanilla JavaScript and existing HTMX lifecycle events (`htmx:beforeRequest`, `htmx:afterSwap`).
- **No disruption to Annuaire OKLCH design language**: Maintain zero-shadow floating layer principles, ink pills, and Swiss typography.

## Implementation Details

### 1. Centered Login Screen (`crates/ui-components/templates/pages/login.html`)
- Remove the left column (`bg-[var(--brand-surface)]`).
- Replace the 2-column grid (`grid-cols-1 md:grid-cols-2`) with a centered container:
  `min-h-screen flex items-center justify-center p-6 bg-background`.
- Keep the editorial wordmark (`MY NEWS`) and Pocket-ID ink pill button neatly grouped in the centered card.

### 2. Remove Last Item Border (`entry_list.html`, `entry_item.html`)
- Update the item container class list to include `last:border-b-0`:
  `class="annuaire-row ar-list-item ... border-b border-border/60 last:border-b-0"`.

### 3. Filter 1-Minute Client Caching (`index.html`)
- Introduce a lightweight client cache dictionary in JavaScript:
  `const filterCache = new Map();` storing `{ html: string, timestamp: number }`.
- Key by filter status (e.g., `'unread'`, `'today'`, `'starred'`, `'all'`).
- Intercept filter clicks: if a valid entry exists within 60,000 ms, inject into `#entries-list` and update active pill styling without issuing HTMX request; otherwise fetch via HTMX and store in `filterCache` on `htmx:afterSwap`.
- Invalidate cache when state-mutating actions occur (e.g. marking page or entry as read).

### 4. Loading State Indicator (`index.html`, `base.html`)
- Add a dedicated HTMX loading indicator bar at the top of the entries container:
  `<div id="list-loading-indicator" class="htmx-indicator h-0.5 bg-brand animate-pulse w-full"></div>`.
- Connect HTMX requests to trigger this indicator via `hx-indicator="#list-loading-indicator"`.

## Confirmation
Confirmed by user: "skip \"Search Clear (x) Button\". and build"
