# System Analysis

**Result:** done

---

## 1. Context and Problem Statement

1. **Executive Brief Markup Redundancy**:
   - When `/hx/executive-brief` is requested, the server sends two complete HTML structures: one standard target (`#page-executive-brief`) and one out-of-band target (`#list-executive-brief hx-swap-oob="true"`).
   - This leads to duplicate rendering logic, double DOM manipulation, and potential desynchronization if one view is altered.

2. **Detail Panel Delay and Cache Invalidation Failure**:
   - `openEntryDetail(id)` checks `cached.html.includes('Executive Brief')` to validate the client-side IndexedDB cache. Since recent detail templates use "Inverted Pyramid Rewrite Content" and no longer output "Executive Brief", cached detail pages are discarded every time.
   - When the cache misses or is pending, `openDetailWithSkeleton()` wipes the detail panel and renders blank skeleton placeholders. Even though the article's title is already rendered in the DOM (`#entry-title-${id}`), the user sees gray pulsing bars instead of immediate content.

3. **Analysis Endpoint Redundant LLM Calls**:
   - `get_entry_analysis_htmx` only reuses cached database records if `impact` and `fallacies` are both present and non-empty.
   - Furthermore, `analyze_entry_impact_and_fallacies` in `AiService` does not check the database before dispatching an LLM execution. If the analysis endpoint is called for an article that already has impact or fallacies, it risks invoking the LLM unnecessarily.

4. **Prompt Verbosity and Boilerplate**:
   - Current prompts instruct the model on the inverted pyramid layout, but do not strictly enforce word counts (<110 words, 30–60 words sweet spot) or eliminate conversational opening and closing phrases.

---

## 2. Technical Risks & Mitigations

- **Risk: Breaking List View Brief Updates**:
  If `#list-executive-brief` is removed from `executive_brief_card.html`, the list view might not display the brief if it was relying on the out-of-band swap.
  - *Mitigation*: Ensure the brief card element is properly targeted or synchronized by client JavaScript (via `isBriefSwap` in `index.html`) or unified into a single responsive container.

- **Risk: Cache Stale Data in Detail Panel**:
  If `openEntryDetail` immediately injects title and brief from the DOM while fetching full details in the background, a mismatch could flash if the article changed upstream.
  - *Mitigation*: The DOM contains the exact title displayed in the user's active feed. Injecting the title immediately gives an instant perception of speed, which is seamlessly updated when HTMX finishes swapping `#detail-panel`.

- **Risk: LLM Prompt Brevity vs Information Loss**:
  Strict word caps (<110 words) could cause the model to truncate important facts.
  - *Mitigation*: Specify inverted pyramid guidance: the first 2 sentences contain the core outcome and impact, followed by 2–3 concise bullet points with essential context.

---

## 3. Open Questions

- *Are there any blocking architectural ambiguities?*
  No. All four areas touch well-defined templates, routes, and prompt formatting helpers with established unit test suites.
