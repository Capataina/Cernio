# Database Maintenance

Automated and user-triggered maintenance operations to keep the database lean and relevant.

---

## Clean DB operation

A single command (TUI button or CLI) that sweeps the database for stale, irrelevant, or dead entries.

### Tiered archival lifecycle (session 5)

Jobs are archived based on their grade — higher-value jobs stay active longer:

| Grade | Active duration | Then | Archive expiry |
|-------|----------------|------|----------------|
| SS | 28 days | → archived | Deleted after 14 days in archive |
| S | 21 days | → archived | Deleted after 14 days in archive |
| A | 14 days | → archived | Deleted after 14 days in archive |
| B | 7 days | → archived | Deleted after 14 days in archive |
| C/F | 3 days | → archived | Deleted after 14 days in archive |

Archive expiry is tracked via the `archived_at` column (migration 005). When an archived job is deleted, it can be re-discovered and re-graded on the next search — giving it a "second chance" with a potentially updated profile.

**Unarchive command:** `cernio unarchive --jobs --grade A` restores all A-graded archived jobs, preserving their grade and assessment but resetting `discovered_at` to now so the tiered timer restarts.

**Companies:**

Cleanup does NOT auto-archive companies by grade. C-tier companies stay active because job grading handles quality filtering. Companies are only archived manually when there's a hard reason (excluded sector, dissolved, duplicates).

### What it preserves

- Any job with a user decision (`watching`, `applied`, `rejected`) — these represent user intent
- Archived companies — they stay in the DB for dedup, just hidden from TUI and searches
- All companies regardless of grade — company cleanup is manual only

### Surfacing

**TUI:** A keybinding (e.g., `D` from the dashboard or a confirmation prompt) that shows what would be removed before executing. Something like:

```
┌─ Clean Database ───────────────────────────────────┐
│                                                     │
│  This will remove:                                  │
│                                                     │
│    12  F-graded jobs                                │
│     5  C-graded jobs                                │
│     3  jobs older than 14 days                      │
│                                                     │
│  Total: 20 jobs removed                             │
│  Preserved: 5 (user decisions or SS/S grade)        │
│                                                     │
│  Press Enter to confirm, Esc to cancel              │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**CLI:** `cernio clean` with `--dry-run` flag.

### Implementation notes

- Use a transaction so the cleanup is atomic
- Log what was removed (count by grade, count by staleness) to stdout or a session summary
- After cleanup, the TUI auto-refreshes and the dashboard stats update immediately
- This operation is safe to run repeatedly — it's idempotent

### Why this matters

Without cleanup, the database accumulates noise over time. 200 companies × 20 jobs each = 4,000 jobs, of which maybe 500 are worth evaluating. After evaluation, the F and C grades are dead weight. Cleanup keeps the signal-to-noise ratio high and makes browsing the TUI pleasant rather than overwhelming.

This also reduces the cognitive load on Claude during evaluation sessions — fewer pending jobs in the queue means more focused evaluation.

---

## Implemented maintenance operations

### ATS verification sweep

Implemented in `cernio check`. Re-probes all resolved companies' ATS slugs to detect provider migrations, dead boards, or changed slugs. Companies that fail verification are flagged in the structured report.

**Trigger:** `cernio check` or `cernio check --ats-only`. Can also be invoked conversationally via the `check-integrity` skill.

### Stale company detection

Implemented in `cernio check`. Flags companies with stale grades, missing data, dead bespoke URLs, and other integrity issues. The `check-integrity` AI skill adds judgment-based assessment on top — detecting profile-driven staleness that timestamps alone cannot catch.

### Format command (`cernio format`)

Implemented in `src/pipeline/format.rs`. Converts raw HTML/entity-encoded job descriptions to clean plaintext directly in the database. Session 6 formatted 922 descriptions.

**Key properties:**
- **Idempotent** — safe to run repeatedly, only processes descriptions that still contain HTML/entities
- **Runs on TUI startup** — called via `run_silent()` so descriptions are always clean when browsing
- **Step 2 in check-integrity** — ensures descriptions are formatted before judgment-based evaluation
- **In-place** — modifies `raw_description` column directly, no separate "formatted" column

### Application package cleanup

The `application_packages` table (migration 006) stores pre-generated JSON answers for job applications. Packages are auto-deleted when a job is marked "applied" through any path (`o` key, `p` key, `a` key). This prevents stale packages accumulating for jobs already applied to.

### Re-evaluation trigger

Implemented in the `check-integrity` skill. Compares profile modification dates against `graded_at` timestamps. When the profile has changed in ways relevant to a company's grade reasoning (e.g., new project fills a gap cited as a weakness), the skill flags the entry for regrading. The skill can also execute remediation when the user approves — regrading, rewriting shallow assessments, and fetching missing data.
