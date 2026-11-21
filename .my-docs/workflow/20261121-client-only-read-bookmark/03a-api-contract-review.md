Result: clean

# API Contract Review: Client-Only Read & Bookmark (Guest Mode) and Server Mutation Rejection

- **Stage:** `api-contract-review` (Mode simple / design-verify-phase)
- **Status:** Paid (`clean`)
- **Review Targets:**
  - [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-client-only-read-bookmark/03-design.md)
  - [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261121-client-only-read-bookmark/04-tasks.md)
  - Route handlers in [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - Template models in [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)

---

## 1. Executive Summary

This review validates the HTTP and interface contracts for client-only interactions and server-side mutation protection when running in guest mode (`AUTH_ENABLED=false`).

- **Zero Critical / Major Issues**: All endpoint signatures, status codes, and template struct properties conform to REST standards and project conventions.
- **Strict Server Protection**: All mutation endpoints immediately reject write attempts with `HTTP 403 Forbidden` (`StatusCode::FORBIDDEN`) when `AUTH_ENABLED=false`. Upstream Miniflux is never contacted for unauthenticated writes.
- **Client Template Decoupling**: In guest mode, templates omit `hx-post` attributes and hook directly into client JavaScript handlers, eliminating unnecessary HTTP calls.
- **Backward Compatibility**: In authenticated mode (`AUTH_ENABLED=true`), all HTMX mutations and upstream sync workflows operate unchanged.

---

## 2. Interface Contract Checklist

### 2.1 Route Mutation Endpoints ([crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs))

| Endpoint | Method | Guest Mode (`AUTH_ENABLED=false`) | Authenticated Mode (`AUTH_ENABLED=true`) |
|----------|--------|-----------------------------------|------------------------------------------|
| `/hx/entries/:id/read` | `POST` | `403 Forbidden` (`StatusCode::FORBIDDEN`) | Marks read/unread in Miniflux; returns `200 OK` |
| `/hx/entries/:id/bookmark` | `POST` | `403 Forbidden` (`StatusCode::FORBIDDEN`) | Toggles bookmark in Miniflux; returns `200 OK` + button HTML |
| `/hx/mark-page-read` | `POST` | `403 Forbidden` (`StatusCode::FORBIDDEN`) | Marks page read in Miniflux; returns refreshed list HTML |
| `/hx/feeds/:id/read` | `POST` | `403 Forbidden` (`StatusCode::FORBIDDEN`) | Marks feed read in Miniflux; returns `200 OK` |
| `/hx/mark-all-read` | `POST` | `403 Forbidden` (`StatusCode::FORBIDDEN`) | Marks all read in Miniflux; returns `200 OK` |

- [x] **HTTP Status Code**: Standard `403 Forbidden` (`StatusCode::FORBIDDEN`) clearly communicates that mutations are disallowed in guest mode.
- [x] **Rejection Payload**: Returns plain error text `(StatusCode::FORBIDDEN, "Mutations require authentication").into_response()` to prevent unexpected client-side JSON parsing errors.

### 2.2 Template Contract ([crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs))

- [x] **`EntryDetailTemplate` Struct**:
  - Adds `pub auth_enabled: bool` field.
  - Matches pattern established in `IndexTemplate` (`pub auth_enabled: bool`).
- [x] **Route Initialization**:
  - `get_entry_detail_htmx` initializes `auth_enabled: config.auth_enabled`.
  - Upstream read and rendering logic remain intact.

### 2.3 HTML & Client DOM Contracts

- [x] **Detail View Read Toggle (`entry_detail.html`)**:
  - `auth_enabled=true`: `hx-post="/hx/entries/{{ entry.id }}/read?status=unread"`, `hx-target="#entry-{{ entry.id }}"`, `hx-swap="outerHTML"`.
  - `auth_enabled=false`: omits `hx-post`; uses `onclick="toggleLocalReadStatus({{ entry.id }})"`.
- [x] **Detail View Bookmark Toggle (`entry_detail.html`)**:
  - `auth_enabled=true`: `hx-post="/hx/entries/{{ entry.id }}/bookmark"`, `hx-target="this"`, `hx-swap="outerHTML"`.
  - `auth_enabled=false`: omits `hx-post`; uses `onclick="triggerNativeBookmark('{{ entry.title|escape }}', '{{ entry.url|escape }}')"`.
- [x] **List Page Mark Read Button (`index.html`)**:
  - `auth_enabled=true`: `hx-post="/hx/mark-page-read"`, `hx-target="#entries-list"`, `hx-swap="innerHTML"`.
  - `auth_enabled=false`: omits `hx-post`; uses `onclick="markVisiblePageLocallyRead()"`.

---

## 3. Findings & Recommendations

### Critical / Major Issues
- **None**. Zero Critical or Major blockers found.

### Minor Suggestions (Design Polish)
1. **Bookmark Title Escaping**:
   - In `entry_detail.html`, when passing `entry.title` into JavaScript `triggerNativeBookmark`, use Askama's `|escape` filter or avoid double quotes inside single quotes to prevent breaking inline JS if titles contain apostrophes.
2. **Toast Feedback on Fallback Bookmark**:
   - When `navigator.share` is unavailable on desktop browsers, copying to clipboard and providing visual toast feedback (`Press Cmd/Ctrl+D to bookmark this article`) ensures clear user guidance.

---

## 4. Contract Sign-Off

The API contracts, template struct properties, and HTTP status code behaviors are sound, secure, and ready for implementation.
