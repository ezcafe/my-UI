# Tasks (TDD-ready)

Each task: behavior, failing test name first, files allowed, done evidence.

## Planned tests

<!-- ≥1 named case → planned tests present (Prefer tdd-review). Empty / N/A → not present. -->

| Case | Failing test name | Covers |
|------|-------------------|--------|
| Date helper formats RFC 3339 (UTC, offset, fractional seconds), `YYYY-MM-DD`, empty, and non-ISO fallback *(already implemented in Round 1)* | `test_format_published_date_ddmmyyyy_parses_rfc3339_and_falls_back` | `format_published_date_ddmmyyyy` in `crates/ui-components/src/templates.rs` |
| Detail template renders `<time datetime="..." data-date-format="dd/mm/yyyy">DD/MM/YYYY</time>` in both headers *(already implemented in Round 1)* | `test_entry_detail_renders_ddmmyyyy_time_element_in_both_headers` | `EntryDetailTemplate` + `entry_detail.html` in `crates/ui-components` |
| `IndexTemplate` wires `formatDetailDates` in `htmx:afterSwap` and `openEntryDetail` cache-hit *(already implemented in Round 1)* | `test_index_template_formats_detail_dates_on_swap_and_cache_restore` | `index.html` JS hooks in `crates/ui-components` |
| `/hx/entries/{id}` renders `DD/MM/YYYY` `<time>` element *(already implemented in Round 1)* | `test_get_entry_detail_formats_published_at_as_ddmmyyyy` | `get_entry_detail_htmx` in `crates/server/src/routes/entries.rs` |
| Batch & single rewrite prompts align on 3–5 sentences, `90-150 words` / `max 180 words`, inverted-pyramid *(already implemented in Round 1)* | `test_batch_prompt_rewrite_rules_word_count_and_inverted_pyramid` & `test_rewrite_and_analyze_prompt_rules` | `crates/server/src/ai/llm.rs` |
| `ArticleExtractor` truncates HTML and rich RSS teasers to 250 words *(already implemented in Round 1)* | `test_extractor_word_truncation_250` & `test_extract_article_fallback_on_http_error_or_block` | `crates/server/src/ai/extractor.rs` |
| Guest list JSON data block embeds sanitized `content`, `feed_title`, `formatted_date`, `url`, omits `author`/`reading_time`, escapes `</script>`/`<`/`>`/`&`, and is absent when `sanitized_content` is `None` | `test_entry_list_guest_json_embeds_sanitized_content_and_escapes_script_tags` | `EntryListTemplate::guest_entries_json` + `entry_list.html` in `crates/ui-components` |
| Guest detail JS (`auth_enabled: false`) defines `AUTH_ENABLED`, `openGuestEntryDetail`, `renderGuestEntryDetail`, `safeHref`, skips `/hx/entries/` fetches, omits `author`/`readingTime`/`/analysis`, handles cold deep-link misses, and wraps `renderLocalAllArticles` in `#entries-list-view` | `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` | `index.html` guest detail & `All` list scripts in `crates/ui-components` |
| Initial load and `Unread` force Magazine (`'ai'`); `Today` and `All` force List (`'list'`) and hide `#ai-toggle-btn`; `Starred` and `Search` default to List with `#ai-toggle-btn` visible | `test_index_template_tab_view_rules_and_toggle_visibility` (plus updating `test_index_template_renders_ai_toggle_button_and_scripts`) | `index.html` tab view controller in `crates/ui-components` |
| Server list routes (`index_page` and `/hx/entries`) embed `#entries-guest-data` with `ammonia`-sanitized Miniflux `content` (preserving sanitized HTML, stripping `<script>`/`onerror`) and omit `author`/`readingTime` in guest mode while omitting `#entries-guest-data` in auth mode | `test_list_routes_embed_sanitized_guest_data_only_when_auth_disabled` | `index_page` (`pages.rs`) & `render_entries_list_html` (`entries.rs`) |
| `GET /hx/entries/{id}` remains active when `auth_enabled = false` and never renders `author` or `readingTime` | `test_get_entry_detail_guest_mode_keeps_route_active_and_omits_author_and_reading_time` | `get_entry_detail_htmx` in `crates/server/src/routes/entries.rs` |

---

## Implementation Tasks

### Phases 1–3: Round 1 Baseline *(already implemented prior to backloop — verify unchanged)*
- Date helper `format_published_date_ddmmyyyy`, `EntryDetailTemplate::formatted_published_at`, `<time data-date-format="dd/mm/yyyy">` in `entry_detail.html`, `formatDetailDates` in `index.html`.
- Aligned 3–5 sentence (`90-150 words`, `max 180 words`) inverted-pyramid prompts in `crates/server/src/ai/llm.rs` and `MAX_EXTRACTED_WORDS = 250` + `strip_html_tags` in `crates/server/src/ai/extractor.rs`.

### Phase 4: TDD Red (Backloop Round 2) — Tests First
1. **Task 4.1 (UI unit tests for guest data block, guest detail renderer, and tab view rules)**:
   - Add `test_entry_list_guest_json_embeds_sanitized_content_and_escapes_script_tags`, `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time`, and `test_index_template_tab_view_rules_and_toggle_visibility` in `crates/ui-components/src/templates.rs`.
   - Update `test_index_template_renders_ai_toggle_button_and_scripts` (replace obsolete `savedMode || 'ai'` check with initial `'ai'` Magazine view check) and supply `sanitized_content: None` on existing `EntryViewModel` test literals.
   - **Files allowed**: `crates/ui-components/src/templates.rs`
   - **Done evidence**: New/updated UI tests fail before implementation.
2. **Task 4.2 (Server integration tests for guest list payload & detail omission)**:
   - Add `test_list_routes_embed_sanitized_guest_data_only_when_auth_disabled` and `test_get_entry_detail_guest_mode_keeps_route_active_and_omits_author_and_reading_time` in `crates/server/src/routes/entries.rs` (testing both `index_page` and `/hx/entries`).
   - **Files allowed**: `crates/server/src/routes/entries.rs`
   - **Done evidence**: New server integration tests fail before implementation.

### Phase 5: TDD Green (Backloop Round 2) — Guest List Payload & Client-Side Detail View
3. **Task 5.1 (`ui-components` guest payload & `entry_list.html`)**:
   - Move `serde_json = "1.0"` from `[dev-dependencies]` to `[dependencies]` in `crates/ui-components/Cargo.toml`.
   - Add `#[serde(default)] pub sanitized_content: Option<String>` to `EntryViewModel`, implement `EntryListTemplate::guest_entries_json` with WHATWG `<script>` escaping (`\u003c`, `\u003e`, `\u0026`, `\u2028`, `\u2029`), and render `<script id="entries-guest-data" type="application/json">{{ guest_json|safe }}</script>` after `#entries-magazine-view` in `crates/ui-components/templates/partials/entry_list.html`.
   - **Files allowed**: `crates/ui-components/Cargo.toml`, `crates/ui-components/src/templates.rs`, `crates/ui-components/templates/partials/entry_list.html`
   - **Done evidence**: `test_entry_list_guest_json_embeds_sanitized_content_and_escapes_script_tags` and existing `EntryListTemplate` tests pass.
4. **Task 5.2 (Server list & detail handlers)**:
   - Update `index_page` in `crates/server/src/routes/pages.rs` and `render_entries_list_html` in `crates/server/src/routes/entries.rs` to populate `sanitized_content: Some(ui_components::sanitizer::sanitize_html(raw_body))` (using `content_str`, falling back to raw `summary_str` before AI overwrite), `reading_time: 0`, and `author: None` when `!auth_enabled` (and `sanitized_content: None` when `auth_enabled`).
   - Update remaining `EntryViewModel` initializers in `crates/server/src/routes/entries.rs` (`sanitized_content: None`), and set `reading_time: 0` + `author: None` in `get_entry_detail_htmx` when `!config.auth_enabled`.
   - **Files allowed**: `crates/server/src/routes/pages.rs`, `crates/server/src/routes/entries.rs`
   - **Done evidence**: `test_list_routes_embed_sanitized_guest_data_only_when_auth_disabled`, `test_get_entry_detail_guest_mode_keeps_route_active_and_omits_author_and_reading_time`, and all existing server/E2E tests pass.
5. **Task 5.3 (Frontend guest detail renderer & IndexedDB `all_articles` in `index.html`)**:
   - Add `const AUTH_ENABLED = {{ auth_enabled }};`, `guestEntriesMap`, `readGuestEntriesJson()`, `safeHref()`, `getGuestArticle(numId)`, `openGuestEntryDetail(numId)`, and `renderGuestEntryDetail(entry)` in `crates/ui-components/templates/pages/index.html`.
   - Short-circuit `openEntryDetail(id)` when `!AUTH_ENABLED` before any `GET /hx/entries/` or `my_news_article_cache_db` call, update `cacheLoadedArticlesFromDom()` to persist `{ id, title, url, feed_title, published_at, formatted_date, sanitized_content, saved_at }`, and wrap `renderLocalAllArticles()` rows in `<div id="entries-list-view" class="space-y-1.5">...</div>`.
   - **Files allowed**: `crates/ui-components/templates/pages/index.html`
   - **Done evidence**: `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` passes.

### Phase 6: TDD Green (Backloop Round 2) — Tab View Locking & `#ai-toggle-btn` Visibility
6. **Task 6.1 (Tab view controller & toggle hiding in `index.html`)**:
   - Implement `currentViewMode`, `isSearchActive`, `shouldHideAiToggle`, `getDefaultViewForState`, and `updateAiToggleVisibility` in `crates/ui-components/templates/pages/index.html`.
   - Wire across `initAiViewMode()` (always `'ai'` on initial load), `toggleAiView()`, `handleFilterClick()`, `htmx:beforeRequest`, `applyCachedList()`, `onSearchSubmit()`, `renderLocalAllArticles()`, and `htmx:afterSwap`.
   - **Files allowed**: `crates/ui-components/templates/pages/index.html`
   - **Done evidence**: `test_index_template_tab_view_rules_and_toggle_visibility` and `test_index_template_renders_ai_toggle_button_and_scripts` pass.

### Phase 7: Verification
7. **Task 7.1 (Full workspace test & lint pass)**:
   - Run `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check` across the workspace.
   - **Files allowed**: None (verification only)
   - **Done evidence**: Zero test failures, zero clippy warnings, formatting clean.
