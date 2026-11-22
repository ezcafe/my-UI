# syntax=docker/dockerfile:1

# ---------------------------------------------------------
# Stage 1: Build release binary
# ---------------------------------------------------------
FROM rust:1-slim-bookworm AS builder

# Install build dependencies for C-bundled rusqlite
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    g++ \
    libc6-dev \
    pkg-config \
    make \
    binutils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy workspace configuration and sources
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary for server and strip symbols
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    cargo build --release --bin server && \
    cp /build/target/release/server /usr/local/bin/server && \
    strip /usr/local/bin/server

# ---------------------------------------------------------
# Stage 2: Minimal runtime image
# ---------------------------------------------------------
FROM debian:bookworm-slim AS runner

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged user (UID/GID 10001)
RUN groupadd -g 10001 appuser && \
    useradd -u 10001 -g appuser -s /usr/sbin/nologin -M appuser

# Pre-create data directory with correct ownership
WORKDIR /app
RUN mkdir -p /app/data && \
    chown -R appuser:appuser /app/data && \
    chmod 750 /app/data

# Copy server binary from builder
COPY --from=builder --chown=appuser:appuser /usr/local/bin/server /usr/local/bin/server
RUN chmod 755 /usr/local/bin/server

# Non-sensitive default environment variables
ENV HOST=0.0.0.0 \
    PORT=3000 \
    AI_DB_PATH=/app/data/summaries.db

# Drop privileges to non-root user
USER 10001:10001

# Volume declaration for database persistence
VOLUME ["/app/data"]

# Expose server port
EXPOSE 3000

# Container healthcheck probe against /health
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f "http://127.0.0.1:${PORT:-3000}/health" || exit 1

# Entrypoint execution
ENTRYPOINT ["/usr/local/bin/server"]
