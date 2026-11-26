# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

## Adversarial

**Result:** clean

### Round 1 Findings & `fix-review` Verification

1. **[Major — M-01] Fixed — `htmx:beforeRequest` / `cancelPendingMarkPageRead()` no longer strips `#entries-loading-bar` `.htmx-request` from incoming filter, search, or pagination requests (nor leaks it on prevented requests):**
   - **Where:** [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L487-L613)
   - **Verified:** `hideEntriesLoadingState()` (`#L487-L494`) clears `.htmx-request` on `#entries-loading-bar` only when `!activeEntriesRequestXhr`. `htmx:beforeRequest` sets `activeEntriesRequestXhr = evt.detail.xhr || null` before calling `cancelPendingMarkPageRead()` for uncached `.filter-pill` (`#L528-L529`) and non-`mark-page-read-btn` `#entries-list` triggers (`#L553-L556`). Prevented `.filter-pill` (`All` / cache hit at `#L503-L527`) and prevented empty-page `#mark-page-read-btn` (`#L534-L542`) clear `#entries-loading-bar` when `!activeEntriesRequestXhr`.
   - **Guarded by:** `test_entries_loading_bar_preserved_and_stale_xhr_guarded` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs).

2. **[Major — M-02] Fixed — XHR ownership guards prevent aborted/stale `#mark-page-read-btn` requests from hiding newer loading states or firing false cancellations:**
   - **Where:** [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L202-L203), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L582-L613), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1084-L1266), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1797-L1913)
   - **Verified:** `pendingAuthMarkReadXhr` and `pendingGuestMarkRead` ensure `cancelPendingMarkPageRead()` and `cancelPendingGuestMarkRead()` are strict no-ops when no mark-read request is in flight, and both paths share the DOM `htmx:abort` / button reset / overlay cleanup inside `cancelPendingMarkPageRead()`. `clearCurrentEntriesLoadingState(xhr)` gates cleanup across `htmx:afterSwap`, `htmx:afterRequest`, `htmx:responseError`, `htmx:sendError`, and Guest `.catch()` against `activeEntriesRequestXhr`.
   - **Guarded by:** `test_entries_loading_bar_preserved_and_stale_xhr_guarded` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs).

### Verified Adversarial Checks (Auth & Guest Mode)
- **Guest next-page fetch (`Q1=A`, `Q4=B`):** `parseVisibleEntryIds()` handles comma-separated string output from `getVisibleEntryIds()` and empty-page early returns; when `#pagination-next-btn` (`hx-get`) exists, `markVisiblePageLocallyRead()` uses `htmx.ajax('GET', nextUrl, { target: '#entries-list', swap: 'innerHTML', source: markBtn })` (outside `#entries-list`) so OOB badges (`#unread-pill`, `#filter-unread-count`), `hx-indicator`, `hx-disabled-elt="this"`, and `htmx:abort` remain attached.
- **Deferred Guest commit & error rollback (`Q2=A`):** `htmx:beforeRequest` captures `pendingGuestMarkRead.xhr`; `htmx:beforeSwap` commits `persistLocallyReadIds(token.ids, { syncDom: false })` only when `target.id === 'entries-list'`, `requestConfig.elt.id === 'mark-page-read-btn'`, `shouldSwap !== false`, `!evt.detail.isError`, `!cancelled`, and `matchesXhr`. On 4xx/5xx or network error, `readSet` and IndexedDB remain untouched and Page 1 stays retryable.
- **Last page / Magazine / Read-aware client-side search:** When no `#pagination-next-btn` exists, `persistLocallyReadIds(ids, { syncDom: true })` runs immediately, hides `.annuaire-row.read` and `#entries-magazine-view` on `Unread` (`style.display = 'none'`, restored when unread count > 0), and re-runs `filterArticlesClientSide(searchQuery)` when `!isSearchActive && searchQuery`, keeping `.read` rows hidden on `Unread`.
- **Auth mode polish (`Q3=A`):** `#mark-page-read-btn` sets `hx-disabled-elt="this"` and `hx-include="#active-status, #active-offset, [name='search']"`, aborts empty pages in `htmx:beforeRequest` before `showEntriesLoadingState()`, resets `scrollTop = 0` across `htmx:afterSwap`, `applyCachedList()`, and `renderLocalAllArticles()`, and calls `invalidateFilterCache()` on `#mark-page-read-btn` swaps before re-caching.

---

## Quality

**Result:** clean

### Five-Axis Summary
1. **Correctness:** Re-verified after `fix-review`: `cargo test -p ui-components` (83 passed, 0 failed), `cargo test -p server` (140 lib unit tests passed + 8 `e2e_tests` passed), `cargo clippy --all-targets` (0 warnings), `cargo fmt --check` (clean).
2. **Readability:** Small single-purpose helpers (`isCurrentEntriesRequest`, `clearCurrentEntriesLoadingState`, `parseVisibleEntryIds`, `persistLocallyReadIds`, `cancelPendingGuestMarkRead`) with shallow nesting and unified abort/cleanup in `cancelPendingMarkPageRead()`.
3. **Architecture:** Strictly UI/HTMX in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) and template tests in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) (`Has API: no`, `Has DB: no`), preserving all existing slice boundaries (`renderLocalAllArticles() .. syncDomReadStates()` and `markVisiblePageLocallyRead() .. showToast()`).
4. **Security:** Static toast copy (`Page marked as read.`, error messages), Askama HTML escaping unchanged, no unsafe innerHTML interpolation of user input.
5. **Performance:** Deduplicates `!readSet.has(id)` into a single synchronous `readwrite` `IDBTransaction` batch; zero redundant `syncDomReadStates()` or DOM re-renders on page transitions.

### UI Craft Check (`Has UI = yes`)
- Zero layout shift using absolute `#entries-loading-overlay` and `#entries-loading-bar`; native `disabled` attribute via `hx-disabled-elt="this"` with explicit cleanup on abort/error/empty guard; Magazine view hides cleanly (`style.display = 'none'`) when Unread reaches 0 visible items; honors `@media (prefers-reduced-motion: reduce)` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L501-L508).

### Verification Sources (Live Web Search Grounded)
- HTMX `htmx.ajax()` API & `source` option (`htmx.org@1.9.12`): <https://htmx.org/api/#ajax>
- HTMX lifecycle events (`htmx:beforeRequest`, `htmx:beforeSwap`, `htmx:afterSwap`, `htmx:afterRequest`, `htmx:responseError`, `htmx:sendError`, `htmx:abort`) & `hx-disabled-elt`: <https://htmx.org/events/>, <https://htmx.org/attributes/hx-disabled-elt/>
- MDN `IDBTransaction` active lifecycle & auto-commit semantics: <https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction>

---

## Merged

**Result:** skipped  
(`Lens plan: none`; `Has API = no`, `Has DB = no`)
