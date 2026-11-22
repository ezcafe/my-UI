# Grill: Batch Page AI Brief & Multilingual Executive Briefs

## Settled Decisions

### 1. Title & Brief Update Strategy
- **Decision**: Option 1 (Instant OOB Swap).
- **Behavior**: When the page batch LLM call finishes, `/hx/executive-brief` returns the completed Executive Brief card and triggers HTMX Out-Of-Band swaps (`hx-swap-oob="outerHTML"`) to dynamically update all article titles on the current page.

### 2. Page Brief Cache Key & Invalidation (Q1)
- **Decision**: Option 1A (Cache by entry-ID fingerprint).
- **Behavior**: The page executive brief is cached in `page_executive_briefs_v2` using a deterministic key generated from the hash of entry IDs visible on that page (`unread:sha256(101,102,103...)`).
- **Rationale**: 100% accurate; eliminates stale summary mismatches when feed updates shift articles between pages.

### 3. Skeleton Loading UI & Style (Q2)
- **Decision**: Option 2A (Annuaire-styled pulsing skeleton card).
- **Behavior**: An Annuaire-compliant placeholder card renders immediately with `animate-pulse`, muted bars, sparkles icon, and "Generating Executive Brief..." label. When `/hx/executive-brief` returns, HTMX swaps `outerHTML` cleanly with zero layout shift.
- **Rationale**: Keeps initial page rendering sub-50ms and prevents jarring visual jumps.

### 4. Decoupling AI from Authentication
- **Decision**: `AI_ENABLED` functions independently of `AUTH_ENABLED`.
- **Behavior**: If `AUTH_ENABLED=false` and `AI_ENABLED=true`, AI summaries, batch page briefs, and titles are fully active (e.g. for guest / single-user mode).
- **Rationale**: Users running personal local instances without OAuth/OIDC can still benefit from AI digests.

### 5. Server-Side Prefetch on Page Threshold
- **Decision**: Prefetch LLM requests on server when there are enough items per page (e.g. threshold of uncached items).
- **Condition**: Items must not have been included in an LLM request before. Once an item is summarized, it is marked/cached and never re-fetched.

### 6. List Page Load Batching
- **Decision**: On list page load, if there are items not included in an LLM request before, perform exactly 1 LLM request for them in batch.
- **Behavior**: The endpoint `/hx/executive-brief` extracts content for the visible unbatched items and submits them in a single batch prompt, saving individual titles/briefs and the page-level executive brief.

---

## Round 1 — Frontier Questions & Resolutions

### ❓ Q1 - Page Brief Cache Key & Invalidation
- **Question**: Should the page executive brief be cached using a fingerprint of the visible article IDs?
- **Resolution**: **Option A Selected**. Hash the list of entry IDs on the page to form the `page_key`. If new unread articles arrive and shift items, the fingerprint changes, triggering an accurate brief.

### ❓ Q2 - Skeleton Loading UI & Style
- **Question**: Should the loading placeholder display an Annuaire-styled card with pulsing skeleton lines?
- **Resolution**: **Option A Selected**. Pulsing skeleton card matching Annuaire styling with `animate-pulse` and `hx-swap="outerHTML"`.

---

## Result

Result: frontier-empty
