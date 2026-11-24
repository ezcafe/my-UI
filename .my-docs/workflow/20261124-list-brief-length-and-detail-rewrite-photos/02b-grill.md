# Grill: Stress-testing Decisions

**Result:** frontier-empty

---

## 1. Settled Decisions

- **Brief Length Specification**:
  - Settled: All item briefs and top executive brief on the list page must have length strictly 3 to 5 sentences.
  - LLM prompt guidelines (`format_prompt` and `format_batch_prompt_structured`) will explicitly specify "strictly 3 to 5 sentences".
  - Extractive summary fallback (`extract_summary`) will extract 3 sentences, ensuring consistent fallback length.
  - Magazine card line-clamping will be adjusted to allow 3 to 5 sentences to render smoothly without truncated endings.

- **Detail Page Editorial Layout (Option 2)**:
  - Settled: As confirmed by the user ("Option 2. Hero image at top with content title inside the hero image, same as attached photo"), the detail view will feature:
    1. A magazine hero image section at the top of the detail panel.
    2. The article title and metadata (feed title, publication date, author) overlaid inside the hero image with a CSS readability gradient scrim.
    3. If an article contains no images, a clean typography header renders without an image frame or layout jump.
    4. The AI inverted pyramid rewrite rendered cleanly below the hero.
    5. Any remaining original photos from the article rendered beneath the rewrite as responsive figures.

- **Image Extraction & Error Handling**:
  - Settled: Parse HTML content to extract all valid `<img>` source URLs.
  - Filter out tracking pixels / blank gifs.
  - Separate into first image (hero) and remaining images (body).
  - All images will use `loading="lazy"`, `referrerpolicy="no-referrer"`, and `onerror="this.style.display='none'"` to ensure dead links fail silently without broken image icons.

---

## 2. Frontier

- All architectural and UX decisions are resolved and confirmed by the user.
- **Grill status:** `frontier-empty` (Grill closed).
