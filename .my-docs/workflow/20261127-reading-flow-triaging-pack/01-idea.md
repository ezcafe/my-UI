# Idea — Reading Flow & Triaging Pack

## Summary & Shared Understanding (Draft)

You selected **Option A — Reading Flow & Triaging Pack** to make scanning, opening, reading, and clearing articles smoother in both **Magazine** and **List** views, across **Guest** and **Auth** modes.

Here is how I understand the four pieces together (guesses marked):

1. **Full-card click on Magazine view**: Clicking anywhere on a P1, P2, or P3 card in `#entries-magazine-view` opens the article (`openEntryFromMagazine`), instead of requiring a click directly on the title text. *(Guess: keep standard accessible card semantics / hover cursor across the whole card while preserving any nested links/actions if added later.)*
2. **Prev / Next buttons in Detail view**: Add Previous and Next article controls inside `#detail-panel` (`entry_detail.html`, loading skeleton, and Guest mode `renderGuestEntryDetail`) so you can step straight through articles without closing back to the list. *(Guess: steps through the currently visible articles on the active page in DOM order, disabling Prev on the first item and Next on the last item.)*
3. **`Feed · Time` metadata in list rows and detail header**: Display the source feed name and a compact time label (e.g., relative time such as `2h ago`) on list rows (`entry_list.html`, `entry_item.html`, Guest local list) and in the detail view header (`entry_detail.html` and Guest detail). *(Guess: `Feed · Relative time` without reading time unless you want reading time too; note that existing tests intentionally assert `entry_detail.html` omits `feed_title` and `<time>`, so those tests will be updated to match the new design.)*
4. **Undo toast on "Mark This Page as Read"**: When you click **Mark This Page as Read**, show a toast with an **Undo** button (accessible `role="status"`, pauses auto-dismiss on hover/focus) in both **Guest mode** (reverting `readSet` / IndexedDB and restoring the page) and **Auth mode**. *(Guess: Auth mode marks read immediately and Undo calls a server HTMX endpoint to mark those entry IDs unread again and reload the list, rather than delaying the server write.)*

## Goals

- Reduce friction when clicking Magazine cards and moving between consecutive articles in Detail view.
- Give immediate source and freshness context (`Feed · Time`) before and during reading.
- Make bulk page triaging safe with a reliable Undo window in both Guest and Auth modes.

## Non-Goals / Anti-Goals

- Redesigning the 3-column Magazine grid, filter bar, or search layout.
- Adding multi-select checkboxes or per-article swipe gestures.
- Changing database schemas (`Has DB: no`) or auth/PII handling (`Has auth/PII/schema: no`).

## Tentative Classification

- **Has UI (tentative):** `yes`
- **Has API (tentative):** `yes` *(if Auth-mode Undo adds a bulk-unread HTMX route, or if Prev/Next relies on client DOM; currently `unknown` on run card until confirmed)*
- **Has DB / auth/PII/schema:** `no` / `no`

## Open Question for User (Confirm or Skip)

- **Skip option:** You can reply **"just start"** if you want to skip brainstorming and jump straight into design.
- **Confirmation question:** Does the playback above match the reading flow you want — especially stepping through the current page's visible articles for Prev/Next, showing `Feed · Relative time`, and having Auth-mode Undo revert immediately via the server — or what would you adjust?
