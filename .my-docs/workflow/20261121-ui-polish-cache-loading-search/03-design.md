# Design: UI Polish, Client-Side Filter Caching & Loading State

Result: done

## Executive Summary

This design provides four targeted UI and client-state enhancements:
1. **Centered Login Card**: Replaces the split dual-panel login screen in `login.html` with a centered Swiss card showcasing "MY NEWS", subtitle, and Pocket-ID sign-in button.
2. **Remove Last Item Border**: Uses `last:border-b-0` in `entry_list.html` and `entry_item.html` so the final entry has no trailing divider.
3. **Filter 1-Minute Client Caching**: Adds an in-memory `filterCache` (`Map`) in `index.html` with a 60-second TTL to instantly swap entries when re-clicking filter tabs without network roundtrips.
4. **HTMX Loading State**: Adds a top 2px brand-red indicator bar (`#entries-loading-bar`) in `index.html` that pulses during `hx-get="/hx/entries"`.
5. **Non-Goal**: The Search Clear (x) button is explicitly excluded.

---

## Architectural Options & Trade-Offs

### 1. Login Screen Layout

#### Option A: Centered Single Floating Swiss Card (Recommended)
- **Explanation**:
  - Remove the left brand panel (`bg-[var(--brand-surface)]`) entirely.
  - Wrap the card in a centered layout: `min-h-screen flex items-center justify-center p-6 bg-background`.
  - Place "MY NEWS" in bold `font-display`, an editorial directory badge, a concise description, and the Pocket-ID ink pill button inside a bordered `bg-card`.
- **Concrete Example**:
  ```html
  <div class="min-h-screen flex items-center justify-center p-6 bg-background">
    <div class="w-full max-w-sm rounded-lg border border-border bg-card p-8 space-y-8 text-center text-foreground">
      <div class="space-y-3">
        <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-border bg-muted text-xs font-bold tracking-wider uppercase">
          Editorial Directory
        </div>
        <h1 class="font-display text-4xl sm:text-5xl font-extrabold uppercase tracking-tight text-foreground">
          MY NEWS
        </h1>
        <p class="font-sans text-sm text-muted-foreground leading-relaxed max-w-xs mx-auto">
          High-density information index and curated reading environment built with Swiss typographic discipline.
        </p>
      </div>

      <div class="pt-2">
        <a href="/auth/login" class="w-full inline-flex items-center justify-center gap-2.5 px-6 py-3 rounded-full bg-primary text-primary-foreground font-bold font-sans text-sm hover:opacity-90 transition-opacity">
          <span>Sign in with Pocket-ID</span>
          <i data-lucide="arrow-right" class="w-4 h-4"></i>
        </a>
      </div>

      <div class="pt-4 text-xs font-sans text-muted-foreground tabular border-t border-border/40">
        <span>Edition 2026</span> • <span>Zero Page Shadows</span>
      </div>
    </div>
  </div>
  ```
- **Pros**:
  - Clean, focused, self-contained view on both mobile and desktop.
  - Eliminates visual clutter and matches the spatial aesthetic.
- **Cons**:
  - Requires updating `test_login_page_renders_dual_panel_swiss_brand` in `templates.rs`.

#### Option B: Centered Minimalist Card with Red Brand Header
- **Explanation**:
  - Keep a red banner inside the card instead of full-page panel.
- **Pros**:
  - Retains more red color on screen.
- **Cons**:
  - Violates Annuaire Rule 1 ("Red marks position, never action") by overusing decorative red in cards.

---

### 2. Client-Side Filter Caching (1-Minute TTL)

#### Option A: In-Memory Map with Event Interception & Mutation Invalidation (Recommended)
- **Explanation**:
  - Maintain `const filterCache = new Map();` in `index.html`.
  - When a filter pill is clicked, `handleFilterClick(this, event)` checks if `Date.now() - entry.timestamp < 60000` for that status (provided search query is empty).
  - On cache hit: call `event.preventDefault()`, set `entriesList.innerHTML = entry.html`, update Lucide icons, sync local read states, and update active pill classes.
  - On cache miss: let HTMX issue the request; cache response HTML in `htmx:afterSwap`.
  - Invalidate cache (`filterCache.clear()`) on any article state change (mark read, bookmark, mark page read).
- **Concrete Example**:
  ```javascript
  const FILTER_CACHE_TTL_MS = 60000;
  const filterCache = new Map();

  function invalidateFilterCache() {
    filterCache.clear();
  }

  function handleFilterClick(btn, event) {
    const status = btn.getAttribute('data-status');
    const searchInput = document.getElementById('search-input');
    const searchQuery = (searchInput && searchInput.value) ? searchInput.value.trim() : '';

    if (!searchQuery && filterCache.has(status)) {
      const cached = filterCache.get(status);
      if (Date.now() - cached.timestamp < FILTER_CACHE_TTL_MS) {
        if (event) {
          event.preventDefault();
          event.stopPropagation();
        }
        const entriesList = document.getElementById('entries-list');
        if (entriesList) {
          entriesList.innerHTML = cached.html;
          lucide.createIcons();
          if (typeof syncDomReadStates === 'function') {
            syncDomReadStates();
          }
        }
        setActiveFilter(btn);
        return;
      }
    }
    setActiveFilter(btn);
  }
  ```
- **Pros**:
  - Zero network overhead when switching back and forth between filters within 1 minute.
  - Guaranteed fresh state upon any mutation due to automatic cache invalidation.
- **Cons**:
  - Requires handling both HTMX event flow and cache storage.

#### Option B: LocalStorage or SessionStorage Caching
- **Explanation**:
  - Serialize HTML snippets into browser `localStorage`.
- **Pros**:
  - Survives page refresh.
- **Cons**:
  - Larger storage footprint, synchronous serialization overhead, and stale data risk across sessions.

---

### 3. HTMX Loading State Indicator

#### Option A: Slim 2px Pulsing Brand Red Bar above Entries List (Recommended)
- **Explanation**:
  - Insert `<div id="entries-loading-bar" class="htmx-indicator h-0.5 bg-brand animate-pulse w-full shrink-0"></div>` directly above `#entries-list`.
  - Add `hx-indicator="#entries-loading-bar"` to filter pills and search triggers.
  - Ensure `.htmx-indicator` is styled with `display: none` by default and `display: block` when active.
- **Concrete Example**:
  ```html
  <div id="entries-loading-bar" class="htmx-indicator h-0.5 bg-brand animate-pulse w-full shrink-0"></div>
  ```
  ```css
  .htmx-indicator {
    display: none;
  }
  .htmx-request.htmx-indicator,
  .htmx-request .htmx-indicator {
    display: block;
  }
  ```
- **Pros**:
  - Zero layout shift (progress bar is fixed height or seamlessly integrated).
  - Immediate visual feedback on network latency.
- **Cons**:
  - Must ensure it does not activate when a cached filter hit prevents the request.

---

## Detailed Implementation Specifications

### 1. `crates/ui-components/templates/pages/login.html`
- Replace 2-column grid with single centered container.
- Display "Editorial Directory" badge, "MY NEWS" title, subtitle, Pocket-ID button, and edition footer.

### 2. `crates/ui-components/templates/partials/entry_list.html` & `entry_item.html`
- Add `last:border-b-0` to the item class list:
  ```html
  <div class="annuaire-row ar-list-item relative px-4 py-2.5 rounded-lg flex items-center justify-between gap-3 cursor-pointer select-none group hover:bg-muted/60 transition-colors border-b border-border/60 last:border-b-0" id="entry-{{ entry.id }}" data-entry-id="{{ entry.id }}">
  ```

### 3. `crates/ui-components/templates/pages/index.html`
- Add `#entries-loading-bar` above `#entries-list`.
- Add `hx-indicator="#entries-loading-bar"` to filter pills and `#search-form`.
- Add `onclick="handleFilterClick(this, event)"` to all filter pills.
- Implement `filterCache`, `handleFilterClick`, and `invalidateFilterCache`.
- Attach `invalidateFilterCache()` to `mark-page-read`, `markLocallyRead`, `unmarkLocallyRead`, and detail panel mutation events.

### 4. `crates/ui-components/templates/base.html`
- Add explicit `.htmx-indicator` styles to `<style>` block.

### 5. `crates/ui-components/src/templates.rs`
- Update `test_login_page_renders_dual_panel_swiss_brand` to `test_login_page_renders_centered_card`.

---

## Verification & Test Plan

1. **Unit Tests (`cargo test -p ui-components`)**:
   - Verify `LoginTemplate` renders the centered layout, "MY NEWS", subtitle, and Pocket-ID button.
   - Verify `EntryListTemplate` and `EntryItemTemplate` include `last:border-b-0`.
   - Verify `IndexTemplate` renders `#entries-loading-bar` and cache handling functions.
2. **Browser / Manual Verification**:
   - Check centered login card at `/auth/login`.
   - Check last item border in directory view.
   - Verify filter clicks within 60 seconds reuse cached HTML without triggering network request.
   - Verify loading indicator activates during remote fetches.
