# Gate A — Framing review

**Result:** ok

## Framing quality

The idea in `01-idea.md` is clear, properly scoped, and actionable.

### Strengths

- **Goal** — well-defined: Rust monorepo wrapper for self-hosted services, Miniflux first
- **Non-goals / anti-goals** — explicitly fence out what this is not (no re-implementation, no DB, no SPA, no REST API surface)
- **Stack** — concrete and consistent: Axum, async-graphql, HTMX, Pocket-ID OIDC, Cargo workspaces
- **Auth model** — single clear path: Pocket-ID OIDC gates all access, Miniflux API key in `.env`
- **UI reference** — two HTML prototypes (`ar-list.html`, `ar-detail.html`) give exact visual spec: glassmorphism, 720px spatial window, dark theme, indigo/purple accents
- **Extensibility** — first-class goal: architecture must support adding more upstream services
- **API surface known** — full Miniflux OpenAPI 2.2.0 spec (2081 lines, 50+ endpoints, 20+ schemas) already in repo

### Scope boundary — what the wrapper covers for Miniflux

Based on the reference UI and the OpenAPI spec, the first-pass scope covers:

| Area | Miniflux endpoints used | UI |
|------|------------------------|----|
| Feed list (title-only) | `getEntries`, `getFeeds`, `fetchCounters` | `ar-list.html` — search, all/unread filter, mark-all-read |
| Article reader | `getEntry`, `updateEntriesStatus`, `fetchContent` | `ar-detail.html` — header, AI summary card, body, prev/next nav |
| Toggle bookmark | `toggleBookmark` | Implied (starred state on Entry) |
| Mark read on view | `updateEntriesStatus` | Auto on open |

Feed management (create/update/delete feeds, categories, OPML import/export, users, API keys) is **out of scope for v1 UI** — users manage feeds in Miniflux's native UI.

### No issues found

- No conflicting constraints
- No ambiguous requirements
- Scope is achievable as a first milestone
