# Code review log

**Overall Result:** clean  
(file-top alone never pays Review clean)

## Key Takeaways

- **All Round 1 review findings are resolved and verified:** Both the **1 Major UI Craft defect** (320px mobile detail header overflow) and all **6 Minor findings** (WCAG 2.2 SC 2.2.1 hover/focus timer resume conflict, Guest deep-link `detailNavIds` race, duplicate ID cache scans, empty `status=""` mismatch, `entries.rs` mutation/count duplication, and skeleton Prev/Next button states) are cleanly fixed in the working tree.
- **No regressions across the workspace:** `cargo clippy --workspace --all-targets -- -D warnings` exits `0` with zero warnings, and `cargo test --workspace` passes all **283** tests (`24` `miniflux-client`, `152` `server` unit, `8` `server` e2e, `99` `ui-components`, `0` failed, `0` ignored).
- **Next in review skill:** `code-review-phase` is `clean`; proceeding to planned `lens-api` (`05-lens-api.md`).

---

## Adversarial

**Result:** clean

### Round 1 Findings Verification

| # | Finding | Status | Verification Notes |
|---|---------|--------|--------------------|
| 1 | `[MINOR]` WCAG 2.2 SC 2.2.1 hover + keyboard focus timer resume conflict | **Resolved** | `resumeToastTimer(toast)` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L643-L653) guards against resuming when `toast.contains(document.activeElement)` or `toast.matches(':hover')` is true. |
| 2 | `[MINOR]` Guest deep-link `/?entry=101` hides active row before `detailNavIds` initializes | **Resolved** | `DOMContentLoaded` in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1372-L1382) runs `syncDetailNavigationIds()` and `checkUrlForEntry()` only when `AUTH_ENABLED`; Guest mode waits for `finishGuestDbInit()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L2334-L2348)) to run `syncDomReadStates()` -> `syncDetailNavigationIds()` -> `checkUrlForEntry()`. |
| 3 | `[MINOR]` Duplicate `entry_ids` trigger redundant cache invalidation lock scans | **Resolved** | `parse_validated_entry_ids` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L44-L92) deduplicates IDs in order via `HashSet` across both JSON-array and CSV paths; verified by `test_parse_validated_entry_ids_rejects_invalid_and_accepts_valid`. |
| 4 | `[MINOR]` Empty `status=""` diverges between `mark_page_read_htmx` and `build_graphql_filter` | **Resolved** | `mark_page_read_htmx` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1545-L1550) normalizes via `.map(str::trim).filter(|s| !s.is_empty())` and treats `None | Some("unread")` as unread, matching `build_graphql_filter`. |

### Remaining Findings

None — zero Critical or Major findings remain.

---

## Quality

**Result:** clean

### Five-Axis & UI Craft Assessment

- **Correctness:** Rust `format_relative_time_at` and JS `formatRelativeTimeLabel` align on all buckets; `ToastUndoPayload` serializes flat `serde_urlencoded`-safe `hx-vals`; `HX-Reswap !== 'none'` keeps Auth Undo available after upstream errors; Guest synchronous `readSet` deletion prevents premature auto-advance. Skeleton Prev/Next buttons in `openDetailWithSkeleton` now use `disabled:opacity-40 disabled:pointer-events-none` and wire `navigateDetailArticle(-1)` / `navigateDetailArticle(1)` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1253-L1272)).
- **Readability & Architecture:** Shared helpers `update_entries_status_and_invalidate` and `query_entries_total` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1434-L1509) eliminate the ~55-line mutation, cache invalidation, and count-query duplication between `mark_page_read_htmx` and `mark_page_unread_htmx`.
- **Security:** `AuthenticatedUser` + `config.auth_enabled` `403` guards, positive `i64` validation (`400 Bad Request`), `DefaultBodyLimit::max(64 * 1024)` (`413`), `serde_json` GraphQL string escaping, `escapeHtml` / `escapeToastHtml`, `dataset.feedTitle` DOM lookup, and non-leaking `handle_htmx_error` responses hold across all modified surfaces.
- **Performance:** `POST /hx/mark-page-unread` batches all reverted IDs into one `updateEntriesStatus` mutation; deduplicated IDs cap cache invalidation to unique IDs; `syncDetailNavigationIds` avoids session-read churn.
- **UI Craft (`Has UI = yes`):**
  - Stretched-link `.magazine-card-link` preserves P1/P2/P3 layout and `.magazine-card:has(.magazine-card-link:focus-visible)` focus outline; hero `text-white/85` and standard `text-muted-foreground` meet WCAG AA contrast; `<time>` inherits `tabular-nums lining-nums`.
  - **Mobile 320px Fit Verified:** `@container workspace (max-width: 360px)` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L286-L299) uses `#detail-panel` ID+class specificity `(1,1,0)` (overriding Tailwind Play CDN `(0,1,0)` utilities) to hide `.detail-original-label` and size `.detail-original-link` / `.detail-original-skeleton` to `2rem`, bringing the 320px detail header from ~316px down to ~259px inside the 296px content width across [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L3-L44), `openDetailWithSkeleton()`, and `renderGuestEntryDetail()`.

### Round 1 Findings Verification

| # | Finding | Status | Verification Notes |
|---|---------|--------|--------------------|
| 1 | `[MAJOR]` Detail header overflows 320px mobile viewports (~316px in 296px width) | **Resolved** | Added `@container workspace (max-width: 360px)` rules in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L286-L299), `.detail-original-link` / `.detail-original-label` in [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L34-L44) and `renderGuestEntryDetail()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L2227-L2237)), and `.detail-original-skeleton` in `openDetailWithSkeleton()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1275)). |
| 2 | `[MINOR]` Duplicated mutation, cache invalidation, and total count query in `entries.rs` | **Resolved** | Extracted `update_entries_status_and_invalidate` and `query_entries_total` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1434-L1509). |
| 3 | `[MINOR]` Skeleton detail nav buttons use unconditional `opacity-40 pointer-events-none` | **Resolved** | Updated `openDetailWithSkeleton` in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1253-L1272) to `disabled:opacity-40 disabled:pointer-events-none` with `onclick="navigateDetailArticle(-1)"` and `onclick="navigateDetailArticle(1)"`. |

### Remaining Findings

None — zero Critical or Major findings remain.

---

## Merged

**Result:** clean

- `05-lens-api.md`: `Result: clean` (copied by parent for `Lens plan: api` per `lenses-overview.md`).

---

## Checks Observed

- `cargo clippy --workspace --all-targets -- -D warnings` -> exit `0`
- `cargo test --workspace` -> exit `0` (`miniflux-client`: `24` passed; `server` unit: `152` passed; `server` `e2e_tests`: `8` passed; `ui-components` unit: `99` passed; total `283` passed, `0` failed, `0` ignored)

## Reference Sources

- Annuaire Design System: [ANNUAIRE_STYLE_GUIDE.md](file:///Users/ptquang86/ws/my-ui/ANNUAIRE_STYLE_GUIDE.md)
- CSS Container Queries, `:has()`, and `:focus-visible`: <https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_containment/Container_queries> · <https://developer.mozilla.org/en-US/docs/Web/CSS/:has> · <https://developer.mozilla.org/en-US/docs/Web/CSS/:focus-visible>
- MDN `focusout` & `relatedTarget` / WCAG 2.2 SC 2.2.1 / WAI-ARIA `status`: <https://developer.mozilla.org/en-US/docs/Web/API/Element/focusout_event> · <https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable.html> · <https://www.w3.org/WAI/ARIA/apg/patterns/alert/>
- HTMX `HX-Reswap`, `hx-vals`, `hx-swap-oob`, `htmx:afterRequest`: <https://htmx.org/reference/#response_headers> · <https://htmx.org/attributes/hx-vals/> · <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/events/#htmx:afterRequest>
- Axum `Form` & `serde_urlencoded`: <https://docs.rs/axum/latest/axum/extract/struct.Form.html> · <https://docs.rs/serde_urlencoded/latest/serde_urlencoded/>
