// Shared stub data for AR RSS Reader
const DEFAULT_ARTICLES = [
  {
    id: "item-1",
    feed: "Ars Technica",
    title: "NASA's Voyager 1 Resumes Full Science Data Transmission",
    date: "2 hours ago",
    readTime: "4 min read",
    unread: true,
    signalScore: 94,
    summary: "After months of communication issues caused by a corrupted memory chip in the flight data system, NASA engineers restored Voyager 1 to full operation 15 billion miles away.",
    aiBullets: [
      "Defective memory chip in the Flight Data Subsystem (FDS) was isolated.",
      "Commands transmitted across a 22.5-hour one-way light time.",
      "All four scientific instruments are once again collecting interstellar data."
    ],
    body: `
      <p>NASA reported today that Voyager 1 has resumed full scientific operations for the first time since November. The 46-year-old spacecraft, currently traveling in interstellar space roughly 15 billion miles from Earth, is returning normal telemetry and science readings from all four active instruments.</p>
      <p>The root cause was traced to a single failing memory chip in the Flight Data Subsystem (FDS). Rather than attempt to replace the hardware remotely, engineers at the Jet Propulsion Laboratory divided the affected code into smaller segments and relocated them to unoccupied sections of the FDS memory.</p>
      <p>Voyager 1 and its twin, Voyager 2, remain humanity's longest-operating spacecraft, offering unprecedented measurements of the interstellar medium beyond the heliosphere.</p>
    `
  },
  {
    id: "item-2",
    feed: "Hacker News",
    title: "SQLite 3.46 Released with Improved JSON and Query Planning",
    date: "4 hours ago",
    readTime: "3 min read",
    unread: true,
    signalScore: 88,
    summary: "SQLite 3.46 brings significant improvements to JSONB functions, faster subquery flattening, and enhanced memory efficiency in embedded environments.",
    aiBullets: [
      "Enhanced JSONB representation accelerates binary serialization by 30%.",
      "Optimizer improvements eliminate unnecessary temp table allocations for complex CTEs.",
      "Strict backwards compatibility maintained across all target architectures."
    ],
    body: `
      <p>The SQLite development team has announced version 3.46. This release introduces enhanced support for binary JSON (JSONB), reducing both memory usage and parse overhead when handling high-throughput document payloads.</p>
      <p>Additionally, query planner refinements allow the optimizer to flatten nested subqueries and Common Table Expressions (CTEs) more aggressively, leading to measurable speedups in analytical queries.</p>
    `
  },
  {
    id: "item-3",
    feed: "GitHub Blog",
    title: "Introducing Workspace-Level Code Intelligence for Monorepos",
    date: "Yesterday",
    readTime: "5 min read",
    unread: true,
    signalScore: 82,
    summary: "GitHub rolled out semantic code navigation across multi-package monorepos, providing instant jump-to-definition and symbol cross-references across repository boundaries.",
    aiBullets: [
      "Language Server Protocol (LSP) indexes dependencies across workspace boundaries.",
      "Supports TypeScript, Go, Rust, and Python monorepo layouts.",
      "Instant symbol search without manual index rebuilds."
    ],
    body: `
      <p>Navigating large-scale monorepos has historically posed challenges for browser-based code viewers. Today, GitHub is rolling out workspace-aware semantic navigation across enterprise repositories.</p>
      <p>Developers can now command-click symbols in one package and jump directly to their declaration in an independent internal package, streamlining code reviews and architectural audits.</p>
    `
  },
  {
    id: "item-4",
    feed: "The Verge",
    title: "EU Finalizes Interoperability Standards for Messaging Protocols",
    date: "Yesterday",
    readTime: "4 min read",
    unread: true,
    signalScore: 78,
    summary: "European regulators established technical guidelines requiring designated messaging platforms to support cross-service end-to-end encrypted messaging.",
    aiBullets: [
      "Gatekeeper messaging apps must support interoperable 1-on-1 text encryption.",
      "Signal protocol or equivalent security guarantees required.",
      "Full compliance deadline set for early next year."
    ],
    body: `
      <p>The European Commission has published the technical architecture guidelines implementing the Digital Markets Act (DMA) interoperability obligations. Gatekeeper messaging platforms must provide open APIs for third-party clients to exchange messages securely.</p>
      <p>The mandate emphasizes that cross-platform communication must preserve end-to-end encryption without compromising cryptographic key safety.</p>
    `
  },
  {
    id: "item-5",
    feed: "MIT Tech Review",
    title: "Next-Generation Solid-State Batteries Enter Pilot Factory Testing",
    date: "2 days ago",
    readTime: "6 min read",
    unread: false,
    signalScore: 91,
    summary: "A new pilot facility in California began manufacturing ceramic electrolyte solid-state battery cells designed to deliver 500 Wh/kg energy density.",
    aiBullets: [
      "Ceramic electrolyte eliminates flammable liquid solvent risk.",
      "Energy density increases by 65% compared to commercial NMC lithium cells.",
      "Targeting initial rollout in consumer hardware followed by automotive platforms."
    ],
    body: `
      <p>Researchers and automotive partners have inaugurated a pilot manufacturing line producing ceramic solid-state battery cells at commercial scale. The design demonstrates resilience across 1,000 deep charge cycles with minimal capacity degradation.</p>
      <p>By replacing organic liquid electrolytes with a non-flammable solid ceramic separator, the cells operate safely at higher voltages while drastically reducing thermal runaway risks.</p>
    `
  }
];

function getStoredArticles() {
  const stored = localStorage.getItem("ar_rss_articles");
  if (!stored) {
    localStorage.setItem("ar_rss_articles", JSON.stringify(DEFAULT_ARTICLES));
    return DEFAULT_ARTICLES;
  }
  try {
    return JSON.parse(stored);
  } catch (e) {
    return DEFAULT_ARTICLES;
  }
}

function saveArticles(articles) {
  localStorage.setItem("ar_rss_articles", JSON.stringify(articles));
}

function getArticleById(id) {
  const articles = getStoredArticles();
  return articles.find(a => a.id === id) || articles[0];
}

function markArticleRead(id) {
  const articles = getStoredArticles();
  const item = articles.find(a => a.id === id);
  if (item) {
    item.unread = false;
    saveArticles(articles);
  }
}
