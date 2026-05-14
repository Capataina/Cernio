# scripts/

Reusable Python helpers for the parallel-agent grading / re-grade workflow.

These are **separate from the `cernio` Rust binary** — they don't replace any
subcommand in `src/pipeline/`. They support the conversational workflow where
Claude spawns N parallel re-grade or grade-jobs agents, each emits SQL into a
JSONL transcript, and the aggregator collects + dedupes + applies in a single
transaction.

The Rust subcommands (`cernio resolve`, `cernio search`, `cernio check`,
`cernio clean`, `cernio format`, `cernio import`, `cernio unarchive`) handle
the volume work directly against the DB and live in `src/pipeline/`. These
Python scripts handle the **judgment-aggregation work** that happens when
Claude itself is the grader and multiple Claude instances are running in
parallel.

## The helpers

### `split_grading_batches.py`
Take every job currently in `evaluation_status IN ('pending','evaluating')`
and stripe it across N batches (default 8), front-loading each batch with the
highest-promise titles (grad/intern/rust/compiler/trading keyword score).
Output: `/tmp/grading_batch_NN.md` per stripe, ready to hand to a parallel
grade-jobs agent.

```bash
python3 scripts/split_grading_batches.py           # 8 stripes
python3 scripts/split_grading_batches.py --n 5     # fewer, larger stripes
python3 scripts/split_grading_batches.py --out /path/to/dir
```

### `split_regrade_batches.py`
Same shape but for **re-grade**: take every active+graded company (in
`status IN ('resolved','bespoke')`) and every active+graded job, stripe each
into N batches (default 10) keeping a similar tier mix per stripe. Output:
`/tmp/regrade_{company,job}_batch_NN.md`.

```bash
python3 scripts/split_regrade_batches.py
python3 scripts/split_regrade_batches.py --skip-companies   # only jobs
python3 scripts/split_regrade_batches.py --n 5 --out /tmp/run2/
```

### `extract_agent_sql.py`
After parallel agents finish, walk the Claude Code task directory (auto-
detected from `/private/tmp/claude-*/<project>/<session>/tasks/`), extract
every `UPDATE companies SET ...` and `UPDATE jobs SET ...` statement from
each agent's JSONL transcript (handles `''` SQL string escapes properly,
matches both inline-text and `Write`-tool emit paths), dedupe by (table, id)
with last-write-wins (handles agent self-corrections), sanity-check against
the live DB, and emit a single `BEGIN/COMMIT`-wrapped SQL file.

```bash
python3 scripts/extract_agent_sql.py
    # → /tmp/all_agent_sql.sql
python3 scripts/extract_agent_sql.py --task-dir /private/tmp/claude-XXX/.../tasks
python3 scripts/extract_agent_sql.py --no-companies
    # only extract job UPDATEs
python3 scripts/extract_agent_sql.py --patch-job 2972:B:0.45
    # manual override: id=2972 from whatever-the-agent-said to grade='B', fit_score=0.45
```

Apply the output with:

```bash
sqlite3 state/cernio.db ".read /tmp/all_agent_sql.sql"
```

### `find_filter_candidates.py`
Diagnostic tool — find unigrams/bigrams/trigrams in C/F job titles that
have **zero** B+ hits and aren't already in `preferences.toml` exclude_keywords.
Useful for spotting universe shifts after every re-grade, even if you don't
end up adding keywords. (Per the 2026-05-14 decision: keyword additions are
low-ROI for this universe — diagnostic value only.)

```bash
python3 scripts/find_filter_candidates.py            # ≥5 C/F hits, top 80
python3 scripts/find_filter_candidates.py --min 3    # widen the net
python3 scripts/find_filter_candidates.py --toml     # paste-ready output form
```

## End-to-end re-grade workflow

A typical parallel re-grade looks like:

1. `python3 scripts/split_regrade_batches.py` — generate 10 company + 10 job
   batch files in `/tmp/`.
2. Hand each batch to a parallel re-grade agent (the `regrade_{company,job}_payload.md`
   files in `/tmp/` are also expected — those are the rubric + profile + anchors
   bundle, hand-written per run).
3. Wait for all 20 agents to complete.
4. `python3 scripts/extract_agent_sql.py` — auto-detects the task dir, walks
   all transcripts, dedupes, sanity-checks, emits `/tmp/all_agent_sql.sql`.
5. Inspect the SQL file. Apply manual `--patch-job` overrides if any agent
   self-flagged a correction.
6. `sqlite3 state/cernio.db ".read /tmp/all_agent_sql.sql"`
7. `sqlite3 state/cernio.db "VACUUM; PRAGMA wal_checkpoint(TRUNCATE);"`
8. `cernio check` — confirm post-apply integrity (`✓ All company grades are
   fresh (<30 days)` is the canonical success signal).
9. `python3 scripts/find_filter_candidates.py` — diagnostic-only sweep to see
   if any new filter patterns surfaced.
10. Commit `state/cernio.db` + the `portfolio-gaps.md` batch entry.

The 2026-05-14 comprehensive re-grade (657 companies + 506 graded jobs across
20 parallel agents) used exactly this workflow; see the corresponding entry
in `profile/portfolio-gaps.md` for the full pattern record.
