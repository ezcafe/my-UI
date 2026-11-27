# Ideation — Details Page Content Filter

- **Has UI (tentative):** yes (`API: no | DB: no | auth/PII/schema: no`)
- **Size:** small change (UI templates, client-side detail rendering, and detail route data)

## Current Playback & Guesses

Today, opening an article from the list opens `#detail-panel` in two different ways depending on mode:

1. **Guest mode (`!auth_enabled` in `index.html` + `templates.rs`):** The list embeds `GuestEntryPayload` JSON (which includes `sanitized_content` from the original article HTML and only one `image_url`), and `renderGuestEntryDetail()` displays the hero photo, feed title + date, title, brief/summary, the **full original HTML content** (`#guest-detail-content`), and lazy-loads `Impact Analysis` + `Logical Fallacy & Objectivity Check`.
2. **Authenticated mode (`auth_enabled` in `entries.rs` + `entry_detail.html`):** `GET /hx/entries/{id}` extracts `hero_image` + `body_images` from the original HTML, loads cached `suggested_title` and `rewrite` from AI DB (falling back to the raw Miniflux `summary`/`content` teaser if no AI rewrite exists yet), and renders/lazy-loads `Impact Analysis` and `Logical Fallacy & Objectivity Check`.

**Working interpretation (guesses marked):**
- **Photos from original content:** Keep extracting photos (`hero_image` + `body_images`) from the original article HTML in both authenticated and guest modes, without showing any original article text.
- **Title & rewrite content passed from list page:** Show the title and rewrite content as prepared/displayed on the list page (instead of re-falling back to original article `content` or `summary` when missing — *guess: needs confirmation*).
- **Impact Analysis and Logical Fallacy & Objectivity Check:** Keep both cards (rendered immediately if cached, or lazy-loaded via `/hx/entries/{id}/analysis`).
- **Do not show original content:** Remove `#guest-detail-content` (`sanitized_content`) in guest mode and stop falling back to raw Miniflux `summary`/`content` in the rewrite slot.
- **Navigation / controls (*guess*):** Keep the top action bar (`Back`, `Original` external link, read/bookmark toggles) so the user can close the panel or open the source article if needed, while removing feed/date/other extra text if desired.

## Framing

- **Goals:**
  - Restrict the details page (`#detail-panel`) in both authenticated and guest modes to show only: photos from original content, title passed from the list page, rewrite content passed from the list page, and Impact Analysis + Logical Fallacy & Objectivity Check.
  - Never display original article text/HTML body in the details page.
- **Non-goals:**
  - Changing the list page layout or adding new AI analysis fields/DB columns.
  - Modifying Miniflux GraphQL schema or authentication flows.
- **Anti-goals:**
  - Leaking raw HTML or original article teaser text inside `#ai-rewrite-box` or `#guest-detail-content` as a fallback.
  - Breaking guest mode offline/local detail viewing or photo extraction.
- **Constraints:**
  - Must apply consistently to both `entry_detail.html` (authenticated HTMX mode) and `renderGuestEntryDetail` / `openDetailWithSkeleton` in `index.html` (guest mode), plus `GuestEntryPayload` in `templates.rs` and `get_entry_detail_htmx` in `entries.rs`.

## Open Question for User

*(You can reply "just start" to skip further ideation questions, or describe your intent below.)*

When you click an article from the list page (whether in Magazine view, List view, or before AI finishes generating a rewrite), what should appear in the details panel—specifically, should the top bar buttons (`Back`, `Original` link, read/bookmark) and source/date stay or be removed, and what should show if the list page didn't have a rewrite ready yet?
