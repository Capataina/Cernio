---
title: Portfolio Gaps
last_updated: 2026-06-01
maintained_by: grade-jobs (per cernio-full-refactor.md §5.6 Phase 3)
last_run: 2026-06-01 grade-jobs full pass (1460 jobs, all 8 lanes regenerated)
---

# Portfolio Gaps — Per-Lane Folder

> Per-lane files rewritten in place by `grade-jobs` Phase 3. Each file is the
> current snapshot of what the lane wants vs what the profile shows, derived
> from the graded jobs in that lane. NOT append-only — re-running `grade-jobs`
> replaces each lane's content from scratch.

## Last full regeneration

| Lane | Jobs graded | SS | S | A | B | C | F |
|---|---:|---:|---:|---:|---:|---:|---:|
| big-tech | 85 | 8 | 7 | 19 | 11 | 8 | 32 |
| ai-ml | 358 | 0 | 25 | 87 | 55 | 81 | 110 |
| hft | 217 | 0 | 10 | 9 | 42 | 85 | 71 |
| crypto-mm | 33 | 0 | 10 | 3 | 3 | 7 | 10 |
| bank-strats | 82 | 4 | 4 | 14 | 12 | 23 | 25 |
| systems-infra | 400 | 6 | 14 | 38 | 71 | 91 | 180 |
| devtools | 97 | 0 | 4 | 3 | 14 | 13 | 63 |
| fintech | 187 | 0 | 4 | 28 | 26 | 25 | 104 |

(Counts as of 2026-06-01; rerun grade-jobs to refresh.)

## Per-lane files

| File | Lane | Maintenance source |
|---|---|---|
| `big-tech.md` | big-tech | grade-jobs Phase 3 |
| `ai-ml.md` | ai-ml | grade-jobs Phase 3 |
| `hft.md` | hft | grade-jobs Phase 3 |
| `crypto-mm.md` | crypto-mm | grade-jobs Phase 3 |
| `bank-strats.md` | bank-strats | grade-jobs Phase 3 |
| `systems-infra.md` | systems-infra | grade-jobs Phase 3 |
| `devtools.md` | devtools | grade-jobs Phase 3 |
| `fintech.md` | fintech | grade-jobs Phase 3 |
| `closed.md` | (lane-agnostic) | dated archive of closed gaps |

## File shape

Each per-lane file follows the template:

- Open Gaps (what the lane wants that the profile lacks; cited, counted, prescription-attached)
- Confirmed Strengths (what the profile has that the lane values)
- Pinnacle Anchors (what would land SS at the lane's pinnacle firms; what's missing for that)
- Lane Internal Calibration (current pool placement; biggest single-addition lever)

## Cross-lane patterns

Most-cited closures that span multiple lanes:
- **Kubernetes operator + Terraform IaC artefact** — surfaces in big-tech, systems-infra, fintech, devtools. One closure unlocks ~20+ A-band rows.
- **FIX protocol literacy + adapter on Nyquestro** — unlocks hft + crypto-mm + bank-strats pinnacle-band roles.
- **C++ at production band** — hft #1, crypto-mm #1, systems-infra adjacent.
- **kdb+/q literacy** — bank-strats + hft (Squarepoint, Goldman, Citi).
- **RLHF + interpretability paper** — ai-ml pinnacle gating for Anthropic Research-Engineer track.

## Closure prescriptions (top 5 by leverage)

1. **k3s + observability artefact** — closes Kubernetes/Terraform across 4 lanes.
2. **FIX 4.4 adapter on Nyquestro** — closes the hft/crypto-mm pinnacle gap with a single ~3-week build.
3. **C++ port of one Nyquestro module + criterion benchmark** — dual-lane unlock + signals C++ portability.
4. **kdb+/q micro-project** — bank-strats + hft credibility, ~1-week investment.
5. **RLHF reproduction on HH-RLHF via TRL** — ai-ml Research-Engineer track unlock, ~2 weeks.
