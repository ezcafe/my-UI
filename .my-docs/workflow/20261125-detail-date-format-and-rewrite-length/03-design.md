# Design: Details Date Format (`DD/MM/YYYY`), Longer AI Rewrite (3–5 Sentences), Guest Client Details & Tab View Rules

**Result:** done

## Key Takeaways

1. **Details Date (`DD/MM/YYYY`)**: `EntryDetailTemplate` (and guest client-side detail rendering) render `<time datetime="..." data-date-format="dd/mm/yyyy">` with Chrono RFC 3339 fallback (`%d/%m/%Y`) on the server and browser local-timezone formatting (`formatDetailDates`) on swap, cache hit, and guest render.
2. **Longer AI Rewrite (3–5 Full Sentences) & 250-Word Extraction**: Both `format_batch_prompt_structured` and `format_rewrite_and_analyze_prompt` in `crates/server/src/ai/llm.rs` align on strictly 3 to 5 sentences (`around 90-150 words total, max 180 words`, inverted-pyramid lead + 2–3 bullets), backed by `MAX_EXTRACTED_WORDS = 250` and `strip_html_tags` in `crates/server/src/ai/extractor.rs`.
3. **Guest Mode (`auth_enabled == false`) Client-Side Details (Q5 & Q6 Option A)**: List responses in guest mode sanitize Miniflux `content` via `ammonia` (`sanitize_html`) and embed a WHATWG-safe JSON `<script id="entries-guest-data" type="application/json">` data block on `EntryListTemplate` (omitting `author` and `readingTime`). In [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html), `!AUTH_ENABLED` renders `#detail-panel` directly from the current list map or IndexedDB `all_articles` (or shows an inline unavailable card on cold deep-link misses) with **zero** `GET /hx/entries/{id}` or `/analysis` requests. The server route `get_entry_detail_htmx` stays active (setting `author: None`, `reading_time: 0` when `!config.auth_enabled`).
4. **Tab View Rules & `#ai-toggle-btn` Visibility (Q7 Option A + Hide on Today/All)**: Initial load and **Unread** always show **Magazine view** (`'ai'`) with `#ai-toggle-btn` visible; **Today** and **All** (including guest `renderLocalAllArticles()` wrapped in `#entries-list-view`, cache hits, and pagination) always show **List view** (`'list'`) and hide `#ai-toggle-btn`; **Starred** and **Search** default to **List view** with `#ai-toggle-btn` visible.

---

## Option A (Recommended): `<script type="application/json">` Guest Data Block + Early-Return Client Renderer + Tab View Controller

- **Explanation**:
  - **Details Date & Rewrite Baseline (Round 1 — Implemented)**:
    - `chrono = "0.4"` in `crates/ui-components/Cargo.toml` backs `format_published_date_ddmmyyyy` and `EntryDetailTemplate::formatted_published_at` in `crates/ui-components/src/templates.rs`, rendered inside `<time datetime="{{ entry.published_at }}" data-date-format="dd/mm/yyyy">` in both hero and standard headers of `crates/ui-components/templates/partials/entry_detail.html`.
    - `formatDetailDates(root)` in `crates/ui-components/templates/pages/index.html` formats ISO dates/datetimes to browser local `DD/MM/YYYY` on `htmx:afterSwap` and `openEntryDetail` cache restore.
    - `crates/server/src/ai/llm.rs` aligns both rewrite prompts (`3 to 5 sentences`, `around 90-150 words total`, `max 180 words`), and `crates/server/src/ai/extractor.rs` uses `MAX_EXTRACTED_WORDS = 250` with `strip_html_tags`.
  - **Guest List Data Payload (`ui-components` & `server`)**:
    - Move `serde_json = "1.0"` from `[dev-dependencies]` to `[dependencies]` in `crates/ui-components/Cargo.toml` (already in `Cargo.lock` — zero new external crates).
    - Add `#[serde(default)] pub sanitized_content: Option<String>` to `EntryViewModel` in `crates/ui-components/src/templates.rs`.
    - In `crates/server/src/routes/pages.rs` (`index_page`) and `crates/server/src/routes/entries.rs` (`render_entries_list_html`), when `!auth_enabled`:
      - Select `raw_body = if !content_str.trim().is_empty() { content_str } else { summary_str }` **before** AI summaries overwrite `entry.summary`, and set `sanitized_content: Some(ui_components::sanitizer::sanitize_html(raw_body))`.
      - Explicitly set `reading_time: 0` and `author: None` when `!auth_enabled` (and do the same in `get_entry_detail_htmx`, while keeping the `/hx/entries/{id}` server handler active).
      - When `auth_enabled` is `true`, set `sanitized_content: None` so authenticated responses carry zero extra payload.
    - Add `EntryListTemplate::guest_entries_json(&self) -> Option<String>` in `crates/ui-components/src/templates.rs`:
      - Returns `None` if no entry has `sanitized_content.is_some()` (so authenticated mode and existing `EntryListTemplate` unit tests emit no script tag and require zero struct-field edits).
      - Serializes `{ id, title, feed_title, published_at, formatted_date, url, sanitized_content }` per entry (`title` uses `entry.suggested_title.as_deref().unwrap_or(&entry.title)`, `formatted_date` uses `format_published_date_ddmmyyyy(&entry.published_at)` — strictly omitting `author` and `reading_time`).
      - Escapes `<` → `\u003c`, `>` → `\u003e`, `&` → `\u0026`, `\u2028` → `\u2028`, `\u2029` → `\u2029` per WHATWG script data-block restrictions so untrusted titles or text can never prematurely close `</script>`.
    - Render `<script id="entries-guest-data" type="application/json">{{ guest_json|safe }}</script>` at the end of `crates/ui-components/templates/partials/entry_list.html` (after `#entries-magazine-view`, keeping `#entries-list-view` rows clean).
  - **Guest Detail Client Rendering (`index.html`)**:
    - Expose `const AUTH_ENABLED = {{ auth_enabled }};` in `crates/ui-components/templates/pages/index.html`.
    - Maintain `guestEntriesMap` hydrated from `#entries-guest-data` on `DOMContentLoaded` and `htmx:afterSwap`, and persist `{ id, title, url, feed_title, published_at, formatted_date, sanitized_content, saved_at }` in IndexedDB `my_news_guest_db` store `all_articles` (schemaless objects at existing DB version `2`).
    - Add `safeHref(rawUrl)` allowing only `http://`, `https://`, or `/` relative URLs (fallback `'#'`).
    - In `openEntryDetail(id)`, short-circuit at the top when `!AUTH_ENABLED`: call `await openGuestEntryDetail(numId)` and `return` (never calling `getCachedArticle` stale AI HTML or `htmx.ajax('GET', '/hx/entries/...')`).
    - `openGuestEntryDetail(numId)` resolves the entry from `guestEntriesMap` → `#entries-guest-data` → IndexedDB `all_articles`:
      - **Found**: Renders `#detail-panel` with `Back` button, `Original` link (`safeHref`, `markArticleRead`), `detail-toggle-read-btn` (`toggleLocalReadStatus`), native bookmark button (`triggerNativeBookmark`), `feed_title` + `<time datetime="..." data-date-format="dd/mm/yyyy">` (`DD/MM/YYYY`), `<h1>` title, and `<div id="guest-detail-content" class="prose dark:prose-invert ...">${sanitized_content}</div>`; then calls `formatDetailDates(panel)`, `alignDetailPanelWidth()`, `markLocallyRead(numId)`, and `lucide.createIcons({ root: panel })`. Zero `author`, zero `readingTime`, zero `#ai-analysis-box`.
      - **Not Found (cold deep link `/?entry={id}` or `/entries/{id}`)**: Renders an inline "Article Not Available Offline" notice with a `Back to List` button (`closeDetailPanel()`) and zero network requests.
    - Wrap non-empty `renderLocalAllArticles()` rows inside `<div id="entries-list-view" class="space-y-1.5">...</div>`.
  - **Tab View Locking & `#ai-toggle-btn` Visibility (`index.html`)**:
    - Track `let currentViewMode = 'ai';` and `let isSearchActive = false;`.
    - `shouldHideAiToggle(status, searchActive)` returns `!searchActive && (status === 'today' || status === 'all')`.
    - `getDefaultViewForState(status, searchActive)` returns `'list'` when `searchActive` or `status === 'today' || status === 'all' || status === 'starred'`, and `'ai'` when `status === 'unread'`.
    - `initAiViewMode()` forces `'ai'` (Magazine view) on initial load (`localStorage.setItem('my_news_view_mode', 'ai')`) and keeps `#ai-toggle-btn` visible.
    - `applyAiViewMode(mode, statusOverride, searchOverride)` applies list/magazine visibility, updates `#ai-toggle-btn` icons/title, and toggles `btn.classList.toggle('hidden', shouldHideAiToggle(status, searchActive))`.
    - Tab clicks (`handleFilterClick`, `htmx:beforeRequest` for `.filter-pill`, `applyCachedList`, `renderLocalAllArticles`, and `htmx:afterSwap` triggered by `.filter-pill`) set `isSearchActive = false` and apply `getDefaultViewForState(status, false)`.
    - Search submits (`onSearchSubmit` / `#search-form` swaps) set `isSearchActive = Boolean(searchQuery)` and apply `'list'` with `#ai-toggle-btn` visible.
    - Pagination / `mark-page-read` / `isBriefSwap` preserve `currentViewMode` on Unread/Starred/Search while forcing `'list'` + hidden `#ai-toggle-btn` whenever `!isSearchActive && (status === 'today' || status === 'all')`.

- **Concrete Examples**:
  ```rust
  // crates/ui-components/src/templates.rs
  #[derive(Serialize)]
  struct GuestEntryPayload<'a> {
      id: i64,
      title: &'a str,
      feed_title: &'a str,
      published_at: &'a str,
      formatted_date: String,
      url: &'a str,
      sanitized_content: &'a str,
  }

  fn escape_json_for_html_script(json: &str) -> String {
      json.replace('&', r"\u0026")
          .replace('<', r"\u003c")
          .replace('>', r"\u003e")
          .replace('\u{2028}', r"\u2028")
          .replace('\u{2029}', r"\u2029")
  }
  ```

  ```html
  <!-- crates/ui-components/templates/partials/entry_list.html (end of template, after #entries-magazine-view) -->
  {% if let Some(guest_json) = self.guest_entries_json() %}
  <script id="entries-guest-data" type="application/json">{{ guest_json|safe }}</script>
  {% endif %}
  ```

  ```javascript
  // crates/ui-components/templates/pages/index.html
  const AUTH_ENABLED = {{ auth_enabled }};
  let currentViewMode = 'ai';
  let isSearchActive = false;

  function shouldHideAiToggle(status, searchActive) {
    return !searchActive && (status === 'today' || status === 'all');
  }

  function getDefaultViewForState(status, searchActive) {
    if (searchActive) return 'list';
    if (status === 'today' || status === 'all' || status === 'starred') return 'list';
    return 'ai';
  }

  async function openEntryDetail(id) {
    const numId = Number(id);
    if (!numId) return;
    if (!AUTH_ENABLED) {
      await openGuestEntryDetail(numId);
      return;
    }
    // ... existing authenticated IndexedDB cache & HTMX fetch flow ...
  }
  ```

- **Pros**:
  - Zero `GET /hx/entries/{id}`, `/analysis`, or `/rewrite` calls from the guest browser UI, and zero `author` or `readingTime` exposed in guest details.
  - Safe by construction: server-side `ammonia::Builder` sanitization + WHATWG `<script type="application/json">` escaping + URL scheme check (`safeHref`).
  - Zero breaking changes to `EntryListTemplate` struct fields, existing list-view title-only assertions, or `get_entry_detail_htmx` backend/E2E tests.
  - Zero IndexedDB version bump or DB schema changes (`all_articles` stores richer objects at existing DB version `2`).
  - Cleanly handles initial load, tab clicks, cache restores, pagination, guest `All`, Starred, and Search without view-mode flicker.

- **Cons**:
  - Adds `#[serde(default)] pub sanitized_content: Option<String>` to `EntryViewModel` (requiring `sanitized_content: None` on existing `EntryViewModel` literals in tests/handlers).
  - Increases guest list-page HTML and IndexedDB `all_articles` payload size proportionally to sanitized article HTML content.

---

## Option B: Per-Row `data-*` Attributes on List Rows + New `auth_enabled` Field on `EntryListTemplate`

- **Explanation**:
  - Embed `data-sanitized-content="..."`, `data-feed-title="..."`, and `data-published-at="..."` attributes directly on each `.annuaire-row` in `entry_list.html`, gated by a new `pub auth_enabled: bool` field on `EntryListTemplate`.
  - Disable `GET /hx/entries/{id}` on the server or leave `localStorage` view mode shared across all tabs except explicit tab overrides.

- **Concrete Example**:
  ```html
  <!-- Inside #entries-list-view row -->
  <div class="annuaire-row ..." data-entry-id="{{ entry.id }}" data-sanitized-content="{{ content }}">
  ```

- **Pros**:
  - Avoids a separate `<script type="application/json">` block and keeps `serde_json` in `[dev-dependencies]` of `ui-components`.

- **Cons**:
  - Bloats every `.annuaire-row` element inside `#entries-list-view` with escaped HTML attributes, risking collisions with `list_view_html` substring assertions in `templates.rs`.
  - Requires updating 17 `EntryListTemplate` struct initializers across `templates.rs` and `entries.rs`.

---

## Recommendation

**Option A** is recommended. Embedding one WHATWG-escaped `<script id="entries-guest-data" type="application/json">` block outside `#entries-list-view` keeps list rows lightweight and preserves `EntryListTemplate`'s struct signature, guarantees zero guest `GET /hx/entries/{id}` or `/analysis` calls with `ammonia`-sanitized Miniflux `content`, and cleanly enforces initial/Unread Magazine vs. Today/All List (with `#ai-toggle-btn` hidden) vs. Starred/Search List defaults.

---

## Reference Sources

- WHATWG HTML Living Standard — Restrictions for contents of script elements (`</script>`, `<!--`, `<script` escaping) & JSON data blocks: https://html.spec.whatwg.org/multipage/scripting.html#restrictions-for-contents-of-script-elements
- MDN Web Docs — `<script>` element (`type="application/json"` data blocks): https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
- MDN Web Docs — `<time>` element & `datetime` attribute: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time
- Askama `0.12.1` Template Methods & Filters: https://docs.rs/askama/0.12.1/askama/
- Ammonia `4.0` HTML Sanitizer: https://docs.rs/ammonia/latest/ammonia/
- `serde_json::to_string`: https://docs.rs/serde_json/latest/serde_json/fn.to_string.html
- Chrono `0.4.45` `DateTime::parse_from_rfc3339` & `strftime`: https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339
- htmx Events (`htmx:beforeRequest`, `htmx:afterSwap`): https://htmx.org/reference/#events
- Codebase Touchpoints:
  - [crates/ui-components/Cargo.toml](file:///Users/ptquang86/ws/my-ui/crates/ui-components/Cargo.toml)
  - [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - [crates/ui-components/src/sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs)
  - [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)
  - [crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)
  - [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
  - [crates/server/src/routes/pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs)
  - [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs)
  - [crates/server/src/ai/extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs)

---

## Proposed Has (parent **or** labeled solo applies to card)

- `Has UI`: `yes` (unchanged — updates `entry_detail.html`, `entry_list.html`, `index.html`, and `templates.rs`)
- `Has API`: `no` (unchanged — no new endpoint or API schema change; `GET /hx/entries/{id}` stays intact)
- `Has DB`: `no` (unchanged — PostgreSQL schema untouched; IndexedDB `my_news_guest_db` keeps existing version `2`)
- `Has auth/PII/schema`: `no` (unchanged)

## Has API / DB (proposed)

`API=no`, `DB=no` (no flip).
