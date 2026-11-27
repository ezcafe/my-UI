Result: clean

# Design Review Log — Details Page Content Filter (Re-verify Round 2)

## Key Takeaways

- **Overall verdict:** **Clean** — all **2 Critical** and **5 Major** findings from Round 1 across API contracts, HTMX/client lifecycle, HTML-to-plain-text sanitization, and task/test plans are resolved in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md). Zero Critical or Major issues remain.
- **Surface statuses:**
  - **API Contract ([03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03a-api-contract-review.md)):** `clean` (re-verified Round 2).
  - **DB Surface:** `skipped` (`Has DB = no`; no schema/database changes).
  - **Grill Gate:** Closed (`Result: frontier-empty` in [02b-grill.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/02b-grill.md); Q1=Option C, Q2=Yes, Q3=Yes settled).

---

## Round 1 Resolution Verification

### API Contract Surface (`03a-api-contract-review.md`)
1. **[CRITICAL] Auth-vs-Guest gate on `EntryListTemplate::guest_entries_json()` & `#[serde(default)]` on `EntryViewModel`:** **Resolved** — `EntryListTemplate` adds `pub auth_enabled: bool`, passed from all list handlers and tests; `EntryViewModel` adds `#[serde(default)]` on `rewrite` and `body_images` and initializes them across all constructors.
2. **[CRITICAL] `/hx/executive-brief` OOB sync clobber, naked `<script hx-swap-oob>`, and partial/cached non-title sync:** **Resolved** — `<div id="executive-brief-sync-data" hidden hx-swap-oob="outerHTML" data-entries="..."></div>` covers cache hit, in-flight wait, and fresh synthesis (including Case C); `syncExecutiveBriefUpdates()` rewrites WHATWG-escaped `#entries-guest-data` before `filterCache.set`, and `hydrateGuestEntriesFromDom()` runs on `isEntriesSwap` only.
3. **[MAJOR] Unscoped `ai-rewrite-done`, missing Auth error fallback, `AnalysisCardsTemplate` OOB attribute, and `"No summary available."` empty fallback:** **Resolved** — entry-scoped `HX-Trigger-After-Settle: ai-rewrite-done-{id}`, `data-fallback-summary` on `#ai-rewrite-box`, shared `analysis_cards_inner.html` + `AnalysisCardsTemplate { oob: true }`, and explicit empty-fallback copy `"No summary available."`.
4. **[MAJOR] Photo URL scheme parity (`extract_article_images` vs `safeHref`) & `EntryDetailTemplate.ai_enabled` omission:** **Resolved** — `extract_article_images` enforces `http://` / `https://`; `EntryDetailTemplate` includes `pub ai_enabled: bool` and `safe_entry_url()`.

### General Design, Client State, Sanitization & Task/Test Findings
5. **[MAJOR] Infinite `/rewrite` retry loop, Guest `.is-error` caching, cross-mode IndexedDB pollution, open-panel brief sync, and invalid `body_images` filtering:** **Resolved** — `htmx:responseError` / `htmx:sendError` replaces `#ai-rewrite-box` via `outerHTML` (stripping `hx-get`/`hx-trigger`/`hx-swap` from the container) and dispatches `ai-rewrite-done-{numId}` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L99)); Guest persistence reads `.prose` text only when `!rewriteBox.querySelector('.is-error, .stroke-destructive')` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L101)); `saveCachedArticle` is gated by `AUTH_ENABLED && !panel.querySelector('[hx-get][hx-trigger], .stroke-destructive, .is-error')` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L102)); `cacheLoadedArticlesFromDom()` preserves existing non-empty `rewrite` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L104)); `syncExecutiveBriefUpdates()` updates open `#detail-panel` and dispatches `ai-rewrite-done-{id}` when `#ai-analysis-box[hx-trigger]` is waiting ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L87)); `renderGuestEntryDetail` filters `body_images` via `safeImg && safeImg !== '#'` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L96)).
6. **[MAJOR] `extract_plain_summary` block-tag spacing, entity decoding, and UTF-8 char boundaries (`sanitizer.rs`):** **Resolved** — `LazyLock<Regex>` inserts `" "` across block/line-break HTML boundaries while keeping `<script>`/`<style>` intact for `ammonia::Builder::default().tags(HashSet::new()).clean(...)`, common entities decode with `&amp;` last, length caps use `.chars()` / `.char_indices()` exclusively, and `record.summary` is also sanitized via `extract_plain_summary(&record.summary, "")` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L19-L23), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L59), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L64), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L69)).
7. **[MAJOR] Incomplete test impact plan in `04-tasks.md`:** **Resolved** — `03-design.md` and `04-tasks.md` explicitly remove `formatDetailDates(root)` and update `test_index_template_formats_detail_dates_on_swap_and_cache_restore` (renamed to `test_index_template_omits_detail_date_formatter_and_guards_auth_cache`), list all `EntryViewModel` / `EntryDetailTemplate` / `EntryListTemplate` / `AnalysisCardsTemplate` struct literal updates, and include all affected unit, route, and E2E tests in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md#L24), [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md#L40), [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md#L60-L75).

---

## Minor Observations (Non-Blocking)

- **Retry-on-error URL & fallback preservation (`index.html`):** In `htmx:responseError` / `htmx:sendError` for `target.id === 'ai-rewrite-box'`, if a retry triggered by the error card's `<button hx-get="...">` fails a second time, `target.getAttribute('hx-get')` is `null`; read `target.getAttribute('hx-get') || evt.detail.elt?.getAttribute('hx-get') || evt.detail.requestConfig?.path`, guard `if (!target.isConnected) return;`, and retain `data-fallback-summary` on the rendered error `#ai-rewrite-box`. Also verify `!rewriteBox.classList.contains('is-error')` alongside `!rewriteBox.querySelector('.is-error, .stroke-destructive')` in `htmx:afterSwap`.
- **Clippy hygiene on `extract_first_image_url` (`entries.rs`):** Once `entry_view_model_from_json` switches to `extract_article_images`, remove `extract_first_image_url` (or scope it under `#[cfg(test)]`) so `cargo clippy --workspace --all-targets -- -D warnings` passes without `dead_code` warnings.

---

## Reference Sources

- `ammonia` 4.x `Builder::tags` & `clean_content_tags`: <https://docs.rs/ammonia/latest/ammonia/struct.Builder.html#method.tags>
- `htmx` 1.9 `hx-swap-oob`, `hx-trigger`, `HX-Trigger-After-Settle`, `responseError`/`sendError`: <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/headers/hx-trigger/>
- `askama` 0.12 docs: <https://docs.rs/askama/0.12.1/askama/>
- Repo artifacts & source verified:
  - [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md)
  - [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md)
  - [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03a-api-contract-review.md)
  - [crates/ui-components/src/sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs)
  - [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - [crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)
  - [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)
  - [crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)
  - [crates/ui-components/templates/partials/analysis_cards.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards.html)
  - [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
  - [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - [crates/server/src/routes/pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs)
  - [crates/server/tests/e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs)
