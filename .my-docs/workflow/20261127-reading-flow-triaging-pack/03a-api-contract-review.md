Result: clean

# API Contract Review — Reading Flow & Triaging Pack

## Key Takeaways

- **Contract verdict:** **Clean** — All **5 Round-1 findings** and the **1 Round-2 Major finding** (`POST /hx/mark-page-unread` `HX-Reswap: none` handling, shared `showClientToast`, and `.toast-undo-btn` error routing) are resolved in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md).
- **Verified server contract:** `POST /hx/mark-page-read` and `POST /hx/mark-page-unread` enforce `AuthenticatedUser` + `403` when `auth_enabled = false`, validate positive `i64` IDs (`400 Bad Request` on malformed tokens), guard empty ID lists, invalidate `"entry:{id}"` alongside list cache keys, clamp `target_offset`, and preserve `ai_service` + `cache` parity.
- **Verified client & HTMX contract:** `ToastUndoPayload` serializes `serde_urlencoded`-safe `hx-vals` (comma-joined `entry_ids`, `skip_serializing_if = "Option::is_none"`), `base.html` removes the Undo toast only when `evt.detail.successful === true && reswap.toLowerCase() !== 'none'`, shared `showClientToast` works in both Auth and Guest modes, and `htmx:responseError` / `htmx:sendError` route `.toast-undo-btn` failures cleanly without duplicate `#entries-list` error toasts.

---

## Surfaces Verified

- `POST /hx/mark-page-read` (`mark_page_read_htmx`, `MarkPageReadForm` with `unread_entry_ids`, [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L31-L39), [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1300-L1441))
- `POST /hx/mark-page-unread` (`mark_page_unread_htmx` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs) & [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L112))
- GraphQL `updateEntriesStatus(entryIds: [ID!]!, status: EntryStatus!)` ([mutation.rs](file:///Users/ptquang86/ws/my-ui/crates/miniflux/src/schema/mutation.rs#L34-L50))
- `ToastTemplate`, `ToastUndoAction`, and `ToastUndoPayload` ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L228-L232), [toast.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html), [error.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L10-L69))
- `GuestEntryPayload<'a>` & `#entries-guest-data` + Auth IndexedDB v3 cache ([templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L172-L216), [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L889-L950))
- Error-handling & Undo lifecycle listeners (`htmx:beforeRequest`, `htmx:afterRequest`, `htmx:afterSwap`, `htmx:responseError`, `htmx:sendError`) in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L586-L636) and [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1620-L1752)

---

## Round 2 Findings Verification

| # | Finding | Status | Verification Notes |
|---|---------|--------|--------------------|
| 1 | Round-1 items (non-test `chrono`, `ToastUndoPayload` `serde_urlencoded`, `unread_entry_ids` filter, strict `i64` validation + `"entry:{id}"` + `ai_service` + offset clamp, v3 cache + `dataset.feedTitle`) | **Resolved** | Confirmed in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L37-L223) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md#L7-L29). |
| 2 | `[MAJOR]` Auth Undo error contract (`200 OK` + `HX-Reswap: none`, missing `showToast` in Auth mode, duplicate `#entries-list` error toast) | **Resolved** | [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L156-L160) and [L202](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md#L202) (plus [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/04-tasks.md#L15-L27)) specify checking `reswap.toLowerCase() !== 'none'` before removing the Undo toast, defining shared `showClientToast(message, isError = false)` in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) while keeping Guest `showToast(message)` in place for slice tests, and routing `.toast-undo-btn` errors first with early `return;` in `htmx:responseError` / `htmx:sendError`. |

---

## Remaining Findings

None — zero Critical or Major findings remain.

---

## Reference Sources

- Axum `Form` & `serde_urlencoded`: <https://docs.rs/axum/latest/axum/extract/struct.Form.html> · <https://docs.rs/serde_urlencoded/latest/serde_urlencoded/>
- HTMX `HX-Reswap`, `hx-vals`, `hx-swap-oob`, `hx-disabled-elt`, `htmx:response:error`: <https://htmx.org/reference/#response_headers> · <https://htmx.org/attributes/hx-vals/> · <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/attributes/hx-disabled-elt/> · <https://htmx.org/events/#htmx:responseError>
- Askama 0.12 escaping & Chrono 0.4 `DateTime::parse_from_rfc3339`: <https://docs.rs/askama/latest/askama/> · <https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339>
