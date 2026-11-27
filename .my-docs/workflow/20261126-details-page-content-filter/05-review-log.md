# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

## Adversarial

**Result:** clean

### Findings
None — all Round 1 Adversarial findings (`1` Major, `2` Minor) were verified resolved after `fix-review` (`verify-pass`).

### Round 1 Resolution Verification
1. **[RESOLVED — MAJOR] Cross-article race condition on `#ai-analysis-box` OOB and Retry `#ai-rewrite-box` swaps:**
   - **Verified in:** [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L88-L128), [analysis_cards.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards.html#L1-L3), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1738-L1758), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1819-L2048), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L916-L922), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1493-L1500), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1555-L1706).
   - **Fix confirmed:** `data-target-entry-id` tags `#ai-rewrite-box` and `#ai-analysis-box` (server templates, OOB fragments, and client-rendered/error fragments); `abortActiveDetailRequests()` aborts active detail `#ai-rewrite-box`, `#ai-analysis-box`, and `button[hx-get]` requests on detail switch/close; `htmx:beforeSwap` and `htmx:oobBeforeSwap` reject mismatched or detached detail swaps via `detail.shouldSwap = false`; `htmx:afterSwap` verifies `reqPath` entry ID matches `#detail-panel [data-entry-id]` before `recordReadArticle`, Guest `rewrite` persistence, or Auth `saveCachedArticle`.
2. **[RESOLVED — MINOR] `/hx/executive-brief` `BriefSyncEntry.summary` plain-text sanitization:**
   - **Verified in:** [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L704-L711), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L915), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1160-L1168).
   - **Fix confirmed:** `clean_opt_summary` routes both cached and freshly synthesized sync summaries through `ui_components::sanitizer::extract_plain_summary(..., "")`.
3. **[RESOLVED — MINOR] Case-insensitive `http://`/`https://` checks in `safeHref` and `EntryDetailTemplate::safe_entry_url`:**
   - **Verified in:** [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L323-L335), [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L157-L170).
   - **Fix confirmed:** Both use lowercase scheme comparisons (`trimmed.toLowerCase()` / `trimmed.to_ascii_lowercase()`), matching `extract_article_images`.

### Non-Blocking Observation
- **Auth outer `#detail-panel` fetch (`openEntryDetail`):** Pre-existing `htmx.ajax('GET', '/hx/entries/' + encodeURIComponent(numId), { target: '#detail-panel', swap: 'innerHTML' })` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1019-L1022)) is not itself aborted by `abortActiveDetailRequests()` or guarded in `htmx:beforeSwap` on `tid === 'detail-panel'` if an Auth user opens an uncached article and immediately opens a cached one or closes the drawer; `afterSwap` already checks `reqPath` against the live `[data-entry-id]` so no cross-article cache corruption occurs.

### Verified Clean Checks
- No original article body HTML (`#guest-detail-content` / `sanitized_content`) or metadata (`feed_title`, `<time>`, `author`, `reading_time`) leaks across Auth, Guest, or skeleton views.
- `extract_plain_summary` handles block-tag spacing, `ammonia` `<script>`/`<style>` removal, single-pass entity decoding (`&amp;` last), and `.char_indices()` truncation safely on multi-byte UTF-8 input.
- `EntryListTemplate::guest_entries_json()` gates `#entries-guest-data` on `!self.auth_enabled && !self.entries.is_empty()` and WHATWG-escapes JSON inside `<script>`.
- Error handling on `#ai-rewrite-box` avoids `hx-trigger="load"` retry loops via `outerHTML` replacement, preserves `data-fallback-summary` across retries, and prevents Guest `.is-error` persistence or Auth skeleton caching.
- Workspace checks verified in review: `cargo test -p ui-components` (`90 passed, 0 failed`), `cargo clippy --workspace --all-targets -- -D warnings` (`0 warnings`), `cargo test -p server --lib --no-run` (compiled cleanly), and `rustfmt --check --edition 2021` on all modified Rust files (passed). Wiremock-based `miniflux-client`/`server` tests require loopback TCP outside the offline review sandbox and passed `269/269` at `fix-review` (`verify-pass`, no subsequent code edits).

## Quality

**Result:** clean

### Round 1 Resolution Verification
1. **Shared `apply_ai_summaries_to_entries` helper:** Extracted in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L98-L116) and reused by both [pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs#L163) and `render_entries_list_html` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L450)).
2. **Shared `extract_images_with_summary_fallback` helper:** Extracted in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L84-L95) and reused in `entry_view_model_from_json` and `get_entry_detail_htmx`.
3. **Shared `append_brief_oob_updates` & `clean_opt_summary` helpers:** Extracted in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L704-L735) and reused across cached and fresh brief OOB assembly.
4. **Shared Guest IndexedDB helpers:** `upsertGuestArticleInDb` and `buildGuestArticleRecord` extracted in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1734-L1768) and reused across `syncExecutiveBriefUpdates`, `htmx:afterSwap`, and `cacheLoadedArticlesFromDom`.
5. **Dead production helpers resolved:** `format_published_date_ddmmyyyy` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L142)), `sanitize_html` / `RE_ANCHOR` / `RE_TARGET` ([sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs#L6-L20)), and `extract_first_image_url` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L48)) are gated under `#[cfg(test)]`, sharing module-level `IMG_REGEX`.

### Five-Axis & UI Craft Evaluation
- **Correctness:** Clean — all Round 1 functional, race-condition, sanitization, and URL-parity issues are fixed and covered by unit/route/E2E tests.
- **Readability & Architecture:** Clean — duplicated logic extracted into concise single-purpose helpers; shared [analysis_cards_inner.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards_inner.html) avoids template drift.
- **Security:** Clean — `ammonia` 4.x HTML stripping, Askama escaping, `escape_json_for_html_script`, `escape_html`, `http://`/`https://` image filtering, `safe_entry_url` / `safeHref`, and `AUTH_ENABLED` gating are consistent.
- **Performance:** Clean — `LazyLock` compiled regexes, single LLM pass for `/rewrite` + OOB `/analysis`, and clean IndexedDB v2/v3 store resets.
- **UI Craft (`Has UI = yes`):** Clean — clear detail hierarchy (top action bar -> hero/title -> rewrite/fallback -> body gallery -> analysis cards), consistent OKLCH tokens, container-query responsiveness, lazy/no-referrer image handling with `onerror` hide, and clear unavailable/generating copy.

### Reference Sources
- `htmx` 1.9 `htmx:beforeSwap`, `htmx:oobBeforeSwap`, `htmx:abort`, `HX-Trigger-After-Settle`: <https://htmx.org/events/> · <https://htmx.org/headers/hx-trigger/>
- `ammonia` 4.x `Builder::tags`: <https://docs.rs/ammonia/latest/ammonia/struct.Builder.html#method.tags>
- `askama` 0.12 docs: <https://docs.rs/askama/0.12.1/askama/>

## Merged

**Result:** clean
- Plan 1 copy from `05-lens-api.md`: `clean`
