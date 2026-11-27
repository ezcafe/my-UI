# Design — Reading Flow & Triaging Pack

**Result:** done

## Design Options

### Option A: Dedicated Batch Undo Endpoint (`POST /hx/mark-page-unread`) + Stretched-Link Cards & Page-Scoped Navigation (Recommended)

- **Explanation:**
  1. **Magazine Full-Card Click (Stretched Link):** Make `.magazine-card` (`position: relative; cursor: pointer`) clickable across its full surface in P1, P2, and P3 by placing `<a id="mag-title-{{ entry.id }}" href="/entries/{{ entry.id }}" class="magazine-card-link ...">` inside each headline (`<h3>`/`<h4>`/`<h5>`) and removing `id`/`onclick` from the outer heading tag. Its `::after` pseudo-element spans `inset: 0` with `z-index: 1` ([MDN ::after](https://developer.mozilla.org/en-US/docs/Web/CSS/::after)), while `.magazine-card-link` (`color: inherit; text-decoration: none`), its parent heading (`line-clamp-*`), and `.hero-content-box` / P2 wrappers remain `position: static` so `::after` fills the entire card rather than clipping to the heading. Moving `id="mag-title-{{ entry.id }}"` onto `<a class="magazine-card-link">` and updating `syncExecutiveBriefUpdates()` to write `(magTitleEl.querySelector('.magazine-card-link') || magTitleEl).textContent = suggestedTitle` ensures `/hx/executive-brief` swaps never destroy the anchor link. Normal clicks read `this.closest('.magazine-card')?.dataset.feedTitle || ''` instead of inline `'{{ entry.feed_title }}'` interpolation (preventing apostrophe JS syntax errors) and call `openEntryFromMagazine(...)`, while modified clicks open `/entries/{id}`. Keyboard focus shows an outer outline on `.magazine-card:has(.magazine-card-link:focus-visible)` so `overflow-hidden` never clips the focus ring.
  2. **Detail Prev / Next Controls:** Place `<button id="detail-prev-btn" type="button">` and `<button id="detail-next-btn" type="button">` next to `<button id="detail-back-btn" type="button">` in the `h-8` header bar using `gap-1.5` on both header button clusters so 320px mobile viewports fit without overflow across [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html), `openDetailWithSkeleton()`, and `renderGuestEntryDetail()`. Maintain a page-scoped ordered `detailNavIds` array built from visible `#entries-list .annuaire-row` elements when lists load, cache-swap, filter, or search—and *not* rebuilt when `markLocallyRead()` hides a row opened during the current session in Guest Unread mode, so `Prev` stays functional. When `entryId` is absent from `detailNavIds` (`indexOf === -1`, such as a deep link `/entries/{id}`), disable both `#detail-prev-btn` and `#detail-next-btn`. Bump Auth detail IndexedDB `ARTICLE_DB_NAME` from v2 to v3 and guard `isCacheValid` with `cached.html.includes('id="detail-prev-btn"')` so stale cached detail HTML refreshes cleanly.
  3. **`Feed · Relative time` Metadata:** Move `chrono = "0.4"` from `[dev-dependencies]` to `[dependencies]` in [crates/ui-components/Cargo.toml](file:///Users/ptquang86/ws/my-ui/crates/ui-components/Cargo.toml) and add `format_relative_time(raw: &str)` / `EntryViewModel::relative_published_at(&self)` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) alongside matching JS `formatRelativeTimeLabel(raw)` + `refreshRelativeTimes(root)` placed in the **shared `<script>` block** above `{% if !auth_enabled %}` in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) so both Auth and Guest modes run them. Guard `" · "` on *both* `feed_title` and `published_at` being non-empty in [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html), [entry_item.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_item.html), [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html), `renderLocalAllArticles()`, and `renderGuestEntryDetail()`. Keep `#entry-{{ entry.id }}-dot` outside the vertical `<div class="flex flex-col min-w-0 flex-1 gap-0.5">` title+meta wrapper so the unread dot stays aligned beside the title and OOB `#entry-title-{id}` swaps preserve `.entry-meta`. Extend `GuestEntryPayload`, `buildGuestArticleRecord`, `syncGuestEntriesScript`, and `cacheLoadedArticlesFromDom` with `feed_title` and `published_at`, and keep Magazine cards free of visible source/time text.
  4. **Undo Toast on "Mark This Page as Read":**
     - **Auth mode:** Keep `#mark-page-read-btn` passing `entry_ids: getVisibleEntryIds()` (preserving the empty-page guard) plus `unread_entry_ids: getVisibleUnreadEntryIds()` (visible `.annuaire-row:not(.read)` only) to `POST /hx/mark-page-read`. In `mark_page_read_htmx`, parse and validate positive `i64` IDs, mutate `updateEntriesStatus(..., READ)` and attach `ToastUndoAction` *only* for `undo_ids` (`unread_entry_ids` when supplied, else `entry_ids`; when empty on all-read `All`/`Today`/`Starred` pages, skip the mutation and omit `undo`). Serialize `ToastUndoPayload` with comma-joined `entry_ids` and `#[serde(skip_serializing_if = "Option::is_none")]` optional fields (`status`, non-empty `search`, `feed_id`, `category_id`, plus clamped `offset`) so `axum::extract::Form<MarkPageReadForm>` (`serde_urlencoded`) never receives JSON arrays or literal `"null"`. Add `mark_page_unread_htmx` at `POST /hx/mark-page-unread` with strict `i64` ID validation (`400 Bad Request` on malformed IDs), empty-ID guard, `"entry:{id}"` cache invalidation in both `mark_page_read_htmx` and `mark_page_unread_htmx`, `ai_service` + `cache` parity in `render_entries_list_html`, and `target_offset` bounds clamping against `total`. On the client, inspect `HX-Reswap` in `base.html` `htmx:afterRequest` so `200 OK` + `HX-Reswap: none` upstream error responses ([error.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L52-L61)) keep the Undo toast alive for retry, share `showClientToast(message, isError = false)` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) for both Auth and Guest modes, and check `.toast-undo-btn` first with early `return;` in `htmx:responseError` / `htmx:sendError` so `#entries-list` error toasts never fire on Undo failures.
     - **Guest mode:** In `markVisiblePageLocallyRead()`, snapshot `newlyReadIds` (`!readSet.has(id)`), `status`, `offset` (`#active-offset`), `search`, and `advancedPage` (`Boolean(nextUrl && window.htmx && markBtn)`). Only show `showUndoToast` when `newlyReadIds.length > 0` (else `showToast('Page marked as read.')`). Clicking **Undo** disables the button, clears its timer, removes the Undo toast, calls `undoGuestMarkPageRead(snapshot)` to delete `snapshot.ids` from `readSet` **synchronously** before restoring (preventing `syncDomReadStates()` from auto-advancing), deletes them from IndexedDB `read_entries`, invalidates `filterCache`, restores the prior page (fetching `/hx/entries?...` via `htmx.ajax` when `snapshot.advancedPage`, or calling `syncDomReadStates()` + `syncDetailNavigationIds()` when `!snapshot.advancedPage`), and shows `"Mark as read undone."`.
     - **Toast accessibility & timing:** Render `role="status" aria-atomic="true"` on all toasts ([toast.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html), `showClientToast`, `showToast`, `showUndoToast`) without stealing focus ([WAI-ARIA Alert/Status Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/alert/)). Use `.toast-undo-btn` (not duplicate fixed IDs), deduplicate existing Undo toasts before adding a new one, escape client toast text while keeping `<span>` as the first `<span>` for text deduplication, use `5000`ms on standard OOB toasts and `8000`ms on Undo toasts paused on `mouseenter`, `focusin`, or in-flight HTMX request ([WCAG 2.2 SC 2.2.1](https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable.html)), and remove the Auth Undo toast only when `htmx:afterRequest` succeeds with `HX-Reswap !== 'none'`.
- **Pros:**
  - Auth-mode Undo performs 1 GraphQL mutation instead of up to 50 parallel requests, staying inside `RateLimiter::default()` (`60` burst capacity in [rate_limit.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/rate_limit.rs#L75-L79)) and preventing partial-revert failures or over-reversion of already-read entries.
  - Strict `serde_urlencoded`-compatible `hx-vals` serialization avoids `422 Unprocessable Entity` on `Form<MarkPageReadForm>`.
  - `data-feed-title` DOM lookup and stretched-link CSS prevent apostrophe syntax errors, clipping, and focus-ring loss.
- **Cons:**
  - Adds one new HTTP endpoint (`POST /hx/mark-page-unread`), flipping `Has API` from `unknown` to `yes`.
  - Requires bumping `ARTICLE_DB_NAME` to v3 and updating existing tests in `templates.rs`, `entries.rs`, and `e2e_tests.rs`.

### Option B: Client-Side Burst Loop Reusing `POST /hx/entries/{id}/read?status=unread` (No New API Route)

- **Explanation:** Keep backend routes unchanged (`Has API: no`). Store the marked `entry_ids` and prior `offset` in a client-side `lastAuthMarkReadUndo` object; on **Undo**, fire `Promise.all(ids.map(id => fetch('/hx/entries/' + id + '/read?status=unread', { method: 'POST' })))` and reload `/hx/entries?...` via `htmx.ajax`.
- **Pros:** Zero backend route changes (`Has API: no`).
- **Cons:** Marking a 50-item page unread fires 50 concurrent `POST` requests plus 1 list `GET` (51 requests in one burst vs 60-request bucket in [rate_limit.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/rate_limit.rs#L75-L79), risking `429 Too Many Requests`), triggers ~150 upstream Miniflux API calls, risks partial state on network error, and cannot use declarative HTMX attributes on the Undo toast.

**Recommendation:** **Option A** — it keeps Auth Undo atomic (1 upstream mutation), avoids tripping the 60-token rate limiter, strictly honors `Form<MarkPageReadForm>` and `WCAG 2.2 SC 2.2.1`, and only reverts newly marked unread entries.

---

## Detailed Component Changes

### 1. [crates/ui-components/Cargo.toml](file:///Users/ptquang86/ws/my-ui/crates/ui-components/Cargo.toml)

- Move `chrono = "0.4"` from `[dev-dependencies]` to `[dependencies]` so non-test relative-time formatting compiles in `ui-components` and `server`.

### 2. [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)

- **Relative time helper (non-test & deterministic):**
  - Remove `#[cfg(test)]` gating on `chrono` parsing and add:
    - `pub fn format_relative_time_at(raw: &str, now: chrono::DateTime<chrono::Utc>) -> String`
    - `pub fn format_relative_time(raw: &str) -> String` (delegating to `chrono::Utc::now()`)
    - Keep `format_published_date_ddmmyyyy` under `#[cfg(test)]` if existing tests call it directly, or expose alongside.
    - **Cross-runtime bucket rules (identical in Rust & JS):**
      1. Trim whitespace; if empty, return `""`.
      2. If exact `YYYY-MM-DD` (`NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")`), return `DD/MM/YYYY`.
      3. If valid RFC 3339 (`DateTime::parse_from_rfc3339(trimmed)`): compute `diff_sec = (now - dt.with_timezone(&Utc)).num_seconds().max(0)`:
         - `0..=59` -> `"Just now"`
         - `60..=3599` -> `format!("{}m ago", diff_sec / 60)`
         - `3600..=86399` -> `format!("{}h ago", diff_sec / 3600)`
         - `86400..=604799` -> `format!("{}d ago", diff_sec / 86400)`
         - `>= 604800` (7 days+) -> `dt.format("%d/%m/%Y").to_string()`
      4. Otherwise (unparseable mock labels like `"2 hours ago"` or `"Just now"`), return `trimmed.to_string()`.
  - Add `pub fn relative_published_at(&self) -> String` on `EntryViewModel`.
- **Toast Undo contract (`ToastUndoAction` & `ToastUndoPayload`):**
  - Define:
    ```rust
    #[derive(Debug, Clone, Serialize)]
    pub struct ToastUndoPayload {
        pub entry_ids: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub search: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub feed_id: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub category_id: Option<i64>,
        pub offset: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ToastUndoAction {
        pub url: String,
        pub vals_json: String,
        pub label: String,
    }
    ```
  - Add `ToastUndoAction::from_payload(url: impl Into<String>, label: impl Into<String>, payload: &ToastUndoPayload) -> Self` so `vals_json` is always serialized from `ToastUndoPayload` (comma-joined `entry_ids`, `None`/empty `search`/`feed_id`/`category_id` omitted).
  - Add `pub undo: Option<ToastUndoAction>` to `ToastTemplate` and update callers in [error.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs) and `test_render_toast_oob_template` (`undo: None`).
- **Guest JSON payload:**
  - Extend `GuestEntryPayload<'a>` with `feed_title: &'a str` and `published_at: &'a str`, populated from `EntryListTemplate::guest_entries_json()`.

### 3. [crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html) & [crates/ui-components/templates/partials/entry_item.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_item.html)

- **List rows (`.annuaire-row`):**
  - Add `data-feed-title="{{ entry.feed_title }}"` and `data-published-at="{{ entry.published_at }}"` to `.annuaire-row` in both templates.
  - Keep `#entry-{{ entry.id }}-dot` **outside** the vertical wrapper as a left sibling inside `<div class="flex items-center gap-1.5 flex-1 min-w-0">`, and wrap `#entry-title-{{ entry.id }}` + `.entry-meta` inside `<div class="flex flex-col min-w-0 flex-1 gap-0.5">`:
    ```html
    <div class="flex items-center gap-1.5 flex-1 min-w-0">
      <span id="entry-{{ entry.id }}-dot" class="inline-flex shrink-0 empty:hidden">...</span>
      <div class="flex flex-col min-w-0 flex-1 gap-0.5">
        <span id="entry-title-{{ entry.id }}" ...>...</span>
        {% if !entry.feed_title.trim().is_empty() || !entry.published_at.trim().is_empty() %}
        <div class="entry-meta text-xs text-muted-foreground font-sans truncate">
          {% if !entry.feed_title.trim().is_empty() %}<span>{{ entry.feed_title }}</span>{% endif %}{% if !entry.feed_title.trim().is_empty() && !entry.published_at.trim().is_empty() %}<span aria-hidden="true"> · </span>{% endif %}{% if !entry.published_at.trim().is_empty() %}<time datetime="{{ entry.published_at }}" data-relative-time="true">{{ entry.relative_published_at() }}</time>{% endif %}
        </div>
        {% endif %}
      </div>
    </div>
    ```
  - `#entry-title-{{ entry.id }}` keeps its exact ID and OOB `outerHTML` contract ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L695-L702)) so title sync never overwrites `.entry-meta`.
- **Magazine cards (`#entries-magazine-view`, P1, P2, P3):**
  - Keep `.magazine-card` (`data-entry-id`, `data-feed-title`, `cursor-pointer`) free of visible `feed_title` or `<time>` labels.
  - Ensure `.magazine-card` is `position: relative` and its headline `<h3>`/`<h4>`/`<h5>` and `.hero-content-box` / P2 body wrapper have **no** `relative` class (`position: static`).
  - Move `id="mag-title-{{ entry.id }}"` (and `onclick`) **off** `<h3>`/`<h4>`/`<h5>` and **onto** `<a id="mag-title-{{ entry.id }}" href="/entries/{{ entry.id }}" class="magazine-card-link" onclick="if (!event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey) { event.preventDefault(); openEntryFromMagazine({{ entry.id }}, this.closest('.magazine-card')?.dataset.feedTitle || ''); }">...</a>` inside `<h3>`/`<h4>`/`<h5>`, keeping `.magazine-title`, `line-clamp-*`, and read/unread color classes on the outer heading (plus `title="{{ entry.summary }}"` on P3 `<h5>` and `<a id="mag-title-{{ entry.id }}" title="{{ entry.summary }}">`). That way `syncExecutiveBriefUpdates()` and `magTitleEl.setAttribute('title', summaryText)` update the `<a>` directly without wiping out `<a class="magazine-card-link">`, while [templates.rs#L3080-L3085](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3080-L3085) (`id="mag-title-101"` / `id="mag-title-102"`) continues to pass.

### 4. [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)

- **Header Prev / Next + 320px mobile fit:**
  - Use `gap-1.5` on both the left cluster (`#detail-back-btn`, `#detail-prev-btn`, `#detail-next-btn`) and the right action cluster (`Original`, `Unread`, `Bookmark`).
  - Render explicit `type="button"` on `#detail-back-btn`, `#detail-prev-btn` (`onclick="navigateDetailArticle(-1)"`, `aria-label="Previous article"`, `title="Previous article"`), and `#detail-next-btn` (`onclick="navigateDetailArticle(1)"`, `aria-label="Next article"`, `title="Next article"`), styled with `w-8 h-8 inline-flex items-center justify-center rounded-full border border-border bg-card hover:bg-muted text-foreground transition-colors shrink-0 disabled:opacity-40 disabled:pointer-events-none`.
- **Hero & standard metadata:**
  - Guard `" · "` on both `feed_title` and `published_at` being non-empty; omit the line when both are empty.
  - Hero overlay uses `text-xs text-white/85 font-sans mt-1.5 truncate` (for WCAG AA contrast over photos); standard header uses `text-xs text-muted-foreground font-sans mt-1.5 truncate`. Both render `<time datetime="{{ entry.published_at }}" data-relative-time="true">{{ entry.relative_published_at() }}</time>` and omit `author` and `reading_time`.

### 5. [crates/ui-components/templates/partials/toast.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html) & [crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)

- **`toast.html`:**
  - Add `role="status" aria-atomic="true"` on the `.pointer-events-auto` toast pill; keep `<span>{{ message }}</span>` as the first `<span>`.
  - When `let Some(u) = undo`:
    ```html
    <button
      type="button"
      hx-post="{{ u.url }}"
      hx-target="#entries-list"
      hx-swap="innerHTML"
      hx-indicator="#entries-loading-bar"
      hx-disabled-elt="this"
      hx-vals="{{ u.vals_json }}"
      class="toast-undo-btn ml-1 px-2.5 py-1 rounded-full bg-background/15 hover:bg-background/25 text-background font-bold transition-colors shrink-0 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-background disabled:opacity-50 disabled:cursor-not-allowed">
      {{ u.label }}
    </button>
    ```
    Using `.toast-undo-btn` instead of a fixed `id` prevents duplicate DOM IDs, and double-quoted `hx-vals="{{ u.vals_json }}"` uses Askama HTML attribute escaping safely.
- **`base.html`:**
  - Add stretched-link & focus CSS (expanding `.magazine-card.read .magazine-title` at [base.html#L507-L509](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L507-L509) to cover `.magazine-card-link` as well):
    ```css
    .magazine-card { position: relative; cursor: pointer; }
    .magazine-card-link { color: inherit; text-decoration: none; }
    .magazine-card-link::after { content: ""; position: absolute; inset: 0; z-index: 1; }
    .magazine-card-link:focus-visible { outline: none; }
    .magazine-card:has(.magazine-card-link:focus-visible) {
      outline: 2px solid var(--ring);
      outline-offset: 2px;
    }
    .magazine-card.read .magazine-title,
    .magazine-card.read .magazine-card-link {
      color: var(--muted-foreground) !important;
    }
    ```
  - Place `escapeToastHtml(str)`, `attachToastAutoDismiss(toast, durationMs)`, `showClientToast(message, isError = false)`, `dismissToast(btn)`, and the `.toast-undo-btn` `htmx:beforeRequest` / `htmx:afterRequest` listeners **before** `document.body.addEventListener("htmx:afterSwap"` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L600-L635) so the `htmx:afterSwap` and `htmx:oobAfterSwap` slices tested by [templates.rs#L4874-L4894](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4874-L4894) remain intact:
    - `attachToastAutoDismiss(toast, durationMs)` stores timer state (`timeoutId`, `remainingMs`, `startedAt`) so `mouseenter`, `focusin`, and `toast.dataset.undoInFlight === 'true'` pause countdown, and `mouseleave` / `focusout` resume it (`WCAG 2.2 SC 2.2.1`).
    - Update `dismissToast(btn)` to clear any attached timer before fading/removing.
    - Define shared `showClientToast(message, isError = false)` in `base.html` (available in both Auth and Guest modes) that deduplicates existing `.pointer-events-auto` toasts matching the first `<span>`, creates `<div role="status" aria-atomic="true" data-auto-dismiss="true" ...>`, escapes text via `escapeToastHtml(message)`, renders `alert-circle` (`text-destructive`) when `isError` else `bookmark` (`text-brand`), runs `lucide.createIcons({ root: toast })`, and calls `attachToastAutoDismiss(toast, 4000)`.
    - Wire `htmx:beforeRequest` / `htmx:afterRequest` for `.toast-undo-btn`: on `beforeRequest`, pause timer and set `toast.dataset.undoInFlight = 'true'`; on `afterRequest`, inspect `const reswap = (evt.detail.xhr && typeof evt.detail.xhr.getResponseHeader === 'function') ? (evt.detail.xhr.getResponseHeader('HX-Reswap') || '') : '';` and remove `toast` **only** when `evt.detail.successful === true && reswap.toLowerCase() !== 'none'`. Otherwise (including `200 OK` + `HX-Reswap: none` upstream error responses from `handle_htmx_error` in [error.rs#L52-L61](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L52-L61), or `4xx`/`5xx`/network failures), clear `delete toast.dataset.undoInFlight` and resume the countdown timer so the user retains their Undo button for retry.
  - In `htmx:oobAfterSwap` (`#toast-container`):
    - If an incoming toast has `.toast-undo-btn`, remove any existing toast containing `.toast-undo-btn` before running standard first-`<span>` deduplication.
    - Keep `lucide.createIcons({ root: evt.detail.target })` and `htmx.process(toast)` (so OOB `hx-post` binds reliably), and call `attachToastAutoDismiss(toast, hasUndo ? 8000 : 5000)` (keeping `5000` literal intact for [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3249)).

### 6. [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)

- **Mark-page-read Auth button & unread selector:**
  - Add `getVisibleUnreadEntryIds()` returning comma-joined IDs of `#entries-list .annuaire-row, #entries-list .ar-list-item` where `item.style.display !== 'none' && !item.classList.contains('read')`.
  - On Auth `#mark-page-read-btn`, keep `hx-include="#active-status, #active-offset, [name='search']"` and set `hx-vals="js:{entry_ids: getVisibleEntryIds(), unread_entry_ids: getVisibleUnreadEntryIds()}"`.
- **IndexedDB detail cache v3 & `isCacheValid`:**
  - Bump `indexedDB.open(ARTICLE_DB_NAME, 3)` and `e.oldVersion < 3` store clear.
  - Require `Boolean(cached && cached.html && cached.html.includes('id="detail-prev-btn"'))` in `isCacheValid`.
- **Relative time JS parity & refresh hooks (shared `<script>` block):**
  - Place `formatRelativeTimeLabel(raw)` and `refreshRelativeTimes(root)` in the **shared `<script>` block** immediately after `safeHref` ([index.html#L350](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L350), above `{% if !auth_enabled %}` at [index.html#L1795](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1795)) — matching `YYYY-MM-DD`, UTC `DD/MM/YYYY`, `<=59s`, `<3600s`, `<86400s`, `<7d`, `>=7d` thresholds identically to Rust — so both Auth mode (`auth_enabled = true`) and Guest mode (`auth_enabled = false`) have them, while still preceding `renderLocalAllArticles()` and preserving every slice boundary in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs).
  - Invoke `refreshRelativeTimes(...)` in: `DOMContentLoaded` (`#entries-list`), `applyCachedList()`, `openEntryDetail()` 30-day cache hit, `renderGuestEntryDetail()`, `renderLocalAllArticles()`, and `htmx:afterSwap` (`#entries-list`, `#detail-panel`, and `targetId.startsWith('entry-')`).
- **Executive Brief sync safety (`syncExecutiveBriefUpdates()`):**
  - In `syncExecutiveBriefUpdates()` ([index.html#L1358-L1361](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1358-L1361)), write `(magTitleEl.querySelector('.magazine-card-link') || magTitleEl).textContent = suggestedTitle;` (defense in depth alongside moving `id="mag-title-{{ entry.id }}"` onto `<a class="magazine-card-link">`) so Executive Brief swaps never replace the anchor tag with plain text, and keep `magTitleEl.setAttribute('title', summaryText)` for `priority-3`.
- **Detail navigation state (`detailNavIds`):**
  - Place `let detailNavIds = [];`, `syncDetailNavigationIds()`, `updateDetailNavButtons(entryId)`, and `navigateDetailArticle(delta)` in the detail helper section before `openEntryDetail`.
  - Call `syncDetailNavigationIds()` only when list views initialize or update: `DOMContentLoaded` (Auth SSR list when `AUTH_ENABLED`), `initGuestDb()` (after initial `syncDomReadStates()` hides prior-session reads, including error/catch fallback branches, and refresh `updateDetailNavButtons(activeId)` if `getActiveDetailEntryId()` is open), `applyCachedList()`, `renderLocalAllArticles()`, `filterArticlesClientSide()`, and `#entries-list` `htmx:afterSwap` (after `syncDomReadStates()`). Do **not** call `syncDetailNavigationIds()` inside `markLocallyRead()` / `unmarkLocallyRead()` / `toggleLocalReadStatus()`.
  - Keep `openDetailWithSkeleton(title, brief, imageUrl)`'s exact 3-argument signature (preserving [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4962) & [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4972)), render `#detail-back-btn` (`type="button"`), `#detail-prev-btn` (`type="button"`), and `#detail-next-btn` (`type="button"`) with `gap-1.5` (and zero `"Feed info & date"` / `"Executive Brief"` text), and call `updateDetailNavButtons(numId)` immediately after `openDetailWithSkeleton(title, brief, imageUrl)` in `openEntryDetail(id)`.
  - Also call `updateDetailNavButtons(numId)` on IndexedDB cache hit in `openEntryDetail()`, in `renderGuestEntryDetail()`, and in `#detail-panel` `htmx:afterSwap`. Inside `updateDetailNavButtons(entryId)`, disable both buttons when `detailNavIds.indexOf(Number(entryId)) === -1`.
- **Magazine click fallback:**
  - In `openEntryFromMagazine(id, feedTitle)`, resolve `feedTitle || (cardEl && cardEl.dataset ? cardEl.dataset.feedTitle : '') || ''`.
- **Guest caching & rendering parity:**
  - Update `buildGuestArticleRecord`, `hydrateGuestEntriesFromDom`, `syncGuestEntriesScript`, and `cacheLoadedArticlesFromDom` to preserve `feed_title` and `published_at` (falling back to `.annuaire-row` `data-feed-title` / `data-published-at` attributes).
  - Update `renderLocalAllArticles()` (preserving the slice boundary before `syncDomReadStates()`) to render `data-feed-title`, `data-published-at`, `#entry-title-${art.id}`, and `.entry-meta` (`Feed · <time>`) with `#entry-${art.id}-dot` outside the `flex-col` wrapper.
  - Update `renderGuestEntryDetail(entry)` to render `#detail-prev-btn` / `#detail-next-btn` (`gap-1.5`, `type="button"`), guarded `feed_title` + `<time>` metadata (`text-white/85` on hero, `text-muted-foreground` on standard header), and call `updateDetailNavButtons(numId)` + `refreshRelativeTimes(panel)`.
- **Guest & Auth toast helpers (`showToast`, `showUndoToast`, `undoGuestMarkPageRead`, `htmx:afterSwap`, `htmx:responseError`, `htmx:sendError`):**
  - Keep `function showToast(message)` in place inside `{% if !auth_enabled %}` directly before `function triggerNativeBookmark(title, url)` (preserving slice boundaries at [templates.rs#L4212](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4212), [L4452](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4452), and [L4841](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4841)), delegating directly to shared `showClientToast(message, false)`.
  - Place `showUndoToast(message, onUndo)` and `undoGuestMarkPageRead(snapshot)` immediately after `triggerNativeBookmark` (keeping the slice between `syncDomReadStates()` and `markLocallyRead()` untouched for script slice tests).
  - Inside `showUndoToast(message, onUndo)`: remove any existing `.toast-undo-btn` toast, set `role="status" aria-atomic="true"`, render `<span>${escapeHtml(message)}</span>` and `<button type="button" class="toast-undo-btn ...">Undo</button>`, attach `attachToastAutoDismiss(toast, 8000)`, and on click disable button, clear timer, remove toast, invoke `onUndo()`, and show `'Mark as read undone.'` via `showClientToast('Mark as read undone.', false)`.
  - In `markVisiblePageLocallyRead()`: compute `newlyReadIds = ids.filter(id => !readSet.has(id))`, read `status = document.getElementById('active-status')?.value || 'unread'`, `offset = document.getElementById('active-offset')?.value || '0'`, `search = document.getElementById('search-input')?.value.trim() || ''`, and `advancedPage = Boolean(nextUrl && window.htmx && markBtn)`, snapshot `{ status, offset, search, ids: newlyReadIds, advancedPage }`, and call `showUndoToast('Page marked as read.', () => undoGuestMarkPageRead(snapshot))` when `newlyReadIds.length > 0` (else `showToast('Page marked as read.')`).
  - In `undoGuestMarkPageRead(snapshot)`:
    1. Guard `!snapshot || !Array.isArray(snapshot.ids) || snapshot.ids.length === 0`.
    2. Reset `autoAdvancingUnreadUrl = null;` and call `cancelPendingMarkPageRead()`.
    3. Delete `snapshot.ids` from `readSet` **synchronously** (`snapshot.ids.forEach(id => readSet.delete(Number(id)))`) *before* restoring or invoking `htmx.ajax` (preventing `syncDomReadStates()` from auto-advancing past the restored page).
    4. Delete `snapshot.ids` from IndexedDB `STORE_NAME` (`read_entries`) in one `readwrite` transaction and call `invalidateFilterCache()`.
    5. When `snapshot.advancedPage` is true: restore the active filter pill for `snapshot.status` if needed (`setActiveFilter`), call `showEntriesLoadingState()`, and re-fetch `/hx/entries?status=${encodeURIComponent(snapshot.status || 'unread')}&offset=${encodeURIComponent(snapshot.offset || '0')}` (+ `&search=${encodeURIComponent(snapshot.search)}` when non-empty) via `htmx.ajax('GET', restoreUrl, { target: '#entries-list', swap: 'innerHTML' })` (whose `htmx:afterSwap` runs `hydrateGuestEntriesFromDom()`, `syncDomReadStates()`, `syncDetailNavigationIds()`, `refreshRelativeTimes()`, and `applyAiViewMode()`).
    6. When `!snapshot.advancedPage`: call `syncDomReadStates()`, re-apply `filterArticlesClientSide(searchQuery)` if `#search-input` has a non-empty trimmed query (and `!isSearchActive`), and call `syncDetailNavigationIds()`.
  - In `#entries-list` `htmx:afterSwap`: invalidate `filterCache` when `triggerElt` matches `.toast-undo-btn` as well as `mark-page-read-btn`.
  - In `htmx:responseError` ([index.html#L1698](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1698)) and `htmx:sendError` ([index.html#L1730](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1730)): check `const triggerElt = evt.detail?.requestConfig?.elt || evt.detail?.elt;` **first**; if `triggerElt?.closest?.('.toast-undo-btn')`, call `showClientToast('Could not undo. Please try again.', true); return;` before checking `evt.detail.target.id === 'entries-list'` so `#entries-list` never emits duplicate load/network error toasts. Also switch the `#entries-list` error handlers (`L1700-L1702`, `L1732-L1734`) to `showClientToast(..., true)` so Auth mode also receives load error toasts.

### 7. [crates/server/src/error.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs), [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs), & [crates/server/src/app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs)

- **`error.rs`:** Pass `undo: None` on `ToastTemplate` literals in `render_llm_error_toast` and `handle_htmx_error`.
- **`entries.rs`:**
  - Add `#[serde(default)] pub unread_entry_ids: Option<String>` to `MarkPageReadForm`.
  - Add shared `parse_validated_entry_ids(raw: Option<&str>, allow_json_array: bool) -> Result<Vec<i64>, AppError>` rejecting non-numeric or `<= 0` tokens.
  - Update `mark_page_read_htmx`:
    - Validate `ids` from `form.entry_ids`; return `(StatusCode::BAD_REQUEST, "Invalid entry IDs")` if invalid.
    - Parse `undo_ids` from `form.unread_entry_ids` when `Some(...)`, else fall back to `ids` (`None`).
    - When `config.auth_enabled && !undo_ids.is_empty()`, execute `updateEntriesStatus(entryIds: [...], status: READ)` for `undo_ids` only and invalidate `cache.invalidate(&format!("entry:{id}")).await` for each mutated ID (in addition to `"entries:"` and `"index_page_entries"`).
    - Keep `current_offset = form.offset.unwrap_or(0).max(0)` and reuse `ai_service` + `Some(&cache)` parity or existing list restore.
    - Attach `undo: (!undo_ids.is_empty()).then(|| ToastUndoAction::from_payload("/hx/mark-page-unread", "Undo", &ToastUndoPayload { entry_ids: undo_ids joined, status, search (non-empty only), feed_id, category_id, offset: current_offset }))`.
  - Add `mark_page_unread_htmx` (`POST /hx/mark-page-unread`) in `entries.rs` and register `.route("/hx/mark-page-unread", post(mark_page_unread_htmx))` in [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L112):
    - Require `auth_enabled` + `AuthenticatedUser` (`403` when `!config.auth_enabled`).
    - Inject `State(schema)`, `State(config)`, `State(cache)`, `State(ai_service)`, `headers`, `Form(form)`.
    - Validate `ids` via `parse_validated_entry_ids(form.entry_ids.as_deref(), false)` -> return `(StatusCode::BAD_REQUEST, "Invalid entry IDs").into_response()` on invalid tokens.
    - Guard `!ids.is_empty()`: only when non-empty, execute `updateEntriesStatus(entryIds: [...], status: UNREAD)` and invalidate `"entries:"`, `"index_page_entries"`, and each `format!("entry:{id}")`.
    - Clamp `let mut target_offset = form.offset.unwrap_or(0).max(0);`. When `target_offset > 0`, query `entriesResult(filter: { ... }) { total }`; if `total <= 0`, set `target_offset = 0`; else if `target_offset >= total`, snap to `((total - 1) / limit) * limit`.
    - Render `render_entries_list_html(&schema, Some(&cache), ai_ref, &headers, &filter_query, config.auth_enabled)` and append `"Mark as read undone."` `ToastTemplate` (`is_error: false`, `undo: None`).
  - Update `mark_entry_read_htmx` fallback timestamp ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2190) & [L2225](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2225)) from `"Just now"` to `""` so missing `publishedAt` cleanly omits `<time>`.

---

## Complete File & Legacy Test Impact Plan

1. **[crates/ui-components/Cargo.toml](file:///Users/ptquang86/ws/my-ui/crates/ui-components/Cargo.toml#L6-L14):** Move `chrono = "0.4"` to `[dependencies]`.
2. **[crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs):**
   - `test_render_title_only_entry_item` ([L359-L466](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L359-L466)) & `test_render_entry_list_with_title_only_items` ([L469-L550](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L469-L550)): assert `Feed · <time>` renders cleanly while summary, author, and reading time remain omitted.
   - `test_detail_template_omits_author` ([L702-L755](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L702-L755)) & `test_detail_template_omits_metadata_and_orders_sections` ([L758-L841](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L758-L841)): assert `Feed · <time>` renders in detail headers while `author` and `reading_time` stay omitted.
   - `test_render_toast_oob_template` ([L1162-L1188](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L1162-L1188)): update `ToastTemplate` struct literal (`undo: None`), verify `role="status"` and `aria-atomic="true"`.
   - `test_ai_magazine_view_renders_3_column_layout` ([L3073](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3073)): update assertion from `<h3>` `onclick="openEntryFromMagazine(101, 'TechCrunch')"` to `<a id="mag-title-101"` `.magazine-card-link` reading `dataset.feedTitle` (preserving `id="mag-title-101"` / `id="mag-title-102"` checks at [L3080-L3085](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3080-L3085)).
   - `test_entry_detail_omits_ddmmyyyy_time_element_in_both_headers` ([L3626-L3689](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3626-L3689)): update assertions on both hero and standard `EntryDetailTemplate` headers to verify `"Daily Wire"`, `<time`, `data-relative-time="true"`, and `" · "` render cleanly while `data-date-format`, `author`, and `reading_time` stay omitted.
   - IndexedDB version assertion ([L3766](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3766)): update `indexedDB.open(ARTICLE_DB_NAME, 2)` to `3`.
   - `test_entry_list_guest_json_excludes_original_content_and_includes_images_and_rewrite` ([L3867-L3868](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3867-L3868)): assert `item["feed_title"]` and `item["published_at"]` are included while `sanitized_content`, `formatted_date`, `author`, and `reading_time` stay excluded.
   - `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` ([L3964-L3966](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3964-L3966)): update assertions on `renderGuestEntryDetail` for `feed_title` and `published_at` while keeping `author`, `reading_time`, and `sanitized_content` omitted.
   - `test_paging_loading_overlay_and_read_blur_removed` ([L4221](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4221)) & `test_guest_mark_page_read_htmx_next_page_and_deferred_commit` ([L4463](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4463)): update assertions checking `showToast('Page marked as read.')` to verify `showUndoToast('Page marked as read.'` / fallback `showToast('Page marked as read.')`.
   - `test_toast_deduplication_and_detail_is_error_cache_guard` ([L4841-L4850](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4841-L4850)): update Guest `showToast` slice check to verify delegation to `showClientToast(message, false)` and assert `showClientToast` in `base.html` sets `data-auto-dismiss="true"`, `role="status"`, and `aria-atomic="true"`.
3. **[crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs):**
   - `test_get_entry_detail_formats_published_at_as_ddmmyyyy` ([L3336-L3340](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L3336-L3340)) & `test_get_entry_detail_triggers_rewrite_when_missing_and_omits_metadata` ([L3442-L3443](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L3442-L3443)): update assertions so `feed_title` and `<time>` ARE expected while `author`, `reading_time`, and raw content stay omitted.
   - Guest JSON payload integration test ([L4506-L4507](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L4506-L4507)): update assertions to expect `feed_title` and `published_at` in `#entries-guest-data`.
4. **[crates/server/tests/e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs#L341-L420) & [crates/server/src/routes/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs):**
   - Extend `test_mutation_endpoints_reject_with_403_when_auth_disabled` (and `routes/mod.rs`) to verify `POST /hx/mark-page-unread` returns `403` when `auth_enabled = false`, plus full Undo integration coverage.

## Reference Sources

- Annuaire Design System: [ANNUAIRE_STYLE_GUIDE.md](file:///Users/ptquang86/ws/my-ui/ANNUAIRE_STYLE_GUIDE.md)
- HTMX `HX-Reswap`, `hx-vals`, `hx-swap-oob`, `hx-disabled-elt`: <https://htmx.org/reference/#response_headers> · <https://htmx.org/attributes/hx-vals/> · <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/attributes/hx-disabled-elt/>
- WAI-ARIA `status` role & WCAG 2.2 SC 2.2.1: <https://www.w3.org/WAI/ARIA/apg/patterns/alert/> · <https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable.html>
- Axum `Form` & `serde_urlencoded`: <https://docs.rs/axum/latest/axum/extract/struct.Form.html> · <https://docs.rs/serde_urlencoded/latest/serde_urlencoded/>
- Askama 0.12 HTML escaping & Chrono 0.4 `DateTime::parse_from_rfc3339`: <https://docs.rs/askama/latest/askama/> · <https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339>
