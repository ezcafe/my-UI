# System Analysis

**Result:** done

---

## 1. Context and Problem Statement

1. **List Page Brief Consistency (3 to 5 Sentences)**:
   - Articles displayed across list and magazine views currently have varying brief lengths or fallbacks.
   - LLM generation in `format_prompt` currently instructs: "Summarize what changed or why this news matters in strictly 3 concise sentences."
   - The user requires that all briefs for items on the list page must have length strictly 3 to 5 sentences.
   - Fallback extraction (`extract_summary` in `crates/miniflux/src/summary.rs`) extracts `sentences_count` sentences; ensuring this extracts 3 sentences guarantees that non-AI or uncached fallback text also aligns with the 3 to 5 sentence standard.
   - Card line-clamping in `entry_list.html` (e.g. `line-clamp-4` on Priority 1, `line-clamp-3` on Priority 2) needs adjustment so 3 to 5 sentences are readable without awkward truncation.

2. **Detail Page AI Rewrite with Original Article Photos**:
   - Currently, when opening an article in the detail panel (`/hx/entries/:id`), the AI rewrite replaces the entire content body, and `get_entry_detail` sets `image_url: None`. This completely strips all photos from the original article.
   - The user explicitly confirmed **Option 2**: An editorial hero image at the top with the content title and metadata overlaid inside the hero image with a readability gradient, followed by the AI inverted pyramid rewrite, followed by the remaining original photos in the article body.
   - Articles without photos must fall back to an elegant clean typography header without broken image frames or layout jumps.

---

## 2. Technical Risks & Mitigations

- **Risk: Broken or Blocked External Image URLs**:
  - *Risk*: RSS article images from third-party hosts might be dead links, HTTP-only, or blocked by CORS / referrer policies.
  - *Mitigation*: Use `referrerpolicy="no-referrer"`, `loading="lazy"`, and `onerror="this.style.display='none'"` on all `<img>` elements. If the hero image fails to load, gracefully fall back to displaying the text cleanly or hide the image container.
- **Risk: Tracking Pixels and Ad Icons Treated as Photos**:
  - *Risk*: Feeds often contain 1x1 tracking gifs, feedburner icons, or tiny share buttons as `<img>` tags.
  - *Mitigation*: Filter extracted image URLs to exclude common tracking pixels (e.g. URLs containing `1x1`, `pixel`, `feedburner`, or tiny dimensions/data URLs if evident).
- **Risk: Text Legibility on Varied Hero Images**:
  - *Risk*: Light, busy, or multi-colored hero images can make overlaid title text unreadable.
  - *Mitigation*: Employ a modern CSS gradient scrim overlay (`linear-gradient(to top, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.45) 50%, rgba(0,0,0,0.05) 100%)`) with white text and subtle text drop-shadow, ensuring WCAG AA contrast compliance across all backgrounds.
- **Risk: Prompt Adherence for 3 to 5 Sentences**:
  - *Risk*: LLMs may output fewer than 3 or more than 5 sentences if instructions are vague.
  - *Mitigation*: Explicitly state "strictly 3 to 5 sentences" in both prompt instructions and schema definitions. Extractive fallback uses 3 sentences.

---

## 3. Open Questions & Frontier

- *Are there any blocking architectural questions requiring user input?*
  - No. The user specifically selected and confirmed Option 2 ("Hero image at top with content title inside the hero image, same as attached photo") and gave exact constraints for list briefs (3 to 5 sentences).
