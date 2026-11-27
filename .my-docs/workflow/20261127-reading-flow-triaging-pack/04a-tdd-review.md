Result: clean

# TDD Plan Review — Reading Flow & Triaging Pack

## Key Takeaways

- **Verdict:** **Clean** (`04a ok`) — All 5 planned TDD test groups and all impacted legacy test updates in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md) align with [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md), [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03a-api-contract-review.md), and [03a-design-review-log.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03a-design-review-log.md).
- **Coverage:** Unit tests (`templates.rs`), route/handler integration tests (`routes/mod.rs` / `entries.rs`), and end-to-end auth/guest guard tests (`e2e_tests.rs`) cover every user-visible behavior, edge case, and security/validation boundary.
- **Note for `build`:** Include the edge-case assertions noted below (backward-compatible `unread_entry_ids: None` fallback, `400 Bad Request` validation on `mark_page_read_htmx`, `entry:{id}` cache invalidation, and `mark_entry_read_htmx` empty fallback) during Red → Green → Refactor.

---

## Planned Test Verification

| # | Planned Test | Target Files | Edge Cases & Assertions Verified | Status |
|---|--------------|--------------|----------------------------------|--------|
| 1 | `test_magazine_cards_use_full_card_stretched_link_click` | [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) | P1/P2/P3 `<a id="mag-title-..." href="/entries/..." class="magazine-card-link">` inside `position: static` headings/wrappers; modifier-key guard; apostrophe-safe `dataset.feedTitle` lookup (e.g., `"O'Reilly"`); `syncExecutiveBriefUpdates()` preserving `.magazine-card-link`; `base.html` CSS (`inset: 0`, `z-index: 1`, `:has(.magazine-card-link:focus-visible)`, `.magazine-card.read .magazine-card-link`). | Covered |
| 2 | `test_detail_prev_next_buttons_and_navigation_helpers` | [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) | `type="button"` `#detail-prev-btn` / `#detail-next-btn` with `gap-1.5` in `entry_detail.html`, `openDetailWithSkeleton(title, brief, imageUrl)` (3-arg signature preserved), and `renderGuestEntryDetail`; `detailNavIds` excluding prior-session reads while keeping current-session reads (not rebuilt in `markLocallyRead`/`unmarkLocallyRead`/`toggleLocalReadStatus`); disabling both buttons on deep links (`indexOf === -1`); `ARTICLE_DB_NAME` v3 + `detail-prev-btn` `isCacheValid` check. | Covered |
| 3 | `test_feed_and_relative_time_metadata_in_list_detail_and_guest` | [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) | `format_relative_time_at` thresholds (`""`, `YYYY-MM-DD`, `0..=59s`, `Xm ago`, `Xh ago`, `Xd ago`, UTC `DD/MM/YYYY` at `>= 604800s`, unparseable fallback); `" · "` guard for empty `feed_title` or `published_at`; `#entry-{id}-dot` outside `.flex-col`; `formatRelativeTimeLabel` / `refreshRelativeTimes` present in `IndexTemplate { auth_enabled: true }` (above `{% if !auth_enabled %}`); `GuestEntryPayload` + `renderLocalAllArticles` + `renderGuestEntryDetail`; `mark_entry_read_htmx` absent `publishedAt` fallback returning `""`; Magazine cards free of source/time metadata; `author` and `reading_time` omitted. | Covered (include `mark_entry_read_htmx` `""` check) |
| 4 | `test_toast_role_status_hover_focus_pause_and_guest_undo` | [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs) | `role="status" aria-atomic="true"` on `toast.html`, `showClientToast`, `showToast`, `showUndoToast`; `showClientToast` using `escapeToastHtml`, `data-auto-dismiss="true"`, and `alert-circle` on `isError`; `8000`ms Undo / `5000`ms OOB auto-dismiss paused on `mouseenter`, `focusin`, and `dataset.undoInFlight === 'true'`; `.toast-undo-btn` deduplication; Auth Undo toast removed **only** when `evt.detail.successful === true && reswap.toLowerCase() !== 'none'` (surviving `200 OK` + `HX-Reswap: none`); `.toast-undo-btn` checked first with early `return;` in `htmx:responseError` / `htmx:sendError`; Guest `markVisiblePageLocallyRead` + `undoGuestMarkPageRead(snapshot)` deleting `snapshot.ids` from `readSet` synchronously before restoring (`htmx.ajax('GET', ...)` when `advancedPage` else `syncDomReadStates()` + `syncDetailNavigationIds()`), clearing IndexedDB, and calling `invalidateFilterCache()`. | Covered |
| 5 | `test_hx_mark_page_read_includes_undo_toast_and_mark_page_unread_restores_page` | [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs), [mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs), [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs) | `POST /hx/mark-page-read`: OOB `role="status"` toast with `hx-post="/hx/mark-page-unread"`, `serde_urlencoded`-compatible `hx-vals` (comma-joined `unread_entry_ids` only, omitted `None`/empty `search`, quote-escaped `search`), fallback to `entry_ids` when `unread_entry_ids` is `None`, skip mutation + omit Undo when `unread_entry_ids` is empty, `400 Bad Request` on invalid IDs in `entry_ids` or `unread_entry_ids`, and `entry:{id}` cache invalidation. `POST /hx/mark-page-unread`: `403` when `auth_enabled = false`, `400` on invalid non-numeric/`<= 0` `entry_ids`, empty-ID no-op guard, `entry:{id}` + list cache invalidation, `offset` clamping against `total`, and `ai_service` + `cache` parity restoring the page with `"Mark as read undone."` toast (`undo: None`). | Covered (include `mark_page_read_htmx` `400` / `None`-fallback / `entry:{id}` checks) |

---

## Legacy Test Update Readiness

All existing test assertions affected by the changes in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md#L30-L31) are explicitly mapped (including `templates.rs`, `entries.rs`, and `e2e_tests.rs`), and slice-boundary anchors in `base.html` (`htmx:afterSwap` / `htmx:oobAfterSwap`) and `index.html` (`openDetailWithSkeleton`, `showToast` → `triggerNativeBookmark`, `syncDomReadStates` → `markLocallyRead`) are preserved by design.

---

## Reference Sources

- HTMX `hx-vals`, `hx-swap-oob`, `HX-Reswap`, `htmx:responseError`: <https://htmx.org/reference/#response_headers> · <https://htmx.org/attributes/hx-vals/> · <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/events/#htmx:responseError>
- Axum `Form` & `serde_urlencoded`: <https://docs.rs/axum/latest/axum/extract/struct.Form.html> · <https://docs.rs/serde_urlencoded/latest/serde_urlencoded/>
- Chrono `DateTime::parse_from_rfc3339`: <https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339>
