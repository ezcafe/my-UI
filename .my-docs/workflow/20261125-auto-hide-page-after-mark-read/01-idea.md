# Ideation — Auto-Hide Page, Show Loading, and Load Next Page After Mark Page as Read

- **Slug:** `20261125-auto-hide-page-after-mark-read`
- **Has UI (tentative):** `yes`
- **Size:** Small UI / HTMX change

> You can reply to confirm or clarify, or say **"just start"** to skip ideation and proceed with defaults.
> **One-line approach:** Reuse `#entries-loading-overlay` (`showEntriesLoadingState()` / `hideEntriesLoadingState()`) and HTMX (`#pagination-next-btn` / `GET /hx/entries` in guest mode, `POST /hx/mark-page-read` in auth mode) so marking a page as read on the `Unread` filter immediately hides the current list, shows the loading spinner, fetches the next page, and reveals it once loaded.

---

## Shared Understanding (Playback)

When viewing the **Unread** filter and clicking **Mark This Page as Read** (`#mark-page-read-btn`), here is how the app behaves today vs. what seems broken or missing:

- **What already works (Auth mode, `auth_enabled = true`):**
  In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L41-L55), clicking `#mark-page-read-btn` triggers `htmx:beforeRequest`, which closes `#detail-panel`, hides `#entries-list` (`invisible`), and shows `#entries-loading-overlay`. It sends one `POST /hx/mark-page-read` request to [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1109-L1250), which marks the items read and returns the next unread page (`offset = 0`) in the same response. `htmx:afterSwap` then hides the loading overlay and shows the new page. *(Guess: `#entries-list.scrollTop = 0` is missing on swap, `hx-disabled-elt="this"` is missing, and other cached filter tabs are not cleared.)*
- **What is broken (Guest mode, `auth_enabled = false`):**
  In [markVisiblePageLocallyRead()](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1694-L1711), clicking `#mark-page-read-btn` marks items in local `readSet` / IndexedDB and calls `syncDomReadStates()`. On the **Unread** filter, that immediately hides all rows and flashes **"All caught up!"** (`#client-empty-state`). It **never** calls `showEntriesLoadingState()`, **never** shows `#entries-loading-overlay`, and **never** requests the next page (`#pagination-next-btn` / `GET /hx/entries`), even when more unread pages exist. *(Note: `getVisibleEntryIds()` also returns a comma-separated string while `markVisiblePageLocallyRead()` checks `Array.isArray(ids)`.)*

### Goals
- Immediately hide the current page content (`#entries-list`) and show the loading indicator (`#entries-loading-overlay`) when marking a page as read on the **Unread** filter.
- Automatically request the next page of unread articles (both in guest mode via `#pagination-next-btn` / `GET /hx/entries` and in auth mode).
- Display the next page (scrolled to the top) and hide the loading indicator once the response arrives; if no next page remains, show the clean empty state (`"All caught up!"`) and toast notification.

### Non-Goals & Anti-Goals
- **No schema or DB changes:** Keep the existing PostgreSQL / Miniflux GraphQL / IndexedDB storage structure unchanged.
- **No redesign of the loading UI:** Reuse `#entries-loading-overlay` and `#entries-loading-bar` rather than building a second spinner or skeleton component.
- **Do not leave `#entries-list` hidden on error:** Always restore visibility and show an error toast if the network or server request fails.

### Constraints
- Works cleanly in both **List** view and **Magazine (AI)** view.
- Preserves existing detail-panel closing (`closeDetailPanel()`) and filter-tab caching rules (`filterCache`).
- Follows existing HTMX 1.9.12 (`hx-*` / `htmx:*` events) and Askama template conventions in `crates/ui-components`.

---

## Open Question for User

Which case are you seeing right now when you mark a page as read on the **Unread** filter — is it **Guest mode (`auth_enabled = false`)** stopping on `"All caught up!"` without loading the next page, **Auth mode (`auth_enabled = true`)** needing a change (for example, hiding/loading behavior, scroll reset, or splitting mark-read from fetching the next page), or **both**?
