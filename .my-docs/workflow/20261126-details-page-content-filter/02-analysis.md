# Analysis — Details Page Content Filter

Result: done

## Key Takeaways

1. **Both rendering paths leak original content and metadata today:** Authenticated mode (`entry_detail.html` + `get_entry_detail_htmx`) falls back to raw Miniflux `summary`/`content` when no AI rewrite exists and displays `feed_title` + `<time>` metadata. Guest mode (`renderGuestEntryDetail` in `index.html` + `GuestEntryPayload` in `templates.rs`) renders the full original article HTML in `#guest-detail-content` (`sanitized_content`) and displays `feed_title` + `<time>` metadata.
2. **Skeleton and IndexedDB caches also hold stale elements:** `openDetailWithSkeleton` (`index.html`) renders metadata skeleton bars (`Feed info & date`), and the 30-day IndexedDB detail cache (`my_news_article_cache_db` / `cached_articles`) plus guest `all_articles` cache store pre-change HTML/fields.
3. **List-to-detail sync and `/rewrite` contract gaps:**
   - Guest mode (`entry_view_model_from_json`) only extracts 1 photo (`extract_first_image_url`) instead of all photos (`extract_article_images`), and `EntryViewModel` / `GuestEntryPayload` do not carry `rewrite` or `body_images`.
   - `/hx/executive-brief` only emits OOB `#entry-title-{id}` for List View (`#entries-list-view`), leaving Magazine cards (`#mag-title-{id}`, `.magazine-brief`, P3 `title`) and `guestEntriesMap` stale after background AI finishes.
   - `get_entry_rewrite_htmx` (`GET /hx/entries/{id}/rewrite`) currently embeds Impact & Fallacy cards *inside* `#ai-rewrite-box` via `render_analysis_fragment`, returns an empty box when `!ai_enabled`, and prints unstripped raw Miniflux HTML via `escape_html` on failure.
4. **Has API flip (`no -> yes`), DB/auth unchanged:** Settled Grill answers (Q1=Option C, Q2=Yes, Q3=Yes) modify the HTTP response payloads of `/hx/entries/{id}`, `/hx/entries/{id}/rewrite`, `/hx/executive-brief`, and embedded `#entries-guest-data` JSON (`GuestEntryPayload`) across `crates/server` and `crates/ui-components`. Database schema (`article_ai_summaries`) and auth flows remain unchanged. Proposed flags: `Has UI=yes, Has API=yes, Has DB=no, Has auth/PII/schema=no`.

---

## Current Behavior Audit

### 1. Authenticated Mode (`AUTH_ENABLED = true`)
- **Route (`crates/server/src/routes/entries.rs:1307-1583` — `get_entry_detail_htmx`):**
  - Extracts `(hero_image, body_images)` from Miniflux `content` (line 1525), then checks `if hero_image.is_none() && !summary.is_empty()` — bug: `summary` at line 1526 was shadowed by `record.summary` (AI summary) instead of raw Miniflux `summary`.
  - Queries `ai_service.db.get_summary_and_title(id)` (ignoring `model == "extractive-fallback"`, lines 1429–1433).
  - Resolves `rewrite` (lines 1435–1479) by falling back to `record.summary`, then raw `teaser_str` (Miniflux `summary` or raw `content`), never triggering `/hx/entries/{id}/rewrite` on demand.
- **On-Demand Rewrite Route (`crates/server/src/routes/entries.rs:1773-1895` — `get_entry_rewrite_htmx`):**
  - Already accessible in both Auth and Guest modes (no `AuthenticatedUser` extractor; registered in `crates/server/src/app.rs:102`).
  - When `!config.ai_enabled` (lines 1780–1783), returns an empty `#ai-rewrite-box` instead of warning + plain-text fallback.
  - On cache hit or LLM success (lines 1796, 1852), calls `render_analysis_fragment` which renders Impact Analysis + Logical Fallacy inside `#ai-rewrite-box` — placing them *above* `body_images` and duplicating `#ai-analysis-box`.
  - On LLM failure (lines 1855–1893), falls back to raw `teaser` with `escape_html` (leaking literal `<p>`/`<a>` tags and potentially full raw content).
- **Template (`crates/ui-components/templates/partials/entry_detail.html:1-134`):**
  - **Top bar (lines 3–65):** `Back` (`#detail-back-btn`), `Original` link, `Mark as unread` (`hx-post`), `Bookmark` (`hx-post`). (Stays.)
  - **Metadata (lines 74–80 & 88–94):** Renders `entry.feed_title` and `<time datetime="{{ entry.published_at }}" data-date-format="dd/mm/yyyy">` in both hero and standard headers. (Must remove.)
  - **Body:** `#ai-rewrite-box` -> `body_images` -> `analysis_cards.html` / `#ai-analysis-box`.
- **Client Skeleton & Cache (`crates/ui-components/templates/pages/index.html:808-877`, `931-1137`, `1400-1419`):**
  - `openDetailWithSkeleton` renders metadata skeleton bars (`<!-- Feed info & date -->`, lines 1108–1111) and an `"Executive Brief"` card when `brief` is present.
  - `my_news_article_cache_db` (v1) holds 30-day cached HTML containing old metadata, and `htmx:afterSwap` saves `panel.innerHTML` even if a child `hx-get` skeleton hasn't finished yet.

### 2. Guest Mode (`AUTH_ENABLED = false`)
- **Payload (`crates/server/src/routes/entries.rs:179-247`, `428-435`, `crates/server/src/routes/pages.rs:163-170`, `crates/ui-components/src/templates.rs:154-203`):**
  - `entry_view_model_from_json` extracts only 1 image via `extract_first_image_url` (`entries.rs:186-187`) and sets `sanitized_content` (`entries.rs:188-197`).
  - `render_entries_list_html` and `index_page` overwrite `entry.suggested_title` and `entry.summary` without filtering `record.model != "extractive-fallback"`, and never copy `record.rewrite` onto `EntryViewModel`.
  - `GuestEntryPayload` serializes `{ id, title, feed_title, published_at, formatted_date, url, summary, image_url, sanitized_content }`. It lacks `body_images` and `rewrite`, sends `sanitized_content`, and sends `feed_title` / `published_at` / `formatted_date`.
- **Client Renderer (`crates/ui-components/templates/pages/index.html:1555-1803`):**
  - `renderGuestEntryDetail(entry)` renders metadata (`feedTitle` + `<time>`), `entry.summary`, `#guest-detail-content` (`sanitized_content`), and `#ai-analysis-box`, with no multi-photo gallery and no Option C `/rewrite` trigger.
  - `my_news_guest_db` (`all_articles`) stores cached guest entries without `body_images` or `rewrite`.

### 3. Background AI Completion Sync (`executive_brief_htmx`)
- `crates/server/src/routes/entries.rs:832-846` and `1041-1055`: Emits only `<span id="entry-title-{id}" hx-swap-oob="outerHTML">` (does not filter `extractive-fallback` in `render_cached_brief`, and does not update `#mag-title-{id}`, `.magazine-brief`, P3 `title`, or `guestEntriesMap`).

### 4. Existing Tests Directly Affected
- `crates/ui-components/src/templates.rs`:
  - `test_detail_template_omits_author` (line 670): asserts `Audio Weekly` and `2 hours ago` are present.
  - `test_entry_detail_renders_ddmmyyyy_time_element_in_both_headers` (line 3356): asserts `<time ... data-date-format="dd/mm/yyyy">` in `EntryDetailTemplate`.
  - `test_entry_list_guest_json_embeds_sanitized_content_and_escapes_script_tags` (line 3485): asserts `sanitized_content` in `GuestEntryPayload`.
  - `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` (line 3607): asserts `id="guest-detail-content"`, `data-date-format="dd/mm/yyyy"`, and `!render_fn.contains("/rewrite")`.
- `crates/server/src/routes/entries.rs`:
  - `test_get_entry_detail_formats_published_at_as_ddmmyyyy` (line 3077): asserts `<time ... data-date-format="dd/mm/yyyy">`.
  - `test_list_routes_embed_sanitized_guest_data_only_when_auth_disabled` (line 3902): asserts `sanitized_content` in `#entries-guest-data`.
  - `test_get_entry_rewrite_htmx_serves_impact_and_fallacies` (line 3241): expects `/hx/entries/505/rewrite` to return `#ai-rewrite-box`, rewrite, and Impact/Fallacy cards.
- `crates/server/tests/e2e_tests.rs`:
  - `test_e2e_page_load_and_detail_drawer_rewrite_flow` (line 799): asserts Story 2 (which has `summary` only, `rewrite: None`) renders `"Executive brief for story 2."` immediately on `/hx/entries/2`.

---

## Risks & Edge Cases

1. **Duplicate / misordered Impact & Fallacy cards on `/hx/entries/{id}/rewrite`:** If `/rewrite` renders Impact & Fallacy inside `#ai-rewrite-box` while `#ai-analysis-box` loads `/analysis` concurrently, users see duplicate cards and wrong ordering (above `body_images`), plus 2 parallel LLM calls. Decoupling `#ai-rewrite-box` (rewrite only) from an OOB `#ai-analysis-box` + `HX-Trigger-After-Settle: ai-rewrite-done` avoids both problems.
2. **Stale IndexedDB caches (`cached_articles` & `all_articles`):** Bumping `my_news_article_cache_db` to v2 clears old detail HTML; updating `my_news_guest_db` to v3 (clearing only `all_articles` while preserving `read_entries`) removes stale guest payloads without losing read history.
3. **Raw HTML tags in Miniflux `summary`/`content` fallback:** Stripping HTML with `ammonia::Builder::default().tags(HashSet::new())` and taking a concise plain-text summary/excerpt guarantees zero raw HTML tags or full original article leakage.
4. **Offline / network errors in Guest mode on `/hx/entries/{id}/rewrite`:** Handling `target.id === 'ai-rewrite-box'` inside `htmx:responseError` and `htmx:sendError` in `index.html` ensures offline guests still see the inline warning + clean plain-text summary/excerpt fallback.
5. **Existing test asserting `!html.contains("class=\"hidden\"")` in `executive_brief_htmx`:** Emitting OOB sync JSON via `<script id="executive-brief-sync-data" type="application/json" hx-swap-oob="outerHTML">` avoids `class="hidden"` while keeping `#entry-title-{id}` OOB spans intact.

---

## Reference Sources

- `htmx` 1.9 docs (`hx-swap-oob`, `HX-Trigger-After-Settle`, `hx-trigger`): https://htmx.org
- `ammonia` HTML sanitizer crate (`Builder::tags(HashSet::new())`, `clean`): https://docs.rs/ammonia/latest/ammonia/
- `askama` template engine (`0.12` in workspace / latest docs): https://docs.rs/askama/
- Repo files:
  - [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)
  - [crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)
  - [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
  - [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - [crates/ui-components/src/sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs)
  - [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - [crates/server/src/routes/pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs)
