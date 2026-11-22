# my-ui

`my-ui` is a modern, fast web frontend for the [Miniflux](https://miniflux.app/) RSS reader. It provides a clean, responsive reader interface with optional OIDC authentication (Pocket-ID) and local AI article summarization.

---

## Prerequisites

Before running `my-ui`, make sure you have the following installed on your host machine:

- **Docker Engine**: version 24.0 or newer.
- **Docker Compose**: version 2.20 or newer (or `docker-compose` v2/v5 standalone).
- **Rust Toolchain** *(optional)*: Rust 1.85+ if compiling natively outside Docker.

---

## Configuration

The application reads configuration from environment variables. A template file is provided at `.env.example`.

### 1. Create your environment file

Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

### 2. Configure environment variables

Open `.env` and fill in your values. Key settings include:

- **Server settings**:
  - `PORT`: HTTP port to listen on inside container (default: `3000`).
  - `HOST`: Bind address (default: `0.0.0.0`).
  - `APP_BASE_URL`: Public base URL of the application (e.g. `http://localhost:3000`).
  - `SESSION_SECRET`: Random 32+ character key for session cookie encryption.
- **Authentication (Pocket-ID OIDC)**:
  - `AUTH_ENABLED`: Set to `true` to require login, or `false` for public/single-user mode.
  - `OIDC_ISSUER_URL`: Issuer URL of your Pocket-ID instance.
  - `OIDC_CLIENT_ID`: Registered OIDC client ID.
  - `OIDC_CLIENT_SECRET`: Registered OIDC client secret.
  - `OIDC_REDIRECT_URL`: OAuth callback URL (e.g. `http://localhost:3000/auth/callback`).
- **Miniflux upstream**:
  - `MINIFLUX_BASE_URL`: Base URL of your Miniflux server (e.g. `https://miniflux.example.com`).
  - `MINIFLUX_API_KEY`: API key generated in Miniflux settings.
- **AI digest**:
  - `AI_ENABLED`: Set to `true` to enable article summarization, `false` to disable.
  - `LLM_CLI_CMD`: Local LLM CLI binary (default: `llm`).
  - `LLM_MODEL`: Model name (default: `gpt-4o-mini`).
  - `AI_DB_PATH`: Path to SQLite database storing summaries (default in container: `/app/data/summaries.db`).

### 3. Generate a secure `SESSION_SECRET`

When `AUTH_ENABLED=true`, `SESSION_SECRET` is required to encrypt session cookies using AES-256-GCM. It must be **at least 32 characters long**.

Generate a random secret using any of the following commands:

```bash
# Using OpenSSL (Recommended):
openssl rand -hex 32

# Or using Python:
python3 -c "import secrets; print(secrets.token_hex(32))"

# Or using /dev/urandom:
head -c 32 /dev/urandom | base64
```

Copy the output and paste it into your `.env`:

```bash
SESSION_SECRET=your_generated_random_string_here
```

*(Note: If `AUTH_ENABLED=false`, a built-in default secret is automatically used for guest mode).*

---

## AI Article Summarization Setup

`my-ui` includes an AI-powered news digest feature that summarizes articles in 1–2 concise sentences (<35 words).

### How it works

- **On-demand summarization**: Summaries are requested as you scroll through articles (`hx-trigger="revealed"`), keeping page loads fast.
- **Persistent caching**: Summaries are saved in SQLite (`data/summaries.db`). Articles are never summarized more than once.
- **Graceful fallback**: If the LLM CLI is unavailable or times out (5s), `my-ui` automatically falls back to an extractive article summary without breaking the UI.
- **CLI-based architecture**: Summarization runs through an external command-line tool (by default [Simon Willison's `llm`](https://llm.datasette.io/)).

### 1. Install the `llm` CLI tool

Install `llm` on your host machine or development environment:

```bash
# macOS or Linux (Homebrew)
brew install llm

# Or via Python pip / pipx
pip install llm
# or
pipx install llm
```

Verify installation:
```bash
llm --version
```

### 2. Configure your LLM model & API keys

The `llm` CLI supports OpenAI, Anthropic, Gemini, local Ollama models, and more:

- **OpenAI (Default: `gpt-4o-mini`)**:
  ```bash
  llm keys set openai
  # Paste your OpenAI API key when prompted
  ```

- **Google Gemini**:
  ```bash
  llm install llm-gemini
  llm keys set gemini
  ```

- **Anthropic Claude**:
  ```bash
  llm install llm-claude-3
  llm keys set claude
  ```

- **Local Ollama (Offline / Free)**:
  ```bash
  llm install llm-ollama
  # Set LLM_MODEL to your local Ollama model (e.g. llama3.2, mistral)
  ```

Quick test to verify:
```bash
llm -m gpt-4o-mini "Say hello in one word"
```

### 3. Configure `.env` for AI

Add or adjust these variables in your `.env` file:

```bash
# Enable AI digest features
AI_ENABLED=true

# Command executed by my-ui
LLM_CLI_CMD=llm

# Target model
LLM_MODEL=gpt-4o-mini

# Extra CLI arguments (optional)
LLM_EXTRA_ARGS="--thinking-budget 0"

# Path to persistent summary database
AI_DB_PATH=data/summaries.db
```

### 4. Using AI with Docker vs Native

- **Native development (`cargo run`)**:
  When running `cargo run --bin server` on your host machine, `my-ui` invokes `llm` directly from your host PATH.
- **Docker container**:
  If `llm` is not present inside the minimal container image, `my-ui` cleanly falls back to extractive article summaries without errors. To use live LLM calls inside Docker, mount your host `llm` configuration directory into the container:
  ```yaml
  # In docker-compose.local.yml under services.my-ui.volumes:
  volumes:
    - ./data:/app/data
    - ~/.config/io.datasette.llm:/home/appuser/.config/io.datasette.llm:ro
  ```

---

## Local Development

For local development, use `docker-compose.local.yml`. This configuration mounts the host `./data` directory directly into the container so you can inspect SQLite databases immediately on your host machine.

### Start the local container

```bash
docker compose -f docker-compose.local.yml up -d --build
```

*(Note: On systems using standalone compose, replace `docker compose` with `docker-compose`)*

### View runtime logs

```bash
docker compose -f docker-compose.local.yml logs -f
```

### Stop the local container

```bash
docker compose -f docker-compose.local.yml down
```

---

## Production Deployment

For production deployments, use `docker-compose.yml`. This configuration includes hardened security options (`no-new-privileges:true`), bounded json-file logging (max 10MB across 3 files), an `always` restart policy, and an isolated Docker named volume.

### Start production services

```bash
docker compose up -d --build
```

### Check service status & health

```bash
docker compose ps
```

### View production logs

```bash
docker compose logs -f
```

### Update to new code

```bash
docker compose down
docker compose up -d --build
```

### Stop production services

```bash
docker compose down
```

---

## Data Persistence & Storage

The SQLite database (`summaries.db`) stores cached AI summaries and user preferences.

- **Local environment**: Data persists on your host machine in `./data/summaries.db`.
- **Production environment**: Data persists inside the Docker named volume `my_ui_data`.
  - Inspect volume location on host:
    ```bash
    docker volume inspect my-ui_my_ui_data
    ```
  - Backup the database volume:
    ```bash
    docker run --rm -v my-ui_my_ui_data:/data -v $(pwd):/backup alpine tar czf /backup/my_ui_data_backup.tar.gz -C /data .
    ```

---

## Troubleshooting

- **Container health check failing**:
  - Inspect container health status directly:
    ```bash
    docker inspect --format='{{json .State.Health}}' my-ui
    ```
  - Verify container can reach `/health`:
    ```bash
    docker exec -it my-ui curl -I http://127.0.0.1:3000/health
    ```
- **Permission errors accessing database (`/app/data`)**:
  - Container runs as non-root user `appuser` with UID `10001`.
  - When using bind mount in local dev, ensure the `./data` directory on host has appropriate permissions:
    ```bash
    chmod -R 777 ./data   # Or chown -R 10001:10001 ./data
    ```
- **Upstream connectivity issues**:
  - Check that the container can reach your Miniflux and Pocket-ID instances:
    ```bash
    docker exec -it my-ui curl -I https://miniflux.example.com
    ```
  - Ensure HTTPS certificates are valid; the runner image includes updated Debian `ca-certificates`.
