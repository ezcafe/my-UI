# Gate A: Framing Review

**Result:** ok

---

## 1. Scope and Clarity Review

- **List Page Briefs (3 to 5 sentences)**:
  - Update AI prompt generation (`format_prompt`, `format_batch_prompt_structured`) and fallback extraction (`miniflux_client::summary::extract_summary`) to strictly enforce a length of 3 to 5 sentences.
  - Review and adjust line-clamping and card heights across list view templates (`entry_item.html`, `entry_list.html`, `executive_brief_card.html`) so that 3 to 5 sentences render smoothly and legibly across viewports without awkward clipping or excessive whitespace.
- **Detail Page Rewrite with Original Photos (Hero Image + Editorial Flow)**:
  - Extract original photos (`<img>` tags, src, alt, caption) from the original article HTML content (`content` field from Miniflux).
  - Implement Option 2 (confirmed by user): A magazine hero image section at the top of the detail page with the content title overlaid inside the hero image with a CSS readability gradient (scrim), accompanied by metadata (feed title, publication date, author).
  - Graceful fallback: If an article contains no images, render the clean typography title header without broken image placeholders or layout shifts.
  - Render the AI inverted pyramid rewrite below the hero image.
  - Render remaining original article photos beneath the rewrite as clean, responsive figures with captions (if present).
  - Ensure image URLs are sanitized, lazy-loaded, and responsive with proper aspect-ratio preservation and error handling (`onerror="this.style.display='none'"`).

## 2. Verdict

- Scope is clear, bounded, directly confirmed by user ("Option 2. Hero image at top with content title inside the hero image, same as attached photo").
- Framing addresses both list page brief consistency and detail page multimedia preservation.
- **Gate A status:** `ok`
