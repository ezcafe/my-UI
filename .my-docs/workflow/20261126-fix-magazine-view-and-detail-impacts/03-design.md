# Design — Fix Magazine View Flash & Detail Impacts

Result: done

## 1. Issue 1: Eliminate List-View Flash After "Mark Page as Read"

### Options

- **Option A (Recommended): CSS-driven `data-view-mode` on `#entries-list` + `applyAiViewMode` sync**
  - **Explanation:** Set `data-view-mode="ai"` or `"list"` on `#entries-list` inside `applyAiViewMode` (and initialize it before first paint / in `htmx:beforeSwap` for `#entries-list`). Add CSS rules in `index.html`:
    ```css
    #entries-list[data-view-mode="ai"] #entries-list-view { display: none; }
    #entries-list[data-view-mode="ai"] #entries-magazine-view { display: block; }
    #entries-list[data-view-mode="list"] #entries-list-view { display: block; }
    #entries-list[data-view-mode="list"] #entries-magazine-view { display: none; }
    ```
    Keep toggling `.hidden` on `#entries-list-view` and `#entries-magazine-view` inside `applyAiViewMode` for compatibility.
  - **Pros:** Zero flash on first paint for every `#entries-list` swap (`mark-page-read-btn`, pagination, filter tabs, search); zero backend changes.
  - **Cons:** Must keep `data-view-mode` synchronized with `effectiveMode` in `applyAiViewMode` and `htmx:beforeSwap` (e.g., if a tab forces `list` mode).

- **Option B: Server-side `view_mode` query/form parameter on `/hx/mark-page-read` & `/hx/entries`**
  - **Explanation:** Send `view_mode` to the server and render `hidden` conditionally in `entry_list.html`.
  - **Pros:** Server HTML matches active view mode directly.
  - **Cons:** Touches backend forms/queries/templates/tests and still doesn't cover client-side `filterCache` swaps without JS.

- **Decision:** **Option A**.

---

## 2. Issue 2: Restore Detail Photo + Title + Rewrite & LLM Impacts Request

### Options

- **Option A (Recommended): Fix both Auth skeleton extraction (`#mag-entry-{id}`) and Guest detail (`GuestEntryPayload` + `renderGuestEntryDetail` + `#ai-analysis-box`)**
  - **Explanation:**
    1. **Auth mode (`openEntryDetail` & `openDetailWithSkeleton` in `index.html`):**
       - Extract `title` from `#entry-title-{id}` or `#mag-title-{id}`.
       - Extract `brief` from `#mag-entry-{id} .magazine-brief` (fallback `#entry-{id} .magazine-brief`, or `#mag-entry-{id}` `title` attribute for P3 cards).
       - Extract `imageUrl` from `#mag-entry-{id} img` `src` (if present).
       - Update `openDetailWithSkeleton(title, brief, imageUrl)` to render the hero image banner (matching `entry_detail.html`) when `imageUrl` is present, plus `title` and `brief` immediately while `/hx/entries/{id}` loads.
    2. **Guest mode (`GuestEntryPayload` in `templates.rs` + `index.html`):**
       - Add `summary: &'a str` and `image_url: Option<&'a str>` to `GuestEntryPayload` in `crates/ui-components/src/templates.rs` (`guest_entries_json`).
       - Preserve `summary` and `image_url` in `cacheLoadedArticlesFromDom()` (reading from `guest` payload or `#mag-entry-{id}` DOM fallback) when saving to IndexedDB `all_articles`.
       - In `renderGuestEntryDetail(entry)`, render:
         - Hero photo banner (if `entry.image_url` is present, sanitized via `safeHref`) with `feed_title`, `date`, and `title` (or standard header if no `image_url`).
         - Rewrite / Executive Brief block from `entry.summary` (if non-empty), followed by `sanitized_content` if needed (or `ai-rewrite-box` with `entry.summary` when present, falling back to `sanitized_content`).
         - `<div id="ai-analysis-box" class="animate-pulse space-y-4 my-4" hx-get="/hx/entries/${numId}/analysis" hx-trigger="load" hx-swap="outerHTML">...</div>` and call `htmx.process(panel)` so HTMX immediately requests LLM impacts from `/hx/entries/{id}/analysis`.
       - Update `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` in `crates/ui-components/src/templates.rs` to assert `/analysis` and `htmx.process` are present (while still asserting no `/hx/entries/${numId}` detail fetch, no `author`, and no `readingTime`).
  - **Pros:** Restores immediate photo + title + rewrite and LLM impacts request in both Auth and Guest modes, from both Magazine and List views, with no new backend endpoints.
  - **Cons:** Updates one existing unit test assertion in `templates.rs` that previously forbade `/analysis` in `renderGuestEntryDetail`.

- **Option B: Remove `openGuestEntryDetail` and always call `/hx/entries/{id}`**
  - **Explanation:** Route guest clicks to `/hx/entries/{id}` instead of local `renderGuestEntryDetail`.
  - **Pros:** Single code path.
  - **Cons:** Breaks offline/guest IndexedDB caching and violates existing guest privacy constraints (`author`, `readingTime`, offline fallback).

- **Decision:** **Option A**.
