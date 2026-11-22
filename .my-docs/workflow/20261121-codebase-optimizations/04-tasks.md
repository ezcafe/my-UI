# Tasks: Codebase Performance, Concurrency, and Quality Optimizations

Result: done

## Tasks Breakdown

### Phase 1: In-Memory Cache & Rate Limiter Bounds (`crates/server`)
- [x] Task 1.1: Refactor `crates/server/src/cache.rs` to encapsulate the 3-element tuple into a named `CacheEntry` struct with `is_expired(&self) -> bool`.
- [x] Task 1.2: Add `MAX_CACHE_CAPACITY = 1000` to `crates/server/src/cache.rs` and implement inline eviction of expired items and oldest items on insert.
- [x] Task 1.3: Update `crates/server/src/rate_limit.rs` to validate `capacity > 0.0` and `refill_per_sec > 0.0` with safe fallbacks.
- [x] Task 1.4: Implement stale bucket eviction in `crates/server/src/rate_limit.rs` for client IPs inactive for > 10 minutes.

### Phase 2: Static Regex & Selector Precompilation (`LazyLock`)
- [x] Task 2.1: In `crates/ui-components/src/sanitizer.rs`, precompile `RE_ANCHOR` and `RE_TARGET` using `std::sync::LazyLock`.
- [x] Task 2.2: In `crates/server/src/ai/extractor.rs`, precompile `ARTICLE_SELECTOR` and `BODY_SELECTOR` using `std::sync::LazyLock`.

### Phase 3: Clippy Warnings & SQLite PRAGMA Optimization (`crates/server/src/ai`)
- [x] Task 3.1: In `crates/server/src/ai/llm.rs`, replace manual semaphore permit match with `.ok()`.
- [x] Task 3.2: In `crates/server/src/ai/db.rs`, execute `PRAGMA journal_mode = WAL;` in `init_schema()`, and remove it from `SqliteCustomizer::on_acquire`.

### Phase 4: HTML Sentence Extraction Cleanup (`crates/miniflux`)
- [x] Task 4.1: In `crates/miniflux/src/summary.rs`, implement `strip_html_tags(content: &str) -> String` to strip tags and normalize whitespace.
- [x] Task 4.2: In `crates/miniflux/src/summary.rs`, pass stripped content into unicode sentence segmentation.

### Phase 5: Frontend Template & UI Fixes (`crates/ui-components/templates`)
- [x] Task 5.1: In `crates/ui-components/templates/base.html`, scope `htmx:afterSwap` icon creation to `evt.detail.target`.
- [x] Task 5.2: In `crates/ui-components/templates/pages/index.html`, scope inline Lucide icon updates to target container elements.
- [x] Task 5.3: In `crates/ui-components/templates/partials/entry_list.html` and `index.html`, ensure pagination is hidden when there is no news or when client-side filter yields 0 matches.
- [x] Task 5.4: In `crates/ui-components/templates/pages/index.html`, remove the unread counting pill next to the app title in the header.
- [x] Task 5.5: In `crates/ui-components/templates/base.html`, fix `.htmx-indicator` using `display: none` / `display: block` so Tailwind's `animate-pulse` keyframe does not override idle opacity.

### Phase 6: Automated Testing & Verification
- [x] Task 6.1: Run unit tests across `miniflux-client`, `ui-components`, and `server`.
- [x] Task 6.2: Run `cargo clippy --workspace --all-targets` to verify clean linter output.
- [x] Task 6.3: Run full workspace test suite `cargo test --workspace`.

---

## Planned tests

1. `test_miniflux_cache_bounded_capacity`:
   - Validates that inserting more than 1,000 entries into `MinifluxCache` maintains capacity at or below 1,000 without unbounded memory growth.
2. `test_miniflux_cache_inline_eviction_of_expired`:
   - Validates that when inserting into a full cache, expired entries are evicted first to make space for new items.
3. `test_rate_limiter_parameter_validation_defaults`:
   - Validates that `RateLimiter::new` with negative or zero parameters gracefully falls back to positive default values.
4. `test_rate_limiter_stale_bucket_pruning`:
   - Validates that client buckets with inactive timestamps (> 10 minutes) are automatically pruned from the bucket map during rate checks.
5. `test_sanitizer_preserves_links_and_attributes_with_lazylock`:
   - Validates that `sanitize_html()` with static `LazyLock` regexes accurately adds `target="_blank"` and `rel="noopener noreferrer"` to anchor tags.
6. `test_extractor_parses_content_with_lazylock_selectors`:
   - Validates that `ArticleExtractor::parse_html` correctly extracts article and body text using static `LazyLock` selectors.
7. `test_summary_extractor_strips_html_tags`:
   - Validates that `extract_summary` removes inline HTML markup (e.g. `<p>Hello <b>World</b></p>`) before sentence segmentation, producing clean text sentences.
8. `test_sqlite_wal_mode_active`:
   - Validates that a connection acquired from `AiDb` has `journal_mode = WAL` active.
9. `test_entry_list_hides_pagination_when_no_news`:
   - Validates that when `entries` is empty, no pagination element is rendered in `entry_list.html`.
10. `test_index_template_omits_title_unread_counting`:
    - Validates that the header title area in `index.html` does not display the unread counter next to the app title.
11. `test_htmx_indicator_hidden_by_default`:
    - Validates that `.htmx-indicator` has `display: none` in CSS to prevent `animate-pulse` keyframe bleed-through.
