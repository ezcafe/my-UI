# Grill: Unread Count, Brief Cache, Title Sentence Restriction & Brief Throttle

## Settled Decisions & Architectural Alignment

### 1. Unread Filter Fallback to Total (Settled)
- **Decision**: In `pages.rs` and `entries.rs`, if `unread_count` is 0 or null and `total > 0` for unread items, fall back to `total as i32`. In `index.html` (`syncDomReadStates()`), if `effectiveUnread === 0` while `visibleUnreadRowCount > 0` on the Unread filter, fall back to `visibleUnreadRowCount`.
- **User Confirmation**: Approved in Ideation (`user: "proceed"`).

### 2. Executive Brief Cache Lifecycle & HTMX Re-Processing (Settled)
- **Decision**: Update `filterCache` whenever `#page-executive-brief` completes its swap in `#entries-list`. Call `htmx.process(entriesList)` in `applyCachedList()`.
- **Behavior**:
  - `htmx:afterSwap` catches inner swaps on `#page-executive-brief` and saves the rendered brief into `filterCache.get(status)`.
  - When switching back to Unread, `applyCachedList()` re-inserts the rendered brief. If any unresolved triggers exist, `htmx.process(entriesList)` binds them.
- **User Confirmation**: Approved in Ideation (`user: "proceed"`).

### 3. Title Constraint strictly in One Sentence (Settled)
- **Decision**: Change prompt instruction in `crates/server/src/ai/llm.rs` line 78 from `<=10 words` to `strictly in one sentence`.
- **Behavior**: Removes word cap while strictly enforcing single-sentence structure for suggested titles.
- **User Confirmation**: Approved in Ideation (`user: "proceed"`).

### 4. Executive Brief Throttle (10 Read Items Threshold) (Settled)
- **Decision**: Track read articles since last brief using a set/counter. Pass `request_brief=false` when clicking the Unread filter if read count is < 10. Pass `request_brief=true` and reset counter if >= 10.
- **Behavior**:
  - Query parameter: `GET /hx/entries?status=unread&request_brief=false`.
  - Server renders `is_brief_eligible = false` and serves existing `latest_page_brief` as static `top_executive_brief`.
  - When >= 10, server renders `is_brief_eligible = true` with `hx-trigger="load"` to generate a fresh brief.
- **User Confirmation**: Approved in Ideation (`user: "proceed"`).

---

## Frontier Status

All technical prerequisites and architectural boundaries are resolved and approved. No open frontier questions remain.

## Result

Result: frontier-empty
