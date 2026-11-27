# Lens

**Lens:** api  
**Result:** clean

## Key Takeaways

- **API contract and design alignment are clean:** All endpoints, form extractors, OOB toast payloads, guest JSON payloads, cache invalidations, and client HTMX error handlers match [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md).
- **Security, validation, and error contracts hold:** `POST /hx/mark-page-read` and `POST /hx/mark-page-unread` enforce `AuthenticatedUser` + `403 Forbidden` when `auth_enabled = false`, reject malformed or non-positive IDs with `400 Bad Request`, deduplicate IDs before cache invalidation, keep upstream errors behind `handle_htmx_error` (`200 OK` + `HX-Reswap: none` + non-leaking OOB toast), and keep Undo clickable on upstream/network failure.

---

## Surfaces Verified

- `POST /hx/mark-page-read` (`mark_page_read_htmx`) in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1511-L1625)
- `POST /hx/mark-page-unread` (`mark_page_unread_htmx`) in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1627-L1697) and [app.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/app.rs#L114)
- Input parsing & shared helpers (`MarkPageReadForm`, `parse_validated_entry_ids`, `update_entries_status_and_invalidate`, `query_entries_total`) in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L32-L92) and [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1434-L1509)
- `ToastUndoPayload`, `ToastUndoAction`, `ToastTemplate`, and `GuestEntryPayload` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L206-L306), [toast.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/toast.html), and [error.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L10-L69)
- Client HTMX request/error/swap handling (`HX-Reswap !== 'none'`, `.toast-undo-btn` error routing, `hx-vals`) in [base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L711-L792) and [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L48-L62) / [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1848-L1916)

---

## API Checklist

| Check | Status | Evidence |
|-------|--------|----------|
| Typed input & output; create-input vs full entity | **Pass** | `MarkPageReadForm` (`entry_ids`, `#[serde(default)] unread_entry_ids`, `status`, `search`, `feed_id`, `category_id`, `offset`), `ToastUndoPayload` (serialized with `skip_serializing_if = "Option::is_none"` for `serde_urlencoded`), `ToastUndoAction`, and `GuestEntryPayload` exclude auth-only fields (`sanitized_content`, `author`, `reading_time`). |
| One stable error shape; correct status codes | **Pass** | Unauthenticated requests redirect via `AuthenticatedUser`; `!config.auth_enabled` returns `403 Forbidden`; invalid IDs return `400 Bad Request` (`"Invalid entry IDs"`); oversized payloads (>64KB) return `413`; upstream errors return `200 OK` + `HX-Reswap: none` + OOB error toast (`handle_htmx_error`), and client `htmx:afterRequest` / `responseError` / `sendError` keep `.toast-undo-btn` intact for retry without duplicate `#entries-list` error toasts. |
| Validate at system edges only | **Pass** | `parse_validated_entry_ids` validates and deduplicates `entry_ids` / `unread_entry_ids` once at the HTTP handler boundary; `update_entries_status_and_invalidate` and `query_entries_total` consume trusted validated slices and typed structs without re-parsing. |
| Lists paginated; filters via query params; PATCH where appropriate | **Pass** | `GET /hx/entries` and list re-renders use `EntriesFilterQuery` (`limit`, `offset`, `status`, `feed_id`, `category_id`, `search`). `mark_page_unread_htmx` clamps `target_offset` against `total`. HTML/HTMX mutation routes follow the repository's `POST /hx/...` convention. |
| Additive fields; no silent breaking changes | **Pass** | `unread_entry_ids` is optional (`#[serde(default)]`) and falls back to `entry_ids` when omitted (`test_hx_mark_page_read_endpoint`). `GuestEntryPayload` adds `feed_title` and `published_at` additively. |
| Naming consistent with the repo | **Pass** | `/hx/mark-page-unread` (`mark_page_unread_htmx`) directly mirrors `/hx/mark-page-read` (`mark_page_read_htmx`) and `/hx/entries/{id}/read` (`mark_entry_read_htmx`). |
| Mutating endpoints: idempotent or documented unsafe-to-retry | **Pass** | `mark_page_read_htmx` (`READ`) and `mark_page_unread_htmx` (`UNREAD`) issue idempotent `updateEntriesStatus` status-set mutations; buttons attach `hx-disabled-elt="this"`. |
| Contract in `03-design` matches what code exposes | **Pass** | Verified `POST /hx/mark-page-read`, `POST /hx/mark-page-unread`, `ToastUndoPayload`, `GuestEntryPayload`, `"entry:{id}"` cache invalidation, `ai_service` + `Some(&cache)` parity, `mark_entry_read_htmx` `""` fallback timestamp, `HX-Reswap: none` handling, and `ARTICLE_DB_NAME` v3 cache guard all match [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/03-design.md). |
| Do not leak internals in errors | **Pass** | `400`/`403` return static user-facing strings; upstream errors go through `handle_htmx_error` without exposing raw GraphQL debug details (`test_handle_htmx_error_returns_clean_toast_without_debug_dump`). |

---

## Findings

None — zero Critical or Major findings.

*Non-blocking note:* In `mark_page_read_htmx` ([entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1574)), `ToastUndoPayload.status` passes `form.status.clone()` rather than `normalized_status.map(str::to_string)`; normal UI requests always send non-empty `#active-status` values, and `build_graphql_filter` falls back to `UNREAD` for empty strings, so no functional impact occurs.

---

## Reference Sources

- Axum `Form` & `serde_urlencoded`: <https://docs.rs/axum/latest/axum/extract/struct.Form.html> · <https://docs.rs/serde_urlencoded/latest/serde_urlencoded/>
- HTMX `HX-Reswap`, `hx-vals`, `hx-swap-oob`, `htmx:afterRequest`: <https://htmx.org/reference/#response_headers> · <https://htmx.org/attributes/hx-vals/> · <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/events/#htmx:afterRequest>
- `test_mutation_endpoints_reject_with_403_when_auth_disabled` ([e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs#L421-L430)) & `test_hx_mark_page_read_includes_undo_toast_and_mark_page_unread_restores_page` ([mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1669-L1786))
