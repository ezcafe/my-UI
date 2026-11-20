# Grill: Design Decisions & Frontier

- **Result:** closed
- **Decisions settled:**
  - **Q1 (Today Scope):** Option B — Filter all articles published today (both read and unread, using `after: <start_of_today_timestamp>` without status constraint). Provenance: user decision `1B`.
  - **Q2 (Row Actions):** Option A — Strict title-only row matching `ar-list.html` with unread dot indicator, title, external link icon button (`target="_blank" rel="noopener noreferrer" onclick="event.stopPropagation()"`), and chevron-right reader opener. Provenance: user decision `2A`.
  - **Q3 (Active Filter Pill):** Option A — Client-side inline toggle on `.filter-pill` buttons for instant styling feedback without server round-trip latency. Provenance: user decision `3A`.

## Design Tree & Frontier

```
Design Frontier
├── Q1: "Today" filter query scope & unread status
├── Q2: Row action buttons (Bookmark/Read toggle vs pure minimal title row)
└── Q3: Active filter pill state management (Client-side vs Server-swapped)
```

---

### ❓ Q1 - "Today" Filter Scope: Should "Today" filter show unread articles published since midnight today?

- **Option A (Recommended)**: Filter **unread** articles published today (since start of current day 00:00:00 UTC).
  - **Explanation**: Users filtering for "Today" in an RSS reader typically want today's fresh unread items to catch up on what arrived today.
  - **Concrete example**:
    ```rust
    // In crates/server/src/routes/entries.rs
    Some("today") => {
        let start_of_today = chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        // GraphQL filter: status: UNREAD, publishedAfter: <start_of_today>
    }
    ```
  - **Pros**: Matches daily RSS reader triage habits; avoids clutter from already-read items from earlier today.
  - **Cons**: If a user already read an item today, it will not appear in this view unless they switch to "All Titles".

- **Option B**: Filter **all** articles published today (both read and unread).
  - **Explanation**: Treats "Today" strictly as a date range filter without any status constraint.
  - **Concrete example**:
    ```rust
    Some("today") => {
        let start_of_today = chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        // GraphQL filter: publishedAfter: <start_of_today> (no status filter)
    }
    ```
  - **Pros**: Completely agnostic to read/unread status.
  - **Cons**: Mixes read and unread items, reducing triage utility for RSS workflows.

➡️ **Recommendation**: Option A — "Today" shows unread items published since midnight today. Answering **"yes"** accepts Option A.

---

### ❓ Q2 - Row Action Buttons: Should the list row be strictly title-only (omitting inline bookmark/read buttons) matching `ar-list.html`?

- **Option A (Recommended)**: Strictly title-only row matching `ar-list.html`. Left side has unread indicator dot and title (`line-clamp-2`); right side has external link button (`target="_blank" rel="noopener noreferrer"`) and chevron reader trigger. Bookmark and read toggle remain available inside the reader detail view.
  - **Concrete example**:
    ```html
    <div class="ar-list-item px-4 py-3 rounded-2xl flex items-center justify-between gap-4 block cursor-pointer select-none" id="entry-{{ entry.id }}">
      <div class="flex items-center gap-2.5 flex-1 min-w-0" hx-get="/hx/entries/{{ entry.id }}" hx-target="#detail-panel" onclick="...">
        {% if !entry.is_read %}
        <span class="w-2 h-2 rounded-full bg-indigo-400 ring-4 ring-indigo-500/20 shrink-0"></span>
        {% endif %}
        <span class="text-sm font-medium leading-snug line-clamp-2 text-slate-100 flex-1">{{ entry.title }}</span>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <a href="{{ entry.url }}" target="_blank" rel="noopener noreferrer" onclick="event.stopPropagation()" class="p-1.5 rounded-full hover:bg-white/10 text-slate-400 hover:text-indigo-300 transition-colors" title="Open in browser">
          <i data-lucide="external-link" class="w-4 h-4"></i>
        </a>
        <i data-lucide="chevron-right" class="w-4 h-4 text-slate-500"></i>
      </div>
    </div>
    ```
  - **Pros**: Matches `ar-list.html` exactly, keeping spatial rows clean, spacious, and gaze-friendly without button clutter.
  - **Cons**: Marking an item read or bookmarked requires opening the detail panel (or bulk "Mark all read").

- **Option B**: Keep inline bookmark and read toggle buttons alongside the external link button on every row.
  - **Explanation**: Retains the bookmark and read toggle icon buttons in the row action group on the right.
  - **Concrete example**: Row contains `[Bookmark Button] [Read Button] [External Link Button] [Chevron]`.
  - **Pros**: Fast single-click bookmarking or marking as read without opening the detail view.
  - **Cons**: Clutters the row and deviates from the minimal spatial `ar-list.html` style.

➡️ **Recommendation**: Option A — Strictly title-only row with external link button and chevron, moving read/bookmark actions to the detail reader. Answering **"yes"** accepts Option A.

---

### ❓ Q3 - Active Filter Pill Styling: Should active pill state be managed via lightweight client-side script?

- **Option A (Recommended)**: Use a lightweight inline JavaScript helper (or event listener) on filter pills that toggles active styling immediately on click, matching `ar-list.html`'s `setFilter` pattern.
  - **Explanation**: When any filter pill is clicked, the active styling classes (`bg-white/10 text-white border-white/20`) are applied to the clicked pill, and inactive styles (`bg-white/5 text-slate-400 border-white/10`) are restored to other pills.
  - **Concrete example**:
    ```javascript
    function setActiveFilter(btn) {
      document.querySelectorAll('.filter-pill').forEach(el => {
        el.classList.remove('bg-indigo-500/20', 'text-indigo-300', 'border-indigo-400/30', 'bg-white/10', 'text-white', 'border-white/20');
        el.classList.add('bg-white/5', 'text-slate-400', 'border-white/10');
      });
      btn.classList.add('bg-indigo-500/20', 'text-indigo-300', 'border-indigo-400/30');
      btn.classList.remove('bg-white/5', 'text-slate-400');
    }
    ```
  - **Pros**: Instant UI feedback without server round-trip latency; doesn't require modifying HTMX response payload structure.
  - **Cons**: Relies on a small client-side script snippet in `index.html`.

- **Option B**: Out-of-band swap (`hx-swap-oob="true"`) for the filter toolbar on every filter response.
  - **Explanation**: Every response from `/hx/entries` returns the updated filter button toolbar alongside the entries list.
  - **Pros**: State is entirely server-rendered.
  - **Cons**: Extra server rendering overhead and more complex template coupling for a simple visual tab switch.

➡️ **Recommendation**: Option A — Instant client-side active pill toggle. Answering **"yes"** accepts Option A.
