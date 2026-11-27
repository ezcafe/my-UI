# Grill — List UI: Empty State Refresh, Light Theme Filter Active State, Pagination Brief, Mobile Search Toggle

**Result:** frontier-empty

All four requirements were confirmed by the user during ideation and verified against the codebase:
1. Empty unread state hides `nav[aria-label="Pagination"]` and adds a Refresh button below the empty message that reloads the current list/filter (`/hx/entries`) without full feed sync.
2. Light theme active state for `Today` and `All` (and `Starred`) matches `Unread` exactly by managing `hover:bg-border` in `setActiveFilter(btn)`.
3. Moving to another page hides the previous page's Executive Brief while loading/showing the new page's brief.
4. Mobile view collapses `#search-form` into an icon button (`#mobile-search-toggle-btn`); clicking it reveals `#search-form` above the filter buttons.

No open ambiguities remain.
