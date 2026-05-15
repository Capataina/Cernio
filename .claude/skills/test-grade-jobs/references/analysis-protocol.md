# Analysis Protocol

## Table of Contents

- [Purpose](#purpose)
- [Inputs](#inputs)
- [The 10 mandatory analysis sections](#the-10-mandatory-analysis-sections)
- [Section 1: Per-job grade distribution table](#section-1-per-job-grade-distribution-table)
- [Section 2: Inter-agent agreement](#section-2-inter-agent-agreement)
- [Section 3: Batch-size effect](#section-3-batch-size-effect)
- [Section 4: Cluster-position effect](#section-4-cluster-position-effect)
- [Section 5: Trigger-case correction rate](#section-5-trigger-case-correction-rate)
- [Section 6: Q1-verdict consistency](#section-6-q1-verdict-consistency)
- [Section 7: Rubric-blind baseline comparison](#section-7-rubric-blind-baseline-comparison)
- [Section 8: Anchor-injection effect](#section-8-anchor-injection-effect)
- [Section 9: Pairwise-ranking consistency](#section-9-pairwise-ranking-consistency)
- [Section 10: Calibration verdict + recommendations](#section-10-calibration-verdict--recommendations)
- [Numericisation of letter grades](#numericisation-of-letter-grades)
- [Report file audience](#report-file-audience)
- [Anti-Patterns](#anti-patterns)

---

## Purpose

The analysis phase converts the per-agent markdown outputs into a single comprehensive report. This reference describes what each section computes, how to compute it, and what the section's output looks like.

The report is human-facing — the user reads it on its own merits to understand how the rubric is performing. Use tight signal density, no process language, reading-grade markdown. Tables wherever the output is comparison-shaped (which is most sections).

`scripts/analyse.py` does the parsing and intermediate computation. The agent writes the report by reading the script's intermediate JSONs and composing the narrative + tables. The script does NOT write the report itself — the report needs judgement (e.g. choosing which findings to highlight, framing the calibration verdict), which a script can't supply.

---

## Inputs

After Phase 3 completes, the analysis script has access to:

| Input | Path | Contents |
|---|---|---|
| Per-agent outputs | `/tmp/test-grade-jobs-<run-id>/agent-*.md` | 20 markdown files, one per agent |
| Coverage matrix | `/tmp/test-grade-jobs-<run-id>/coverage-matrix.json` | job_id → list of agent IDs that graded it |
| Cluster A jobs | `/tmp/test-grade-jobs-<run-id>/cluster-a.json` | 30 job IDs |
| Cluster B jobs | `/tmp/test-grade-jobs-<run-id>/cluster-b.json` | 30 job IDs |
| All jobs | `/tmp/test-grade-jobs-<run-id>/jobs-all.json` | 60 full job records |
| Trigger cases | `/tmp/test-grade-jobs-<run-id>/trigger-cases.json` | subset of cluster A IDs |
| Old DB grades | `/tmp/test-grade-jobs-<run-id>/db-grades.json` | job_id → existing DB grade (read separately, NOT in any agent's manifest) |

The script parses every per-agent output into structured form:

```json
{
  "agent_id": "cluster-a-30job",
  "agent_role": "core-grading",
  "batch_size": 30,
  "cluster_scope": "a-only",
  "grades": [
    {"job_id": 2447, "grade": "C", "q1_verdict": "real-headwind", "reasoning_summary": "..."},
    ...
  ],
  "summary_table_intact": true
}
```

These structured records feed every analysis section.

---

## The 10 mandatory analysis sections

Each section is mandatory on every run. If a section lacks data (e.g. zero trigger cases this run), the section still appears but states "no data this run; cause: ..." rather than being silently omitted.

| # | Section | Headline metric | Output shape |
|---|---|---|---|
| 1 | Per-job grade distribution | mode + range per job | One row per job (60 rows) |
| 2 | Inter-agent agreement | % exact match, % within-1-letter | One number + most-disagreed table |
| 3 | Batch-size effect | mean grade by batch-size bucket | 3-row mean comparison |
| 4 | Cluster-position effect | grade stability per job across cluster vs cross-cluster agents | Per-job-pair table |
| 5 | Trigger-case correction rate | % of trigger cases that aggregated to C/F | Single number + per-trigger-case table |
| 6 | Q1-verdict consistency | agent agreement on Q1 verdict regardless of letter | Headline number + disagreement modes |
| 7 | Rubric-blind comparison | distribution gap (blind vs rubric-loaded) | Distribution-comparison table |
| 8 | Anchor-injection effect | distribution gap (anchor-injected vs plain full-60) | Distribution-comparison table |
| 9 | Pairwise-ranking consistency | letter-rank vs pair-rank disagreement count | Per-disagreement table |
| 10 | Calibration verdict | working / partially-working / not-working with reasoning | Plain English |

---

## Section 1: Per-job grade distribution table

For each of the 60 jobs, list every grade the job received across the agents that saw it.

```python
# pseudocode
for job_id, agent_ids in coverage_matrix.items():
    grades = [parse_grade(agents[aid], job_id) for aid in agent_ids]
    mode = statistics.mode(grades)
    grade_range_letters = letter_range(grades)  # e.g. ["C","B","A"] -> 2 letters
    agreement_pct = grades.count(mode) / len(grades) * 100
```

**Output shape (per-job rows):**

| Job ID | Company | Title | DB grade | Grades observed | Mode | Range | Agreement | Trigger? |
|---|---|---|---|---|---|---|---|---|
| 2447 | HRT | SWE Grad | SS | [C, C, F, C, C, C, F] | C | 2 letters | 5/7 = 71% | ✓ |
| 3134 | Lendable | Grad Analyst | B | [B, B, B, B, C, B, B] | B | 1 letter | 6/7 = 86% | — |
| ... | | | | | | | | |

Sort rows by `Range` descending (most-disagreed jobs at top) so the reader's eye lands on the controversial cases first.

**Headline at the top of the section:**

- "X of 60 jobs had unanimous agreement (range = 0 letters)."
- "Y jobs had broad agreement (range ≤ 1 letter)."
- "Z jobs had broad disagreement (range ≥ 3 letters) — flagged as calibration-fuzzy cases."

---

## Section 2: Inter-agent agreement

Pairwise % exact match across the 16 core grading agents. Also report % within-1-letter (treating B and A as "close enough" but B and SS as not).

```python
# pseudocode
agreement_matrix = {}
for a1, a2 in itertools.combinations(core_agents, 2):
    shared_jobs = set(a1.jobs) & set(a2.jobs)
    if not shared_jobs: continue
    exact_match = sum(1 for j in shared_jobs if a1.grade(j) == a2.grade(j)) / len(shared_jobs)
    within_one = sum(1 for j in shared_jobs if abs(letter_to_num(a1.grade(j)) - letter_to_num(a2.grade(j))) <= 1) / len(shared_jobs)
    agreement_matrix[(a1, a2)] = (exact_match, within_one)
```

**Headline numbers:**

- Mean pairwise % exact match across all agent pairs (excluding pairs with no shared jobs).
- Mean pairwise % within-1-letter.

**Cohen's-kappa equivalent (if there are enough shared jobs per pair):**

```python
from sklearn.metrics import cohen_kappa_score
# Use ordinal kappa given letter grades have a natural order
kappa = cohen_kappa_score(a1_grades, a2_grades, weights='quadratic')
```

If sklearn isn't available, report a simpler metric: % exact match adjusted by chance agreement (expected agreement = 1/6 for random uniform across SS/S/A/B/C/F).

**Most-disagreed jobs table** (the 5 jobs with the largest letter-range across their agents):

| Job ID | Company | Title | Grades | Range | Likely cause |
|---|---|---|---|---|---|
| ... | | | | | (e.g. "split on Q1 reading — half saw '2+ years' as borderline-clearable, half as hard floor") |

---

## Section 3: Batch-size effect

Bucket the agent grades by batch size: 10-job-batch, 15-job-batch, 30-job-batch, 60-job-batch. For each job, compute the mean numericised grade in each bucket. Then compute the per-job difference between buckets.

The hypothesis being tested: do agents grading 60 jobs at once satisfice toward a distribution (e.g. drift toward the mode) compared to agents grading 10 jobs each? If yes, the 60-job mean grades are closer to each other than the 10-job mean grades (compressed distribution).

```python
# pseudocode
per_job_by_bucket = {
    job_id: {
        "10-batch": mean([grade for agent in 10-batch-agents if job_id in agent.jobs]),
        "15-batch": ...,
        "30-batch": ...,
        "60-batch": ...,
    } for job_id in coverage_matrix
}
# Compute std dev of per-job means within each bucket
std_devs = {bucket: stdev([per_job_by_bucket[j][bucket] for j in jobs_in_bucket])}
```

**Output shape:**

| Batch size | Mean grade (numericised) | Std dev | Distribution shape |
|---|---|---|---|
| 10-job | 2.7 | 1.3 | wider spread |
| 15-job | 2.8 | 1.2 | |
| 30-job | 2.9 | 1.0 | |
| 60-job | 3.0 | 0.8 | narrower (toward centre) |

If the std dev narrows monotonically as batch size grows, that's evidence of satisficing toward the mode. State the headline ("60-job agents compressed the distribution by Δσ = X" or "no batch-size effect detected — std dev stable across buckets").

---

## Section 4: Cluster-position effect

For each job, find the agents that saw it in same-cluster context (e.g. cluster A in `cluster-a-30job`) and in cross-cluster context (e.g. mixed in `cross-cluster-1`). Compare the grades.

The hypothesis: does the rubric's grade depend on the composition of the batch around the job? An A-grade job graded inside cluster A (surrounded by stress tests) might shift if graded in cluster B (surrounded by stability anchors).

```python
# pseudocode
position_effect = {}
for job_id in coverage_matrix:
    same_cluster_grades = [agent.grade(job_id) for agent in agents if agent.cluster_scope == job's_cluster_scope and job_id in agent.jobs]
    cross_cluster_grades = [agent.grade(job_id) for agent in agents if agent.cluster_scope == 'cross' and job_id in agent.jobs]
    if same_cluster_grades and cross_cluster_grades:
        position_effect[job_id] = {
            "same_cluster_mean": mean(same_cluster_grades),
            "cross_cluster_mean": mean(cross_cluster_grades),
            "delta": ...,
        }
```

**Output shape:**

| Job ID | Same-cluster mean | Cross-cluster mean | Δ |
|---|---|---|---|
| 2447 | C | C | 0 |
| 3068 | A | B | -1 (shifted down in cross-cluster) |
| ... | | | |

Headline: "X of 60 jobs had Δ ≥ 1 letter between same-cluster and cross-cluster grading — composition-dependence is [absent / present in N% of jobs]."

---

## Section 5: Trigger-case correction rate

For each trigger case (jobs identified by the cluster-design heuristic as having "stretch" / "lottery" / "sub-1%" / etc. in their old fit_assessment), check: did the new rubric aggregate this case down to C or F across the agents that graded it?

```python
# pseudocode
trigger_correction = {}
for trigger_job_id in trigger_cases:
    new_grades = [agent.grade(trigger_job_id) for agent in core_agents if trigger_job_id in agent.jobs]
    corrected_count = sum(1 for g in new_grades if g in ('C', 'F'))
    trigger_correction[trigger_job_id] = corrected_count / len(new_grades)

overall_correction_rate = mean(trigger_correction.values())
```

**Output shape:**

Headline: "N trigger cases identified this run. Trigger-case correction rate: X% across all agents."

| Trigger job ID | Company | Title | DB grade | New grade (mode) | Correction |
|---|---|---|---|---|---|
| 2447 | HRT | SWE Grad | SS | C | ✓ corrected to C |
| 3334 | XTX | Research Tech | A | F | ✓ corrected to F |
| ... | | | | | |

A trigger-case correction rate above ~80% suggests the rubric is reliably catching the cases it was built for. Below ~50% suggests the rubric still has work to do.

If zero trigger cases were identified this run (sparse `portfolio-gaps.md` patterns), state so and note that this section is statistically thin.

---

## Section 6: Q1-verdict consistency

For each job, compare the Q1 verdicts produced by each agent (cleared-decisively / cleared-with-friction / real-headwind / hard-fail). Two separate measures:

- **Letter-agreement WITH Q1-agreement** — agents agree on both letter and Q1. The rubric is being applied consistently end-to-end.
- **Letter-disagreement WITH Q1-agreement** — agents agree on Q1 but produce different letters. The aggregation rule is fuzzy.
- **Letter-disagreement WITH Q1-disagreement** — agents disagree on Q1 itself. The Q1 definition is fuzzy.
- **Letter-agreement WITH Q1-disagreement** — rare but possible. Agents land on the same letter through different reasoning paths.

Different cells indicate different rubric problems. Aggregation fuzziness is fixed by clarifying §How to Grade a Job Step 3. Q1 fuzziness is fixed by sharpening Q1's definition in the rubric.

**Output shape:**

| Pattern | Count of jobs | Implication |
|---|---|---|
| Letter agreement + Q1 agreement | 38 | Rubric applied consistently |
| Letter agreement + Q1 disagreement | 4 | Different paths to same letter — usually OK |
| Letter disagreement + Q1 agreement | 12 | Aggregation rule is fuzzy — sharpen Step 3 |
| Letter disagreement + Q1 disagreement | 6 | Q1 definition is fuzzy — sharpen Q1 wording |

Then list the top examples in each non-trivial category (especially Letter-disagreement + Q1-agreement, which is the strongest signal for rubric iteration).

---

## Section 7: Rubric-blind baseline comparison

The single rubric-blind agent graded all 60 jobs using its own judgement (no rubric). Compare its distribution to the aggregate distribution of the rubric-loaded agents.

The hypothesis: if the rubric-blind agent's distribution matches the rubric-loaded distribution, the rubric isn't doing much work — competent judgement reaches the same place. If they differ, the rubric is steering grades (in some direction).

```python
# pseudocode
blind_distribution = Counter([blind_agent.grade(j) for j in all_60_jobs])
rubric_aggregate_distribution = Counter([mode_of(grades_for_job(j, core_agents)) for j in all_60_jobs])
```

**Output shape:**

| Grade | Rubric-blind count | Rubric-loaded mode count | Δ |
|---|---|---|---|
| SS | 0 | 0 | 0 |
| S | 4 | 2 | +2 (blind grades more S) |
| A | 12 | 5 | +7 |
| B | 18 | 14 | +4 |
| C | 19 | 28 | -9 |
| F | 7 | 11 | -4 |

If the rubric-loaded distribution skews more pessimistic than the blind one (more C/F, fewer S/A), the rubric is correcting upward bias. If they match, the rubric isn't doing much. State the verdict.

Also list the specific jobs where blind and rubric-loaded disagreed by ≥1 letter — these are the rubric's high-leverage cases.

---

## Section 8: Anchor-injection effect

The single anchor-injected agent saw the rubric's worked examples (Amazon SDE-I=SS, Jane Street=C, Cloudflare Grad=SS) prepended before grading. Compare its 60-job distribution against the two plain full-60 agents.

The hypothesis: if anchoring shifts the distribution toward the anchors (more SS / more C, fewer middle grades), the rubric is anchor-bias-sensitive. If the distributions match, anchoring isn't doing significant work — the rubric's prose stands on its own.

**Output shape:** same distribution-comparison table as Section 7, plus a paragraph naming the most-shifted job classes.

If anchoring substantially shifts grades, that's evidence the rubric should either embrace anchoring (add more anchors) or refactor to be less anchor-dependent.

---

## Section 9: Pairwise-ranking consistency

The two pairwise-ranking agents each ranked ~20 pairs. Build a partial-order graph from the pair winners: edge `A > B` if pair-A-winner. Compare against the letter grades from the core agents: do the pairs the letter grades say "A > B" also win in the pair rankings?

```python
# pseudocode
disagreements = []
for pair in all_pairs:
    a_letter = mode_grade(pair.job_a)
    b_letter = mode_grade(pair.job_b)
    letter_says = compare_letters(a_letter, b_letter)  # 'a>b' / 'b>a' / 'tie'
    pair_says = pair.winner
    if letter_says != pair_says and 'tie' not in (letter_says, pair_says):
        disagreements.append(pair)
```

**Output shape:**

Headline: "X pairwise rankings agreed with letter ordering; Y disagreed. Disagreement rate: Z%."

| Pair ID | Job A | Job B | Letter says | Pair says | Likely cause |
|---|---|---|---|---|---|
| ... | | | A > B | B > A | (e.g. "Letter graded both at B (tie) but pair ranker found A's tech-stack alignment decisive") |

Disagreements between letter-grading and pairwise ranking are the strongest evidence of internal rubric inconsistency — the rubric produces incompatible orderings under different elicitation methods.

---

## Section 10: Calibration verdict + recommendations

Plain English verdict on the rubric's current state, anchored in the prior 9 sections' findings. Three possible verdicts:

- **Working as intended.** Trigger-case correction rate >80%. Inter-agent agreement >70% exact match. Rubric-blind distribution differs meaningfully from rubric-loaded (rubric is doing work). Q1-verdict consistency >75%. No regressions in cluster B.
- **Partially working.** One or more headline metrics is in the middle range. Specific sub-mechanisms work (e.g. Q1 verdicts agree); others don't (e.g. aggregation step is fuzzy).
- **Not working.** Trigger-case correction <50%, OR cluster B has regressions, OR rubric-loaded matches rubric-blind exactly (rubric does no work).

Plus 3-7 specific recommendations anchored in observed disagreements:

- "Sharpen the §How to Grade a Job Step 3 aggregation rule — Section 6 surfaced 12 jobs where agents agreed on Q1 but disagreed on letter, suggesting the aggregation step is the locus of fuzziness."
- "Add a worked example for the 'intern role + 2025 graduate' boundary case — Section 1 surfaced 4 jobs in this pattern with range ≥ 2 letters, suggesting agents don't have shared calibration."
- "Re-examine the SS calibration anchor list — the Cloudflare Intern job (which the rubric lists as SS-anchor) consistently came back as B/A across agents; the anchor may no longer match the agents' aggregation."
- etc.

Recommendations should cite the specific sub-mechanism in the rubric to change AND the section evidence that motivates the change. Vague "improve the rubric" recommendations are non-compliant.

---

## Numericisation of letter grades

For statistical computations, map letters to numbers:

```python
LETTER_TO_NUM = {"SS": 5, "S": 4, "A": 3, "B": 2, "C": 1, "F": 0}
NUM_TO_LETTER = {v: k for k, v in LETTER_TO_NUM.items()}
```

Mean / std dev / range computations use the numeric form. Output uses the letter form (round means to nearest integer letter, or report as a decimal like "C+ (mean 1.6)" when fractional precision is informative).

The numericisation is ordinal — the gap between B and A is treated as equal to the gap between A and S. This is the standard convention for letter-graded systems; if the actual rubric has non-uniform gaps (e.g. SS is much harder to reach than A), note that as a limitation in the report.

---

## Report file audience

The report at `context/test-runs/test-grade-jobs-<run-id>.md` is **human-facing**. The user reads it on its own merits to understand how the rubric is performing.

Per the audience-routing principle:

- Tight signal density. Every paragraph earns its place.
- No process language. The user doesn't need "Phase 1 evidence:" or "Stage 4 output" framing in the report — that belongs in the chat summary or skill log.
- No admin metadata. Frontmatter / evidence-block / WIDND-style sections don't belong in the report itself — they belong in the chat summary the agent prints.
- Reading-grade prose. Markdown formatting calibrated for visual scanning. Tables, callouts, hierarchy.
- Self-contained. The file should make sense to a reader who hasn't seen the chat output.

A 2-4 sentence executive summary at the top of the report (before any of the 10 sections) names the headline finding ("rubric is working as intended" / "X needs iteration") so a skim reader gets the verdict immediately.

---

## Anti-Patterns

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| **Computing only the headline number per section** | Loses the per-job detail that lets the reader trace a finding to specific cases. | Every section produces a headline + a backing table with per-row detail. |
| **Skipping sections that "don't apply" this run** | The 10-section structure makes the test legible across runs. Silent omission breaks comparability. | Sections with sparse data still appear, stating "no data this run; cause: ..." |
| **Subjective verdict without section-evidence anchors** | The Calibration Verdict (§10) reads as opinion without traceability. | Every verdict claim cites the section that motivates it. |
| **Mixing letter grades and numeric forms in the same column** | Reader has to translate as they read. | Letters in the per-row data; numerics in the aggregate computations. |
| **Mean grades reported to 2 decimal places without context** | "Mean grade = 2.43" doesn't tell the reader whether this is good or bad. | Round to nearest letter, or report as "C+ (mean 1.6)" with the letter form in front. |
| **Recommendations not anchored in observed cases** | "Improve the aggregation rule" is satisficing prose. | Cite the specific jobs / sections whose disagreement motivates the change. |

---

## Additive-Freedom Permission for Prescribed Lists in This File

The lists in this file are non-exhaustive and may be extended:

- **The 10 analysis sections** are the current minimum. If a future test dimension warrants a Section 11+ (e.g. "Confidence-Calibration Comparison" if confidence-grading agents are added), add it; the existing 10 remain mandatory.
- **The 4-cell Q1-verdict-consistency taxonomy** (agree/agree, agree/disagree, disagree/agree, disagree/disagree) is the current shape. If the rubric introduces orthogonal axes that warrant a finer-grained taxonomy, extend it.
- **The 3 calibration verdicts** (Working / Partially working / Not working) are the current bands. If a finer scale becomes useful (e.g. five bands), extend; the existing three remain valid anchors.
- **The anti-pattern catalogue** — when a new analysis failure is observed, add a row.

Additions must be purely additive — they may not weaken or replace any existing section.
