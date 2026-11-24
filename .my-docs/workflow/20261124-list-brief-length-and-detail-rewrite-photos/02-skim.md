# Codebase Skim

**Result:** done

---

## 1. Relevant Code Paths and Locations

### List Page Briefs (3 to 5 Sentences)
- **`crates/miniflux/src/summary.rs`**:
  - `extract_summary(content: &str, sentences_count: usize)`: Extracts the first `sentences_count` sentences using `unicode_segmentation`.
  - Used as fallback when AI is disabled or summaries are uncached.
- **`crates/server/src/ai/extractor.rs`**:
  - Lines 43, 54: Calls `miniflux_client::summary::extract_summary(teaser, 2)`.
  - Needs update to 3 sentences to satisfy the 3 to 5 sentence requirement.
- **`crates/server/src/ai/llm.rs`**:
  - `format_prompt` (lines 75–81): Currently says "Summarize what changed or why this news matters in strictly 3 concise sentences." Needs to enforce strictly 3 to 5 sentences.
  - `format_batch_prompt_structured` (lines 83–99): "top" executive brief prompt synthesizing the page themes. Needs to ensure executive brief produces 3 to 5 sentences.
  - Fallback extraction calls in `llm.rs` (lines 614, 705, 719, 745, 761): Currently use `extract_summary(..., 3)`.
- **`crates/ui-components/templates/partials/entry_list.html`**:
  - Magazine view Priority 1 card has `line-clamp-4`. If a brief is 5 sentences, 4 lines can clamp abruptly.
  - Magazine view Priority 2 card has `line-clamp-3`.
  - Executive brief cards (`executive_brief_card.html`, `entry_list.html`) display the top executive brief cleanly with `leading-relaxed text-xs`.

### Detail Page Rewrite with Photos (Option 2: Editorial Hero Image + Body Photos)
- **`crates/server/src/routes/entries.rs`**:
  - `extract_first_image_url(html: &str) -> Option<String>` (lines 41–51): Uses regex to find `<img[^>]+src=["']([^"']+)["']`.
  - `get_entry_detail` (lines 1250–1400): Fetches Miniflux entry with `content` HTML. However, line 1366 hardcodes `image_url: None`, discarding all photos from the article.
  - Needs an image extraction function (`extract_article_images(html: &str) -> Vec<String>` or similar) that separates the hero image (first image) from body images (remaining photos).
- **`crates/ui-components/src/templates.rs`**:
  - `EntryDetailTemplate` (lines 106–115): Holds `entry: EntryViewModel`, `summary`, `rewrite`, `impact`, `fallacies`, `has_analysis`, `auth_enabled`.
  - Can be extended to hold `hero_image: Option<String>` and `body_images: Vec<String>` (or use `entry.image_url` for hero and a dedicated `body_images: Vec<String>` field).
- **`crates/ui-components/templates/partials/entry_detail.html`**:
  - Lines 68–77: Currently renders title in a plain `<h1>` followed by metadata.
  - Lines 79–84: Renders `#ai-rewrite-box` with only `{{ rewrite }}` text.
  - Needs Option 2 layout:
    - If `hero_image` exists: Render full-width / rounded magazine hero container with the image, a gradient scrim (`linear-gradient(to top, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0.4) 50%, transparent 100%)`), and the title + metadata overlaid inside the hero at the bottom.
    - If `hero_image` is None: Render the clean typography header (title + metadata) without broken image placeholders.
    - In `#ai-rewrite-box`: Render `{{ rewrite }}` inverted pyramid text, followed by any remaining `body_images` in responsive editorial figure blocks.
- **`crates/ui-components/templates/pages/index.html`**:
  - `openDetailWithSkeleton`: When opening an article before HTMX loads, renders placeholder skeletons. Needs to cleanly support the hero structure when available without layout flash.
