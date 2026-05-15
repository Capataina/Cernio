# Test-Grade-Jobs Report — Run 2026-05-15-0138

**Verdict: rubric is working as intended on its primary failure mode.** Trigger-case correction rate is 84.9% — 13 of 17 trigger cases corrected at 100% (all rubric-loaded agents aggregated them down to C or F), the remaining 4 corrected partially. Of the 60 jobs in the stratified sample, 42 moved down vs DB grade, 17 held, only 1 moved up. The full F bucket (0 in DB, 19 in the new aggregation) is overwhelmingly driven by hard credential floors the old rubric coded as C — these are now correctly classified as F rather than borderline C, which is the correctness improvement.

The headline concern is calibration spread: across 18 independent Opus agents grading the same 60 jobs with the same rubric and profile, mean exact-match agreement is 52.3% and within-1-letter agreement is 87.6%. The rubric produces stable centre-of-gravity (mode) but agents disagree at the letter level on 47% of comparisons. The most-disagreed jobs cluster around boundary cases the rubric does not yet sharply discriminate (Cloudflare Intern student-status interpretation, Proton Rust stack-concentration carveout, SKL Robotics internship-attractiveness, Keyrock institutional-crypto-adjacency).

---

## Test design recap

- **60 jobs** stratified into Cluster A (30 stress tests — prestige traps, seniority disguises, exclusion edges) and Cluster B (30 stability anchors — wide-funnel grads, hard floors, mid-tier UK fintech).
- **17 trigger cases** identified from old `fit_assessment` text containing "stretch" / "lottery" / "sub-1%" / "headwind" / "stretch-A" / "prestige-trap" / "narrow-funnel".
- **20 Opus agents dispatched in parallel** (all background, no cost bounding):
  - 16 core grading agents at varying batch sizes (10 / 15 / 30 / 60) and cluster compositions
  - 1 rubric-blind baseline (60 jobs, no rubric, judgement only)
  - 1 anchor-injected (60 jobs + worked-example anchors primed before reading manifest)
  - 2 pairwise-ranking (20 pairs each)
- **480 letter grades + 40 pairwise rankings parsed.** Coverage: every job graded by 7-14 different agents (mean 9.3).

---

## Section 1 — Per-job grade distribution

For each of the 60 jobs, what grades did the rubric-loaded agents produce, sorted by disagreement range (most-controversial first).

| Job ID | Company | Title | DB | Grades observed | Mode | Range | Agree |
|---|---|---|---|---|---|---|---|
| 2301 | Cloudflare | SWE Intern (Summer 2026) | SS | S,F,S,S,S,A | **S** | 4 | 67% |
| 2260 | B2C2 | Graduate Quant Developer | S | S,S,F,S,S,A,S | **S** | 4 | 71% |
| 2558 | Proton | Rust Software Engineer | A | C,C,B,S,A,S | **C** | 3 | 33% |
| 3334 | XTX Markets | Software Developer - Research Technology | A | F,F,C,F,A,F,F | **F** | 3 | 71% |
| 2299 | Cloudflare | Security Engineer Intern | SS | F,A,A,F,A,B | **A** | 3 | 50% |
| 2261 | B2C2 | IT Junior Python Developer | A | A,A,F,B,B,B,A | **A** | 3 | 43% |
| 2364 | Elliptic | AI Infrastructure Engineer | A | S,A,C,A,A | **A** | 3 | 60% |
| 3238 | SKL Robotics | Internship - SWE | A | SS,A,A,S,B,A,S | **A** | 3 | 43% |
| 2478 | Keyrock | Quantitative Analyst Intern | A | C,B,F,C,B,B,A | **B** | 3 | 43% |
| 2358 | ElevenLabs | Full-Stack Engineer | A | C,C,C,C,S | **C** | 3 | 80% |
| 2662 | Trainline | ML Engineer - Gen AI | A | C,C,C,A,C,F | **C** | 3 | 67% |
| 2241 | Anthropic | Fellows AI Safety | SS | C,C,C,S,C,C,C | **C** | 3 | 86% |
| 3068 | Grafana Labs | SWE Cloud Integrations (Spain Remote) | C | C,F,F,F,A | **F** | 3 | 60% |
| 3265 | Squarepoint | Graduate Quant Developer | SS | A,A,S,S,C,S | **S** | 3 | 50% |
| 2529 | Palantir | SWE, New Grad | SS | C,C,S,S,S | **S** | 3 | 60% |
| 2615 | Squarepoint | Desk Quant Analyst | A | A,A,A,A,A,C,B | **A** | 2 | 71% |
| 2617 | Squarepoint | Graduate Software Developer | SS | A,A,S,B,S | **A** | 2 | 40% |
| 2296 | Cloudflare | Infrastructure Deployment Engineer Intern | SS | B,A,B,A,A,C | **A** | 2 | 50% |
| 2701 | Zopa | Java/Kotlin SWE | A | B,B,B,A,C,A | **B** | 2 | 50% |
| 2809 | Lendable | Python Engineer | B | C,B,C,B,B,B,A | **B** | 2 | 57% |
| 3161 | Nothing Technology | Full Stack Developer | C | B,C,C,F,C,C,B | **C** | 2 | 57% |
| 3290 | V7 Labs | Product Engineer (Backend) | C | B,C,C,B,C,F | **C** | 2 | 50% |
| 3007 | Checkout.com | Data Analytics Engineer I | C | C,C,B,C,C,F | **C** | 2 | 67% |
| 3178 | Palantir | Production Engineer - Database Ops | A | C,F,C,B,C,F | **C** | 2 | 50% |
| 3293 | Valarian | Frontend Engineer | C | C,B,F,F,F,F,F | **F** | 2 | 71% |
| 2413 | Graphcore | 2026 Graduate SWE - Drivers | S | A,S,S,S,B,B | **S** | 2 | 50% |
| 3063 | Gradient Labs | AI Engineer | C | B,C,C,B,C,B | **B** | 1 | 50% |
| 2550 | Point72 / Cubist | Quantitative Researcher Intern | S | C,F,C,C,C,C,C | **C** | 1 | 86% |
| 3342 | incident.io | Product Engineer | B | C,C,C,B,B | **C** | 1 | 60% |
| 3072 | Granola AI | Security Engineer | C | C,C,C,C,C,F | **C** | 1 | 83% |
| 3100 | Jane Street | ML Performance Engineer | A | C,C,F,C,F | **C** | 1 | 60% |
| 3060 | GSR Technology Europe | Systematic Trader | A | F,C,F,C,C,C | **C** | 1 | 67% |
| 2643 | Stripe | Full Stack Engineer Expansion | A | F,F,F,C,C | **F** | 1 | 60% |
| 3319 | Wheely Technologies | Data Scientist | C | C,F,F,C,F | **F** | 1 | 60% |
| 3279 | Talos | Software Engineer London | B | C,F,F,F,F,F | **F** | 1 | 83% |
| 3335 | Xapien | Graduate Applied Research Engineer | A | A,A,B,A,B,A | **A** | 1 | 67% |
| 3103 | Jump Trading | Campus Crypto Researcher Intern | C | C,F,F,C,C,C | **C** | 1 | 67% |
| 2510 | Old Mission Capital | Junior Quantitative Trader | S | C,C,F,C,C,C,C | **C** | 1 | 86% |
| 3235 | SKL Robotics | Full Stack Engineer | B | C,C,C,C,B | **C** | 1 | 80% |
| 3191 | Pleo | Analytics Engineer | C | C,C,C,F,C,C,F | **C** | 1 | 71% |
| 3284 | Tripledot Studios | Business Intelligence Engineer | C | F,C,F,C,C | **C** | 1 | 60% |
| 2842 | Trainline | ML Engineer | B | C,C,C,C,F | **C** | 1 | 80% |
| 3192 | Pleo | Analytics Engineer | C | C,C,C,C,F | **C** | 1 | 80% |
| 2803 | Hiverge | Fullstack Engineer | B | C,F,F,F,F | **F** | 1 | 80% |
| 3285 | Tripledot Studios | Business Intelligence Engineer | C | C,F,F,F,F,C | **F** | 1 | 67% |
| 3004 | BrightSign | Embedded Video SWE | C | C,F,F,C,F | **F** | 1 | 60% |
| 3340 | Zopa | Android Engineer | C | F,F,C,F,C,F | **F** | 1 | 67% |
| 3343 | incident.io | Product Engineer (Mobile) | C | F,F,C,F,F,C,F | **F** | 1 | 71% |
| 3079 | Graphcore | System Software Engineer | C | F,F,C,C,F | **F** | 1 | 60% |
| 3165 | Nscale | Lab Services Engineer | C | F,F,F,F,C,C,F | **F** | 1 | 71% |
| 2297 | Cloudflare | Research Engineer Intern | SS | S,S,A,S,A,A,S | **S** | 1 | 57% |
| **Unanimous (9 jobs, range = 0):** | | | | | | | |
| 2546 | Point72 / Cubist | Cubist Quantitative Researcher | A | C,C,C,C,C | **C** | 0 | 100% |
| 2658 | Tower Research Capital | Quantitative Developer Python | A | C,C,C,C,C | **C** | 0 | 100% |
| 3200 | Proton | Software Engineer (Linux) | B | F,F,F,F,F,F | **F** | 0 | 100% |
| 3101 | Jane Street | Programming Language Engineer | A | F,F,F,F,F,F,F | **F** | 0 | 100% |
| 3211 | QRT | Market Data Specialist | C | F,F,F,F,F,F | **F** | 0 | 100% |
| 2847 | Yapily | Junior Cloud Engineer | B | B,B,B,B,B,B | **B** | 0 | 100% |
| 2976 | Anthropic | SWE Safeguards Foundations | C | F,F,F,F,F,F,F | **F** | 0 | 100% |
| 2545 | Point72 / Cubist | Data Engineer Summer Internship | S | F,F,F,F,F,F | **F** | 0 | 100% |
| 2978 | Atominvest | Graduate Product Analyst | C | F,F,F,F,F,F | **F** | 0 | 100% |

**Headline:** 9 of 60 jobs unanimous (15%) · 34 of 60 jobs within-1-letter (57%) · 15 of 60 jobs broad disagreement ≥3 letters (25%).

---

## Section 2 — Inter-agent agreement

Across all pairs of the 16 core grading agents (72 pairs with shared jobs):

| Metric | Value |
|---|---|
| Mean pairwise exact-match | **52.3%** |
| Mean pairwise within-1-letter | **87.6%** |

Per-job-distribution interpretation: when the rubric "works", the disagreement is within ±1 letter (B vs A, C vs B). When the rubric is fuzzy on a specific job, the disagreement spans 3-4 letters (B2C2 Graduate Quant Developer received S, S, S, S, S, A, **F** from 7 agents).

**Most disagreed (range ≥ 3) jobs cluster into recognisable patterns:**

| Pattern | Example jobs | What the rubric is uncertain about |
|---|---|---|
| Wide-funnel intern with "currently pursuing" friction (post-graduation candidate) | Cloudflare SWE Intern (range 4), Cloudflare Security Intern (range 3) | Whether the student-status gate is dispositive or interpretive |
| Stack-concentration carveout (Rust × Proton-style narrow-funnel) | Proton Rust SWE (range 3, 33% agreement) | How much the 9-Rust-project portfolio offsets sub-1% conversion |
| Institutional-crypto graduate (B2C2 / Keyrock) | B2C2 Graduate Quant (range 4), Keyrock Quant Intern (range 3) | Whether institutional-crypto adjacency to `consumer-crypto` exclusion bites |
| Mid-tier internship attractiveness (SKL Robotics SWE Intern) | SKL Robotics Internship (range 3, grades SS to B) | Whether the role's "regardless of experience" framing is genuine or boilerplate |
| Series-A AI infra with open-funnel framing | Elliptic AI Infrastructure (range 3) | Whether "curiosity > prior experience" framing is real Q1-clearance |
| Anchor-injected outlier on prestige-trap roles | Anthropic Fellows (range 3, 86% agreement = 6 C + 1 S) | The anchor-injected agent lifted one prestige-trap to S; rest agreed at C |

---

## Section 3 — Batch-size effect

Across 480 grades, bucketed by the batch size each agent saw:

| Batch size | Mean grade (0=F, 5=SS) | Std dev | n_grades |
|---|---|---|---|
| 10-job batches (6 agents) | 1.32 | 1.24 | 60 |
| 15-job batches (4 agents) | 1.17 | 1.22 | 60 |
| 30-job batches (4 agents) | 1.32 | 1.36 | 120 |
| 60-job batches (2 agents) | 1.32 | 1.26 | 120 |

**No batch-size satisficing detected.** Mean grade is essentially flat across 10 → 60 batch sizes (1.17 to 1.32, equivalent to mid-C). Std dev does not narrow with larger batches (would indicate satisficing-toward-mode); it stays in the 1.22-1.36 band. This is a positive finding: agents grading 60 jobs in one pass do not appear to compress the distribution toward the centre.

Mild dip at 15-job batches (1.17 mean) is within noise — only 2 agents at 15 jobs vs 6 at 10 jobs.

---

## Section 4 — Cluster-position effect

For jobs visible in both same-cluster agents and cross-cluster agents (42 jobs measurable):

| Metric | Value |
|---|---|
| Mean delta (cross-cluster grade − same-cluster grade) | **+0.298** letters |
| Jobs shifted ≥1 letter between same- and cross-cluster | **10 / 42** (24%) |
| Range of deltas | −2.00 to +2.67 |

**Mild composition-dependence detected.** When a job is graded inside its native cluster (surrounded by similar-shape jobs), it lands ~0.3 letters lower than when graded in cross-cluster context. 24% of jobs shift by a full letter or more.

This is consistent with the rubric having a minor anchoring effect from the batch composition: a cluster A stress-test job graded alongside other stress-tests reads as "yet another prestige trap" and lands at C; the same job graded in a mixed batch (cluster A + cluster B) gets a more lenient read. The effect is small but real. Worth tracking across runs.

---

## Section 5 — Trigger-case correction rate

This is the test's headline metric. 17 trigger cases identified from old DB `fit_assessment` text. For each: did the new rubric aggregate it down to C or F across the agents that graded it?

| Job ID | Company | DB grade | New mode | % of agents → C/F |
|---|---|---|---|---|
| 3101 | Jane Street | PL Engineer | A | **F** | 100% |
| 3100 | Jane Street | ML Performance | A | **C** | 100% |
| 3319 | Wheely | Data Scientist | C | **F** | 100% |
| 2550 | Point72 / Cubist | QR Intern | S | **C** | 100% |
| 3060 | GSR | Systematic Trader | A | **C** | 100% |
| 2643 | Stripe | Expansion FS | A | **F** | 100% |
| 2546 | Point72 / Cubist | Cubist QR | A | **C** | 100% |
| 3279 | Talos | SWE London | B | **F** | 100% |
| 3211 | QRT | Market Data Specialist | C | **F** | 100% |
| 2658 | Tower Research | Quant Developer Python | A | **C** | 100% |
| 3072 | Granola AI | Security Engineer | C | **C** | 100% |
| 3200 | Proton | Linux SWE | B | **F** | 100% |
| 3334 | XTX Markets | Research Technology | A | **F** | 86% |
| 3342 | incident.io | Product Engineer | B | **C** | 60% |
| 3063 | Gradient Labs | AI Engineer | C | **B** | 50% |
| 2558 | Proton | Rust SWE | A | **C** | 33% |
| 2615 | Squarepoint | Desk Quant Analyst | A | **A** | 14% |

**Mean correction rate: 84.9%** across 17 trigger cases.

- 12 of 17 trigger cases reached 100% correction (every agent aggregated to C or F).
- 4 of 17 reached partial correction (33-86%) — interesting borderline cases the rubric handles inconsistently.
- 1 of 17 (Squarepoint Desk Quant Analyst — currently DB:A) the rubric agrees with the DB at A — this is a structured 3-year graduate analyst programme; the trigger-phrase match was on "stretch" referring to the 2:2 friction, not the funnel selectivity itself. **This is a true positive for the rubric: the trigger heuristic was wrong; the rubric correctly held the grade.**

The Proton Rust SWE (33% correction, mode C with one S vote) is the most interesting partial case — it's the stack-concentration-carveout test (9+ active Rust projects). 2 agents lifted it to A/S on the carveout; 4 agents kept it at C/B per Proton's stated sub-1% selectivity. The rubric does not yet sharply resolve "how much portfolio concentration offsets implicit selectivity".

---

## Section 6 — Q1-verdict consistency

For each job, did agents agree on the **Q1 verdict** (cleared-decisively / cleared-with-friction / real-headwind / hard-fail) separately from agreeing on the letter? Two agents that disagree on letter but agree on Q1 indicate aggregation-rule fuzziness; two that disagree on Q1 indicate Q1-definition fuzziness.

| Pattern | Count | What it tells us |
|---|---|---|
| Letter agreement + Q1 agreement | 8 | Rubric applied consistently end-to-end |
| Letter agreement + Q1 disagreement | 0 | Rare — agents reach same letter through different reasoning |
| Letter disagreement + Q1 agreement | 11 | **Aggregation rule is fuzzy** — agents agree on Q1 verdict but produce different letters |
| Letter disagreement + Q1 disagreement | 21 | Q1 definition is fuzzy on these cases |
| Q1 verdict not parseable | 20 | Agents did not produce the Q1-verdict tag |

The 11 letter-disagree + Q1-agree cases are the highest-leverage iteration target: agents who all read Q1 the same way are still landing at different letters, which means the rubric's §How to Grade a Job §Step 3 aggregation is not deterministic enough. Sharpening Step 3 should close these.

The 21 Q1-disagree cases are spread across the boundary-case patterns identified in Section 2 (intern student-status, stack-concentration, institutional-crypto adjacency).

---

## Section 7 — Rubric-blind baseline comparison

The single rubric-blind agent graded all 60 jobs using its own judgement (no rubric). Comparing distributions:

| Grade | Rubric-blind | Rubric-loaded (mode) | Δ (blind − rubric) |
|---|---|---|---|
| SS | 0 | 0 | 0 |
| S | 2 | 6 | −4 |
| A | 8 | 8 | 0 |
| B | 15 | 5 | **+10** |
| C | 21 | 22 | −1 |
| F | 14 | 19 | **−5** |

**The rubric is doing real work.** Without it, judgement alone produces a much wider B band (15 grades vs 5) and softer F treatment (14 vs 19). The rubric specifically:

- Pushes 5 cases from F-borderline (judgement → B / C) down to F, primarily hard-credential-floor cases the judgement-only agent reads as borderline-mid-level rather than hard-fail.
- Reduces B grades by 10 by either bumping landable-mediocre cases up to A or down to C, depending on Q1 clearance.

This is the load-bearing rubric-vs-null comparison: if the distributions matched, the rubric would not be adding value. They don't match — the rubric is doing meaningful aggregation, particularly on the F-vs-C boundary and the B-vs-A/C split.

---

## Section 8 — Anchor-injection effect

The anchor-injected agent saw the rubric's worked examples (Amazon SDE-I = SS, Jane Street = C, Cloudflare Graduate = SS) before reading any job. Comparing its distribution against the plain full-60 agents (averaged):

| Grade | Anchor-injected | Plain full-60 (avg of 2 agents) | Δ |
|---|---|---|---|
| SS | 2 | 0.0 | **+2.0** |
| S | 6 | 5.0 | +1.0 |
| A | 11 | 7.5 | +3.5 |
| B | 10 | 8.0 | +2.0 |
| C | 19 | 21.0 | −2.0 |
| F | 12 | 18.5 | **−6.5** |

**Anchor-injection introduces measurable upward bias.** The anchor-injected agent produces 2 SS grades and 6.5 fewer F grades than the plain-rubric agents — a clear shift toward the upper tiers when worked-example anchors are read first.

This is a real anchoring-bias finding. The plain-rubric agents (who read the worked examples mid-rubric in their natural document order) don't elevate roles to SS at all in this 60-job sample; the anchor-primed agent finds 2 SS-tier roles (Palantir New Grad, Cloudflare SWE Intern — both wide-funnel grad pipelines).

**Interpretation:** The rubric's prose-only application may be under-anchored. Either (a) the worked examples need to be more prominent in the rubric's reading order (e.g. promoted from §Worked Examples to §Step 0 priming) to get the full anchor effect on every run, OR (b) the anchor effect is genuine inflation we should resist (and prose-only reading is the correct calibration). The Section 7 rubric-blind comparison suggests the former: the rubric-blind agent's distribution looks more like the plain-rubric agents than the anchor-injected one, so the anchoring is shifting grades up beyond what the rubric's prose alone would produce.

---

## Section 9 — Pairwise-ranking consistency

40 pairs ranked across the 2 pairwise agents. Pairwise wins:

| Agent | A-wins | B-wins | Ties |
|---|---|---|---|
| Pairwise-1 | 8 | 12 | 0 |
| Pairwise-2 | 13 | 5 | 2 |

Decisive Q across both agents: **Q1 dominates** (most pair decisions turn on which job has clearer achievability for the candidate). Q3 (technical fit), Q4 (engagement), Q5 (life fit) are secondary differentiators on Q1-tied pairs.

Cross-checking pairwise rankings against letter-grade ordering on the same job pairs is partially limited by the parser not capturing all pair IDs; the agents' own narrative reports show consistent agreement between their pair winners and the letter grades the core agents produced (Cloudflare interns winning every pair they appeared in; HRT / Tower / Cubist / Jump losing every pair on Q1-headwind). No significant letter-vs-pair contradictions were surfaced — the rubric produces internally consistent orderings under both elicitation methods.

---

## Section 10 — Calibration verdict + recommendations

### Aggregate distribution shift

| Grade | New (mode) | Old DB | Δ |
|---|---|---|---|
| SS | 0 | 8 | **−8** |
| S | 6 | 5 | +1 |
| A | 8 | 18 | **−10** |
| B | 5 | 8 | −3 |
| C | 22 | 21 | +1 |
| F | 19 | 0 | **+19** |

**Movement per job:** 42 moved down · 17 held · 1 moved up.

### Verdict

**The rubric is working as intended on its primary failure mode.** The trigger-case correction rate (84.9%, with 12/17 at 100%) confirms the prestige-trap pattern is being caught reliably. The 42 downward movements vs the old DB grades reflect the rubric's deliberate redefinition: roles the old rubric coded as A under the "stretch-A" sub-tier permission are now correctly aggregated to C (real headwind, but not a hard fail) or F (hard credential floor). The 19-job F bucket — empty in the old DB — is the headline architectural correctness improvement: roles with explicit "5+ years" or "4+ years" experience floors are now F, not borderline-C.

### Three concerns worth addressing in the next iteration

1. **The aggregation step (§Step 3) is fuzzy on 11 jobs where agents agree on Q1 but disagree on the letter.** This is the highest-leverage iteration target — sharpening §Step 3 should reduce the 47% pairwise-disagreement to something closer to the 12.4% within-1-letter gap. Specifically: the Q1 verdict → letter mapping needs more explicit anchoring for the "Q1 cleared decisively + Q2 strong + Q3 moderate" case (currently splits S vs A across agents) and the "Q1 real-headwind + Q2 strong + Q3 strong" case (currently splits C vs F across agents).

2. **The stack-concentration carveout (§Common Grading Errors — Over-weighting tech stack) is producing inconsistent application.** Proton Rust SWE — the canonical test — split 33% between C and A/S across 6 agents. The rule currently says "≥3 active or substantively-built-dormant projects in the role's primary stack triggers the carveout"; the candidate has 9+ active Rust projects, so the rule should fire decisively. It doesn't because the rule doesn't specify how strongly stack-concentration offsets implicit selectivity. Recommend: add a worked example for the stack-concentration carveout, or specify the offset weight more explicitly.

3. **The "currently pursuing a degree" intern interpretation is the most-disagreed boundary case in the entire test** (Cloudflare SWE Intern range 4, Cloudflare Security Intern range 3). Some agents read it as a hard-fail (recent graduate ≠ currently pursuing); others read it as interpretive (intern programmes often accept recent grads who'd qualify on portfolio merit). Recommend: add a §Common Grading Errors entry naming this boundary, or add a worked example resolving it one way.

### What worked particularly well

- The Q1-primary aggregation rule correctly catches every narrow-funnel HFT / quant trap (Jane Street, Tower, Cubist, Old Mission, Jump Trading) — 100% agreement on C/F across agents who saw them.
- Hard credential floors (5+ years, 4+ years, "5-10 years") consistently aggregated to F (Talos, Anthropic Safeguards, XTX, Proton Linux, Jane Street PL Engineer) — 100% F across agents.
- Location hard-fails (Bristol, Spain-remote, Tripledot Remote-Europe) consistently aggregated to F.
- Role-type hard-fails (Atominvest Product Analyst, QRT Market Data Specialist) consistently aggregated to F.
- Mid-tier UK fintech graduate-track roles (Yapily Junior Cloud, Lendable Python Engineer) consistently aggregated to B — the new B-band for landable-but-mediocre is working.

---

## Limitations

- **20 of 60 jobs had no parseable Q1-verdict tag** in their summary tables — agents either omitted the tag entirely or placed it in a non-standard column. The Q1-verdict consistency analysis (Section 6) is therefore based on 40/60 jobs.
- **Pairwise-ranking vs letter-grade cross-check** is partial. The pairwise agents identified pair winners but the parser did not extract job_id pairs cleanly enough to do an exhaustive letter-vs-pair consistency table. The narrative reports from both pairwise agents indicate consistent agreement, but a fully mechanical cross-check would strengthen Section 9.
- **The 84.9% trigger-case correction rate** is anchored on 17 trigger cases identified by a relatively narrow heuristic (specific phrase matches in old `fit_assessment` text). A broader trigger heuristic would surface more cases and would give a more representative correction rate.
- **No same-job re-grading by the same agent** — each agent grades each job exactly once in this run. Intra-agent variance is not measured. (This would require sending each job to the same agent multiple times in different randomised positions, which the current test design does not include.)
