# Idea & Framing: Details Page Date Format (`DD/MM/YYYY`) and Longer AI Rewrite (3–5 Sentences)

## Key Takeaway

We need two targeted updates to the article reading experience in `my-ui`:
1. **Details Page Date Format**: Format `published_at` on the article details page as `DD/MM/YYYY` (for example, `25/11/2026`) instead of showing the raw ISO 8601 / RFC 3339 timestamp.
2. **Longer AI Rewrite (3–5 Sentences)**: Make the AI article rewrite longer and genuinely **3 to 5 full sentences** across both batch generation and on-demand detail generation, lifting the tight word-count squeeze (`30-60 words`) that currently keeps rewrites too short.

Before designing and coding, we need your confirmation on **timezone handling for `DD/MM/YYYY`** and **how much to expand the rewrite length rules** (or you can reply **"just start"** to accept the recommended options).

---

## Current vs. Proposed State

- **Current Date Display**: `get_entry_detail_htmx` in [entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1407-L1411) passes Miniflux's raw `publishedAt` string (e.g., `2026-11-19T12:00:00Z`) straight into [entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L77-L89) for both the hero-image header and standard header.
- **Proposed Date Display**: Display the date in `DD/MM/YYYY` format (e.g., `25/11/2026`) in the details page header, with graceful fallback if a test or feed provides a non-ISO string.
- **Current Rewrite Prompting**: In [llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L83-L185), `format_batch_prompt_structured` already mentions `strictly 3 to 5 sentences`, **but** caps `"r"` at `under 110 words total (sweet spot 30-60 words)` (`first 2 sentences, followed by 2-3 bullet points`). Meanwhile, `format_rewrite_and_analyze_prompt` (used by `/hx/entries/:id/rewrite`) does not mention `3 to 5 sentences` at all, and [extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs#L49-L66) truncates source text to `100` words.
- **Proposed Rewrite Prompting**: Align both rewrite prompt builders to require **3 to 5 full sentences** with enough word budget and extracted context so rewrites read clearly and substantially on the details page.

---

## Scope & Sizing

- **Size**: Small focused UI + AI prompt change (`Mode: simple`, `Review profile: lite`).
- **Has UI (tentative)**: `yes`
- **Has API (tentative)**: `no` (updates existing HTML partial `/hx/entries/{id}` and internal LLM prompt templates; no new endpoint or schema change)
- **Has DB (tentative)**: `no` (existing `article_ai_summaries.rewrite` column stores arbitrary `TEXT`)
- **Has auth/PII/schema (tentative)**: `no`

### Goals
- **Readable Date**: Show `DD/MM/YYYY` consistently in both hero-image and non-hero detail headers.
- **Richer Rewrites**: Generate 3 to 5 full, meaningful sentences for article rewrites across both batch prefetch (`format_batch_prompt_structured`) and single-article rewrite (`format_rewrite_and_analyze_prompt`).
- **Backwards Compatibility**: Keep existing list view behavior (which omits dates per tests) and non-ISO test fixtures intact.

### Non-Goals
- Changing list-view or magazine-view card layouts (aside from `entry.summary` receiving the longer cached rewrite where shared).
- Retroactively rewriting already-cached rows in PostgreSQL (`article_ai_summaries`) or browser IndexedDB (`my_news_article_cache_db`) unless cleared or newly fetched.
- Fixing unrelated `LlmBackend::rewrite_and_analyze` HTTP vs. CLI routing unless needed during design.

### Anti-Goals
- Crashing or showing blank dates if Miniflux returns fractional seconds, non-UTC offsets, or non-RFC 3339 strings.
- Keeping contradictory `30-60 words` instructions that force the LLM to write clipped fragments instead of 3–5 real sentences.

---

## Decisions & Options

### Decision 1: Timezone Handling for `"DD/MM/YYYY"` on the Details Page

Miniflux returns `published_at` as an RFC 3339 timestamp (for example `"2026-11-24T20:00:00Z"` or `"2023-09-10T12:48:43.428196-07:00"`). Near midnight, UTC and a reader's local timezone (such as `+07:00`) fall on different calendar days (`24/11/2026` vs. `25/11/2026`). Also, Docker containers often default to UTC unless `TZ` is configured.

#### Option 1A (Recommended): Server-side `DD/MM/YYYY` fallback + browser local timezone update via `<time>`
- **Explanation**: Parse `published_at` on the server with Chrono (`DateTime::parse_from_rfc3339`) to render a clean `DD/MM/YYYY` fallback inside `<time datetime="...">`, then format it to the reader's local browser timezone (`DD/MM/YYYY`) when the detail panel renders or loads from IndexedDB.
- **Concrete Example**:
  ```html
  <time datetime="{{ entry.published_at }}" data-date-format="dd/mm/yyyy">{{ formatted_published_at }}</time>
  ```
  ```javascript
  // In browser local timezone:
  const d = new Date(el.getAttribute('datetime'));
  if (!Number.isNaN(d.getTime())) {
    const dd = String(d.getDate()).padStart(2, '0');
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    el.textContent = `${dd}/${mm}/${d.getFullYear()}`;
  }
  ```
- **Pros**:
  - Matches the reader's actual local calendar date without needing container `TZ` configuration.
  - Still renders valid `DD/MM/YYYY` directly from Rust if JS is disabled or inspected in HTML responses.
  - Safe fallback for non-ISO test strings (e.g., `"2 hours ago"`).
- **Cons**:
  - Adds a tiny client-side DOM pass in [index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html) when opening or restoring cached details.

#### Option 1B: Server-side only formatting (`chrono::DateTime::parse_from_rfc3339`)
- **Explanation**: Format `published_at` purely in Rust inside `get_entry_detail_htmx` using Chrono `.format("%d/%m/%Y")` (either preserving the feed's parsed offset or converting to server `Local`), falling back to the raw string if parsing fails.
- **Concrete Example**:
  ```rust
  chrono::DateTime::parse_from_rfc3339(raw)
      .map(|dt| dt.format("%d/%m/%Y").to_string())
      .unwrap_or_else(|_| raw.to_string())
  ```
- **Pros**:
  - Pure server-side Rust change; zero JavaScript.
  - Simplest implementation and unit tests.
- **Cons**:
  - Dates near midnight can be off by one day for users in a different timezone from the feed timestamp or container `TZ`.

---

### Decision 2: Expanding Rewrite Length ("from 3 to 5 sentences")

Right now, `format_batch_prompt_structured` says `"Strictly 3 to 5 sentences, under 110 words total (sweet spot 30-60 words)"`, `format_rewrite_and_analyze_prompt` omits `"3 to 5 sentences"` and keeps the same `"30-60 words / under 110 words"` cap, and `ArticleExtractor` caps extracted article input at `100` words.

#### Option 2A (Recommended): Require 3–5 full sentences in both prompts, raise word budget, and extract more source text
- **Explanation**: Update both `format_batch_prompt_structured` and `format_rewrite_and_analyze_prompt` to instruct the LLM to write **3 to 5 full, substantive sentences** (raising the target word budget, e.g., ~90–150 words, max ~180 words, instead of `30-60 words / under 110 words`), and raise `ArticleExtractor` source text truncation (e.g., from `100` to `250` words) so the LLM has enough context to write 3–5 informative sentences.
- **Concrete Example**:
  ```text
  REWRITE RULES FOR "r":
  - Length: Strictly 3 to 5 complete sentences (around 90-150 words, max 180 words).
  - Structure: Inverted pyramid — lead with the main outcome and impact in the first 1-2 sentences, followed by 2-3 complete sentences or concise bullet-point sentences with essential context.
  ```
- **Pros**:
  - Ensures rewrites are actually longer and richer across both batch prefetch and on-demand `/hx/entries/{id}/rewrite`.
  - Prevents 100-word source truncation from starving 5-sentence rewrites of facts.
- **Cons**:
  - Slightly higher LLM token usage per batch/article.
  - Magazine cards (`line-clamp-6` / `line-clamp-4` in [entry_list.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_list.html#L128-L161)) will truncate longer summaries visually, while the details panel displays the full rewrite.

#### Option 2B: Update sentence count in both prompts only (prose 3–5 sentences, keep 110-word cap and 100-word extraction)
- **Explanation**: Add `"strictly 3 to 5 sentences"` to `format_rewrite_and_analyze_prompt` and clarify prose/sentence count in `format_batch_prompt_structured`, keeping the existing `110`-word output cap and `100`-word source extraction limit.
- **Concrete Example**:
  ```text
  "rewrite": "Inverted pyramid rewrite in strictly 3 to 5 complete sentences, under 110 words total. Lead with outcome and key impacts, followed by essential context."
  ```
- **Pros**:
  - Minimal token and prompt change.
- **Cons**:
  - Tight word cap and 100-word extraction can still force short, choppy sentences.

---

## Verification References

- Detail route handler: [crates/server/src/routes/entries.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/routes/entries.rs#L1228-L1452) (`get_entry_detail_htmx`)
- Detail HTML template: [crates/ui-components/templates/partials/entry_detail.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/partials/entry_detail.html#L69-L102)
- Client detail cache & swap hooks: [crates/ui-components/templates/pages/index.html](file:///Users/ptquang86/ws/my-ui/crates/ui-components/templates/pages/index.html#L512-L993)
- Rewrite prompt builders: [crates/server/src/ai/llm.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/llm.rs#L83-L185) (`format_batch_prompt_structured`, `format_rewrite_and_analyze_prompt`)
- Article text truncation: [crates/server/src/ai/extractor.rs](file:///Users/ptquang86/ws/my-ui/crates/server/src/ai/extractor.rs#L49-L78)
- Chrono `DateTime::parse_from_rfc3339` & `strftime`: https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.parse_from_rfc3339
- Askama 0.12 docs: https://docs.rs/askama/0.12.0/askama/
- HTML `<time>` element & `Intl.DateTimeFormat`: MDN Web Docs (`<time>` `datetime` global date-and-time format + `Intl.DateTimeFormat`)
- Miniflux REST API `published_at` RFC 3339 / ISO 8601 format: https://miniflux.app/docs/api.html

---

## Open Question for User

Please confirm our shared understanding and let us know your preference on these two choices (or reply **"just start"** to go with the recommended options):
1. **Date timezone (`DD/MM/YYYY`)**: Should we use **Option 1A (Recommended: server `DD/MM/YYYY` fallback + browser local timezone via `<time>`)** or **Option 1B (server-side Chrono formatting only)**? Also confirm only the details page needs `DD/MM/YYYY` (not `ai_grouped_list.html`).
2. **Rewrite length & structure**: Should we use **Option 2A (Recommended: 3–5 full sentences across both prompts with a higher word budget ~90–150 words and ~250-word source extraction)** or **Option 2B (keep the 110-word cap / 100-word extraction)** — and do you want to keep the **lead + 2–3 bullet points** format or switch to **plain prose sentences without bullets**?
