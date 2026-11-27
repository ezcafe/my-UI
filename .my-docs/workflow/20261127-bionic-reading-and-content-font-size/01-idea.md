# Idea — Bionic Reading for Brief & Rewrite + 1rem Content Typography

- **Has UI (tentative):** yes
- **Has API / DB / auth/PII/schema (tentative):** no | no | no
- **Size:** small-to-medium UI & typography enhancement across templates, client JS/CSS, and view tests

## Summary & Shared Understanding (Draft)

Your request has two connected goals to improve reading speed and comfort in `my-ui`:

1. **Apply Bionic Reading to `brief` and `rewrite` content:** Bold the leading letters (fixation point) of words in both Vietnamese and English text without breaking layout, plain-text caching (`textContent`), or XSS escaping.
2. **Display reading content at `1rem` (`16px` / Tailwind `text-base`):** Increase body reading text from the current smaller sizes (`text-xs` / `0.75rem` and `text-sm` / `0.875rem`) to `1rem` so reading matches browser-root accessibility standards while respecting user zoom preferences.

## Playback of What Exists & Tentative Scope (Guesses Marked)

Based on the current codebase (`crates/ui-components/templates/`, `crates/server/src/routes/entries.rs`, and `crates/ui-components/templates/pages/index.html`):

- **Rewrite content (`#ai-rewrite-box .prose`):** Currently renders at `text-sm` (`0.875rem`) across `entry_detail.html`, server-rendered HTMX responses/fallbacks in `entries.rs`, and client-side Guest/sync/error rendering in `index.html`.
  - *Guess:* Apply **both Bionic Reading and `1rem` (`text-base`)** to all rewrite text (ready AI rewrite and fallback summary).
- **Brief surfaces (`Executive Brief` vs. Magazine `.magazine-brief`):** Currently there are two brief surfaces, both styled at `text-xs` (`0.75rem`):
  1. **Executive Brief** (`#page-executive-brief` in Magazine view via `entry_list.html` / `executive_brief_card.html`, and `#list-executive-brief` in List view via `entry_list.html` / `index.html`).
  2. **Magazine card article brief** (`.magazine-brief` summary snippet on Priority 1 hero & Priority 2 grid cards in `entry_list.html`, updated in `index.html`). *(Note: Priority 3 cards only show headlines with `title="..."` browser tooltips, where HTML bolding cannot render.)*
  - *Guess:* Apply **Bionic Reading** to **both Executive Brief and Magazine card `.magazine-brief`** (plus `brief_card.html` / `ai_summary_card.html` if rendered).
- **`1rem` content font size scope (`text-base`):**
  - *Guess (Option A — Recommended):* Apply `1rem` (`text-base`) to **detail reading content** — Rewrite body (`#ai-rewrite-box .prose`) and Detail analysis cards (**Impact Analysis** and **Logical Fallacy & Objectivity Check** in `analysis_cards_inner.html`, currently `text-sm`), while keeping **Executive Brief** and **Magazine `.magazine-brief`** at `text-sm` or `text-xs` so the 3-column Magazine grid does not bloat.
  - *Alternative (Option B):* Apply `1rem` (`text-base`) to **both** detail rewrite content AND Executive Brief / `.magazine-brief` (all "brief + rewrite" content).
  - *Alternative (Option C):* Apply `1rem` (`text-base`) **only** to rewrite content (`#ai-rewrite-box .prose`), leaving analysis cards at `text-sm` and briefs at `text-xs`.
- **Bionic Reading toggle vs. always-on:**
  - *Guess:* **Always-on** by default (zero extra header clutter, consistent with `ANNUAIRE_STYLE_GUIDE.md`), unless you want a toggle button saved in `localStorage`.

## Goals

- Make `brief` and `rewrite` faster to scan with clean, Unicode/Vietnamese-safe Bionic Reading fixation bolding (using `Work Sans` weights `400` normal and `700` bold already loaded in `base.html`).
- Raise body reading font size to `1rem` (`text-base`) consistently across Auth mode, Guest mode, HTMX partial swaps, and cached detail views.
- Keep existing plain-text extraction (`textContent`), IndexedDB/filter caches, and XSS protection intact.

## Non-Goals / Anti-Goals

- Applying Bionic Reading to article headlines/titles, navigation bars, buttons, timestamps, or filter pills.
- Changing database schemas, Miniflux GraphQL queries, or LLM prompts.
- Modifying `title="..."` tooltip attributes on Priority 3 cards (native browser tooltips only accept plain text).

## Constraints

- **Multilingual safety:** Must handle Vietnamese diacritics (e.g., `à`, `á`, `ả`, `ã`, `ạ`, `đ`) and English contractions/hyphens accurately without splitting combining characters or breaking HTML escaping.
- **Annuaire design grammar:** Preserve `Work Sans` (`400`/`700`), light/dark theme contrast, and clean layout hierarchy (`ANNUAIRE_STYLE_GUIDE.md`).
- **Auth + Guest + Cache parity:** Must work identically on initial SSR render, HTMX swaps (`/hx/entries`, `/hx/executive-brief`, `/hx/entries/:id/rewrite`), `syncExecutiveBriefUpdates()`, error fallbacks, and Guest IndexedDB rendering.

## References (Primary Sources Checked)

- Word/grapheme Unicode segmentation & safe DOM traversal: MDN [`Intl.Segmenter`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/Segmenter) and [`TreeWalker`](https://developer.mozilla.org/en-US/docs/Web/API/Document/createTreeWalker)
- Bionic fixation algorithm reference: [`text-vide`](https://github.com/Gumball12/text-vide) (`fixationPoint` word-length highlighting pattern)
- `1rem` accessible body typography: [W3C WAI Accessibility Tutorials / WCAG 2.x SC 1.4.4 & SC 1.4.12](https://www.w3.org/WAI/)

## Open Questions for User (Confirm or Skip)

- **Skip option:** Reply **"just start"** if you want to skip brainstorming and proceed directly with the recommended guesses above.
- **Please confirm the shared scope before design:**
  1. **Which "brief" surfaces get Bionic Reading?** Both **Executive Brief** (Magazine & List views) **and Magazine card summaries (`.magazine-brief`)**, or **only Executive Brief**? *(And confirm Impact/Fallacy analysis cards stay normal text.)*
  2. **Which "nội dung" gets `1rem` (`text-base`) font size?**
     - **Option A (Recommended):** Detail reading body (**Rewrite + fallback** and **Impact/Fallacy analysis cards**) at `1rem`, keeping Executive Brief & Magazine cards compact (`text-xs` or `text-sm`).
     - **Option B:** **Rewrite + Executive Brief** (and/or `.magazine-brief`) all at `1rem`.
     - **Option C:** **Rewrite body only** at `1rem` (leaving analysis cards at `text-sm` and briefs at `text-xs`).
  3. **Always-on or toggle?** Should Bionic Reading be **always-on**, or do you want a **toggle button** (saved in `localStorage`) to turn it on/off?
