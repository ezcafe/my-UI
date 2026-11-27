# Design — Bionic Reading for Brief & Rewrite + 1rem Content Typography

**Result:** done

**Design present** only when Result **`done`** (file alone / `pending` never pays) — [../derive-next.md](../derive-next.md).

## Option A: Shared Client-Side Pure-DOM Bionic Post-Processor (`base.html`) + Template/CSS `text-base` Updates (Recommended)

- **Explanation:**
  1. **Always-On Pure-DOM Bionic Engine (`base.html`):** Keep all Askama templates and Rust HTMX HTML builders emitting plain, HTML-escaped text, and run one shared client-side DOM post-processor (`applyBionicReading(root)` / `applyBionicToElement(el, force)`) defined in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) (loaded for both Auth and Guest modes).
  2. **Explicit Target Marking & Error Exclusion:** Add marker class `bionic-text` to all non-error Bionic targets (`#list-executive-brief p` non-error, `#page-executive-brief p`, `executive_brief_card.html p`, `brief_card.html p`, `ai_summary_card.html p`, P1/P2 `.magazine-brief`, and `#ai-rewrite-box .prose` across SSR, HTMX, sync, error-fallback, and Guest render sites). In `applyBionicReading(root)`, match `.bionic-text, .magazine-brief, #ai-rewrite-box .prose` while explicitly rejecting any element inside `.is-error`, `button`, `a`, `script`, `style`, `code`, `pre`, or an already-processed `b.bionic-fixation` ancestor.
  3. **Unicode & Vietnamese-Safe Fixation (`Intl.Segmenter` + `TreeWalker`):**
     - Normalize text node strings with `.normalize('NFC')` so decomposed (`NFD`) Vietnamese diacritics stay bound to their base vowels before slicing.
     - Segment words with cached `new Intl.Segmenter('vi', { granularity: 'word' })` and grapheme clusters with `new Intl.Segmenter('vi', { granularity: 'grapheme' })` (with a safe Unicode regex + `Array.from` fallback if `Intl.Segmenter` is unavailable).
     - Transform a segment **only** when `segment.isWordLike && /^\p{L}/u.test(segment.segment)` so pure numbers (`2026`, `40`, `1,000`), units starting with digits (`1rem`, `100ms`), punctuation, and whitespace stay unbolded.
     - Compute fixation grapheme length via `computeBionicFixationLength(len)`: `1` for `len <= 3` (`<b>I</b>`, `<b>ở</b>`, `<b>t</b>he`, `<b>n</b>hà`), and `Math.ceil(len * 0.4)` for `len >= 4` (`4->2`, `5->2`, `6->3`, `7->3`, `8->4`).
     - Replace each matched text node via `document.createTreeWalker(el, NodeFilter.SHOW_TEXT, ...)` with a `DocumentFragment` of `<b class="bionic-fixation">...</b>` (`b.textContent = lead`) and plain text nodes (`document.createTextNode(rest)`). Never use `innerHTML` string parsing, and never insert `.sr-only` duplicates so `.textContent`, `.innerText`, clipboard copy, and `whitespace-pre-line` remain 100% unchanged.
     - CSS in `base.html`: `b.bionic-fixation { font-weight: 700; display: inline; }` (using the real `Work Sans` `700` font face loaded at [base.html#L11](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L11) against `400` body text).
     - Stamp `el.dataset.bionicApplied = 'true'` for idempotency across cache hits and repeated `htmx:afterSwap` runs; pass `force = true` when `syncExecutiveBriefUpdates()` directly reassigns `magBriefEl.textContent = summaryText`, with deterministic unwrapping of any prior `b.bionic-fixation` nodes + `el.normalize()`.
  4. **`1rem` (`text-base`) Typography, Card Title Hierarchy & Center-Column Container Query:**
     - Upgrade `#ai-rewrite-box .prose` across all 9 render sites (`entry_detail.html`, `entries.rs`, `index.html`), Executive Brief (`entry_list.html`, `executive_brief_card.html`, `brief_card.html`, `ai_summary_card.html`, `index.html` non-error sync), `.magazine-brief` (P1 & P2 in `entry_list.html`), and `Impact Analysis` / `Logical Fallacy & Objectivity Check` (`analysis_cards_inner.html`) from `text-xs` / `text-sm` to `text-base` (`1rem` / `16px`, paired with `leading-relaxed` = unitless `1.625` per WCAG SC 1.4.12).
     - Scale card headlines above `1rem` bolded summaries across both Magazine and Grouped AI Summary views: Priority 1 `<h3 class="magazine-title ...">` (`entry_list.html`) from `text-base` → `text-xl font-bold` (`1.25rem`), Priority 2 `<h4 class="magazine-title ...">` (`entry_list.html`) from `text-xs font-semibold line-clamp-2` → `text-lg font-bold line-clamp-3` (`1.125rem`), and `ai_grouped_list.html` headline link from `text-sm font-medium` → `text-lg font-bold` (`1.125rem`). Resize `ai_summary_card.html`'s leading `sparkles` icon from `w-3.5 h-3.5 ... mt-0.5` to `w-4 h-4 ... mt-1` to match `text-base` line height (`26px`).
     - Fix `.magazine-p2-grid` responsiveness by making `#mag-col-center` (`entry_list.html`) and the matching Magazine skeleton center column (`index.html`) an inline container (`.magazine-center-col` with `container-type: inline-size; container-name: mag-center;`). Set `.magazine-p2-grid`'s default to `repeat(auto-fit, minmax(min(100%, 220px), 1fr))` and move `repeat(3, minmax(0, 1fr))` into `@container mag-center (min-width: 43rem)` (`3 * 220px + 2 * 0.875rem = 688px`). That avoids crushing Priority 2 cards into `117px–190px` columns when `.magazine-container` is `48rem`–`78rem`, scopes Priority 1's `@container (min-width: 36rem)` side-by-side rule to `.magazine-center-col`, keeps [templates.rs#L1635](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L1635) passing, and uses zero hardcoded viewport breakpoints.
  5. **Auth Detail Cache Refresh (`ARTICLE_DB_NAME` v4):**
     - Bump `indexedDB.open(ARTICLE_DB_NAME, 4)` and `e.oldVersion < 4` clear in [index.html#L952-L959](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L952-L959), and extend `isCacheValid` at [index.html#L1131](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1131) to reject cached HTML still carrying legacy `text-sm` classes on `.prose` or analysis cards.
- **Pros:**
  - Single source of truth for Bionic segmentation in `base.html`—works identically across SSR, HTMX partials, `syncExecutiveBriefUpdates()`, error fallbacks, `filterCache`, and Guest IndexedDB views.
  - Zero Askama `|safe` expansion and zero `innerHTML` HTML injection for Bionic markup, keeping XSS boundaries airtight.
  - Preserves every existing server/template plain-text substring assertion and `.textContent` / `.innerText` extractor.
- **Cons:**
  - Runs post-render on the client (`DOMContentLoaded` and `htmx:afterSwap`), so raw SSR curl output shows plain text until JS executes.
  - Requires bumping `ARTICLE_DB_NAME` to v4 and updating the two existing `indexedDB.open(ARTICLE_DB_NAME, 3)` assertions in `templates.rs`.

## Option B: Server-Side Rust Bionic HTML Generator (`ui-components/src/bionic.rs`) + Duplicate JS Engine

- **Explanation:** Add `unicode-segmentation` to `crates/ui-components/Cargo.toml`, build a Rust HTML-escaping Bionic formatter emitting `<b class="bionic-fixation">...</b>` via Askama `|safe` filters in templates and `entries.rs`, **and** duplicate the same `Intl.Segmenter` Bionic logic in client JS (`base.html` / `index.html`) because Guest mode (`#entries-guest-data`), `syncExecutiveBriefUpdates()`, `#list-executive-brief` copy, and `handleRewriteError()` still construct DOM content client-side from plain strings.
- **Pros:** Initial SSR HTML responses from `/`, `/hx/entries`, `/hx/executive-brief`, and `/hx/entries/:id/rewrite` contain `<b>` markup before JS runs.
- **Cons:**
  - Duplicates the entire word/grapheme segmentation and fixation logic across Rust and JS, risking subtle mismatch between SSR and live-synced/Guest views.
  - Adds a new Rust dependency and switches multiple interpolated AI/RSS template fields to `|safe`, widening XSS review surface.
  - Breaks existing server/template tests asserting contiguous plain-text substrings (`"Pre-existing Executive Brief Summary"`, `"Inverted pyramid rewrite text."`) in rendered HTML.

## Recommendation

**Option A** — it eliminates Rust/JS algorithm duplication, preserves strict Askama auto-escaping (`|safe` never added), protects `.textContent` / `.innerText` / clipboard / `whitespace-pre-line`, keeps existing server plain-text substring contracts intact, and cleanly delivers both Bionic Reading and `1rem` (`text-base`) typography across every Auth, Guest, HTMX, and cached view.

---

## Detailed Component Changes

### 1. [crates/ui-components/templates/base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)

- **CSS additions & Magazine center-column container query (`<style>`):**
  - Add Bionic fixation style near `.tabular` (`L190`):
    ```css
    b.bionic-fixation {
      font-weight: 700;
      display: inline;
    }
    ```
  - Replace `.magazine-p2-grid` + `@container (max-width: 32rem)` (`L488-L498`) with `.magazine-center-col` inline-size containment (`mag-center`) and responsive `auto-fit` default (`>= 220px` per card, upgrading to 3 columns only once `#mag-col-center` reaches `43rem`):
    ```css
    .magazine-center-col {
      container-type: inline-size;
      container-name: mag-center;
    }

    .magazine-p2-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
      gap: 0.875rem;
    }

    @container mag-center (min-width: 43rem) {
      .magazine-p2-grid {
        grid-template-columns: repeat(3, minmax(0, 1fr));
      }
    }
    ```
- **Shared Bionic Reading engine (placed before `document.addEventListener("DOMContentLoaded", ...)` at `L743`):**
  - Define:
    - `bionicWordSegmenter` & `bionicGraphemeSegmenter` (lazy or module-level `Intl.Segmenter('vi', { granularity: 'word' })` / `'grapheme'` when supported).
    - `computeBionicFixationLength(graphemeCount)`: returns `1` for `<= 3`, `Math.ceil(graphemeCount * 0.4)` for `>= 4`.
    - `createBionicFragment(rawText)`: normalizes `.normalize('NFC')`, iterates word segments, transforms only when `seg.isWordLike && /^\p{L}/u.test(seg.segment)`, slices grapheme clusters into `<b class="bionic-fixation">` + trailing text node.
    - `applyBionicToElement(el, force = false)`:
      - Guards `!el || el.closest('.is-error')` and `if (!force && el.dataset.bionicApplied === 'true') return;`.
      - When `force === true`: unwrap any existing `b.bionic-fixation` descendants via `el.querySelectorAll('b.bionic-fixation').forEach((b) => b.replaceWith(document.createTextNode(b.textContent || '')))` and call `el.normalize()` so split text nodes merge back deterministically before walking.
      - Walk text nodes via `document.createTreeWalker(el, NodeFilter.SHOW_TEXT)` and snapshot matching nodes into an array *before* replacing them: skip nodes whose `parentElement` is inside `b.bionic-fixation, script, style, code, pre, button, a`, and skip nodes where `!/\p{L}/u.test(node.nodeValue)` (avoids churning pure whitespace or punctuation nodes).
      - Replace each matched text node via `node.parentNode.replaceChild(createBionicFragment(node.nodeValue), node)` (live DOM mutation never shifts `TreeWalker` traversal).
      - Stamps `el.dataset.bionicApplied = 'true'`.
    - `applyBionicReading(root)`: queries `(root || document).querySelectorAll('.bionic-text, .magazine-brief, #ai-rewrite-box .prose')` (plus checks if `root` itself matches `.matches(...)` for `outerHTML` swaps directly targeting `#ai-rewrite-box` or `.bionic-text`) and invokes `applyBionicToElement(el, false)`.
  - Wire lifecycle hooks in `base.html`:
    - Inside `DOMContentLoaded` (`L743-L747`): call `applyBionicReading(document);`.
    - Inside `document.body.addEventListener("htmx:afterSwap", ...)` (`L748-L757`): compute `liveRoot` unconditionally (not gated on `window.lucide`), run `lucide.createIcons({ root: liveRoot })` when `window.lucide` is available (preserving existing slice assertions in `templates.rs#L4966-L4980`), then call `if (typeof applyBionicReading === 'function') applyBionicReading(liveRoot);`. For `outerHTML` swaps whose replacement root has no `id` (e.g. `ai_summary_card.html` replacing `#ai-summary-slot-{{ item.id }}`), `liveRoot` falls back to `document.body` and stamps newly inserted `.bionic-text` nodes cleanly while skipping already-stamped elements.

### 2. Askama Partial Templates (`crates/ui-components/templates/partials/`)

1. **[entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html):**
   - `L38` (`#list-executive-brief p`): `text-xs` → `text-base` + `bionic-text` (`class="text-base text-foreground/90 leading-relaxed font-sans bionic-text"`).
   - `L110` (`#page-executive-brief p`): `text-xs` → `text-base` + `bionic-text` (`class="text-base text-foreground/90 leading-relaxed font-sans bionic-text"`).
   - `L118` (`#mag-col-center`): add `magazine-center-col` (`class="magazine-center-col w-full flex flex-col gap-6"`).
   - `L137` (Priority 1 Hero `<h3 class="magazine-title ...">`): `text-base` → `text-xl` (`font-bold` retained; no `md:` breakpoint).
   - `L144` (Priority 1 Hero `.magazine-brief`): `text-xs` → `text-base` + `bionic-text` (`class="magazine-brief text-base text-muted-foreground/90 mt-3 leading-relaxed font-sans line-clamp-6 bionic-text"`).
   - `L169` (Priority 2 Grid `<h4 class="magazine-title ...">`): `text-xs font-semibold ... line-clamp-2` → `text-lg font-bold ... line-clamp-3`.
   - `L176` (Priority 2 Grid `.magazine-brief`): `text-xs` → `text-base` + `bionic-text`, keeping `"line-clamp-4 leading-relaxed font-sans flex-1"` contiguous (`class="magazine-brief text-base text-muted-foreground/90 mt-1.5 line-clamp-4 leading-relaxed font-sans flex-1 bionic-text"`) to preserve `test_magazine_view_brief_line_clamping` (`templates.rs#L3462`).
2. **[executive_brief_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/executive_brief_card.html):**
   - `L8`: `text-xs` → `text-base` + `bionic-text`.
3. **[brief_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/brief_card.html):**
   - `L8`: `text-sm` → `text-base` + `bionic-text`.
4. **[ai_grouped_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html):**
   - `L12` headline link (`<a href="{{ item.url }}" ...>`): `text-sm font-medium` → `text-lg font-bold` (`1.125rem` plain text, keeping `font-sans text-foreground hover:text-foreground/80 transition-colors line-clamp-2` without `bionic-text`) so headlines stay visually larger than the `1rem` summary cards below them.
5. **[ai_summary_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_summary_card.html):**
   - `L2`: `text-xs` → `text-base`.
   - `L3`: `<i data-lucide="sparkles" class="w-4 h-4 text-foreground shrink-0 mt-1"></i>` (upgraded from `w-3.5 h-3.5 ... mt-0.5` to align with `text-base` `26px` line height).
   - `L4`: add `bionic-text` (`<p class="font-normal bionic-text">{{ summary }}</p>`). *(Leave fallback string `"Summary unavailable"` in `entries.rs#L1392,1406` unchanged without `bionic-text`.)*
6. **[entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html):**
   - `L123` (SSR ready rewrite `.prose`): `text-sm` → `text-base` + `bionic-text`.
   - `L139` (SSR `!ai_enabled` fallback `.prose`): `text-sm` → `text-base` + `bionic-text`.
7. **[analysis_cards_inner.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards_inner.html):**
   - `L10` (`Impact Analysis` `<p>`): `text-sm` → `text-base` (no `bionic-text`).
   - `L25` (`Logical Fallacy & Objectivity Check` `<p>`): `text-sm` → `text-base` (no `bionic-text`).

### 3. [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)

- `L2006` (`render_rewrite_box_html`): `text-sm` → `text-base` + `bionic-text` on `.prose`.
- `L2237` (`get_entry_rewrite_htmx` `!config.ai_enabled` fallback): `text-sm` → `text-base` + `bionic-text` on `.prose`.
- `L2277` (`get_entry_rewrite_htmx` LLM error fallback): `text-sm` → `text-base` + `bionic-text` on `.prose`.

### 4. [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)

- **Magazine skeleton center column (`L201`):** add `magazine-center-col` (`class="magazine-center-col w-full flex flex-col gap-6"`) so loading skeletons match the real `#mag-col-center` `.magazine-p2-grid` column count.
- **`seedInitialFilterCache()` (`L572-584`):** right after `const entriesList = document.getElementById('entries-list');`, call `if (entriesList && typeof applyBionicReading === 'function') applyBionicReading(entriesList);` so initial `filterCache.set` stores already-bionicized HTML (even though `index.html` registers its `DOMContentLoaded` listener before `base.html`'s footer `<script>`).
- **`applyCachedList(cachedHtml, statusOverride)` (`L592-620`):** call `if (typeof applyBionicReading === 'function') applyBionicReading(entriesList);` right after `refreshRelativeTimes(entriesList);` (`L614`).
- **Auth detail IndexedDB cache v4 (`L952-L959` & `L1131`):**
  - Bump `indexedDB.open(ARTICLE_DB_NAME, 4)` and `e.oldVersion < 4` store clear.
  - Update `isCacheValid` (`L1131`) to:
    ```js
    const isCacheValid = Boolean(
      cached &&
      cached.html &&
      cached.html.includes('id="detail-prev-btn"') &&
      !cached.html.includes('prose dark:prose-invert max-w-none text-foreground/90 text-sm') &&
      !cached.html.includes('text-sm text-foreground/90 leading-relaxed font-sans')
    );
    ```
  - On cache hit (`L1151`), call `if (typeof applyBionicReading === 'function') applyBionicReading(panel);`.
- **`syncExecutiveBriefUpdates()` (`L1498-L1566`):**
  - Right after `magBriefEl.textContent = summaryText;` (`L1502`), call `if (typeof applyBionicToElement === 'function') applyBionicToElement(magBriefEl, true);`.
  - At `L1559`, change `text-sm` → `text-base` and add `bionic-text` on `#ai-rewrite-box .prose`, then right after setting `rewriteBox.outerHTML`, fetch `const updatedRewriteBox = document.getElementById('ai-rewrite-box');` and call `if (updatedRewriteBox && typeof applyBionicReading === 'function') applyBionicReading(updatedRewriteBox);`.
- **`htmx:afterSwap` (`L1585-L1760`):**
  - At `L1594` (error `#list-executive-brief`): call `listBrief.classList.add('is-error');` (keep `<p class="text-xs ...">` plain, no `bionic-text`).
  - At `L1620-L1628` (non-error `#list-executive-brief`): call `listBrief.classList.remove('is-error');` and use `class="text-base text-foreground/90 leading-relaxed font-sans bionic-text"`.
  - Place `if (typeof applyBionicReading === 'function') applyBionicReading(entriesListEl || evt.detail.target);` **outside** `if (isBriefSwap)` right before `if (status && !searchQuery && entriesListEl)` (`L1664`), so `filterCache.set` (`L1665-L1669`) always captures bionicized DOM on both `#entries-list` swaps and `#page-executive-brief` swaps.
  - In the `#detail-panel` / `#ai-rewrite-box` / `#ai-analysis-box` branch (`L1755`), call `if (typeof applyBionicReading === 'function') applyBionicReading(panel);` right before `saveCachedArticle(entryId, panel.innerHTML)` (leaving Guest `proseEl.textContent.trim()` at `L1735` intact as plain text).
- **`handleRewriteError(evt, msg)` (`L1829-L1841`):**
  - Change `text-sm` → `text-base` and add `bionic-text` on fallback `.prose` (`L1829`).
  - Inside `if (newBox)` (`L1834`), call `if (typeof applyBionicReading === 'function') applyBionicReading(newBox);`.
- **`renderGuestEntryDetail(entry)` (`L2140`, `L2157`, `L2283`):**
  - Change `text-sm` → `text-base` and add `bionic-text` on both `.prose` blocks (`L2140`, `L2157`).
  - Call `if (typeof applyBionicReading === 'function') applyBionicReading(panel);` right alongside `refreshRelativeTimes(panel)`.

---

## Complete File & Legacy Test Impact Plan

1. **[crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs):**
   - Update legacy `indexedDB.open(ARTICLE_DB_NAME, 3)` assertions at `L3853` and `L5443` to `4` (and `e.oldVersion < 4` at `L5444`).
   - Preserve `test_magazine_view_brief_line_clamping` (`L3455-L3465`) by keeping `line-clamp-6` on P1 and `line-clamp-4 leading-relaxed font-sans flex-1` contiguous on P2, and keeping zero `md:` viewport breakpoints (`L3449`).
   - Preserve `test_magazine_grid_css_rules_present` (`L1635`) by keeping `repeat(3, minmax(0, 1fr))` inside `@container mag-center (min-width: 43rem)`.
   - Preserve `test_toast_deduplication_and_detail_is_error_cache_guard` (`L4966-L4980`) by placing `applyBionicReading(liveRoot)` after `lucide.createIcons({ root: liveRoot })` inside `base.html` `htmx:afterSwap`.
2. **Add new TDD tests** across `templates.rs` and `entries.rs` covering:
   - `1rem` (`text-base`) across all rewrite, brief, `.magazine-brief`, `ai_summary_card.html`, and `analysis_cards_inner.html` surfaces + card title hierarchy (`text-xl` P1, `text-lg` P2, `text-lg font-bold` in `ai_grouped_list.html`) + `ai_summary_card.html` `w-4 h-4 ... mt-1` icon + `.magazine-center-col` & `.magazine-p2-grid` `minmax(min(100%, 220px), 1fr)` / `@container mag-center (min-width: 43rem)`.
   - Bionic Reading DOM helper in `base.html` (`Intl.Segmenter` with `NFC`, `TreeWalker`, pure-number guard `/^\p{L}/u`, `!/\p{L}/u` skip, deterministic `force === true` unwrap with `replaceWith` + `normalize()`, `b.bionic-fixation` CSS `font-weight: 700`, `.is-error` exclusion, idempotency, and `DOMContentLoaded` + unconditional `liveRoot` `htmx:afterSwap` hook) + all `index.html` dynamic hooks (`seedInitialFilterCache`, `applyCachedList`, `openEntryDetail`, `syncExecutiveBriefUpdates`, `htmx:afterSwap` pre-cache placement, `handleRewriteError`, `renderGuestEntryDetail`, and `ARTICLE_DB_NAME` v4 cache invalidation).
   - Server `entries.rs` rewrite + fallback endpoints returning `text-base` + `bionic-text` without corrupting plain text.

## Proposed Has (parent **or** labeled solo applies to card)

Stage records proposed values only — parent after Parent-relay **or** labeled solo (acting as orchestrator) writes card Has via Always §4 ([../lenses.md](../lenses.md) · [../solo-entry.md](../solo-entry.md) · [../hitl.md](../hitl.md)).

- **Proposed Has:** `UI=yes API=no DB=no auth/PII/schema=no` (unchanged from seed card — no Has flip)

## Has API / DB (proposed)

- **Has API:** `no` (zero backend endpoints, request shapes, or GraphQL schemas added or modified)
- **Has DB:** `no` (zero SQLite/Miniflux schema changes; `ARTICLE_DB_NAME` v4 is client browser IndexedDB cache versioning only)

## Reference Sources

- Annuaire Design System: [ANNUAIRE_STYLE_GUIDE.md](file:///Users/ptquang86/ws/my-ui/ANNUAIRE_STYLE_GUIDE.md)
- MDN CSS Container Queries (`container-type`, `container-name`, `@container`): <https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_containment/Container_queries>
- MDN `Intl.Segmenter`, `TreeWalker`, `Node.normalize()`, `ChildNode.replaceWith()`: <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Segmenter> · <https://developer.mozilla.org/en-US/docs/Web/API/TreeWalker> · <https://developer.mozilla.org/en-US/docs/Web/API/Node/normalize> · <https://developer.mozilla.org/en-US/docs/Web/API/ChildNode/replaceWith>
- HTMX `htmx:afterSwap` & `hx-swap="outerHTML"` lifecycle: <https://htmx.org/events/#htmx:afterSwap> · <https://htmx.org/attributes/hx-swap/>
- Bionic fixation length reference (`text-vide`): <https://github.com/Gumball12/text-vide>
- Tailwind CSS `text-base` & `line-clamp`: <https://tailwindcss.com/docs/font-size> · <https://tailwindcss.com/docs/line-clamp>
- W3C WCAG 2.x SC 1.4.4 (Resize Text) & SC 1.4.12 (Text Spacing): <https://www.w3.org/WAI/WCAG22/Understanding/resize-text.html> · <https://www.w3.org/WAI/WCAG22/Understanding/text-spacing.html>
