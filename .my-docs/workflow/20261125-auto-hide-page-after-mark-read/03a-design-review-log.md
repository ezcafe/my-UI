Result: clean

# Design Review Log: Auto-Hide Page, Show Loading, and Load Next Page After Mark Page as Read

- **Stage:** `design-verify-phase` (Re-verification after `design-update`)
- **Mode:** simple | **Profile:** lite
- **Surfaces:** UI (`yes`), API (`no` — `skipped`), DB (`no` — `skipped`), auth/PII/schema (`no`)
- **Design & Tasks Reviewed:** [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/03-design.md) (`Result: done`), [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/04-tasks.md) (5 planned tests)

---

## 1. Key Takeaway

Re-verification of [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/04-tasks.md) confirms that all prior Major and Minor findings (**M-01**, **M-02**, **D-01**–**D-03**) are cleanly resolved with zero remaining Critical or Major gaps. The design is ready to proceed.

- **Isolated API Review:** Skipped (`Has API = no`, Notes-skip only — no `03a-api-contract-review.md`).
- **Isolated DB Review:** Skipped (`Has DB = no`, Notes-skip only — no `03a-db-design-review.md`).
- **Grill Status:** Closed (`Result: frontier-empty` in [02b-grill.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/02b-grill.md); Q1=A, Q2=A, Q3=A, Q4=B).
- **Critical / Major Findings:** 0 Critical, 0 Major, 0 Minor.

---

## 2. Prior Findings Resolution Verification

| ID | Prior Severity | Verification Result | Grounded Evidence in `03-design.md` / `04-tasks.md` |
|---|---|---|---|
| **M-01** | **Major** | **Resolved** | Shared `cancelPendingMarkPageRead()` checks `typeof cancelPendingGuestMarkRead === 'function'` (safe in Auth mode with zero `ReferenceError`), triggers `htmx:abort` on `#mark-page-read-btn`, clears `disabled` / `.htmx-request`, and calls `hideEntriesLoadingState()`. Called in `handleFilterClick()` (covering `All` and `filterCache` hits intercepted via `stopImmediatePropagation`), `onSearchSubmit()`, and `htmx:beforeRequest` (covering `.filter-pill` intercepts and non-`mark-page-read-btn` `#entries-list` triggers). `.catch()` guards `hideEntriesLoadingState()` with `if (pendingGuestMarkRead === token)`, and `hideEntriesLoadingState()` clears `.htmx-request` on `#entries-loading-bar`. |
| **M-02** | **Major** | **Resolved** | 1) Guest `htmx:beforeSwap` commit is strictly gated on `target.id === 'entries-list'`, `requestConfig.elt.id === 'mark-page-read-btn'`, `shouldSwap !== false`, `!isError`, `!pendingGuestMarkRead.cancelled`, and matching `xhr`. 2) `filterArticlesClientSide()` checks `#active-status` (`'unread'`) and keeps `.read` rows `display: none` (excluded from `visibleCount`) so re-applying search after `syncDomReadStates()` never reveals just-read rows on `Unread`. `04-tasks.md` includes both the implementation task and `test_guest_filter_articles_client_side_keeps_read_hidden_on_unread`. |
| **D-01** | Minor | **Resolved** | `03-design.md` and `04-tasks.md` place `pendingGuestMarkRead`, `parseVisibleEntryIds()`, `persistLocallyReadIds()`, `cancelPendingGuestMarkRead()`, and Guest HTMX listeners **after `syncDomReadStates()` and before `markLocallyRead()`**, preserving both the `renderLocalAllArticles() .. syncDomReadStates()` slice ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3658-L3673)) and the `markVisiblePageLocallyRead() .. showToast` slice ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3868-L3880)). |
| **D-02** | Minor | **Resolved** | `persistLocallyReadIds()` deduplicates IDs with `new Set(ids.map(Number).filter(id => id > 0 && !readSet.has(id)))` before `recordReadArticle()` and opening a single batched IndexedDB `readwrite` transaction. |
| **D-03** | Minor | **Resolved** | Section 4 of `03-design.md` and Scope of `04-tasks.md` explicitly document keeping `#search-input` text on Guest next-page swap (Q4=B, matching `#pagination-next-btn`) and preserving `#list-executive-brief` above `#client-empty-state` in List view as accepted/out-of-scope behavior. |

---

## 3. Detailed Surface & Lifecycle Check

### 3.1 HTMX 1.9.12 & IndexedDB Technical Verification

- **`htmx.ajax()` with `source: markBtn`:** In HTMX 1.9.12 ([base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L77)), passing `source: markBtn` sets `evt.detail.elt` (`htmx:beforeRequest`) and `evt.detail.requestConfig.elt` (`htmx:beforeSwap` / `htmx:afterSwap` / `htmx:afterRequest`) to `#mark-page-read-btn`, which sits in `<header>` outside `#entries-list` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L56-L65)). Because `#mark-page-read-btn` stays attached during `innerHTML` swaps of `#entries-list`, HTMX attributes (`hx-indicator="#entries-loading-bar"`, `hx-disabled-elt="this"`), OOB swaps (`#unread-pill`, `#filter-unread-count` in [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L233-L238)), and `htmx:abort` targeting behave reliably ([htmx.ajax API](https://htmx.org/api/#ajax), [HTMX events](https://htmx.org/events/)).
- **Deferred Guest Commit & Error Recovery (Q2=A):** `htmx:beforeRequest` captures `evt.detail.xhr`, `htmx:beforeSwap` defers `readSet` / IndexedDB commits to non-error `#mark-page-read-btn` swaps (`!isError && shouldSwap !== false`), and `htmx:afterRequest` / `.catch()` clear stale tokens without mutating local read state on 4xx/5xx or network failures.
- **Batched `IDBTransaction` Auto-Commit:** `persistLocallyReadIds()` opens one `readwrite` transaction on `STORE_NAME` (`read_entries`) and queues all `store.put(...)` calls synchronously inside the `openGuestDb().then(...)` turn, avoiding async gaps that trigger premature `IDBTransaction` auto-commit ([MDN IDBTransaction](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction)).
- **Auth Mode Polish (Q3=A):** Updating `hx-include` to `"#active-status, #active-offset, [name='search']"` cleanly sends `#active-offset` ([entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L1)) to `mark_page_read_htmx` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1163-L1213)) and is safely ignored when absent ([hx-include](https://htmx.org/attributes/hx-include/)). Empty-page prevention in `htmx:beforeRequest`, `scrollTop = 0` resets in `htmx:afterSwap` / `applyCachedList()` / `renderLocalAllArticles()`, and `invalidateFilterCache()` before `filterCache.set()` on `#mark-page-read-btn` swaps close the remaining Auth-mode polish gaps.

### 3.2 UI Craft & Test Plan Check

- **UI Craft:** Reuses `#entries-loading-overlay` and `#entries-loading-bar` with zero layout shift; hides `#entries-magazine-view` via `style.display = 'none'` when `isUnreadFilter && visibleUnreadRowCount === 0 && rows.length > 0` (preserving hiding even after `applyAiViewMode('ai', ...)` removes Tailwind `hidden`); applies native `disabled` via `hx-disabled-elt="this"` with explicit cleanup on empty-guard/abort/error; honors `@media (prefers-reduced-motion: reduce)` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L501-L508).
- **TDD Coverage ([04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-auto-hide-page-after-mark-read/04-tasks.md)):** All 5 planned test cases in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) directly cover Auth attributes & empty guard, scroll reset & `filterCache` invalidation, Guest `htmx.ajax()` + deferred `beforeSwap` commit + cancellation, Magazine empty-state hiding, and read-aware `filterArticlesClientSide()`.

---

## 4. Verification Sources

- HTMX `htmx.ajax()` (`source`, `target`, `swap`): <https://htmx.org/api/#ajax>
- HTMX lifecycle & `htmx:abort` / `htmx:beforeRequest` / `htmx:beforeSwap` / `htmx:afterRequest`: <https://htmx.org/events/>
- HTMX `hx-disabled-elt` & `hx-include`: <https://htmx.org/attributes/hx-disabled-elt/>, <https://htmx.org/attributes/hx-include/>
- MDN `IDBTransaction` auto-commit & error behavior: <https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction>
- Codebase files: [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html), [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html), [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs), [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)

---

## 5. Gate Sign-Off

- **Overall Result:** `clean`
- **Isolated API Result:** `skipped` (`Has API = no`)
- **Isolated DB Result:** `skipped` (`Has DB = no`)
- **Ready for TDD Review / Gate B:** Yes
