# Grill: Client-Only Read & Bookmark (Guest Mode)

Result: frontier-empty

## Frontier Rounds

### Round 1

❓ **Q1** - **Guest Mode Read & Bookmark Handling**:
When `AUTH_ENABLED=false` (Guest Mode), should mark read/unread, bookmarking, and bulk page read avoid server network calls entirely and operate strictly on local client state (IndexedDB / Web Share / Toast)?

➡️ **User decision**: Yes, update both mark read/unread and bookmarking to client-only interactions when `AUTH_ENABLED=false`.

---

❓ **Q2** - **Server Rejection Behavior for Mutations in Guest Mode**:
When a client sends a mutation request (e.g. `POST /hx/entries/{id}/read`, `POST /hx/entries/{id}/bookmark`, `POST /hx/mark-page-read`, `POST /hx/feeds/{id}/read`, or `POST /hx/mark-all-read`) while `AUTH_ENABLED=false`, how should the server respond?

➡️ **User decision**: The server must reject the mutation with `HTTP 403 Forbidden` (`StatusCode::FORBIDDEN`).

---

❓ **Q3** - **Template Variable Injection for EntryDetail**:
To conditionally toggle between HTMX server mutations and client-side JavaScript actions in `entry_detail.html`, should `EntryDetailTemplate` accept `auth_enabled: bool` from route context?

➡️ **User decision**: Yes, pass `auth_enabled: bool` into `EntryDetailTemplate`.

---

❓ **Q4** - **Guest Mode Bookmark UX**:
Since bookmarks cannot be synchronized to upstream Miniflux without authentication, how should the bookmark action behave on the client?

➡️ **User decision**: Use native browser sharing via `navigator.share` if available, or copy the link to clipboard with an instructive toast guiding the user to bookmark with their browser shortcut (`Cmd/Ctrl+D`).
