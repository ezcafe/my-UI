# Design

**Result:** done

## Overview

This design secures AI features behind both authentication and feature flags, while resolving Docker build failures on Raspberry Pi OS and modern toolchains.

---

## Docker Builder Base Image Options

### Option 1 (Recommended): `rust:1-slim-bookworm` (Floating 1.x Stable on Bookworm)

- **Explanation**: Uses the official Rust stable 1.x series on Debian Bookworm. It automatically provides compiler updates required by newer dependencies while preserving Debian Bookworm glibc compatibility with the runtime container (`debian:bookworm-slim`).
- **Concrete Example**:
  ```dockerfile
  FROM rust:1-slim-bookworm AS builder
  ```
- **Pros**:
  - Automatically supports modern dependencies requiring rustc >= 1.95 (e.g., `lock_api 0.5.0`, `parking_lot 0.13.0`).
  - Native multi-architecture support (`linux/amd64`, `linux/arm64/v8`, `linux/arm/v7`) ideal for Raspberry Pi OS.
  - Matches Debian Bookworm glibc ABI exactly with the `debian:bookworm-slim` runner stage.
- **Cons**:
  - Builds pick up new stable Rust minor versions as they release, which is standard for applications using `Cargo.lock`.

### Option 2: `rust:1.85-slim-bookworm` (Strict Minor Version Pin)

- **Explanation**: Pins the Docker builder image to exact Rust compiler minor version 1.85 on Debian Bookworm.
- **Concrete Example**:
  ```dockerfile
  FROM rust:1.85-slim-bookworm AS builder
  ```
- **Pros**:
  - Fully deterministic compiler version.
- **Cons**:
  - Fails to build when any transitive dependency requires a newer compiler (causing the Raspberry Pi OS build issue).
  - Requires manual Dockerfile edits every time a dependency raises its MSRV.

### Recommendation

**Option 1 (`rust:1-slim-bookworm`)** is recommended and approved by the user. It ensures future-proof builds across developer laptops and Raspberry Pi OS without compromising image size or stability.

---

## Exact Code Changes

### 1. Configuration Gate (`crates/server/src/config.rs`)

Make `ai_enabled` require both `auth_enabled` and `AI_ENABLED`:

```rust
// In Config::from_map:
let ai_enabled = auth_enabled
    && map
        .get("AI_ENABLED")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);
```

### 2. Route Protection (`crates/server/src/routes/entries.rs`)

Inject `State(config)` and `headers: HeaderMap` into `get_entry_ai_summary_htmx`, rejecting calls when AI is disabled:

```rust
pub async fn get_entry_ai_summary_htmx(
    _user: AuthenticatedUser,
    State(schema): State<AppSchema>,
    State(config): State<Arc<Config>>,
    State(ai_service): State<Arc<AiService>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    if !config.ai_enabled {
        return handle_htmx_error(&headers, AppError::NotFound("AI digest is disabled".to_string()));
    }
    // Existing summary generation logic follows...
}
```

### 3. Dockerfile Builder Update (`Dockerfile`)

Update builder stage:

```dockerfile
# Change line 6:
FROM rust:1-slim-bookworm AS builder
```

---

## Proposed Has (parent applies to card)

- Has UI: `no`
- Has API: `yes`
- Has DB: `no`
- Has auth/PII/schema: `yes`

*No Has flip detected.*

## Has API / DB (proposed)

- Has API: `yes` (modifies HTTP route handler signature and response codes for `/hx/entries/{id}/ai-summary`)
- Has DB: `no` (no schema or migration changes)
