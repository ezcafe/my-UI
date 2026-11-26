# Grill — Fix Magazine View Flash & Detail Impacts

Result: frontier-empty

## Frontier Check

- **Issue 1 (Magazine view flash):** Clear bug with a single clean fix (CSS `data-view-mode` on `#entries-list` + `applyAiViewMode` / `htmx:beforeSwap` sync). No product ambiguity.
- **Issue 2 (Detail photo + title + rewrite + LLM impacts):** Clear regression in both Auth skeleton (`#mag-entry-{id}` brief/photo extraction) and Guest detail (`GuestEntryPayload` + `renderGuestEntryDetail` hero photo, rewrite, and `#ai-analysis-box` HTMX trigger to existing unauthenticated `/hx/entries/{id}/analysis`). No product ambiguity.
- **Constraints:** Preserve guest privacy/auth boundary (still skip `/hx/entries/{id}` fetch, `author`, and `readingTime` in guest mode) and keep existing tests green.
