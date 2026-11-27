# Test & Verification Report — Bionic Reading for Brief & Rewrite + 1rem Content Typography

**Smoke source:** Build-verify

## Smoke

- **Workspace Build & Test (`cargo test --workspace`):** exit `0` (`286` passed, `0` failed, `0` ignored)
  - `miniflux-client` (`lib`): `24` passed, `0` failed, `0` ignored
  - `server` (`lib`): `153` passed, `0` failed, `0` ignored
  - `server` (`tests/e2e_tests.rs`): `8` passed, `0` failed, `0` ignored (`test_app_starts_and_responds_to_health_check`, `test_mutation_endpoints_reject_with_403_when_auth_disabled`, `test_executive_brief_ineligible_views_and_empty_guard`, `test_executive_brief_cache_hit_and_oob_swaps`, `test_e2e_detail_drawer_analysis_flow`, `test_e2e_authenticated_session_flow`, `test_e2e_guest_mode_flow`, `test_e2e_page_load_and_detail_drawer_rewrite_flow`)
  - `ui-components` (`lib`): `101` passed, `0` failed, `0` ignored
- **Linter (`cargo clippy --workspace --all-targets -- -D warnings`):** exit `0` (zero warnings across all workspace targets)

## Build Verification (`cargo test`)

- **Planned TDD Tests Verified (Red → Green):**
  1. `test_1rem_typography_and_magazine_title_hierarchy_across_templates` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)) — verifies `text-base` and `bionic-text` across Executive Brief (`entry_list.html`, `executive_brief_card.html`, `brief_card.html`, `ai_summary_card.html`), P1/P2 `.magazine-brief`, and `#ai-rewrite-box .prose` (`entry_detail.html`); `text-base` without `bionic-text` on `analysis_cards_inner.html`; P1 `text-xl font-bold`, P2 `text-lg font-bold line-clamp-3`, `ai_grouped_list.html` headline `text-lg font-bold` without `bionic-text`; `ai_summary_card.html` `w-4 h-4 text-foreground shrink-0 mt-1` icon; `.magazine-center-col` and `.magazine-p2-grid` `repeat(auto-fit, minmax(min(100%, 220px), 1fr))` + `@container mag-center (min-width: 43rem)`; error cards stay `text-xs` without `bionic-text`; and `entry_list.html` preserves zero `md:` breakpoints and contiguous `line-clamp-4 leading-relaxed font-sans flex-1`.
  2. `test_bionic_reading_engine_and_hooks_in_base_and_index` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)) — verifies `b.bionic-fixation` CSS, `Intl.Segmenter('vi', ...)` + fallback word segmenter, `.normalize('NFC')`, `createTreeWalker`, `/^\p{L}/u` number guard, `!/\p{L}/u.test(node.nodeValue)` skip, deterministic `force === true` unwrap + `el.normalize()`, `.is-error` / interactive-tag exclusions, `DOMContentLoaded` + unconditional `liveRoot` `htmx:afterSwap` hook in `base.html`; plus `.magazine-center-col` skeleton, `seedInitialFilterCache`, `applyCachedList`, `ARTICLE_DB_NAME` v4 + `isCacheValid`, `openEntryDetail`, `syncExecutiveBriefUpdates` (`force = true`), `htmx:afterSwap` `.is-error` guard + pre-cache placement, `handleRewriteError`, and `renderGuestEntryDetail` in `index.html`.
  3. `test_entry_rewrite_endpoints_use_text_base_and_bionic_marker` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)) — verifies `render_rewrite_box_html`, `get_entry_rewrite_htmx` `!config.ai_enabled` fallback, and LLM error fallback emit `text-base` and `bionic-text` on `#ai-rewrite-box .prose` while keeping plain escaped text intact and preserving `.is-error` + `text-xs` on the error card.
- **Legacy Tests Updated:** `ARTICLE_DB_NAME` v3 checks at [templates.rs#L3853](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3853) and [templates.rs#L5443-L5444](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L5443-L5444) updated to v4.
