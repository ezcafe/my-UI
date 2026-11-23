# Idea / Framing: AI Magazine View Toggle & Hot Topic Highlighting

## Intent
Add an AI toggle button in the header (after the theme button) that switches between the standard article list and an editorial "AI Magazine View" (which is active by default on initial page load). The magazine view displays item briefs for the current page with responsive card layouts, photos (when available), hot topic badges, and pagination matching the list view.

## Goals
- **Header AI Toggle**: Place an AI toggle button immediately after `theme-toggle-btn`. Clicking toggles between the AI view and list view, persisting state in `localStorage` with AI view as default on first visit.
- **Magazine View Layout**: Render item briefs for current page entries in an editorial magazine layout (responsive grid).
- **Hot Topic Highlighting**: Identify the news source (feed) with the highest user click count (tracked client-side), and highlight its category name (e.g. "Tech", "News") as the "Hot Topic" badge. If no clicks yet, defaults to the category of the top feed on the page.
- **Article Photos**: Display extracted hero images/photos from article content or enclosures within the magazine cards when present.
- **Detail Integration & Read States**: Clicking an article title in AI view opens the slide-over detail panel (identical to list view). The clicked title changes color to grey (`text-muted-foreground`) to indicate read status, while remaining visible on the page.
- **Consistent Pagination**: Provide the same bottom pagination controls (Previous / Page X of Y / Next) in sync with the current page number.

## Non-goals
- Server-side database schema changes or user tracking tables (source clicks and view preferences are stored client-side via `localStorage` or `IndexedDB`).
- Full RSS crawler for external photos beyond what is in the article payload / content HTML.
- Modifying the existing detail panel markup or backend auth workflows.

## Anti-goals
- Hardcoded screen breakpoints (use container queries and CSS auto-fit minmax per Annuaire design guidelines).
- Cluttering the clean Annuaire black/white aesthetic with loud colors (hot topic follows Annuaire brand/red positional accent rules).
- Removing or hiding read articles from the magazine page upon clicking.

## Constraints
- **Design System**: Annuaire style guide rules: no page box shadows, tabular numbers, red marks position/badge only, ink pill buttons.
- **Performance**: Instant client-side switching between list and AI views without redundant network refetches where possible.
- **Fallback**: Graceful fallback when an article has no photo or when click history is empty (default to top feed).

## Has UI (tentative)
yes

## Shared Understanding
- [x] User confirmed (brainstorm hard-gate) — user confirmed: "use source feed's category name"
