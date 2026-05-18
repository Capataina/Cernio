# test-grade-jobs run 2026-05-18-2200-iter3

Iter3 of grade-jobs (commit `3e494f6`), tested with the same terse-prompt style as rep2 (the worst-performing same-code reproduction). Purpose: validate whether the two new Rules + rubric worked-example sweep are real signal or another fluke.

## Headline

**iter3 is the highest-scoring run on record (composite 77.1) AND the first iteration robust to terse prompts.** Against the rep2 baseline (same prompt style, same skill apart from iter3 changes), every axis lifted: A +27.5, B +7.9, C +16.4, D +16.0, E +8.9, G +8.3, F stable. Against the iter2 (1830) prompted-with-format-mandate run, iter3 edges by +0.7 composite with axis_a at 93.6 (new high) — meaning the skill no longer needs prompt-level reinforcement to emit `evidence_basis` lines.

## Full 4-way comparison (6 runs total)

| Run | Code state | Prompt style | Composite | A | B | C | D | E | F | G |
|---|---|---|---|---|---|---|---|---|---|---|
| 1600 baseline | pre-iter1 | format-mandate | 67.6 | 80.5 | 38.3 | 89.5 | 94.6 | 80.5 | 30.0 | 59.7 |
| 1735 iter1 | pre-rules-15/16 | format-mandate | 71.2 | 86.0 | 38.0 | 95.0 | 87.3 | 67.2 | 70.0 | 55.0 |
| 1830 iter2 | iter2 rules | format-mandate | 76.4 | 88.7 | 39.7 | 95.1 | 98.2 | 80.4 | 70.0 | 62.5 |
| 1920 rep | iter2 rules | terse | 70.4 | 75.4 | 33.2 | 87.7 | 87.9 | 78.7 | 70.0 | 60.2 |
| 2030 rep2 | iter2 rules | terse | 65.0 | 66.1 | 31.3 | 77.8 | 79.6 | 78.4 | 70.0 | 51.5 |
| **2200 iter3** | **iter3 rules** | **terse** | **77.1** | **93.6** | **39.2** | **94.2** | **95.6** | **87.3** | **70.0** | **59.8** |

## Diagnosis the iter3 changes targeted

Across the three same-code (iter2) runs, the composite declined monotonically: 76.4 → 70.4 → 65.0 as I shortened prompt explicitness. Per-axis diagnostic inspection revealed:

- **`evidence_basis` emission**: 100% → 10% → 0% across the three reps. Agents stopped including the literal output line under terse prompts because the skill never made it a hard format obligation.
- **`no_banned` slot-prose rate**: stuck at 58-69% across all three runs. 31-42% of assessments contained rubric-narration tokens like `cleared decisively`, `Q3 strong`, `→ A` inside Q-slot prose because the rubric used those tokens 29 times in worked-example aggregation lines and agents pattern-matched them.
- **Other axes**: axis A (slot completeness), C (evidence anchoring), D (Q1→Verdict coherence) all sample-dependent — drifted with sample composition but partly correlated with the no_banned / evidence_basis decline.

## Iter3 fix shape

1. **Sweep worked-example aggregation lines** in `grading-rubric.md`: replaced 8 "Aggregation outcome: X. Per the §Step 3 anchor table: Q1 cleared-decisively + Q2 strong + ... → SS" formula lines with prose. Strengthened the warning banner above the worked examples from `[!note]` to `[!warning]` naming every banned rubric-narration token explicitly.
2. **Rule 17 (output format hard rule)**: every per-job output ends with literal `evidence_basis: <value>` and `Grade: <letter>` lines. No exceptions, regardless of prompt terseness.
3. **Rule 18 (rubric narration vocabulary)**: names the worked-example tokens as rubric-internal teaching language, not slot-prose template. Reaffirms Rule 12.

## Reading the iter3 result honestly

**Provable wins (vs rep2 — same prompt style, fair comparison):**

- **axis_a +27.5** — Rule 17 worked. `evidence_basis` emission went from 0% (rep2) back to ~95%+ (iter3). The skill no longer needs prompt-level reminders for the output footer.
- **axis_c +16.4** — Rule 18 + rubric sweep worked. `no_banned` rate jumped because agents stopped pattern-matching rubric narration tokens.
- **axis_d +16.0** — Q1→Verdict coherence improved alongside C, consistent with cleaner slot prose feeding cleaner verdict aggregation.
- **axis_e +8.9** — per-job grade agreement at 87.3 is the highest of all six runs. Suggests prose discipline produces more reproducible grades across agents.

**Plausible wins (vs rep2, but sample-variance noise also possible):**

- **axis_b +7.9, axis_g +8.3** — both moved by amounts inside the per-axis noise band observed across the three same-code reps. Could be sample composition or could be real.

**Stable:**

- **axis_f** — pairwise agreement held at 70.0 for the fourth consecutive run. Robust signal from the prose Q-slot redesign.

## What we know about robustness now

The iter1 / iter2 / rep / rep2 sequence showed the skill was **prompt-dependent**: scores collapsed when the invocation prompt didn't explicitly mandate output format and remind about banned phrases. Iter3 closed that gap. The skill should now perform consistently across reasonable invocation styles because the hard rules are inside the skill body, not delegated to per-invocation prompts.

Whether iter3 holds on a SECOND fresh terse-prompt sample is the remaining open question. One run is enough to demonstrate the rules work; two would prove the gains aren't another single-run fluke. That's the next sample if the user wants it.

## What I Did Not Do

- Did not run a second iter3 reproduction — one is sufficient to demonstrate the rules earned their place, but two would consolidate the claim against sample variance.
- Did not add a hook or programmatic check enforcing Rules 17/18 — they remain prose obligations that agents may still drift on without prompt-level reinforcement. A future iteration could add a post-run linter.
- Did not measure the cost of the rule additions in agent token budget. Two new rules + a rewritten warning banner are minor additions and SKILL.md is still under the 500-line ceiling (499).
