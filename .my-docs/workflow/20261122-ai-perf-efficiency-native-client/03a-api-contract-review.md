Result: clean

# API Contract Review: AI Performance, Efficiency & Native REST Client

**Date:** 2026-11-22  
**Workflow Slug:** `20261122-ai-perf-efficiency-native-client`  
**Review Stage:** `api-contract-review`  
**Target Design:** `.my-docs/workflow/20261122-ai-perf-efficiency-native-client/03-design.md`  

---

## 1. Executive Summary

The API contract design in `03-design.md` defines clear, robust, and industry-standard contracts for integrating native HTTP REST endpoints (OpenAI Chat Completions and Google Gemini REST API) alongside resilient fallback to standard input (`stdin`) CLI execution.

- **Status:** **Zero Critical / Major issues identified**.
- **Overall Verdict:** `Result: clean`.

---

## 2. API Contract Analysis

### 2.1 OpenAI-Compatible Chat Completions Contract

- **Endpoint URL:** `{base_url}/chat/completions` (Default: `https://api.openai.com/v1/chat/completions`, configurable via `LLM_API_BASE_URL`).
- **HTTP Method:** `POST`
- **Request Headers:**
  - `Content-Type: application/json`
  - `Authorization: Bearer {api_key}`
- **Payload Schema:**
  ```json
  {
    "model": "<model>",
    "messages": [
      {
        "role": "user",
        "content": "<batch_prompt>"
      }
    ],
    "response_format": {
      "type": "json_object"
    }
  }
  ```
- **Evaluation:**
  - Matches the standard OpenAI specification for JSON Mode (`response_format: {"type": "json_object"}`).
  - Prompt in `LlmCliRunner::format_batch_prompt` explicitly instructs the model to return JSON, satisfying OpenAI's requirement for JSON mode prompts.
  - Response extraction path `choices[0].message.content` accurately aligns with the standard Chat Completion response schema.

### 2.2 Google Gemini REST Contract

- **Endpoint URL:** `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`
- **HTTP Method:** `POST`
- **Request Headers:**
  - `Content-Type: application/json`
  - Recommended authentication header: `x-goog-api-key: {api_key}` (also supports `?key={api_key}` query param).
- **Payload Schema:**
  ```json
  {
    "contents": [
      {
        "parts": [
          {
            "text": "<batch_prompt>"
          }
        ]
      }
    ],
    "generationConfig": {
      "responseMimeType": "application/json"
    }
  }
  ```
- **Evaluation:**
  - Conforms to Google Gemini `generateContent` v1beta specification.
  - Using `responseMimeType: "application/json"` in `generationConfig` ensures Gemini outputs structured JSON matching the requested schema.
  - Response extraction path `candidates[0].content.parts[0].text` is standard.
  - Parsing gracefully degrades through `serde_json` and regex extraction (`TOP_REGEX` and `ITEM_REGEX`), guarding against edge cases like safety blocks or missing fields.

### 2.3 Resilient CLI Fallback (`stdin` Piping)

- **Execution Model:**
  - Command invocation excludes positional prompt arguments (`self.build_args_without_prompt()`).
  - Uses `Stdio::piped()` for standard input, writes prompt bytes asynchronously, and drops/flushes stdin to signal EOF.
- **Evaluation:**
  - Eliminates OS-level `ARG_MAX` buffer overflow risk when handling large batches of articles.
  - Fully compatible with Simon Willison's `llm` CLI and standard shell LLM wrappers.

### 2.4 Internal Endpoint Contract (`/hx/executive-brief`)

- **Trigger Conditions:**
  - All articles cached in SQLite + uncached page brief -> triggers short cached synthesis prompt (`synthesize_page_brief`).
  - Cache hit -> returns HTML snippet with OOB title updates immediately (<10ms).
- **Response Format:**
  - HTML fragment compatible with HTMX swaps (`hx-swap="outerHTML"` for the card and `#article-title-<id>` outerHTML swaps).

---

## 3. Findings & Observations

### Critical / Major Issues
- **None**. (0 findings).

### Minor Recommendations (Non-blocking)
1. **Gemini Header Authentication:**
   - *Recommendation:* When invoking Google Gemini REST API, prefer passing the API key via the `x-goog-api-key: {api_key}` HTTP header rather than the URL query parameter (`?key=...`) to avoid secret exposure in reverse proxy access logs or tracing spans.
2. **Expose `summarize` on `LlmBackend`:**
   - *Recommendation:* Ensure `LlmBackend` exposes both single-item `summarize(&self, title: &str, content: &str)` and `summarize_batch` so that `AiService::get_or_create_summary` can also take advantage of native HTTP REST without spawning subprocesses.

---

## 4. Conclusion

The API contracts in `03-design.md` are well-structured, secure, and ready for implementation.
