# Skim — workspace scan

**Result:** done

## State

**Greenfield project.** No Rust code, no `Cargo.toml`, no build config yet.

## Existing files

| File | Purpose |
|------|---------|
| `ar-list.html` (267 lines) | Reference UI — feed list view with spatial glassmorphism |
| `ar-detail.html` (249 lines) | Reference UI — article reader with AI summary card |
| `ar-data.js` (130 lines) | Stub data (5 mock articles) + localStorage helpers |
| `index.html` | Placeholder / hub page |
| `option1-workspace.html` | Earlier design exploration (not used) |
| `option2-graph.html` | Earlier design exploration (not used) |
| `minimal-reader.html` | Earlier design exploration (not used) |
| `miniflux-openapi.yaml` (2081 lines) | Complete Miniflux REST API v2.2.0 spec |

## Design system extracted from reference files

- **Dark glassmorphism** — `rgba(15,23,42,0.72)` base, `blur(32px) saturate(190%)`
- **720px spatial window** — `max-width: 720px`, `height: min(88vh, 780px)`, `border-radius: 28px`
- **Passthrough BG** — dual radial gradients (indigo + purple) on slate-950 base
- **List items** — `rgba(255,255,255,0.04)` base, indigo hover glow, `translateY(-1px) scale(1.008)` lift
- **Typography** — Tailwind defaults, `text-sm`, `prose-invert` for article body
- **Icons** — Lucide icon set
- **Color palette** — indigo-400/500 accents, slate-100..500 text, white/5..15 borders

## OpenAPI spec highlights

- **50+ endpoints** across Users, Categories, Feeds, Entries, Enclosures, Icons, API Keys, OPML, Integrations
- **Auth** — `X-Auth-Token` header (API key) or HTTP Basic
- **Key schemas** — Entry (title, content, status, starred, reading_time, tags, feed, enclosures), Feed, Category, FeedCounters
- **Pagination** — offset/limit/order/direction params, `EntryResultSet` wrapper with total
- **Search** — full-text search param on entry list endpoints
- **Status enum** — `read | unread | removed`
