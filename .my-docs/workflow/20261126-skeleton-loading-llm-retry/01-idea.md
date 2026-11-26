# Ideation — Skeleton Loading & LLM 1-Second Auto-Retry

- **Slug:** `20261126-skeleton-loading-llm-retry`
- **Size:** Small-to-medium UI + backend AI resilience change
- **Has UI (tentative):** `yes`
- **Has API (tentative):** `yes`
- **Has DB (tentative):** `no`
- **Has auth/PII/schema (tentative):** `no`

---

## Key Takeaways

- **Skeleton instead of circle spinner:** Clicking **Next**, **Previous**, or **Mark Done** (`#mark-page-read-btn`) currently hides `#entries-list` and shows a centered spinning circle (`#entries-loading-overlay`). We will replace that spinner with an accessible `animate-pulse` skeleton layout.
- **1-second auto-retry on LLM failures:** LLM calls in `crates/server/src/ai/llm.rs` currently fail on the first error (or fall back immediately without retrying). We will add **1 automatic retry after a 1-second delay** (`tokio::time::sleep(Duration::from_secs(1))`) before failing.
- **Visible failure feedback:** If the retry also fails, we will surface a clear user-friendly notice instead of silently dropping errors (especially in background prefetch, List view, or `/hx/entries/{id}/ai-summary`).

---

## Shared Understanding (Playback)

Based on the current codebase:

1. **Circle Loading State on Next / Previous / Mark Done (`crates/ui-components/templates/pages/index.html`, `crates/ui-components/templates/partials/entry_list.html`):**
   - Currently, `#entries-loading-overlay` renders a spinning SVG circle with `"Loading..."` when `showEntriesLoadingState()` is triggered in `htmx:beforeRequest` for `#pagination-next-btn`, `#pagination-prev-btn`, and `#mark-page-read-btn` (both authenticated HTMX requests and guest-mode `markVisiblePageLocallyRead()` when advancing to the next page).
   - **Goal:** Replace the circle spinner inside `#entries-loading-overlay` with an accessible skeleton (`animate-pulse`, `motion-reduce:animate-none`, `aria-busy="true"`, `aria-hidden="true"` on skeleton shapes, and `sr-only` loading text) matching the Annuaire theme tokens (`bg-muted`, `border-border`).
   - *Guess to confirm:* We plan to make the skeleton **view-aware**—showing a 3-column Magazine skeleton when in Magazine view (`currentViewMode === 'ai'`) and stacked row skeletons when in List view (`currentViewMode === 'list'`).

2. **Auto-Retry LLM Requests After 1 Second (`crates/server/src/ai/llm.rs`, `crates/server/src/routes/entries.rs`, `crates/server/src/ai/prefetch.rs`):**
   - Currently, `LlmCliRunner` (`summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, `analyze_impact_and_fallacies`) and `LlmHttpClient::request_chat_completion` run only 1 attempt before returning an error or `extractive-fallback`.
   - Also, `executive_brief_htmx` waits up to 4 seconds (`40 * 100ms`) for an `in_flight` background prefetch job (`AiPrefetchService::run_prefetch`) before showing an error—which can time out prematurely if a request + 1-second retry takes longer.
   - **Goal:** When an LLM request fails (process error, non-2xx HTTP status, timeout, empty output, or unparseable batch response), wait 1 second and automatically retry once, and coordinate `executive_brief_htmx` waiting with the retry flow.

3. **Notify User If Retry Still Fails (`crates/server/src/routes/entries.rs`, `crates/server/src/error.rs`, `crates/ui-components/templates/`):**
   - Currently, `/hx/executive-brief`, `/hx/entries/{id}/analysis`, and `/hx/entries/{id}/rewrite` render an inline `AiErrorCardTemplate` (with `LLM_HIGH_DEMAND_ERROR_MESSAGE` and a manual Retry button), while `/hx/entries/{id}/ai-summary` silently falls back to extractive text and `AiPrefetchService::run_prefetch` fails silently in the background.
   - **Goal:** Ensure that when an LLM request still fails after the 1-second auto-retry, the user is clearly notified with a friendly message (no raw stack traces or technical codes).

---

## Non-Goals & Constraints

- **Non-Goals:**
  - Do not change Miniflux GraphQL pagination queries or database schema (`article_ai_summaries`, `page_executive_briefs_v2`).
  - Do not remove the existing manual **Retry** button on inline AI error cards.
- **Constraints:**
  - Limit automatic retries to **1 retry after 1 second** (2 total attempts) so requests do not stall indefinitely.
  - Follow the Annuaire Design System (`ANNUAIRE_STYLE_GUIDE.md`: zero page shadows, `bg-muted` / `border-border` tokens, `prefers-reduced-motion` support).
  - Never leak raw stderr, HTTP status dumps, or API keys in user-facing error text.

---

## Decision: How to Notify the User When LLM Still Fails After Retry

### Option 1 (Recommended): Inline Error Cards + Toast Notification for Silent/Background Failures
- **Explanation:** Keep the existing contextual `AiErrorCardTemplate` (with Retry button) in `#page-executive-brief`, `#list-executive-brief`, `#ai-analysis-box`, and `#ai-rewrite-box`, extend `AiErrorCardTemplate` to `/hx/entries/{id}/ai-summary`, AND trigger a floating `ToastTemplate` (`LLM_HIGH_DEMAND_ERROR_MESSAGE`) when an LLM failure occurs in a view/flow that wouldn't otherwise be clearly visible (or alongside the failed brief/summary/analysis response so the user always notices it regardless of scroll position or active view).
- **Concrete Example:**
  - Server (`crates/server/src/routes/entries.rs`): return the inline `AiErrorCardTemplate` **plus** `crate::error::render_llm_error_toast()` (using existing `#toast-container` `hx-swap-oob="beforeend"`) when retry is exhausted, or notify when background/on-demand summary fails.
- **Pros:**
  - Users are guaranteed to see the failure immediately even if scrolled down or in List view.
  - Preserves one-click inline **Retry** buttons right inside the affected section.
  - Reuses existing `AiErrorCardTemplate` and `render_llm_error_toast()`.
- **Cons:**
  - Users viewing the Executive Brief or Detail pane see both the inline card and a temporary 5-second toast.

### Option 2: Inline Contextual Error Cards Only (No Floating Toast)
- **Explanation:** Keep zero floating toasts (matching `20261124-contextual-ai-error-display`) and add `AiErrorCardTemplate` to `/hx/entries/{id}/ai-summary` as well, relying only on inline cards where AI content is displayed.
- **Concrete Example:**
  - In `get_entry_ai_summary_htmx`: on failure after retry, render `AiErrorCardTemplate` targeting `#ai-summary-slot-<id>` instead of silent extractive text.
- **Pros:**
  - Zero toast clutter; all errors live strictly inside the box that failed.
- **Cons:**
  - If a background prefetch fails while the user is in List view (or scrolled past the brief card), they may not see why AI titles/summaries didn't appear unless they inspect the brief card.

### Option 3: Floating Toast Only + Silent Fallback Content
- **Explanation:** Show `render_llm_error_toast()` / `showToast(...)` on LLM failure after retry, while rendering fallback text without inline error cards.
- **Concrete Example:**
  - Append `render_llm_error_toast()` and keep fallback text in `#ai-rewrite-box` / `#ai-summary-card`.
- **Pros:**
  - Keeps layout minimal with no error boxes inside reading panes.
- **Cons:**
  - Reverts the manual inline **Retry** buttons introduced in `20261124-contextual-ai-error-display`.

**Recommendation:** **Option 1** — keeping inline error cards with manual Retry buttons while ensuring a floating toast (`LLM_HIGH_DEMAND_ERROR_MESSAGE`) guarantees the user is informed even if scrolled away or when background/summary requests fail.

---

## Reference Sources

- **Codebase touchpoints:**
  - [`crates/ui-components/templates/pages/index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L168-L174) (`#entries-loading-overlay`, `showEntriesLoadingState`, `htmx:beforeRequest`)
  - [`crates/ui-components/templates/partials/entry_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L194-L230) (`#pagination-prev-btn`, `#pagination-next-btn`, `#entries-list-view`, `#entries-magazine-view`)
  - [`crates/server/src/ai/llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L474-L1158) (`LlmCliRunner`, `LlmHttpClient`, `LlmBackend`)
  - [`crates/server/src/ai/prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs#L50-L185) (`AiPrefetchService::run_prefetch`)
  - [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L677-L1815) (`executive_brief_htmx`, `get_entry_ai_summary_htmx`, `get_entry_analysis_htmx`, `get_entry_rewrite_htmx`, `mark_page_read_htmx`)
  - [`crates/server/src/error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L7-L17) (`LLM_HIGH_DEMAND_ERROR_MESSAGE`, `render_llm_error_toast`)
- **Official documentation:**
  - HTMX request lifecycle & indicators (`htmx.org`)
  - Tokio async sleep (`tokio::time::sleep` on `tokio.rs` / `docs.rs`)
  - Tailwind CSS `animate-pulse` (`tailwindcss.com`) and MDN `aria-busy` (`developer.mozilla.org`)

---

## Open Question for User

*(Reply to confirm or say **"just start"** to skip ideation and proceed with defaults: view-aware skeleton + Option 1.)*

Does the summary above match what you want—using a **view-aware skeleton** (matching Magazine or List view) on Next / Previous / Mark Done, retrying failed LLM requests **once after 1 second**, and for notifying the user when it still fails, do you prefer **Option 1 (Recommended: keep inline error cards with Retry AND show a toast notification)** or **Option 2 (inline error cards only, no toast)**?
