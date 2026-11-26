Result: clean

# Design Review Log — Skeleton Loading, LLM 1s Auto-Retry, and Option 1 Failure Feedback

- **Slug:** `20261126-skeleton-loading-llm-retry`
- **Stage:** `design-verify-phase` (Mode `simple`, Review profile `lite`)
- **Has UI / API / DB / auth/PII/schema:** `yes | yes | no | no`
- **Isolated API:** `clean` ([`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md))
- **Isolated DB:** `skipped` (Notes-skip — `Has DB = no`)

## Key Takeaways

- **Overall Verdict — `clean` (0 Critical, 0 Major):** All 4 Major and 4 Minor findings from Round 1 are resolved across [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) and [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md), and the Isolated API review is `clean`.
- **HTMX `outerHTML` lifecycle & Lucide hydration fixed:** [`index.html#L1201-L1215`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L1201-L1215) now matches `#ai-analysis-box` and `#ai-rewrite-box` swaps explicitly and excludes `.is-error` cards from IndexedDB caching, while [`base.html#L576-L580`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L576-L580) resolves the connected live DOM node before `lucide.createIcons({ root: liveRoot })`.
- **Accessible view-aware skeletons & toast safety verified:** `#entries-skeleton-magazine` and `#entries-skeleton-list` (`min-h-[48px]` matching `.ar-list-item`) satisfy `ANNUAIRE_STYLE_GUIDE.md` and accessibility standards, `#ai-summary-slot-{{ item.id }}` preserves `mt-3` spacing inside a wrapper `<div>`, and Guest `showToast()` + OOB deduplication avoid double timers and duplicate error pills.
- **TDD plan ready:** [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) defines 6 named failing-first test cases across all 3 tasks with accurate crate/module filters and exact test update targets.

---

## FIRST Gate Verification

| Check | Evidence | Status |
|---|---|---|
| **Design present** | [`02-analysis.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/02-analysis.md) (`Result: done`), [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) (`Result: done`), [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) (6 planned tests) | **Pass** |
| **Grill closed** | [`02b-grill.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/02b-grill.md) (`Result: frontier-empty`) | **Pass** |
| **Stale / unpaid gates** | No `design-stale`, `design-update` paid for `settle-gen:1` | **Pass** |
| **Isolated API review** | [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md) (`Result: clean`) | **Pass** |
| **Isolated DB review** | `Has DB = no` (no schema/query changes; Notes-skip only) | **Skipped** |

---

## Round 1 Findings Verification (UI, Lifecycle, TDD)

| Prior Finding | Resolution Verified in `03-design.md` & `04-tasks.md` | Status |
|---|---|---|
| **Major #1: HTMX `outerHTML` detached `evt.detail.target` broke `index.html:1201` detail caching & `base.html:578` Lucide hydration** | [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md#L142-L178) adds explicit `targetId === 'ai-analysis-box' || targetId === 'ai-rewrite-box'` and `evt.target?.closest?.('#detail-panel')` checks plus `.is-error` exclusion in `index.html`, and updates `base.html` `htmx:afterSwap` to resolve the live connected replacement root (`swapTarget.isConnected ? swapTarget : (swapTarget.id ? document.getElementById(swapTarget.id) : null) || ...`) before `lucide.createIcons`. | **Resolved** |
| **Major #2–4: API / Retry / 404 contracts** | Verified clean in [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md). | **Resolved** |
| **Minor #1: Legacy `"extractive-fallback"` DB rows & `batch_res.top.is_empty()` clarity** | Guarded across detail/analysis/rewrite handlers and `AiService`; partial vs full batch failure rules explicitly specified in `03-design.md` §3.B & §3.D. | **Resolved** |
| **Minor #2: Toast deduplication vs Guest `showToast()`** | Guest `showToast()` sets `data-auto-dismiss="true"` before appending to `#toast-container`, preventing `base.html`'s `.pointer-events-auto:not([data-auto-dismiss])` selector from adding a duplicate `5000ms` timer while deduplicating active OOB toasts by trimmed `span.textContent`. | **Resolved** |
| **Minor #3: Skeleton row height & `#ai-summary-slot-{id}` spacing** | `#entries-skeleton-list` rows use `min-h-[48px]` matching `.ar-list-item` ([`base.html#L294`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L294)); [`ai_grouped_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html#L17-L25) wraps `#ai-summary-slot-{{ item.id }}` in `<div class="mt-3">` so `outerHTML` swap to `AiErrorCardTemplate` retains top spacing while `AiSummaryCardTemplate` collapses cleanly in block flow. | **Resolved** |
| **Minor #4: `04-tasks.md` test filter typo (`llm::tests` vs `ai::llm::tests`)** | Fixed to `cargo test -p server ai::llm::tests` in [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md#L35-L42). | **Resolved** |

---

## Checklist Matrix

| Review Axis | Checks & Evidence | Status |
|---|---|---|
| **API Contract (`03a-api-contract-review.md`)** | 0 Critical, 0 Major (`Result: clean`) | **Pass** |
| **Database Design** | `Has DB = no` (no schema/query changes; Notes-skip) | **Skipped** |
| **UI Craft, Accessibility & `ANNUAIRE_STYLE_GUIDE.md`** | View-aware skeletons (`bg-muted`, `border-border`, zero shadows, `min-h-[48px]`, `animate-pulse motion-reduce:animate-none`, `role="status"`, `aria-live="polite"`, `aria-busy`, `sr-only`); error cards preserve `mt-3` wrapper and Annuaire tokens. | **Pass** |
| **Client HTMX Lifecycle & Caching** | Connected-node Lucide hydration (`base.html:576-580`), `#ai-analysis-box`/`#ai-rewrite-box` swap matching + `.is-error` exclusion (`index.html:1201-1215`), non-colliding Guest/OOB toast auto-dismiss (`data-auto-dismiss="true"`), existing loading/stale-XHR contracts preserved ([`templates.rs#L4244-L4349`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L4244-L4349)). | **Pass** |
| **TDD Plan (`04-tasks.md`)** | 6 named failing-first tests covering every UI, retry, HTTP validation/fallback, `in_flight` wait, 404 separation, and Option 1 contract requirement; all modified test assertions explicitly listed (`entries.rs:3519, 3600, 3711, 3814`, `mod.rs:1258, 1259`). | **Pass** |

---

## Verdict

- **Critical / Major Findings:** 0 Critical, 0 Major
- **Overall Result:** `clean`

---

## Reference Sources

- **HTMX `outerHTML`, `htmx:afterSwap`, `hx-swap-oob`:** https://htmx.org/docs/#swapping · https://htmx.org/events/#htmx:afterSwap · https://htmx.org/attributes/hx-swap-oob/
- **Lucide `createIcons({ root })`:** https://lucide.dev/guide/packages/lucide
- **Tailwind CSS `animate-pulse` & `motion-reduce`:** https://tailwindcss.com/docs/hover-focus-and-other-states#reduced-motion
- **MDN `aria-busy`:** https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-busy
- **Tokio `sleep` & `Semaphore`:** https://docs.rs/tokio/latest/tokio/time/fn.sleep.html · https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html
- **Workflow & Source Artifacts:** [`03-design.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03-design.md) · [`04-tasks.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/04-tasks.md) · [`03a-api-contract-review.md`](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-skeleton-loading-llm-retry/03a-api-contract-review.md) · [`index.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) · [`base.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html) · [`ai_grouped_list.html`](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_grouped_list.html)
