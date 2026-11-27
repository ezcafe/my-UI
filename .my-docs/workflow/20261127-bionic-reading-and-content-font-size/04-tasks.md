# Tasks (TDD-ready)

Each task: behavior, failing test name first, files allowed, done evidence.

**Result:** done

## Planned tests

<!-- ≥1 named case → planned tests present (Prefer tdd-review). Empty / N/A → not present. -->

| Case | Failing test name | Covers |
|------|-------------------|--------|
| 1 | `test_1rem_typography_and_magazine_title_hierarchy_across_templates` | `text-base` on Executive Brief (`entry_list.html`, `executive_brief_card.html`, `brief_card.html`, `ai_summary_card.html`), P1/P2 `.magazine-brief`, `#ai-rewrite-box .prose` (`entry_detail.html`), and `Impact Analysis` / `Logical Fallacy & Objectivity Check` (`analysis_cards_inner.html`); P1 `h3` `text-xl font-bold`, P2 `h4` `text-lg font-bold line-clamp-3`, and `ai_grouped_list.html` headline `text-lg font-bold`; `ai_summary_card.html` icon `w-4 h-4 text-foreground shrink-0 mt-1`; `.magazine-center-col` on `#mag-col-center` (`entry_list.html`) and Magazine skeleton (`index.html`); `.magazine-p2-grid` default `repeat(auto-fit, minmax(min(100%, 220px), 1fr))` + `@container mag-center (min-width: 43rem)` `repeat(3, minmax(0, 1fr))`; error cards stay `text-xs` |
| 2 | `test_bionic_reading_engine_and_hooks_in_base_and_index` | `b.bionic-fixation` (`font-weight: 700`), `Intl.Segmenter('vi', ...)` + `.normalize('NFC')` + `TreeWalker` + `/^\p{L}/u` number guard + `!/\p{L}/u.test(node.nodeValue)` skip + deterministic `force === true` unwrap (`replaceWith(document.createTextNode(...))` + `el.normalize()`) + `computeBionicFixationLength` in `base.html`; `.is-error` exclusion; `DOMContentLoaded` + unconditional `liveRoot` `htmx:afterSwap` hook; `index.html` hooks (`seedInitialFilterCache` pre-cache `applyBionicReading`, `applyCachedList`, `openEntryDetail`, `syncExecutiveBriefUpdates` with `force = true`, `htmx:afterSwap` `applyBionicReading(entriesListEl \|\| evt.detail.target)` before `filterCache.set`, `handleRewriteError`, `renderGuestEntryDetail`), `text-base` in JS-rendered `.prose` and `#list-executive-brief p`, and `ARTICLE_DB_NAME` v4 + `isCacheValid` check |
| 3 | `test_entry_rewrite_endpoints_use_text_base_and_bionic_marker` | `render_rewrite_box_html` and `get_entry_rewrite_htmx` (`!ai_enabled` fallback and LLM error fallback) in `crates/server/src/routes/entries.rs` emit `text-base` and `bionic-text` on `#ai-rewrite-box .prose` while keeping plain escaped text intact |

## Implementation Steps

1. **Task 1 — Add `base.html` Bionic Reading CSS, pure-DOM engine, and `.magazine-center-col` P2 grid container query**
   - **Failing test first:** `test_bionic_reading_engine_and_hooks_in_base_and_index` (asserting `b.bionic-fixation`, `Intl.Segmenter`, `.normalize('NFC')`, `createTreeWalker`, `/^\p{L}/u`, deterministic `force` unwrap with `el.normalize()`, `.magazine-center-col` + `@container mag-center (min-width: 43rem)`, `minmax(min(100%, 220px), 1fr)`, and `applyBionicReading(liveRoot)` in `base.html`).
   - **Behavior:** Add `b.bionic-fixation { font-weight: 700; display: inline; }`, define `.magazine-center-col { container-type: inline-size; container-name: mag-center; }`, change `.magazine-p2-grid` default to `repeat(auto-fit, minmax(min(100%, 220px), 1fr))` and move `repeat(3, minmax(0, 1fr))` into `@container mag-center (min-width: 43rem)` (replacing `@container (max-width: 32rem)`), implement `computeBionicFixationLength`, `createBionicFragment`, `applyBionicToElement` (deterministic `force` unwrap via `b.replaceWith(document.createTextNode(b.textContent || ''))` + `el.normalize()`, plus `!/\p{L}/u.test(node.nodeValue)` skip), and `applyBionicReading`, and call `applyBionicReading` in `DOMContentLoaded` and `htmx:afterSwap` (computing `liveRoot` unconditionally so `applyBionicReading(liveRoot)` runs after `lucide.createIcons({ root: liveRoot })` even if `lucide` is absent).
   - **Files allowed:** `crates/ui-components/templates/base.html`, `crates/ui-components/src/templates.rs`.
   - **Done evidence:** `cargo test -p ui-components` passes with `test_bionic_reading_engine_and_hooks_in_base_and_index`, `test_magazine_grid_css_rules_present`, and `test_toast_deduplication_and_detail_is_error_cache_guard`.

2. **Task 2 — Upgrade Askama partial templates to `text-base` (`1rem`), `bionic-text`, adjusted card titles, and `.magazine-center-col`**
   - **Failing test first:** `test_1rem_typography_and_magazine_title_hierarchy_across_templates` (asserting `text-base` + `bionic-text` on brief/rewrite partials, `text-base` without `bionic-text` on `analysis_cards_inner.html`, P1 `text-xl` / P2 `text-lg` `line-clamp-3` / `ai_grouped_list.html` `text-lg font-bold` headlines, `ai_summary_card.html` `w-4 h-4 ... mt-1` icon, and `.magazine-center-col` on `#mag-col-center`).
   - **Behavior:** Update `entry_list.html` (`L38`, `L110`, `L118` add `magazine-center-col`, `L137`, `L144`, `L169`, `L176`), `executive_brief_card.html` (`L8`), `brief_card.html` (`L8`), `ai_grouped_list.html` (`L12`: `text-sm font-medium` → `text-lg font-bold`), `ai_summary_card.html` (`L2` `text-base`, `L3` `w-4 h-4 text-foreground shrink-0 mt-1`, `L4` `bionic-text`), `entry_detail.html` (`L123`, `L139`), and `analysis_cards_inner.html` (`L10`, `L25`). Keep `"line-clamp-4 leading-relaxed font-sans flex-1"` contiguous on P2 `.magazine-brief` and zero `md:` viewport breakpoints.
   - **Files allowed:** `crates/ui-components/templates/partials/entry_list.html`, `crates/ui-components/templates/partials/executive_brief_card.html`, `crates/ui-components/templates/partials/brief_card.html`, `crates/ui-components/templates/partials/ai_grouped_list.html`, `crates/ui-components/templates/partials/ai_summary_card.html`, `crates/ui-components/templates/partials/entry_detail.html`, `crates/ui-components/templates/partials/analysis_cards_inner.html`, `crates/ui-components/src/templates.rs`.
   - **Done evidence:** `cargo test -p ui-components` passes including `test_magazine_view_brief_line_clamping`, `test_ai_grouped_list_template_renders_feed_headers_and_canonical_route`, and `test_1rem_typography_and_magazine_title_hierarchy_across_templates`.

3. **Task 3 — Wire `text-base`, `bionic-text`, Bionic hooks, `.magazine-center-col` skeleton, and `ARTICLE_DB_NAME` v4 in `index.html`**
   - **Failing test first:** Extend `test_bionic_reading_engine_and_hooks_in_base_and_index` (and update legacy `ARTICLE_DB_NAME` v3 checks at `templates.rs#L3853,L5443` to v4) so `index.html` fails until all dynamic paths and skeleton container class are wired.
   - **Behavior:** Add `magazine-center-col` to the Magazine skeleton center column (`L201`), call `applyBionicReading(entriesList)` at the top of `seedInitialFilterCache()` (`L573`), update `applyCachedList()`, `openArticleCacheDb()` (v4), `isCacheValid`, `openEntryDetail()` cache hit, `syncExecutiveBriefUpdates()`, `htmx:afterSwap` (brief sync + `.is-error` class guard + `applyBionicReading(entriesListEl || evt.detail.target)` outside `if (isBriefSwap)` right before `filterCache.set` at `L1664` + detail pre-cache `applyBionicReading(panel)`), `handleRewriteError()`, and `renderGuestEntryDetail()`.
   - **Files allowed:** `crates/ui-components/templates/pages/index.html`, `crates/ui-components/src/templates.rs`.
   - **Done evidence:** `cargo test -p ui-components` passes all unit tests cleanly.

4. **Task 4 — Update server-side rewrite HTML builders in `entries.rs` to `text-base` + `bionic-text`**
   - **Failing test first:** `test_entry_rewrite_endpoints_use_text_base_and_bionic_marker` in `crates/server/src/routes/entries.rs`.
   - **Behavior:** Update `render_rewrite_box_html` (`L2006`), `get_entry_rewrite_htmx` `!config.ai_enabled` fallback (`L2237`), and `get_entry_rewrite_htmx` LLM error fallback (`L2277`) from `text-sm` to `text-base` + `bionic-text`.
   - **Files allowed:** `crates/server/src/routes/entries.rs`.
   - **Done evidence:** `cargo test --workspace` passes all `ui-components`, `miniflux`, `server`, and `e2e_tests` tests without skipped or relaxed assertions.
