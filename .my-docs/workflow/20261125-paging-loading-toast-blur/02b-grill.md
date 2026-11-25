# Grill — Paging Loading, Mark-Page-Read Toast, and Lighter Read Blur

- **Slug:** `20261125-paging-loading-toast-blur`
- **Result:** `frontier-empty`

## Resolved Design Choices (Defaults Applied)

The user skipped ideation with `"Just start"` to proceed with sensible defaults. All requirements are self-contained in the codebase:

1. **Hidden-page loading state:** Use a centered spinner overlay (`#entries-loading-overlay`) inside `.workspace-layout` while hiding `#entries-list` content during paging (`Previous` / `Next`) and `Mark This Page as Read` requests. A centered spinner works uniformly for both List view and Magazine view without layout shift, while preserving `#entries-loading-bar` for existing template assertions.
2. **Read-item blur reduction:** Remove `blur(...)` (effectively `0px` blur) from `.ar-list-item.read` and `.magazine-card.read` in `crates/ui-components/templates/base.html`, keeping `opacity`, `grayscale(...)`, and `--muted-foreground` title color so read items remain clearly dimmed without sub-pixel text fuzziness.
3. **Toast message:** Always return `ToastTemplate` in `mark_page_read_htmx` (`"Page marked as read. You are on the last page."` on last page, `"Page marked as read."` otherwise), and call `showToast("Page marked as read.")` in guest mode `markVisiblePageLocallyRead()`.
4. **Hide detail panel:** Call `closeDetailPanel()` on `Previous`, `Next`, and `Mark This Page as Read` (both auth and guest).
