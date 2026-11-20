# Grill — design stress-test

**Result:** frontier-empty

## Round 1 — frontier

✅ **Q1** - **HTMX + GraphQL bridge pattern**: Template routes calling GraphQL resolvers internally (no JS bridge, no JSON in the browser). **Accepted.**

---

✅ **Q2** - **Template engine**: Askama (compile-time, type-safe, Jinja2-like syntax). **Accepted.**

---

✅ **Q3** - **Session strategy**: Encrypted/signed cookies for OIDC tokens (no server-side state). **Accepted.**

---

✅ **Q4** - **AI summary card**: Keep card UI with extractive summary (first 2 sentences); defer AI service to v2. **Accepted.**

---

✅ **Q5** - **Crate layout**: Three crates: `server` (Axum server, OIDC, routing, main), `miniflux` (REST client + GraphQL resolvers), `ui-components` (Askama templates, static assets, shared design tokens). **Accepted.**
