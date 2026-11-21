# Grill: Annuaire Theme Frontier Decisions

- **Work slug:** `20261120-annuaire-style-theme`
- **Mode:** simple
- **Has UI / API / DB / auth:** yes | no | no | no
- **Result:** frontier-empty

---

## Settled Decisions (Round 1)

- **Q1 (Entry Row Layout):** **Option A** — Relative timestamp connected by an Annuaire Swiss dotted leader (`.leader`) on wider screens (`sm:flex`), with tabular numerals (`tabular-nums`).
- **Q2 (Theme Toggle Behavior):** **Option A** — 2-state direct toggle button (Sun/Moon icon in header) with initial OS auto-detection (`prefers-color-scheme`) and `localStorage` persistence.
- **Q3 (Login Screen Composition):** **Option A** — Dual-panel Swiss layout featuring a left brand panel in rich Annuaire red (`--brand-surface`: `#D8261D`) with Big Shoulders wordmark, paired with a right login card.

---

## Frontier Round 1

Please review the three design and interaction decisions below. Each includes concrete options, pros, cons, and a recommended choice.

---

### ❓ **Q1** - **Entry Row Layout**: Should list items include a relative timestamp connected by an Annuaire Swiss dotted leader (`.leader`) on wider screens?

**Explanation**:
The signature visual pattern of Annuaire is the dotted-leader row (Rule 6.3): connecting the primary title to right-aligned numbers or timestamps with repeating radial dots. Currently, the list items are strictly title-only (`title + external-link + chevron`). Adding the timestamp with a dotted leader brings the authentic Swiss directory look while preserving high density.

#### Option A (Recommended): Add relative timestamp with dotted leader
- **Snippet**:
  ```html
  <div class="ar-list-item flex items-center justify-between gap-3 px-3 py-2">
    <div class="flex items-center gap-2.5 flex-1 min-w-0">
      <span class="w-[3px] h-4 rounded-full bg-brand ..."></span>
      <span class="text-name truncate">{{ entry.title }}</span>
      <span class="leader hidden sm:block"></span>
      <span class="text-xs text-muted-foreground tabular shrink-0 hidden sm:block">{{ entry.published_at }}</span>
    </div>
    <!-- Actions -->
  </div>
  ```
- **Pros**:
  - Implements the core Annuaire visual identity (Rule 6.3).
  - Makes scanning articles faster by displaying publication freshness at a glance.
  - Responsive: collapses cleanly on narrow mobile viewports.
- **Cons**:
  - Requires passing `published_at` in the row layout.

#### Option B: Strictly title-only without timestamp
- **Snippet**:
  ```html
  <div class="ar-list-item flex items-center justify-between gap-3 px-3 py-2">
    <span class="text-name truncate flex-1">{{ entry.title }}</span>
    <span class="leader hidden sm:block"></span>
    <!-- Actions -->
  </div>
  ```
- **Pros**:
  - Maximum brevity for titles.
- **Cons**:
  - Misses the classic directory feel where dotted leaders connect to tabular data.

➡️ **Recommendation: Option A (Add relative timestamp with dotted leader on `sm:` screens). Answering "yes" selects Option A.**

---

### ❓ **Q2** - **Theme Toggle Behavior**: Should the header theme switcher be a 2-state direct toggle (Light / Dark) with initial OS auto-detection?

**Explanation**:
We agreed in ideation to support both Light and Dark modes with auto-detect and a header toggle. We need to decide how the toggle button behaves when clicked.

#### Option A (Recommended): 2-State Direct Toggle (Light / Dark) with automatic initial detection
- **Snippet**:
  ```javascript
  // On load: check localStorage, fallback to OS preference
  const saved = localStorage.getItem('theme');
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  const current = saved || (prefersDark ? 'dark' : 'light');
  document.documentElement.setAttribute('data-theme', current);

  // On click: flip between light and dark
  function toggleTheme() {
    const next = document.documentElement.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    localStorage.setItem('theme', next);
  }
  ```
- **Pros**:
  - Zero-friction: a single click flips mode immediately.
  - Clean header icon: displays Sun icon in dark mode, Moon icon in light mode.
  - Persists seamlessly in `localStorage` across page reloads.
- **Cons**:
  - Resetting back to "follow OS" requires clearing browser storage.

#### Option B: 3-State Dropdown / Cycle (System / Light / Dark)
- **Snippet**:
  ```html
  <select onchange="setTheme(this.value)">
    <option value="auto">System</option>
    <option value="light">Light</option>
    <option value="dark">Dark</option>
  </select>
  ```
- **Pros**:
  - Explicit option to re-enable OS automatic mode at any time.
- **Cons**:
  - Requires extra clicks or a bulky dropdown in an already compact Swiss header.

➡️ **Recommendation: Option A (2-State Direct Toggle with Sun/Moon icon and initial OS auto-detection). Answering "yes" selects Option A.**

---

### ❓ **Q3** - **Login Screen Composition**: Should the login page use the signature Annuaire dual-panel layout with a rich red brand panel?

**Explanation**:
Rule 1 of Annuaire permits red exclusively for positional orientation, wordmarks, active markers, and the **brand login panel** (`--brand-surface`: `#D8261D`). Currently, the login page is a centered dark glass box.

#### Option A (Recommended): Dual-panel Swiss layout
- **Snippet**:
  ```html
  <div class="grid grid-cols-1 md:grid-cols-2 min-h-screen">
    <!-- Left: Brand Surface Panel -->
    <div class="bg-brand-surface text-brand-surface-foreground p-8 flex flex-col justify-between">
      <h1 class="font-display text-wordmark font-bold uppercase tracking-wider">Annuaire</h1>
      <p class="font-sans text-body opacity-90">Curated feeds in an editorial Swiss directory interface.</p>
    </div>
    <!-- Right: Login Card -->
    <div class="bg-background flex items-center justify-center p-8">
      <a href="/auth/login" class="rounded-full bg-primary text-primary-foreground font-bold px-6 py-3">Sign in with Pocket-ID</a>
    </div>
  </div>
  ```
- **Pros**:
  - Perfectly matches Annuaire Style Guide section 1 & 2 ("brand login panel").
  - Striking, premium editorial Swiss aesthetic.
  - Full responsive layout: stacks into a brand header banner on mobile.
- **Cons**:
  - Slightly larger visual footprint on desktop than a small modal box.

#### Option B: Single centered card with red wordmark
- **Snippet**:
  ```html
  <div class="min-h-screen flex items-center justify-center bg-background p-4">
    <div class="border border-border rounded-xl p-8 max-w-sm text-center">
      <h1 class="font-display text-title font-bold text-brand mb-2">Annuaire</h1>
      <!-- Login Button -->
    </div>
  </div>
  ```
- **Pros**:
  - Minimalist and compact.
- **Cons**:
  - Does not showcase the full editorial brand character defined in the style guide.

➡️ **Recommendation: Option A (Dual-panel Swiss layout with brand panel). Answering "yes" selects Option A.**
