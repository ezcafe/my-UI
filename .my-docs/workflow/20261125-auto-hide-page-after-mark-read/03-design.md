# Design — Auto-Hide Page, Show Loading, and Load Next Page After Mark Page as Read

**Result:** done

## Key Takeaway

Keep all changes in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) (`Has UI: yes`, `Has API: no`, `Has DB: no`, `Has auth/PII/schema: no`) and route Guest mode's next-page fetch through `htmx.ajax()` using `#mark-page-read-btn` as `source`, committing deduplicated unread IDs to `readSet` / IndexedDB in `htmx:beforeSwap` only for non-error `#mark-page-read-btn` swaps.

- **Guest mode (`auth_enabled = false`):** Fix visible ID parsing (`getVisibleEntryIds()` returns a comma-separated string), guard empty pages, immediately hide `#entries-list` and show `#entries-loading-overlay` when `#pagination-next-btn` exists (regardless of filter tab or active client-side search), defer committing deduplicated unread IDs (`!readSet.has(id)`) until a matching `#mark-page-read-btn` `htmx:beforeSwap` succeeds, keep `.read` rows hidden in `filterArticlesClientSide()` when on `Unread`, and show `"All caught up!"` right away with no spinner when on the last page.
- **Auth mode (`auth_enabled = true`):** Add `hx-disabled-elt="this"`, include `#active-offset`, guard empty pages before `showEntriesLoadingState()`, clear stale tabs in `filterCache`, reset `#entries-list.scrollTop = 0` on swap, and support clean cancellation via shared `cancelPendingMarkPageRead()`.
- **UI & Craft:** Reuse `#entries-loading-overlay` and `#entries-loading-bar` with zero layout shift, hide `#entries-magazine-view` when Unread has 0 remaining items so `"All caught up!"` renders cleanly in both List and Magazine views, and abort in-flight mark-read requests across all filter/search transitions (including intercepted `All` and `filterCache` clicks).

---

## Design Options

### Option 1 (Recommended): HTMX-Native Lifecycle via `htmx.ajax(..., { source: markBtn })` and Deferred `htmx:beforeSwap` Commit

- **Explanation:**
  Use HTMX 1.9.12's `htmx.ajax()` API whenever `#pagination-next-btn` (`hx-get`) exists in Guest mode, passing `source: document.getElementById('mark-page-read-btn')` (which lives in `<header>`, outside `#entries-list`). Because `#mark-page-read-btn` carries `hx-indicator="#entries-loading-bar"` and `hx-disabled-elt="this"`, HTMX automatically disables the button, activates `#entries-loading-bar`, triggers `htmx:beforeRequest` (`closeDetailPanel()` + `showEntriesLoadingState()`), handles OOB swaps (`#unread-pill`, `#filter-unread-count` in [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L233-L238)), and fires `htmx:responseError` / `htmx:sendError` on failures.
  Local `readSet`, `readSinceLastBriefSet`, `filterCache` invalidation, and a single batched IndexedDB `readwrite` transaction are **deferred until `htmx:beforeSwap`** and guarded strictly by `target.id === 'entries-list'`, `requestConfig?.elt?.id === 'mark-page-read-btn'`, `shouldSwap !== false`, `!isError`, `!token.cancelled`, and matching `xhr` (in HTMX 1.9.12, `htmx.ajax()` can resolve on HTTP 4xx/5xx, so gating on `htmx:beforeSwap` prevents committing on errors or unrelated swaps).
  To prevent stuck overlays or late swaps when switching tabs (including `All` and `filterCache` hits intercepted before `htmx:beforeRequest`), a shared `cancelPendingMarkPageRead()` helper runs in `handleFilterClick()`, `onSearchSubmit()`, and `htmx:beforeRequest` (for non-`mark-page-read-btn` `#entries-list` triggers).

- **Concrete example:**

```html
<!-- Guest mode button in crates/ui-components/templates/pages/index.html -->
<button
  id="mark-page-read-btn"
  type="button"
  hx-indicator="#entries-loading-bar"
  hx-disabled-elt="this"
  onclick="markVisiblePageLocallyRead()"
  class="p-2 rounded-full border border-border bg-card hover:bg-muted text-foreground transition-colors"
  title="Mark This Page as Read">
  <i data-lucide="check-check" class="w-4 h-4"></i>
</button>
```

```javascript
// Shared <script> cancellation helper (Auth + Guest safe; no ReferenceError in Auth mode)
function cancelPendingMarkPageRead() {
  if (typeof cancelPendingGuestMarkRead === 'function') {
    cancelPendingGuestMarkRead();
    return;
  }
  const markBtn = document.getElementById('mark-page-read-btn');
  if (markBtn) {
    if (window.htmx && typeof htmx.trigger === 'function') {
      htmx.trigger(markBtn, 'htmx:abort');
    }
    markBtn.disabled = false;
    markBtn.classList.remove('htmx-request');
  }
  hideEntriesLoadingState();
}
```

```javascript
// Guest <script> block (placed AFTER syncDomReadStates() and BEFORE markLocallyRead())
let pendingGuestMarkRead = null;

function parseVisibleEntryIds() {
  const raw = (typeof getVisibleEntryIds === 'function') ? getVisibleEntryIds() : '';
  if (Array.isArray(raw)) return raw.map(Number).filter(Boolean);
  return String(raw).split(',').map(s => Number(s.trim())).filter(Boolean);
}

function persistLocallyReadIds(ids, { syncDom = true } = {}) {
  const numericIds = Array.from(
    new Set(ids.map(Number).filter(id => id > 0 && !readSet.has(id)))
  );
  if (numericIds.length === 0) return;

  numericIds.forEach(numId => {
    if (typeof recordReadArticle === 'function') recordReadArticle(numId);
    readSet.add(numId);
  });
  if (typeof invalidateFilterCache === 'function') invalidateFilterCache();
  if (syncDom) {
    syncDomReadStates();
    const searchInput = document.getElementById('search-input');
    const searchQuery = (searchInput && searchInput.value) ? searchInput.value.trim() : '';
    if (!isSearchActive && searchQuery && typeof filterArticlesClientSide === 'function') {
      filterArticlesClientSide(searchQuery);
    }
  }

  openGuestDb()
    .then((db) => {
      if (!db.objectStoreNames.contains(STORE_NAME)) return;
      const now = Date.now();
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      numericIds.forEach(numId => store.put({ id: numId, read_at: now }));
    })
    .catch(() => {});
}

function cancelPendingGuestMarkRead() {
  if (pendingGuestMarkRead) {
    pendingGuestMarkRead.cancelled = true;
    pendingGuestMarkRead = null;
  }
  const markBtn = document.getElementById('mark-page-read-btn');
  if (markBtn) {
    if (window.htmx && typeof htmx.trigger === 'function') {
      htmx.trigger(markBtn, 'htmx:abort');
    }
    markBtn.disabled = false;
    markBtn.classList.remove('htmx-request');
  }
  hideEntriesLoadingState();
}

document.addEventListener('htmx:beforeRequest', (evt) => {
  const trigger = evt.detail && evt.detail.elt;
  if (!evt.defaultPrevented && trigger && trigger.id === 'mark-page-read-btn' && pendingGuestMarkRead && !pendingGuestMarkRead.xhr) {
    pendingGuestMarkRead.xhr = evt.detail.xhr;
  }
});

document.addEventListener('htmx:beforeSwap', (evt) => {
  if (!pendingGuestMarkRead || pendingGuestMarkRead.cancelled || !evt.detail) return;
  const target = evt.detail.target;
  const triggerElt = evt.detail.requestConfig && evt.detail.requestConfig.elt;
  const matchesXhr = !pendingGuestMarkRead.xhr || pendingGuestMarkRead.xhr === evt.detail.xhr;
  if (
    target &&
    target.id === 'entries-list' &&
    triggerElt &&
    triggerElt.id === 'mark-page-read-btn' &&
    evt.detail.shouldSwap !== false &&
    !evt.detail.isError &&
    matchesXhr
  ) {
    const token = pendingGuestMarkRead;
    token.committed = true;
    pendingGuestMarkRead = null;
    persistLocallyReadIds(token.ids, { syncDom: false });
  }
});

document.addEventListener('htmx:afterRequest', (evt) => {
  if (pendingGuestMarkRead && evt.detail && evt.detail.xhr === pendingGuestMarkRead.xhr) {
    pendingGuestMarkRead = null;
  }
});

// Existing markLocallyRead(id) ... unmarkLocallyRead(id) ... toggleLocalReadStatus(id) ...
// Placed immediately before showToast() to preserve existing template slice assertions:
function markVisiblePageLocallyRead() {
  if (typeof closeDetailPanel === 'function') closeDetailPanel();
  if (pendingGuestMarkRead) return;

  const ids = parseVisibleEntryIds();
  if (ids.length === 0) return;

  const nextBtn = document.getElementById('pagination-next-btn');
  const nextUrl = nextBtn ? nextBtn.getAttribute('hx-get') : null;
  const markBtn = document.getElementById('mark-page-read-btn');

  if (nextUrl && window.htmx && markBtn) {
    const token = { ids, committed: false, cancelled: false, xhr: null };
    pendingGuestMarkRead = token;
    htmx.ajax('GET', nextUrl, {
      target: '#entries-list',
      swap: 'innerHTML',
      source: markBtn
    }).then(() => {
      if (pendingGuestMarkRead === token) pendingGuestMarkRead = null;
      if (token.committed && !token.cancelled) {
        showToast('Page marked as read.');
      }
    }).catch(() => {
      if (pendingGuestMarkRead === token) {
        pendingGuestMarkRead = null;
        hideEntriesLoadingState();
      }
    });
    return;
  }

  persistLocallyReadIds(ids, { syncDom: true });
  showToast('Page marked as read.');
}
```

- **Pros:**
  - Single unified loading, error-toast, OOB-badge, and swap pipeline shared with Auth mode (`htmx:beforeRequest`, `htmx:afterSwap`, `htmx:afterRequest`, `htmx:responseError`, `htmx:sendError`).
  - Deferring until `#mark-page-read-btn` non-error `htmx:beforeSwap` makes **Q2=A** rollback atomic and prevents false commits from unrelated `#entries-list` swaps.
  - Using `#mark-page-read-btn` (outside `#entries-list`) as `source` prevents HTMX 1.9.12 detached-source cleanup issues.
  - Single IndexedDB `readwrite` transaction for deduplicated unread IDs (`!readSet.has(id)`) + single `syncDomReadStates()` call avoids redundant writes on `Today` / `All`.
  - Explicit cancellation in `handleFilterClick()`, `onSearchSubmit()`, and `htmx:beforeRequest` protects `All` and `filterCache` intercepts as well as uncached requests, with zero `ReferenceError` in Auth mode.
- **Cons:**
  - Splits Guest mark-read completion across `markVisiblePageLocallyRead()` and `htmx:beforeSwap` / `htmx:afterRequest`, requiring token/XHR correlation (`token.xhr`, `token.cancelled`) so aborted promises never clear a newer request's loading overlay.

---

### Option 2: Optimistic Local Commit + Snapshot Rollback via Native `fetch()`

- **Explanation:**
  Write `ids` into `readSet` and IndexedDB immediately on click while recording a `previousReadIds` snapshot, call `fetch(nextUrl, { headers: { 'HX-Request': 'true' } })` with an `AbortController`, swap `#entries-list.innerHTML` manually and run `htmx.process()`, and delete newly added IDs from `readSet` and IndexedDB if `fetch()` throws or returns `!response.ok`.

- **Concrete example:**

```javascript
async function markVisiblePageLocallyRead() {
  closeDetailPanel();
  const ids = parseVisibleEntryIds().filter(id => !readSet.has(id));
  if (!ids.length) return;
  const nextUrl = document.getElementById('pagination-next-btn')?.getAttribute('hx-get');
  if (!nextUrl) { persistLocallyReadIds(ids); showToast('Page marked as read.'); return; }

  showEntriesLoadingState();
  persistLocallyReadIds(ids, { syncDom: false });
  try {
    const res = await fetch(nextUrl, { headers: { 'HX-Request': 'true' } });
    if (!res.ok) throw new Error('HTTP ' + res.status);
    document.getElementById('entries-list').innerHTML = await res.text();
    htmx.process(document.getElementById('entries-list'));
    showToast('Page marked as read.');
  } catch (err) {
    rollbackLocallyReadIds(ids); // delete from readSet + IndexedDB
    showToast('Failed to load articles. Please try again.');
  } finally {
    hideEntriesLoadingState();
  }
}
```

- **Pros:**
  - Keeps async flow self-contained in one `async/await` block with `AbortController`.
  - No dependency on `htmx:beforeSwap` timing.
- **Cons:**
  - Bypasses HTMX OOB swaps (`#unread-pill` and `#filter-unread-count` would fail to swap or appear inline inside `#entries-list` unless manually parsed and extracted).
  - Duplicates `htmx:afterSwap` responsibilities (Executive Brief rendering, `hydrateGuestEntriesFromDom()`, `cacheLoadedArticlesFromDom()`, `applyAiViewMode()`, `filterCache` updates, Lucide icons, error toasts).
  - Optimistic IndexedDB writes before fetch require compensating `delete()` calls on error, risking stale local read state if the user navigates away before rollback completes.

---

### Recommendation

**Option 1 is recommended.** It keeps OOB unread counts, Magazine/List mode switching, Executive Brief initialization, loading bar/overlay transitions, and error handling inside one single HTMX lifecycle while guaranteeing atomic state recovery on error, safe tab-switch cancellation, and clean Auth/Guest separation.

---

## Detailed Behavior & Edge Cases

### 1. Shared HTMX & UI State Helpers ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html))

- **Auth `#mark-page-read-btn` (`{% if auth_enabled %}`):**
  - Add `hx-disabled-elt="this"` to disable the button while `POST /hx/mark-page-read` is in flight.
  - Update `hx-include` from `"#active-status, [name='search']"` to `"#active-status, #active-offset, [name='search']"` so non-`unread` tabs (`today`, `starred`, `all`) advance from the current offset instead of always defaulting to `0`.
- **Guest `#mark-page-read-btn` (`{% else %}`):**
  - Add `hx-indicator="#entries-loading-bar"` and `hx-disabled-elt="this"` so `htmx.ajax(..., { source: markBtn })` activates the top loading bar and disables the button automatically during fetch.
- **`hideEntriesLoadingState()`:**
  - Remove `invisible` from `#entries-list`, add `hidden` to `#entries-loading-overlay`, and remove `htmx-request` from `#entries-loading-bar` so the indicator never sticks after swap or abort.
- **Unified Cancellation (`cancelPendingMarkPageRead()`):**
  - Defined in the shared `<script>`: calls `cancelPendingGuestMarkRead()` when `typeof cancelPendingGuestMarkRead === 'function'` (Guest mode), and otherwise aborts `#mark-page-read-btn` via `htmx.trigger(markBtn, 'htmx:abort')`, explicitly resets `markBtn.disabled = false` and removes `.htmx-request`, and calls `hideEntriesLoadingState()`. Safe in Auth mode with zero `ReferenceError`.
  - Called in:
    1. `handleFilterClick(btn, event)` right after `closeDetailPanel()` — covers `All` (`renderLocalAllArticles()`), `filterCache.has(status)` (`applyCachedList()`), and uncached filter transitions before any request starts.
    2. `onSearchSubmit(event)` right after `closeDetailPanel()`.
    3. `htmx:beforeRequest` inside the `.filter-pill` intercept and for any non-`mark-page-read-btn` trigger targeting `#entries-list` (before `showEntriesLoadingState()` runs for pagination buttons).
- **Empty-page guard & button reset (`htmx:beforeRequest` / `htmx:afterRequest`):**
  - In `htmx:beforeRequest`, before calling `showEntriesLoadingState()` when `trigger.id === 'mark-page-read-btn'`, check `getVisibleEntryIds()`. If empty (`""` or empty array), call `evt.preventDefault()`, reset `trigger.disabled = false` and `trigger.classList.remove('htmx-request')`, and return early so no network request or empty loading spinner fires.
  - In `htmx:afterRequest` for `target.id === 'entries-list'`, after `hideEntriesLoadingState()`, explicitly reset `triggerElt.disabled = false` and `triggerElt.classList.remove('htmx-request')` when `triggerElt?.id === 'mark-page-read-btn'` so HTMX 1.9.12 abort/error paths never leave the button disabled.
- **Scroll reset & cache invalidation:**
  - In `htmx:afterSwap` (`isEntriesSwap`), reset `evt.detail.target.scrollTop = 0`. Also reset `entriesList.scrollTop = 0` in `applyCachedList()` and `list.scrollTop = 0` in `renderLocalAllArticles()`.
  - When `triggerElt && triggerElt.id === 'mark-page-read-btn'` swaps `#entries-list` (covers Auth mode; Guest mode also clears in `htmx:beforeSwap`), call `invalidateFilterCache()` **before** `filterCache.set(status, ...)` so stale `today`, `all`, and `starred` tabs are cleared while the newly returned active page is cached cleanly.
- **Read-aware `filterArticlesClientSide(query)`:**
  - Check `const isUnreadFilter = Boolean(activeStatusInput && activeStatusInput.value === 'unread')`.
  - Skip un-hiding `.read` items when `isUnreadFilter` is true: force `item.style.display = 'none'` and do not increment `visibleCount` so `.read` rows never reappear when filtering or re-applying search on `Unread`.

### 2. Guest Mode Flow Matrix (Grounded in Q1–Q4)

| Scenario | Trigger Behavior | State Commit | UI Result |
|---|---|---|---|
| `#pagination-next-btn` exists (on `Unread`, `Today`, or active client-side search — **Q4=B**) | `closeDetailPanel()`, `showEntriesLoadingState()`, disable button (`hx-disabled-elt="this"`), `htmx.ajax('GET', nextUrl, { target: '#entries-list', swap: 'innerHTML', source: markBtn })` | Deferred until `#mark-page-read-btn` non-error `htmx:beforeSwap` (`!isError && shouldSwap !== false && !cancelled && matchesXhr`); deduplicates `!readSet.has(id)` into one `readwrite` IndexedDB tx | Next page swaps in, `scrollTop = 0`, `hideEntriesLoadingState()`, `syncDomReadStates()`, toast `"Page marked as read."` |
| Next page (`Page 2`) returned, but all items already in `readSet` (**Q1=A**) | Same single `htmx.ajax` fetch (`offset + limit`) | Same deferred commit for Page 1 unread IDs | `syncDomReadStates()` hides read rows + `#entries-magazine-view`, renders `#client-empty-state` (`"All caught up!"`) + keeps `#pagination-next-btn` visible if Page 3 exists |
| On last page (no `#pagination-next-btn`, or `All` local tab, or `htmx` missing) (**Q1=A**) | No loading overlay shown | Immediate `persistLocallyReadIds(ids, { syncDom: true })` (deduplicated `!readSet.has(id)`) | On `Unread`: rows + `#entries-magazine-view` hide immediately, `"All caught up!"` shown; on `Today`/`All`: rows mark `.read` in place; if `!isSearchActive && searchQuery`, re-applies `filterArticlesClientSide(searchQuery)` (which keeps `.read` rows `display: none` on `Unread`); toast `"Page marked as read."` |
| HTTP 4xx/5xx (`htmx:responseError`) or network failure (`htmx:sendError`) (**Q2=A**) | Fetch fails; `htmx:beforeSwap` does not commit (`isError === true` or not reached) | `pendingGuestMarkRead` cleared, `readSet` and IndexedDB untouched | `hideEntriesLoadingState()` restores Page 1 intact & retryable, error toast shown, button re-enabled, no success toast |
| 0 visible entry IDs on page | Early return | None | No spinner, no request, no toast |
| User clicks filter pill (incl. `All` or cached tab) or submits search while `markBtn` fetch is in flight | `cancelPendingMarkPageRead()` -> `cancelPendingGuestMarkRead()` marks `cancelled = true`, clears `pendingGuestMarkRead`, triggers `htmx:abort`, resets `markBtn.disabled = false` / `.htmx-request`, calls `hideEntriesLoadingState()` | Aborted request does not commit `readSet`/IndexedDB; stale `.catch()` guarded by `pendingGuestMarkRead === token` | New filter tab or search result wins cleanly without stuck overlay or late swap overwrite |

### 3. Helper Placement, `markLocallyRead(id)`, and `syncDomReadStates()` Polish

- **Script placement constraint (D-01):** Insert `pendingGuestMarkRead`, `parseVisibleEntryIds()`, `persistLocallyReadIds()`, `cancelPendingGuestMarkRead()`, and the Guest HTMX lifecycle listeners **after `syncDomReadStates()` and before `markLocallyRead()`** — never between `renderLocalAllArticles()` and `syncDomReadStates()`, because [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3658-L3673) slices from `function renderLocalAllArticles()` to `function syncDomReadStates()` and asserts on that boundary. Keep `markVisiblePageLocallyRead()` immediately before `showToast()` per `test_paging_loading_overlay_and_read_blur_removed` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3868-L3880)).
- **`markLocallyRead(id)` reuse:** Delegate `markLocallyRead(id)` to `persistLocallyReadIds([numId], { syncDom: true })` so single-article and page-level read persistence share one deduplicated IndexedDB, cache invalidation, and read-aware search re-filter path.
- **Magazine empty-state hiding:** In `syncDomReadStates()`, when `isUnreadFilter && visibleUnreadRowCount === 0 && rows.length > 0`, set `magView.style.display = 'none'` on `#entries-magazine-view` (and reset `magView.style.display = ''` otherwise) so Magazine view shows `"All caught up!"` cleanly on the last Unread page without leftover greyed-out cards above the empty state.
- **Read-aware search re-filter:** When `persistLocallyReadIds(..., { syncDom: true })` runs synchronously (last page / no swap), if `!isSearchActive` and `#search-input` has a non-empty value, re-run `filterArticlesClientSide(searchQuery)` after `syncDomReadStates()`. Because `filterArticlesClientSide()` skips `.read` rows when `#active-status.value === 'unread'`, newly read rows stay hidden on `Unread` while non-matching unread rows stay filtered out, and matching `.read` rows remain visible on `Today`/`All`.

### 4. Accepted / Out-of-Scope Behavior (D-03)

- **Guest `#search-input` text after next-page swap:** Per **Q4=B**, when Guest mode has a non-empty client-side filter in `#search-input` (`!isSearchActive`) and `#pagination-next-btn` is present, clicking `#mark-page-read-btn` swaps in the server's next page while leaving `#search-input` unchanged (matching existing `#pagination-next-btn` behavior; `htmx:afterSwap` also skips `filterCache.set` while `searchQuery` is non-empty).
- **List view `#list-executive-brief` on empty `Unread`:** When `syncDomReadStates()` hides all `.annuaire-row` items on `Unread` in List view, `#list-executive-brief` remains visible above `#client-empty-state`, matching existing `syncDomReadStates()` behavior and out of scope for this change.

### 5. UI Craft Notes

- **Feedback & Perceived Latency:** Immediate visual transition from click (`#entries-list` hidden with `.invisible`, `#entries-loading-overlay` spinner centered, `#entries-loading-bar` top pulse) prevents double-reading old titles while loading.
- **No Layout Shift:** `#entries-loading-overlay` and `#entries-loading-bar` are already positioned `absolute` inside `.workspace-layout` (`position: relative`), keeping container dimensions stable.
- **Accessibility & Motion:** `hx-disabled-elt="this"` applies the native `disabled` attribute to `#mark-page-read-btn` during requests (explicitly cleared on abort/error/empty guard); `#entries-loading-overlay` uses `aria-hidden="true"` on the spinner SVG and readable `"Loading..."` status text; `@media (prefers-reduced-motion: reduce)` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L501-L508) remains honored.

---

## Verification Sources

- HTMX `htmx.ajax()` API & `source` option: <https://htmx.org/api/#ajax>
- HTMX lifecycle events (`htmx:beforeRequest`, `htmx:beforeSwap`, `htmx:afterSwap`, `htmx:afterRequest`, `htmx:responseError`, `htmx:sendError`, `htmx:abort`): <https://htmx.org/events/>
- HTMX `hx-disabled-elt` & `hx-include`: <https://htmx.org/attributes/hx-disabled-elt/>, <https://htmx.org/attributes/hx-include/>
- MDN `IDBTransaction` & auto-commit semantics: <https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction>
- Codebase references: [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html), [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs), [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
