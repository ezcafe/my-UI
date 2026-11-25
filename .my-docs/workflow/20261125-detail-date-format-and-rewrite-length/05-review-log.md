# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

## Adversarial

**Result:** clean

### Re-Verification of Round 1 Fixes
1. **`initGuestDb()` cold deep-link `readSet` preservation ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1335-L1353))**:
   - Verified `getAllReq.onsuccess` now merges loaded IndexedDB keys via `(ev.target.result || []).forEach(id => readSet.add(id))` instead of reassigning `readSet = new Set(...)`, preventing concurrent `openGuestEntryDetail()` calls from losing read state on cold `/?entry={id}` or `/entries/{id}` loads.
   - Guarded by `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3676-L3685).
2. **`renderLocalAllArticles()` empty-state isolation ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1437-L1475), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1572-L1591))**:
   - Verified `renderLocalAllArticles()` renders `#local-all-empty-state` when `articles.length === 0`, so `syncDomReadStates()` only manages `#client-empty-state` for the Unread empty filter and no longer deletes the `All` tab empty state.
   - Guarded by `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3669-L3673).

### Verified Adversarial Checks
- **WHATWG `<script type="application/json">` data-block escaping**: `escape_json_for_html_script()` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L165-L171) escapes `&`, `<`, `>`, `\u{2028}`, and `\u{2029}`, eliminating `<!--`, `<script`, and `</script` sequences per WHATWG HTML §4.12.1.3.
- **HTML & URL sanitization**: `entry_view_model_from_json()` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L179-L247) selects `content` (falling back to raw `summary` before AI overwrite) and sanitizes via `ui_components::sanitizer::sanitize_html` (`ammonia 4.2.5`, latest stable release in `Cargo.lock`) only when `!auth_enabled`; `safeHref()` in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1140-L1151) restricts guest links to `http://`, `https://`, or single-leading-`/` paths and escapes attribute/text values via `escapeHtml()`.
- **Guest detail isolation**: `openEntryDetail(id)` short-circuits to `openGuestEntryDetail(numId)` before `getCachedArticle(numId)` and `htmx.ajax` when `!AUTH_ENABLED`; `GuestEntryPayload` and `renderGuestEntryDetail(entry)` omit `author`, `reading_time`/`readingTime`, `/analysis`, and `/rewrite`; `get_entry_detail_htmx` omits `author` (`None`) and sets `reading_time: 0` when `!config.auth_enabled`.
- **Tab view state machine & HTMX trigger detection**: `htmx:afterSwap` in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1021-L1077) inspects `evt.detail.requestConfig.elt` so `.filter-pill` clicks reset `isSearchActive = false` (`Unread` → Magazine + visible `#ai-toggle-btn`; `Today`/`All` → List + hidden `#ai-toggle-btn`; `Starred` → List + visible `#ai-toggle-btn`), `#search-form` sets `isSearchActive` (`Search` → List + visible `#ai-toggle-btn`), and pagination/brief swaps preserve `currentViewMode` unless `Today`/`All` locks List.
- **Date formatting & timezone handling**: `format_published_date_ddmmyyyy()` (`chrono 0.4.45`) handles RFC 3339, `YYYY-MM-DD`, empty, and non-ISO fallback; `formatDetailDates(root)` guards date-only strings against UTC day shifts and formats `<time datetime="..." data-date-format="dd/mm/yyyy">` in browser local timezone across HTMX swaps, IndexedDB cache restores, and guest detail rendering.

---

## Quality

**Result:** clean

### Five-Axis Summary
1. **Correctness**: All Round 1 and Round 2 requirements plus `fix-review` fixes verified across workspace tests (`cargo test --workspace`: 24 `miniflux-client` + 140 `server` lib + 8 `server` `e2e_tests` + 76 `ui-components` = 248 passed, 0 failed), `cargo clippy --all-targets` (0 warnings), and `cargo fmt --check` (clean).
2. **Readability**: Clear, single-purpose helpers (`entry_view_model_from_json`, `format_published_date_ddmmyyyy`, `escape_json_for_html_script`, `shouldHideAiToggle`, `getDefaultViewForState`, `openGuestEntryDetail`, `clean_teaser`).
3. **Architecture**: Re-verified Round 1 quality fixes:
   - Extracted shared `entry_view_model_from_json` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L179-L247) and reused it in both [pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs#L149) and [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L405), eliminating the 65-line duplication.
   - Simplified `EntryListTemplate::guest_entries_json()` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L174-L198) to collect `payloads` once via `.filter_map(...)` and check `payloads.is_empty()` without a redundant pre-scan.
4. **Security**: Clean — `ammonia 4.2.5` (latest stable release with CSS/URL fixes), WHATWG script data-block escaping, `escapeHtml()` + `safeHref()` on guest DOM interpolation, no `author`/`readingTime` leakage.
5. **Performance**: `sanitized_content` is populated only when `!auth_enabled`; `hydrateGuestEntriesFromDom()` parses `#entries-guest-data` once per swap; `All` tab reads local IndexedDB without server round-trips.

### UI Craft Check (`Has UI = yes`)
- Reuses Annuaire design tokens (`bg-card`, `border-border`, `prose dark:prose-invert`, `max-w-[44rem] mx-auto`), semantic `<time>` with automatic `tabular-nums lining-nums` from [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L187-L190), no hardcoded viewport breakpoints, and removes `#ai-toggle-btn` from layout and tab order via Tailwind `hidden` on `Today` and `All`.

### Reference Sources (Live Web Search Verified)
- WHATWG HTML §4.12.1.3 Restrictions for contents of script elements: https://html.spec.whatwg.org/multipage/scripting.html#restrictions-for-contents-of-script-elements
- `ammonia` `4.2.5` crate docs & releases: https://docs.rs/ammonia/ · https://github.com/rust-ammonia/ammonia
- `chrono` `0.4.45`, `serde_json` `1.0.154`, `askama` `0.12.1` crate docs: https://docs.rs/chrono/ · https://docs.rs/serde_json/ · https://docs.rs/askama/
- htmx events (`htmx:beforeRequest`, `htmx:afterSwap` `requestConfig`): https://htmx.org/reference/#events

---

## Merged

**Result:** skipped  
(`Lens plan: none`; `Has API = no`, `Has DB = no`)
