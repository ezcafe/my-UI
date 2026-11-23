# Idea framing

## Goal
Refactor the AI view (`#entries-magazine-view`) into a 3-column editorial layout matching the provided screenshot:
1. **Left column (Executive Brief)**: Editorial sidebar styled like the screenshot's SPOTLIGHT box, displaying the AI-generated Executive Brief as a single narrative paragraph with appropriate header/status.
2. **Center column (Priority 1 + Priority 2)**:
   - **Priority 1 (Hero)**: Lead story with image on left, title + source/date on right, summary paragraph below.
   - **Priority 2 (Featured Row)**: 3 featured articles side-by-side with aspect-ratio thumbnail on top, category tag, title, and source/date.
3. **Right column (Priority 3)**:
   - Vertical stack of secondary editorial cards (priority 3 items) with category tag, title, and compact brief or metadata.
4. **Fluid Responsiveness**:
   - Built using container queries and CSS grid (`repeat`, `auto-fit`, `minmax`) rather than rigid hardcoded breakpoints, adhering to user rules.

## Non-goals
- No backend API / GraphQL schema changes.
- No database migrations or persistent schema changes.
- No changes to authentication or external Miniflux sync logic.

## Constraints
- Must preserve HTMX dynamic loading (`#page-executive-brief` hx-get swap).
- Must preserve click interactions: `openEntryFromMagazine(id, feedTitle)`, `markArticleRead`.
- Must preserve unread styling, suggested AI titles, and category badges.
- Strict compliance with user rules: no hardcoded breakpoints, use modern CSS features (`repeat`, `auto-fit`, `minmax`, container queries).
