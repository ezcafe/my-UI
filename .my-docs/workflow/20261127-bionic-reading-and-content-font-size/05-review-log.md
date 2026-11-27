# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

- **Stage:** `code-review-phase` (Mode `simple`, Review profile `lite`, `Has UI=yes | API=no | DB=no | auth/PII/schema=no`, `Lens plan: none`)
- **Build Verify Baseline:** `06-test-report.md` (`Smoke source: Build-verify`) — `cargo test --workspace` passed (`286` passed, `0` failed, `0` ignored) and `cargo clippy --workspace --all-targets -- -D warnings` exited `0`.

## Adversarial

**Result:** clean

### Breakage, Security & Edge-Case Hunt

- **Unicode / Vietnamese & Non-Word Guard (`base.html` L754–L852):**
  - `createBionicFragment` normalizes input with `.normalize('NFC')` before word segmentation (`Intl.Segmenter('vi', { granularity: 'word' })`) and grapheme slicing (`Intl.Segmenter('vi', { granularity: 'grapheme' })`), preventing decomposed diacritics from splitting off their base vowel.
  - Only segments where `seg.isWordLike && /^\p{L}/u.test(seg.segment)` receive `<b class="bionic-fixation">` fixation bolding; pure numbers (`2026`), digit-prefixed units (`1rem`), whitespace, and punctuation remain plain text nodes. Verified via Node `Intl.Segmenter('vi')` execution on decomposed Vietnamese (`tiê\u0301ng Viê\u0323t`), English contractions (`don't`), hyphenated tokens (`state-of-the-art`, `AI-powered`), and numbers/units (`2026`, `1rem`).
  - `applyBionicToElement` snapshots `TreeWalker` (`NodeFilter.SHOW_TEXT`) nodes into an array before DOM replacement, skips nodes without `\p{L}`, and skips nodes inside `b.bionic-fixation, script, style, code, pre, button, a`.
  - When `force === true` (`syncExecutiveBriefUpdates()`), prior `b.bionic-fixation` elements are unwrapped via `b.replaceWith(document.createTextNode(b.textContent || ''))` and `el.normalize()` before re-walking, preventing nested `<b>` elements or broken words.
- **Plain-Text Extraction, Error Isolation & XSS Boundaries:**
  - Zero Askama `|safe` filters were added and `createBionicFragment` uses `document.createElement('b')` + `b.textContent` + `document.createTextNode` + `DocumentFragment` (zero `innerHTML` HTML injection). Dynamic HTML builders in `entries.rs` (`L2006`, `L2237`, `L2277`) and `index.html` (`L1577`, `L1625`, `L1652`, `L1859`, `L2173`, `L2190`) preserve `escape_html` / `escapeHtml`.
  - Because `createBionicFragment` inserts no `.sr-only` duplicates, `.textContent` and `.innerText` remain exact plain text across `openEntryDetail()` (`L1200–L1204`), `cacheLoadedArticlesFromDom()` (`L2438`), `#page-executive-brief` → `#list-executive-brief` copy (`L1652`), and Guest `#ai-rewrite-box` IndexedDB persistence (`L1762`).
  - `.is-error` cards (`ai_error_card.html`, `#list-executive-brief.is-error` at `index.html` `L1617`, and `handleRewriteError` / `entries.rs` error boxes) retain `text-xs` without `bionic-text`, and `applyBionicToElement` rejects any node with `el.closest('.is-error')`. On recovery (`index.html` `L1644`), `.is-error` is removed before applying `bionic-text`.
- **HTMX `outerHTML` & Cache Lifecycle:**
  - `base.html` `htmx:afterSwap` (`L860–L874`) resolves `liveRoot` unconditionally via `swapTarget.isConnected ? swapTarget : (swapTarget.id ? document.getElementById(swapTarget.id) : null) || (evt.target && evt.target.isConnected ? evt.target : document.body)` so `outerHTML` swaps (`#page-executive-brief`, `#ai-rewrite-box`, and id-less `ai_summary_card.html`) are bionicized even when `swapTarget` is detached or `window.lucide` is absent.
  - `seedInitialFilterCache()` (`L574`), `htmx:afterSwap` pre-`filterCache.set` (`L1689`), `applyCachedList()` (`L618`), and `htmx:afterSwap` pre-`saveCachedArticle` (`L1782`) ensure cached list/detail fragments stay consistent and idempotent (`dataset.bionicApplied === 'true'`).
  - `ARTICLE_DB_NAME` bumps from v3 to v4 (`L958`, `L963`) and `isCacheValid` (`L1137–L1143`) rejects stale cached detail HTML carrying legacy `text-sm` classes on `.prose` or analysis cards.
- **Structural Simplification Check:**
  - Using one pure-DOM client processor in `base.html` eliminates duplicating Unicode word/grapheme segmentation across Rust (`unicode-segmentation`) and client JS (`Guest` + live sync + error fallbacks) and avoids introducing `|safe` template surfaces.
- **Low-Severity Non-Blocking Notes:**
  - In `base.html` `L772`, the legacy non-`Intl.Segmenter` regex fallback (`[\p{L}\p{N}_]+`) omits `\p{M}`; `.normalize('NFC')` composes standard Vietnamese characters first and all modern browsers (Chrome 87+, Safari 14.1+, Firefox 125+) use `Intl.Segmenter`.

## Quality

**Result:** clean

### Five-Axis Review & UI Craft

1. **Correctness:**
   - All `brief` and `rewrite` surfaces use `text-base` (`1rem`) + `bionic-text` across SSR, HTMX partials, server fallbacks (`entries.rs` `L2006`, `L2237`, `L2277`), and client JS (`index.html` `L1577`, `L1652`, `L1859`, `L2173`, `L2190`).
   - `analysis_cards_inner.html` (`L10`, `L25`) upgrades `Impact Analysis` and `Logical Fallacy & Objectivity Check` to `text-base` without `bionic-text`, matching confirmed scope.
   - `ai_summary_card.html` (`L2–L4`) upgrades container to `text-base`, aligns `sparkles` icon (`w-4 h-4 text-foreground shrink-0 mt-1`), and marks `<p class="font-normal bionic-text">`.
2. **Readability & Maintainability:**
   - Single Bionic module in `base.html` (`L754–L852`) with small focused functions (`computeBionicFixationLength`, `segmentBionicWords`, `createBionicFragment`, `applyBionicToElement`, `applyBionicReading`) and shallow nesting.
   - Headline-above-body visual hierarchy is clear against `1rem` bolded summaries: P1 `h3` (`text-xl font-bold`, `entry_list.html` `L137`), P2 `h4` (`text-lg font-bold line-clamp-3`, `entry_list.html` `L169`), and `ai_grouped_list.html` (`text-lg font-bold`, `L12`).
   - CSS container queries (`base.html` `L494–L509`, `entry_list.html` `L118`, `index.html` `L201`) use `.magazine-center-col` (`container-name: mag-center`) and `repeat(auto-fit, minmax(min(100%, 220px), 1fr))` up to `@container mag-center (min-width: 43rem)` (`repeat(3, minmax(0, 1fr))`), avoiding hardcoded viewport breakpoints and keeping `Work Sans` `400`/`700` and Annuaire contrast tokens intact.
3. **Architecture:**
   - Presentation templates emit plain escaped text; `base.html` owns Bionic DOM enhancement; `index.html` invokes the shared helper only at dynamic render/cache boundaries.
4. **Security:**
   - Zero `|safe` additions; pure DOM node construction in JS; `escape_html` / `escapeHtml` preserved on all HTML string builders.
5. **Performance:**
   - Module-scoped `Intl.Segmenter` singletons avoid repeated locale collation initialization; `!/\p{L}/u` short-circuit skips whitespace/punctuation text nodes; `dataset.bionicApplied === 'true'` provides O(1) idempotency on repeat swaps and cache hits.

## Merged

**Result:** pending  
*(N/A — `Lens plan: none`)*

## Reference Sources

- MDN `Intl.Segmenter`: <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Segmenter>
- MDN `Document.createTreeWalker()` & `Node.normalize()`: <https://developer.mozilla.org/en-US/docs/Web/API/Document/createTreeWalker> · <https://developer.mozilla.org/en-US/docs/Web/API/Node/normalize>
- MDN CSS Container Queries (`container-type: inline-size`, `container-name`, `@container`): <https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_containment/Container_queries>
- HTMX `htmx:afterSwap` & `outerHTML` swap lifecycle: <https://htmx.org/events/#htmx:afterSwap> · <https://htmx.org/attributes/hx-swap/>
- Annuaire Design System: [ANNUAIRE_STYLE_GUIDE.md](file:///Users/ptquang86/ws/my-ui/ANNUAIRE_STYLE_GUIDE.md)
