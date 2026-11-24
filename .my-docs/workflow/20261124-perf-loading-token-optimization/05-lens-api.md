# Lens: API Contract Review

- **Workflow Slug:** `20261124-perf-loading-token-optimization`
- **Lens:** api
- **Result:** clean

## Summary

The API changes for performance, loading, and token optimization are fully additive, type-safe, and backward-compatible. Route contracts for `GET /hx/entries`, `GET /hx/entries/{id}`, `GET /hx/entries/{id}/rewrite`, `GET /static/annuaire-theme.css`, and `POST /hx/entries/mark-page-read` preserve existing parameter contracts, status codes, and HTMX swap targets. The new rewrite endpoint gracefully serves cached rewrites in < 5ms, extracts and generates on-demand rewrites asynchronously, and falls back cleanly to executive briefs on failure.

---

## API Checklist

| Requirement | Status | Notes |
|-------------|--------|-------|
| **Typed input and output** | Pass | Strong Rust typing across `BatchAiItem`, `AiSummaryRecord`, `EntryDetailTemplate`, `MarkPageReadForm`, and query structs. |
| **Additive changes only** | Pass | `GET /hx/entries/{id}/rewrite` and `GET /static/annuaire-theme.css` are additive routes. `rewrite` field in data models is `Option<String>`, defaulting safely to `None`. |
| **Stable error shapes** | Pass | Errors handled gracefully; on-demand rewrite endpoint falls back to existing executive brief or teaser in `#ai-rewrite-box` without crashing or returning unhandled 500s. |
| **Edge validation** | Pass | Path parameter `id: i64` and query parameters (`status`, `search`, `offset`, `limit`) validated at HTTP system entry points. |
| **Lists paginated & bounded** | Pass | Batch prompt input text truncated to 130 words per article; batch candidate extraction bounded to page limit; `active-offset` hidden field maintains pagination state across mark-as-read. |
| **Mutating endpoints idempotent** | Pass | On-demand rewrite generation and persistence use idempotent SQLite upserts with `COALESCE`. Mark-page-read is idempotent. |
| **Design parity** | Pass | Implementation matches all contract specifications and sequence diagrams in `03-design.md`. |
| **Leak-free responses** | Pass | All dynamic strings in HTML templates and endpoints are sanitized with HTML entity escaping (`escape_html` and Askama auto-escaping). Zero internal paths or stack traces exposed. |

---

## Findings

- **No Critical or Major issues found.**
- **Contracts Verified**:
  - `GET /hx/entries/{id}/rewrite` (`entries::get_entry_rewrite_htmx`):
    - Reads cached rewrite from `AiDb` in < 5ms. If found, returns escaped `<div id="ai-rewrite-box">...</div>`.
    - If uncached: queries Miniflux for entry metadata, extracts content via `ArticleExtractor` (25-word teaser fast-path / 1.5s timeout), invokes `ai_service.generate_rewrite()`, persists result to `AiDb`, and returns rendered HTML fragment.
    - Fallback: On LLM error or timeout, renders executive brief or teaser safely inside `#ai-rewrite-box`.
  - `GET /static/annuaire-theme.css` (`pages::serve_annuaire_theme_css`):
    - Serves precompiled CSS with `Content-Type: text/css; charset=utf-8` and `Cache-Control: public, max-age=86400`.
  - `GET /hx/entries/{id}` (`entries::get_entry_detail_htmx`):
    - Renders instant drawer. If `record.rewrite` is present, renders direct rewrite; otherwise renders `#ai-rewrite-box` skeleton with `hx-get="/hx/entries/{id}/rewrite" hx-trigger="load" hx-swap="outerHTML"`.
  - `POST /hx/entries/mark-page-read` (`entries::mark_page_read_htmx`):
    - Accepts optional `offset` from form input, maintaining pagination and navigating to the next page smoothly.
- **Verification**: All 222 workspace tests pass cleanly (`cargo test --workspace`).
