Result: clean

# Lens: api

**Lens:** api  
**Result:** clean

## API Contract Review Checklist

| Check | Status | Notes |
|---|---|---|
| Typed input and output; create-input vs full entity | **Pass** | `EntryFilterInput`, `EntryStatus`, `OrderDirection`, and `EntryGql` are strongly typed in GraphQL schema. |
| One stable error shape; correct status codes | **Pass** | Axum HTMX error handler returns 200 with `HX-Reswap: none` and toast OOB; unauthenticated returns 303 / `HX-Redirect: /login` or 401 Unauthorized for `/graphql`. |
| Validate at system edges only | **Pass** | Input validation on negative limit/offset in GraphQL resolvers; parameter validation at route boundaries. |
| Lists paginated; filters via query params | **Pass** | `GET /hx/entries` supports `limit`, `offset`, `status`, `feed_id`, `category_id`, `search`. |
| Additive fields; no silent breaking changes | **Pass** | Schema matches `03-design.md` specifications without breaking field signatures. |
| Naming consistent with the repo | **Pass** | CamelCase for GraphQL schema fields (`isBookmarked`, `readingTime`, `unreadCount`); snake_case in Rust internals. |
| Mutating endpoints: idempotent or documented unsafe-to-retry | **Pass** | `toggleBookmark`, `markAllAsRead`, `markFeedAsRead`, `updateEntryStatus` are idempotent or safe batch mutations. |
| Contract in 03-design matches what code exposes | **Pass** | `POST /hx/feeds/{id}/mark-all-read` is mapped to `mark_feed_read_htmx`; `Query.entry` and `Mutation.markFeedAsRead` properly delegate to Miniflux native REST endpoints. |
| Do not leak internals in errors | **Pass** | Upstream errors sanitized into glass toast messages or generic upstream communication error messages. |

## Findings Verification

### API-1 [Resolved - Incomplete Miniflux Client Single-Entry API & Broken GraphQL Entry Resolver]
- **Location**: `crates/miniflux/src/client.rs:275`, `crates/miniflux/src/schema/query.rs:101`.
- **Resolution**:
  1. `MinifluxClient::get_entry(&self, entry_id: i64) -> Result<Entry, MinifluxError>` added, querying `GET /v1/entries/{entryID}`.
  2. `Query::entry` in `crates/miniflux/src/schema/query.rs` delegates directly to `client.get_entry(entry_id)`.
  3. Verified by `test_get_entry_success` and `test_graphql_query_single_entry`.

### API-2 [Resolved - Native Miniflux Feed Mark-as-Read & Route Mapping]
- **Location**: `crates/miniflux/src/client.rs:297`, `crates/miniflux/src/schema/mutation.rs:26`, `crates/server/src/routes/entries.rs:315`.
- **Resolution**:
  1. `MinifluxClient::mark_feed_as_read(&self, feed_id: i64) -> Result<(), MinifluxError>` added, dispatching `PUT /v1/feeds/{feedID}/mark-all-as-read`.
  2. GraphQL `Mutation::mark_feed_as_read` delegates directly to `client.mark_feed_as_read(fid)`.
  3. `POST /hx/feeds/{id}/mark-all-read` mapped in `app.rs:47` and handled by `mark_feed_read_htmx`.
  4. Verified by `test_mark_feed_as_read_endpoint`, `test_graphql_mutation_mark_feed_as_read`, and `test_mark_feed_read_htmx`.

### API-3 [Resolved - Real PKCE Token Exchange in OIDC Auth Callback]
- **Location**: `crates/server/src/auth/oidc.rs:99-154`, `crates/server/src/auth/handlers.rs:67`.
- **Resolution**:
  1. `exchange_token` implemented in `crates/server/src/auth/oidc.rs` performing `POST /token` with `grant_type=authorization_code` and `code_verifier`.
  2. Invoked in `callback_handler` with error handling and session construction.
  3. Verified by `test_oidc_callback_clears_temp_cookie` and `test_e2e_authenticated_session_flow`.
