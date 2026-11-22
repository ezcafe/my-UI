# Codebase Skim: Batch Page AI Brief & Multilingual Fidelity

## Core Modules & Touchpoints

1. **AI Subsystem (`crates/server/src/ai/`)**:
   - `llm.rs`:
     - `format_batch_prompt`: Update prompt instructions to enforce language consistency (articles in Vietnamese output titles/briefs in Vietnamese; English in English).
     - `summarize_batch`: Handles parsing of `{"top":"...","items":[{"i":...,"t":"...","b":"..."}]}` with regex fallback.
   - `db.rs`:
     - Schema migration to version 3 adding `page_executive_briefs_v2` table with `page_key TEXT NOT NULL UNIQUE, brief TEXT NOT NULL, created_at TEXT NOT NULL`.
     - Methods: `get_page_brief_by_key(&self, page_key: &str) -> Option<String>` and `save_page_brief_with_key(&self, page_key: &str, brief: &str)`.
     - Maintains existing `get_batch_summaries` and `save_summary_and_title`.
   - `prefetch.rs` / `AiService`:
     - Handles batch processing for active page entries.
     - Saves page brief with page key and updates individual article summaries in `AiDb`.

2. **Routes & Endpoints (`crates/server/src/routes/entries.rs`)**:
   - `is_brief_eligible_filter`: Restrict eligibility strictly to `status == Some("unread")` or `None` (default unread). Explicitly disable on `today`, `read`, `starred`, `all`, and search.
   - `get_executive_brief_htmx`: Endpoint (`GET /hx/executive-brief`) that handles page executive brief lazy-loading. Returns the rendered brief card plus HTMX Out-Of-Band (`hx-swap-oob="outerHTML"`) updates for article titles on the page.
   - `render_entry_detail`: Remove on-demand `get_or_create_summary` invocation. Check `AiDb` directly; if absent, fall back to extractive teaser without spawning an LLM process.

3. **Templates & Components (`crates/ui-components/`)**:
   - `templates/partials/entry_list.html`:
     - When on `unread` filter and page brief is uncached, render `#page-executive-brief` loading skeleton with `hx-get="/hx/executive-brief?offset=...&limit=..." hx-trigger="load" hx-swap="outerHTML"`.
     - Omit executive brief completely when viewing `today`, `read`, `starred`, etc.
   - `templates/partials/entry_item.html`:
     - Wrap article titles with an explicit element ID (e.g. `id="entry-title-{{ entry.id }}"`) to enable HTMX Out-Of-Band swaps when the batch returns.
   - `templates/partials/brief_card.html`:
     - Clean presentation for article detail view using precomputed brief or fallback teaser.

## Existing Test Infrastructure

- Unit tests in `crates/server/src/ai/db.rs`, `llm.rs`, `prefetch.rs`.
- Integration tests in `crates/server/tests/`.

## Result

Result: done
