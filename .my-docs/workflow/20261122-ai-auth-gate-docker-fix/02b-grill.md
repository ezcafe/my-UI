# Grill log

**Result:** frontier-empty

## Round log

### Round 1

❓ **Q1** - **Matrix: auth=off & ai=on**: Should AI features be disabled if authentication is turned off?
- Setting `AUTH_ENABLED=false` and `AI_ENABLED=true` must resolve `config.ai_enabled = false` to prevent unauthenticated guest visitors from running LLM summaries.
➡️ Yes, disable AI whenever authentication is disabled.

---

❓ **Q2** - **Matrix: auth=on & ai=off**: When auth is enabled but AI is disabled, should AI routes reject requests?
- Authenticated requests to `/hx/entries?status=ai` and `/hx/entries/{id}/ai-summary` must return 404 / NotFound ("AI digest is disabled") without calling the LLM runner or reading the AI database.
➡️ Yes, return NotFound and prevent LLM execution.

---

❓ **Q3** - **Matrix: auth=on & ai=on**: Should AI features require an active authenticated user session?
- Unauthenticated requests must redirect to `/login` (or return 401/HX-Redirect), while authenticated requests properly generate and display AI digests and cards.
➡️ Yes, require active authentication for all AI routes.

---

❓ **Q4** - **Matrix: auth=off & ai=off**: In guest mode with AI disabled, should both auth and AI remain inactive?
- The app operates in pure guest reading mode without login prompts, and all AI endpoints return 404.
➡️ Yes, operate purely in basic guest mode with AI disabled.

---

❓ **Q5** - **Direct Request: /hx/entries/1/ai-summary**: What happens on direct access when AI is disabled?
- Even if invoked directly via curl, script, or forged HTMX headers, the endpoint must check `config.ai_enabled` first and immediately reject with NotFound, bypassing the AI service.
➡️ Yes, reject early before calling `AiService`.

---

❓ **Q6** - **Direct Request: /hx/entries?status=ai**: What happens on direct access when AI is disabled?
- Return NotFound using `handle_htmx_error` consistent with the existing filter route behavior.
➡️ Yes, return NotFound and do not render the digest view.

---

❓ **Q7** - **Docker Multi-Arch Compatibility**: Should the builder base image support `linux/amd64`, `linux/arm64`, and Raspberry Pi OS targets?
- Using `rust:1-slim-bookworm` provides official multi-arch container images, latest 1.x stable compiler support, and exact glibc matching with `debian:bookworm-slim`.
➡️ Yes, use `rust:1-slim-bookworm` for multi-architecture builds.
