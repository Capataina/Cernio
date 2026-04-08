# Database Maintenance

Automated and user-triggered maintenance operations to keep the database lean and relevant.

---

## Clean DB operation

A single command (TUI button or CLI) that sweeps the database for stale, irrelevant, or dead entries.

### What it removes / archives

**Jobs (deleted):**

| Target | Condition | Rationale |
|--------|-----------|-----------|
| **F-graded jobs** | `grade = 'F'` | Categorically irrelevant — legal, finance, recruiting, etc. No reason to keep them cluttering the view |
| **C-graded jobs** | `grade = 'C'` | Weak fit, not worth revisiting. If the profile changes significantly, a re-search would surface better-matched versions of these roles anyway |
| **Stale jobs** | `discovered_at` older than 14 days AND no user decision | Jobs older than two weeks are almost certainly closed or filled. If the user marked one as "watching" or "applied", preserve it regardless of age |
| **Stale evaluations** | Jobs whose `posted_date` is older than 14 days (when available) | Some ATS boards keep old listings live. Use `posted_date` when available, fall back to `discovered_at` |

**Companies (soft-archived, not deleted):**

| Target | Condition | Rationale |
|--------|-----------|-----------|
| **C-graded companies** | `grade = 'C'` | Marginal relevance. Set `status = 'archived'` so they don't appear in TUI or get searched for jobs. Preserved for deduplication — discovery won't re-discover them |
| **Unresolvable companies** | `status = 'potential'` with failed resolution attempts | Couldn't find ATS or careers page. Not worth keeping in active view |

All thresholds (which grades to remove, stale age, which company grades to archive) are configurable in `profile/preferences.toml`.

### What it preserves

- Any job with a user decision (`watching`, `applied`, `rejected`) — these represent user intent
- SS and S graded jobs regardless of age — high-value matches are worth keeping visible even if the listing has closed, as a record of what the company looks for
- Archived companies — they stay in the DB for dedup, just hidden from TUI and searches
- All S/A/B-graded companies — company cleanup only targets C-tier and below

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

## Future maintenance operations

### ATS verification sweep

Re-probe all resolved companies' ATS slugs to detect provider migrations, dead boards, or changed slugs. Companies that fail verification get flagged for re-resolution.

**Trigger:** Manual or periodic (monthly). Not urgent — ATS migrations are rare.

### Stale company detection

Flag companies whose last job search returned zero results for 3+ consecutive searches. They may have changed their ATS, stopped hiring, or gone under.

### Re-evaluation trigger

When the user's profile changes significantly (new project added, new skill, shifted preferences), mark all jobs with strong/weak fit for re-evaluation. The original assessment may no longer be accurate.
