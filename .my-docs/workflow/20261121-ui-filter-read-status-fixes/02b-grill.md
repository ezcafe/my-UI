# 02b-grill: Grill Rounds & Frontier Status

- **Result:** frontier-empty

## 1. Goal

Stress-test assumptions, verify architecture choices, and confirm all user decisions before implementation.

## 2. Review of Questions & Invariants

1. **User Decision on Guest Mode "All Titles"**:
   - The user explicitly selected **Option B**: Restore `#filter-all` in guest mode, storing all loaded articles into local IndexedDB (`all_articles` in `my_news_guest_db`), and rendering "All Titles" completely locally from IndexedDB.
   - For guest unread scope: "Unread" in guest mode displays today's unread news only (`publishedAfter: start_of_today` on backend + client `readSet` filtering).

2. **Detail View Panel "Original" Link**:
   - Opening the detail view panel already auto-marks the article as read.
   - Marking read is completely idempotent; wiring `markArticleRead(id)` to the "Original" button is safe and ensures consistent behavior.

## 3. Frontier Status

- **Guest Unread Scope**: Settled. Backend queries filter by `publishedAfter: start_of_today` when `!auth_enabled` and status is unread or default.
- **Guest All Titles**: Settled. Option B selected (local IndexedDB `all_articles` store + client rendering).
- **Filter Visibility**: Settled. `#filter-starred` is hidden when `!auth_enabled`; `#filter-all` is displayed.
- **External Link Mark-Read**: Settled. Call unified `markArticleRead(id)` in `entry_item.html`, `entry_list.html`, and `entry_detail.html`.
- **Guest Unread Decoupling**: Settled. Derive global unread as `Math.max(0, baseServerUnread - readSet.size)`.
- **Detail Panel Dismissal**: Settled. Filter clicks and search close the detail panel.

No unresolved frontier decisions remain.
