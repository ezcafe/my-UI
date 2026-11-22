# API contract review

**Result:** clean

## Scope & Endpoints

Review of AI feature gating contract changes across Axum route handlers and configuration:
1. `GET /hx/entries/{id}/ai-summary` (`get_entry_ai_summary_htmx` in `crates/server/src/routes/entries.rs`)
2. `GET /hx/entries?status=ai` (`list_entries_htmx` in `crates/server/src/routes/entries.rs`)

---

## Contract Evaluation

### 1. `GET /hx/entries/{id}/ai-summary`

- **Purpose**: Fetch or trigger an AI-generated summary for an RSS entry via HTMX.
- **Handler Signature**:
  ```rust
  pub async fn get_entry_ai_summary_htmx(
      _user: AuthenticatedUser,
      State(schema): State<AppSchema>,
      State(config): State<Arc<Config>>,
      State(ai_service): State<Arc<AiService>>,
      headers: HeaderMap,
      Path(id): Path<i64>,
  ) -> Response
  ```
- **Authentication & Gating Behavior**:
  - **Auth Enabled (`AUTH_ENABLED=true`), AI Enabled (`AI_ENABLED=true`)**: Authenticated users successfully invoke `ai_service.get_or_create_summary` and receive the rendered summary HTML partial. Unauthenticated users are redirected to `/login` via `AuthenticatedUser` rejection.
  - **Auth Enabled (`AUTH_ENABLED=true`), AI Disabled (`AI_ENABLED=false`)**: Authenticated user session passes, but handler checks `if !config.ai_enabled` and immediately returns `handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()))` (HTTP 404). Bypasses `AiService` and LLM CLI completely.
  - **Auth Disabled (`AUTH_ENABLED=false`)**: Guest mode resolves `_user` to `guest`. `config.ai_enabled` evaluates strictly to `false` (`auth_enabled && AI_ENABLED`). Handler immediately returns HTTP 404 / NotFound without calling LLM CLI.
- **Consistency**: Exact parity with error status and payload format of `/hx/entries?status=ai`.

### 2. `GET /hx/entries?status=ai`

- **Purpose**: Render entries list filtered by AI digest status.
- **Handler Signature**: `list_entries_htmx` extracting `Query<EntriesQuery>`, `State(schema)`, `State(cache)`, `State(config)`, `headers: HeaderMap`.
- **Gating Behavior**:
  - When `query.status.as_deref() == Some("ai")`:
    - Checks `if !config.ai_enabled`. If disabled, returns `handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()))`.
    - If enabled, invokes `render_ai_entries_list`.
- **Verdict**: Correctly enforces gating and returns 404 before database or schema access.

---

## Findings

- **Critical**: 0
- **Major**: 0
- **Minor / Observational**: 0
  - Axum extractor ordering in `get_entry_ai_summary_htmx` adheres to Axum rules (path and state extractors positioned correctly).
  - Error responses utilize `handle_htmx_error`, returning proper HTMX swap-compatible error alert templates with HTTP 404 status.
