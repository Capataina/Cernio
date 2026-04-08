# Job Search Strategy

How job searching should work when we get to that stage.

---

## Job titles are unreliable for filtering

Not all companies use obvious naming conventions like "Junior Engineer" or "Entry Level Backend Developer." Many companies — Spotify is a good example — use titles like "Backend Engineer - Personalization" for positions that are genuinely entry-level but don't say so in the title.

Filtering by title alone would miss these entirely. The correct approach: read the title to decide whether to investigate further, then read the actual job description page to assess fit. The title is a triage signal, not a filter.

**Implication:** Job search cannot be a keyword-only operation. After the initial scan (which scripts handle at volume), Claude must read each promising job's full description to determine seniority, requirements, and fit. This is where the "scripts for volume, Claude for judgment" split matters most.

---

## Discovery skill should read the database first

The discovery skill was designed before the database existed. Now that the database is the source of truth for the company universe, discovery must read from it before searching — not just from `companies/potential.md`. Otherwise it will re-discover companies already in the system.

The deduplication check should be: query all company websites from the database, plus any entries still in `potential.md`, and skip anything already known. Website URL is the stable dedup key (company names vary across sources).

**Implication:** The discover-companies skill needs updating to read SQLite before dispatching agents, and each agent needs the existing website list to deduplicate against.

---

## Companies may use multiple ATS platforms

Some companies post jobs on more than one ATS. ClearBank was found to have both an active Ashby board (25 jobs) and a residual Workable board (0 jobs) from a migration.

The schema now handles this via the `company_portals` table — each company can have multiple portal entries, with `is_primary` flagging the active one. The `cernio resolve` script probes ALL providers per company and records all hits, not just the first match.

**Lesson from production:** Finding a company on two platforms is expected, not an error. The script marks the portal with the most jobs as primary. The search script uses the primary portal for job fetching.
