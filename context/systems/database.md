# Database

> The SQLite data layer backing all structured state in Cernio.

---

## Scope / Purpose

Provides the single persistent data store for all structured state in Cernio — company universe, job listings, and user decisions. Backed by SQLite via `rusqlite` with WAL mode for concurrent access from multiple callers (TUI, Rust scripts, Claude).

---

## Boundaries / Ownership

- **File:** `state/cernio.db` (gitignored — the `.db` file is not tracked, the schema in code is)
- **Library:** `rusqlite` 0.35 with `bundled` feature (compiles SQLite from source, no system dependency)
- **Journal mode:** WAL — enables concurrent reads from the TUI while scripts or Claude write
- **Foreign keys:** Enforced (`PRAGMA foreign_keys = ON`)
- **Source:** `src/db/schema.rs` — schema definition, migration, and tests. `src/db/mod.rs` — public interface

The database layer owns schema definition, migration, and the public query/insert interface. It does not own business logic for discovery, evaluation, or presentation — those belong to their respective subsystems.

---

## Current Implemented Reality

### Schema

#### `companies`

The company universe. Each row represents one company at any lifecycle stage.

| Column | Type | Constraints | Category | Notes |
|--------|------|-------------|----------|-------|
| `id` | INTEGER | PRIMARY KEY | — | Auto-increment |
| `name` | TEXT | NOT NULL | Fact | Company display name |
| `website` | TEXT | NOT NULL, UNIQUE | Fact | Primary dedup key |
| `what_they_do` | TEXT | NOT NULL | Fact | 1-2 sentence description |
| `discovery_source` | TEXT | NOT NULL | Fact | Where the company was found |
| `discovered_at` | TEXT | NOT NULL | Fact | ISO date |
| `status` | TEXT | NOT NULL, DEFAULT 'potential' | Checkpoint | `potential`, `resolved`, or `bespoke` |
| `location` | TEXT | — | Checkpoint | HQ city/country |
| `sector_tags` | TEXT | — | Checkpoint | Comma-separated sector labels |
| `ats_provider` | TEXT | CHECK constraint | Checkpoint | `greenhouse`, `ashby`, `lever`, `workable`, `smartrecruiters`, `workday`, `eightfold`, or NULL |
| `ats_slug` | TEXT | — | Checkpoint | The slug/identifier for API queries |
| `ats_extra` | TEXT | — | Checkpoint | Provider-specific data (Workday subdomain+site, Eightfold subdomain+domain) |
| `ats_verified_at` | TEXT | — | Checkpoint | Last date the slug was confirmed working |
| `careers_url` | TEXT | — | Checkpoint | Direct careers page URL (primarily for bespoke) |
| `why_relevant` | TEXT | NOT NULL | Judgment | Connection to user's profile |
| `relevance_updated_at` | TEXT | NOT NULL | Judgment | When relevance was last assessed |

**Lifecycle:** `potential` (from discovery, unresearched) → `resolved` (ATS found and verified) or `bespoke` (no supported ATS, careers URL preserved).

#### `jobs`

Individual job listings linked to companies.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| `id` | INTEGER | PRIMARY KEY | Auto-increment |
| `company_id` | INTEGER | NOT NULL, FK → companies | |
| `title` | TEXT | NOT NULL | |
| `url` | TEXT | NOT NULL, UNIQUE | Dedup key |
| `location` | TEXT | — | |
| `remote_policy` | TEXT | — | |
| `posted_date` | TEXT | — | |
| `raw_description` | TEXT | — | Full HTML/text from ATS |
| `parsed_tags` | TEXT | — | Structured extraction (tech stack, seniority, etc.) |
| `evaluation_status` | TEXT | NOT NULL, DEFAULT 'pending' | `pending`, `evaluating`, `strong_fit`, `weak_fit`, `no_fit` |
| `fit_assessment` | TEXT | — | Claude's reasoning for the evaluation |
| `fit_score` | REAL | — | Numeric fit score |
| `discovered_at` | TEXT | NOT NULL | |

#### `user_decisions`

User actions on evaluated jobs.

| Column | Type | Constraints | Notes |
|--------|------|-------------|-------|
| `id` | INTEGER | PRIMARY KEY | Auto-increment |
| `job_id` | INTEGER | NOT NULL, FK → jobs | |
| `decision` | TEXT | NOT NULL, CHECK | `watching`, `applied`, `rejected` |
| `decided_at` | TEXT | NOT NULL | |
| `notes` | TEXT | — | User's notes on the decision |

#### Indexes

| Index | On | Purpose |
|-------|-----|---------|
| `idx_companies_status` | companies(status) | Filter by lifecycle stage |
| `idx_companies_ats_provider` | companies(ats_provider) | Filter by ATS type |
| `idx_jobs_company_id` | jobs(company_id) | Lookup jobs by company |
| `idx_jobs_evaluation_status` | jobs(evaluation_status) | Filter by evaluation state |
| `idx_user_decisions_job_id` | user_decisions(job_id) | Lookup decisions by job |

### Migration Strategy

The schema is defined as a single `CREATE TABLE IF NOT EXISTS` batch in `MIGRATION_001`. It runs on every startup and is idempotent. Future schema changes will be added as `MIGRATION_002`, etc., each gated by a version check.

The database can always be recreated from scratch by deleting `state/cernio.db` and restarting — the migration rebuilds everything. The data is lost, but the schema is code.

### Tests

7 tests in `src/db/schema.rs`:

| Test | Verifies |
|------|----------|
| `schema_creates_successfully` | All tables exist after migration |
| `migrate_is_idempotent` | Running migration twice doesn't error |
| `insert_and_query_company` | Basic insert and select |
| `company_status_constraint` | Invalid status values rejected |
| `website_uniqueness` | Duplicate websites rejected |
| `job_links_to_company` | Job insert with valid company FK |
| `foreign_key_enforced` | Job insert with invalid company FK rejected |

---

## Key Interfaces / Data Flow

### Access Patterns

| Caller | Access | Typical queries |
|--------|--------|-----------------|
| **Claude (conversation)** | Read + Write | Insert companies from discovery, write evaluations, update relevance |
| **Rust scripts** | Write | Insert job search results, update ATS verification |
| **TUI** | Read + Write | Query companies/jobs with filters and sorts, write user decisions |

WAL mode ensures the TUI can read concurrently while other processes write.

### Public Interface

Defined in `src/db/mod.rs`. Schema definition and migration live in `src/db/schema.rs`.

---

## Implemented Outputs / Artifacts

- `state/cernio.db` — the runtime SQLite database file (gitignored, recreatable from migrations)
- `src/db/schema.rs` — schema SQL and migration logic
- `src/db/mod.rs` — public Rust interface for database access

---

## Known Issues / Active Risks

None at this stage.

---

## Partial / In Progress

None at this stage.

---

## Planned / Missing / Likely Changes

- Future migrations (`MIGRATION_002`, etc.) will be needed as the schema evolves.
- Higher-level query functions in `src/db/mod.rs` will grow as subsystems (discovery, evaluation, TUI) are implemented.

---

## Durable Notes / Discarded Approaches

- Field categories (Fact / Checkpoint / Judgment) were chosen to clarify which columns are immutable discovery data, which track pipeline progress, and which represent assessed quality — this distinction guides which fields can be safely overwritten on re-evaluation.
- `website` was chosen as the primary dedup key for companies over `name` because company names are ambiguous and often duplicated.

---

## Obsolete / No Longer Relevant

None at this stage.
