# Grill: Fix Stale Content in Article Detail View

Result: frontier-empty

## Frontier Analysis & Settled Decisions

### Frontier Round 1

❓ **Q1** - **Detail view placeholder strategy**:
When navigating from the article list to an article detail, should the UI immediately un-hide with a spatial skeleton placeholder, wait until network swap finishes, or show a spinner?

➡️ **Settled**: Option 1 chosen by user in Ideation stage (`user: "option 1"`). Render an immediate spatial glassmorphism skeleton placeholder on open, and reset the DOM on back navigation.

---

❓ **Q2** - **Dismissal and interactive Back button in skeleton state**:
If network latency is high or a request fails, should the skeleton include a working Back button to allow immediate dismissal, or remain purely static?

➡️ **Settled**: The skeleton will include a working Back button wired to `closeDetailPanel()`. This ensures the user is never trapped in a loading state and can dismiss the view at any time.

---

❓ **Q3** - **Race condition handling between rapid clicks and in-flight HTMX swaps**:
If a user rapidly clicks Article A, immediately closes the panel, and clicks Article B, how should state be managed?

➡️ **Settled**: `openDetailWithSkeleton()` unconditionally sets `panel.innerHTML` to the fresh skeleton and un-hides `#detail-panel`. When the HTMX swap for Article B finishes, it replaces `panel.innerHTML` with Article B's content. Any lingering or earlier responses are cleanly superseded by the explicit DOM overwrite on open.

---

## Conclusion

All frontier decisions are settled. No outstanding questions remain for the user. Frontier is empty.
