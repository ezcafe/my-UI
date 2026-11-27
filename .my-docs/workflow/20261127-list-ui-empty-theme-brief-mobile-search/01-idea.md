# Idea — List UI: Empty State Refresh, Light Theme Filter Active State, Pagination Brief, Mobile Search Toggle

- **Has UI (tentative):** yes
- **Has API / DB / auth (tentative):** no | no | no
- **Size:** small UI refinement across the list page (`index.html`, `entry_list.html`, and list-view/pagination handlers)

## Shared Understanding (Goals)

1. **Empty unread state — hide paging bar & add Refresh button:**
   - When there are no unread items (both server-rendered empty state and client-side empty state after marking items read), hide the pagination bar (`nav[aria-label="Pagination"]`).
   - Show a **Refresh** button directly below the empty message ("All caught up! / No articles match your current filter.") so the user can reload entries without using the top header refresh button.

2. **Light theme — consistent active state for `Today` / `All` / `Unread` filter buttons:**
   - On light theme, the active state of `Today` and `All` currently looks different from `Unread`. Make all filter pills (`Unread`, `Today`, `Starred`, `All`) use the exact same active visual styling.

3. **List page pagination — hide the previous page's Executive Brief:**
   - When moving to the next page (e.g. `offset > 0` or clicking Next/Previous), do not keep or display the Executive Brief from the previous page.

4. **Mobile view — collapsible Search icon button above filter buttons:**
   - On mobile screens, collapse the Search input (`#search-form`) into a compact search icon button.
   - Clicking the search icon button reveals the search input box **above** the filter buttons row (clicking again or closing hides it).

## Non-Goals

- No backend database schema changes, authentication changes, or new API endpoints.
- No changes to desktop layout for the Search bar (remains inline on wider viewports).
- No changes to the article detail panel or AI analysis cards.

## Constraints

- Preserve existing HTMX + Askama + guest-mode (`renderLocalAllArticles` / client-side empty state) behavior.
- Use plain CSS / container queries or existing Tailwind utility classes consistent with `ANNUAIRE_STYLE_GUIDE.md` and `base.html`.

## Open Question for Confirmation

Before moving into design, please confirm this shared understanding (or reply "just start" to skip ideation), and clarify one quick detail:

- **On page 2+ (when moving to the next page):** Should the Executive Brief be **hidden completely** on pages after page 1, or should it **hide the previous page's brief while loading/showing a brief for the new page**?
  *(Also, for the Refresh button on the empty state, should it trigger the same refresh action as the top header Refresh button?)*
