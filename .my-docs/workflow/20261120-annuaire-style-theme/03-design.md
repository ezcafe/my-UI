# Design: Annuaire Theme Application

- **Work slug:** `20261120-annuaire-style-theme`
- **Mode:** simple
- **Has UI / API / DB / auth:** yes | no | no | no
- **Result:** done

---

## 1. Executive Summary & Settled Decisions

This design transitions the RSS reader UI from dark AR/spatial glass styling to the **Annuaire Design System**. The new visual language delivers an editorial Swiss directory aesthetic characterized by high information density, strict functional color roles, zero page shadows, dual light/dark OKLCH palettes, and tabular numerals.

### Settled Frontier Decisions (from Grill Round 1)
- **Q1 (Entry Row Layout):** **Option A** — Relative timestamp connected by an Annuaire Swiss dotted leader (`.leader`) on wider screens (`sm:flex`), with tabular numerals (`tabular-nums`).
- **Q2 (Theme Toggle Behavior):** **Option A** — 2-state direct toggle button (Sun/Moon icon in header) with initial OS auto-detection (`prefers-color-scheme`) and `localStorage` persistence.
- **Q3 (Login Screen Composition):** **Option A** — Dual-panel Swiss layout featuring a left brand panel in rich Annuaire red (`--brand-surface`: `#D8261D`) with Big Shoulders wordmark, paired with a right login card.

Primary design sources for verification:
- [ANNUAIRE_STYLE_GUIDE.md](file:///Users/ptquang86/ws/my-ui/ANNUAIRE_STYLE_GUIDE.md)
- [annuaire-theme.css](file:///Users/ptquang86/ws/my-ui/annuaire-theme.css)

---

## 2. Architectural Approaches & Options

We evaluated two viable approaches for integrating Annuaire tokens and CSS styles into the Askama template architecture.

### Option A (Recommended): Direct Inline CSS & Utility Classes in `base.html`

- **Explanation**:
  Embed the Annuaire OKLCH color variables, dark theme overrides, and essential utility classes (`.font-display`, `.font-sans`, `.tabular`, `.leader`, `.field`, `.shadow-float`, `.shadow-overlay`) directly inside a `<style>` block in `crates/ui-components/templates/base.html`.
- **Concrete Example**:
  ```html
  <!-- crates/ui-components/templates/base.html -->
  <style>
    :root {
      --radius: 0.625rem;
      --background: oklch(1 0 0);
      --foreground: oklch(0.21 0.008 274.5);
      --brand: oklch(0.569 0.213 29);
      --primary: oklch(0.21 0.008 274.5);
      /* ... other light tokens ... */
    }
    [data-theme="dark"], :root.dark {
      --background: oklch(0.187 0.006 271.1);
      --foreground: oklch(0.943 0 0);
      /* ... other dark tokens ... */
    }
    .leader { flex: 1; border-bottom: 2px dotted var(--leader); margin: 0 0.5rem; height: 1px; }
    .tabular { font-variant-numeric: tabular-nums lining-nums; }
  </style>
  ```
- **Pros**:
  - **Zero build step**: Works out of the box with existing Askama compile-time templates and Tailwind CDN script.
  - **Self-contained tests**: All unit tests in `crates/ui-components` render complete markup and styles without relying on external file serving.
  - **Zero network roundtrip**: Eliminates an additional HTTP request for an external stylesheet on initial page load.
- **Cons**:
  - Adds ~150 lines of CSS directly to `base.html`.

### Option B: External Static Asset (`/static/css/annuaire-theme.css`)

- **Explanation**:
  Copy `annuaire-theme.css` into a dedicated public static asset folder and link it via `<link rel="stylesheet" href="/static/css/annuaire-theme.css">`.
- **Concrete Example**:
  ```html
  <link rel="stylesheet" href="/static/css/annuaire-theme.css" />
  ```
- **Pros**:
  - Clean separation between CSS and HTML templates.
  - Browser can cache the stylesheet across multiple sessions.
- **Cons**:
  - Requires updating the Axum web server routing to register and serve static file directories.
  - Isolated template unit tests in `crates/ui-components` cannot inspect or verify token declarations without spinning up a live HTTP server.
  - Adds potential caching invalidation complexity during rapid UI iteration.

➡️ **Recommendation: Option A (Direct Inline CSS & Utility Classes in `base.html`).** It maintains the zero-build architectural contract, keeps the template crate fully self-contained, and allows direct unit testing of rendered style tags.

---

## 3. The 6 Golden Rules Adherence

1. **Red marks position, never action**:
   - The brand red (`--brand: oklch(0.569 0.213 29)` / `#D8261D`) is strictly reserved for:
     - Header wordmark ("ANNUAIRE RSS").
     - The 3px vertical active indicator bar (`w-[3px] rounded-full bg-brand`) on the selected entry row.
     - The brand identity panel on the login page (`--brand-surface`).
   - Red is strictly forbidden on buttons, links, or action triggers.
2. **Primary action is an Ink pill**:
   - Primary buttons (login button, pagination buttons, primary filters) are pills (`rounded-full`) in high-contrast solid ink (`bg-primary text-primary-foreground`).
3. **Destructive actions are isolated**:
   - Error callouts and retry triggers use isolated neutral cards with warning icons and clear text labels.
4. **Two typefaces with strict roles**:
   - **Display (`Big Shoulders Display 700/800`)**: Wordmark, screen title, section headers.
   - **Body (`Work Sans 400/500/700`)**: Article titles, body prose, inputs, dates, metadata.
5. **Zero page shadows**:
   - Cards, lists, rows, and headers sit on the page with zero box shadows (`shadow: none`). Contrast is produced using subtle background shifts (`bg-background` vs `bg-muted` vs `bg-card`) and 1px borders (`border-border`).
   - Shadows are restricted strictly to floating overlays: `.shadow-float` (toasts) and `.shadow-overlay` (modal dialogs).
6. **Numbers are tabular**:
   - Dates, unread counters, reading times, and pagination numbers use monospace tabular numerals (`tabular-nums lining-nums` / `.tabular`).

---

## 4. Design Tokens & Color Palette

All tokens are defined in OKLCH with exact Hex values, supporting automatic light and dark mode switching via `data-theme` on `<html>`:

```css
:root {
  --radius: 0.625rem;
  color-scheme: light;

  /* Surfaces & Text */
  --background: oklch(1 0 0);                    /* #FFFFFF */
  --foreground: oklch(0.21 0.008 274.5);          /* #17181C */
  --card: oklch(1 0 0);                          /* #FFFFFF */
  --card-foreground: oklch(0.21 0.008 274.5);    /* #17181C */
  --muted: oklch(0.967 0 0);                     /* #F4F4F3 */
  --muted-foreground: oklch(0.493 0.011 273.2);  /* #5F6168 */
  --border: oklch(0.918 0.003 106.5);            /* #E4E4E2 */
  --input: oklch(0.587 0.009 271.3);             /* #7A7C82 */
  --ring: oklch(0.21 0.008 274.5);               /* #17181C */

  /* Actions & Brand */
  --primary: oklch(0.21 0.008 274.5);            /* #17181C */
  --primary-foreground: oklch(1 0 0);            /* #FFFFFF */
  --brand: oklch(0.569 0.213 29);                /* #D8261D - Positional red */
  --brand-foreground: oklch(1 0 0);              /* #FFFFFF */
  --brand-surface: oklch(0.569 0.213 29);        /* #D8261D */
  --brand-surface-foreground: oklch(1 0 0);      /* #FFFFFF */
  --selection: oklch(0.954 0.02 25.2);           /* #FDEBE9 */
  --selection-foreground: oklch(0.21 0.008 274.5);/* #17181C */
  --destructive: oklch(0.501 0.178 28.7);        /* #B3261E */
  --destructive-foreground: oklch(1 0 0);        /* #FFFFFF */
  --leader: oklch(0.789 0.006 274.9);            /* #B9BABE */
  --highlight: oklch(0.903 0.05 25.7);           /* #FFD3CE */

  /* Neutral Avatar Tints */
  --avatar-tint-1: #e8d5b5;
  --avatar-tint-2: #cfe0c8;
  --avatar-tint-3: #c9dbee;
  --avatar-tint-4: #dccfea;
  --avatar-tint-5: #c4e2de;
  --avatar-tint-6: #ebd9a6;

  /* Floating Elevation */
  --elevation-float: 0 0 0 1px transparent, 0 1px 2px oklch(0.2 0.006 270 / 0.06), 0 8px 24px -6px oklch(0.2 0.006 270 / 0.16);
  --elevation-overlay: 0 0 0 1px transparent, 0 2px 4px oklch(0.2 0.006 270 / 0.06), 0 24px 48px -12px oklch(0.2 0.006 270 / 0.28);
}

[data-theme="dark"], :root.dark {
  color-scheme: dark;

  /* Surfaces & Text */
  --background: oklch(0.187 0.006 271.1);        /* #121316 */
  --foreground: oklch(0.943 0 0);                /* #ECECEC */
  --card: oklch(0.257 0.009 276.7);              /* #222328 */
  --card-foreground: oklch(0.943 0 0);          /* #ECECEC */
  --muted: oklch(0.227 0.008 274.6);             /* #1B1C20 */
  --muted-foreground: oklch(0.719 0.01 273.3);   /* #A2A4AB */
  --border: oklch(0.298 0.009 276.8);            /* #2C2D32 */
  --input: oklch(0.573 0.011 273.2);             /* #76787F */
  --ring: oklch(0.943 0 0);                      /* #ECECEC */

  /* Actions & Brand */
  --primary: oklch(0.943 0 0);                   /* #ECECEC */
  --primary-foreground: oklch(0.21 0.008 274.5); /* #17181C */
  --brand: oklch(0.644 0.204 27.8);              /* #F04A40 */
  --brand-foreground: oklch(0.154 0.04 31.1);    /* #1A0503 */
  --brand-surface: oklch(0.462 0.167 28.7);      /* #A11F18 */
  --brand-surface-foreground: oklch(1 0 0);      /* #FFFFFF */
  --selection: oklch(0.266 0.049 22.8);          /* #3A1B1A */
  --selection-foreground: oklch(0.925 0.022 24.4);/* #F5E1DF */
  --destructive: oklch(0.71 0.183 26.6);         /* #FF6B61 */
  --destructive-foreground: oklch(0.154 0.04 31.1);/* #1A0503 */
  --leader: oklch(0.417 0.01 271.2);             /* #4A4C52 */
  --highlight: oklch(0.397 0.118 27.8);          /* #7A2620 */

  /* Dark Neutral Avatar Tints */
  --avatar-tint-1: #4a3f2c;
  --avatar-tint-2: #34452f;
  --avatar-tint-3: #2c3f55;
  --avatar-tint-4: #43365a;
  --avatar-tint-5: #2a4744;
  --avatar-tint-6: #4a4228;

  /* Floating Elevation */
  --elevation-float: 0 0 0 1px oklch(1 0 0 / 0.07), 0 1px 2px transparent, 0 8px 24px -6px oklch(0 0 0 / 0.55);
  --elevation-overlay: 0 0 0 1px oklch(1 0 0 / 0.09), 0 2px 4px transparent, 0 24px 48px -12px oklch(0 0 0 / 0.65);
}
```

---

## 5. Typography Scale & Font Loading

Zero-build CDN loading via Google Fonts in `<head>`:
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Big+Shoulders+Display:wght@700;800;900&family=Work+Sans:ital,wght@0,400;0,500;0,700;1,400&display=swap" rel="stylesheet">
```

### Font Utility Classes
- `.font-display`: `"Big Shoulders Display", "Big Shoulders", ui-sans-serif, sans-serif;`
- `.font-sans`: `"Work Sans", ui-sans-serif, system-ui, sans-serif;`
- `.tabular`: `font-variant-numeric: tabular-nums lining-nums;`

---

## 6. Component Mapping & Layout Architecture

### 6.1 Shell & Container Queries
Container queries are preserved without hardcoded breakpoints:
```css
.annuaire-shell {
  container-type: inline-size;
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: clamp(0.5rem, 2vw, 2rem);
  background-color: var(--background);
  color: var(--foreground);
}

.annuaire-window {
  width: 100%;
  max-width: min(100cqi, 1440px);
  height: min(90vh, 840px);
  display: flex;
  flex-direction: column;
  background-color: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
```

### 6.2 Header Component (`pages/index.html`)
- **Wordmark**: `font-display text-xl font-bold uppercase tracking-wider text-brand`.
- **Unread Counter Pill**: `rounded-full px-2.5 py-0.5 text-xs font-bold tabular bg-muted text-muted-foreground border border-border`.
- **Theme Toggle Button**: Ink pill / ghost circle with Sun (in dark mode) or Moon (in light mode) icon, calling `toggleTheme()`.
- **Action Icons**: Mark all read (`#mark-page-read-btn`) and Logout (`/auth/logout`) styled with subtle ghost borders and hover states.

### 6.3 Filter Bar & Hybrid Search (`pages/index.html`)
- **Filter Tabs**: Pill buttons (`rounded-full`) in a horizontal flex container. Active tab uses `bg-primary text-primary-foreground font-bold`, inactive tabs use `bg-muted text-muted-foreground hover:bg-border`.
- **Search Field**: Annuaire `.field` recipe with ink focus ring:
  ```html
  <div class="relative flex items-center">
    <input type="search" id="search-input" name="search" placeholder="Filter articles..."
           class="field rounded-full border border-input bg-card pl-3.5 pr-8 py-1.5 text-xs text-foreground placeholder:text-muted-foreground w-48" />
    <button type="submit" class="field-button-hit-area absolute right-2.5 text-muted-foreground hover:text-foreground">
      <i data-lucide="search" class="w-3.5 h-3.5"></i>
    </button>
  </div>
  ```

### 6.4 Directory List Item (`partials/entry_item.html`)
- Follows Annuaire Recipe 6.3 and Settled Decision Q1:
  - 3px positional red bar for active state (`w-[3px] bg-brand rounded-full`).
  - Unread indicator dot when `!entry.is_read` (`title="Unread"`).
  - Title in `font-sans font-medium text-sm text-foreground`.
  - Dotted leader (`.leader hidden sm:block`).
  - Relative timestamp (`text-xs text-muted-foreground tabular hidden sm:block`).
  - Action buttons: external link and chevron.

### 6.5 Detail Drawer & Skeleton (`partials/entry_detail.html` & `openDetailWithSkeleton`)
- Sits on `bg-muted` with clean `1px border-l border-border`, zero box shadow.
- Header has Ink pill Back button (`rounded-full bg-primary text-primary-foreground px-3 py-1.5 text-xs font-bold`).
- Tabular date and feed name at the top.
- Executive Brief Card styled in crisp neutral border and muted fill (`bg-card border border-border rounded-lg p-4`).
- Prose content styled with `Work Sans`, comfortable line length (`max-w-[44rem]`), and tabular figures.

### 6.6 Swiss Red Brand Panel Login Screen (`pages/login.html`)
- Follows Settled Decision Q3:
  - Dual-panel layout (`min-h-screen grid grid-cols-1 md:grid-cols-2`).
  - Left brand panel: deep red `--brand-surface` with `Big Shoulders` wordmark, directory tagline, and Swiss geometric purity.
  - Right login card: clean card on `--card` with high-contrast ink pill button (`bg-primary text-primary-foreground`) for Pocket-ID authentication.

### 6.7 Floating Toasts & Error States (`partials/toast.html`, `partials/error_card.html`)
- **Toast**: Floats at the bottom center with `.shadow-float`, `rounded-full bg-foreground text-background px-4 py-2 text-xs font-bold`.
- **Error Card**: High-contrast bordered card with clear retry pill button (`rounded-full bg-primary text-primary-foreground`).

---

## 7. Test Alignment Plan

To ensure 100% of tests pass without regression:
1. `test_render_title_only_entry_item`: Asserts title, `line-clamp-2`, `text-sm font-medium`, external link attributes, `title="Unread"`, and `#unread-pill`. All maintained.
2. `test_filter_bar_reordering`: Sequence (Unread < Today < Starred < All Titles) preserved. Update active tab class assertions to match Annuaire active pill tokens (`bg-primary text-primary-foreground`).
3. `test_spatial_window_fluid_width_container_query`: Container query assertions (`container-type: inline-size;`, `max-width: min(100cqi, 1440px);`) fully satisfied.
4. `test_index_template_skeleton_structure`: `animate-pulse`, `onclick="closeDetailPanel()"`, and skeleton layout matched.
5. `test_annuaire_theme_tokens_and_toggle_present`: New unit test asserting that `base.html` includes Annuaire OKLCH tokens, Google Fonts links, and theme toggle JavaScript.
6. All 16 other tests continue to pass unchanged.
