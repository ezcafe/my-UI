# Ideation: Fix Executive Brief Duplication, Cache Detail Title/Brief, DB Reuse for Impact/Fallacy, and Rewrite Rules

**Sizing:** Medium change with written design  
**Has UI (tentative):** yes (eliminate duplicate brief markup in executive brief card/response, instant title & brief retrieval from list page/cache on detail open)  
**Has API (tentative):** yes (clean up `/hx/executive-brief` response, detail route `/hx/entries/{id}` and analysis endpoint caching/reuse)  
**Has DB (tentative):** yes (ensure `get_entry_analysis_htmx` strictly reuses impact/fallacies from DB and does not trigger LLM when already cached)  
**Has auth/PII/schema (tentative):** no (no schema changes required; all columns exist)  

---

## 1. Intent Summary & Playback

### User Requirements
1. **Executive Brief Duplicate in Response**: When `/hx/executive-brief` is fetched, the response returns both `#page-executive-brief` and `#list-executive-brief hx-swap-oob="true"` with identical text. Fix this duplication so only the intended element is returned or cleanly swapped.
2. **Instant Title & Brief from Cache on Detail Load**: When opening `/hx/entries/{id}`, retrieve title and brief from client-side cache/data since the list page already contains this information, avoiding unnecessary blocking or redundant fetching.
3. **Database Reuse for Analysis**: When loading a news item a second time (or when already analyzed), reuse `Impact Analysis` and `Logical Fallacy & Objectivity Check` stored in the database. Never call the LLM if the item was already requested/cached.
4. **Rewrite Rule Guidelines**:
   - Under 110 words (30–60 words sweet spot, 2–3 bullet points).
   - Inverted pyramid structure (outcome and primary impact in the first 2 sentences).
   - Plain, simple language focused strictly on essential details (no unnecessary intro/closing boilerplate).

---

## 2. Framing

### Goals
- **Clean Executive Brief Response**: No duplicate brief markup in `/hx/executive-brief` response.
- **Immediate Detail Rendering**: Title and brief load instantly from available list/IndexedDB cache when an article is opened.
- **Strict DB Reuse**: Zero LLM requests when an article already has impact and fallacy analysis in the database.
- **Focused Rewrites**: Update prompt instructions for single and batch rewrites to be under 110 words, inverted pyramid (first 2 sentences lead with outcome/impact), and essential bullet points without fluff.

### Non-Goals
- Adding new database columns or altering PostgreSQL/SQLite schemas.
- Changing authentication mechanisms.

### Constraints
- Multilingual fidelity: Vietnamese sources must produce Vietnamese outputs; English must produce English.
- TDD iron law: All changes must be backed by unit and integration tests.
