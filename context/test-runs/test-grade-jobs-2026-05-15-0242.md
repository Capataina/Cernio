# Test-Grade-Jobs Run 2026-05-15-0242 — Post-Iteration Report

**Verdict: partial improvement.** The grade-jobs iteration successfully eliminated the highest-variance failure mode (Proton-class C/B/A/S spread → unanimous A) and halved the broad-disagreement count (15 → 6 jobs with range ≥ 3 letters). However, mean inter-agent exact-match agreement slipped 1.2pp (52.3% → 51.1%) and the trigger-case correction rate dropped 25pp (84.9% → 59.8%) — the latter partly because the new stack-concentration carveout intentionally lifts some trigger-tagged roles back to A when the candidate has matching stack depth.

The iteration is the "transferring problems" pattern the user warned against: solving the Proton-class variance moved some prestige-trap calibration from "consistently C" to "split A/C". Per the user's "improvement flat-out in all aspects, not transferring problems" standard, this iteration is **not a clean win**. The next move per the user's framework should focus on the skill file (`profile/skills.md` §Concepts and Domains) to give agents sharper signals for the carveout's application.

---

## Run summary

| | Run 1 (pre-iteration) | Run 2 (post-iteration) | Δ |
|---|---|---|---|
| Run ID | 2026-05-15-0138 | 2026-05-15-0242 | |
| Rubric commit | 4983f11 | ec999d5 | iterated |
| Jobs sampled | 60 | 60 | same |
| Trigger cases identified | 17 | 21 | sample difference |
| Agents dispatched | 20 | 20 | same |
| Agents succeeded | 20 | 17 | **−3 failures** (cross-cluster-1, rubric-blind, anchor-injected — all stalled at 600s watchdog) |
| Grades parsed | 480 | ~430 (3 fewer agents) | |

---

## Headline metric comparison

| Metric | Run 1 | Run 2 | Δ | Interpretation |
|---|---|---|---|---|
| **Unanimous jobs (range=0)** | 9 | **10** | +1 | Slight improvement |
| **Within-1-letter (range≤1)** | 34 | 33 | −1 | Essentially flat |
| **Broad disagreement (range≥3)** | **15** | **6** | **−9 (−60%)** | **Major improvement** — the Proton-class variance is largely resolved |
| Mean exact-match agreement | 52.3% | 51.1% | −1.2pp | Slight regression |
| Mean within-1-letter agreement | 87.6% | 85.7% | −1.9pp | Slight regression |
| Trigger-case correction rate | 84.9% | **59.8%** | **−25pp** | **Regression** (partly by design — see below) |
| Trigger cases at 100% correction | 12/17 (71%) | 6/21 (29%) | −42pp | Same as above |
| Q1-agree + letter-disagree count | 11 | 11 | 0 | **No improvement** on aggregation Step 3 fuzziness despite the new anchor table |

---

## The Proton test — the headline win

**The canonical test case from Run 1: Proton Rust SWE (id=2558).** This is what motivated the iteration in the first place.

| Run | Grades observed | Mode | Range | Agreement |
|---|---|---|---|---|
| Run 1 (pre-iteration) | C, C, B, S, A, S | C | **3 letters** | 33% |
| Run 2 (post-iteration) | A, A, A, A, A | **A** | **0** | **100%** |

**The stack-concentration carveout produced unanimous agreement on a job that previously split 4 letters across 6 agents.** Every Run-2 agent grading Proton cited the carveout: 7+ active Rust projects + Proton's "complex Rust pet projects" admission language → ~1.5-letter offset on the stated 1% selectivity → cleared-with-friction + strong Q2-Q5 → A per the §Step 3 anchor table.

This is the iteration's single strongest evidence. The exact pattern Run 1 surfaced as the highest-variance failure was eliminated structurally.

---

## Distribution comparison

| Grade | Run 1 (mode) | Run 2 (mode) | DB | Run 1 Δ vs DB | Run 2 Δ vs DB |
|---|---|---|---|---|---|
| SS | 0 | **2** | 4-8 (varies by sample) | −8 | −6 |
| S | 6 | 1 | 5-6 | +1 | −5 |
| A | 8 | **16** | 18-20 | −10 | **−4** |
| B | 5 | 8 | 8-9 | −3 | −1 |
| C | 22 | 16 | 21 | +1 | −5 |
| F | 19 | 17 | 0 | +19 | +17 |

**Key shifts:**
- **SS bucket reopened**: 0 → 2 in Run 2. Wide-funnel grad/intern roles (Palantir New Grad, Cloudflare Research Intern, Humanoid Internship) now reach SS where Run 1 hard-capped them at S.
- **A bucket doubled**: 8 → 16. The stack-concentration carveout + Concept-fit decomposition lift cleared-with-friction roles from C to A where the candidate has paradigm-match (Proton Rust, Keyrock, Elliptic AI Infra, GSR Trader carveout).
- **C bucket tightened**: 22 → 16. Fewer prestige-trap-as-C results; more split between F (hard floors) and A (carveout-eligible).
- **F bucket roughly held**: 19 → 17. Hard credential floors still go to F. This is the architectural win preserved.

---

## Per-job movement (48 jobs common to both runs)

- **Moved UP**: 12 jobs (the largest mover: Proton C→A, GSR C→A, SKL Robotics C→A, Trainline C→A, all driven by carveout or Concept-fit)
- **Held**: 31 jobs
- **Moved DOWN**: 5 jobs (Granola C→F, Cloudflare Deploy Intern A→B, Graphcore Driver S→A, B2C2 S→A, Checkout C→F)

**Notable up-shifts (carveout firing correctly):**

| Job | Run 1 mode | Run 2 mode | Mechanism |
|---|---|---|---|
| Proton Rust SWE | C | **A** | Stack-concentration carveout |
| GSR Systematic Trader | C | **A** | Carveout + Concept-fit (trading-systems) |
| SKL Robotics Internship | C | **A** | Wide-funnel intern + Concept-fit (ML/RL portfolio) |
| Trainline ML Gen AI | C | **A** | Concept-fit (LLM portfolio anchors Q3) |
| Stripe Expansion FS | F | **B** | "currently pursuing" boundary detection rule |
| Valarian Frontend | F | **C** | Boilerplate-vs-filter detection |

**Notable down-shifts (corrections):**

| Job | Run 1 mode | Run 2 mode | Reason |
|---|---|---|---|
| Granola AI Security | C | F | Sharpened to F via experience-floor reading |
| Checkout.com Analytics | C | F | Sharpened to F via 2+yr floor |
| B2C2 Grad Quant | S | A | Q1 cleared-with-friction not cleared-decisively per anchor table |
| Graphcore Drivers Grad | S | A | Q5 (no-sponsorship clause) cap |

---

## Per-job range improvements

| Job | Run 1 range | Run 2 range | Δ |
|---|---|---|---|
| **Proton Rust SWE** | **3** | **0** | **−3 (unanimous)** |
| Palantir New Grad | 3 | 1 | −2 |
| Keyrock Quant Intern | 3 | 1 | −2 |
| Elliptic AI Infra | 3 | 1 | −2 |
| B2C2 Grad Quant | 4 | 2 | −2 |

**15 jobs reduced their inter-agent variance. 12 jobs worsened.** Net improvement on per-job variance is small (+3), but the JOBS where improvement landed are exactly the high-variance pre-iteration cases (Proton, Palantir, Keyrock, Elliptic, B2C2) — these had been the most-disagreed jobs in Run 1 Section 2.

---

## Why trigger-case correction dropped — and why that's partly OK

Run 1: 12 of 17 trigger cases (71%) had 100% correction (every agent → C/F).  
Run 2: 6 of 21 trigger cases (29%) had 100% correction.

The metric drop has two distinct causes:

**1. Carveout intentionally lifts some trigger cases back to A** — this is by design.

Examples: Keyrock Rust Engineer Trading was tagged as a trigger case (narrow-funnel quant firm). Caner has 9 active Rust projects. The carveout fires; the role aggregates to A. This is the rubric's intended behaviour. The trigger-correction metric measures "did it stay at C/F" — which incorrectly counts these as "uncorrected".

If we filter to trigger cases where the carveout SHOULDN'T fire (no stack-concentration match, just selectivity), the correction rate should still be near-100%. The metric as-defined conflates "correctly demoted" with "correctly lifted via the carveout".

**2. New variance on prestige-trap edges** — this is genuine regression.

Examples that DID worsen:
- XTX Markets: Run 2 grades F, F, F, A, A (range 3, agreement 60%) — 2 agents inappropriately applied the carveout despite the 5-10yr hard floor. The rubric explicitly says the carveout cannot offset hard credential floors, but 2 agents misread the floor.
- Talos: Run 2 grades A, A, A, C, F, F (range 3, agreement 50%) — 3 agents lifted to A via "concept-fit on trading systems", 3 held at C/F per the 3+yr floor. The rubric should give consistent results here; it doesn't.

These two cases are the real regressions. The iteration introduced ambiguity at the carveout/hard-floor boundary that didn't exist before.

---

## The aggregation Step 3 anchor table — no measurable effect

Run 1 had 11 jobs in the "agents agree on Q1 verdict but disagree on letter" cell (aggregation rule fuzziness).
Run 2 has 11 jobs in the same cell — **identical count**.

The anchor table I added in this iteration did not measurably reduce aggregation fuzziness. Possible explanations:

1. **Agents aren't actually consulting the anchor table**. The table is in the rubric body but agents may be running the aggregation by judgement and citing the table post-hoc. The §Step 3 prose still says "judgement, not arithmetic" — the table may be reading as illustrative rather than load-bearing.
2. **The fuzziness is at a different layer**. Agents may agree on Q1 verdict, but disagree on Q2/Q3/Q4 *strength* — which the anchor table requires as input. The table can't disambiguate if the Q2-Q5 strength reading itself is variable.

This is the strongest evidence for the user's "maybe the issue is the skill file" hypothesis. If Q2-Q5 strength readings vary because the profile doesn't give sharp enough signals (paradigms, brand-tier definitions, work-shape preferences), no amount of aggregation-rule sharpening fixes downstream variance.

---

## Inter-agent agreement decomposition

| Pair-class | Run 1 % | Run 2 % | Δ |
|---|---|---|---|
| Letter+Q1 both agree | ~17% (10/60) | 15% (9/60) | −2pp |
| Letter agree, Q1 disagree | 0% | 2% (1/60) | +2pp |
| Letter disagree, Q1 agree | 18% (11/60) | 18% (11/60) | 0 |
| Letter disagree, Q1 disagree | 35% (21/60) | **65% (39/60)** | **+30pp** |
| Q1 missing/unparseable | 33% (20/60) | 0 | −33pp |

**Notable: Run 2 had zero Q1-missing jobs** — agents are now consistently producing the Q1-verdict tag in the format the parser expects. That's a side win from the iteration's emphasis on Q1 tagging in the prompt and the prompt's "Q1-verdict tag" requirement.

But the **Letter-disagree + Q1-disagree** cell ballooned from 35% to 65%. This is the highest-leverage finding: when agents disagree on the letter, they also disagree on the underlying Q1 verdict 65% of the time. The disagreement is upstream of the anchor table — at the Q1 reading itself.

This strongly suggests the next iteration target is **sharpening Q1 detection signals in the profile or rubric** — exactly the user's "maybe the issue is the skill file" hypothesis.

---

## What worked (the iteration's wins)

1. **Proton-class variance eliminated** — the canonical failure case unanimous now.
2. **Stack-concentration carveout magnitude table is effective** when applied correctly (Proton, Keyrock with matching stack).
3. **Concept-fit decomposition is being used** — agents now cite paradigm matches (low-latency, distributed systems, ML infra) separately from language matches. Visible in Trainline ML Gen AI moving C→A on Concept-fit grounds.
4. **Worked Examples reframing didn't introduce regressions** — agents continue to cite the §Step 3 anchor row alongside the example pattern, treating them as illustrative.
5. **Q1-verdict tagging is now 100% parseable** (up from 67% in Run 1).
6. **Distribution is more graduated** — 2 SS, 1 S, 16 A, 8 B, 16 C, 17 F vs Run 1's 0/6/8/5/22/19. The rubric now uses the full grade scale instead of collapsing toward C/F.

## What didn't work (the iteration's regressions)

1. **Carveout/hard-floor boundary is fuzzy** — XTX (5-10yr floor) graded F/F/F/A/A by 5 agents. The rubric explicitly forbids carveout offsetting hard credential floors, but agents misread the boundary on roles where the floor language is less explicit than "5+ years".
2. **Aggregation Step 3 fuzziness unchanged** — the new anchor table didn't measurably reduce the 11-job Q1-agree+letter-disagree count.
3. **Mean inter-agent agreement slipped slightly** (52.3% → 51.1%) — within noise but not the "flat-out improvement" the user wanted.
4. **Some Q1 disagreement increased** — 65% of letter-disagreed jobs also disagree on Q1 verdict (vs 35% in Run 1). The Q1 detection signals aren't sharp enough.

---

## Limitations

- **3 agents failed at 600s watchdog timeout**: cross-cluster-1, rubric-blind, anchor-injected. The two analytical baselines (rubric-blind and anchor-injected) lost — Sections 7 and 8 of the standard test report can't be computed for Run 2. The rubric-blind baseline gap means I can't measure whether the rubric is still doing distinguishable work vs unaided judgement post-iteration. The anchor-injection gap means I can't measure whether the worked-examples reframing reduced the +2 SS / −6.5 F anchor-bias from Run 1.
- **The trigger-case correction metric is partly miscalibrated** for the new rubric — the carveout is supposed to lift some trigger cases, but the metric counts that as "uncorrected". A future test should distinguish "carveout-eligible trigger" from "pure-selectivity trigger".
- **15 of 60 jobs are new this run** (the selection seeds by run-id timestamp). The 48-job overlap with Run 1 is the comparable subset.

---

## Recommendation: skill-file iteration is the next move

Per the user's framework: "if you can't see measurable improvement, maybe the issue is with the skill file. and maybe we should be putting more emphasis on the skill file."

The data supports this. The aggregation Step 3 anchor table didn't reduce aggregation fuzziness, suggesting the bottleneck is upstream — at the Q1 reading and the Q2-Q5 strength readings. Both are downstream of how rich the profile signals are.

The specific skill-file iteration target: **profile/skills.md §Concepts and Domains expansion + per-project paradigm tagging**. Currently §Concepts and Domains lists paradigm names. If each paradigm linked to the specific projects that demonstrate it (and ideally with depth-of-evidence ratings), agents would have less variance in reading paradigm-match strength.

This could be folded into a populate-from-lifeos iteration to produce a richer skills.md from LifeOS source content, with explicit paradigm-to-project mapping.

The carveout/hard-floor boundary fuzziness (XTX-class) is a separate grade-jobs iteration target — adding worked examples of borderline cases where the floor language is ambiguous would help. But this is a smaller move than the skill-file revision.

---

## Was this iteration worth it?

**Yes, narrowly.** The Proton case is now unanimous (the headline win). The carveout produced meaningfully different aggregation in 12 jobs across the sample. The Concept-fit decomposition is being applied. The distribution is more graduated.

**But it's not a flat-out improvement** per the user's standard. Some metrics regressed (trigger-correction rate, mean agreement), some metrics didn't move (aggregation Step 3 fuzziness count). The next iteration should either:
(a) Sharpen the carveout/hard-floor boundary in grade-jobs (smaller move), OR
(b) Iterate the skill-file via populate-from-lifeos to give agents richer Q1 / Q2-Q5 strength signals (larger move per user's hypothesis).

The user's instruction was clear: try grade-jobs alone first, then move to the skill file if iteration there doesn't produce measurable improvement on all metrics. This run's evidence supports moving to skill-file iteration next.
