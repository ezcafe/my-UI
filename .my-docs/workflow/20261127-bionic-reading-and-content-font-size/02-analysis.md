# Analysis — Bionic Reading for Brief & Rewrite + 1rem Content Typography

**Result:** done

## Context

### Confirmed User Scope (`01-idea.md` + `00-run.md`)
- **Bionic Reading targets (always-on, no toggle button):**
  1. **Executive Brief** (`#page-executive-brief` in Magazine view, `#list-executive-brief` in List view, plus `brief_card.html` / `ai_summary_card.html` if rendered).
  2. **Magazine card article summaries** (`.magazine-brief` on Priority 1 hero & Priority 2 grid cards). *(Priority 3 cards only expose `title="..."` browser tooltips, which only accept plain text.)*
  3. **Article rewrite content** (`#ai-rewrite-box .prose`, including ready AI rewrites and fallback summaries).
- **1rem (`16px` / Tailwind `text-base`) font size:** Confirmed for rewrite content and Executive Brief (Option B).
- **Fonts & Theme (`base.html` & `ANNUAIRE_STYLE_GUIDE.md`):**
  - `base.html` loads Google Fonts `Work Sans` weights `400`, `500`, and `700` (italic `400`). Using `font-weight: 700` (`<b>`) against `400` body text uses the real bold font face with zero faux-bolding.
  - Tailwind CSS is loaded via CDN (`https://cdn.tailwindcss.com`) **without** the `@tailwindcss/typography` plugin. Therefore `.prose` in `#ai-rewrite-box` has no typography plugin font-size override; it currently gets `0.875rem` directly from `text-sm` (and `line-height: 1.625` from `leading-relaxed`). Note that JS (`index.html`) queries `.prose` via `rewriteBox.querySelector('.prose')`, so the `.prose` class must be preserved.

### Complete Surface & Render Path Inventory

1. **Executive Brief (`#page-executive-brief` & `#list-executive-brief`):**
   - [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L38) (`#list-executive-brief p`): currently `text-xs`.
   - [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L110) (`#page-executive-brief p`): currently `text-xs`.
   - [executive_brief_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/executive_brief_card.html#L8) (`ExecutiveBriefCardTemplate` returned by `GET /hx/executive-brief`): currently `text-xs`.
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1619-L1633) (`htmx:afterSwap` brief sync): copies `pageBrief.innerText || pageBrief.textContent` into `#list-executive-brief p` (currently `text-xs`) using `escapeHtml(...)`. Error card branch (`isErrorCard` at lines 1589–1618) must remain `text-xs` and **not** receive Bionic Reading.
   - [brief_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/brief_card.html#L8) (`BriefCardTemplate`, tested in `templates.rs`): currently `text-sm`.
   - [ai_summary_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_summary_card.html#L2-L4) (`AiSummaryCardTemplate`, used by `status=ai` -> `GET /hx/entries/:id/ai-summary`): currently `text-xs`.

2. **Magazine Card Briefs (`.magazine-brief`):**
   - [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L144) (Priority 1 Hero card): `<h3 class="magazine-title text-base font-bold ...">` (`1rem`), `<p class="magazine-brief text-xs ... line-clamp-6">`.
   - [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L176) (Priority 2 3-col Grid cards): `<h4 class="magazine-title text-xs font-semibold ...">` (`0.75rem`), `<p class="magazine-brief text-xs ... line-clamp-4">`.
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1498-L1503) (`syncExecutiveBriefUpdates()`): dynamically overwrites `magBriefEl.textContent = summaryText` after `/hx/executive-brief` returns OOB `#executive-brief-sync-data`. Any server-only Bionic markup on `.magazine-brief` would be wiped out here unless JS re-applies Bionic Reading.
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1183-L1189) (`openEntryDetail`) & [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L2393-L2402) (`cacheLoadedArticlesFromDom`): read `magBriefEl.textContent.trim()`.

3. **Article Rewrite (`#ai-rewrite-box .prose`) — 9 render sites (all currently `text-sm`):**
   - [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L123) (SSR ready rewrite)
   - [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L139) (SSR `!ai_enabled` fallback summary)
   - [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2006) (`render_rewrite_box_html` for `/hx/entries/:id/rewrite`)
   - [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2237) (`get_entry_rewrite_htmx` `!config.ai_enabled` fallback)
   - [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2277) (`get_entry_rewrite_htmx` LLM error fallback)
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1559) (`syncExecutiveBriefUpdates()` live update to `#ai-rewrite-box`)
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1829) (`handleRewriteError()` client fallback)
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L2140) (`renderGuestEntryDetail()` ready rewrite)
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L2157) (`renderGuestEntryDetail()` `!AI_ENABLED` fallback)
   - [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1734-L1735) reads `proseEl.textContent.trim()` in Guest mode after `/hx/entries/:id/rewrite` swap to persist plain text into IndexedDB.

4. **Detail Analysis Cards (`analysis_cards_inner.html`):**
   - [analysis_cards_inner.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards_inner.html#L10) (`Impact Analysis`) and [line 25](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards_inner.html#L25) (`Logical Fallacy & Objectivity Check`) currently use `text-sm`.

5. **IndexedDB & Filter Cache Lifecycle (`index.html`):**
   - `filterCache` (`FILTER_CACHE_TTL_MS = 60000`) caches `#entries-list.innerHTML` and restores via `applyCachedList()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L592-L620)).
   - Auth detail cache (`my_news_article_cache_db`, version `3`, 30-day TTL, [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L952)) stores `panel.innerHTML` at line 1757 and validates `cached.html.includes('id="detail-prev-btn"')` at line 1131. Old v3 entries cached in users' browsers still hold `text-sm` and un-bionicized HTML unless upgraded (`version 4` or `isCacheValid` check + re-applying bionic).

## Risks

1. **Unicode & Vietnamese Diacritic Splitting:** Naive `.split('')` / `.slice()` on decomposed (`NFD`) Vietnamese strings splits combining tone/vowel marks from base letters. Mitigation: normalize with `.normalize('NFC')`, segment words with `Intl.Segmenter('vi', { granularity: 'word' })`, and slice leading fixation points by grapheme clusters via `Intl.Segmenter('vi', { granularity: 'grapheme' })`.
2. **Pure Numbers & Punctuation Bolding:** `Intl.Segmenter` marks numeric tokens (`2026`, `40`, `1,000`) as `isWordLike: true`. Bolding half of a number (`<b>20</b>26`) hurts numeric clarity. Mitigation: only apply fixation bolding when a word-like segment contains at least one Unicode letter (`/\p{L}/u.test(segment)`).
3. **`textContent` / `innerText` / `whitespace-pre-line` Corruption:** Duplicating text via `.sr-only` inside `.prose`, `.magazine-brief`, or `#page-executive-brief p` would double the string read by `magBriefEl.textContent`, `proseEl.textContent`, `pageBrief.innerText`, and clipboard copy. Using inline `<b>...</b>` text nodes without extra hidden text keeps `.textContent`, `.innerText`, and `\n\n` breaks (`whitespace-pre-line`) 100% intact.
4. **Line-clamp Interaction (`line-clamp-4`, `line-clamp-6`):** Inline `<b>` elements inside `-webkit-line-clamp` containers work across modern engines provided `<b>` stays `display: inline` with no vertical padding/margin and flex parents maintain `min-width: 0`.
5. **Double-Bolding / Idempotency:** Running Bionic transformation multiple times (e.g., on `applyCachedList()`, `openEntryDetail()` cache hit, or `htmx:afterSwap`) would corrupt text if already transformed. Mitigation: walk only raw text nodes not inside `b.bionic-fixation` and stamp `data-bionic-applied="true"` on processed containers (clearing/re-applying only when raw `.textContent` is freshly overwritten).
6. **Existing Test Contracts:** Server/template tests assert exact plain-text substrings in SSR/HTMX HTML (`"Cached Executive Synthesis"`, `"Inverted pyramid rewrite text."`, etc.) and check `indexedDB.open(ARTICLE_DB_NAME, 3)` (`templates.rs` lines 3853, 5443) and `renderLocalAllArticles()`/`syncDomReadStates()` slice boundaries. Client-side JS Bionic application preserves all server/template plain-text substring contracts.

## Open Questions

See Round 1 frontier questions in `02b-grill.md` regarding:
1. `.magazine-brief` font size (`text-xs` vs `1rem`) given Priority 2 card headline hierarchy (`text-xs`).
2. Detail analysis cards (`Impact Analysis` & `Logical Fallacy & Objectivity Check`) font size (`text-sm` vs `1rem`).
