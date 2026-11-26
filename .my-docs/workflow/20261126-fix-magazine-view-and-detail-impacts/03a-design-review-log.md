# Design Review Log — Fix Magazine View Flash & Detail Impacts

Result: clean

**Stage:** `design-verify-phase` (Mode: `simple`, Review profile: `lite`)  
**API / DB surfaces:** `skipped` (Notes-skip — `Has API = no`, `Has DB = no`)  
**Grill bar:** `frontier-empty` (`02b-grill.md`)

## Summary

Reviewed `01-idea.md`, `02-analysis.md`, `02b-grill.md`, `03-design.md`, and `04-tasks.md` against the codebase (`crates/ui-components/templates/pages/index.html`, `crates/ui-components/templates/partials/entry_list.html`, `crates/ui-components/templates/partials/entry_detail.html`, and `crates/ui-components/src/templates.rs`) and HTMX documentation (`htmx:beforeSwap` and `htmx.process()`).

- **Critical:** 0
- **Major:** 0
- **Minor / Implementation Notes:** 1
  1. **`htmx:beforeSwap` guard ordering (`index.html:1937`):** The existing `htmx:beforeSwap` listener returns early when `!pendingGuestMarkRead || pendingGuestMarkRead.cancelled`. When adding `data-view-mode` synchronization on `#entries-list` in `htmx:beforeSwap`, place it **before** that guest-mark-read early return so it runs for auth-mode `mark-page-read-btn`, pagination, filter tabs, and search swaps as well.

## Checklist Verification

- **Design completeness:** Both regressions (Magazine view flash on `#entries-list` swap; Auth skeleton & Guest detail photo/title/rewrite + `/hx/entries/{id}/analysis` trigger) are fully specified with exact selectors, data fields, and test updates.
- **Security & privacy:** Guest mode preserves privacy boundaries (no `/hx/entries/{id}` fetch, no `author`, no `readingTime`); `image_url` in `renderGuestEntryDetail` is sanitized via `safeHref` and `escapeHtml`.
- **HTMX lifecycle grounding:** `htmx:beforeSwap` fires before DOM insertion (preventing list-view flash before `htmx:afterSwap`), and `htmx.process(panel)` activates `hx-get` + `hx-trigger="load"` on dynamically rendered `#ai-analysis-box`.
