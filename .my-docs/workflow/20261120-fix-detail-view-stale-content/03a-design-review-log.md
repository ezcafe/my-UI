Result: clean

# Design Review Log: Fix Stale Content in Article Detail View

- **Stage:** `design-verify-phase`
- **Mode:** simple | **Profile:** lite
- **Surfaces:** UI (yes), API (no), DB (no), auth/PII/schema (no)
- **Status:** Paid (`clean`)

---

## 1. Review Summary

The design artifacts (`01-idea.md`, `02-analysis.md`, `02b-grill.md`, `03-design.md`, and `04-tasks.md`) have been evaluated against architecture principles, zero-stale flash guarantees, edge case resiliency, glassmorphism design fidelity, and TDD specifications.

- **API Surface**: Skipped (`Has API = no`). All changes are client-side template enhancements; backend Axum routes and Miniflux client remain untouched.
- **Database Surface**: Skipped (`Has DB = no`). No database schema or migrations are involved.
- **UI & Interaction Surface**: Verified compliant with Spatial glassmorphism design and the user rule against hardcoded breakpoints. Eliminates stale content display by immediately wiping old DOM content and rendering a pulsing skeleton placeholder on open, plus a full reset on back navigation.
- **Grill Stress-Test**: Settled (`frontier-empty` in `02b-grill.md`). Settled all frontier questions regarding skeleton structure, interactive dismissal during slow networks, and rapid click handling.
- **Critical / Major Findings**: 0.

---

## 2. Detailed Surface Reviews

### 2.1 UI & Interaction Architecture
- **Zero-Stale Flash Guarantee**:
  - `openDetailWithSkeleton()` synchronously wipes `#detail-panel.innerHTML` and renders a glassmorphic skeleton before the network request begins.
  - `closeDetailPanel()` adds `hidden` and empties `innerHTML`, ensuring no stale article data lingers in the DOM.
  - When HTMX completes the fetch, it swaps `EntryDetailTemplate` directly into `#detail-panel`, cleanly replacing the skeleton.
- **Interactive Skeleton Back Navigation**:
  - The skeleton header includes a working Back button wired to `closeDetailPanel()`.
  - Users are never trapped in an un-dismissible loading state if network latency is high or an upstream error occurs.
  - The Back button in the skeleton uses an inline SVG icon, ensuring instant rendering without requiring dynamic `lucide.createIcons()` execution.
- **Glassmorphism Design Fidelity**:
  - Matches the existing visual tokens: `bg-white/[0.03]` brief card, `border-white/10` borders, `rounded-full` pills, and `animate-pulse`.
  - Preserves container-query responsiveness inside `#detail-panel` without fixed breakpoints.

### 2.2 Edge Cases & Network State Handling
- **Slow Network & Request Failure**:
  - An interactive Back button in the skeleton allows the user to dismiss the panel at any time.
  - If the server encounters an error, the backend renders an out-of-band toast into `#toast-container` without breaking the panel layout.
- **In-Flight Response After Close**:
  - If a user clicks Back while an article is loading, `closeDetailPanel()` adds `hidden`.
  - When HTMX completes the swap, `#detail-panel` remains hidden because `entry_detail.html` does not modify the panel's visibility class.
  - Opening any future article immediately replaces the contents with a fresh skeleton.
- **Rapid Article Switching**:
  - Unconditional DOM replacement in `openDetailWithSkeleton()` ensures previous article markup is purged on every click.

### 2.3 Test Plan Review (`04-tasks.md`)
The TDD plan defines 5 targeted unit tests in `crates/ui-components/src/templates.rs`:
1. `test_entry_item_wires_open_detail_with_skeleton`: Verifies `EntryItemTemplate` renders `onclick="openDetailWithSkeleton()"`.
2. `test_entry_list_wires_open_detail_with_skeleton`: Verifies `EntryListTemplate` renders list entries with `onclick="openDetailWithSkeleton()"`.
3. `test_entry_detail_wires_close_detail_panel`: Verifies `EntryDetailTemplate` Back button renders with `onclick="closeDetailPanel()"`.
4. `test_index_template_defines_detail_panel_helpers`: Verifies `IndexTemplate` defines `openDetailWithSkeleton` and `closeDetailPanel` in the `<script>` block.
5. `test_index_template_skeleton_structure`: Verifies the pulse skeleton structure, interactive back button, and placeholder blocks in `IndexTemplate`.

---

## 3. Findings & Notes

| ID | Severity | Surface | Finding | Resolution |
|---|---|---|---|---|
| D-01 | Minor | UI | Lucide icons in dynamically injected HTML require manual `createIcons()` | Use an inline SVG for the skeleton Back button to ensure instant visual rendering without calling `lucide.createIcons()`. |
| D-02 | Minor | UI | Maintain focus/pointer events on interactive elements within pulse container | Add `pointer-events-auto` to the skeleton Back button so users can click it even inside the pulsing skeleton container. |

---

## 4. Gate Sign-Off

- **Overall Result:** `clean`
- **Isolated API Result:** `skipped` (`Has API = no`)
- **Isolated DB Result:** `skipped` (`Has DB = no`)
- **Ready for Gate B:** Yes
