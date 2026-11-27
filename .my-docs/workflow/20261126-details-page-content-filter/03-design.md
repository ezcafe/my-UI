# Design — Details Page Content Filter

Result: done

## Key Takeaways

- **Strict content filter across Auth, Guest, and Skeleton:** Details view renders only: top action bar (`Back`, `Original`, read/unread, bookmark), hero photo with overlaid AI/article title, AI rewrite (or Option C on-demand generation + warning/plain-text fallback), body photos gallery, and Impact Analysis + Logical Fallacy & Objectivity Check. No `feed_title`, no `<time>` / date metadata, and zero original article body HTML (`#guest-detail-content` / `sanitized_content`).
- **Auth-vs-Guest gate, serde compatibility & URL parity:** `EntryListTemplate` adds `pub auth_enabled: bool` so `guest_entries_json()` gates `#entries-guest-data` on `!self.auth_enabled` (without relying on `sanitized_content`), `EntryViewModel` adds `#[serde(default)]` to `rewrite` and `body_images`, and `extract_article_images` restricts extracted photo URLs to `http://` / `https://` to match `safeHref` parity.
- **Scoped Option C rewrite + OOB analysis without retry loops:** `#ai-rewrite-box` triggers `GET /hx/entries/{id}/rewrite` on load when missing; `/rewrite` emits rewrite-only `#ai-rewrite-box`, an out-of-band `<div id="ai-analysis-box" hx-swap-oob="outerHTML">` via `AnalysisCardsTemplate { oob: true }`, and `HX-Trigger-After-Settle: ai-rewrite-done-{id}`. Client error handlers embed `data-fallback-summary`, replace `#ai-rewrite-box` via `outerHTML` (preventing infinite `hx-trigger="load"` loops), and guard Guest `rewrite` persistence against HTTP 200 `.is-error` cards.
- **HTMX-safe `/hx/executive-brief` OOB sync without DOM clobbering:** `/hx/executive-brief` emits `<div id="executive-brief-sync-data" hidden hx-swap-oob="outerHTML" data-entries="..."></div>` across all OOB-emitting paths (cache hit, in-flight wait, and fresh synthesis including Case C partial responses) for any non-`extractive-fallback` `suggested_title`, `rewrite`, or `summary`. `syncExecutiveBriefUpdates()` updates Magazine cards, List View, open `#detail-panel`, `guestEntriesMap`, IndexedDB `all_articles`, Auth `deleteCachedArticle`, and `#entries-guest-data` JSON **before** `filterCache.set(...)`.

---

## Option A (Recommended): Unified HTMX OOB Sync & Decoupled Rewrite/Analysis Fragments

### Explanation
Keep server-driven rendering via Askama partials and HTMX 1.9 (`hx-get`, `hx-swap-oob="outerHTML"`, `HX-Trigger-After-Settle`), while updating `GuestEntryPayload` (`#entries-guest-data`) and client-side `renderGuestEntryDetail` to match the exact structure, ordering, URL safety, and Option C rewrite-on-demand flow of Authenticated mode.

1. **Shared plain-text fallback helper (`crates/ui-components/src/sanitizer.rs`):**
   - Add `extract_plain_summary(summary_html: &str, content_html: &str) -> String`:
     - **Block-tag spacing before `ammonia`:** Normalize block/line-break HTML boundaries (`<br>`, `<br/>`, `</p>`, `<p...>`, `</div>`, `</li>`, `</blockquote>`, `</tr>`, `</td>`, `</th>`, `</h1..h6>`, `</section>`, `</article>`) to a single space `" "` via a `LazyLock<Regex>` before calling `ammonia` so `<p>First.</p><p>Second.</p>` becomes `"First. Second."` rather than `"First.Second."`. Leave `<script>` and `<style>` tags intact for `ammonia`'s default `clean_content_tags` to wipe completely.
     - **HTML stripping & entity decoding:** Run `ammonia::Builder::default().tags(HashSet::new()).clean(...)`, then decode common entities (`&nbsp;` -> `" "`, `&quot;` / `&#34;` -> `"\""`, `&apos;` / `&#39;` / `&#x27;` -> `"'"`, `&lt;` -> `"<"`, `&gt;` -> `">"`, and `&amp;` **last** -> `"&"` to avoid double-unescaping). Collapse whitespace.
     - **UTF-8 char-boundary truncation:** Never slice by raw byte index. If `summary_html` cleans to non-empty text, cap at 400 Unicode scalar values (`.chars()` / `.char_indices()`, trimming back to a whitespace word boundary and appending `"..."` if truncated). Otherwise extract a brief opening excerpt from `content_html` (up to 2–3 sentences capped at 300 chars on a `.char_indices()` boundary — never the full article body).

2. **Templates, Partials & Guest Payload (`crates/ui-components`):**
   - **`EntryViewModel` & `EntryListTemplate` (`templates.rs`):**
     - Add `#[serde(default)] pub rewrite: Option<String>` and `#[serde(default)] pub body_images: Vec<String>` to `EntryViewModel`.
     - Add `pub auth_enabled: bool` to `EntryListTemplate`.
     - Update `guest_entries_json(&self)` to return `None` when `self.auth_enabled || self.entries.is_empty()`. When `!self.auth_enabled`, serialize `GuestEntryPayload { id, title, url, summary, rewrite, image_url, body_images }` (`rewrite: e.rewrite.as_deref()`, `body_images: &e.body_images`), removing `sanitized_content`, `feed_title`, `published_at`, and `formatted_date`. Escape via `escape_json_for_html_script`.
   - **`EntryDetailTemplate` & `entry_detail.html`:**
     - Add `pub needs_rewrite: bool`, `pub ai_enabled: bool`, and `pub fallback_summary: String` to `EntryDetailTemplate`.
     - Remove `formatted_published_at()` from `EntryDetailTemplate`, and add `pub fn safe_entry_url(&self) -> &str` allowing `http://`, `https://`, or root-relative `/` (excluding `//`), returning `"#"` otherwise.
     - In `entry_detail.html`: keep the top bar (`Back`, `Original` using `href="{{ self.safe_entry_url() }}"`, read/unread, bookmark). Remove `feed_title` and `<time ... data-date-format="dd/mm/yyyy">` from both Hero and Standard headers.
     - Render `#ai-rewrite-box`:
       - If `!needs_rewrite && !rewrite.trim().is_empty()`: render `{{ rewrite }}`.
       - Else if `needs_rewrite` (`ai_enabled == true`): render `<div id="ai-rewrite-box" class="space-y-6" hx-get="/hx/entries/{{ entry.id }}/rewrite" hx-trigger="load" hx-swap="outerHTML" data-fallback-summary="{{ fallback_summary }}">` containing an inline notice (`"AI rewrite is not ready yet — generating..."`) and pulse skeleton lines.
       - Else (`!ai_enabled` or fallback-only): render inline warning (`"AI rewrite is unavailable."`) + `{% if !fallback_summary.trim().is_empty() %}{{ fallback_summary }}{% else %}No summary available.{% endif %}`.
     - Render `body_images` gallery directly after `#ai-rewrite-box`.
     - Render analysis section last:
       - If `has_analysis`: `<div id="ai-analysis-box" class="space-y-6 mt-6">{% include "analysis_cards_inner.html" %}</div>`.
       - Else if `ai_enabled`: `<div id="ai-analysis-box" class="animate-pulse space-y-4 my-4" hx-get="/hx/entries/{{ entry.id }}/analysis" hx-trigger="{% if needs_rewrite %}ai-rewrite-done-{{ entry.id }} from:body{% else %}load{% endif %}" hx-swap="outerHTML">...</div>`.
       - Else (`!ai_enabled`): omit `#ai-analysis-box`.
   - **`AnalysisCardsTemplate` & `analysis_cards.html` / `analysis_cards_inner.html`:**
     - Extract card markup into `partials/analysis_cards_inner.html` (shared by `entry_detail.html` and `analysis_cards.html` so `EntryDetailTemplate` needs no `oob` field).
     - Add `pub oob: bool` to `AnalysisCardsTemplate<'a>`.
     - Update `analysis_cards.html` to `<div id="ai-analysis-box"{% if oob %} hx-swap-oob="outerHTML"{% endif %} class="space-y-6 mt-6">{% include "analysis_cards_inner.html" %}</div>`.
   - **`entry_list.html`:**
     - Add `<div id="executive-brief-sync-data" hidden></div>` (boolean `hidden` attribute; never `class="hidden"`, preserving `!html.contains("class=\"hidden\"")` assertions).

3. **Server Routes (`crates/server/src/routes/entries.rs` & `pages.rs`):**
   - **`extract_article_images`:**
     - Restrict accepted `src` values to case-insensitive `http://` or `https://` URLs (`lower.starts_with("http://") || lower.starts_with("https://")`) alongside existing tracking-pixel filters so Auth and Guest modes extract identical, XSS-safe image sets.
   - **`entry_view_model_from_json`:**
     - Call `extract_article_images(content_str)` and fall back to `extract_article_images(summary_str)` when `hero_image.is_none()` to populate `image_url` and `body_images`.
     - Set `summary` via `ui_components::sanitizer::extract_plain_summary(summary_str, content_str)`.
     - Set `rewrite: None` and `sanitized_content: None`.
   - **`render_entries_list_html` & `index_page`:**
     - Ignore `record.model == "extractive-fallback"`.
     - Populate `entry.suggested_title`, `entry.rewrite = record.rewrite.clone().filter(|r| !r.trim().is_empty())`, and `entry.summary` (using `extract_plain_summary(&record.summary, "")` when non-empty, else preserving the clean `entry.summary`).
     - Pass `auth_enabled` (and `config.auth_enabled` in `index_page`, `mark_feed_read_htmx`, `mark_all_read_htmx`) to `EntryListTemplate`, and initialize `rewrite: None, body_images: Vec::new()` in `render_ai_entries_list` and `mark_entry_read_htmx`.
   - **`get_entry_detail_htmx`:**
     - Preserve `raw_summary` separately before resolving AI fields so `extract_article_images` falls back to `raw_summary` accurately when `content` has no images.
     - Only set `needs_rewrite = false` with non-empty `rewrite` when `config.ai_enabled && record.model != "extractive-fallback"` and `record.rewrite` is non-empty (never substitute `record.summary` or raw Miniflux `teaser_str` as `rewrite`).
     - When `rewrite` is missing: set `needs_rewrite = config.ai_enabled`, and compute `fallback_summary` as `extract_plain_summary(&record.summary, "")` (if non-`extractive-fallback` and non-empty), else `extract_plain_summary(&raw_summary, content)`, else `"No summary available."`.
     - Pass `needs_rewrite`, `ai_enabled: config.ai_enabled`, `fallback_summary` to `EntryDetailTemplate`.
   - **`get_entry_rewrite_htmx` (`/hx/entries/{id}/rewrite`):**
     - Attach response header `HX-Trigger-After-Settle: ai-rewrite-done-{id}` on **all** `/rewrite` responses (`!config.ai_enabled`, cache hit, LLM success, and LLM failure).
     - On cache hit or LLM success: render rewrite-only HTML inside `<div id="ai-rewrite-box" class="space-y-6">...</div>` (removing `render_analysis_fragment`). If `impact` or `fallacies` is non-empty after trimming, append `AnalysisCardsTemplate { impact, fallacies, oob: true }` as an out-of-band sibling so `#ai-analysis-box` swaps below `body_images` and avoids a redundant `/analysis` trigger.
     - When `!config.ai_enabled` or LLM generation returns `Err(_)`: query Miniflux entry `summary`/`content` (in both branches), check non-`extractive-fallback` `record.summary` via `extract_plain_summary(&record.summary, "")`, fall back to `extract_plain_summary(raw_summary, content)`, and default to `"No summary available."` if empty.
       - On `!config.ai_enabled`: render inline warning (`"AI rewrite is unavailable."`) + plain-text fallback inside `#ai-rewrite-box`.
       - On LLM `Err(_)`: render `AiErrorCardTemplate` (`title: "AI Rewrite Unavailable"`, Retry button) + plain-text fallback inside `#ai-rewrite-box` + `render_llm_error_toast()`.
   - **`get_entry_analysis_htmx`:**
     - Reuse `AnalysisCardsTemplate { impact, fallacies, oob: false }` instead of manual HTML string formatting (`render_analysis_cards_fragment`).
   - **`executive_brief_htmx` (`/hx/executive-brief`):**
     - Filter `rec.model != "extractive-fallback"` in `render_cached_brief` and fresh synthesis.
     - In fresh synthesis `ArticleSummaryInput`, set `summary` by preferring `!item.brief.trim().is_empty()` with fallback to `item.rewrite.clone()`.
     - Continue emitting `#entry-title-{id}` OOB spans for List View and append `<div id="executive-brief-sync-data" hidden hx-swap-oob="outerHTML" data-entries="..."></div>` across **all** OOB-emitting paths (fast-path cache hit, in-flight wait completion, and fresh synthesis including Case C partial responses where `batch_res.top` is empty and `items` is non-empty). Include every entry on the page with `model != "extractive-fallback"` that has any non-empty `suggested_title`, `rewrite`, or `summary`.

4. **Client JS, Error Handling, Sync & Cache Versioning (`crates/ui-components/templates/pages/index.html`):**
   - **Global config & date-formatter removal:** Expose `const AI_ENABLED = {{ ai_enabled }};`. Remove `formatDetailDates(root)` and all its call sites (`openEntryDetail`, `htmx:afterSwap`, `renderGuestEntryDetail`), as no `<time data-date-format="dd/mm/yyyy">` metadata exists anywhere in the app.
   - **Brief Sync (`syncExecutiveBriefUpdates()`):**
     - Run inside `htmx:afterSwap` when `isBriefSwap` **before** `filterCache.set(status, ...)`.
     - Parse JSON from `document.getElementById('executive-brief-sync-data')?.getAttribute('data-entries')`.
     - For each synced item: update `#entry-title-{id}` (text + `title`), `#mag-title-{id}`, `#mag-entry-{id} .magazine-brief`, P3 `#mag-entry-{id}` & `#mag-title-{id}` `title` attributes, `guestEntriesMap` (merging `title`, `rewrite`, `summary`), IndexedDB `all_articles` (merging while keeping `image_url`/`body_images`/`url`), and Auth cache invalidation (`deleteCachedArticle(id)`).
     - Update the `<script id="entries-guest-data">` text inside `#entries-list` with the merged array (WHATWG-escaped via `escapeJsonForHtmlScript` so `<`, `>`, `&` remain safe inside `filterCache` snapshots) **before** `filterCache.set(status, ...)` saves `entriesListEl.innerHTML`.
     - Only call `hydrateGuestEntriesFromDom()` on `isEntriesSwap` (not `isBriefSwap`), and in `hydrateGuestEntriesFromDom()` preserve any existing non-empty `rewrite` in `guestEntriesMap` if the DOM item lacks one.
     - If `#detail-panel` is open for a synced `id` (`[data-entry-id="${id}"]`): update `<h1>`, and if `#ai-rewrite-box` is still waiting/skeleton (`[hx-get][hx-trigger]` or `.animate-pulse`) or error/fallback (`.is-error` or `[data-fallback-summary]`) and synced `rewrite` is non-empty, replace `#ai-rewrite-box` via `outerHTML` and dispatch `new CustomEvent('ai-rewrite-done-' + id)` on `document.body` if `#ai-analysis-box[hx-trigger]` is still waiting.
   - **Auth Skeleton (`openDetailWithSkeleton`):**
     - Remove metadata skeleton bars (`<!-- Feed info & date -->`) and `"Executive Brief"` card heading; render clean Hero/Title + prose skeleton lines.
   - **Guest Detail (`renderGuestEntryDetail`):**
     - Remove `feed_title`, `<time>` / `formatted_date`, and `#guest-detail-content`.
     - Render in exact order:
       1. Top bar (`Back`, `Original` via `safeHref`, `#detail-toggle-read-btn`, `triggerNativeBookmark`)
       2. Hero photo with overlaid `entry.title` (or standard `<h1>` if no valid `entry.image_url`)
       3. `#ai-rewrite-box`: if `entry.rewrite` is non-empty, render it; else if `AI_ENABLED`, render Option C skeleton with `hx-get="/hx/entries/${numId}/rewrite" hx-trigger="load" hx-swap="outerHTML" data-fallback-summary="${escapeHtml(summaryText)}"`; else render inline warning (`"AI rewrite is unavailable."`) + `escapeHtml(summaryText || 'No summary available.')`.
       4. Body photos gallery: map `entry.body_images` through `safeHref` and keep only entries where `safeImg && safeImg !== '#'` (with `loading="lazy"`, `referrerpolicy="no-referrer"`, `onerror` hide).
       5. `#ai-analysis-box`: `hx-get="/hx/entries/${numId}/analysis"` with `hx-trigger="ai-rewrite-done-${numId} from:body"` (when awaiting `/rewrite`) or `hx-trigger="load"` (when `entry.rewrite` already exists), omitted if `!AI_ENABLED`.
   - **Network Error Replacement & IndexedDB Guards:**
     - In `htmx:responseError` / `htmx:sendError`: when `target && target.id === 'ai-rewrite-box'`, read `retryUrl` (`target.getAttribute('hx-get')`), resolve `numId`, and resolve fallback text (`target.getAttribute('data-fallback-summary')` -> `guestEntriesMap.get(numId)?.summary` -> `'No summary available.'`). Replace via `target.outerHTML = ...` (no `hx-get` / `hx-trigger` / `hx-swap` on `#ai-rewrite-box`; Retry button carries them), run `htmx.process(newBox)` and `lucide.createIcons({ root: newBox })` on the live `document.getElementById('ai-rewrite-box')`, and dispatch `new CustomEvent('ai-rewrite-done-' + numId)` on `document.body`.
     - In `htmx:afterSwap` detail branch:
       - When `!AUTH_ENABLED` and swapping `#ai-rewrite-box`: resolve the live `#ai-rewrite-box`, check `!rewriteBox.querySelector('.is-error, .stroke-destructive')`, read ONLY `.prose` text (`rewriteBox.querySelector('.prose')?.textContent.trim()`), and if non-empty persist into `guestEntriesMap` + IndexedDB `all_articles`.
       - Guard `saveCachedArticle(entryId, panel.innerHTML)` with `if (AUTH_ENABLED && entryId && !panel.querySelector('[hx-get][hx-trigger], .stroke-destructive, .is-error'))`.
     - Bump `ARTICLE_DB_NAME` (`my_news_article_cache_db`) to version `2` (clearing `cached_articles` on upgrade) and `DB_NAME` (`my_news_guest_db`) to version `3` (clearing `all_articles` on upgrade while preserving `read_entries`).
     - In `cacheLoadedArticlesFromDom()`, persist `{ id, title, url, summary, rewrite, image_url, body_images, schema_version: 2, saved_at }` and preserve any already-fetched/synced non-empty `rewrite` from `guestEntriesMap.get(numId)`.

### Concrete Example (OOB Contracts for `/hx/entries/{id}/rewrite` & `/hx/executive-brief`)
```html
<!-- GET /hx/entries/505/rewrite (header: HX-Trigger-After-Settle: ai-rewrite-done-505) -->
<div id="ai-rewrite-box" class="space-y-6">
  <div class="prose dark:prose-invert max-w-none text-foreground/90 text-sm leading-relaxed space-y-4 font-sans whitespace-pre-line">
    Lead sentence with key outcome. Second sentence with primary impact.
  </div>
</div>
<div id="ai-analysis-box" hx-swap-oob="outerHTML" class="space-y-6 mt-6">
  <!-- Impact Analysis + Logical Fallacy & Objectivity Check cards -->
</div>

<!-- GET /hx/executive-brief OOB payload (boolean hidden; never class="hidden") -->
<div id="executive-brief-sync-data" hidden hx-swap-oob="outerHTML" data-entries="[{&quot;id&quot;:505,&quot;original_title&quot;:&quot;Raw&quot;,&quot;suggested_title&quot;:&quot;Punchy&quot;,&quot;rewrite&quot;:&quot;Lead...&quot;,&quot;summary&quot;:&quot;Brief...&quot;}]"></div>
```

### Pros
- **Zero extra JSON endpoints & single LLM pass:** Reuses `/hx/entries/{id}/rewrite` and `/hx/executive-brief` with standard HTMX 1.9 OOB swaps and entry-scoped `HX-Trigger-After-Settle`.
- **Resilient across modes & failures:** Both Auth and Guest modes carry `data-fallback-summary`, handle HTTP/network failures without retry loops, and avoid caching `.is-error` cards or cross-mode IndexedDB entries.
- **No stale state or clobbering:** Rewriting `#entries-guest-data` before `filterCache.set`, preserving `rewrite` in `cacheLoadedArticlesFromDom`, and bumping IndexedDB versions eliminate stale UI/cache drift.

### Cons
- Requires updating existing test assertions in `templates.rs`, `entries.rs`, and `e2e_tests.rs` that asserted `<time>`, `feed_title`, `formatDetailDates`, `#guest-detail-content` (`sanitized_content`), or inline analysis inside `#ai-rewrite-box`, plus updating struct literals for `EntryViewModel`, `EntryDetailTemplate`, `EntryListTemplate`, and `AnalysisCardsTemplate`.

---

## Option B: Separate JSON Detail API + Client-Side Fetching for All Detail Views

### Explanation
Expose a new JSON endpoint `GET /api/entries/{id}/detail` returning `{ title, hero_image, body_images, rewrite, fallback_summary, impact, fallacies }` and rewrite both Auth and Guest detail panels as pure client-side `fetch()` + DOM builders (removing `entry_detail.html` HTMX swapping and `GuestEntryPayload` embedding).

### Concrete Example
```javascript
const res = await fetch(`/api/entries/${numId}/detail`);
const data = await res.json();
renderDetailPanel(data);
```

### Pros
- Single JS rendering function for both Auth and Guest modes.
- Avoids HTMX OOB / `HX-Trigger-After-Settle` coordination.

### Cons
- Introduces a brand-new JSON API endpoint and drops HTMX hypermedia architecture used throughout the app.
- Breaks offline Guest mode (since guest articles are currently embedded in the page list HTML and `all_articles` IndexedDB so clicking works without a round-trip to Miniflux).
- Larger blast radius across existing tests and routing (`e2e_tests.rs`, `entries.rs`, `templates.rs`).

---

## Recommendation

Choose **Option A**. It preserves the HTMX + Askama hypermedia architecture and Guest offline readiness, satisfies every requirement from Q1 (Option C), Q2, Q3, and Ideation (top bar kept, metadata removed, zero original article body), and fixes every API contract, client lifecycle, sanitization, and test-coverage gap found in review.

---

## Proposed Has (parent applies to card)

- **Has UI:** `yes`
- **Has API:** `yes` *(Flip `no -> yes`: modifies response contracts for `GET /hx/entries/{id}`, `GET /hx/entries/{id}/rewrite`, `GET /hx/executive-brief`, and embedded `#entries-guest-data` JSON `GuestEntryPayload` across `crates/server` and `crates/ui-components`.)*
- **Has DB:** `no`
- **Has auth/PII/schema:** `no`

---

## Reference Sources

- `htmx` 1.9 `hx-swap-oob`, `processScripts` & `HX-Trigger-After-Settle`: https://htmx.org/attributes/hx-swap-oob/ · https://htmx.org/headers/hx-trigger/
- `ammonia` 4.x `Builder::tags`: https://docs.rs/ammonia/latest/ammonia/struct.Builder.html#method.tags
- `askama` 0.12 docs: https://docs.rs/askama/0.12.1/askama/
- Repo touchpoints:
  - [crates/ui-components/src/sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs)
  - [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)
  - [crates/ui-components/templates/partials/analysis_cards.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards.html)
  - [crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)
  - [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
  - [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - [crates/server/src/routes/pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs)
