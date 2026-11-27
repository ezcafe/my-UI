# Grill — Details Page Content Filter

Result: frontier-empty

## Design Tree

1. **Missing Rewrite Warning + Fallback Behavior** (settled — Round 1, Q1: **Option C**)
   - When an article has no AI rewrite ready (`record.rewrite` missing or `extractive-fallback`) and `ai_enabled == true`, render an inline notice/skeleton inside `#ai-rewrite-box` that automatically fetches `GET /hx/entries/{id}/rewrite` on load.
   - Only if generation fails (LLM error, HTTP/network failure, or `ai_enabled == false`) show the inline warning banner + Retry button (when AI enabled) + a clean plain-text summary/excerpt fallback (AI `summary` if available, else Miniflux `summary` with HTML stripped, else brief plain-text opening excerpt — never full original article HTML).
2. **List-to-Detail Title & Rewrite Sync (including background `/hx/executive-brief` & List View)** (settled — Round 1, Q2: **Yes**)
   - Sync `/hx/executive-brief` updates (`suggested_title` and `rewrite`) across List View (`#entry-title-{id}`), Magazine cards (P1/P2 `#mag-title-{id}` & `.magazine-brief`, P3 `title` attribute), and Guest state (`guestEntriesMap` + IndexedDB `all_articles`), and invalidate any stale cached Auth detail HTML for updated IDs.
   - Both Auth (`/hx/entries/{id}`) and Guest (`renderGuestEntryDetail`) display the synced AI title & rewrite regardless of whether the article was opened from Magazine View, List View, or Guest `All` tab.
3. **Photos Extraction (Hero + Body Images in Guest Mode) & Section Order** (settled — Round 1, Q3: **Yes**)
   - Extract all non-pixel original photos (`hero_image` + `body_images` via `extract_article_images`) in both Auth and Guest modes.
   - Keep the exact detail body section order:
     1. Hero photo with overlaid title (or standard title if no hero photo)
     2. Rewrite (or Option C loading notice/skeleton -> warning + clean plain-text fallback on failure)
     3. Body photos gallery (`body_images`, if non-empty)
     4. Impact Analysis & Logical Fallacy & Objectivity Check (`analysis_cards` or `#ai-analysis-box`)
4. **Top Bar & Metadata Removal & No Original Content** (settled via ideation)
   - Top action bar stays (`Back`, `Original` link, read/unread toggle, bookmark).
   - Remove `feed_title` and date/time metadata (`<time>`, `formatted_date`) from Auth detail, Guest detail, and Auth loading skeleton.
   - Remove `#guest-detail-content` / `sanitized_content` and never display original article text/HTML in the details panel.

---

## Round 1 (Frontier) — Settled

❓ **Q1** - **Missing Rewrite Fallback Content**
- **User Answer:** **Option C** — Show an inline notice/skeleton and automatically fetch/generate the rewrite on demand via `GET /hx/entries/{id}/rewrite` (falling back to warning + clean plain-text summary/excerpt only if generation fails).

---

❓ **Q2** - **List-to-Detail Title & Rewrite Sync (Async AI + List View)**
- **User Answer:** **Yes** — Sync `/hx/executive-brief` updates across List View, Magazine cards, and `guestEntriesMap`, and have both Auth (`/hx/entries/{id}`) and Guest (`renderGuestEntryDetail`) display that synced AI title & rewrite.

---

❓ **Q3** - **Multi-Photo Extraction in Guest Mode & Detail Section Order**
- **User Answer:** **Yes** — Extract `hero_image` + `body_images` in both Auth and Guest modes, and keep the detail body order: Hero photo with overlaid title -> Rewrite (or warning + fallback) -> Body photos gallery -> Impact Analysis & Logical Fallacy & Objectivity Check.
