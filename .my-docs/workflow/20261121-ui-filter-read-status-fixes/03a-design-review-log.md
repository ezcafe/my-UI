Result: clean

# 03a-design-review-log: Guest Unread Scope, Local All Titles (Option B), and UI Filter Fixes

- **Stage:** design-verify-phase (Mode simple)
- **Review Profile:** lite
- **Surface Scope:** Has UI = yes | Has API = no (tiny-contract-tweak notes-skip) | Has DB = no (notes-skip) | Has auth/PII/schema = no

---

## 1. Executive Summary

The updated design in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-ui-filter-read-status-fixes/03-design.md) and tasks in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-ui-filter-read-status-fixes/04-tasks.md) cleanly integrate user-confirmed **Option B** (Guest unread scoped to today + local IndexedDB "All Titles") alongside the previous read status and filter fixes:

1. **Guest Mode Unread Scope**: In `crates/server/src/routes/entries.rs` and `pages.rs`, guest mode unread/default queries apply `publishedAfter: start_of_today` in GraphQL, and guest unread count maps to `todayCounter`.
2. **Local IndexedDB "All Titles" (Option B)**:
   - `#filter-all` is unconditionally visible to guest users, while `#filter-starred` remains auth-only.
   - Bump IndexedDB version to 2 with an `all_articles` object store (`keyPath: 'id'`).
   - Articles rendered into the DOM are cached into `all_articles`.
   - Clicking "All Titles" in guest mode intercepts the request, reads cached articles from `all_articles`, renders `.annuaire-row` items client-side, and re-applies `syncDomReadStates()`.
3. **Mark as Read on Browser Open**: A unified `markArticleRead(id)` helper handles optimistic DOM updates and local/remote persistence, wired to all external link anchors.
4. **Guest Unread Count Decoupling**: Base unread count is stored in `data-base-unread` and derived as `Math.max(0, baseServerUnread - readSet.size)`.
5. **Detail Panel Dismissal**: Filter button clicks and search submissions invoke `closeDetailPanel()`.

Review findings: zero Critical, zero Major, zero blocking issues. The design is **clean** and ready for TDD implementation.

---

## 2. Review Findings & Checklist

| Id | Severity | Surface | Finding / Recommendation | Status |
|----|----------|---------|--------------------------|--------|
| F-01 | Info | UI | **IndexedDB Versioning**: Version bump from 1 to 2 cleanly handles `onupgradeneeded` checking `db.objectStoreNames.contains(STORE_ARTICLES)`. Existing `read_entries` records are fully preserved. | Verified |
| F-02 | Info | UI | **Fallback & Icon Re-hydration**: `renderLocalAllArticles()` invokes `lucide.createIcons()` and `syncDomReadStates()` after DOM injection to ensure icons render properly and read indicators match local state. | Verified |
| F-03 | Info | API | **Has API = no / tiny-contract-tweak**: Route query parameter tweak for `publishedAfter` is internal; isolated review skipped per Mode simple rules. | Skipped |
| F-04 | Info | DB | **Has DB = no**: Server DB is untouched; client-side IndexedDB is UI state storage. | Skipped |
| F-05 | Info | TDD | **Test Suite Alignment**: [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-ui-filter-read-status-fixes/04-tasks.md) details 6 Red-Green-Verify tasks with comprehensive unit and template coverage. | Verified |

---

## 3. Surface Verification

### UI / Templates
- **Template conditionals**: Correctly renders `#filter-all` unconditionally while retaining `{% if auth_enabled %}` on `#filter-starred`.
- **Event interception**: In guest mode, `#filter-all` prevents default HTMX request and renders from IndexedDB without network latency.
- **Event propagation**: Preserves `event.stopPropagation()` on browser-open links to avoid triggering row detail panel expansion.
- **Empty state logic**: Renders a clean `#client-empty-state` in `#entries-list` if no articles have been cached in `all_articles` yet.

### API & Data Model Surfaces
- **Has API = no (tiny-contract-tweak)**: Notes-skip applied. Internal route GraphQL query builder change only.
- **Has DB = no**: Notes-skip applied. Server database remains unchanged.

---

## 4. Conclusion

- **Result:** `clean`
- **Gate B Readiness:** Ready for Gate B settlement.
