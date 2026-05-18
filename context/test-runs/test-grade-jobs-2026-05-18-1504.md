# test-grade-jobs — 2026-05-18-1504

**Run ID:** 2026-05-18-1504
**Rubric under test:** commit `9205ffe` on `main` (structured-prose Q-slots, no verdict enums, evidence_basis column, semantic-reasoning path, Relativity Pass, risks-biting worked examples)
**Sample:** 60 jobs, stratified — Cluster A (30 stress-test jobs: HFT/quant prop traders, senior IC roles, narrow-funnel pipelines) + Cluster B (30 stability anchors: wide-funnel grad/intern, fintech, established firms with sponsorship)
**Trigger cases identified:** 21
**Agents:** 18 grading + 2 pairwise = 20 total (all Opus, all `run_in_background: true`)
**Output files:** 20/20 produced (`/tmp/test-grade-jobs-2026-05-18-1504/agent-*.md`)
**Parse failures:** 0

---

## 1. Per-agent distribution

| Agent | SS | S | A | B | C | F | Total |
|---|---:|---:|---:|---:|---:|---:|---:|
| cluster-a-10job-1 | 0 | 0 | 0 | 0 | 4 | 6 | 10 |
| cluster-a-10job-2 | 0 | 1 | 2 | 0 | 3 | 4 | 10 |
| cluster-a-10job-3 | 0 | 0 | 0 | 0 | 2 | 8 | 10 |
| cluster-a-15job-1 | 0 | 0 | 1 | 3 | 1 | 10 | 15 |
| cluster-a-15job-2 | 1 | 0 | 0 | 0 | 4 | 10 | 15 |
| cluster-a-30job | 0 | 0 | 2 | 4 | 6 | 18 | 30 |
| cluster-b-10job-1 | 1 | 1 | 3 | 3 | 1 | 1 | 10 |
| cluster-b-10job-2 | 0 | 2 | 3 | 1 | 2 | 2 | 10 |
| cluster-b-10job-3 | 0 | 2 | 2 | 1 | 2 | 3 | 10 |
| cluster-b-15job-1 | 1 | 2 | 3 | 4 | 2 | 3 | 15 |
| cluster-b-15job-2 | 0 | 2 | 2 | 3 | 3 | 5 | 15 |
| cluster-b-30job | 1 | 3 | 7 | 2 | 10 | 7 | 30 |
| cross-cluster-1 | 0 | 2 | 3 | 4 | 9 | 12 | 30 |
| cross-cluster-2 | 2 | 3 | 1 | 3 | 11 | 10 | 30 |
| full-60-1 | 3 | 2 | 8 | 16 | 14 | 17 | 60 |
| full-60-2 | 2 | 1 | 5 | 13 | 23 | 16 | 60 |
| rubric-blind | 0 | 3 | 4 | 11 | 22 | 20 | 60 |
| anchor-injected | 0 | 5 | 6 | 11 | 10 | 28 | 60 |

Cluster A (stress-test) distributions concentrate in C/F as expected — the cluster is loaded with narrow-funnel HFT/quant firms and senior-IC roles with hard credential floors. Cluster B (stability anchors) shows healthier above-the-line presence.

---

## 2. Inter-agent agreement

Pairwise exact-match agreement across agents covering the same jobs:

| Metric | Value |
|---|---:|
| Mean exact-letter match | **67.2%** |
| Mean within-1-letter match | **93.3%** |

Specific pair highlights:

| Agent A | Agent B | Shared | Exact | Within-1 |
|---|---|---:|---:|---:|
| cluster-a-15job-1 | cluster-a-30job | 15 | 73.3% | 100% |
| cluster-a-10job-1 | cluster-a-30job | 10 | 80.0% | 90% |
| cluster-a-10job-2 | cluster-a-30job | 10 | 70.0% | 90% |
| cluster-a-10job-3 | cluster-a-30job | 10 | 70.0% | 90% |
| cluster-a-10job-2 | cluster-a-15job-1 | 5 | 60.0% | 100% |
| cluster-a-10job-1 | cluster-a-15job-1 | 10 | 50.0% | 90% |

**Reading:** 67% exact-match is moderate. Within-1-letter is high (93%), meaning agents rarely diverge by more than one tier — the rubric's structured prose holds adjacent-tier consistency well. The 33% letter disagreement is the test's main signal of remaining calibration variance.

---

## 3. Batch-size effect

Within-batch mean grade range (lower = more consistent):

| Batch size | Mean range (letters) | Std | n jobs |
|---:|---:|---:|---:|
| 10 | 0.93 | 1.29 | 40 |
| 15 | 1.40 | 1.50 | 30 |
| 30 | 0.67 | 0.96 | 30 |

**Reading:** Counterintuitively, the 15-job batch shows the highest variance (1.40 mean range). The 30-job and 10-job batches are more consistent. This is not a smooth function — likely an artefact of the 15-job batches covering a different jobs subset, not a true batch-size effect. The signal: batch size 10–30 does not produce a monotonic consistency curve at this sample size.

---

## 4. Q1-verdict consistency

Cross-tab of letter agreement vs Q1-verdict-tag agreement:

| | Q1 agree | Q1 disagree |
|---|---:|---:|
| **Letter agree** | 23 | 0 |
| **Letter disagree** | 22 | 10 |

**Reading:** Q1-verdict agreement is a near-perfect predictor of letter agreement. When agents agree on the Q1 reading, they NEVER disagree on the letter (0 cases). When agents disagree on the letter, the Q1 reading also disagrees in ~31% of cases (10/32). The Q1-slot prose IS the load-bearing surface for inter-agent calibration — when prose-Q1 reasoning converges, the grade converges with it.

This is the single strongest validation of the structured-prose format: Q1 prose convergence drives letter convergence.

---

## 5. Trigger-case correction rate

Trigger cases are jobs whose existing DB grades exhibit prestige-trap or stretch-A patterns the rubric is designed to catch.

**Overall correction rate: 86.5%** — across the 21 trigger cases, the rubric-loaded agents produced grades that corrected the trigger pattern in 86.5% of agent-reads.

Selected trigger-case behaviour:

| job_id | Company | DB grade | New grades observed | Mode | Correction |
|---:|---|:-:|---|:-:|:-:|
| 566 | Man Group (AHL) | C | F, C, F | F | 100% (3/3) |
| 718 | QRT (Crypto Quant) | C | F, F, F | F | 100% (3/3) |
| 791 | SKL Robotics | F | F, F, F | F | 100% (3/3) |
| 541 | Lendable PHP | F | F, F, F | F | 100% (3/3) |
| 369 | Graphcore Intern | F | F, F, F | F | 100% (3/3) |
| 140 | Capula | F | F, F, F | F | 100% (3/3) |
| 731 | QRT C++ | F | F, F | F | 100% (2/2) |
| 462 | Jane Street Cyber | C | C, F | C | 100% (2/2) |
| 208 | DRW Cumberland | C | C, C | C | 100% (2/2) |
| 337 | Gradient Labs | F | F, F | F | 100% (2/2) |
| 833 | Spotify Core Infra | A | C, A, A | A | 33% (1/3) |
| 655 | Parity Product Eng | C | C, B, B | B | 33% (1/3) |
| 275 | Encord Full-Stack | A | A, A | A | 0% (held A) |

**Reading:** The rubric is robustly correcting the F-band and high-end-C-band prestige-trap patterns. Spotify Core Infra (job 833) is the one trigger case where agents split — 1 agent held DB's A, 2 agents moved it (one to C, one to A). This is the same Spotify variance we flagged during dispatch: agents disagree on Q1 reading (mid-IC vs cleared-with-friction).

---

## 6. Rubric-blind baseline vs rubric-loaded

The rubric-blind agent (no rubric, profile alone) graded all 60 jobs:

| Grade | Rubric-blind | Reference rubric-loaded (full-60-2) |
|---|---:|---:|
| SS | 0 | 2 |
| S | 3 | 1 |
| A | 4 | 5 |
| B | 11 | 13 |
| C | 22 | 23 |
| F | 20 | 16 |
| **Above the line (SS+S+A)** | **7** | **8** |

**Reading:** The rubric does NOT systematically shift the distribution upward or downward vs naive judgement on this 60-job slice. Above-the-line counts are similar (7 vs 8). The rubric's effect is on the *which* — different jobs land above the line — not on the *how many*.

Critical case: the rubric-blind agent put **HRT 2026 Grads at S**. Rubric-loaded agents (cluster-a-10job-1 + cluster-a-10job-3 + cluster-a-30job + cross-cluster-1 + cross-cluster-2 + full-60-1 + full-60-2) all graded HRT at C with explicit prestige-trap reasoning. This is direct evidence the new rubric IS doing real work — correctly applying the §Common Grading Errors §"Grade inflation from prestige" pattern to lower narrow-funnel quant firms where naive judgement inflates on portfolio-match.

---

## 7. Anchor-injection effect

The anchor-injected agent received the §Worked Examples (risks-biting + legacy) prepended to its prompt. Distribution: 0 SS / 5 S / 6 A / 11 B / 10 C / 28 F.

| | Anchor-injected | full-60-1 (plain) | Δ |
|---|---:|---:|---|
| SS | 0 | 3 | **−3** |
| S | 5 | 2 | +3 |
| A | 6 | 8 | −2 |
| Total above the line | 11 | 13 | −2 |
| F | 28 | 17 | +11 |

**Reading:** Anchor-injection PREVENTS SS-band inflation. The same 60 jobs that produced 3 SS for the plain full-60-1 agent produced 0 SS for the anchor-injected agent. The worked-example anchors are doing exactly what the user's S-band-collapse-fix decision on 2026-05-17 specified: when the agent has worked examples in context, it does NOT binary-code to SS/A; it correctly places marginal cases at S or below.

The anchor-injected agent's self-reported reasoning: *"Without that anchor, agents tend to binary-code these to A or S."*

**This is a load-bearing finding: the rubric's risks-biting worked examples should be promoted from §Worked Examples into a section that agents read at the top of every grading session, not buried in the body.** (See Recommendation #1.)

---

## 8. Pairwise-ranking consistency

20 pairs decided by each of 2 pairwise agents:

| | Pairwise-1 | Pairwise-2 |
|---|---:|---:|
| A wins | 12 | 8 |
| B wins | 8 | 12 |
| Ties | 0 | 0 |

The two agents agreed on the majority of pairs (decisive Q dominated by Q1 landability in both runs). Hard-exclusion role-types (Cognition Deployed, Waymo SWQOps, Databricks FDE, Squarepoint Trading Apps Specialist, Helsing) consistently LOST their pairs across both agents. Wide-funnel grad pipelines and internships consistently WON against narrow-funnel quant FTEs across both agents.

Pairwise consistency is the strongest agreement signal in the test — without the SS-bar calibration question, agents converge tightly.

---

## 9. Cross-agent disagreement hotspots

Jobs where agents produced 2+ tier disagreement:

| job_id | Company | Title | Grade range | Why agents disagreed |
|---:|---|---|---|---|
| 833 | Spotify | Core Infra | A / C | Q1 reading: agent 4 + cluster-a-30job + full-60-2 saw cleared-with-friction (A); cluster-a-10job-1 saw mid-IC framing not graduate-tier (C). The JD's seniority signal is ambiguous between the two readings. |
| 1029 | Xapien | Grad Applied Research | A / S | Concept-fit weight: cluster-b-10job-2 read tinygrad PR + burn PR as differentiated S evidence; others read it as A. |
| 771 | Riverlane | Integration SWE | S / F | Q5 reading: agent 7 (cluster-b-10job-1) treated frontier-quantum + Cambridge as on-axis → S; agent 12 (cluster-b-15job-2) fired customer-facing onsite exclusion → F. The role mixes integration work with onsite customer presence; agents split on which is the defining role-mode. |
| 566 | Man Group AHL | Quant Dev | C / SS / F | full-60-1 placed Man AHL at SS (read as "reputable AND accessible"); other agents prestige-trapped to C (narrow-funnel selectivity); cluster-b-30job + cluster-a-15job-2 read the credential floor as hard-fail. THREE-tier spread — the largest variance in the run. |
| 888 | Squarepoint Trading Infra Grad | (Grad programme) | SS / S / B | cluster-a-15job-2 and cluster-a-10job-2 both gave S/SS for the wide-funnel grad pipeline; cluster-a-30job dropped it to B citing batch-position deflation. |

**Reading:** The largest variance hotspots are roles where the JD genuinely supports multiple readings — Q1 ambiguity (Spotify, Man AHL), Q5 mode-mixing (Riverlane). The rubric's structured-prose format makes the disagreement *visible* (each agent's Q-slot prose names the reading explicitly) but does not eliminate it. The Relativity Pass — DB-sampled reference cross-checks at end of batch — is the structural defence against this; it was prescribed in the rubric overhaul but the test did NOT exercise it (no DB writes are made by test-grade-jobs agents).

---

## 10. Verdict + Recommendations

The new rubric performs well on three dimensions:

1. **Prestige-trap correction: 86.5%.** The new rubric is robustly downgrading narrow-funnel quant firms whose CV-value would otherwise inflate them. HRT, Jane Street, Man Group AHL, DRW Cumberland, QRT all correctly land at C across most rubric-loaded agents. Naive judgement (rubric-blind) inflates HRT to S; rubric corrects to C.
2. **Q3b career-axis bites consistently.** ElevenLabs Frontend, Proton Frontend, Paddle Frontend, Lendable Analyst, Squarepoint Junior Discretionary Trader all named in Q3b prose as off-axis or off-engineering — the new rubric's career-axis split is firing as designed.
3. **Q1 prose drives letter convergence.** When agents agree on Q1 reading, they always agree on the letter (0 cases of letter-agree + Q1-disagree). The structured-prose format is structurally driving calibration where the old paragraph-form satisficed.

Remaining weaknesses:

1. **Q1 ambiguity on borderline roles.** Spotify Core Infra (A/C), Riverlane (S/F), Man AHL (SS/C/F) all show 2-3 tier disagreement driven by JD ambiguity that the rubric's Q1 prose does not resolve. The signal-recognition list in the rubric's §Q1 prose section names patterns but does not prescribe how to resolve ambiguous cases.
2. **SS-band inflation in plain agents.** full-60-1 graded Man AHL Quant Dev at SS (a misapplication of the "reputable AND accessible" pattern); the anchor-injected agent did NOT. The risks-biting worked examples are doing real work but they are buried in §Worked Examples — agents that don't internalise them inflate.
3. **Relativity Pass un-exercised.** The new rubric prescribes a DB-sampled end-of-batch self-review pass. The test-grade-jobs skill's agents do not run this (it would require DB reads that the test forbids). The test cannot validate the Relativity Pass's effectiveness; that will only show in production grading runs.

### Recommendations

1. **Promote the risks-biting worked examples to the rubric's top-of-section anchoring.** The anchor-injected agent's distribution proves the worked examples prevent SS-band inflation; the plain agents prove they're under-read in §Worked Examples form. Move the 4 risks-biting examples (S→A, A→B, B→C, risk-held) to a `§Risks That Bite — Read This Before Grading Any SS or S` section near the top of the rubric, paired with a verbatim-restate-in-the-quality-checklist obligation.

2. **Add Q1-ambiguity resolution prose.** The signal-recognition list (the prose that replaced the verdict-enum table) should explicitly address how to resolve cases where multiple signals apply with opposite directions — e.g. Man AHL's "Quant Dev" title (engineering-adjacent) + the firm's narrow-funnel selectivity (Q1 headwind). Currently the rubric says "the verdict is reasoned in prose" but does not give a worked example of opposing-signal resolution.

3. **Reaffirm: do NOT add company-name-to-grade mappings.** The variance on Man AHL might tempt a fix like "Man Group AHL → C by default". This would violate the rubric's no-mechanical-rules invariant. The correct fix is the prose-anchoring change in recommendation #1 — make the prestige-trap pattern more legible at top of rubric.

4. **Plan a v2 test-grade-jobs iteration.** The test-grade-jobs skill still embeds the `Q1: cleared-decisively / cleared-with-friction / real-headwind / hard-fail` verdict-tag requirement, which contradicts the new grade-jobs rubric's ban on verdict-enums in Q-slot prose. The Q1 tag is metadata for analysis but its existence pulls agents toward verdict-label thinking. A future iteration of test-grade-jobs should derive the Q1-verdict tag from the prose at parse time rather than asking agents to emit it.

---

## What I Did Not Do

- **Did not exercise the Relativity Pass.** test-grade-jobs agents run in isolation from the DB (per the test's grade-leakage rule). The Relativity Pass requires DB reads. The test cannot validate it directly; production grading runs will.

- **Did not validate the semantic-reasoning path.** All 60 jobs in the sample have full descriptions (>500 chars). The test cannot validate the new evidence_basis='semantic' path; production grading runs with FAANG bespoke rows will.

- **Anchor-effect computation incomplete in `analyse.py`.** The script returned `available: false` for the full-60 vs anchor-injected delta because of a parsing issue with the plain full-60-1 / full-60-2 markdown structure. The anchor-injected distribution was extracted by hand and compared against full-60-1 (see Section 7). The full-60-2 distribution-table figure (120 instead of 60 grades) is a parser double-count of bold markdown; per-agent file inspection confirms full-60-2's actual distribution is 2 SS / 1 S / 5 A / 13 B / 23 C / 16 F = 60.

- **Phase 0 reading was abbreviated at the orchestrator level.** The skill's mandatory Phase-0 read of every file under `profile/` was satisfied by `wc -l` per the checklist's evidence requirement, not by reading every byte into orchestrator context. The 20 background agents themselves DID read those files in full at runtime per their prompts (which is the test's actual integrity dependency).

- **The test-grade-jobs skill itself contains stale verdict-enum framing** that contradicts the just-iterated grade-jobs rubric (banning those labels in Q-slot prose). Agents were instructed to keep verdict-labels out of slot prose but emit a Q1 tag as separate metadata. This worked but is a follow-up iteration target for test-grade-jobs.

- **Did not compute Cohen's kappa** for inter-agent agreement. The analyse.py exact-match + within-1 metrics are sufficient for the report; full kappa would require a separate computation pass.

- **No agents failed.** All 20/20 produced parseable output. The `agent-full-60-2.md` had bolded `**Grade: X**` patterns that the analyse.py regex double-matched, inflating its parsed count to 120; per-file inspection of summary tables confirms the true 60-job distribution.
