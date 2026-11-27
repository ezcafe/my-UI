Result: clean

# TDD Plan Review — Bionic Reading for Brief & Rewrite + 1rem Content Typography

**Verdict:** Planned tests in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-bionic-reading-and-content-font-size/04-tasks.md) cover all template, CSS, client DOM, IndexedDB cache, and server route changes from [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-bionic-reading-and-content-font-size/03-design.md) in clean failing-test-first order.

## Case-by-Case Review

1. **Case 1 — `test_1rem_typography_and_magazine_title_hierarchy_across_templates`**
   - **Coverage:** Verifies `text-base` (`1rem`) and `bionic-text` across Executive Brief partials, `.magazine-brief` (P1/P2), and `#ai-rewrite-box .prose` in [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html), plus `text-base` without `bionic-text` in [analysis_cards_inner.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards_inner.html).
   - **Hierarchy & Layout:** Checks P1 `text-xl font-bold`, P2 `text-lg font-bold line-clamp-3`, [ai_grouped_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html) headline `text-lg font-bold`, [ai_summary_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_summary_card.html) `w-4 h-4 text-foreground shrink-0 mt-1` icon, `.magazine-center-col`, and `.magazine-p2-grid` `auto-fit` + `@container mag-center (min-width: 43rem)`.
   - **Negative Guards:** Confirms error cards remain `text-xs` and [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html) preserves existing `line-clamp-4 leading-relaxed font-sans flex-1` and zero viewport-breakpoint contracts in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs).
   - **Build Note:** Assert explicitly that [analysis_cards_inner.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards_inner.html) and [ai_grouped_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html) headline links do **not** include `bionic-text`.

2. **Case 2 — `test_bionic_reading_engine_and_hooks_in_base_and_index`**
   - **Engine Coverage ([base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)):** Covers `b.bionic-fixation` (`font-weight: 700`), `Intl.Segmenter('vi', ...)` word/grapheme segmentation, `.normalize('NFC')`, `createTreeWalker`, `/^\p{L}/u` number guard, `!/\p{L}/u.test(node.nodeValue)` skip, deterministic `force === true` unwrap + `el.normalize()`, `computeBionicFixationLength`, `.is-error` / interactive-tag exclusions, `DOMContentLoaded`, and unconditional `liveRoot` `htmx:afterSwap` hook.
   - **Dynamic Hooks ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)):** Covers `seedInitialFilterCache`, `applyCachedList`, `openEntryDetail`, `syncExecutiveBriefUpdates` (`force = true`), `htmx:afterSwap` pre-cache placement before `filterCache.set`, `handleRewriteError`, `renderGuestEntryDetail`, `text-base` + `bionic-text` in JS-rendered `.prose` / `#list-executive-brief p`, and `ARTICLE_DB_NAME` v4 + `isCacheValid`.
   - **Legacy Updates:** Planned Task 3 correctly updates existing v3 assertions at [templates.rs#L3853](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3853) and [templates.rs#L5443-L5444](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L5443-L5444) to v4.
   - **Build Note:** Ensure Case 2 also verifies the non-`Intl.Segmenter` fallback branch in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) and confirms `.is-error` class toggle on `#list-executive-brief` in `htmx:afterSwap` keeps error `<p>` plain (`text-xs`, no `bionic-text`).

3. **Case 3 — `test_entry_rewrite_endpoints_use_text_base_and_bionic_marker`**
   - **Coverage ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)):** Verifies `render_rewrite_box_html` ([L2006](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2006)), `get_entry_rewrite_htmx` `!config.ai_enabled` fallback ([L2237](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2237)), and LLM error fallback ([L2277](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L2277)) emit `text-base` and `bionic-text` on `#ai-rewrite-box .prose` with plain escaped text intact.
   - **Build Note:** Assert the LLM error fallback card itself retains `.is-error` and plain `text-xs` while only its fallback `.prose` body carries `text-base` and `bionic-text`.

## TDD Ordering Check

- Tasks 1–4 in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-bionic-reading-and-content-font-size/04-tasks.md) each specify writing/updating the failing test before editing templates, JS, CSS, or Rust handlers, followed by `cargo test -p ui-components` and `cargo test --workspace`.
- No unrequested changes or missing behavioral contracts remain.
