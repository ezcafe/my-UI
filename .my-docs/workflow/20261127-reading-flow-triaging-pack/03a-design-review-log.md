Result: clean

# Design Review Log — Reading Flow & Triaging Pack

## Key Takeaways

- **Overall verdict:** **Clean** — All Round-1 findings and all **4 Round-2 findings** (1 Critical, 3 Major) are resolved in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md).
- **Surface statuses:**
  - **API Contract ([03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03a-api-contract-review.md)):** `clean`
  - **DB Surface:** `skipped` (`Has DB = no`; no database schema changes)
  - **Grill Gate:** Closed (`Result: frontier-empty` in [02b-grill.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/02b-grill.md))
- **Ready for Gate B / TDD build:** Every slice boundary in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs), all impacted legacy assertions in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs), and [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs), and all Auth/Guest script paths are accounted for.

---

## Round 2 Findings Verification

| # | Round 2 Finding | Status | Verification Notes |
|---|-----------------|--------|--------------------|
| 1 | `[CRITICAL]` `syncExecutiveBriefUpdates()` destroys `<a class="magazine-card-link">` inside Magazine headings | **Resolved** | [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L107-L110), [L143-L154](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L143-L154), [L176-L177](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L176-L177), and [L234](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L234) move `id="mag-title-{{ entry.id }}"` and `onclick` onto `<a class="magazine-card-link">`, add `.magazine-card-link { color: inherit; text-decoration: none; }` and `.magazine-card.read .magazine-card-link` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html), update `syncExecutiveBriefUpdates()` to target `(magTitleEl.querySelector('.magazine-card-link') || magTitleEl).textContent`, and update [templates.rs#L3073](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L3073) while keeping `id="mag-title-101"` / `id="mag-title-102"` checks passing. |
| 2 | `[MAJOR]` Auth Undo error-response contract (`HX-Reswap: none`, missing `showToast` in Auth mode, duplicate `#entries-list` error toast) | **Resolved** | Verified in [03a-api-contract-review.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03a-api-contract-review.md): [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) preserves the Undo toast on `HX-Reswap: none`, shared `showClientToast(message, isError = false)` supports Auth + Guest without breaking Guest `showToast` slice boundaries, and `htmx:responseError` / `htmx:sendError` handle `.toast-undo-btn` first with early `return;`. |
| 3 | `[MAJOR]` `formatRelativeTimeLabel` & `refreshRelativeTimes` inside Guest-only `{% if !auth_enabled %}` block | **Resolved** | [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L173-L175) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md#L23) place both helpers in the shared `<script>` block immediately after `safeHref` ([index.html#L350](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L350), above `{% if !auth_enabled %}` at [index.html#L1795](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1795)), making them available in both Auth and Guest modes. |
| 4 | `[MAJOR]` Missed legacy test (`templates.rs#L3626-L3689`) & Guest `undoGuestMarkPageRead` restoration contract | **Resolved** | [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L235) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md#L31) cover `test_entry_detail_omits_ddmmyyyy_time_element_in_both_headers` (`L3626-L3689`), and [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L193-L200) specifies `offset`, `advancedPage`, synchronous `readSet` deletion before restore, IndexedDB cleanup, `invalidateFilterCache()`, and `htmx.ajax('GET', ...)` vs `syncDomReadStates()` + `syncDetailNavigationIds()`. |

---

## Remaining Findings

None — zero Critical or Major findings remain.

*(Minor implementation note: in `format_relative_time_at`, using `dt.with_timezone(&Utc).format("%d/%m/%Y")` for ages `>= 604800` seconds ensures exact UTC date parity with JS `getUTCDate()` / `getUTCMonth()` / `getUTCFullYear()` when RFC 3339 timestamps carry non-UTC offsets.)*

---

## Reference Sources

- Annuaire Design System: [ANNUAIRE_STYLE_GUIDE.md](file:///Users/ptquang86/ws/my-ui/ANNUAIRE_STYLE_GUIDE.md)
- HTMX `HX-Reswap`, `hx-vals`, `hx-swap-oob`, `htmx:response:error`: <https://htmx.org/reference/#response_headers> · <https://htmx.org/attributes/hx-vals/> · <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/events/#htmx:responseError>
- W3C WAI-ARIA `status` role (WCAG 2.2 SC 4.1.3) & WCAG 2.2 SC 2.2.1 Timing Adjustable: <https://www.w3.org/WAI/ARIA/apg/patterns/alert/> · <https://www.w3.org/WAI/WCAG22/Understanding/timing-adjustable.html> · <https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html>
- Axum `Form` & `serde_urlencoded`: <https://docs.rs/axum/latest/axum/extract/struct.Form.html> · <https://docs.rs/serde_urlencoded/latest/serde_urlencoded/>
- Askama 0.12 & Chrono 0.4: <https://docs.rs/askama/latest/askama/> · <https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339>
