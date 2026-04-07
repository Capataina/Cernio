# Data Layer Decisions

SQLite is the single source of truth for all structured data in Cernio.

---

## Why SQLite

Evaluated against markdown files, JSON/JSONL, and Postgres/MySQL in Docker.

| Criterion | SQLite |
|-----------|--------|
| Infrastructure | Single file, zero ops — no Docker, no server, no connection string |
| Scale | 740 companies is trivial; handles millions |
| Querying | Full SQL with indexing — "all fintech companies sorted by date" is instant |
| Concurrency | WAL mode enables TUI reads while scripts write |
| Rust support | Mature via `rusqlite` (sync) and `sqlx` (async) |
| Backup | Copy the file. That's it. |
| Local-first | As local as it gets — aligns with every design principle |

Postgres/MySQL would require Docker, be massively overengineered for our scale, and violate the local-first principle.

---

## Schema field categories

Database fields are categorised by how they go stale, which drives the verification strategy:

- **Facts** (name, website, what_they_do, discovery_source, discovered_at) — stable, rarely change. Only stale if the company pivots or rebrands.
- **Checkpoints** (ats_provider, ats_slug, ats_extra, ats_verified_at, careers_url, status, location, sector_tags) — need periodic verification. A Rust script can ping slugs and flag stale entries.
- **Judgments** (why_relevant, relevance_updated_at) — tied to the user's profile, not the company. Go stale when the profile changes, not when the company changes.

Continuously changing metrics (headcount, funding, Glassdoor ratings) are deliberately excluded. Look them up live during evaluation, don't cache stale guesses.

---

## What lives where

| Store | Contains | Why |
|-------|----------|-----|
| SQLite | Companies (full lifecycle), jobs, evaluations, user decisions | Queryable, indexable, fast for TUI |
| `profile/` (markdown) | User's structured profile | Human-edited, read by Claude at startup |
| `companies/potential.md` | Discovery landing zone | Claude writes markdown naturally; entries migrate to SQLite during population |
| `exports/` | Markdown reports | Generated on demand from TUI, not a data store |

---

## Database grows organically

Companies are added in small batches as discovery runs across sessions. A typical flow: discovery finds 30 companies → user reviews → 25 approved → inserted into SQLite. No bulk loading of hundreds at once.

---

## Safety

- Schema is tracked in code (`src/db/schema.rs`) — the database can always be recreated from scratch
- The `.db` file is gitignored — no risk of accidentally committing a large binary
- WAL mode journal files (`.db-wal`, `.db-shm`) are also gitignored
- Backup before risky operations: `cp state/cernio.db state/cernio.db.bak`
