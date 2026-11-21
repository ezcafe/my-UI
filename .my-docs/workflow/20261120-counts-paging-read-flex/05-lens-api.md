# Lens

**Lens:** api  
**Result:** clean

## API Contract Checklist

| Contract / Requirement | Status | Notes |
|---|---|---|
| Typed input and output; create-input vs full entity | Pass | `EntryResultSetGql` returns typed `total: i64` and `entries: Vec<EntryGql>`. `EntryFilterInput` strictly typed with enums and signed numeric limits/offsets. |
| One stable error shape; correct status codes | Pass | `handle_htmx_error` provides standard status codes (400, 404, 500) and toast error responses. GraphQL queries enforce boundary constraints (`limit < 0`, `offset < 0`). |
| Validate at system edges only (not re-validate trusted internals) | Pass | Inputs sanitized and clamped at edge route handlers (`limit.clamp(1, 100)`, `offset.max(0)`) and GraphQL resolver boundary. |
| Lists paginated; filters via query params; PATCH/batch for updates | Pass | `entriesResult` is fully paginated with `limit` and `offset`. All query filters (`status`, `search`, `limit`, `offset`) preserved in pagination navigation links. |
| Additive fields; no silent breaking changes | Pass | `entriesResult` and `todayCounter` added without altering or breaking existing `entries` query or `unreadCounter`. Backwards-compatible. |
| Naming consistent with the repo | Pass | `entriesResult`, `todayCounter`, `EntryResultSetGql` adhere to codebase async-graphql and Rust naming standards. |
| Mutating endpoints: idempotent or documented unsafe-to-retry | Pass | Entry auto-mark read (`update_entry_status` / `updateEntriesStatus`) is completely idempotent and only called when `is_unread` is true. OOB swaps are idempotent DOM replacements. |
| Contract in 03-design matches what code exposes (and vice versa) | Pass | All GraphQL types, resolvers, route parameters, and template swaps match `03-design.md` specifications verbatim. |
| Do not leak internals in errors | Pass | Errors mapped cleanly through `AppError` without exposing stack traces or sensitive internal states. |

## Findings

No Critical, Major, or Minor blockers found. All interface contracts adhere to best practices and project architecture.
