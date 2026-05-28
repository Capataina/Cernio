---
title: Portfolio Gaps
last_updated: 2026-05-29
maintained_by: grade-jobs (per cernio-full-refactor.md §5.6 Phase 3)
seeded_from: legacy profile/portfolio-gaps.md (576 lines, append-only batch log)
---

# Portfolio Gaps — Per-Lane Folder

> Replaces the legacy single `profile/portfolio-gaps.md` flat file per the
> lane-based-relativity refactor (cernio-full-refactor.md §5.6 Phase 3).
> Maintained by `grade-jobs` Phase 3 — each per-lane file is rewritten in
> place at the end of every grade-jobs batch (NOT append-only).

## Per-lane files

| File | Lane | Maintenance source |
|---|---|---|
| `big-tech.md` | big-tech | grade-jobs Phase 3, jobs tagged big-tech |
| `ai-ml.md` | ai-ml | grade-jobs Phase 3, jobs tagged ai-ml |
| `hft.md` | hft | grade-jobs Phase 3, jobs tagged hft |
| `crypto-mm.md` | crypto-mm | grade-jobs Phase 3, jobs tagged crypto-mm |
| `bank-strats.md` | bank-strats | grade-jobs Phase 3, jobs tagged bank-strats |
| `systems-infra.md` | systems-infra | grade-jobs Phase 3, jobs tagged systems-infra |
| `devtools.md` | devtools | grade-jobs Phase 3, jobs tagged devtools |
| `fintech.md` | fintech | grade-jobs Phase 3, jobs tagged fintech |
| `closed.md` | (lane-agnostic) | dated archive of closed gaps; lane-tagged entries |

## File shape (canonical)

Each per-lane file has a fixed shape, rewritten in place per batch:

1. **Open gaps** — technologies / domains the lane wants that Caner doesn't have
2. **Confirmed strengths** — what Caner has that the lane explicitly values
3. **Closure prescriptions** — concrete portfolio investments per gap
4. **Last-updated timestamp** + grade-jobs batch identifier

## Initial seed (2026-05-29)

These files were seeded by extracting patterns from the legacy
`profile/portfolio-gaps.md` (576 lines of append-only batch log from
2026-04-09 through 2026-05-17) and re-organising them per lane. The next
canonical `grade-jobs` Phase 3 run will overwrite each file with fresh
agent-derived content. The seed serves as a useful starting state; treat
the content as best-effort until the next grade-jobs run.

## Legacy file disposition

The legacy `profile/portfolio-gaps.md` is preserved in the repo for historical
reference (576 lines, append-only batch log; not maintained going forward).
It can be deleted once `grade-jobs` Phase 3 has run end-to-end against the
new folder structure.
