# Design — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

**Result:** done

**Design present** only when Result **`done`** (file alone / `pending` never pays).

## Key Conclusions

- **View-aware accessible skeleton:** `#entries-loading-overlay` renders pre-structured Magazine (3-column `.magazine-3col-grid`) and List (stacked `min-h-[48px]` Annuaire directory rows) skeletons toggled by `showEntriesLoadingState()` via `currentViewMode`, using `animate-pulse`, `motion-reduce:animate-none`, `aria-busy`, and `sr-only` status text.
- **HTMX `outerHTML` lifecycle & toast safety:** `index.html` explicitly matches `#ai-analysis-box` and `#ai-rewrite-box` swaps for IndexedDB caching (instead of relying on detached `evt.detail.target.closest('#detail-panel')`), `base.html` resolves the connected DOM target before `lucide.createIcons(...)`, and Guest `showToast()` sets `data-auto-dismiss="true"` so OOB toast deduplication never double-timers client toasts.
- **DRY 1-second retry & coordinated wait:** A shared async retry helper in `crates/server/src/ai/llm.rs` runs 1 retry after `tokio::time::sleep(Duration::from_secs(1))` across `LlmCliRunner` (releasing CLI semaphore permits between attempts) and `LlmHttpClient` (rejecting empty bodies and invalid batch/analysis JSON before `LlmBackend::Http` fallback), while `executive_brief_htmx` polls `in_flight` up to 250s and exits immediately once prefetch clears without a brief.
- **Option 1 unified error cards + toast & clean 404/fallback boundaries:** LLM exhaustion across `/hx/executive-brief`, `/hx/entries/{id}/ai-summary`, `/hx/entries/{id}/analysis`, and `/hx/entries/{id}/rewrite` returns `AiErrorCardTemplate` + `crate::error::render_llm_error_toast()`, while Miniflux `404`/not-found on `/ai-summary` keeps `"Summary unavailable"` without a demand toast and detail/rewrite endpoints ignore legacy `"extractive-fallback"` DB rows.

---

## Option A (Recommended): Shared `retry_once_after_delay` Helper + Declarative View-Aware Skeletons + Full Option 1 Toast & Error Card Parity

- **Explanation:**
  1. **Backend retry:** Extract a single generic async helper `retry_once_after_delay` in [`crates/server/src/ai/llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs) used by `LlmHttpClient` (transport, empty output, and strict parse paths) and all 4 `LlmCliRunner` execution paths (`summarize_batch_structured`, `summarize`, `rewrite_and_analyze`, `analyze_impact_and_fallacies`). Acquire `LlmCliRunner::semaphore` per attempt so the 1-second `tokio::time::sleep` never blocks concurrent requests.
  2. **Prefetch & detail safety:** Coordinate `executive_brief_htmx` to exit early once `page_key` leaves `in_flight` (up to `250s`, covering HTTP + CLI retry windows), skip persisting `"extractive-fallback"` payloads from failed batch runs, and ignore pre-existing `"extractive-fallback"` DB rows in detail/rewrite/analysis handlers.
  3. **Error UX & 404 separation:** Return `AiErrorCardTemplate` + `crate::error::render_llm_error_toast()` on LLM failure across `/hx/executive-brief`, `/hx/entries/{id}/ai-summary`, `/hx/entries/{id}/analysis`, and `/hx/entries/{id}/rewrite`, while keeping `"Summary unavailable"` (with no LLM toast) when Miniflux returns `404`/not-found on `/ai-summary`. Fix HTMX `outerHTML` detached-target hooks in [`index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) and [`base.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) so icons hydrate and retried detail panels cache cleanly.
  4. **UI skeleton:** Replace the spinner inside `#entries-loading-overlay` (`index.html`) with `#entries-skeleton-magazine` and `#entries-skeleton-list` matching [`entry_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html) grid/row proportions (`min-h-[48px]` matching `.ar-list-item`) and Annuaire tokens, toggled by `showEntriesLoadingState()` according to `currentViewMode`.

- **Concrete Examples:**
  ```rust
  async fn retry_once_after_delay<T, E, F, Fut>(
      operation: &str,
      delay: Duration,
      mut op: F,
  ) -> Result<T, E>
  where
      F: FnMut(u8) -> Fut,
      Fut: std::future::Future<Output = Result<T, E>>,
      E: std::fmt::Display,
  {
      match op(1).await {
          Ok(val) => Ok(val),
          Err(first_err) => {
              tracing::warn!(
                  target: "ai::retry",
                  "[AI_RETRY] operation=\"{}\" attempt=1 delay_ms={} reason=\"{}\"",
                  operation,
                  delay.as_millis(),
                  first_err
              );
              tokio::time::sleep(delay).await;
              op(2).await
          }
      }
  }
  ```

  ```html
  <div id="entries-loading-overlay"
       role="status"
       aria-live="polite"
       class="hidden absolute inset-0 z-20 p-4 overflow-hidden bg-card pointer-events-none">
    <span class="sr-only">Loading articles...</span>
    <div id="entries-skeleton-magazine" class="hidden magazine-container pt-1 pb-6 w-full animate-pulse motion-reduce:animate-none" aria-hidden="true">
      <div class="magazine-3col-grid w-full">
        <!-- Brief col + Hero/P2 center col + P3 right col -->
      </div>
    </div>
    <div id="entries-skeleton-list" class="hidden space-y-1.5 animate-pulse motion-reduce:animate-none" aria-hidden="true">
      <!-- Executive brief banner + 10 min-h-[48px] Annuaire row skeletons -->
    </div>
  </div>
  ```

- **Pros:**
  - Zero duplicated retry/backoff or JSON validation logic across CLI and HTTP runners.
  - Declarative HTML skeletons stay aligned with `entry_list.html` CSS (`magazine-3col-grid`, `magazine-p2-grid`, `.ar-list-item`, Annuaire tokens) and work cleanly with Askama compile checks.
  - Guarantees visible, actionable failure feedback (hydrated icons on inline Retry + non-stacking floating toast), accurate `404` vs LLM error semantics, and working IndexedDB detail caching after retry.

- **Cons:**
  - Requires updating 5 existing server test assertions (`crates/server/src/routes/entries.rs` lines `3519, 3600, 3711, 3814`, and `crates/server/src/routes/mod.rs` lines `1258, 1259`) whose old assertions explicitly forbade `toast-container` or expected silent `ai-summary-card` fallback (while keeping `mod.rs:1222` 404 ownership test intact).
  - When `LlmBackend::Http` falls back to CLI and both fail, total wait includes both 1-second retries (2s total backoff).

---

## Option B: Per-Method Inline Retry Loops + JS-Built Skeleton Strings + Partial Toast Scope

- **Explanation:**
  Write `for attempt in 0..2` loops inside each LLM method independently, construct skeleton HTML strings inside `showEntriesLoadingState()` in JS, and emit `render_llm_error_toast()` only on `/hx/executive-brief` and `/hx/entries/{id}/ai-summary`.

- **Concrete Example:**
  ```rust
  let mut attempt = 0;
  loop {
      attempt += 1;
      let res = run_once().await;
      if res.is_ok() || attempt >= 2 { break res; }
      tokio::time::sleep(Duration::from_secs(1)).await;
  }
  ```

- **Pros:**
  - Leaves detail-pane tests (`test_get_entry_rewrite_htmx_contextual_error_card_on_failure`, `test_get_entry_analysis_htmx_contextual_error_card_on_failure`) untouched.
  - Smaller HTML footprint in `index.html`.

- **Cons:**
  - Repeats retry, logging, and semaphore discipline across 5 functions.
  - JS template literals for skeletons bypass Askama HTML templates and duplicate markup logic in `<script>`.
  - Leaves detail-pane users without floating toast visibility if scrolled inside long articles.

---

## Recommendation

**Option A** is recommended. A single `retry_once_after_delay` helper plus shared strict parsers prevents inconsistent retry behavior across CLI and HTTP runners, declarative view-aware skeletons keep Annuaire styling maintainable, and unified Option 1 error cards + deduplicated floating toasts + live-node HTMX hooks fulfill the user's confirmed requirements across every AI view.

---

## Detailed Specifications by Component

### 1. View-Aware Skeleton Overlay, HTMX `outerHTML` Lifecycle & Accessibility (`crates/ui-components`)

#### A. [`crates/ui-components/templates/pages/index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
- **Loading overlay (`lines 168-174`):** Replace spinner markup with:
  - Wrapper `<div id="entries-loading-overlay" role="status" aria-live="polite" class="hidden absolute inset-0 z-20 p-4 overflow-hidden bg-card pointer-events-none">`
  - `<span class="sr-only">Loading articles...</span>`
  - `#entries-skeleton-magazine` (`class="hidden magazine-container pt-1 pb-6 w-full animate-pulse motion-reduce:animate-none" aria-hidden="true"`):
    - `.magazine-3col-grid w-full` containing:
      - Left column: Executive Brief skeleton card (`rounded-lg p-5 bg-muted/40 border border-border space-y-3`) with header pill + 4 text lines (`bg-muted rounded`).
      - Center column (`flex flex-col gap-6`):
        - Priority 1 Hero skeleton (`rounded-lg border border-border bg-card overflow-hidden flex flex-col sm:flex-row`) with hero image placeholder (`bg-muted min-h-[220px] sm:w-1/2`) and content lines (`p-5 sm:w-1/2 space-y-3`).
        - Priority 2 grid (`.magazine-p2-grid`) with 6 skeleton cards (`rounded-lg border border-border bg-card overflow-hidden`), each with `h-28 bg-muted` image block and `p-3.5 space-y-2` lines.
      - Right column (`flex flex-col gap-3`): 5 Priority 3 compact skeleton cards (`rounded-lg border border-border bg-card p-3.5 space-y-2`).
  - `#entries-skeleton-list` (`class="hidden space-y-1.5 animate-pulse motion-reduce:animate-none" aria-hidden="true"`):
    - Executive brief banner placeholder (`rounded-lg p-3.5 bg-muted/40 border border-border mb-3 space-y-2`).
    - 10 stacked Annuaire directory row placeholders (`min-h-[48px] px-4 py-2.5 rounded-lg flex items-center justify-between gap-3 border-b border-border/60`, matching `.ar-list-item` `min-height: 48px` in `base.html:294`), each with a 3px bar placeholder (`w-[3px] h-5 rounded-full bg-muted shrink-0`), unread dot placeholder (`w-2 h-2 rounded-full bg-muted shrink-0`), varying-width title lines (`h-3.5 bg-muted rounded`), and trailing action circle (`w-6 h-6 rounded-full bg-muted shrink-0`).
- **`showEntriesLoadingState()` and `hideEntriesLoadingState()` (`lines 480-494`):**
  - `showEntriesLoadingState()`:
    - Adds `'invisible'` and sets `aria-busy="true"` on `#entries-list`.
    - Reads `const isMagazine = currentViewMode === 'ai';`
    - Toggles `hidden` on `#entries-skeleton-magazine` (`!isMagazine`) and `#entries-skeleton-list` (`isMagazine`).
    - Removes `'hidden'` from `#entries-loading-overlay`.
  - `hideEntriesLoadingState()`:
    - Removes `'invisible'` and sets `aria-busy="false"` on `#entries-list`.
    - Adds `'hidden'` to `#entries-loading-overlay`, `#entries-skeleton-magazine`, and `#entries-skeleton-list`.
    - Preserves `if (loadingBar && !activeEntriesRequestXhr) loadingBar.classList.remove('htmx-request');`.
- **`htmx:afterSwap` detail-panel match & IndexedDB cache check (`lines 1201-1215`):**
  - Because `#ai-analysis-box` (`entry_detail.html:122`) and `#ai-rewrite-box` (`ai_error_card.html` / `render_analysis_fragment`) use `hx-swap="outerHTML"`, HTMX `1.9.12` detaches `evt.detail.target` before firing `htmx:afterSwap`, causing `evt.detail.target.closest('#detail-panel')` to return `null`.
  - Update line `1201` to match explicit IDs and live ancestors:
    ```javascript
    } else if (
      targetId === 'detail-panel' ||
      targetId === 'ai-analysis-box' ||
      targetId === 'ai-rewrite-box' ||
      (evt.detail.target.closest && evt.detail.target.closest('#detail-panel')) ||
      (evt.target && evt.target.closest && evt.target.closest('#detail-panel'))
    ) {
    ```
  - Update line `1211` from `!panel.querySelector('.stroke-destructive')` to `!panel.querySelector('.stroke-destructive, .is-error')` so failed AI error cards are never cached in IndexedDB, while successful `/analysis` or retried `/rewrite` loads refresh the cached detail HTML.
- **Guest `showToast(message)` (`lines 1920-1934`):**
  - Set `toast.setAttribute('data-auto-dismiss', 'true');` on the client-created toast node (and optionally remove any existing matching-text toast) before appending to `#toast-container`, preventing `base.html`'s `htmx:oobAfterSwap` handler from attaching a second `5000ms` timer to an active Guest toast (`4000ms`).

#### B. [`crates/ui-components/templates/partials/ai_grouped_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html)
- Wrap `#ai-summary-slot-{{ item.id }}` in `<div class="mt-3">` and keep `pt-3 border-t border-border` on `#ai-summary-slot-{{ item.id }}` itself (moving `mt-3` to the wrapper) so `outerHTML` replacement with `AiErrorCardTemplate` (`ai_error_card.html` has no top margin) preserves `mt-3` spacing below the title row, while `AiSummaryCardTemplate` collapses cleanly in block flow.
- Set `id="ai-summary-slot-{{ item.id }}"` on `.ai-summary-slot` so `/hx/entries/{id}/ai-summary` error card Retry targets `#ai-summary-slot-{{ id }}` cleanly.
- Replace the `<i data-lucide="sparkles" class="w-3.5 h-3.5 animate-spin"></i>` loading row with accessible `animate-pulse motion-reduce:animate-none` skeleton bars (`aria-hidden="true"` on shapes + `<span class="sr-only">Generating summary...</span>`), preserving `.ai-summary-slot`, `hx-get`, `hx-trigger="revealed"`, and `hx-swap="outerHTML"`.

#### C. [`crates/ui-components/templates/base.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html)
- **Live-root Lucide hydration in `htmx:afterSwap` (`lines 576-580`):**
  - When `hx-swap="outerHTML"` detaches `evt.detail.target` (`!evt.detail.target.isConnected`), resolve the connected replacement element before calling `lucide.createIcons`:
    ```javascript
    document.body.addEventListener("htmx:afterSwap", (evt) => {
      if (window.lucide && typeof lucide.createIcons === 'function' && evt.detail && evt.detail.target) {
        const swapTarget = evt.detail.target;
        const liveRoot = swapTarget.isConnected
          ? swapTarget
          : (swapTarget.id ? document.getElementById(swapTarget.id) : null) ||
            (evt.target && evt.target.isConnected ? evt.target : document.body);
        lucide.createIcons({ root: liveRoot });
      }
    });
    ```
  - This hydrates `alert-circle` and `refresh-cw` on `AiErrorCardTemplate` (`#page-executive-brief`, `#ai-analysis-box`, `#ai-rewrite-box`, `#ai-summary-slot-{id}`) and `sparkles` / `trending-up` / `shield-alert` on success cards (including `AiSummaryCardTemplate` where the replacement omits the slot ID).
- **OOB toast deduplication in `htmx:oobAfterSwap` (`lines 581-594`):**
  - For each newly added `.pointer-events-auto:not([data-auto-dismiss])` toast inside `#toast-container`:
    - Read its trimmed `span.textContent`.
    - Remove any earlier `.pointer-events-auto` element inside `#toast-container` whose trimmed `span.textContent` matches (protecting Guest toasts via `data-auto-dismiss="true"` set in `showToast()`).
    - Set `toast.setAttribute('data-auto-dismiss', 'true')` and schedule the single `5000ms` fade-out and removal timer.

---

### 2. LLM 1-Second Auto-Retry (`crates/server/src/ai/llm.rs`, `crates/server/src/ai/mod.rs`)

#### A. Shared Helper, Strict Parse Helpers & `LlmCliRunner` (`llm.rs`)
- Add `const LLM_RETRY_DELAY: Duration = Duration::from_secs(1);` and `retry_once_after_delay(operation, delay, op)`.
- Add shared strict parsers on `LlmCliRunner` (reused by both `LlmCliRunner` and `LlmHttpClient` to avoid duplication):
  - `try_parse_batch_response(raw_output: &str) -> Option<BatchAiResponse>`: calls `Self::parse_batch_response(raw_output)` and returns `Some(parsed)` only when `!parsed.top.trim().is_empty() || !parsed.items.is_empty()` (else `None`).
  - `try_parse_impact_and_fallacies_response(raw_output: &str) -> Option<(String, String)>`: extracts direct or fenced JSON into `RawAnalysis`, returns `Some((impact, normalized_fallacies))` only when JSON deserialization succeeds (else `None`). `parse_impact_and_fallacies_response` delegates to `try_parse_impact_and_fallacies_response(raw_output).unwrap_or_else(...)` so existing public parser behavior remains identical.
- Add `LlmCliRunner::run_cli_prompt(&self, timeout_secs: u64, prompt_bytes: &[u8], log_target: &str) -> Result<String, CliAttemptError>`:
  - Acquires `self.semaphore.acquire().await.ok()` for that single attempt (automatically dropped on return, never held during `tokio::time::sleep`).
  - Spawns `Command::new(&self.cli_cmd)` with `kill_on_drop(true)`, pipes `prompt_bytes` via stdin, waits up to `Duration::from_secs(timeout_secs)`.
  - Treats spawn error, non-zero exit status, timeout, and empty trimmed stdout as `Err(...)` with structured `[AI_ERROR]` tracing logs.
- Wire `retry_once_after_delay` (with `LLM_RETRY_DELAY = 1s`) into all 4 `LlmCliRunner` methods:
  1. `summarize_batch_structured` (90s timeout per attempt): fails an attempt on CLI error OR if `Self::try_parse_batch_response(&text)` returns `None`. After 2nd failure, returns `Self::fallback_extractive_structured(articles)`.
  2. `summarize` (30s timeout per attempt): fails an attempt on CLI error / empty output. After 2nd failure, returns `(fallback_text, "extractive-fallback".to_string())`.
  3. `rewrite_and_analyze` (30s timeout per attempt): fails an attempt on CLI error / empty output OR if `Self::parse_rewrite_and_analyze_response(&text).rewrite.trim().is_empty()`. After 2nd failure, returns `(DetailAnalysisResult::default(), "error".to_string())`.
  4. `analyze_impact_and_fallacies` (30s timeout per attempt): fails an attempt on CLI error / empty output OR `Self::try_parse_impact_and_fallacies_response(&text).is_none()`. After 2nd failure, returns `(Self::parse_impact_and_fallacies_response(""), "error".to_string())`.

#### B. `LlmHttpClient` (`llm.rs`)
- Extract `request_chat_completion_once(&self, prompt: &str) -> Result<String, String>` containing the Gemini/OpenAI `reqwest` POST, HTTP status check, JSON envelope extraction, **and** empty/whitespace check:
  - Reject `text.trim().is_empty()` with `Err("Empty LLM HTTP response".to_string())` on both Gemini and OpenAI paths so HTTP `200` with whitespace-only content retries and/or falls back to CLI.
- Implement `request_chat_completion(&self, prompt: &str) -> Result<String, String>` via `retry_once_after_delay("http_chat_completion", LLM_RETRY_DELAY, |_| self.request_chat_completion_once(prompt)).await` (used directly by `LlmHttpClient::summarize`).
- In `LlmHttpClient::summarize_batch_structured` and `LlmHttpClient::analyze_impact_and_fallacies`: wrap `self.request_chat_completion_once(&prompt)` + strict inner parse (`LlmCliRunner::try_parse_batch_response` / `LlmCliRunner::try_parse_impact_and_fallacies_response`) inside `retry_once_after_delay(...)` directly (calling `request_chat_completion_once` rather than `request_chat_completion` so retries stay capped at 1 retry / 2 attempts, never 2×2=4).
  - If HTTP attempt 1 returns unparseable text or empty batch/non-JSON analysis, HTTP sleeps `1s` and retries once; if attempt 2 still fails, it returns `Err(...)` so `LlmBackend::Http { client, fallback }` falls back cleanly to `LlmCliRunner`.

#### C. `AiService::get_or_create_summary` & `analyze_entry_impact_and_fallacies` (`ai/mod.rs`)
- Change signature of `AiService::get_or_create_summary` to return `Result<String, String>`:
  - Cache hit (`record.model != "extractive-fallback" && !record.summary.is_empty()`) -> `Ok(record.summary)`.
  - Calls `self.llm.summarize(title, &content).await`:
    - If `model == "extractive-fallback"` or `summary.trim().is_empty()`: do **not** persist `"extractive-fallback"` in `AiDb`; return `Err("LLM summary unavailable".to_string())`.
    - Otherwise persist `self.db.save_summary(entry_id, &summary, &model).await.ok()` and return `Ok(summary)`.
- In `AiService::analyze_entry_impact_and_fallacies` (`ai/mod.rs:136-142`): guard cache hit with `record.model != "extractive-fallback"`.

---

### 3. Prefetch Coordination, Legacy Fallback Guards & Option 1 Route Contracts (`crates/server/src/ai/prefetch.rs`, `crates/server/src/routes/entries.rs`)

#### A. `AiPrefetchService::run_prefetch` (`prefetch.rs`)
- After `ai.llm.summarize_batch_structured(&articles_for_llm).await`, check `if model == "extractive-fallback"`: log warning and return without writing `"extractive-fallback"` rows into `AiDb` (so detail drawer never renders extractive fallback as a completed AI rewrite, and `executive_brief_htmx` detects prefetch failure immediately).

#### B. `executive_brief_htmx` (`entries.rs`)
- **Coordinated `in_flight` wait:** Replace fixed `for _ in 0..40` (4s) loop with a polling loop up to `2500` iterations (`250s` max at `100ms` interval — safely above `LlmHttpClient` `61s` + `LlmCliRunner` `181s` + article extraction headroom):
  1. Sleep `100ms`.
  2. Check `ai_service.db.get_page_brief_by_key(&page_key).await` — if `Some(brief)`, render `ExecutiveBriefCardTemplate` + OOB titles and return `200 OK`.
  3. Check `let still_in_flight = ai_service.in_flight.lock().await.contains(&page_key);` — if `!still_in_flight`, re-check `get_page_brief_by_key(&page_key)` once (guards DB commit vs `InFlightGuard` drop ordering) and, if still `None`, break immediately and return `format!("{}{}", render_brief_error(&retry_url), crate::error::render_llm_error_toast())` (avoiding duplicate retry attempts on initial load and returning right away on fast failures).
- **Error helper:** Combine `render_brief_error(&retry_url)` with `crate::error::render_llm_error_toast()` on all brief error responses.
- **Batch failure & `top.is_empty()` persistence rules (`entries.rs:951-1029`):**
  1. **Full failure (`model == "extractive-fallback"` OR `(batch_res.top.is_empty() && batch_res.items.is_empty())`):** skip DB persistence (`save_batch_results` is not called) and return `format!("{}{}", render_brief_error(&retry_url), crate::error::render_llm_error_toast())`.
  2. **Partial real-LLM response (`model != "extractive-fallback"` AND `batch_res.top.is_empty()` AND `!batch_res.items.is_empty()`):** persist the valid non-fallback `summary_inputs` (`brief_text = None` so no empty brief is saved), emit OOB title spans for valid `suggested_title` items, and return `format!("{}{}{}", render_brief_error(&retry_url), oob_titles_html, crate::error::render_llm_error_toast())`.

#### C. `get_entry_ai_summary_htmx` (`entries.rs:1089-1106`)
- **Miniflux `404` / not-found (`None`/`null` entry_data) OR Askama `AiSummaryCardTemplate.render()` error:** return the existing `<div class="ai-summary-card text-xs text-muted-foreground italic mt-3 pt-3 border-t border-border font-sans">Summary unavailable</div>` (`200 OK`) with **no** `AiErrorCardTemplate` and **no** `render_llm_error_toast()`. Preserves [`test_hx_entries_ai_summary_requires_auth_and_verifies_ownership`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1222).
- **LLM failure (`ai_service.get_or_create_summary(id, &url, &title, &teaser).await` returns `Err(_)`):** return `AiErrorCardTemplate` (`title: "AI Summary Unavailable"`, `message: crate::error::LLM_HIGH_DEMAND_ERROR_MESSAGE`, `retry_url: &format!("/hx/entries/{id}/ai-summary")`, `target: &format!("#ai-summary-slot-{id}")`, `swap: "outerHTML"`, `container_id: &format!("ai-summary-slot-{id}")`) concatenated with `crate::error::render_llm_error_toast()`, returning `200 OK`.
- **LLM success (`Ok(summary)`):** render `AiSummaryCardTemplate { summary }`.

#### D. `get_entry_detail_htmx`, `get_entry_analysis_htmx` & `get_entry_rewrite_htmx` (`entries.rs`)
- **Legacy `"extractive-fallback"` DB guards:**
  - `get_entry_detail_htmx` (`entries.rs:1374-1390`): check `let is_real_ai = record.model != "extractive-fallback";` so legacy `"extractive-fallback"` rows do not supply `rewrite`, `summary`, `suggested_title`, `impact`, or `fallacies` (falling back to `teaser_str` / `summary` with `has_analysis = false`).
  - `get_entry_analysis_htmx` (`entries.rs:1625-1632`): require `record.model != "extractive-fallback"` on the cache hit.
  - `get_entry_rewrite_htmx` (`entries.rs:1717-1726` and `1783-1791`): require `record.model != "extractive-fallback"` both on the cached `record.rewrite` check and when reading `record.summary` in the `Err(_)` graceful-fallback block.
- **Option 1 error toast parity:**
  - `get_entry_analysis_htmx`: on `Err(_)`, append `crate::error::render_llm_error_toast()` to `card_html`.
  - `get_entry_rewrite_htmx`: on `Err(_)`, append `crate::error::render_llm_error_toast()` to `full_html` (keeping `<div id="ai-rewrite-box" ...>{error_card}{fallback_html}</div>`).

---

## Proposed Has (parent or labeled solo applies to card)

Stage records proposed values only — parent after Parent-relay or labeled solo (acting as orchestrator) writes card Has via Always §4.

- **Has UI:** `yes`
- **Has API:** `yes`
- **Has DB:** `no`
- **Has auth/PII/schema:** `no`

## Has API / DB (proposed)

- **`Has API = yes`** (not `tiny-contract-tweak`): Changes multiple HTMX API response contracts (`/hx/executive-brief`, `/hx/entries/{id}/ai-summary`, `/hx/entries/{id}/analysis`, `/hx/entries/{id}/rewrite` returning `AiErrorCardTemplate` + OOB `ToastTemplate`, while preserving `404` `"Summary unavailable"` on `/ai-summary`), backend LLM retry & HTTP validation/fallback semantics, and legacy `"extractive-fallback"` cache guards across `crates/server/src/ai/` and `crates/server/src/routes/entries.rs`.
- **`Has DB = no`**: No SQL schema, migration, or query changes to `article_ai_summaries` or `page_executive_briefs_v2`.

---

## Reference Sources

- **Codebase touchpoints:**
  - [`crates/ui-components/templates/pages/index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L165-L180) (`#entries-loading-overlay`, `showEntriesLoadingState`, `hideEntriesLoadingState`, `htmx:afterSwap`, `showToast`)
  - [`crates/ui-components/templates/partials/ai_grouped_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html#L17-L25) (`.ai-summary-slot`)
  - [`crates/ui-components/templates/base.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L576-L594) (`htmx:afterSwap` live-root Lucide hydration, `htmx:oobAfterSwap` `#toast-container`)
  - [`crates/server/src/ai/llm.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L474-L1158) (`LlmCliRunner`, `LlmHttpClient`, `LlmBackend`)
  - [`crates/server/src/ai/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/mod.rs#L66-L171) (`AiService`)
  - [`crates/server/src/ai/prefetch.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/prefetch.rs#L50-L185) (`AiPrefetchService::run_prefetch`)
  - [`crates/server/src/routes/entries.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L677-L1815) (`executive_brief_htmx`, `get_entry_ai_summary_htmx`, `get_entry_detail_htmx`, `get_entry_analysis_htmx`, `get_entry_rewrite_htmx`)
  - [`crates/server/src/routes/mod.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1188-L1260) (`test_hx_entries_ai_summary_requires_auth_and_verifies_ownership`, `test_hx_entries_ai_summary_guarantees_200_swap_on_failure`)
  - [`crates/server/src/error.rs`](file:///Users/ptquang86/ws/my-ui/crates/server/src/error.rs#L7-L17) (`LLM_HIGH_DEMAND_ERROR_MESSAGE`, `render_llm_error_toast`)
- **Primary official documentation (verified via web search):**
  - Tokio `tokio::time::sleep` & `tokio::sync::Semaphore`: https://docs.rs/tokio/latest/tokio/time/fn.sleep.html · https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
  - Reqwest `ClientBuilder::timeout`: https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
  - HTMX `outerHTML`, `htmx:afterSwap`, `hx-swap-oob`: https://htmx.org/docs/#swapping · https://htmx.org/events/#htmx:afterSwap · https://htmx.org/attributes/hx-swap-oob/
  - Lucide `createIcons({ root })`: https://lucide.dev/guide/packages/lucide
  - Tailwind CSS `motion-reduce` / `animate-pulse`: https://tailwindcss.com/docs/hover-focus-and-other-states#reduced-motion
  - MDN `aria-busy`: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-busy
