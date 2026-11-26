Result: clean

# TDD Plan Review — Fix Magazine View Flash & Detail Impacts

Reviewed `04-tasks.md` Planned Tests against `03-design.md`, `03a-design-review-log.md`, `crates/ui-components/src/templates.rs`, and `crates/ui-components/templates/pages/index.html`.

## Coverage Check

1. **Issue 1 — List-View Flash (`data-view-mode` + `htmx:beforeSwap`):**
   - Planned test verifies `#entries-list[data-view-mode="ai"]` / `[data-view-mode="list"]` CSS rules, `applyAiViewMode` `data-view-mode` updates, and `htmx:beforeSwap` pre-swap `data-view-mode` synchronization.
   - Implementation note from `03a-design-review-log.md`: ensure `htmx:beforeSwap` updates `data-view-mode` before the `pendingGuestMarkRead` early return so auth-mode swaps are covered.

2. **Issue 2 — Auth Skeleton (`openEntryDetail` + `openDetailWithSkeleton`):**
   - Planned test verifies `openEntryDetail` extracts `title`, `brief`, and `imageUrl` from `#mag-entry-` (with fallback) and passes `imageUrl` to `openDetailWithSkeleton`, which renders the hero image when present.

3. **Issue 2 — Guest Detail (`GuestEntryPayload` + `cacheLoadedArticlesFromDom` + `renderGuestEntryDetail`):**
   - Planned update to `test_index_template_guest_detail_skips_entry_fetch_and_omits_author_reading_time` verifies `#ai-analysis-box`, `/analysis`, `htmx.process(panel)`, `image_url`, and `summary`, while preserving omission of `author`, `readingTime`, `reading_time`, `/rewrite`, and `/hx/entries/${numId}` detail fetch.
   - Planned test verifies `guest_entries_json` serializes `summary` and `image_url`.

No missing critical test cases or edge cases blocking Gate B.
