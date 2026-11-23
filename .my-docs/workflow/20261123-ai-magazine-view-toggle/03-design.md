# System Design: AI Magazine View Toggle & Hot Topic Highlighting

**Result:** done

## 1. Executive Summary & Core Milestones

This design introduces an editorial **AI Magazine View** that coexists with the classic high-density list view, allowing users to toggle between them effortlessly with zero latency.

- **Milestone 1 (Header AI Toggle Button)**: Add an AI button immediately after `theme-toggle-btn`. It toggles between the AI view and list view, persists user preference in `localStorage` (`my_news_view_mode`), and defaults to the AI view on initial load.
- **Milestone 2 (Data Model & Photo/Topic Extraction)**: Extend `EntryViewModel` with `category_title: Option<String>` and `image_url: Option<String>`. Extract the first valid image URL from article HTML content, and include the feed category title.
- **Milestone 3 (Magazine Grid Template & Hot Topic Highlighting)**: In `entry_list.html`, render the magazine layout alongside the list view. Display cards in a responsive grid (`grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4`) with photos (when available), briefs, and a prominent "Hot Topic" badge for the category of the most-clicked source feed.
- **Milestone 4 (Interactive Detail Navigation & Read Color Transition)**: Wire title clicks to `openEntryDetail(id)` to open the slide-over detail panel. When read, the title transitions to grey (`text-muted-foreground`), keeping the article card on the page. Increment the feed's click count in client storage to dynamically reflect hot topic interest.
- **Milestone 5 (Shared Bottom Pagination)**: Keep the bottom pagination navigation (Previous / Page X of Y / Next) visible and functional across both views.

---

## 2. Design Options & Trade-Offs

### Option 1: Dual-View Template with Client-Side Instant Toggle (Recommended)

- **Explanation**: `EntryListTemplate` renders both the classic list view (`#entries-list-view`) and the magazine view (`#entries-magazine-view`). Both sections use the exact same paginated entries from the server. A CSS/JS toggle shows one and hides the other instantaneously based on the active mode. The bottom pagination bar is placed below both views so it remains identical in position and function.
- **Concrete Example**:
  ```html
  <!-- Header Toggle -->
  <button id="ai-toggle-btn" type="button" onclick="toggleAiView()" 
          class="p-2 rounded-full border border-border bg-card hover:bg-muted text-foreground transition-colors" 
          title="Toggle AI Magazine View">
    <i data-lucide="sparkles" class="w-4 h-4 text-brand"></i>
  </button>
  ```
  ```javascript
  function toggleAiView() {
    const current = localStorage.getItem('my_news_view_mode') || 'ai';
    const next = current === 'ai' ? 'list' : 'ai';
    localStorage.setItem('my_news_view_mode', next);
    applyAiViewMode(next);
  }
  ```
- **Pros**:
  - Zero-latency switching (no spinner, no network roundtrip when flipping views).
  - Both views share identical pagination state, search queries, and status filters.
  - Read states sync between views immediately without duplicate server calls.
- **Cons**:
  - Slightly larger HTML payload per page load because both view containers are in the DOM (minimal impact: ~5-10 KB per 50 items).

### Option 2: Server-Side Query Parameter (`?view=magazine`) with HTMX Swaps

- **Explanation**: The AI toggle modifies the HTMX URL query parameter to fetch either list HTML or magazine HTML from the server on every toggle.
- **Pros**:
  - Smaller HTML payload per response since only one representation is returned.
- **Cons**:
  - Network delay on every view toggle.
  - Potential flicker and loss of client-side scroll position during toggling.
  - Requires maintaining separate server route endpoints or view parameters across all filter buttons.

**Recommendation**: Option 1 is strongly recommended for instant UX responsiveness and seamless state continuity.

---

## 3. Detailed Architecture & Implementation

### 3.1 Model Changes (`crates/ui-components/src/templates.rs`)

Update `EntryViewModel`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryViewModel {
    pub id: i64,
    pub title: String,
    pub suggested_title: Option<String>,
    pub feed_title: String,
    #[serde(default)]
    pub category_title: Option<String>,
    pub published_at: String,
    pub reading_time: i32,
    pub summary: String,
    #[serde(default)]
    pub image_url: Option<String>,
    pub url: String,
    pub author: Option<String>,
    pub is_read: bool,
    pub is_bookmarked: bool,
}
```

### 3.2 Image & Category Extraction Helper

In `crates/server/src/routes/entries.rs` and `pages.rs`:
Extract first image URL from HTML content:
```rust
pub fn extract_first_image_url(html: &str) -> Option<String> {
    let re = regex::Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).ok()?;
    re.captures(html).and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}
```
And populate `category_title` from `item.feed.category.title` (or "News" / "General").

### 3.3 Magazine View Layout (`crates/ui-components/templates/partials/entry_list.html`)

```html
<div id="entries-container">
  <!-- Classic List View -->
  <div id="entries-list-view" class="space-y-1.5">
    {% for entry in entries %}
      ... existing list rows ...
    {% endfor %}
  </div>

  <!-- AI Magazine View -->
  <div id="entries-magazine-view" class="hidden grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {% for entry in entries %}
    <div class="magazine-card group rounded-lg border border-border bg-card hover:border-foreground/30 transition-colors flex flex-col overflow-hidden" 
         id="mag-entry-{{ entry.id }}" 
         data-entry-id="{{ entry.id }}" 
         data-feed-title="{{ entry.feed_title }}"
         data-category="{% if let Some(cat) = entry.category_title %}{{ cat }}{% else %}General{% endif %}">
      
      {% if let Some(img) = entry.image_url %}
      <div class="h-36 w-full overflow-hidden bg-muted relative shrink-0">
        <img src="{{ img }}" alt="{{ entry.title }}" loading="lazy" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300">
      </div>
      {% endif %}

      <div class="p-4 flex flex-col flex-1">
        <div class="flex items-center justify-between gap-2 mb-2">
          <span class="magazine-topic-badge px-2 py-0.5 rounded-full text-[11px] font-bold uppercase tracking-wider bg-muted text-muted-foreground border border-border">
            {% if let Some(cat) = entry.category_title %}{{ cat }}{% else %}General{% endif %}
          </span>
          <span class="text-xs text-muted-foreground tabular font-sans">{{ entry.published_at }}</span>
        </div>

        <h4 class="magazine-title text-sm font-semibold font-sans leading-snug line-clamp-2 cursor-pointer transition-colors {% if entry.is_read %}text-muted-foreground{% else %}text-foreground group-hover:text-foreground/80{% endif %}"
            id="mag-title-{{ entry.id }}"
            onclick="openEntryFromMagazine({{ entry.id }}, '{{ entry.feed_title }}')">
          {% if let Some(suggested) = entry.suggested_title %}{{ suggested }}{% else %}{{ entry.title }}{% endif %}
        </h4>

        <p class="magazine-brief text-xs text-muted-foreground/90 mt-2 line-clamp-3 leading-relaxed font-sans flex-1">
          {{ entry.summary }}
        </p>

        <div class="flex items-center justify-between mt-3 pt-2.5 border-t border-border/60 text-xs text-muted-foreground">
          <span class="truncate font-medium">{{ entry.feed_title }}</span>
          <a href="{{ entry.url }}" target="_blank" rel="noopener noreferrer" 
             onclick="event.stopPropagation(); recordSourceClick('{{ entry.feed_title }}'); markArticleRead({{ entry.id }})" 
             class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
             title="Open original">
            <i data-lucide="external-link" class="w-3.5 h-3.5"></i>
          </a>
        </div>
      </div>
    </div>
    {% endfor %}
  </div>

  <!-- Shared Pagination -->
  {% if total_pages > 1 %}
    ... pagination nav ...
  {% endif %}
</div>
```

### 3.4 Hot Topic Highlighting Logic (`index.html`)

- Clicks per source are stored in `localStorage.getItem('my_news_source_clicks')` as a JSON map: `{"Source Name": click_count}`.
- Helper `recordSourceClick(feedTitle)`:
  1. Increments count for `feedTitle`.
  2. Identifies the source with highest click count.
  3. Updates `.magazine-topic-badge`: For cards where `data-feed-title === mostClickedFeed`, adds class `hot-topic-badge text-brand border-brand/40 bg-brand/10` and prefixes with an active fire/sparkle indicator.
  4. If no source has been clicked yet, the top feed on the current page is treated as the hot topic by default.

### 3.5 Read Title Color Change

- When an article title is clicked in magazine view:
  ```javascript
  function openEntryFromMagazine(id, feedTitle) {
    recordSourceClick(feedTitle);
    const titleEl = document.getElementById('mag-title-' + id);
    if (titleEl) {
      titleEl.classList.remove('text-foreground');
      titleEl.classList.add('text-muted-foreground');
    }
    openEntryDetail(id);
  }
  ```
- In `syncDomReadStates()`, loop through all `[id^="mag-title-"]`. If `readSet.has(id)`, ensure `text-muted-foreground` is applied and card stays visible.

---

## 4. Annuaire Style Guide Compliance

- **Rule 1 (Red marks position)**: The hot topic badge uses `--brand` red to mark position/relevance, never as a destructive action.
- **Rule 2 (Ink pill action)**: When active, `#ai-toggle-btn` displays high contrast ink styling.
- **Rule 5 (Zero page shadows)**: Magazine cards use crisp 1px borders (`border-border`), no box shadows.
- **Rule 6 (Tabular numbers)**: All dates and reading times in the magazine card use tabular fonts.
