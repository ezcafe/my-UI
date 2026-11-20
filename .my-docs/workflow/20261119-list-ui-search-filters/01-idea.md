# Idea / framing

## Intent
Transform the article list page into a clean title-only list adhering to `ar-list.html` AR spatial styling, provide an external link open button on each row, introduce a "Today" filter button, and fix the broken Search and Filter functionality.

## Goals
1. **Title-Only List Row**: Replace the current multi-line snippet/feed/meta card in `entry_item.html` with a concise, single/two-line title row matching `ar-list.html`.
2. **External Link Icon Button**: Add an icon button on the right side of each list row (`target="_blank" rel="noopener noreferrer"`) to open the original article URL directly in a new browser tab.
3. **Today Filter**: Add a "Today" filter button right after "Unread" in the filter toolbar (`All Titles`, `Unread`, `Today`, `Starred`).
4. **Fix Search**: Ensure typing into the search input properly triggers filtered search results through HTMX and Miniflux.
5. **Fix Filter**: Ensure clicking filter tabs updates the query parameter (`status` / `starred` / `after`), passes it through GraphQL/Miniflux client, and visually updates the active filter pill.

## Non-goals
- Modifying reader/article detail view logic beyond what is needed to navigate cleanly.
- Adding full offline caching or persistence beyond Miniflux backend.

## Anti-goals
- Breaking existing OIDC auth or Miniflux integration.
- Introducing heavy JS frameworks or breaking HTMX-driven hypermedia workflow.

## Constraints
- Visual design must match `ar-list.html` spatial glassmorphism style (dark glassmorphism, blur, Lucide icons, smooth hover states).
- Never read or parse `.env`.
- External links must use `target="_blank" rel="noopener noreferrer"`.
- Must follow TDD: failing tests before code implementation.

## Has UI (tentative)
- Yes: `entry_item.html`, `index.html`.

## Shared understanding
- [x] User confirmed (brainstorm hard-gate): `"Title-Only List Row" -> also add an icon button, on the right side, click on it will open the original link in external browser`
