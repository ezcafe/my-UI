# Analysis: Details Page Date Format (`DD/MM/YYYY`) and Longer AI Rewrite (3–5 Sentences)

**Result:** done

## Context

This task updates two parts of the article reading experience in `my-ui`, following the confirmed choices in [01-idea.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/01-idea.md):

1. **Details Page Date Format (`DD/MM/YYYY`)**:
   - Currently, `get_entry_detail_htmx` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1407-L1411) passes Miniflux's raw `publishedAt` RFC 3339 string (for example, `"2026-11-19T12:00:00Z"`) directly into `EntryViewModel.published_at`, and [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L74-L90) renders `<span>{{ entry.published_at }}</span>` in both the hero-image header and standard header.
   - `EntryDetailTemplate` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L116-L128) is instantiated in 1 production location (`get_entry_detail_htmx`) and 9 unit tests in `templates.rs` (including `test_detail_template_omits_author` with `"2 hours ago"`).
   - Client-side detail caching in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L616-L654) and [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L975-L988) swaps HTML into `#detail-panel` via HTMX (`htmx:afterSwap`), stores `panel.innerHTML` in IndexedDB (`my_news_article_cache_db`), and restores `panel.innerHTML = cached.html` on cache hits (`openEntryDetail`).
   - `ai_grouped_list.html` is out of scope per user confirmation; `entry_list.html` and `entry_item.html` do not render `published_at`.

2. **Longer AI Rewrite (3–5 Full Sentences) & Input Truncation**:
   - In [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L83-L100), `format_batch_prompt_structured` (used by background page prefetch and `LlmHttpClient`/`LlmCliRunner` batch calls) includes `"strictly 3 to 5 sentences"`, **but** `REWRITE RULES FOR "r"` restricts output to `"under 110 words total (sweet spot 30-60 words)"`.
   - In [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L166-L185), `format_rewrite_and_analyze_prompt` (used on-demand by `/hx/entries/{id}/rewrite`) does not require `"3 to 5 sentences"` and uses the same `"strictly under 110 words total (sweet spot 30-60 words)"` cap.
   - In [extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs#L41-L79), `ArticleExtractor` truncates extracted text to `100` words in `parse_html` and `extract_content`, and also runs `miniflux_client::summary::extract_summary(teaser, 3)` before the `>= 50` word fast-path check and HTTP fallback. For feeds with rich RSS teasers or full `<content>`, stopping at 3 sentences can starve a 3–5 sentence (~90–150 word) rewrite even after raising the word limit from 100 to 250.
   - `LlmBackend::rewrite_and_analyze` HTTP vs. CLI routing and PostgreSQL schema (`article_ai_summaries.rewrite TEXT`) remain unchanged.

## Risks

1. **Timezone Drift Between Server Fallback and Browser Local Time**:
   - **Risk**: If server fallback formats using server `Local` timezone, tests and Docker containers in UTC vs. `+07:00` behave differently for midnight timestamps (`2026-11-24T20:00:00Z`).
   - **Mitigation**: Server fallback uses `DateTime::parse_from_rfc3339` and formats in the timestamp's parsed offset (`dt.format("%d/%m/%Y")`) deterministically, while `<time datetime="..." data-date-format="dd/mm/yyyy">` lets browser JS format into the user's exact local calendar date (`new Date(raw)`).

2. **Date-Only ISO Strings Shifting Back One Day in Browser JS**:
   - **Risk**: `new Date("2026-11-24")` parses as UTC midnight per ECMAScript, which shifts to `23/11/2026` in negative UTC offsets.
   - **Mitigation**: Check `/^(\d{4})-(\d{2})-(\d{2})$/` first in client JS and format directly as `DD/MM/YYYY`, and format `DateTime` timestamps with `new Date(raw)` only when `!Number.isNaN(d.getTime())`.

3. **IndexedDB Cached Detail HTML Skipping Date Formatting**:
   - **Risk**: If date formatting only runs on `htmx:afterSwap`, cached articles restored in `openEntryDetail(id)` or cached before timezone changes would show stale text.
   - **Mitigation**: Run `formatDetailDates(panel)` in **both** `htmx:afterSwap` (before `saveCachedArticle`) and `openEntryDetail` cache-hit restoration.

4. **Test & Fallback Compatibility on Non-ISO Strings and Existing Tests**:
   - **Risk**: `test_detail_template_omits_author` passes `"2 hours ago"`, and 9 `EntryDetailTemplate` tests in `templates.rs` construct the struct directly; existing tests in `llm.rs` and `extractor.rs` assert `"under 110 words"`, `"30-60 words"`, `"Extra sentence 4"`, and `100` words.
   - **Mitigation**: Fall back to raw `published_at` text when Chrono parsing fails; update `llm.rs` and `extractor.rs` unit tests to assert the new 90–150/max-180 word budget and 250-word extraction cap while preserving non-word-cap assertions (`"strictly 3 to 5 sentences"`, `"inverted pyramid rewrite"`, `"first 2 sentences"`, `"2-3 bullet points"`).

5. **RSS Teaser 3-Sentence Pre-Truncation Bottleneck**:
   - **Risk**: Keeping `extract_summary(teaser, 3)` before `truncate_words(..., 250)` in `ArticleExtractor::extract_content` clips rich RSS feeds to ~50–60 words on the fast-path.
   - **Mitigation**: Switch `ArticleExtractor` teaser cleaning to `miniflux_client::summary::strip_html_tags(teaser)` followed by `Self::truncate_words(..., 250)` so both HTML extraction and teaser fast-path/fallback provide up to 250 words.

## Open questions

None — all user-facing choices (Option 1A, Option 2A, details-only scope, inverted-pyramid lead + 2–3 bullets) were confirmed in ideation, and remaining implementation details are ready for Grill and Design.
