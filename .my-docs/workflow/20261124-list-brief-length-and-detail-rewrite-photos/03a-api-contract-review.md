Result: clean

# API Contract Review: List Brief Length & Detail Hero / Body Photos

## 1. Executive Summary
The API surface modifications are strictly additive and backward-compatible. No public HTTP endpoints, routes, query parameters, or response status codes are altered. The additions are isolated to internal Rust crate signatures (`EntryDetailTemplate` struct fields in `ui-components`, `extract_article_images` helper in `server`) and HTMX partial HTML responses.

---

## 2. API Surface Evaluation

### 2.1 HTTP & HTMX Endpoints
- **`GET /hx/entries/{id}` (`get_entry_detail_htmx`)**:
  - **Contract**: Accepts article ID as path parameter; returns `text/html; charset=utf-8` representing `#detail-panel` inner content.
  - **Query & Headers**: No changes to required headers or query parameters.
  - **Status Codes**: 200 OK for successful HTML render; 200 OK with `ErrorCardTemplate` for recoverable HTMX error swaps (unchanged).
  - **HTMX Swap Target**: Targets `#detail-panel`. Out-of-band updates (`#unread-pill`) remain fully functional.
  - **Assessment**: **PASS**. No breaking changes.

- **`GET /hx/entries` (`get_entries_list_htmx`)**:
  - **Contract**: Accepts `EntriesFilterQuery` (status, feed_id, category_id, search, limit, offset, request_brief); returns rendered article cards.
  - **Changes**: Presentation-only updates (`line-clamp-6` on Priority 1, `line-clamp-4` on Priority 2) and prompt brevity guidance.
  - **Assessment**: **PASS**. Fully compliant with existing client contracts.

### 2.2 Data Types & Template Contracts
- **`EntryDetailTemplate` (`crates/ui-components/src/templates.rs`)**:
  - **Existing Fields**: `entry`, `summary`, `auth_enabled`, `rewrite`, `impact`, `fallacies`, `has_analysis`.
  - **New Fields**:
    - `pub hero_image: Option<String>`
    - `pub body_images: Vec<String>`
  - **Nullability & Defaults**: `hero_image` is an `Option<String>`, allowing graceful absence. `body_images` is an empty `Vec<String>` when no additional photos exist.
  - **Assessment**: **PASS**. Struct additions require updating internal constructors across test fixtures, which is documented in Task 4.

- **`extract_article_images` (`crates/server/src/routes/entries.rs`)**:
  - **Signature**: `pub fn extract_article_images(html: &str) -> (Option<String>, Vec<String>)`.
  - **Behavior**: Extracts image URLs from Miniflux entry HTML, strips tracking pixels/feedburner gifs, and returns `(hero_image, body_images)`.
  - **Assessment**: **PASS**. Clean pure function with zero external side effects.

- **AI Prompt & Extractive Contracts (`crates/server/src/ai/llm.rs`, `crates/miniflux/src/summary.rs`)**:
  - **`format_prompt` & `format_batch_prompt_structured`**: Signatures remain unchanged (`&str, &str` and `&[T]`); string output instructs strictly 3 to 5 sentences.
  - **`extract_summary`**: Retains signature `extract_summary(text: &str, sentence_count: usize)`. Teaser caller standardizes on `sentence_count: 3`.
  - **Assessment**: **PASS**.

### 2.3 Security & Content Integrity
- **Escaping & XSS**: Image URLs are rendered via Askama (`{{ hero }}` and `{{ img }}`), ensuring HTML entities (`"`, `<`, `>`, `&`) are safely escaped, preventing attribute breakout.
- **Privacy & Referrer Policy**: All extracted `<img>` tags specify `referrerpolicy="no-referrer"` to prevent leaking user origin to third-party image hosts.
- **Lazy Loading**: `loading="lazy"` prevents bandwidth waste on non-visible images.

---

## 3. Findings & Recommendations
- **Minor Finding (Resolved in Design Review)**: On the hero image tag, error handling must ensure the parent container holding the article title does not get hidden if the hero image fails to load. The `onerror` handler on the hero image should hide the image itself (`this.style.display='none'`) rather than `this.parentElement`, preserving the title overlay and gradient backdrop.

---

## 4. Verdict
**Result:** `clean`  
All API and template contracts are backward-compatible, secure, and properly typed.
