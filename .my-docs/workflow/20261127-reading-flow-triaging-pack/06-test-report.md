# Test & Verification Report — Reading Flow & Triaging Pack

**Result:** success
**Test path last run:** test-lite
**Test path:** lite
**Smoke source:** Build-verify
**Updated:** 2026-11-27T21:01+07:00

## Test-lite

- **Test Suite (`cargo test --workspace`):** exit `0` (`283` passed, `0` failed, `0` ignored)
  - `miniflux-client` (`lib`): `24` passed, `0` failed, `0` ignored
  - `server` (`lib`): `152` passed, `0` failed, `0` ignored
  - `server` (`tests/e2e_tests.rs`): `8` passed, `0` failed, `0` ignored (`test_app_starts_and_responds_to_health_check`, `test_e2e_authenticated_session_flow`, `test_e2e_guest_mode_flow`, `test_mutation_endpoints_reject_with_403_when_auth_disabled`, `test_executive_brief_ineligible_views_and_empty_guard`, `test_executive_brief_cache_hit_and_oob_swaps`, `test_e2e_page_load_and_detail_drawer_rewrite_flow`, `test_e2e_detail_drawer_analysis_flow`)
  - `ui-components` (`lib`): `99` passed, `0` failed, `0` ignored
- **Linter (`cargo clippy --workspace --all-targets -- -D warnings`):** exit `0` (zero warnings across all workspace targets)
- **Reference Sources:** Cargo & Clippy documentation (`doc.rust-lang.org/clippy/`, `doc.rust-lang.org/cargo/commands/cargo-test.html`)

## Smoke

- **Workspace Build & Test (`cargo test --workspace`):**
  - `miniflux-client` (`lib`): 24 passed, 0 failed, 0 ignored
  - `server` (`lib`): 152 passed, 0 failed, 0 ignored
  - `server` (`tests/e2e_tests.rs`): 8 passed, 0 failed, 0 ignored
  - `ui-components` (`lib`): 99 passed, 0 failed, 0 ignored
  - **Total:** 283 passed, 0 failed, 0 ignored

## Build Verification (`cargo test`)

- **Planned TDD Tests Verified:**
  1. `test_magazine_cards_use_full_card_stretched_link_click` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)) — P1, P2, and P3 cards use `<a id="mag-title-..." class="magazine-card-link">` inside `position: static` wrappers, read `dataset.feedTitle` safely for apostrophes (`"O'Reilly"`), preserve `.magazine-card-link` in `syncExecutiveBriefUpdates()`, and wire stretched-link + `:has(.magazine-card-link:focus-visible)` CSS in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html).
  2. `test_detail_prev_next_buttons_and_navigation_helpers` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)) — [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html), `openDetailWithSkeleton`, and `renderGuestEntryDetail` render `type="button"` `#detail-prev-btn` / `#detail-next-btn` with `gap-1.5`, `detailNavIds` keeps current-session reads while excluding prior-session reads, `updateDetailNavButtons` disables both on deep links (`indexOf === -1`), and `ARTICLE_DB_NAME` uses v3 with `detail-prev-btn` cache guard.
  3. `test_feed_and_relative_time_metadata_in_list_detail_and_guest` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)) — `format_relative_time_at` thresholds, guarded `" · "` separator, `#entry-{id}-dot` placement outside `.flex-col`, shared `formatRelativeTimeLabel` / `refreshRelativeTimes` in both Auth and Guest modes, and `Feed · <time>` rendering across list, detail, and Guest views without `author` or `reading_time`.
  4. `test_toast_role_status_hover_focus_pause_and_guest_undo` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)) — `role="status" aria-atomic="true"`, shared `showClientToast`, `.toast-undo-btn` deduplication, `8000`ms Undo / `5000`ms OOB auto-dismiss with hover/focus/in-flight pause, `HX-Reswap !== 'none'` Auth Undo removal guard, `.toast-undo-btn` error routing before `#entries-list`, and Guest `markVisiblePageLocallyRead` + `undoGuestMarkPageRead` synchronous `readSet` deletion + restore.
  5. `test_hx_mark_page_read_includes_undo_toast_and_mark_page_unread_restores_page` ([routes/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs)) and `test_mutation_endpoints_reject_with_403_when_auth_disabled` ([e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs)) — `POST /hx/mark-page-read` emits `role="status"` Undo toast for `unread_entry_ids` only and omits Undo when empty; `POST /hx/mark-page-unread` validates `entry_ids` (`400 Bad Request` on malformed IDs), skips mutation when empty, invalidates `entry:{id}` cache, clamps out-of-range `offset`, preserves `ai_service` + `cache` parity, and rejects guest mode with `403`.
- **Legacy Tests Updated & Verified:** `test_render_title_only_entry_item`, `test_render_entry_list_with_title_only_items`, `test_detail_template_omits_author`, `test_detail_template_omits_metadata_and_orders_sections`, `test_render_toast_oob_template`, `test_ai_magazine_view_renders_3_column_layout`, `test_entry_list_template_renders_magazine_view_and_cards`, `test_entry_detail_omits_ddmmyyyy_time_element_in_both_headers`, `test_entry_list_guest_json_excludes_original_content_and_includes_images_and_rewrite`, `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time`, `test_toast_deduplication_and_detail_is_error_cache_guard`, `test_get_entry_detail_formats_published_at_as_ddmmyyyy`, `test_get_entry_detail_triggers_rewrite_when_missing_and_omits_metadata`, Guest JSON integration test in `entries.rs`, and `test_mutation_endpoints_reject_with_403_when_auth_disabled` in `e2e_tests.rs`.
