Result: clean

# Design Review Log: Unread Count, Brief Infinite Loading, Title Sentence Restriction, and Brief Throttle Fixes

**Stage:** `design-verify-phase` (Mode simple)  
**Result:** `clean`  
**Date:** 2026-11-23  

---

## 1. Executive Summary

This design verification evaluated the architecture, state management, UI lifecycles, and test plans documented in:
- [00-run.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-unread-filter-brief-title-fixes/00-run.md)
- [01-idea.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-unread-filter-brief-title-fixes/01-idea.md)
- [02-analysis.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-unread-filter-brief-title-fixes/02-analysis.md)
- [02b-grill.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-unread-filter-brief-title-fixes/02b-grill.md)
- [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-unread-filter-brief-title-fixes/03-design.md)
- [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261123-unread-filter-brief-title-fixes/04-tasks.md)

### Isolated Review Status
- **API Contract Review**: `skipped` (Has API = no; internal query parameter `request_brief` within existing route).
- **Database Design Review**: `skipped` (Has DB = no; no schema migrations or new tables).

### Overall Assessment
- **Overall Verdict:** **`clean`** (Zero Critical, zero Major blockers).
- The proposed solution cleanly and defensively addresses all four issues without introducing state desynchronization, unnecessary backend statefulness, or breaking changes to existing data flows.

---

## 2. Detailed Technical Review by Domain

### 2.1 Unread Count Fallback (`pages.rs`, `entries.rs`, `index.html`)
- **Server Fallback Safety**:
  - In `pages.rs` and `entries.rs`, if the upstream Miniflux or guest counter yields 0 but `total > 0` for unread items, defaulting to `total as i32` guarantees that initial SSR HTML accurately presents the badge matching the visible rows.
  - Edge case handling: When `total == 0`, `unread_count` legitimately evaluates to 0, preventing phantom badge counts.
- **Client DOM Re-synchronization**:
  - In `syncDomReadStates()` (`index.html`), falling back to `visibleUnreadRowCount` when `effectiveUnread === 0 && visibleUnreadRowCount > 0` on the Unread filter prevents the badge from displaying `(0)` when unread rows are actively visible on screen.

### 2.2 Brief Cache Updates & HTMX Processing (`index.html`)
- **Swap Event Interception**:
  - Extending `htmx:afterSwap` to recognize swaps on `#page-executive-brief` ensures that when background brief generation finishes, the completed brief card is stored into `filterCache` immediately instead of keeping the stale skeleton cached.
  - The check safely isolates list updates and ignores detail panel swaps (`#detail-panel`), avoiding cache pollution from article reads.
- **Dynamic Trigger Re-activation**:
  - Invoking `htmx.process(entriesList)` in `applyCachedList()` resolves the infinite loading bug: any newly restored DOM fragment with dynamic HTMX triggers (such as `hx-trigger="load"`) is properly registered and processed by HTMX.

### 2.3 Title Prompt Sentence Constraint (`llm.rs`)
- **Prompt Precision**:
  - Changing `<=10 words` to `strictly in one sentence` cleanly removes artificial truncation of informative headlines while preserving concise, punchy presentation.
  - Critical invariant preserved: The multilingual rule ("exact same language as source text") and the compact JSON schema remain strictly enforced.

### 2.4 Executive Brief Throttling (`entries.rs`, `index.html`)
- **Stateless Backend Design**:
  - Utilizing `request_brief: Option<bool>` as an HTTP query parameter avoids fragile server session state, cookies, or distributed cache dependencies.
- **Set-Based Client Deduplication**:
  - Tracking read items via `Set` in JavaScript ensures repeated clicks or detail panel re-openings of the same article do not inflate the throttle count.
- **Fallback to Stored Brief**:
  - When `request_brief` is false (read count < 10), serving `top_executive_brief` from SQLite gives users instant contextual information without incurring LLM token costs or latency.

### 2.5 TDD & Implementation Verification (`04-tasks.md`)
- **Test-First Ordering**:
  - Unit tests for prompt constraints, query deserialization, unread counter fallback, and template rendering are all staged before production code modifications.
- **Verification Gates**:
  - Includes full `cargo test --workspace` and `cargo clippy --workspace --all-targets` checks to ensure zero compiler warnings or regressions.

---

## 3. Findings & Recommendations Summary

| # | Domain | Finding | Severity | Recommendation |
|---|--------|---------|----------|----------------|
| 1 | Client State | Full page refresh resets `readSinceLastBriefSet` | Low (Advisory) | Desired behavior: fresh page loads should generate a fresh brief if eligible. |
| 2 | DOM State | Fallback unread count during active search query | Low (Advisory) | Ensure `syncDomReadStates` only falls back to visible row count when `status == 'unread'` and no search filter is active. Handled cleanly in design. |
| 3 | Accessibility | Static brief container announcement | Low (Advisory) | Keep `id="page-executive-brief"` consistent across dynamic skeleton and static brief for layout stability. Handled cleanly in design. |

**Blockers:** 0  
**Critical:** 0  
**Major:** 0  
**Minor:** 3 (all advisory / non-blocking)

---

## 4. Conclusion & Next Steps

The design is sound, robust, and ready for implementation.

- **Overall Result:** `clean`
- **Next Stage:** Gate B settlement / TDD Build (`my-dev-flow-v3-code`).
