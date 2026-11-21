# Annuaire Design System & Style Guide

A comprehensive guide for building high-density, editorial, directory-style web applications inspired by the **Natel "Annuaire" direction**.

---

## 1. Core Visual Metaphor

The **Annuaire** style takes inspiration from classic European telephone directories and Swiss typography. It blends utilitarian print craftsmanship with modern digital interaction.

- **High signal-to-noise ratio**: Prioritize scannable text, structured data, and high information density without visual clutter.
- **Directory grammar**: Lead with the primary sorting key in bold, secondary descriptors in medium weight, connect details using dotted leaders, and right-align numbers.
- **Strict functional roles**: Colors, fonts, and shadows have exact single purposes. Never decorate for the sake of decoration.

---

## 2. The 6 Golden Rules

Follow these six fundamental rules to keep the visual identity consistent:

1. **Red marks position, never action.**
   - Allowed: Wordmarks, A–Z group letters, the 3px vertical active selection indicator, and the brand login panel.
   - Strictly forbidden: Buttons, links, error badges, or icons indicating deletion.
2. **Primary action is an Ink pill.**
   - The primary call-to-action is always a high-contrast pill button in solid ink (`#17181C` in light mode, `#ECECEC` in dark mode).
3. **Destructive actions are isolated.**
   - Destructive actions (Delete, Discard) use a dedicated destructive red (`#B3261E` / `#FF6B61`). They must **always** include both an icon and a text label.
4. **Two typefaces with strict roles.**
   - **Display font (Big Shoulders 700)**: Reserved for wordmarks, A–Z index letters, screen and dialog titles, and avatar initials. Never use it for running text or form labels.
   - **Body font (Work Sans 400 / 500 / 700)**: Used for all names, body text, form fields, inputs, and metadata.
5. **Zero page shadows.**
   - Elements sitting on the page background (cards, lists, rows, headers) never have box shadows. Contrast is created with subtle borders and surface shifts.
   - Shadows are reserved strictly for floating layers (dropdown menus, popovers, toasts, modal dialogs).
6. **Numbers are tabular.**
   - All phone numbers, dates, timestamps, and counters must use monospace tabular numbers (`font-variant-numeric: tabular-nums lining-nums`).

---

## 3. Color Palette & Design Tokens

The palette uses modern OKLCH color definitions with exact HEX fallbacks. Every color pair complies with WCAG AA (4.5:1 minimum contrast).

### Light & Dark Token Values

| Token | Light (HEX) | Dark (HEX) | Role & Usage |
|---|---|---|---|
| `--background` | `#FFFFFF` | `#121316` | Main page canvas (never pure `#000000` in dark mode) |
| `--foreground` | `#17181C` | `#ECECEC` | Primary text and headings |
| `--muted` | `#F4F4F3` | `#1B1C20` | Secondary surface, detail pane background |
| `--muted-foreground` | `#5F6168` | `#A2A4AB` | Subtitles, helper text, trailing values |
| `--card` | `#FFFFFF` | `#222328` | Card and form field background |
| `--border` | `#E4E4E2` | `#2C2D32` | Structural separators, row dividers |
| `--input` | `#7A7C82` | `#76787F` | Form input borders and outlines |
| `--primary` | `#17181C` | `#ECECEC` | Primary pill button background |
| `--primary-foreground` | `#FFFFFF` | `#17181C` | Primary pill button text |
| `--brand` | `#D8261D` | `#F04A40` | Positional marks (group letters, 3px active bar) |
| `--brand-surface` | `#D8261D` | `#A11F18` | Deep red surfaces (login sidebar, demo badges) |
| `--selection` | `#FDEBE9` | `#3A1B1A` | Active selected list item row background |
| `--destructive` | `#B3261E` | `#FF6B61` | Delete buttons and discard actions |
| `--leader` | `#B9BABE` | `#4A4C52` | Dotted line leader between text and trailing values |
| `--highlight` | `#FFD3CE` | `#7A2620` | Search match highlight background |

### The 6 Avatar Tints

For items or contacts without an image, derive a tint deterministically from the item's unique ID. These tints share identical perceptual lightness and deliberately omit red and pink:

- **Tint 1**: `#E8D5B5` (Light) / `#4A3F2C` (Dark) — Sand
- **Tint 2**: `#CFE0C8` (Light) / `#34452F` (Dark) — Sage
- **Tint 3**: `#C9DBEE` (Light) / `#2C3F55` (Dark) — Mist Blue
- **Tint 4**: `#DCCFEA` (Light) / `#43365A` (Dark) — Lavender
- **Tint 5**: `#C4E2DE` (Light) / `#2A4744` (Dark) — Seafoam
- **Tint 6**: `#EBD9A6` (Light) / `#4A4228` (Dark) — Warm Ochre

---

## 4. Typography Scale

Fonts are bundled locally (via Fontsource) with zero external CDN dependencies:
- `@fontsource-variable/big-shoulders`
- `@fontsource-variable/work-sans`

### Type Scale Reference

| Class / Token | Font Family | Size / Line Height | Usage |
|---|---|---|---|
| `text-caption` | Sans | 12px (0.75rem) / 16px | Counts, keyboard keys (`<kbd>`), hints |
| `text-small` | Sans | 13px (0.8125rem) / 18px | A–Z rail letters, tabs |
| `text-body` | Sans | 15px (0.9375rem) / 24px | Running body text (16px on mobile to avoid iOS zoom) |
| `text-name` | Sans | 17px (1.0625rem) / 24px | Primary item titles and contact names |
| `text-wordmark` | Display | 32px (2rem) / 1 | Brand logo / header wordmark |
| `text-title-sm` | Display | 36px (2.25rem) / 1 | Group letters; mobile screen headers |
| `text-title` | Display | 44px (2.75rem) / 0.95 | Desktop screen titles and section headers |
| `text-person` | Display | 56px (3.5rem) / 0.95 | Large prominent hero titles |

---

## 5. Elevation, Borders & Shadows

- **Surfaces on the canvas**: `shadow: none`. Contrast is created strictly by placing `bg-card` or `bg-background` against `bg-muted` and crisp 1px borders (`border-border`).
- **Floating elements**:
  - `shadow-float`: Popovers, dropdown menus, context menus, and floating toast pills.
  - `shadow-overlay`: Modal dialogs and mobile bottom sheets.
- **Dark mode elevation**: In dark mode, shadows convert into a subtle 1px light keyline (`oklch(1 0 0 / 0.07)`) with diffused dark falloff.

### Corner Radii Scale

- `--radius-sm` (6px): Badges, keyboard shortcut keys (`<kbd>`).
- `--radius-md` (8px): Form input fields, textareas, and selects.
- `--radius-lg` (10px): List item rows, rounded-square avatars.
- `--radius-xl` (14px): Modal dialog cards and bottom sheets.
- `rounded-full` (9999px): Buttons, search inputs, active selection pills, circular avatars.

---

## 6. Component Patterns & Recipes

### 6.1 Pill Buttons

Buttons are always rounded pills (`rounded-full`) with bold text. Minimum touch target is 40px (44–48px on mobile).

```html
<!-- Primary (Ink Pill) -->
<button class="inline-flex shrink-0 items-center justify-center gap-2 rounded-full font-bold h-10 px-4 text-sm bg-primary text-primary-foreground hover:bg-primary/85 active:bg-primary/75 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
  Primary Action
</button>

<!-- Outline -->
<button class="inline-flex shrink-0 items-center justify-center gap-2 rounded-full font-bold h-10 px-4 text-sm border border-input bg-background text-foreground hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
  Secondary Action
</button>

<!-- Destructive (Must have icon + text label) -->
<button class="inline-flex shrink-0 items-center justify-center gap-2 rounded-full font-bold h-10 px-4 text-sm bg-destructive text-destructive-foreground hover:bg-destructive/85 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
  <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M3 6h18m-2 0v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6m3 0V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
  </svg>
  Delete Item
</button>

<!-- Ghost / Quiet -->
<button class="inline-flex shrink-0 items-center justify-center gap-2 rounded-full font-bold size-10 text-muted-foreground hover:bg-muted hover:text-foreground">
  <svg class="size-5" ...></svg>
</button>
```

### 6.2 Form Input Fields

Inputs display labels above in bold, and help/errors below. Focus states render an ink border doubled by a 1px ring on both mouse and keyboard focus.

```html
<div class="space-y-1.5">
  <label for="username" class="block text-sm font-bold text-foreground">
    Username
  </label>
  <input
    id="username"
    type="text"
    aria-describedby="username-hint"
    class="field block w-full rounded-md border border-input bg-card px-3 h-10 text-base text-foreground placeholder:text-muted-foreground focus:outline-none focus:border-ring focus:ring-1 focus:ring-ring aria-[invalid=true]:border-destructive lg:text-body"
    placeholder="e.g. adessendre"
  />
  <p id="username-hint" class="text-caption text-muted-foreground">
    Your organizational directory login.
  </p>
</div>
```

#### In-Field Button Hit Target
Buttons placed inside inputs (e.g. search clear button or password toggle) render at 32px height visually, but expand invisibly to a 44px touch target using an `after` pseudo-element:

```css
.field-button-hit-area {
  position: relative;
}
.field-button-hit-area::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  top: -6px;
  bottom: -6px;
}
```

### 6.3 Dotted-Leader Directory Row

The signature visual pattern of Annuaire:

```html
<a href="#item" class="group relative flex min-h-[3.5rem] items-center gap-3 rounded-lg px-2 py-1.5 transition-colors hover:bg-muted data-[status=active]:bg-selection sm:min-h-[3.25rem] sm:py-0">
  <!-- Active Selection Bar (3px wide, 20px high, red brand) -->
  <span
    aria-hidden="true"
    class="absolute top-1/2 left-0 h-5 w-[3px] -translate-y-1/2 rounded-full bg-brand opacity-0 transition-opacity group-data-[status=active]:opacity-100"
  ></span>

  <!-- Avatar -->
  <div class="size-10 shrink-0 rounded-lg bg-avatar text-avatar-foreground flex items-center justify-center font-display font-bold text-base">
    AD
  </div>

  <!-- Content with Name, Leader, and Number -->
  <div class="min-w-0 flex-1 sm:flex sm:items-center sm:gap-3">
    <span class="block min-w-0 truncate text-name">
      <strong class="font-bold">Dessendre</strong>
      <span class="font-medium text-foreground/80"> Aline</span>
    </span>

    <!-- Dotted Leader Line -->
    <span class="leader hidden sm:block" aria-hidden="true"></span>

    <!-- Right-aligned Tabular Value -->
    <span class="block truncate text-sm tabular text-muted-foreground sm:shrink-0 sm:text-foreground">
      +41 21 693 11 11
    </span>
  </div>
</a>
```

#### The Dotted Leader CSS Utility
```css
.leader {
  flex: 1 1 1rem;
  min-width: 1rem;
  height: 0.25rem;
  align-self: center;
  transform: translateY(0.3em);
  background-image: radial-gradient(circle, var(--leader) 1px, transparent 1.2px);
  background-size: 6px 4px;
  background-repeat: repeat-x;
  background-position: left center;
}
```

### 6.4 Modals & Bottom Sheets

- **Mobile**: Anchored bottom sheet sliding up (`rounded-t-xl`).
- **Desktop (≥ 640px)**: Centered modal card (`sm:rounded-xl`).
- Must trap keyboard `Tab` navigation and close on `Escape` or backdrop click.

```html
<div class="fixed inset-0 z-50 flex items-end justify-center bg-black/40 sm:items-center dark:bg-black/60">
  <div
    role="dialog"
    aria-modal="true"
    class="max-h-[90dvh] w-full max-w-md overflow-y-auto rounded-t-xl bg-background p-6 shadow-overlay sm:rounded-xl"
  >
    <h2 class="font-display text-title-sm font-bold text-foreground">
      Dialog Title
    </h2>
    <p class="mt-2 text-body text-muted-foreground">
      Concise explanation of the action or setting being modified.
    </p>
    <div class="mt-6 flex justify-end gap-3">
      <button class="...button-outline...">Cancel</button>
      <button class="...button-primary...">Confirm</button>
    </div>
  </div>
</div>
```

### 6.5 Floating Toasts

Toasts are short notification pills floating at the bottom center of the viewport:

```html
<div class="pointer-events-none fixed inset-x-0 bottom-16 z-50 flex justify-center px-4">
  <p class="rounded-full bg-foreground px-4 py-2 text-sm font-bold text-background shadow-float animate-in fade-in-0 slide-in-from-bottom-2">
    Contact saved
  </p>
</div>
```

---

## 7. Layout & Responsive Architecture

### Master-Detail Split Screen (Desktop ≥ 1024px)
- **Header**: Fixed bar with brand wordmark, switcher, search bar, and action pills.
- **Left Pane (List)**: Fixed-width (or flexible percentage, min 320px) with its own vertical scrollbar.
- **Right Pane (Detail)**: Sits on `bg-muted` with independent scroll. Reading column is capped at `max-w-[44rem]` to maintain comfortable line lengths.

### Mobile View (< 1024px)
- Single column layout with full page scroll.
- Sticky header height defined in `--header-h` (typically `7rem` for header + search row).
- List view and Detail view run on separate routes/screens.

---

## 8. Motion & Micro-Interactions

- **Snappy transitions**: Standard duration is `120ms` using `cubic-bezier(0.2, 0.7, 0.2, 1)`.
- **Reduced motion**: Honor accessibility preferences by reducing animations to `1ms`:

```css
@media (prefers-reduced-motion: reduce) {
  *, ::before, ::after {
    animation-duration: 1ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 1ms !important;
    scroll-behavior: auto !important;
  }
}
```

---

## 9. Implementation Checklist for New Apps

When building a new app or page with this design system:

- [ ] Import `Big Shoulders` (Display) and `Work Sans` (Body).
- [ ] Configure `oklch` theme variables with light and dark mode pairs.
- [ ] Ensure **red is used only for positional orientation** (letters, wordmark, 3px active marker).
- [ ] Make all primary buttons **ink pills** (`rounded-full`).
- [ ] Format phone numbers, dates, timestamps, and numbers with `tabular-nums`.
- [ ] Remove box shadows from in-page elements; retain shadows only for floating layers (`shadow-float`, `shadow-overlay`).
- [ ] Use dotted leaders (`.leader`) on wide screens between items and right-aligned meta values.
- [ ] Keep minimum touch targets at 40px or larger.
