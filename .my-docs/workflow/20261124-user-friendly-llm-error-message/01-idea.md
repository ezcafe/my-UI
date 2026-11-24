# Framing & Shared Understanding

## Goals
1. When an upstream LLM operation (such as generating an inverted-pyramid rewrite or extracting impact and fallacy analysis) fails due to high load, timeout, or provider error, display a clean, non-blocking toast notification.
2. Ensure the toast displays the exact user-friendly warning without technical jargon:
   > "Error: This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later."
3. Keep the user reading experience seamless: the article detail view retains its original content or summary rather than showing broken empty containers or raw error dumps.

## Non-goals
1. Implementing client-side retry countdowns or automatic model re-routing.
2. Altering error behavior for non-AI workflows (e.g. session expiration, Miniflux sync errors).

## Anti-goals
1. Exposing backend technical errors (HTTP 503, stderr, stack traces, JSON parse errors) to the end user.
2. Blocking or freezing the reading interface when an LLM call fails.

## Constraints
1. Leverage the existing HTMX OOB `ToastTemplate` mechanism rendered into `#toast-container`.
2. Ensure consistent appearance in light and dark themes using modern CSS without hardcoded pixel breakpoints.
