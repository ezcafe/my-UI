# Lens: API Contract Review

- **Lens:** api
- **Result:** clean

## Summary

The API changes for the AI Rewrite batch pre-request feature are fully backward-compatible and additive. Route contracts for `GET /hx/entries`, `GET /hx/entries/{id}`, and `POST /hx/entries/mark-page-read` preserve existing parameter contracts, status codes, and HTMX swap targets. The detail endpoint gracefully serves precomputed rewrites or clean fallbacks in under 5ms without blocking requests.

---

## API Checklist

| Requirement | Status | Notes |
|-------------|--------|-------|
| **Typed input and output** | Pass | Strong Rust typing across `BatchAiResponse`, `BatchAiItem`, `AiSummaryRecord`, and `EntryDetailTemplate`. |
| **Additive changes only** | Pass | `rewrite` is an `Option<String>` in models and records, defaulting safely to `None`. No breaking changes to existing fields. |
| **Stable error shapes** | Pass | `ErrorCardTemplate` and OOB toast notifications maintain uniform structure; no stack traces or internal DB errors leaked. |
| **Edge validation** | Pass | Query parameters (`status`, `search`, `offset`, `limit`) and path parameters (`id: i64`) validated at system entry points. |
| **Lists paginated & bounded** | Pass | Batch extraction bounded to 20 uncached items per page; individual article text truncated to 300 words before LLM input. |
| **Mutating endpoints idempotent** | Pass | Mark-as-read and background prefetching are idempotent; duplicate calls do not corrupt state or overwrite existing rewrites. |
| **Design parity** | Pass | Implementation matches all contract specifications and sequence diagrams in `03-design.md`. |
| **Leak-free responses** | Pass | All user-facing strings and error messages are sanitized; no internal database paths or LLM CLI stderr exposed. |

---

## Findings

- **No Critical or Major issues found.**
- **Contracts Verified**:
  - `GET /hx/entries/{id}` (`entries::get_entry_detail_htmx`):
    - Retrieves `rewrite` from SQLite cache in `< 5ms`.
    - If `rewrite` is available, sets `is_rewrite: true` and renders the concise Inverted Pyramid rewrite as primary body.
    - If `rewrite` is missing (cache miss or pending background prefetch), falls back to `summary` / `teaser` with `is_rewrite: false`, displaying the executive brief card.
  - `POST /hx/entries/batch/summarize` & background prefetch (`prefetch::spawn_prefetch_job`):
    - Background task triggers single-call batch LLM prefetch for uncached page entries.
    - Captures `top`, `t` (suggested title), `b` (brief), and `r` (rewrite) in a unified JSON payload.
  - `POST /hx/entries/mark-page-read` (`entries::mark_page_read_htmx`):
    - Accepts optional `offset` from hidden `#active-offset` input, preserving pagination and enabling seamless transition to next page.
- **Verification**: All 201 workspace unit, integration, and template tests pass cleanly (`cargo test --workspace`).
