# Grill: Counts, Paging, Read-Flex

Result: frontier-empty

## Frontier Rounds

### Round 1

❓ **Q1** - **Keep read item in list during unread filter view**:
When viewing an article detail under the "Unread" filter, auto-marking as read removes the blue unread indicator dot via out-of-band update, but keeps the item row visible in `#entries-list` until the user navigates or re-filters. Do you agree with keeping the row visible to prevent sudden scroll jumps?

➡️ **User decision**: Yes, keep the article row visible with only the unread indicator dot removed to preserve reading context and avoid layout jumping.

---

❓ **Q2** - **Pagination bar position**:
When articles exceed 1 page (> 50 items), where should the pagination navigation bar (`Previous`, `Page X of Y`, `Next`) be placed?

➡️ **User decision**: Alternative chosen — place at the bottom of the scrollable list (scrolls with entries, not pinned).

---

❓ **Q3** - **Inline auto-mark read on detail load**:
Should marking as read occur automatically within the `GET /hx/entries/{id}` request (returning both detail content and out-of-band updates for counts and unread indicator), rather than an extra roundtrip request?

➡️ **User decision**: Yes, mark as read directly during the detail fetch and return out-of-band updates for counts and indicator.
