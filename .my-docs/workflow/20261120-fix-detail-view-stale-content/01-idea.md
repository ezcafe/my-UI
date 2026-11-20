# Idea & Framing: Fix Stale Content in Article Detail View

**Result:** ideation-drafted  
**Has UI (tentative):** yes  
**Has API / DB / auth/PII/schema:** no | no | no  

---

## Key Takeaway

When navigating between articles, the detail panel currently un-hides immediately while retaining the previous article's rendered DOM content until the new HTMX request finishes. This causes previous article details to be displayed for 1–2 seconds. We will fix this by resetting panel content on close and immediately rendering a skeleton loading placeholder on open, ensuring the user never sees stale content.

---

## Problem Breakdown

1. **Premature un-hiding**: In [`crates/ui-components/templates/partials/entry_item.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_item.html#L2) and [`crates/ui-components/templates/partials/entry_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L10), `onclick="document.getElementById('detail-panel').classList.remove('hidden')"` runs instantly before HTMX fetches the new content.
2. **Persistent DOM on back**: In [`crates/ui-components/templates/partials/entry_detail.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L5), clicking "Back" only adds the `hidden` class. It does not reset or empty `#detail-panel`.
3. **Network round-trip delay**: Fetching an article detail (`GET /hx/entries/{id}`) queries the backend GraphQL service and auto-marks the entry as read. During this 1–2s network latency, `#detail-panel` displays the previous entry's title, AI brief, and text.

---

## Goals

1. **Zero Stale Content**:
   - The user must never see the previous article's content when opening a new article.
2. **Immediate Visual Feedback**:
   - Clicking an article immediately opens the detail view with a clean, smooth loading state (skeleton placeholder).
3. **Clean Teardown on Back**:
   - Clicking "Back" hides the panel and clears existing content so stale data never persists in the DOM.
4. **Seamless Content Replacement**:
   - When HTMX completes the fetch, the real article details seamlessly replace the loading state without layout flicker.

---

## Non-Goals & Anti-Goals

- **Non-Goals**:
  - No changes to database schema, Miniflux GraphQL schema, or user authentication.
  - No pagination or filter logic alterations.
- **Anti-Goals**:
  - Do not keep the user stuck in the list view without visual feedback while an article loads.
  - Do not introduce heavy external JavaScript libraries or complex client state frameworks.

---

## Constraints

- Pure client-side / template fix within Askama templates and HTMX patterns.
- Preserve Spatial Glassmorphism styling (`ar-glass-panel`, dark palette, Tailwind typography).
- TDD: Test coverage verifying detail endpoints and template markup attributes.

---

## Decision Options

### Option 1 (Recommended): Immediate Skeleton Placeholder on Open & Reset on Back

- **Explanation**:
  Clicking an entry immediately resets `#detail-panel`, renders a matching pulsing skeleton placeholder (header, brief card, body text), and reveals the panel. Clicking "Back" hides the panel and clears the DOM content. When HTMX finishes loading, the new article swaps in cleanly.
- **Concrete Example**:
  ```javascript
  function openDetailWithSkeleton() {
    const panel = document.getElementById('detail-panel');
    panel.innerHTML = `
      <div class="flex flex-col h-full animate-pulse">
        <div class="flex items-center justify-between pb-4 border-b border-white/10 mb-6 shrink-0">
          <div class="h-7 w-20 bg-white/10 rounded-full"></div>
          <div class="h-7 w-24 bg-white/10 rounded-full"></div>
        </div>
        <div class="h-4 bg-white/10 rounded w-1/4 mb-3"></div>
        <div class="h-7 bg-white/15 rounded-lg w-3/4 mb-6"></div>
        <div class="h-24 bg-white/5 rounded-2xl mb-6 border border-white/5"></div>
        <div class="space-y-3">
          <div class="h-4 bg-white/10 rounded w-full"></div>
          <div class="h-4 bg-white/10 rounded w-5/6"></div>
          <div class="h-4 bg-white/10 rounded w-4/6"></div>
        </div>
      </div>`;
    panel.classList.remove('hidden');
  }

  function closeDetailPanel() {
    const panel = document.getElementById('detail-panel');
    panel.classList.add('hidden');
    panel.innerHTML = '';
  }
  ```
- **Pros**:
  - **Instant response**: The user sees immediate confirmation of their click.
  - **Zero stale flash**: Old article content is wiped instantly.
  - **Polished UX**: Skeleton matches the spatial glassmorphism detail layout.
- **Cons**:
  - Requires a small helper function in `index.html` or template partial.

---

### Option 2: Reveal Panel Only After HTMX Swap Completes

- **Explanation**:
  Keep `#detail-panel` hidden while the HTMX request is in flight. Show an inline loading spinner or pulse on the clicked list item. Reveal `#detail-panel` only when `htmx:afterSwap` completes.
- **Concrete Example**:
  ```html
  <div ...
    hx-get="/hx/entries/{{ entry.id }}"
    hx-target="#detail-panel"
    hx-indicator="#entry-{{ entry.id }}-spinner"
    hx-on::after-swap="document.getElementById('detail-panel').classList.remove('hidden')">
  ```
- **Pros**:
  - No skeleton HTML needed; panel only appears when final content is 100% ready.
- **Cons**:
  - **Sluggish feel**: For 1–2 seconds after clicking an item, nothing in the main viewport changes, leaving the user wondering if their click registered.

---

### Option 3: Immediate Panel Open with Centered Spinner

- **Explanation**:
  Immediately open `#detail-panel` on click, but display a simple centered spinner rather than a skeleton. Clear content on close.
- **Concrete Example**:
  ```javascript
  function openDetailWithSpinner() {
    const panel = document.getElementById('detail-panel');
    panel.innerHTML = '<div class="flex items-center justify-center h-full text-indigo-400"><i data-lucide="loader-2" class="w-8 h-8 animate-spin"></i></div>';
    lucide.createIcons();
    panel.classList.remove('hidden');
  }
  ```
- **Pros**:
  - Simple implementation; prevents stale content display.
- **Cons**:
  - Blank glass panel with a solitary spinner feels less fluid and modern than a structural skeleton.

---

## Clarification Question for User

> **Skip option**: If you prefer to proceed immediately with Option 1, you can simply reply **"just start"** or **"proceed with Option 1"**.

**Question:**  
When you click an article from the list, would you like the detail view to open immediately showing a glassmorphic skeleton placeholder (Option 1, recommended), or do you prefer one of the alternative options (e.g. waiting to open until loaded, or showing a centered spinner)?
