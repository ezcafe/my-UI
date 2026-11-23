# Design

## Overview
Refactor `#entries-magazine-view` in `crates/ui-components/templates/partials/entry_list.html` and `crates/ui-components/templates/partials/executive_brief_card.html` to create a 3-column editorial layout matching the reference screenshot.

## Layout Architecture

### 1. Magazine Container & CSS Rules
- Apply `container-type: inline-size;` to `#entries-magazine-view`.
- Use a 3-column CSS Grid layout defined via container queries:
  - Default (narrow container): single column `grid-cols-1 gap-6`.
  - Container width >= `56rem` (desktop / wide container):
    `grid-template-columns: minmax(220px, 1fr) minmax(380px, 2.5fr) minmax(220px, 1.1fr); gap: 1.5rem;`
  - Fully compliant with the user rule: no hardcoded fixed media query breakpoints, relying strictly on modern container queries and `minmax`.

### 2. Left Column: Executive Brief
- Housed in `#mag-col-brief` inside `#entries-magazine-view`:
  - Contains `#page-executive-brief`.
  - If brief is loading: displays skeleton pulse card with editorial styling.
  - If brief is loaded: displays `executive_brief_card.html` with:
    - Editorial header: "Executive Brief" (bold uppercase tracking-wider text with sparkles icon).
    - Narrative text: Single narrative paragraph (`text-xs md:text-sm text-foreground/90 leading-relaxed font-sans`).
    - Shaded background: `bg-muted/30 border border-border rounded-lg p-5`.

### 3. Center Column: Priority 1 + Priority 2
- Housed in `#mag-col-center`:
  - **Priority 1 (Hero - `loop.index0 == 0`)**:
    - Top section: Flex split with article image on left (`w-1/2 overflow-hidden rounded-md`) and headline + category + feed metadata on right (`w-1/2 p-4 flex flex-col justify-center`).
    - Bottom section: Narrative brief / excerpt flowing underneath (`text-xs md:text-sm text-muted-foreground/90 leading-relaxed mt-3`).
  - **Priority 2 (Featured Row - `loop.index0 >= 1 && loop.index0 <= 3`)**:
    - Placed directly below Priority 1 with a neat separator or gap.
    - Rendered as an auto-fitting 3-column grid (`grid-template-columns: repeat(auto-fit, minmax(130px, 1fr)); gap: 0.875rem;`).
    - Each card displays:
      - Aspect-ratio thumbnail image at top (`h-28 w-full object-cover rounded-md bg-muted`).
      - Category badge (`text-brand text-[11px] font-bold uppercase mt-2`).
      - Bold article title (`text-xs md:text-sm font-bold line-clamp-2 mt-1`).
      - Source feed and published date (`text-[11px] text-muted-foreground mt-1`).

### 4. Right Column: Priority 3
- Housed in `#mag-col-right`:
  - Vertical stack of secondary editorial cards (`flex flex-col gap-3.5`).
  - Displays remaining articles (`loop.index0 >= 4`).
  - Each Priority 3 card displays:
    - Category name in bold uppercase (`text-[11px] font-bold uppercase text-muted-foreground`).
    - Article title (`text-xs md:text-sm font-semibold text-foreground line-clamp-2 mt-0.5`).
    - Compact summary/excerpt (`text-xs text-muted-foreground line-clamp-2 leading-relaxed mt-1`).
    - Source feed & date metadata.

## Interaction & State Compatibility
- Retains `openEntryFromMagazine(id, feedTitle)` on titles and cards.
- Retains read/unread visual indicators.
- Retains `data-entry-id`, `data-feed-title`, and `data-category` for hot topic highlight scripting.
- Retains OOB swaps from HTMX when titles are updated.

## Result: done
