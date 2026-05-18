# Cluster Design

## Table of Contents

- [Purpose](#purpose)
- [The Two-Cluster Stratification](#the-two-cluster-stratification)
- [Cluster A — Stress-pattern Sample (30 jobs)](#cluster-a--stress-pattern-sample-30-jobs)
- [Cluster B — Stability-pattern Sample (30 jobs)](#cluster-b--stability-pattern-sample-30-jobs)
- [Selection SQL](#selection-sql)
- [Coverage Matrix](#coverage-matrix)
- [Anti-Patterns](#anti-patterns)

---

## Purpose

This reference describes how `scripts/select-jobs.py` picks the 60 jobs for a test run. The selection is deliberately stratified by **pattern** so the run measures grade-jobs across a varied surface — narrow-funnel firms, hard-floor descriptions, wide-funnel grad pipelines, off-stack roles, customer-facing roles, etc. A random sample of 60 jobs would oversample the corpus's mid-band and undersample the cases that exercise different parts of grade-jobs' rubric.

**The clusters are diversity samples, not expectation labels.** The names "stress" and "stability" describe the *patterns* the samples cover — patterns the rubric is likely to react to differently — not what grade-jobs is expected to produce. The test does not assert what grade-jobs "should" output for any specific job, pattern, or cluster.

The script picks fresh every run. The DB evolves as new jobs land, new grades are written, and the rubric itself iterates. Yesterday's sample is not the right sample for today's rubric.

---

## The Two-Cluster Stratification

```
┌──────────────────────────────┐    ┌──────────────────────────────┐
│  Cluster A: Stress patterns  │    │  Cluster B: Stability patterns│
│  (30 jobs)                   │    │  (30 jobs)                    │
│                              │    │                              │
│  Patterns the rubric is      │    │  Patterns the rubric is      │
│  likely to react to with     │    │  likely to react to with     │
│  HIGH between-Q variance:    │    │  LOW between-Q variance:     │
│                              │    │                              │
│  ▪ Narrow-funnel firms       │    │  ▪ Wide-funnel grad/intern   │
│  ▪ Boundary seniority        │    │  ▪ Explicit hard floors      │
│  ▪ Exclusion-edge cases      │    │  ▪ Hard location/sector      │
│  ▪ Role-type ambiguity       │    │  ▪ Mid-tier UK fintech       │
│  ▪ Off-stack mid-level       │    │  ▪ Standard junior engineering│
│                              │    │                              │
│  Tests: inter-agent variance │    │  Tests: inter-agent variance │
│  on the rubric's harder      │    │  on the rubric's easier      │
│  decisions. High variance is │    │  decisions. High variance is │
│  expected and informative.   │    │  a stronger drift signal.    │
└──────────────────────────────┘    └──────────────────────────────┘
```

Cluster B inter-agent variance is a stronger drift signal than cluster A variance, because cluster B patterns are the ones where the rubric's reasoning should converge most. If cluster B shows high variance, the rubric has internal coherence problems. Cluster A high variance is also informative but harder to interpret — it could mean the rubric is genuinely fuzzy on hard cases (a real problem) or that the cases are genuinely ambiguous (which is acceptable).

The test does NOT assert specific grades for either cluster. The cluster names describe pattern diversity, not outcome expectations.

---

## Cluster A — Stress-pattern Sample (30 jobs)

Selection criteria. The 30 stress-pattern jobs are picked from the union of these patterns:

| Stress pattern | Detection heuristic | Why it's a stress test |
|---|---|---|
| **Narrow-funnel firms** | Company name in {Jane Street, Hudson River Trading, XTX, Citadel, Two Sigma, DE Shaw, Jump, Tower, Old Mission, Optiver, Squarepoint, Point72, Cubist, Millennium, G-Research, Susquehanna, SIG, Belvedere, DRW, IMC Trading} | Rubric must apply Q1 selectivity reasoning, not pattern-match on prestige |
| **Implicit-seniority disguise** | Title contains "Software Engineer" / "Software Developer" without "Graduate" / "Junior" / "Intern" qualifier; description contains "X+ years" where X ≥ 3 | Rubric must read past the generic title to the years floor |
| **"Currently pursuing" intern misfit** | Title contains "Intern" / "Internship"; description contains "currently pursuing" / "current student" / "expected graduation date" | Tests boilerplate-vs-structural-filter detection |
| **Role-type ambiguity** | Title contains "Analyst" / "Trader" / "Quantitative Researcher" / "Data Scientist"; on a company with engineering reputation | Tests Q3b career-axis reasoning vs Q2 company brand |
| **Customer-facing borderline** | Description contains "interact with customers" / "customer-engagement" / "product feedback from users" | Tests preferences.toml exclude_role_types boundary |
| **Stack-zero misfit** | Description requires a stack with no portfolio exposure (Kotlin/Android, iOS/Swift, Salesforce) | Tests stack-fit vs concept-fit decomposition |
| **Defence/SC clearance** | Description mentions "security clearance" / "SC clearance" / "DV clearance" / "UK national required" | Tests visa.md clearance constraint |
| **Staff-tier comp band disguise** | Salary range > £200k; role title generic; description uses "lead", "own", "shape", "principal" | Tests seniority detection from comp + scope language |

The selection script picks up to N jobs per pattern with `LIMIT` clauses, then mixes them so cluster A has variety without any single pattern dominating.

**Coverage**: cluster A spans jobs at different DB grades. A cluster A composed entirely of A-grade-in-DB jobs would oversample the rubric's downward-correction surface; include some B/C/F jobs in cluster A to span the variance surface fully.

---

## Cluster B — Stability-pattern Sample (30 jobs)

Selection criteria. The 30 stability-pattern jobs are picked from:

| Stability pattern | Detection heuristic |
|---|---|
| **Wide-funnel graduate roles** | Title contains "Graduate" / "New Grad" / "2026 Grad"; company is mid-or-large tech firm with structured graduate intake (Cloudflare, Stripe, Spotify, Palantir, B2C2, Wise, Monzo, GitLab) |
| **Hard years-floor** | Description states "4+ years" / "5+ years" / "5-10 years" experience required |
| **Hard location exclusion** | Job location not in `preferences.toml::hard.locations` (Bristol, Edinburgh, Manchester, Berlin, non-Remote-UK) |
| **Mid-tier UK fintech, junior tier** | Company in {Lendable, Trainline, Monzo, Zopa, Starling, Wise, Cleo}; role is graduate-or-junior |
| **Off-stack mid-level** | Description requires Kotlin/Android/iOS/Salesforce or similar non-portfolio stack at mid-level |
| **Standard junior engineering at recognised firm** | Description is graduate-explicit + portfolio-anchored stack (Rust, Python, TypeScript) + recognised firm |

The pattern descriptions name the structural shape of the job, NOT what grade-jobs should output. The rubric may or may not produce a particular grade for any pattern — that's measurement, not specification.

If the same job was previously included in a prior run's cluster B and the agents now disagree on the letter, that's a calibration drift signal — louder than the same signal in cluster A. But the test does not assert what the prior or current correct grade is; it reports the disagreement and the user judges whether the rubric drifted or genuinely improved.

---

## Selection SQL

The script's underlying query shape (parameterised, not literal SQL — the script uses sqlite3 with bound params):

```python
# Cluster A: stress patterns
SELECT j.id, j.title, j.url, j.location, j.remote_policy,
       j.raw_description, c.name AS company_name, c.what_they_do
FROM jobs j JOIN companies c ON c.id = j.company_id
WHERE j.evaluation_status <> 'archived'
  AND LENGTH(j.raw_description) > 500
  AND (
    c.name IN (...)                                   -- narrow-funnel firms
    OR (j.title LIKE '% Engineer%' AND j.raw_description LIKE '%3+ years%')  -- seniority disguise
    OR (j.title LIKE '%Intern%' AND j.raw_description LIKE '%currently pursuing%')
    OR j.title LIKE '%Analyst%' OR j.title LIKE '%Trader%'
    OR j.raw_description LIKE '%security clearance%'
    OR j.raw_description LIKE '%customer engagement%'
  )
ORDER BY RANDOM()
LIMIT 30;
```

The script does NOT select `j.grade` or `j.fit_assessment` into any output. Those columns are deliberately omitted from the manifest construction; the grep guard in SKILL.md Phase 1 Step 1.6 catches any leakage.

Cluster B uses an analogous query against the stability patterns.

---

## Coverage Matrix

The script writes `/tmp/test-grade-jobs-<run-id>/coverage-matrix.json` mapping each job to the list of agent IDs that will grade it. Shape:

```json
{
  "2447": ["agent-cluster-a-10job-1", "agent-cluster-a-15job-1", "agent-cluster-a-30job", "agent-cross-cluster"],
  "3334": ["agent-cluster-a-10job-2", "agent-cluster-a-15job-2", "agent-cluster-a-30job", "agent-cross-cluster"],
  ...
}
```

Every job in cluster A appears in: one of the three 10-job agents + one of the two 15-job agents + the 30-job agent + (possibly) the cross-cluster agent. Every job in cluster B appears in: one of the three 10-job-B agents + the 30-job-B agent + (possibly) the cross-cluster agent. Minimum coverage = 4 agents per job. Some jobs hit by the cross-cluster agent get 5.

The coverage matrix is the analysis script's input — without it, inter-agent variance cannot be computed.

---

## Anti-Patterns

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| **Random sample of 60 jobs** | Dilutes the pattern-diversity signal. Most random samples are dominated by mid-band B/C jobs where the rubric's behaviour is unremarkable. | Stratify into cluster A + cluster B per the patterns above. |
| **Encoding per-pattern grade expectations** | "Wide-funnel grad should be S/A" is a curated answer that the user rejected. The test must not assert outcomes per pattern. | Pattern descriptions name the structural shape only. The test reports what grade-jobs produced; the user judges. |
| **Cluster A composed entirely of A-grade-in-DB jobs** | Only spans part of the variance surface. | Include jobs across DB grades so cluster A spans the full surface. |
| **Selection seeded deterministically across runs** | Two runs against the same DB pick the same 60 jobs. Each run becomes a re-test, not an independent test. | Seed by current timestamp (the default). For reproducibility (rare), the user can pass an explicit seed. |
| **Adding pattern-specific "expected" comments inline** | Drifts toward curated-answers over time. Any comment of the form "the rubric should grade this X" is a violation. | Comments describe pattern detection, not pattern outcomes. |

---

## Additive-Freedom Permission for Prescribed Lists in This File

The lists in this file are non-exhaustive and may be extended:

- **The stress-pattern catalogue (8 patterns)** — when a new pattern class is observed in production, add a row. New patterns describe structural shape, never outcome expectations.
- **The stability-pattern catalogue (6 patterns)** — same.
- **The anti-pattern catalogue** — when a new selection failure is observed, add a row.

Additions must be purely additive — they may not weaken or replace any existing pattern. Pattern descriptions must NEVER state "the rubric should produce X for jobs matching this pattern" — that's the no-curated-answers rule applied to this file.
