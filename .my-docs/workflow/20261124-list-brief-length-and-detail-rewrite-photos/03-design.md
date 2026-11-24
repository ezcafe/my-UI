# Technical Design: List Brief Length (3 to 5 Sentences) & Detail Rewrite with Hero Image + Original Photos

**Result:** done

---

## 1. Executive Summary

This design delivers two primary enhancements:
1. **List Page Brief Length (Strictly 3 to 5 Sentences)**:
   - Enforce strictly 3 to 5 sentences across AI prompt templates (`format_prompt` and `format_batch_prompt_structured`).
   - Standardize extractive fallbacks (`miniflux_client::summary::extract_summary`) to extract 3 sentences when AI generation is unavailable.
   - Adjust CSS line-clamping in `entry_list.html` (`line-clamp-6` on Priority 1, `line-clamp-4` on Priority 2) so 3 to 5 sentences render smoothly without awkward mid-sentence clipping.
2. **Detail Page Rewrite with Original Photos (Option 2: Editorial Magazine Hero + Body Photos)**:
   - Extract original article photos (`<img>` elements) from Miniflux entry HTML, separating the first photo (Hero image) from subsequent photos (Body images), filtering out tracking pixels.
   - Implement **Option 2** (user confirmed): A magazine hero section at the top of the detail page with the content title and metadata overlaid inside the hero image with a CSS readability gradient scrim.
   - Provide an elegant fallback when an article has no images (clean typography header without broken image frames or layout jumps).
   - Render the AI inverted pyramid rewrite beneath the hero section, followed by responsive figures for remaining original photos.

---

## 2. Decisions, Options, and Trade-offs

### Decision 1: Detail Page Photo Extraction & Layout Architecture

#### Option 1 (Recommended): Structured Separation of Hero Image & Body Images in Template
- **Explanation**:
  - In `crates/server/src/routes/entries.rs`, introduce an image extraction helper `extract_article_images(html: &str) -> (Option<String>, Vec<String>)`.
  - The first valid image URL becomes the `hero_image` (`Option<String>`), and any remaining valid image URLs become `body_images` (`Vec<String>`).
  - Extend `EntryDetailTemplate` in `crates/ui-components/src/templates.rs` with `pub hero_image: Option<String>` and `pub body_images: Vec<String>`.
  - In `crates/ui-components/templates/partials/entry_detail.html`:
    - If `hero_image` is present: Render a rounded magazine hero container with the image, a gradient scrim (`linear-gradient(to top, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.45) 50%, rgba(0,0,0,0.05) 100%)`), and the title + metadata overlaid inside at the bottom.
    - If `hero_image` is absent: Render the clean typography title header in `text-foreground`.
    - Below `rewrite`: If `body_images` is non-empty, render responsive figure cards with `object-cover`, `loading="lazy"`, `referrerpolicy="no-referrer"`, and `onerror="this.parentElement.style.display='none'"`.
- **Concrete Example**:
  ```html
  {% if let Some(hero) = hero_image %}
  <div class="relative w-full rounded-xl overflow-hidden mb-6 bg-muted min-h-[220px] md:min-h-[280px] flex items-end">
    <img src="{{ hero }}" alt="{{ entry.title }}" loading="lazy" referrerpolicy="no-referrer" onerror="this.parentElement.style.display='none'" class="absolute inset-0 w-full h-full object-cover">
    <div class="absolute inset-0 bg-gradient-to-t from-black/90 via-black/45 to-transparent"></div>
    <div class="relative z-10 p-5 md:p-6 w-full">
      <div class="flex items-center gap-2 text-xs text-white/80 mb-2 font-sans tabular drop-shadow-sm">
        <span class="font-medium text-white">{{ entry.feed_title }}</span>
        <span>•</span>
        <span>{{ entry.published_at }}</span>
      </div>
      <h1 class="text-xl md:text-2xl font-bold font-sans text-white leading-snug drop-shadow-sm">
        {% if let Some(suggested) = entry.suggested_title %}{{ suggested }}{% else %}{{ entry.title }}{% endif %}
      </h1>
    </div>
  </div>
  {% else %}
  <!-- Fallback clean typography header -->
  <div class="mb-6">
    <div class="flex items-center gap-2 text-xs text-muted-foreground mb-2 font-sans tabular">
      <span class="font-medium text-foreground">{{ entry.feed_title }}</span>
      <span>•</span>
      <span>{{ entry.published_at }}</span>
    </div>
    <h1 class="text-2xl font-bold font-sans text-foreground leading-snug">
      {% if let Some(suggested) = entry.suggested_title %}{{ suggested }}{% else %}{{ entry.title }}{% endif %}
    </h1>
  </div>
  {% endif %}
  ```
- **Pros**:
  - Exactly implements Option 2 requested by the user.
  - Zero layout jumping; cleanly handles articles with 0, 1, or multiple photos.
  - Strong readability: Dark gradient scrim guarantees WCAG AA contrast (≥4.5:1) regardless of image colors.
  - Error handling: Broken image links fail silently without broken icon outlines.
- **Cons**:
  - Requires adding two fields to `EntryDetailTemplate` and updating related unit test initializations.

#### Option 2: Inject Raw HTML Images Directly into Rewrite Markdown
- **Explanation**:
  - Concatenate `<img>` tags directly into the `rewrite` text string before template rendering.
- **Pros**:
  - Avoids adding fields to `EntryDetailTemplate`.
- **Cons**:
  - Cannot render the hero image with title overlaid on top (breaks Option 2 user requirement).
  - Mixing raw HTML into AI markdown risks XSS or formatting issues if markdown parsers escape tags.

- **Recommendation**: **Option 1**. It cleanly decouples editorial layout from text content, meets user confirmation for Option 2, and enforces robust styling and error handling.

---

### Decision 2: Brief Length Enforcement & Template Line Clamping

#### Option 1 (Recommended): Strict 3 to 5 Sentences in Prompts + 3-Sentence Fallback + Expanded Clamping
- **Explanation**:
  - In `crates/server/src/ai/llm.rs`:
    - `format_prompt`: Update prompt text to "Summarize what changed or why this news matters in strictly 3 to 5 concise sentences. Do not include introductory filler."
    - `format_batch_prompt_structured`: Enforce strictly 3 to 5 sentences for both the top executive brief and item rewrites.
  - In `crates/miniflux/src/summary.rs` and `crates/server/src/ai/extractor.rs`:
    - Ensure default extractive summary sentence count is 3 sentences (e.g. `extract_summary(content, 3)`), which comfortably sits within the 3 to 5 sentence boundary.
  - In `crates/ui-components/templates/partials/entry_list.html`:
    - Priority 1 hero card: Change `line-clamp-4` to `line-clamp-6` so 3 to 5 sentences can be fully read on all screens.
    - Priority 2 card: Change `line-clamp-3` to `line-clamp-4` for balanced visual proportion.
- **Concrete Example**:
  ```rust
  pub fn format_prompt(title: &str, content: &str) -> String {
      format!(
          "Summarize what changed or why this news matters in strictly 3 to 5 concise sentences. Do not include introductory filler.\n\nArticle Title: {}\nArticle Content:\n{}",
          title.trim(),
          content.trim()
      )
  }
  ```
- **Pros**:
  - Directly fulfills the user prompt: "updating briefs of all items on the list page to have length strictly 3 to 5 sentences".
  - Both AI-generated summaries and extractive fallback summaries adhere to the same 3 to 5 sentence standard.
  - Prevents truncated sentence fragments in cards.
- **Cons**:
  - Slightly taller cards on Priority 1 when a brief reaches 5 sentences.

#### Option 2: Rely Only on CSS Height Clamping Without Prompt Changes
- **Explanation**:
  - Keep prompts unchanged and only change CSS.
- **Pros**:
  - Zero Rust backend changes for prompts.
- **Cons**:
  - LLM continues generating 1-2 sentence or overly long summaries; does not solve the prompt requirement.

- **Recommendation**: **Option 1**. It coordinates the prompt instruction, the fallback extractor, and the CSS presentation.

---

## 3. Detailed Component Design

### 3.1 Image Extraction Function
```rust
/// Extracts all valid image URLs from HTML content, filtering out tracking pixels.
pub fn extract_article_images(html: &str) -> (Option<String>, Vec<String>) {
    static IMG_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::RegexBuilder::new(r#"<img[^>]+src=["']([^"']+)["']"#)
            .case_insensitive(true)
            .build()
            .unwrap()
    });

    let mut images = Vec::new();
    for cap in IMG_REGEX.captures_iter(html) {
        if let Some(src) = cap.get(1).map(|m| m.as_str().trim().to_string()) {
            let lower = src.to_lowercase();
            // Filter out common tracking pixels, 1x1 gifs, and empty src
            if !src.is_empty()
                && !lower.contains("1x1")
                && !lower.contains("pixel")
                && !lower.contains("feedburner.com/~r")
                && !lower.starts_with("data:image/svg+xml")
            {
                if !images.contains(&src) {
                    images.push(src);
                }
            }
        }
    }

    let hero_image = if !images.is_empty() {
        Some(images.remove(0))
    } else {
        None
    };

    (hero_image, images)
}
```

### 3.2 Detail Panel Template Changes (`EntryDetailTemplate`)
```rust
#[derive(Template)]
#[template(path = "partials/entry_detail.html")]
pub struct EntryDetailTemplate {
    pub entry: EntryViewModel,
    pub summary: String,
    pub auth_enabled: bool,
    pub rewrite: String,
    pub impact: String,
    pub fallacies: String,
    pub has_analysis: bool,
    pub hero_image: Option<String>,
    pub body_images: Vec<String>,
}
```

### 3.3 List View Clamping (`entry_list.html`)
- Priority 1 card paragraph:
  `class="magazine-brief text-xs text-muted-foreground/90 mt-3 leading-relaxed font-sans line-clamp-6"`
- Priority 2 card paragraph:
  `class="magazine-brief text-xs text-muted-foreground/90 mt-1.5 line-clamp-4 leading-relaxed font-sans flex-1"`

---

## 4. Decision 3: Wide Desktop Details View On Click & Slide-from-Right Animation

### 4.1 Requirement & Problem
- Previously, under `@container workspace (min-width: 58rem)` in `base.html`, the layout forced a two-column split (`grid-template-columns: minmax(20rem, 24rem) 1fr`) where `.detail-pane` was set to `display: flex !important`, automatically rendering `#detail-empty-placeholder` on page load before any item was selected.
- The user requested:
  1. On wide desktop view, show the details page **only when user clicks on a news item**.
  2. Do **not** show details page on load.
  3. Use **slide from right animation** for the details page on load/open.

### 4.2 Solution Architecture
1. **Remove Automatic Split on Load**:
   - In `crates/ui-components/templates/base.html`, remove `grid-template-columns: minmax(20rem, 24rem) 1fr` and `display: flex !important` from `@container workspace (min-width: 58rem)`.
   - The workspace layout remains a single full-width list until an article is selected.
   - The `#detail-empty-placeholder` is no longer shown on initial load.
2. **Slide from Right Animation**:
   - Define a smooth CSS animation in `crates/ui-components/templates/base.html`:
     ```css
     @keyframes slideInFromRight {
       from {
         transform: translateX(100%);
         opacity: 0.9;
       }
       to {
         transform: translateX(0);
         opacity: 1;
       }
     }
     .animate-slide-in-right {
       animation: slideInFromRight 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
     }
     ```
   - In `crates/ui-components/templates/pages/index.html`:
     - When `openEntryDetail(id)` or `openDetailWithSkeleton` opens `#detail-panel`, apply `.animate-slide-in-right`.
     - `#detail-panel` is positioned as `absolute inset-0 z-30 bg-card border-l border-border flex flex-col p-6 overflow-y-auto` (or right-aligned overlay).
3. **Preserve Back Button on All Screens**:
   - In `base.html`, remove `.workspace-layout .detail-pane #detail-back-btn { display: none !important; }`.
   - The back button (`#detail-back-btn`) stays accessible on both mobile and wide desktop, allowing the user to seamlessly dismiss the details panel and return to the full list view.

