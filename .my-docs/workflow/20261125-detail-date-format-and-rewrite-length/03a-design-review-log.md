Result: clean

# Design Review (Mode simple): Details Date Format (`DD/MM/YYYY`), Longer AI Rewrite (3–5 Sentences), Guest Client Details & Tab View Rules

**Stage:** `design-verify-phase` (`my-dev-flow-v3-design-review`)  
**Overall Result:** `clean`  
**Isolated Surfaces:**
- **API Contract Review:** `skipped` (`Has API = no` — updates existing HTML partials/pages and internal LLM prompt templates; `GET /hx/entries/{id}` contract stays intact)
- **DB Design Review:** `skipped` (`Has DB = no` — PostgreSQL schema untouched; IndexedDB `my_news_guest_db` keeps existing schema version `2`)

**Findings Summary:** `0` Critical · `0` Major · `4` Minor implementation hardening notes (non-blocking).

---

## 1. Key Takeaway

The updated design in [03-design.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/03-design.md) and task breakdown in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/04-tasks.md) accurately address both the Round 1 baseline (`DD/MM/YYYY` date formatting + 3–5 sentence AI rewrites) and the Round 2 backloop requirements (`auth_enabled == false` client-side detail rendering without `author`/`readingTime` or `GET /hx/entries/{id}`, plus **Unread** → Magazine vs. **Today**/**All** → List with `#ai-toggle-btn` hidden). All gates pass with zero Critical or Major findings.

---

## 2. Gate & Scope Verification

- **Design Present:** Verified (`03-design.md` `Result: done`).
- **Grill Closed:** Verified (`02b-grill.md` `Result: frontier-empty`; Round 1 Q1–Q4 and Round 2 Q5–Q7 settled).
- **design-update Paid:** Verified (`00-run.md` Run log shows `design-update — settle-gen:1` completed after `design-backloop`).
- **Has Flags Confirmation:** `Has UI = yes`, `Has API = no`, `Has DB = no`, `Has auth/PII/schema = no` (no flip).

---

## 3. Technical, Security & UI Craft Verification

### 3.1 Round 1 Baseline (Date Format + 3–5 Sentence AI Rewrite)
- Confirmed intact across [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L74-L94), [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L130-L150), and [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L616-L633).

### 3.2 Guest Mode List Payload & Sanitization (`ui-components` & `server`)
- **Pre-AI Body Selection & Sanitization:** Selecting `raw_body` (`content_str`, falling back to `summary_str` when empty) before `ai_summaries` overwrites `entry.summary` in [pages.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/pages.rs#L150-L220) and [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L333-L413) guarantees guest details show Miniflux `content` via [sanitizer.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/sanitizer.rs#L11-L53) (`ammonia 4.x`, stripping `<script>`, `<iframe>`, and `onerror` while enforcing `target="_blank"` and `rel="noopener noreferrer"`).
- **WHATWG-Safe JSON Data Block:** `EntryListTemplate::guest_entries_json()` escapes `&`, `<`, `>`, `\u{2028}`, and `\u{2029}` to `\u0026`, `\u003c`, `\u003e`, `\u2028`, and `\u2029` inside `<script id="entries-guest-data" type="application/json">{{ guest_json|safe }}</script>` at the end of [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L227-L232), preventing premature `</script>` termination and preserving `#entries-list-view` / `#entries-magazine-view` structure. Returning `None` when `sanitized_content` is `None` keeps authenticated HTML payloads and all existing `EntryListTemplate` unit tests unchanged.
- **No Author / Reading Time Exposure:** Setting `reading_time: 0` and `author: None` in guest mode across `pages.rs`, `render_entries_list_html`, and `get_entry_detail_htmx`, and omitting both from `GuestEntryPayload` and client `renderGuestEntryDetail`, satisfies Q5 and Q6 while keeping [routes/mod.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/mod.rs#L1331-L1377) and [e2e_tests.rs](file:///Users/ptquang86/ws/my-ui/crates/server/tests/e2e_tests.rs#L840-L940) passing.

### 3.3 Guest Client Detail Rendering & Tab View Rules (`index.html` & `Has UI = yes`)
- **Zero Guest Detail Network Requests:** Short-circuiting `openEntryDetail(id)` when `!AUTH_ENABLED` before `getCachedArticle` and `htmx.ajax('GET', '/hx/entries/...')` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L635-L717)) eliminates `/hx/entries/{id}`, `/analysis`, and `/rewrite` requests in guest mode, while `safeHref(rawUrl)` blocks non-`http(s)`/`/` link schemes.
- **Tab View Locking & Toggle Visibility:** `getDefaultViewForState` and `shouldHideAiToggle` enforce Magazine (`'ai'`) on initial load and **Unread**, List (`'list'`) with `#ai-toggle-btn` hidden (`hidden` class) on **Today** and **All** (including guest `renderLocalAllArticles()` wrapped in `#entries-list-view`), and List (`'list'`) with `#ai-toggle-btn` visible on **Starred** and **Search**.
- **UI Craft & Accessibility:** Guest detail view reuses Annuaire tokens (`bg-card`, `border-border`, `prose dark:prose-invert`, `max-w-[44rem] mx-auto`), semantic `<time datetime="..." data-date-format="dd/mm/yyyy">`, clear empty/offline fallback copy with `Back to List`, and zero hardcoded viewport breakpoints.

### 3.4 TDD Plan Verification (`04-tasks.md`)
- All 5 new Round 2 test cases in [04-tasks.md](file:///Users/ptquang86/ws/my-ui/.my-docs/workflow/20261125-detail-date-format-and-rewrite-length/04-tasks.md) (plus updating `test_index_template_renders_ai_toggle_button_and_scripts` in [templates.rs](file:///Users/ptquang86/ws/my-ui/crates/ui-components/src/templates.rs#L2771-L2847) and filling `sanitized_content: None` on existing `EntryViewModel` initializers) are sequenced Red-before-Green and cover all new guest payload, URL/XSS, detail omission, and tab view rules.

---

## 4. Minor Implementation Directives (Non-Blocking)

1. **Read `evt.detail.requestConfig?.elt` in `htmx:afterSwap` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L916-L995)):** Under HTMX 1.9.12 ([base.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/base.html#L77)), `evt.detail.elt` inside `htmx:afterSwap` references the swapped target container (`#entries-list`), not the triggering button or form. Check `const triggerElt = evt.detail.requestConfig && evt.detail.requestConfig.elt;` when detecting `.filter-pill`, `#search-form`, or `data-status` in `htmx:afterSwap` so non-cached **Unread** tab clicks reliably restore Magazine view (`'ai'`) and `#search-form` swaps reliably detect search state.
2. **Flip `currentViewMode` in `toggleAiView()` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L203-L208)):** Update `toggleAiView()` to derive `next` from `currentViewMode` (`currentViewMode === 'ai' ? 'list' : 'ai'`) rather than `localStorage.getItem('my_news_view_mode')`, and pass the active status and `isSearchActive` into `applyAiViewMode(next, activeStatus, isSearchActive)`. Otherwise, toggling on **Starred** or **Search** while `localStorage` still holds `'ai'` would require two clicks.
3. **Sync `?entry=` URL, panel reveal classes, and `openGuestDb()` upgrade handler in `openGuestEntryDetail` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L647-L695)):** Because `openEntryDetail(id)` short-circuits when `!AUTH_ENABLED`, ensure `openGuestEntryDetail(numId)` updates `?entry=${numId}` via `history.pushState`, applies `has-content` / `animate-slide-in-right` / removes `hidden` on `#detail-panel`, calls `markLocallyRead(numId)` and `syncDomReadStates()` when found, and uses a shared `openGuestDb()` helper with `onupgradeneeded` (creating `read_entries` and `all_articles` at version `2`) so cold deep links (`/?entry={id}` or `/entries/{id}`) do not race ahead of `initGuestDb()`.
4. **Re-hydrate `guestEntriesMap` and reset search/view state in `applyCachedList` & `renderLocalAllArticles` ([index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L369-L385)):** In `applyCachedList(cachedHtml)`, reset `isSearchActive = false`, apply `getDefaultViewForState(activeStatus, false)` (instead of stale `localStorage`), and call `hydrateGuestEntriesFromDom()` when `!AUTH_ENABLED`. In `renderLocalAllArticles()`, set `isSearchActive = false` and call `applyAiViewMode('list', 'all', false)` on completion (including the empty state).

---

## 5. External Reference Sources

- WHATWG HTML Living Standard — Restrictions for contents of script elements & JSON data blocks: https://html.spec.whatwg.org/multipage/scripting.html#restrictions-for-contents-of-script-elements
- Ammonia 4.x HTML Sanitizer (`docs.rs`): https://docs.rs/ammonia/latest/ammonia/
- Askama 0.12 `safe` filter (`docs.rs`): https://docs.rs/askama/latest/askama/
- `serde_json::to_string` (`docs.rs`): https://docs.rs/serde_json/latest/serde_json/fn.to_string.html
- htmx Events Reference (`htmx:beforeRequest`, `htmx:afterSwap`, `detail.elt`, `detail.requestConfig.elt`): https://htmx.org/reference/#events
