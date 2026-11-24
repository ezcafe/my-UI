Result: clean

# Design Review Log: List Brief Length & Detail Hero / Body Photos

## 1. Overview & Verdict
- **Overall Result:** `clean`
- **Isolated Reviews Paid:** `03a-api-contract-review.md` (`clean`)
- **Mode:** full | **Review Profile:** full
- **Critical Issues:** 0
- **Major Issues:** 0
- **Minor Observations / Implementation Directives:** 1 (Hero image error fallback)

The technical design in `03-design.md` comprehensively covers user requirements (Option 2 magazine hero image with title overlaid, original body photos, strictly 3 to 5 sentences briefs, and extractive 3-sentence fallback). The plan in `04-tasks.md` is well-sequenced with clear TDD unit tests.

---

## 2. Review by Dimension

### 2.1 Editorial Magazine Hero Layout (Option 2)
- **Visual Presentation**:
  - Rounded container (`rounded-xl overflow-hidden`) with responsive minimum height (`min-h-[220px] md:min-h-[280px]`).
  - Dark gradient scrim (`bg-gradient-to-t from-black/90 via-black/45 to-transparent`) placed between image and text layer (`relative z-10`).
  - Drop-shadow utilities (`drop-shadow-sm`) provide additional micro-contrast along text edges.
- **Accessibility & Contrast**:
  - Grounded against WCAG AA standards (≥4.5:1 for normal text, ≥3:1 for large text).
  - White text on dark scrim gradient bottom achieves >7:1 contrast ratio regardless of the underlying photo brightness or color distribution.
- **Graceful Absence (Zero Images)**:
  - Clean typography fallback when `hero_image` is `None` renders the title and metadata in standard `text-foreground` without broken image frames or awkward whitespace.

### 2.2 Brief Length Standardization (3 to 5 Sentences)
- **AI Prompt Enforcement**:
  - Explicit phrasing ("strictly 3 to 5 concise sentences. Do not include introductory filler.") added to `format_prompt` and `format_batch_prompt_structured`.
- **Extractive Teaser Fallback**:
  - `miniflux_client::summary::extract_summary(teaser, 3)` extracts 3 sentences, cleanly within the 3–5 sentence requirement.
- **CSS Clamping**:
  - Priority 1 adjusted to `line-clamp-6` and Priority 2 to `line-clamp-4`, avoiding mid-sentence truncation for 3–5 sentence briefs.

### 2.3 Image Extraction & Edge Case Handling
- **Image Filtering**:
  - Filters out 1x1 tracking pixels, feedburner tracker URLs, and data:image/svg svgs.
  - Deduplicates repeated image URLs.
  - First image designated as `hero_image`; subsequent images populate `body_images`.
- **Lazy Loading & Security**:
  - `loading="lazy"` prevents unnecessary network bandwidth on hidden body photos.
  - `referrerpolicy="no-referrer"` ensures user privacy when fetching images from external domains.

---

## 3. Implementation Directive for TDD Build

> [!NOTE] Hero Image Error Fallback Directive
> In `crates/ui-components/templates/partials/entry_detail.html`:
> For body photos (`body_images`), `onerror="this.parentElement.style.display='none'"` is correct because hiding the whole `<figure>` card removes the broken image frame.
> For the hero photo (`hero_image`), however, the parent container houses the overlaid `<h1>` article title and metadata. Therefore, the hero `<img>` tag must use:
> ```html
> onerror="this.style.display='none'"
> ```
> This hides only the broken image, while the parent container and background gradient gracefully retain the article title and metadata in high contrast.

---

## 4. Conclusion
Both `03a-api-contract-review.md` and `03a-design-review-log.md` are `clean`. The technical design is validated and ready for Gate B.
