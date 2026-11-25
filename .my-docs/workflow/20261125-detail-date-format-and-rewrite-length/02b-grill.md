# Grill log

**Result:** frontier-empty

## Round log

### Round 1

All initial product and UX decisions were confirmed during ideation (`Notes: ideation-confirmed`), leaving no open user-facing ambiguities for Round 1. Each design-tree node below records the settled position and rationale.

---

❓ **Q1** - **Details Page Date Scope (`DD/MM/YYYY`)**: Should `DD/MM/YYYY` formatting apply strictly to the article details page ([entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)) and not [ai_grouped_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html)?

➡️ **Settled**: Yes — details page only (both hero-image header and standard header). `ai_grouped_list.html` and `EntryViewModel.published_at` raw value remain untouched.

---

❓ **Q2** - **Timezone & Fallback Strategy (Option 1A)**: How should `published_at` handle server rendering, non-ISO test strings, and browser local timezones (including IndexedDB cached details)?

➡️ **Settled**: Use Option 1A confirmed by user:
- Server renders deterministic `DD/MM/YYYY` fallback via `chrono::DateTime::parse_from_rfc3339` (with `NaiveDate` `%Y-%m-%d` fallback and raw string passthrough if unparseable) inside `<time datetime="{{ entry.published_at }}" data-date-format="dd/mm/yyyy">...</time>`.
- Client JS in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) updates `<time data-date-format="dd/mm/yyyy">` to the browser's local timezone on both `htmx:afterSwap` (before IndexedDB save) and IndexedDB cache-hit restore in `openEntryDetail`.

---

❓ **Q3** - **Rewrite Length & Structure (Option 2A)**: How should batch and single-article rewrite prompts align on sentence count, word budget, and structure?

➡️ **Settled**: Use Option 2A + inverted-pyramid structure confirmed by user:
- Both `format_batch_prompt_structured` and `format_rewrite_and_analyze_prompt` in [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) specify **strictly 3 to 5 sentences** (full, substantive sentences), word budget **around 90-150 words total (max 180 words)**, and **lead with outcome and key impacts in first 2 sentences, followed by 2-3 bullet points (complete sentences) with essential context**.

---

❓ **Q4** - **`ArticleExtractor` Input Budget & Teaser Pre-Truncation**: Should `ArticleExtractor` raise `truncate_words` from `100` to `250` words and also replace `extract_summary(teaser, 3)` in `extract_content` / fallback with HTML-stripped text capped at 250 words?

➡️ **Settled**: Yes — raise `MAX_EXTRACTED_WORDS` to `250` across both `parse_html` and `extract_content` in [extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs), and use `miniflux_client::summary::strip_html_tags(teaser)` before 250-word truncation so rich RSS feeds are not artificially clipped at 3 sentences before reaching the LLM.

---

### Round 2 (Design Backloop — Guest Details & Tab View Rules)

New user requirements added via `design-backloop`:
1. When authentication is false (`auth_enabled == false`, guest mode), do NOT show `author` or `readingTime` on the details page; show title + content from the list page, and do NOT call `GET /hx/entries/{id}`.
2. Clicking on **Unread** filter tab always shows **Magazine view**.
3. Clicking on **Today** and **All** filter tabs always show **List view**.

---

❓ **Q5** - **Guest Mode (`auth_enabled == false`) Details Content**: When rendering the details drawer purely client-side from list-page data without calling `GET /hx/entries/{id}`, what content should be displayed below the title (and omitting `author` and `readingTime`)?

➡️ **Settled (Option A — User confirmed)**:
- Render `title` + server-sanitized Miniflux `content` (sanitized via `sanitize_html` in [sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs), falling back to sanitized list-page `summary` only if Miniflux `content` is empty) + `feed_title` + `DD/MM/YYYY` date + `Original` link + local read/bookmark controls directly from list-page data and IndexedDB `all_articles`.
- Emit **zero** `GET /hx/entries/{id}`, `/hx/entries/{id}/analysis`, or `/hx/entries/{id}/rewrite` requests in guest mode.
- Strictly omit `author` and `readingTime`.

---

❓ **Q6** - **Server Route `GET /hx/entries/{id}` & Cold Deep Links (`/?entry={id}` / `/entries/{id}`) in Guest Mode**: Should `GET /hx/entries/{id}` only be skipped by frontend JS, or also disabled on the server when `auth_enabled == false`?

➡️ **Settled (Option A — User confirmed)**:
- Skip all frontend `GET /hx/entries/{id}` (and `/analysis` / `/rewrite`) calls in guest mode (`!AUTH_ENABLED`), including showing a friendly inline unavailable notice with a `Back to List` action on cold deep-link cache misses (`/?entry={id}` or `/entries/{id}`).
- Keep the server route `get_entry_detail_htmx` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) active for direct calls and existing server/E2E tests, while explicitly omitting `author` (`None`) and `reading_time` (`0`) when `!config.auth_enabled` (and [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html) already omits both).

---

❓ **Q7** - **Tab View Locking (**Unread** → Magazine, **Today**/**All** → List) and Header Toggle (`#ai-toggle-btn`)**: How should the header Magazine/List toggle button (`#ai-toggle-btn`), initial load, `Starred` (auth mode), and search interact with the new tab view rules?

➡️ **Settled (Option A + Hide `#ai-toggle-btn` on Today and All — User confirmed)**:
- **Initial load / Unread**: Always show **Magazine view** (`'ai'`) and display `#ai-toggle-btn`.
- **Today and All**: Always force **List view** (`'list'`) — across tab clicks, cache restores, pagination, and guest `renderLocalAllArticles()` (wrapped in `#entries-list-view`) — AND **hide `#ai-toggle-btn`**.
- **Starred (auth mode) and Search**: Default to **List view** (`'list'`) with `#ai-toggle-btn` visible so the user can manually switch views within Unread, Starred, or Search.

---

## Frontier Status

`Result: frontier-empty` — All Round 1 (Q1–Q4) and Round 2 (Q5–Q7) design-tree decisions are settled. Ready for `design-update` to revise `03-design.md` and `04-tasks.md`.
