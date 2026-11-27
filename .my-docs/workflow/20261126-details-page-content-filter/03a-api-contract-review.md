Result: clean

# API Contract Review — Details Page Content Filter (Re-verify Round 2)

## Key Takeaways

- **Contract verdict:** **Clean** — all **2 Critical** and **2 Major** API/payload contract findings from Round 1 are fully resolved in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md) and [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md). Zero Critical or Major issues remain.
- **Auth vs. Guest gate & serde safety:** `EntryListTemplate` adds `pub auth_enabled: bool` (passed from all server list routes and template literals) so `guest_entries_json()` gates `#entries-guest-data` on `!self.auth_enabled && !self.entries.is_empty()` without relying on `sanitized_content`, while `EntryViewModel` marks `rewrite` and `body_images` with `#[serde(default)]`.
- **`/hx/executive-brief` OOB sync contract:** Replaces naked `<script hx-swap-oob>` with `<div id="executive-brief-sync-data" hidden hx-swap-oob="outerHTML" data-entries="..."></div>` across all OOB-emitting paths (cache hit, in-flight wait, and fresh synthesis including Case C partial responses), syncs `#entries-guest-data` (WHATWG-escaped) before `filterCache.set`, and moves `hydrateGuestEntriesFromDom()` to `isEntriesSwap` only.
- **Entry-scoped rewrite/analysis orchestration & URL parity:** `GET /hx/entries/{id}/rewrite` emits `HX-Trigger-After-Settle: ai-rewrite-done-{id}` across all responses, uses `AnalysisCardsTemplate { oob: true }` for OOB `#ai-analysis-box` swaps, embeds `data-fallback-summary` on `#ai-rewrite-box`, and restricts `extract_article_images` to `http://` / `https://` URLs.

---

## Surfaces Reviewed

- `GET /hx/entries/{id}` (`get_entry_detail_htmx`, `EntryDetailTemplate`, [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html))
- `GET /hx/entries/{id}/rewrite` (`get_entry_rewrite_htmx`)
- `GET /hx/entries/{id}/analysis` (`get_entry_analysis_htmx`, `AnalysisCardsTemplate`, [analysis_cards.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards.html))
- `GET /hx/executive-brief` (`executive_brief_htmx`)
- Embedded `#entries-guest-data` JSON (`EntryViewModel`, `EntryListTemplate`, `GuestEntryPayload`, [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html))

---

## Round 1 Resolution Verification

| Round 1 Finding | Status | Verified Resolution in Design & Tasks |
|---|---|---|
| **#1 [CRITICAL]** `EntryListTemplate::guest_entries_json()` loses Auth-vs-Guest gate without `sanitized_content`; missing `#[serde(default)]` on `EntryViewModel` | **Resolved** | `EntryListTemplate` adds `pub auth_enabled: bool` (`guest_entries_json()` returns `None` when `self.auth_enabled \|\| self.entries.is_empty()`); `index_page`, `render_entries_list_html`, `mark_feed_read_htmx`, `mark_all_read_htmx`, and tests pass `auth_enabled`. `EntryViewModel` adds `#[serde(default)]` on `rewrite` and `body_images` and initializes them in `render_ai_entries_list` and `mark_entry_read_htmx` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L26-L30), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L58-L60)). |
| **#2 [CRITICAL]** `/hx/executive-brief` sync overwritten by `hydrateGuestEntriesFromDom()`, naked `<script hx-swap-oob>`, missing Case C / non-title cached sync, and `brief`/`rewrite` preference | **Resolved** | `entry_list.html` places `<div id="executive-brief-sync-data" hidden></div>` (boolean `hidden`, no `class="hidden"`). `executive_brief_htmx` filters `extractive-fallback`, prefers `!item.brief.trim().is_empty()` with fallback to `item.rewrite`, and emits `#executive-brief-sync-data` OOB on cache hit, in-flight wait, and fresh synthesis (including Case C partial responses). Client runs `syncExecutiveBriefUpdates()` on `isBriefSwap`, updates WHATWG-escaped `#entries-guest-data` before `filterCache.set`, and runs `hydrateGuestEntriesFromDom()` only on `isEntriesSwap` while preserving existing non-empty `rewrite` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L48), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L75-L87)). |
| **#3 [MAJOR]** Unscoped `ai-rewrite-done` race, missing Auth network fallback, `AnalysisCardsTemplate` OOB omission, missing `"No summary available."` empty fallback | **Resolved** | `/hx/entries/{id}/rewrite` emits `HX-Trigger-After-Settle: ai-rewrite-done-{id}` on all responses, returns rewrite-only `#ai-rewrite-box` plus `AnalysisCardsTemplate { oob: true }` when `impact`/`fallacies` are non-empty, and defaults empty fallback to `"No summary available."` on `!config.ai_enabled` and LLM `Err(_)`. `entry_detail.html` and `renderGuestEntryDetail` embed `data-fallback-summary` on `#ai-rewrite-box` and listen for `ai-rewrite-done-{id} from:body` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L35-L46), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L66-L73), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L95-L99)). |
| **#4 [MAJOR]** `extract_article_images` vs. `safeHref` scheme parity and `EntryDetailTemplate.ai_enabled` spec omission | **Resolved** | `extract_article_images` restricts extracted `src` values to case-insensitive `http://` and `https://` URLs; `EntryDetailTemplate` explicitly adds `pub ai_enabled: bool` and `safe_entry_url()` ([03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L31-L32), [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md#L52)). |

---

## Minor Observations (Non-Blocking)

- **Consecutive retry failure on `#ai-rewrite-box` (`index.html`):** When a user clicks the `Retry` button on an already-errored `#ai-rewrite-box` and that retry fails at the HTTP/network layer (`htmx:responseError` / `htmx:sendError`), `hx-get` lives on the Retry `<button>` (`evt.detail.elt`) rather than `#ai-rewrite-box` (`target`). During Task 4 implementation, resolve `retryUrl` via `target.getAttribute('hx-get') || evt.detail.elt?.getAttribute('hx-get') || evt.detail.requestConfig?.path`, guard `if (!target.isConnected) return;`, and preserve `data-fallback-summary` on the replaced error `#ai-rewrite-box` so subsequent retries still carry the URL and Auth fallback text.
- **Bookmark button `data-url` in `entry_detail.html`:** Line 58 passes `data-url="{{ entry.url }}"` to `triggerNativeBookmark`; using `{{ self.safe_entry_url() }}` keeps parity with the `Original` link.

---

## Reference Sources

- HTMX 1.9 `hx-swap-oob` & `HX-Trigger-After-Settle`: <https://htmx.org/attributes/hx-swap-oob/> · <https://htmx.org/headers/hx-trigger/>
- Repo contracts & templates inspected:
  - [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/03-design.md)
  - [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261126-details-page-content-filter/04-tasks.md)
  - [crates/ui-components/src/templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs)
  - [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html)
  - [crates/ui-components/templates/partials/entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html)
  - [crates/ui-components/templates/partials/analysis_cards.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/analysis_cards.html)
  - [crates/ui-components/templates/partials/ai_error_card.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/ai_error_card.html)
  - [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html)
  - [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs)
  - [crates/server/src/routes/pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs)
