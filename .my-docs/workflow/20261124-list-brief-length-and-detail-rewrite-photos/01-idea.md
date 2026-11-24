# 01-idea: List Brief Length (3 to 5 sentences) & Detail Rewrite with Hero Image + Original Photos

## 1. Problem Statement & Motivation
- **List Page Briefs**: Previously, list page article briefs either lacked consistent sentence-count constraints or were clamped too aggressively, leading to truncated or inconsistent reading experiences. The user requested updating briefs of all items on the list page to have length strictly 3 to 5 sentences.
- **Detail Page Rewrite Photos**: When viewing article details, the AI rewrite previously replaced the entire content body, which unintentionally removed all photos from the original article. The user requested that the rewrite on the details page must include the photos from the original content, styled with a magazine hero image at the top containing the title overlaid on the hero image (similar to the provided editorial design reference), and remaining photos rendered in the article body.

## 2. Goals & Confirmation
- **Goal 1 (List Page)**:
  - Update the prompt generation for items on the list page to explicitly require an executive brief of 3 to 5 sentences.
  - Update fallback extraction (`extract_summary`) for when AI is disabled or uncached to extract 3 to 5 sentences (e.g. 3 sentences).
  - Adjust list card line clamping (`line-clamp`) in templates so the 3 to 5 sentence brief renders comfortably across viewports.
- **Goal 2 (Detail Page)**:
  - Extract all photos (`<img>` elements) and captions from the original article HTML content.
  - If a hero photo exists (or first image), render it as an editorial magazine hero section at the top of the detail page, with the article title and metadata (feed title, published date, author) elegantly overlaid inside the hero image with a readability gradient.
  - If no photos exist in the article, render the standard title and metadata header cleanly without a broken image box.
  - Render the AI inverted pyramid rewrite below the hero.
  - Render any additional original photos extracted from the content as clean, responsive figures beneath the rewrite text.

## 3. Non-Goals & Anti-Goals
- Do not lose the existing Inverted Pyramid AI rewrite structure or the asynchronous Impact & Fallacies analysis.
- Do not break existing caching in `ai_article_summaries` or `MinifluxCache`.
- Do not render broken images (always include fallback / error handling for external image URLs).

## 4. Provenance
- User confirmation: `"Option 2. Hero image at top with content title inside the hero image, same as attached photo"`
