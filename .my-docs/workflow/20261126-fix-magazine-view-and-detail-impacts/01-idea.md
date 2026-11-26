# Idea & Framing — Fix Magazine View Flash & Detail Impacts

- **Size:** Small change (2 targeted UI/HTMX bugfixes)
- **Has UI (tentative):** yes
- **Has API (tentative):** unknown (may stay `no` if fixed purely in templates/JS, or `yes` if `/hx/mark-page-read` / `/hx/entries/{id}` handlers need updates)

## Shared Understanding (Playback)

We are fixing two regressions in the My News reader UI (`crates/ui-components/templates/pages/index.html`, `entry_list.html`, `entry_detail.html`, and related server routes if needed):

1. **Magazine view flash after "Mark Page as Read":**
   - When you click **Mark This Page as Read** while in Magazine view, the next page loads into `#entries-list` with `#entries-list-view` visible and `#entries-magazine-view` hidden by default (`entry_list.html`).
   - Only after the swap completes does JavaScript (`applyAiViewMode`) hide the list view and unhide the magazine view, causing a visible flash of list view before switching to magazine view.
   - **Goal:** Keep Magazine view active immediately when the next page renders, with zero list-view flash.

2. **Detail view photo + title + rewrite from list page & LLM impacts request:**
   - When clicking a news item, the detail panel should immediately show the **photo + title + rewrite** taken from the list/magazine page, and request **LLM impacts** (`#ai-analysis-box` / `/hx/entries/{id}/analysis` or equivalent).
   - Currently in `index.html`, `openEntryDetail` only reads `.magazine-brief` from `#entry-{id}` (the list row, not `#mag-entry-{id}`), doesn't pass the photo to `openDetailWithSkeleton`, and in guest mode (`!AUTH_ENABLED`) short-circuits to `openGuestEntryDetail` which renders locally without requesting LLM impacts. (We will also check `get_entry_detail_htmx` and `entry_detail.html` to see what changed.)
   - **Goal:** Restore immediate photo + title + rewrite display from the list/magazine item and ensure LLM impacts are requested properly.

### Non-goals & Constraints
- **Non-goals:** No redesign of the magazine layout, list layout, or detail panel styling.
- **Constraints:** Preserve both auth-enabled and guest-mode behavior where applicable, and keep existing tests passing.

---

## Open Question for User

*(You can say **"just start"** if this looks right and you want me to proceed straight to design & fix.)*

When you click a news item and see the photo + title + rewrite missing and LLM impacts not requested, are you running with **login/auth enabled** or in **guest mode** (and are you clicking from **Magazine view** or **List view**)?
