# Scaling Architecture

How the system needs to evolve to handle hundreds of companies and thousands of jobs without hitting session limits or context constraints.

---

## The problem

Evaluating 25 Palantir jobs used significant context. At 200 companies × 20 relevant jobs each = 4,000 jobs, conversational evaluation can't scale. Even across sessions, the volume is too high for Claude to read every description.

---

## What moves to scripts

The Rust CLI needs to become a proper pipeline tool, not just individual fetch commands. Scripts handle everything mechanical:

| Task | Why a script |
|------|-------------|
| Fetch all jobs at a company | HTTP calls, no judgment |
| Fetch full job descriptions | HTTP calls, store in DB |
| Location filtering | Binary check — London/UK/Remote or not |
| Obvious department filtering | "Legal", "Finance", "Recruiting", "Sales" don't need AI |
| Insert everything into DB | SQL inserts |
| Track what's new vs already seen | URL dedup against existing DB entries |

**Critical rule: the script should be aggressive about including, not about excluding.** A loose location filter and a minimal department filter. If in doubt, keep it. False negatives (missing good jobs) hurt far more than false positives (evaluating extra irrelevant ones). Speed and coverage are more important than precision at this stage.

---

## What Claude does

Claude only touches jobs that survive the script's mechanical filtering — and works from the database, not from raw API responses:

| Task | Why Claude |
|------|-----------|
| Grading the survivors | Reading descriptions and assessing fit needs judgment |
| Deciding what to search for | Strategy is conversational |
| Discussing results with the user | Core value of the collaborative model |
| Updating portfolio gaps | Pattern recognition across many evaluations |

Claude can work in batches across sessions since everything is persisted in SQLite. "Evaluate the next 30 pending jobs" is a valid session task.

---

## Target CLI commands

```bash
cernio search --company palantir     # fetch all jobs, filter, store in DB
cernio search --all                  # run for every resolved company
cernio search --grade S              # only S-tier companies
cernio pending                       # show jobs awaiting evaluation
cernio pending --count               # how many pending
cernio stats                         # overview of DB state
```

---

## Expected flow at scale

```
Script: 200 companies → fetch all jobs → location filter → department filter
        → fetch details for survivors → store in DB

Result: ~4,000 total jobs fetched
        Location filter → ~800 (London/UK/Remote)
        Department filter → ~500 (anything possibly technical)
        All 500 stored in DB with full descriptions

Claude: reads pending jobs from DB, evaluates in batches
        "Evaluate the next 30 pending Palantir jobs"
        "Show me all SS/S grades from this week"
        Works across multiple sessions — DB persists everything
```

---

## False negatives are the enemy

At every stage of filtering, the bias must be toward inclusion. A job that gets through mechanical filtering and turns out irrelevant costs Claude 30 seconds to grade as F. A job that gets mechanically filtered out and was actually a perfect fit is a lost opportunity that can never be recovered.

The mechanical filters should only remove things that are physically impossible (wrong country, no remote) or categorically irrelevant (legal counsel, payroll analyst). Everything else goes to Claude for judgment.
