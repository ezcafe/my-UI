# Grill: UI Polish, Client-Side Filter Caching & Loading State

Result: frontier-empty

## Frontier Rounds

### Round 1

❓ **Q1** - **Centered Login Layout & Brand Treatment**:
Should `login.html` replace the dual-panel layout with a centered, self-contained Swiss card on `min-h-screen flex items-center justify-center p-6 bg-background`, showcasing the "MY NEWS" title in `font-display`, an editorial subtitle, and the Pocket-ID ink pill button?

➡️ **User decision**: Yes, remove the left brand panel and present a centered card with "MY NEWS", subtitle, and Pocket-ID button.

---

❓ **Q2** - **Last Item Border Cleanup**:
Should the bottom border on the final entry item in `entry_list.html` and `entry_item.html` be removed using the Tailwind utility `last:border-b-0`?

➡️ **User decision**: Yes, add `last:border-b-0` to prevent an awkward trailing divider before pagination or list bounds.

---

❓ **Q3** - **Filter Cache Strategy & Invalidation Policy**:
Should client-side filter caching in `index.html` use an in-memory `Map` with a 60-second TTL keyed by status (e.g. `unread`, `today`, `starred`, `all`, `ai`), serving instant cached HTML on cache hits, and invalidating the entire cache (`filterCache.clear()`) whenever an article status changes (read, unread, bookmark, mark page read)?

➡️ **User decision**: Yes, implement a 60-second TTL cache keyed by filter status that invalidates on any state-mutating operation.

---

❓ **Q4** - **HTMX Loading State Presentation**:
Should the loading indicator for entry requests be a subtle 2px brand red pulsing top bar (`#entries-loading-bar`) placed above `#entries-list` and activated via `hx-indicator="#entries-loading-bar"`?

➡️ **User decision**: Yes, use a slim top indicator bar in brand red that pulses during data requests.

---

❓ **Q5** - **Explicit Non-Goal Scope Confirmation**:
Should the "Search Clear (x) Button" remain excluded from this sprint?

➡️ **User decision**: Yes, explicitly skip the Search Clear button as directed.
