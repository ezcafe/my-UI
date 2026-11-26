# Code Review (Lite) — Fix Magazine View Flash & Detail Impacts

## Adversarial Review

Result: clean

- **List-view flash (`data-view-mode` on `#entries-list`):**
  - CSS rules in `crates/ui-components/templates/pages/index.html` (`#entries-list[data-view-mode="ai"] #entries-list-view { display: none; }` / `#entries-magazine-view { display: block; }` and vice versa for `"list"`) have selector specificity `(2, 1, 0)`, overriding Tailwind `.hidden` `(0, 1, 0)` on `#entries-magazine-view` even before `htmx:afterSwap` runs.
  - `htmx:beforeSwap` sets `data-view-mode` on `#entries-list` synchronously before the new DOM fragment is inserted, using `getDefaultViewForState` for search/filter-tab swaps and `shouldHideAiToggle` / `currentViewMode` for pagination and `mark-page-read-btn` swaps.
  - Edge cases checked: empty `entries` in `entry_list.html` renders the "All caught up!" block without `#entries-list-view` or `#entries-magazine-view`, so the CSS rules do not hide the empty state. `applyAiViewMode` also keeps `data-view-mode` and `.hidden` classes synchronized.
- **Detail hero photo, rewrite, and LLM impacts:**
  - Auth skeleton (`openEntryDetail` -> `openDetailWithSkeleton`): extracts title (`#entry-title-{id}` / `#mag-title-{id}`), brief (`#mag-entry-{id} .magazine-brief` -> `#entry-{id} .magazine-brief` -> `#mag-entry-{id}[title]`), and hero image (`#mag-entry-{id} img[src]`). `imageUrl` is validated with `safeHref` (allowing only `http://`, `https://`, or single-leading-slash relative paths) and escaped with `escapeHtml`; `onerror="this.parentElement.style.display='none'"` hides broken image banners. `lucide.createIcons({ root: panel })` is invoked immediately after `panel.innerHTML` assignment.
  - Guest detail (`GuestEntryPayload` + `renderGuestEntryDetail` + `cacheLoadedArticlesFromDom`): `summary` and `image_url` are serialized in `guest_entries_json` (with `</` escaped to `<\/`), preserved in IndexedDB `all_articles`, and rendered in `renderGuestEntryDetail` with `safeHref` + `escapeHtml` for `image_url` and `escapeHtml` for `summary`. `#ai-analysis-box` (`hx-get="/hx/entries/${numId}/analysis" hx-trigger="load" hx-swap="outerHTML"`) is activated via `htmx.process(panel)`. Existing guest privacy constraints (`author`, `readingTime`, `/rewrite`, `/hx/entries/${numId}` full detail fetch) remain omitted.

## Quality Review

Result: clean

- **Correctness:** Both auth and guest paths handle missing `image_url` and empty `summary`/`brief` gracefully without broken layouts or XSS vectors.
- **Readability & Architecture:** CSS `data-view-mode` avoids threading `view_mode` through backend handlers and templates while covering all `#entries-list` HTMX swaps cleanly. `GuestEntryPayload` extension is minimal and typed.
- **Security:** All newly interpolated strings (`title`, `brief`, `summary`, `image_url`) pass through `escapeHtml` and, for URLs, `safeHref`. Askama auto-escaping and `</` JSON script escaping remain intact.
- **Performance:** Zero extra network requests for auth skeleton or guest detail hero/rewrite; `htmx.process(panel)` scopes processing to `#detail-panel` only.
- **Verification:** `cargo test -p ui-components` (87 passed, 0 failed) and `cargo clippy --all-targets --workspace -- -D warnings` (0 warnings) passed after final edits per `06-test-report.md`.
