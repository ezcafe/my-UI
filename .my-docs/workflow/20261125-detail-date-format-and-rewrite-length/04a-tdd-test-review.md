Result: clean

# TDD test plan review

**Stage:** `tdd-review` (`my-dev-flow-v3-code`)  
**Scope:** Review of planned Red-Green test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/04-tasks.md) against [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/03-design.md) and [03a-design-review-log.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/03a-design-review-log.md).

## Key Takeaway

All 7 planned test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/04-tasks.md) are specific, follow strict Red-before-Green ordering, map cleanly to existing test suites, and cover the required edge cases without missing tests. Ready for Gate B.

## Planned cases

| Case | Test name | Coverage & edge-case check |
|------|-----------|----------------------------|
| Server date helper | `test_format_published_date_ddmmyyyy_parses_rfc3339_and_falls_back` | Covers RFC 3339 (`Z`, explicit `-07:00` offset, fractional seconds), `YYYY-MM-DD`, empty/whitespace-only (`""`), and non-ISO fallback (`"2 hours ago"`) in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs). |
| Detail template headers | `test_entry_detail_renders_ddmmyyyy_time_element_in_both_headers` | Covers `<time datetime="..." data-date-format="dd/mm/yyyy">DD/MM/YYYY</time>` in both hero (`hero_image: Some(...)`) and standard (`hero_image: None`) branches in [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html), plus omitting `•` and `<time>` when `published_at` is empty/whitespace. |
| Client swap & cache restore JS | `test_index_template_formats_detail_dates_on_swap_and_cache_restore` | Verifies `formatDetailDates(root)` in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) handles `YYYY-MM-DD` and guarded ISO date-times, and runs in both `htmx:afterSwap` (before `saveCachedArticle`) and `openEntryDetail` (`isCacheValid` cache hit). |
| Route integration | `test_get_entry_detail_formats_published_at_as_ddmmyyyy` | Verifies `/hx/entries/{id}` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) renders `<time datetime="..." data-date-format="dd/mm/yyyy">DD/MM/YYYY</time>` from Miniflux RFC 3339 `publishedAt`. |
| Batch rewrite prompt | `test_batch_prompt_rewrite_rules_word_count_and_inverted_pyramid` | Updates existing assertion in [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) for `strictly 3 to 5 sentences`, `90-150 words` / `max 180 words`, inverted-pyramid lead (first 2 sentences) + 2–3 bullet points, and absence of legacy `30-60 words` / `under 110 words`; preserves substrings checked by lines 1621–1640. |
| Single-article rewrite prompt | `test_rewrite_and_analyze_prompt_rules` | Updates existing assertion in [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) for `strictly 3 to 5 sentences`, `90-150 words` / `max 180 words`, inverted-pyramid lead (first 2 sentences) + 2–3 bullet points, and absence of legacy `30-60 words` / `under 110 words`. |
| Extractor 250-word cap & teaser | `test_extractor_word_truncation_250` & `test_extract_article_fallback_on_http_error_or_block` | Updates [extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs) truncation test from `100` to `250` words and updates HTTP 403 / fast-path teaser assertions to verify `strip_html_tags` preserves content past 3 sentences up to 250 words. |

## Implementation Notes for Build

- Keep `test_get_entry_detail_formats_published_at_as_ddmmyyyy` alongside `test_get_entry_detail_route_loads_rewrite_from_ai_db` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) (or reuse the mock setup helper in [routes/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs)).
- Ensure `test_extract_article_fallback_on_http_error_or_block` (or an accompanying assertion) tests HTML tags in a >3-sentence teaser so `miniflux_client::summary::strip_html_tags` + 250-word cap is verified end-to-end.
- Keep non-word-cap substrings (`strictly 3 to 5 sentences`, `inverted pyramid rewrite`, `first 2 sentences`, `2-3 bullet points`) intact in [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) per [03a-design-review-log.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/03a-design-review-log.md).

## External Reference Sources

- Chrono `0.4` `DateTime::parse_from_rfc3339` and `NaiveDate::parse_from_str`: https://docs.rs/chrono/latest/chrono/
- Askama `0.12` auto-escaping on `{{ self.method() }}` expressions: https://docs.rs/askama/0.12.1/askama/
- MDN `<time>` element and `datetime` attribute specification: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time
