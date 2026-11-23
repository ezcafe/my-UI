# Grill Log: AI Magazine View Toggle & Hot Topic Highlighting

**Result:** frontier-empty

## Round Log

### Round 1

- **Q1: How should the default AI view be applied on page load without visual flicker or layout jump?**
  - *Recommendation*: Use an inline script or immediate DOM initialization that checks `localStorage.getItem('my_news_view_mode')`. If `null`, it defaults to `'ai'`. The shell container receives an active class or data attribute (`data-view="ai"` or `data-view="list"`), showing the magazine section and hiding the list section before first paint, just like the theme toggle.
  - *Status*: Settled.

- **Q2: How should the hot topic badge be styled under Annuaire design system constraints?**
  - *Recommendation*: Follow Annuaire Rule 1 ("Red marks position, never action"). The hot topic badge will be a small editorial pill badge with brand red accent (`text-brand border-brand/30 bg-brand/10 uppercase tracking-wider font-bold text-[11px] px-2 py-0.5 rounded-full inline-flex items-center gap-1`). Non-hot topics will use muted styling (`text-muted-foreground border-border bg-muted`).
  - *Status*: Settled.

- **Q3: How are article photos extracted, and how should cards without photos be displayed?**
  - *Recommendation*: Extract the first `<img src="...">` URL from the article content/summary. For articles with an image, display an editorial image header (`h-36 w-full object-cover rounded-t-lg border-b border-border`). For articles without images, display a clean, high-density typographic card without blank placeholders or broken image icons, consistent with Annuaire print aesthetic.
  - *Status*: Settled.

- **Q4: How does title clicking in AI view preserve the item on the page while marking it as read?**
  - *Recommendation*: Clicking an article title triggers `openEntryDetail(id)` and `markArticleRead(id)`. In the magazine view, the article title has class `magazine-title`. When read, it receives class `text-muted-foreground` and removes `text-foreground font-semibold`, while the card remains fully in place in the grid (unlike the Unread filter in list view which might filter it out upon next refresh).
  - *Status*: Settled.
