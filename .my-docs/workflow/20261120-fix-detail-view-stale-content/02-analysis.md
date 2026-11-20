# Analysis: Fix Stale Content in Article Detail View

Result: done

## Context & Architecture

The application is an AR Spatial RSS reader built with:
- **Backend**: Rust using Axum, Askama templates, async-graphql, and a Miniflux HTTP client.
- **Frontend**: Server-rendered HTML with Tailwind CSS, HTMX for reactive out-of-band updates, Lucide icons, and modern CSS container queries.
- **Detail View Flow**:
  1. Entry items in `#entries-list` trigger `GET /hx/entries/{id}` with `hx-target="#detail-panel"`.
  2. The server route in `crates/server/src/routes/entries.rs` fetches the article from Miniflux, marks it as read, sanitizes content, and renders `EntryDetailTemplate` (`crates/ui-components/templates/partials/entry_detail.html`).
  3. The rendered markup is swapped into `#detail-panel`.

---

## Root Cause Analysis

Currently, when a user clicks an article from the list:
1. **Premature un-hiding**:
   In `crates/ui-components/templates/partials/entry_item.html` and `crates/ui-components/templates/partials/entry_list.html`, the row contains:
   ```html
   onclick="document.getElementById('detail-panel').classList.remove('hidden')"
   ```
   This un-hides `#detail-panel` synchronously and immediately, before the network request begins or completes.
2. **Persistent DOM on back navigation**:
   In `crates/ui-components/templates/partials/entry_detail.html`, the Back button contains:
   ```html
   onclick="document.getElementById('detail-panel').classList.add('hidden')"
   ```
   This only toggles the CSS `hidden` class. The previous article's entire DOM tree remains inside `#detail-panel`.
3. **Network round-trip latency**:
   Fetching `GET /hx/entries/{id}` queries the backend and Miniflux, taking 500ms–2000ms. During this interval, the user is presented with the old article's title, publication metadata, AI summary, and body text.

---

## User Decision & Selected Approach

In `01-idea.md`, three options were evaluated:
- **Option 1 (Selected)**: Immediate glassmorphic skeleton placeholder on open & DOM reset on close.
- **Option 2**: Reveal panel only after HTMX swap completes.
- **Option 3**: Immediate panel open with centered spinner.

The user explicitly selected **Option 1**: Immediate Skeleton Placeholder on Open & Reset on Back (`provenance: ideation-confirmed: user: "option 1"`).

---

## Scope & Impact Analysis

### Affected Components
1. `crates/ui-components/templates/pages/index.html`:
   - Add client helper functions `openDetailWithSkeleton()` and `closeDetailPanel()` in the `<script>` section.
   - Design spatial glassmorphic skeleton layout matching `entry_detail.html`.
2. `crates/ui-components/templates/partials/entry_item.html`:
   - Replace raw `onclick="document.getElementById('detail-panel').classList.remove('hidden')"` with `onclick="openDetailWithSkeleton()"`.
3. `crates/ui-components/templates/partials/entry_list.html`:
   - Replace raw `onclick="document.getElementById('detail-panel').classList.remove('hidden')"` with `onclick="openDetailWithSkeleton()"`.
4. `crates/ui-components/templates/partials/entry_detail.html`:
   - Replace raw `onclick="document.getElementById('detail-panel').classList.add('hidden')"` with `onclick="closeDetailPanel()"`.
5. `crates/ui-components/src/templates.rs`:
   - Add unit tests verifying template contract updates, presence of client helper functions, and skeleton structure.

### Non-Goals & Anti-Goals
- No changes to backend database schema, Miniflux GraphQL schema, or server route logic.
- No heavy external JavaScript dependencies; maintain zero-dependency pure vanilla JS and HTMX.
- Do not trap users in a non-responsive skeleton if network hangs; skeleton will provide an interactive Back button.
