Result: clean

# TDD Test Plan Review — Details Page Content Filter

## Key Takeaways

- **Verdict:** **Clean** — all 13 planned test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md#L60-L75) give full TDD coverage across HTML-to-plain-text sanitization, template metadata removal, `auth_enabled` / `serde(default)` payload contracts, `/hx/entries/{id}/rewrite` + OOB analysis, `/hx/executive-brief` OOB sync, client error/cache guards, and E2E flow.
- **No blocking gaps:** Happy paths, failure/offline fallbacks (`!ai_enabled`, LLM `Err(_)`, HTMX `responseError`/`sendError`), XSS/scheme boundaries (`http://`/`https://`, `safe_entry_url`, WHATWG script escaping), and UTF-8 char-boundary safety are all covered.

---

## Planned Cases

| Task | Planned Test(s) | Coverage Check |
|------|-----------------|----------------|
| **Task 1 (`sanitizer.rs`)** | `sanitizer::tests::test_extract_plain_summary_strips_html_and_truncates_content` | **Pass** — Covers block-tag spacing (`<p>First.</p><p>Second.</p>`), `ammonia` `<script>`/`<style>` content stripping, single-pass entity decoding (`&amp;` last), Vietnamese/UTF-8 `.chars()`/`.char_indices()` truncation (400-char summary / 300-char excerpt), and empty fallback behavior. |
| **Task 2 (`crates/ui-components`)** | `templates::tests::test_detail_template_omits_metadata_and_orders_sections`, `test_detail_template_omits_author`, `test_entry_detail_renders_ddmmyyyy_time_element_in_both_headers` (updated), `test_entry_list_guest_json_excludes_original_content_and_includes_images_and_rewrite`, `test_entry_view_model_category_and_image_url`, `test_analysis_cards_template_oob_attribute` | **Pass** — Covers `feed_title`/`<time>`/author omission in Hero & Standard headers, `safe_entry_url()`, Option C `data-fallback-summary` + `ai-rewrite-done-{id}`, section order, `!ai_enabled` fallback (`"No summary available."`), `auth_enabled` gate on `guest_entries_json()`, `GuestEntryPayload` field filtering, `EntryViewModel` `#[serde(default)]`, and `AnalysisCardsTemplate { oob }`. |
| **Task 3 (`crates/server`)** | `routes::entries::tests::test_extract_article_images_hero_and_body`, `test_get_entry_detail_triggers_rewrite_when_missing_and_omits_metadata`, `test_get_entry_detail_formats_published_at_as_ddmmyyyy` (updated), `test_get_entry_rewrite_htmx_oob_analysis_and_plain_text_fallback`, `test_get_entry_rewrite_htmx_serves_impact_and_fallacies` (updated), `test_executive_brief_emits_sync_data_for_magazine_and_guest`, `test_executive_brief_waits_for_inflight_and_surfaces_error_with_toast`, `test_list_routes_embed_sanitized_guest_data_only_when_auth_disabled`, `e2e_tests::test_e2e_page_load_and_detail_drawer_rewrite_flow` | **Pass** — Covers `http://`/`https://` image filtering, raw Miniflux fallback removal, `extractive-fallback` filtering, rewrite-only `#ai-rewrite-box` + OOB `#ai-analysis-box` + `HX-Trigger-After-Settle: ai-rewrite-done-{id}`, `!ai_enabled` and LLM error plain-text fallbacks, `#executive-brief-sync-data` OOB across cache hit / in-flight / fresh synthesis / Case C partial responses without `class="hidden"`, Guest list multi-image embedding, and E2E Story 2 Option C flow. |
| **Task 4 (`index.html`, `templates.rs`)** | `templates::tests::test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time`, `test_index_template_omits_detail_date_formatter_and_guards_auth_cache` | **Pass** — Covers `formatDetailDates` removal, `openDetailWithSkeleton` cleanup, `syncExecutiveBriefUpdates()` before `filterCache.set`, `hydrateGuestEntriesFromDom` (`isEntriesSwap` only), `safeImg !== '#'`, `outerHTML` error replacement on `#ai-rewrite-box`, `.is-error`/`.prose` Guest rewrite guard, `AUTH_ENABLED` `saveCachedArticle` guard, and IndexedDB v2/v3 upgrades. |

---

## Implementation Notes for Build (Non-Blocking)

1. **Existing tests requiring struct-literal or assertion checks:** During Task 2–3 TDD updates, also update struct literals / verify assertions in `templates::tests::test_entry_detail_template_shows_rewrite_and_analysis_trigger_when_not_cached` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L774)), `routes::entries::tests::test_ai_endpoints_return_error_card_and_toast_after_retry_exhausted` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L4394)), and `e2e_tests::test_e2e_detail_drawer_analysis_flow` ([e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs#L840)).
2. **Security & retry-path assertions to include inside planned tests:**
   - In `test_detail_template_omits_metadata_and_orders_sections`, assert `safe_entry_url()` blocks `javascript:` / `//` URLs on both the `Original` link and bookmark `data-url`.
   - In `test_index_template_omits_detail_date_formatter_and_guards_auth_cache`, assert `htmx:responseError` / `htmx:sendError` resolves `retryUrl` from `target.getAttribute('hx-get') || evt.detail.elt?.getAttribute('hx-get') || evt.detail.requestConfig?.path`, preserves `data-fallback-summary`, and checks `!rewriteBox.classList.contains('is-error')` alongside `!rewriteBox.querySelector('.is-error, .stroke-destructive')`.

---

## Reference Sources

- `htmx` 1.9 `HX-Trigger-After-Settle` & `hx-swap-oob`: <https://htmx.org/headers/hx-trigger/> · <https://htmx.org/attributes/hx-swap-oob/>
- `ammonia` `Builder::clean_content_tags` & `tags`: <https://docs.rs/ammonia/latest/ammonia/struct.Builder.html>
- Planned tasks & design:
  - [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md)
  - [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md)
  - [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03a-api-contract-review.md)
  - [03a-design-review-log.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03a-design-review-log.md)
