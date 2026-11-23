# Analysis: AI Magazine View Toggle & Hot Topic Highlighting

**Result:** done

## Context
The application is an Annuaire-themed news directory built with Axum, Askama, Miniflux GraphQL client, SQLite AI cache, and HTMX.
Users currently view articles in a high-density, vertical list view with a pagination bar at the bottom.
The user requested an editorial "AI Magazine View" toggled by a new AI button in the header (placed after `#theme-toggle-btn`).
The AI view must:
1. Be the default view on first load.
2. Toggle between list and magazine view on button click.
3. Show item briefs of the current page in an editorial magazine layout.
4. Highlight the "hot topic", defined as the first topic (category name) of the source feed that the user clicks the most.
5. Display a photo for each news item if available.
6. Display the bottom pagination bar identical to the list page.
7. Open the slide-over detail page when clicking an article title in AI view.
8. Change clicked title color to grey (`text-muted-foreground`), indicating it has been read, while keeping it on the page.

## Risks
1. **Layout shift or flicker on initial load**: Defaulting to AI view must be instantaneous to avoid a flash of list view.
2. **Missing photos**: Many RSS articles do not have images. The magazine card layout must look clean and balanced whether an image is present or absent.
3. **Empty click history**: When the user first visits the app, no sources have been clicked yet. We must gracefully fall back to the category of the top feed on the current page as the hot topic.
4. **Read state synchronization**: When an article is read (in either view), its read status must reflect in both views and persist across tab/filter changes without disappearing from the magazine view.

## Open Questions
- *Resolved in ideation*: Hot topic is defined as the source feed's category name. Client-side tracking in `localStorage` tracks source click counts.
