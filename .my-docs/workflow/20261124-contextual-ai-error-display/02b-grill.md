# Grill: Design Tree & Frontier

## Result: frontier-empty

## Decision Frontier & Settled Positions

---

### ❓ **Q1** - **Contextual Error Card Visual Presentation & Structure**
How should the inline error card be structured visually across the different target containers (`#page-executive-brief`, `#ai-analysis-box`, `#ai-rewrite-box`)?
- Option A: Minimalist one-line alert with an inline retry link.
- Option B (Recommended): Styled contextual card using container query / flex layout (`rounded-lg p-4 bg-muted/40 border border-border`), featuring a header with an alert icon (`alert-circle`), user-friendly message (`LLM_HIGH_DEMAND_ERROR_MESSAGE`), and a prominent retry button with a refresh icon.

➡️ **Settled Position**: Option B. Consistent with the application's clean design system, the card provides clear visual boundaries, legible typography, and an accessible retry button.

---

### ❓ **Q2** - **Retry URL Construction in Executive Brief Endpoint**
How should `/hx/executive-brief` build the retry URL for the button?
- Option A: Hardcode `/hx/executive-brief?status=unread`.
- Option B (Recommended): Dynamically format the retry URL incorporating all active query filters (`status`, `offset`, `limit`, `feed_id`, `category_id`) from `EntriesFilterQuery`.

➡️ **Settled Position**: Option B. Retrying must preserve the reader's active feed, category, and pagination context without resetting to default.

---

### ❓ **Q3** - **List View Synchronization Strategy (`#list-executive-brief`)**
How should the list view handle executive brief failures when the user is in list mode?
- Option A: Leave `#list-executive-brief` pulsing or hide it.
- Option B (Recommended): In `index.html` `htmx:afterSwap`, inspect `#page-executive-brief`. If it contains an error card, synchronize that state to `#list-executive-brief` by removing `animate-pulse` and rendering a matching compact error card with a retry button targeting `#page-executive-brief`.

➡️ **Settled Position**: Option B. Ensures consistent feedback across both Magazine View and List View. Clicking "Retry" from either view triggers the request and updates both.

---

### ❓ **Q4** - **Article Detail Rewrite Error Presentation**
When `/hx/entries/{id}/rewrite` fails, how should the error and fallback content be presented inside `#ai-rewrite-box`?
- Option A: Show only the error card; do not show the fallback article text.
- Option B (Recommended): Render the contextual error card with "Retry" at the top of `#ai-rewrite-box`, followed by the fallback teaser / article summary text below it.

➡️ **Settled Position**: Option B. The reader can immediately read the article without disruption while knowing AI enhancement is temporarily paused with an option to retry.

---

### ❓ **Q5** - **Toast Notification Disposition**
Should floating toasts still be emitted on AI failures?
- Option A: Keep toasts alongside contextual cards (Option 2 from Ideation).
- Option B (Recommended): Completely remove floating toasts on AI failures (Option 1 explicitly selected by user).

➡️ **Settled Position**: Option B. All AI failure feedback is 100% contextual and inline.

---

## Frontier Status
All design tree decisions are settled based on the user's explicit choice of Option 1 and codebase requirements.
Frontier is empty (`Result: frontier-empty`). Ready to proceed to Design.
