# Grill — Reading Flow & Triaging Pack

**Result:** frontier-empty

## Design Tree & Settled Decisions

All product and UX branches were confirmed by the user in [01-idea.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261127-reading-flow-triaging-pack/01-idea.md) (`Notes: ideation-confirmed`), leaving zero open questions for the user:

1. **Full-card click on Magazine view**
   - **Settled:** Clicking anywhere on a P1, P2, or P3 `.magazine-card` inside `#entries-magazine-view` triggers `openEntryFromMagazine`. Use accessible stretched-link card semantics (`cursor-pointer` + `position: relative` card + heading anchor `::after` covering the card, preserving `z-10` isolation for any nested interactive elements).
2. **Prev / Next buttons in Detail view**
   - **Settled:** Add Prev (`#detail-prev-btn`) and Next (`#detail-next-btn`) controls to the `#detail-panel` top action bar (`entry_detail.html`, `openDetailWithSkeleton`, and `renderGuestEntryDetail`), stepping through the current page's articles in DOM order and disabling `Prev` on the first item and `Next` on the last item. Maintain the active page's ordered article IDs across `syncDomReadStates()` row hiding in Guest Unread mode and dynamically update button disabled states on cache hits.
3. **`Feed · Time` metadata in list rows and detail header**
   - **Settled:** Show `Feed · Relative time` (e.g., `Feed · 2h ago`) without reading time or author on list rows ([entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html), [entry_item.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_item.html), Guest `renderLocalAllArticles()`) and in detail view headers ([entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html) hero & standard headers, Guest `renderGuestEntryDetail()`). Keep Magazine cards free of source/time metadata and update legacy tests that asserted metadata omission.
4. **Undo toast on "Mark This Page as Read"**
   - **Settled:** Show an accessible `role="status"` toast with an **Undo** button that pauses auto-dismiss on hover/focus in both Guest and Auth modes. Auth mode marks read immediately via `POST /hx/mark-page-read` and wires **Undo** to a new batch endpoint (`POST /hx/mark-page-unread`) to restore the marked entries and reload `#entries-list`; Guest mode reverts only newly marked IDs in `readSet` / IndexedDB and restores the previous page.

## Round log

### Round 1

No open ambiguities remain after ideation confirmation and codebase inspection.
