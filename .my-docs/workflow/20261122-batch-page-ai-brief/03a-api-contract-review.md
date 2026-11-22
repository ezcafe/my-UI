Result: clean

# API Contract Review: Batch Page AI Brief & Multilingual Executive Briefs

## 1. Executive Summary & Review Verdict

This review re-evaluates the API contract specified in `03-design.md` for `/hx/executive-brief`, the batch LLM prompt/payload, HTMX Out-Of-Band (`hx-swap-oob`) updates, error handling, query parameters, and auth-decoupled behavior after the latest design update.

- **Verdict**: **clean**.
- **Reason**: All 3 Major issues and 4 Minor hardening recommendations identified in the prior review have been completely and accurately resolved in `03-design.md`.

---

## 2. Verification of Prior Findings

### Issue 1: Missing Query Parameters in `/hx/executive-brief` (Resolved)
- **Status**: **VERIFIED CLEAN**.
- **Evidence**:
  - `03-design.md` Section 3.E now explicitly reuses `EntriesFilterQuery` on `GET /hx/executive-brief` (`status`, `feed_id`, `category_id`, `search`, `offset`, `limit`).
  - Section 3.E Step 2 mandates querying Miniflux/GraphQL cache with the exact filter parameters (`feed_id`, `category_id`, `offset`, `limit`, `status`), guaranteeing 1:1 synchronization between visible row items and the generated executive brief.
  - Section 4 provides the updated template snippet in `executive_brief_skeleton.html` that forwards `query.feed_id` and `query.category_id`.

### Issue 2: Skeleton Failure Contract & Error Handling (Resolved)
- **Status**: **VERIFIED CLEAN**.
- **Evidence**:
  - `03-design.md` Sections 1, 2, 3.E Step 8, and 4 explicitly define the failure contract: on LLM timeout, process crash, or malformed JSON, the endpoint returns `200 OK` with `<div id="page-executive-brief" class="hidden"></div>` and zero OOB spans.
  - HTMX performs an `outerHTML` swap, cleanly collapsing the pulsing skeleton card without layout shift or endless pulsing.

### Issue 3: Filter Scope Discrepancy (`today` vs `unread`) (Resolved)
- **Status**: **VERIFIED CLEAN**.
- **Evidence**:
  - `03-design.md` Section 3.H clarifies that `today`, `read`, `starred`, `all`, and views with active `search` are strictly prohibited from generating executive briefs.
  - Section 3.H reconciles the codebase by specifying the update to `is_brief_eligible_filter` in `crates/server/src/routes/entries.rs` to remove `Some("today")`.

---

## 3. Verification of Defensive Hardening Items

1. **HTML Escaping in OOB Title Swaps**:
   - Specified in Section 3.E Step 10: `{escaped_suggested_title}` inside `<span id="entry-title-{id}" hx-swap-oob="outerHTML">` prevents XSS.
2. **Explicit Response Headers**:
   - Specified in Section 3.E: returns `Content-Type: text/html; charset=utf-8` and `Cache-Control: no-cache, no-store, must-revalidate` to prevent stale browser caching.
3. **Robust Batch JSON Parsing**:
   - Specified in Section 3.D: `LlmCliRunner::summarize_batch` defensively strips markdown fences and handles conversational preambles/trailers.
4. **Thundering Herd Deduplication**:
   - Specified in Section 3.E Step 6 and Section 3.F: in-flight processing lock or atomic key set in `AiService` ensures concurrent requests for the same page key await the first job.

---

## 4. Conclusion & Next Steps

The API contract in `03-design.md` is robust, complete, and aligned across all client and server components. Zero Critical or Major issues remain.
