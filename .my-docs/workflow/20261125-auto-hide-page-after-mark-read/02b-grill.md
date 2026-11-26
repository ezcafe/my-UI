# Grill log

**Result:** frontier-empty

**Grill open** = ¬Notes `grill-skip` ∧ (no `02b` ∨ Result ≠ `frontier-empty`) (incl. template **`pending`** alone — does not close). **Closed** on Notes `grill-skip` (parent after Parent-relay **or** labeled solo after AskQuestion) **or** Result `frontier-empty` only — Result `skipped` does **not** close — [../grill.md](../grill.md) · [../derive-next.md](../derive-next.md) · [../solo-entry.md](../solo-entry.md) · [../hitl.md](../hitl.md).

## Round log

### Round 1

❓ **Q1** - **Guest mode when the next page is already locally read**: In Guest mode (`auth_enabled = false`), the server does not know which entries are in local `readSet`/IndexedDB. After marking Page 1 as read on **Unread**, hiding the page, showing `#entries-loading-overlay`, and fetching `#pagination-next-btn` (`offset + limit`), what should happen if all items on that fetched Page 2 are already in `readSet` (e.g., visited earlier) while Page 3 exists?
- **Option A (Recommended):** Fetch only the immediate next page (`offset + limit`), run `syncDomReadStates()`, and display it (if all items on Page 2 are already read, `"All caught up!"` + the `Next` button appear; and when on the last page with no `#pagination-next-btn`, immediately show `"All caught up!"` without showing a loading overlay). One click maps cleanly to at most one request with no hidden request loops.
- **Option B:** Auto-advance across consecutive already-read pages (up to a cap such as 5 pages) while keeping the loading overlay visible until a page with at least one unread item arrives or the last page is reached.

➡️ Yes — fetch the single immediate next page (`#pagination-next-btn`) when present, and immediately show `"All caught up!"` without a loading spinner when no `#pagination-next-btn` exists.
- **User answer:** **Option A** — Fetch single immediate next page (`#pagination-next-btn`), or show `"All caught up!"` immediately if on last page.

---

❓ **Q2** - **Guest mode recovery if the next-page request fails**: In Guest mode, marking read is local while loading the next page is an async `GET /hx/entries` request. If hiding the page + showing the loading overlay triggers `GET /hx/entries` and the network or server returns an error, how should the local read state behave when `#entries-list` becomes visible again?
- **Option A (Recommended):** Defer committing `readSet` / IndexedDB (or roll back the page's IDs on `htmx:responseError` / `htmx:sendError`), restore `#entries-list` visibility, and show an error toast — so the current page remains visible and the user can retry by clicking **Mark This Page as Read** again instead of seeing `"All caught up!"` next to an error toast.
- **Option B:** Commit `readSet` / IndexedDB immediately on click; on fetch error restore `#entries-list` visibility (showing `"All caught up!"`) and show an error toast.

➡️ Yes — defer committing (or roll back) local `readSet` / IndexedDB on fetch error so the current page stays intact and retryable when `#entries-list` is restored.
- **User answer:** **Option A** — Defer/roll back local `readSet`/IndexedDB on fetch error so current page stays intact and retryable.

---

❓ **Q3** - **Auth mode scope (`Has API: no` UI-only vs. backend AI brief on `POST /hx/mark-page-read`)**: In Auth mode (`auth_enabled = true`), `POST /hx/mark-page-read` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1225-L1233) currently passes `ai_service = None`, so the returned next page omits AI headlines and `#page-executive-brief` when `AI_ENABLED=true`. Should we keep this task purely UI/HTMX (`Has API: no`: scroll-to-top reset, `hx-disabled-elt="this"`, empty-page guard, `#active-offset` include, `invalidateFilterCache()`, and Guest next-page loading), or also update `mark_page_read_htmx` on the backend to pass `ai_service` (`Has API: yes`)?
- **Option A (Recommended):** Keep changes strictly in the UI/HTMX layer ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html), resolving `Has API: no`), fixing Guest next-page loading + Auth mode scroll reset, double-click disable, empty guard, `#active-offset` include, and `filterCache` invalidation without touching backend routes.
- **Option B:** Also update `mark_page_read_htmx` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) (and route/state wiring if needed) to pass `ai_service` (`Has API: yes`, requiring API contract review and blocking Gate B).

➡️ Yes — keep changes strictly in the UI/HTMX layer (`Has API: no`) unless you also want backend AI brief wiring included on `POST /hx/mark-page-read`.
- **User answer:** **Option A** — Keep strictly UI/HTMX in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) (`Has API: no`) — Guest next-page loading + Auth scroll/disable/cache fixes.

---

❓ **Q4** - **Guest mode behavior on `Today` / `All` or active client-side search filter**: Should Guest mode auto-hide, show loading, and fetch `#pagination-next-btn` **only** when on the **Unread** filter with no active client-side text filter (`#search-input` empty), while marking visible items read in place on `Today`, `All`, or active search filter (preserving hidden unread rows on the current page)?
- **Option A (Recommended):** Auto-hide + show loading + load next page only on **Unread** when `#search-input` is empty; on `Today`, `All`, or active client-side search, mark visible items read in place and show `"Page marked as read."` toast.
- **Option B:** Always advance to `#pagination-next-btn` after marking read whenever a next page button exists, even on `Today` or while client-side search is hiding rows.

➡️ Yes — auto-hide, show loading, and fetch the next page only on the **Unread** filter without an active client-side search filter; mark visible rows in place otherwise.
- **User answer:** **Option B** — Always advance to `#pagination-next-btn` whenever a next page button exists, even on `Today` or active search.

## Frontier Status

All Round 1 decisions are settled (`Q1=A`, `Q2=A`, `Q3=A`, `Q4=B`). Remaining HTMX lifecycle details (`source: markBtn`, deferred `htmx:beforeSwap` commit, single IndexedDB batch transaction, scroll reset, and request cancellation on tab switch) are technical implementation facts grounded in official HTMX 1.9.12 and IndexedDB specifications with no new user-facing decisions required. Frontier is empty.
