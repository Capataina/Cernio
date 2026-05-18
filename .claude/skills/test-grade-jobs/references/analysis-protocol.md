# Analysis Protocol

## Table of Contents

- [Purpose](#purpose)
- [Inputs](#inputs)
- [The seven structural axes](#the-seven-structural-axes)
- [Axis A — Format adherence](#axis-a--format-adherence)
- [Axis B — Reasoning specificity](#axis-b--reasoning-specificity)
- [Axis C — Q3a/Q3b differentiation](#axis-c--q3aq3b-differentiation)
- [Axis D — Internal consistency](#axis-d--internal-consistency)
- [Axis E — Inter-agent variance](#axis-e--inter-agent-variance)
- [Axis F — Pairwise consistency](#axis-f--pairwise-consistency)
- [Axis G — Risk acknowledgment](#axis-g--risk-acknowledgment)
- [Cross-run regression diff](#cross-run-regression-diff)
- [Q1 prose inference (no agent-emitted tags)](#q1-prose-inference-no-agent-emitted-tags)
- [Report file audience](#report-file-audience)
- [Anti-Patterns](#anti-patterns)

---

## Purpose

The analysis phase converts the per-agent markdown outputs into a multi-axis report. This reference describes what each axis measures, how to compute it, and what the section's output looks like.

The seven axes measure grade-jobs' own structural properties: does the output follow grade-jobs' rubric format, is the reasoning specific or generic, do the rubric's structural splits (Q3a vs Q3b) produce distinct content, is the output internally coherent, how stable are grades across agents, how stable is relative ranking, how often does the rubric acknowledge and weigh risks. None of these compare grade-jobs against any external "correct answer." When agents disagree, the disagreement is measured and reported; the test does not adjudicate which agent is right.

`scripts/analyse.py` does the parsing and per-axis scoring. The agent writes the report by reading the script's intermediate JSONs and composing the narrative + tables. The script does NOT write the report itself — the report needs judgement on framing, which a script can't supply. But the metrics themselves come from the script verbatim; the agent does not invent numbers.

---

## Inputs

After Phase 3 completes, the analysis script has access to:

| Input | Path | Contents |
|---|---|---|
| Per-agent outputs | `/tmp/test-grade-jobs-<run-id>/agent-*.md` | 13 markdown files, one per agent |
| Coverage matrix | `/tmp/test-grade-jobs-<run-id>/coverage-matrix.json` | job_id → list of agent IDs that graded it |
| Cluster A jobs | `/tmp/test-grade-jobs-<run-id>/cluster-a.json` | 30 job IDs |
| Cluster B jobs | `/tmp/test-grade-jobs-<run-id>/cluster-b.json` | 30 job IDs |
| All jobs | `/tmp/test-grade-jobs-<run-id>/jobs-all.json` | 60 full job records |
| Prior baseline | `context/test-runs/baseline.json` (if present) | per-axis scores from the prior run |

The script parses every per-agent output into structured form per the rubric's Q-slots:

```json
{
  "agent_id": "cluster-a-30job",
  "agent_role": "core-grading",
  "batch_size": 30,
  "cluster_scope": "a-only",
  "assessments": [
    {
      "job_id": 2447,
      "grade": "C",
      "evidence_basis": "jd",
      "q1_prose": "...",
      "q2_prose": "...",
      "q3a_prose": "...",
      "q3b_prose": "...",
      "q4_prose": "...",
      "q5_prose": "...",
      "verdict_prose": "...",
      "q1_inferred_verdict": "real-headwind",
      "jd_quotes_in_q1": ["3+ years"],
      "jd_quotes_in_q3a": ["Go primary"],
      "project_anchors_in_q3a_q3b": ["nyquestro.md", "cernio.md"]
    },
    ...
  ]
}
```

These structured records feed every axis.

---

## The seven structural axes

Each axis produces a numeric score (0-100, higher = better adherence to the rubric's own rules) AND a per-agent or per-job table of evidence. The aggregate report shows headline scores plus the evidence tables.

| # | Axis | Headline metric | What it measures |
|---|---|---|---|
| A | Format adherence | mean per-agent format-compliance score | Does the output follow the rubric's slot structure? |
| B | Reasoning specificity | mean per-agent specificity score | Citation density vs generic-phrase rate |
| C | Q3a/Q3b differentiation | mean per-agent Q3a/Q3b distinctness | Does the rubric's split produce distinct content? |
| D | Internal consistency | mean per-agent coherence score | Does Verdict prose support Grade letter? Q-slot reasoning consistent? |
| E | Inter-agent variance | exact-letter agreement %, within-1 % | How stable are grades across agents on shared jobs? |
| F | Pairwise consistency | cross-agent pair agreement %, transitivity rate | How stable is relative ranking across agents? |
| G | Risk acknowledgment | % of assessments with named risks, risk-direction correlation | Does Q3b name and weigh risks? |

Each section in the report names the axis, gives the headline metric, provides per-agent or per-job evidence tables, and reports the regression diff from the prior run if a baseline exists.

---

## Axis A — Format adherence

For every per-agent assessment, check structural compliance with grade-jobs' rubric:

```python
# pseudocode
checks = {
    "all_seven_slots_present": all(slot in asmt for slot in ["q1","q2","q3a","q3b","q4","q5","verdict"]),
    "q1_has_jd_quote": asmt.evidence_basis != "jd" or has_quoted_substring(asmt.q1_prose, asmt.jd_text),
    "q3a_has_jd_quote": asmt.evidence_basis != "jd" or has_quoted_substring(asmt.q3a_prose, asmt.jd_text),
    "named_project_in_q3a_or_q3b": any(proj in (asmt.q3a_prose + asmt.q3b_prose) for proj in profile_project_filenames),
    "grade_letter_on_own_line": re.search(r"^Grade:\s*[SAFBC]{1,2}\s*$", asmt.raw, re.M),
    "no_banned_strings_in_slots": not any(s in slot_text for s in BANNED for slot_text in [asmt.q1_prose, ...]),
    "evidence_basis_set": asmt.evidence_basis in ("jd", "semantic", "insufficient"),
}
per_agent_compliance = mean(sum(check.values())/len(check) for asmt in agent.assessments)
```

Banned-string set (from grade-jobs' no-verdict-enums rule):
`"cleared-decisively", "cleared-with-friction", "real-headwind", "hard-fail", "Q1 cleared", "Q2 strong", "Q3 moderate", "Q3 weak", "→ A", "→ B", "→ C", "→ S", "→ F"`. These are banned inside Q-slot prose; the rubric explicitly bans them. (The Q1-inferred-verdict label is extracted separately from prose; see §Q1 prose inference.)

Per-slot length distributions (mean, p25, p75) — informational, not a pass/fail.

**Output shape (per-agent rows):**

| Agent | All 7 slots | Q1 JD-quote | Q3a JD-quote | Project anchor | Grade-line | No-banned | evidence_basis | Score |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cluster-a-10job-1 | 100% | 95% | 87% | 78% | 100% | 100% | 100% | 94 |
| cluster-a-15job-1 | 100% | 100% | 90% | 85% | 100% | 100% | 100% | 96 |
| ... | | | | | | | | |

Headline axis score: mean per-agent compliance %.

---

## Axis B — Reasoning specificity

For every assessment, measure citation density and generic-phrase rate:

```python
GENERIC_PHRASES = [
    "good company", "decent fit", "relevant experience", "strong tech stack",
    "great role", "good role", "good fit", "decent role", "solid choice",
    "broadly relevant", "interesting role"
]
# Specific reference count = JD substrings quoted + named profile projects + named skills.md entries cited
specific_refs = count_jd_substrings(asmt) + count_named_projects(asmt) + count_skill_entries(asmt)
total_words = wordcount(asmt.full_text)
specificity_density = specific_refs / total_words * 100
generic_count = sum(asmt.full_text.lower().count(p) for p in GENERIC_PHRASES)
project_diversity = len(set(named_projects_across_agent_assessments))
```

**Output shape (per-agent rows):**

| Agent | Generic phrases / assmt | Specific refs / 100 words | Project diversity | Score |
|---|---:|---:|---:|---:|
| cluster-a-10job-1 | 0.3 | 2.1 | 8 | 88 |
| ... | | | | |

Score formula: weighted (specificity_density × 0.6) + (project_diversity_normalised × 0.3) + (1 / (1 + generic_count) × 0.1) × 100.

Headline axis score: mean per-agent specificity score.

---

## Axis C — Q3a/Q3b differentiation

For every assessment with both Q3a and Q3b populated, measure how distinct the two slots are:

```python
def word_set(text):
    return set(w.lower() for w in re.findall(r"\b\w{4,}\b", text) if w.lower() not in STOPWORDS)

def jaccard(a, b):
    return len(a & b) / len(a | b) if (a | b) else 0.0

q3a_words = word_set(asmt.q3a_prose)
q3b_words = word_set(asmt.q3b_prose)
overlap_ratio = jaccard(q3a_words, q3b_words)
# lower overlap = more distinct = better

CAREER_AXIS_TERMS = ["on-axis", "adjacent", "off-axis", "trajectory", "career", "specialism", "build toward", "engineer caner is becoming"]
q3b_has_career_axis_term = any(t in asmt.q3b_prose.lower() for t in CAREER_AXIS_TERMS)
q3a_has_jd_tech_quote = bool(asmt.jd_quotes_in_q3a)
```

**Output shape (per-agent rows):**

| Agent | Q3a/Q3b mean overlap | Q3b career-axis term % | Q3a JD-tech quote % | Score |
|---|---:|---:|---:|---:|
| cluster-a-10job-1 | 0.12 | 87% | 95% | 91 |
| ... | | | | |

Score formula: `(1 - mean_overlap) × 0.4 + q3b_career_axis_term_pct × 0.3 + q3a_jd_tech_quote_pct × 0.3 × 100`. Target: low overlap (distinct content), high career-axis term presence in Q3b, high JD-quote presence in Q3a.

Headline axis score: mean per-agent Q3a/Q3b differentiation score.

---

## Axis D — Internal consistency

For every assessment, check whether the Q-slot reasoning supports the Grade letter and Verdict prose.

Three sub-checks:

```python
# 1. Verdict ↔ Grade alignment
VERDICT_POSITIVE_WORDS = ["axis bet", "career launch", "make the cut", "yes if there's room", "compelling"]
VERDICT_NEGATIVE_WORDS = ["does not make the cut", "skip", "deadweight", "not worth", "no"]
verdict_sentiment = "positive" if any(p in asmt.verdict_prose.lower() for p in VERDICT_POSITIVE_WORDS) else \
                   "negative" if any(n in asmt.verdict_prose.lower() for n in VERDICT_NEGATIVE_WORDS) else "neutral"
grade_position = {"SS": "high", "S": "high", "A": "high", "B": "mid", "C": "low", "F": "low"}[asmt.grade]
verdict_grade_align = (
    (verdict_sentiment == "positive" and grade_position == "high") or
    (verdict_sentiment == "negative" and grade_position == "low") or
    verdict_sentiment == "neutral"
)

# 2. Q1 hard-floor mention → Grade=F implication (STRUCTURAL coherence)
HARD_FLOOR_PATTERNS = [
    r"\b(5\+|6\+|7\+|8\+|9\+|10\+)\s*years\b",
    r"\b[5-9]-\d+\s*years\b",
    r"\bstaff-level\b", r"\bprincipal\b", r"\b£200\s*-?\s*\d+k\b",
]
q1_names_hard_floor = any(re.search(p, asmt.q1_prose.lower()) for p in HARD_FLOOR_PATTERNS)
if q1_names_hard_floor and asmt.grade not in ("F", None):
    # Q1 prose says hard floor but grade is not F — structural incoherence
    inconsistency_flag = True

# 3. Q3b risk-mention → Verdict-acknowledgment correlation
q3b_names_risk = any(t in asmt.q3b_prose.lower() for t in ["off-axis", "friction", "stretch", "gap"])
verdict_engages_risk = any(t in asmt.verdict_prose.lower() for t in ["off-axis", "friction", "pushback", "trade-off"])
risk_acknowledged_in_verdict = (q3b_names_risk and verdict_engages_risk) or not q3b_names_risk
```

**Output shape (per-agent rows):**

| Agent | Verdict↔Grade aligned % | Q1-hard-floor coherent % | Q3b-risk→Verdict % | Score |
|---|---:|---:|---:|---:|
| cluster-a-10job-1 | 92% | 100% | 78% | 90 |
| ... | | | | |

Headline axis score: mean per-agent coherence score across the three sub-checks.

---

## Axis E — Inter-agent variance

Pairwise exact-match agreement and within-1-letter agreement across the 11 core grading agents.

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

- Mean pairwise exact-match across all agent pairs with shared jobs.
- Mean pairwise within-1-letter.
- Per-job grade-letter range (max - min across the agents that saw the job, in letter units).

**Most-disagreed jobs table** (the 10 jobs with the largest letter-range across their agents):

| Job ID | Company | Title | Grades observed | Range |
|---|---|---|---|---:|
| 2447 | (company) | (title) | [SS, C, F, S] | 5 letters |
| ... | | | | |

**Important framing**: this table is INFORMATIONAL. The test does NOT claim which grade is "correct" for any job. The variance is reported as a property of grade-jobs' output. A high-variance job tells the user where grade-jobs disagrees with itself; the user decides whether the disagreement reflects ambiguity in the job, fuzziness in the rubric, or noise.

Headline axis score: a composite — `(mean_exact_match × 60) + (mean_within_one × 40)` × 100.

---

## Axis F — Pairwise consistency

Two pairwise agents each rank ~20 pairs. Measure agreement across them on shared pairs (if the manifests overlap) and within-agent transitivity.

```python
# pseudocode
shared_pairs = set(pw1.pair_ids) & set(pw2.pair_ids)
cross_agent_pair_agreement = sum(1 for p in shared_pairs if pw1.winner(p) == pw2.winner(p)) / len(shared_pairs)

# Within-agent transitivity: if pairs are (A,B), (B,C), (A,C), do the winners satisfy transitivity?
transitive_triples = 0
total_triples = 0
for triple in find_triples(pw1.pairs):  # (A,B), (B,C), (A,C) where the same job appears across pairs
    if is_transitive(pw1.winner(AB), pw1.winner(BC), pw1.winner(AC)):
        transitive_triples += 1
    total_triples += 1
within_agent_transitivity = transitive_triples / total_triples if total_triples else None

tie_rate = sum(1 for p in pw1.pairs + pw2.pairs if "tie" in p.winner) / (len(pw1.pairs) + len(pw2.pairs))
```

**Output shape:**

| Metric | Value |
|---|---:|
| Cross-agent pair agreement (shared pairs only) | 84% |
| Pairwise-1 within-agent transitivity | 92% |
| Pairwise-2 within-agent transitivity | 89% |
| Tie rate (combined) | 3% |

Headline axis score: `cross_agent_agreement × 60 + mean_transitivity × 30 + (1 - tie_rate) × 10` × 100. Higher = more consistent pairwise ranking.

If no shared pairs exist (manifests don't overlap), report cross-agent-agreement as N/A and use transitivity + tie-rate only.

---

## Axis G — Risk acknowledgment

For every assessment, measure how often the rubric names and engages with risks:

```python
RISK_PHRASES = [
    "friction", "gap", "off-axis", "adjacent", "stretch", "concern",
    "headwind", "soft floor", "narrow funnel", "selectivity",
    "credential floor", "stack mismatch", "career-axis mismatch"
]
risk_named = sum(1 for p in RISK_PHRASES if p in asmt.full_text.lower())
risk_in_q3b = sum(1 for p in RISK_PHRASES if p in asmt.q3b_prose.lower())
risk_in_verdict = sum(1 for p in RISK_PHRASES if p in asmt.verdict_prose.lower())

# Risk-direction correlation (STRUCTURAL coherence, not "is grade right"):
# When risks are named, does the grade tend lower than when no risks are named?
# This is a within-agent comparison — does THIS agent's grading honour the risks IT named?
risks_named_grades = [a.grade for a in agent.assessments if has_risk(a)]
no_risks_named_grades = [a.grade for a in agent.assessments if not has_risk(a)]
risk_direction_correlation = mean_grade_position(no_risks_named_grades) - mean_grade_position(risks_named_grades)
# Higher = risks correlate with grade decreases (risk-acknowledgment is real, not decorative)
```

**Output shape:**

| Agent | Assessments with named risk % | Mean risks / assmt | Risk→grade-decrease delta | Score |
|---|---:|---:|---:|---:|
| cluster-a-10job-1 | 76% | 1.4 | +0.8 letters | 84 |
| ... | | | | |

A high "risk → grade-decrease delta" means when this agent named risks in an assessment, the grade tended lower. That's structural coherence: the rubric's risk-naming actually bites. A delta near zero means risks are named decoratively (the §Worked Examples §"Risks-That-Bite" pattern is not firing).

Headline axis score: `risk_named_pct × 40 + risk_in_q3b_pct × 30 + risk_direction_correlation_normalised × 30` × 100.

---

## Cross-run regression diff

After computing the seven axes for the current run, load `context/test-runs/baseline.json` (if it exists) and compute the per-axis delta.

```python
def regression_diff(current_scores, baseline_scores):
    return {axis: current_scores[axis] - baseline_scores[axis] for axis in current_scores}
```

**Output shape:**

| Axis | Current | Baseline (run-id) | Δ | Direction |
|---|---:|---:|---:|---|
| A — Format adherence | 91 | 86 | +5 | improved |
| B — Reasoning specificity | 88 | 82 | +6 | improved |
| C — Q3a/Q3b differentiation | 76 | 71 | +5 | improved |
| D — Internal consistency | 90 | 87 | +3 | improved |
| E — Inter-agent variance | 72 | 67 | +5 | improved |
| F — Pairwise consistency | 84 | 80 | +4 | improved |
| G — Risk acknowledgment | 81 | 74 | +7 | improved |
| **Composite** | **83.1** | **78.1** | **+5.0** | **improved** |

After computing the diff, the script writes the new scores to `context/test-runs/baseline.json`, overwriting the prior baseline. The prior baseline is preserved as a row in `context/test-runs/baseline-history.json` (append-only) so the user can trace iteration history.

**Important framing**: the regression diff reports direction. It does NOT claim "improvement is good" or "regression is bad" in absolute terms. The user judges whether the direction reflects an actual rubric improvement or an artefact of sample variance.

If `baseline.json` does not exist on first run, the regression-diff section reports "first run; no baseline to diff against" and the current scores become the new baseline.

---

## Q1 prose inference (no agent-emitted tags)

Earlier versions of the test asked agents to emit a `Q1: <verdict>` metadata tag. This was removed because grade-jobs' rubric bans verdict-enum labels in slot prose, and the metadata tag pulled agents back toward label-thinking even outside the slots. The Q1 verdict reading is now INFERRED from Q1-slot prose at parse time using rule-based pattern detection.

Pattern rules:

```python
def infer_q1_verdict(q1_prose):
    lower = q1_prose.lower()
    # Hard-floor patterns first (most restrictive wins)
    if any(re.search(p, lower) for p in HARD_FLOOR_PATTERNS):
        return "hard-fail"
    # Narrow-funnel selectivity language
    if any(t in lower for t in [
        "narrow-funnel", "narrow funnel", "selectivity", "sub-1%",
        "less than 1%", "highly selective", "lottery", "prestige-trap"
    ]):
        return "real-headwind"
    # Cleared-with-friction signals
    if any(t in lower for t in [
        "soft floor", "2+ years", "friction", "stretch", "borderline",
        "cleared with friction"  # legacy label that may leak in
    ]):
        return "cleared-with-friction"
    # Default: cleared-decisively (graduate programmes, wide-funnel, no friction language)
    return "cleared-decisively"
```

The inferred verdict is used in Axis D (internal consistency: Q1-hard-floor → Grade=F coherence) and reported in the per-job table for transparency. It is NOT used to assert any agent's grade is correct.

The pattern lists are minimal seeds and may extend per the additive-freedom permission below.

---

## Report file audience

The report at `context/test-runs/test-grade-jobs-<run-id>.md` is **human-facing**. The user reads it on its own merits to understand how grade-jobs is performing.

Per the audience-routing principle:

- Tight signal density. Every paragraph earns its place.
- No process language. The user doesn't need "Phase 1 evidence:" framing in the report.
- No admin metadata. Frontmatter / evidence-block sections belong in the chat summary.
- Reading-grade prose. Markdown formatting calibrated for visual scanning. Tables, hierarchy.
- Self-contained. The file should make sense to a reader who hasn't seen the chat output.
- **No grade assertions**. The report describes what grade-jobs produced. It does not judge that any specific grade is right or wrong.

A 2-4 sentence executive summary at the top names the headline finding — per-axis scores + regression direction — so a skim reader gets the verdict immediately.

---

## Anti-Patterns

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| **Asserting a specific grade is "correct" anywhere in the report** | Violates the no-bias rule. The test measures grade-jobs' coherence, not its agreement with an external judge. | Describe what grade-jobs produced. Variance is reported, not adjudicated. |
| **Hand-curating expected outcomes per job** | Encodes external answers. The user explicitly rejected this. | Measurements are computed from the rubric's structural rules, not from per-job expectations. |
| **Importing a "correct distribution shape" target** | Same shape — frames one distribution as right. | Distribution is reported as-is. Regression diff shows direction, not correctness. |
| **Computing only the headline number per axis** | Loses the per-agent or per-job detail that lets the user trace the score to specific cases. | Every axis produces a headline + a backing table with per-agent or per-job evidence. |
| **Skipping axes that "don't apply" this run** | The seven-axis structure makes runs comparable across time. Silent omission breaks comparability. | Axes with sparse data still appear, stating "no data this run; cause: ..." |
| **Subjective verdict without axis-evidence anchors** | The user gets opinion without traceability. | Every verdict claim cites the axis-score that motivates it. |
| **Mixing letter grades and numeric forms in the same column** | Reader has to translate as they read. | Letters in the per-row data; numerics in the aggregate computations. |
| **Asking agents to emit verdict-enum metadata tags** | Pulls agents back toward label-thinking, contradicts grade-jobs' rubric. | Infer the Q1 verdict from Q1-slot prose at parse time. |

---

## Additive-Freedom Permission for Prescribed Lists in This File

The lists in this file are non-exhaustive and may be extended:

- **The seven axes** are the current minimum. If a future test dimension warrants Axis H+ (e.g. "stack-concentration carveout firing rate" if that becomes load-bearing), add it; the existing seven remain mandatory.
- **The banned-string set** in Axis A reflects grade-jobs' current no-verdict-enums rule. If grade-jobs adds new banned vocabulary, mirror it here.
- **The GENERIC_PHRASES list** in Axis B is the current observed-in-practice set. New generic-sounding phrases observed in agent output extend it.
- **The CAREER_AXIS_TERMS list** in Axis C reflects grade-jobs' Q3b vocabulary. Iterations of grade-jobs that introduce new career-axis vocabulary should be mirrored here.
- **The RISK_PHRASES list** in Axis G reflects grade-jobs' risk-naming vocabulary. Same — mirror grade-jobs.
- **The Q1 prose inference patterns** are minimal seeds. New patterns observed in production grade-jobs output extend the lists.
- **The anti-pattern catalogue** — when a new analysis failure is observed, add a row.

Additions must be purely additive — they may not weaken or replace any existing section. Document additions in the next commit's message.
