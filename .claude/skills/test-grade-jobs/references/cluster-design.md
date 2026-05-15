# Cluster Design

## Table of Contents

- [Purpose](#purpose)
- [The Two-Cluster Stratification](#the-two-cluster-stratification)
- [Cluster A — Stress Tests (30 jobs)](#cluster-a--stress-tests-30-jobs)
- [Cluster B — Stability Anchors (30 jobs)](#cluster-b--stability-anchors-30-jobs)
- [Selection SQL](#selection-sql)
- [Trigger-Case Identification](#trigger-case-identification)
- [Coverage Matrix](#coverage-matrix)
- [Anti-Patterns](#anti-patterns)

---

## Purpose

This reference describes how `scripts/select-jobs.py` picks the 60 jobs for a test run. The selection is deliberately stratified so the run measures two different rubric behaviours simultaneously — does the rubric correctly catch its known stress cases (Cluster A), and does it preserve the cases it should clearly get right (Cluster B). A random sample of 60 jobs would dilute both signals.

The script picks fresh every run. The DB evolves as new jobs land, new grades are written, and the rubric itself iterates. Yesterday's sample is not the right sample for today's rubric.

---

## The Two-Cluster Stratification

```
┌──────────────────────────────┐    ┌──────────────────────────────┐
│  Cluster A: Stress Tests     │    │  Cluster B: Stability Anchors │
│  (30 jobs)                   │    │  (30 jobs)                    │
│                              │    │                              │
│  ▪ Trigger cases             │    │  ▪ Wide-funnel grad roles    │
│    (prestige traps)          │    │    (should be S / A)         │
│  ▪ Boundary cases            │    │  ▪ Hard floors → F           │
│    (2+yr borderlines,        │    │    (5+yr, staff comp,        │
│     "currently pursuing")    │    │     location hard-fail)      │
│  ▪ Exclusion-edge cases      │    │  ▪ Mid-tier UK fintech → B   │
│    (defence/SC, customer-    │    │  ▪ Off-stack mid-level → C   │
│     facing borderline)       │    │  ▪ Legit-A engineering roles │
│                              │    │                              │
│  Tests: does the rubric      │    │  Tests: does the rubric      │
│  catch what it should catch? │    │  preserve what it gets right?│
└──────────────────────────────┘    └──────────────────────────────┘
```

A regression in Cluster B is a red flag (the new rubric broke something obvious). A miss in Cluster A is the failure mode the test exists to detect (the rubric hasn't caught the stress case after all).

---

## Cluster A — Stress Tests (30 jobs)

Selection criteria. The 30 stress-test jobs are picked from the union of these patterns:

| Stress pattern | Detection heuristic | Why it stresses the rubric |
|---|---|---|
| **Prestige trap (narrow-funnel quant / HFT)** | Company name in {Jane Street, Hudson River Trading, XTX, Citadel, Two Sigma, DE Shaw, Jump, Tower, Old Mission, Optiver}; OR `fit_assessment` contains "narrow-funnel" / "sub-1%" / "lottery" / "stretch" / "headwind" | These are the canonical Q1-headwind cases the rubric is meant to aggregate down to C |
| **Implicit-seniority disguise** | Title contains "Software Engineer" / "Software Developer" without "Graduate" / "Junior" / "Intern" qualifier; description requires N+ years where N >= 3 | The rubric must read past the generic title to the years floor |
| **"Currently pursuing" intern misfit** | Title contains "Intern" / "Internship"; description contains "currently pursuing" / "current student" / "expected graduation date" | Tests handling of the 2025-graduate-but-still-want-an-intern-role mismatch |
| **Role-type mismatch (analyst/trader as engineering substitute)** | Title contains "Analyst" / "Trader" / "Quantitative Researcher" / "Data Scientist"; on a company profile that overlaps with engineering interests | Tests whether rubric correctly weighs role-type alignment vs CV brand |
| **Customer-facing borderline** | Description contains "interact with customers" / "customer-engagement" / "product feedback from users" | Tests against the `preferences.toml::exclude_role_types` boundary |
| **Stack-zero misfit** | Description requires a stack the profile has zero exposure to (Kotlin/Android, iOS/Swift, Salesforce, etc.) | Tests hard stack-zero detection |
| **Defence/SC clearance** | Description mentions "security clearance" / "SC clearance" / "DV clearance" / "UK national required" | Tests against `visa.md` SC-eligibility constraint |
| **Staff-tier comp band disguise** | Salary range >£200k; role title generic; description uses "lead", "own", "shape", "principal" | Tests rubric's ability to read seniority from comp + scope language even when title is ambiguous |

The selection script picks up to N jobs per pattern with `LIMIT` clauses, then mixes them so cluster A has variety without any single pattern dominating.

**Coverage constraint**: cluster A should span every existing letter grade in the DB. A cluster A composed entirely of A-grade-in-DB jobs would only test downward correction. Include some B/C/F jobs in cluster A to also test cases the old rubric got right (the same job graded again should not move).

---

## Cluster B — Stability Anchors (30 jobs)

Selection criteria. The 30 stability-anchor jobs are picked from:

| Stability pattern | Detection heuristic | What it anchors |
|---|---|---|
| **Wide-funnel graduate roles** | Title contains "Graduate" / "New Grad" / "2026 Grad"; company is mid-or-large tech firm with structured graduate intake (Cloudflare, Stripe, Spotify, Palantir, B2C2, Squarepoint) | Should consistently land at S or A |
| **Hard years-floor F** | Description states "4+ years" / "5+ years" / "5-10 years" experience required | Should consistently land at F (or close-to-F C) |
| **Location hard-fail** | Job location not in `preferences.toml::hard.locations` (e.g. Bristol, Edinburgh, Manchester, Berlin, non-Remote-UK) | Should consistently land at F |
| **Mid-tier UK fintech → B** | Company in {Lendable, Trainline, Monzo, Zopa, Starling, Wise, Cleo}; role is graduate-or-junior | Should consistently land at B |
| **Off-stack mid-level → C** | Description requires Kotlin/Android/iOS/Salesforce or similar non-portfolio stack at mid-level | Should consistently land at C |
| **Legit-A engineering** | Description is graduate-explicit + portfolio-anchored stack (Rust, Python, TypeScript) + recognised firm | Should consistently land at A |

If the same job was previously graded by the test rubric and the agents disagree on the letter, that's a calibration drift signal in cluster B — louder than the same signal in cluster A.

---

## Selection SQL

The script's underlying query shape (parameterised, not literal SQL — the script uses sqlite3 with bound params):

```python
# Cluster A: stress tests
SELECT j.id, j.title, j.url, j.location, j.remote_policy,
       j.raw_description, c.name AS company_name, c.what_they_do,
       j.grade AS db_grade_DO_NOT_INCLUDE_IN_MANIFEST
FROM jobs j JOIN companies c ON c.id = j.company_id
WHERE j.evaluation_status <> 'archived'
  AND LENGTH(j.raw_description) > 500
  AND (
    c.name IN (...)                                   -- prestige-trap firms
    OR j.title LIKE '% Engineer%' AND j.raw_description LIKE '%3+ years%'   -- seniority disguise
    OR j.title LIKE '%Intern%' AND j.raw_description LIKE '%currently pursuing%'
    OR j.title LIKE '%Analyst%' OR j.title LIKE '%Trader%'
    OR j.raw_description LIKE '%security clearance%'
    OR j.raw_description LIKE '%customer engagement%'
    OR j.fit_assessment LIKE '%stretch%' OR j.fit_assessment LIKE '%lottery%'
  )
ORDER BY RANDOM()
LIMIT 30;
```

The `db_grade_DO_NOT_INCLUDE_IN_MANIFEST` column is selected so the script can compute coverage statistics, but the column is stripped before any per-agent manifest is written. The grep guard in SKILL.md Phase 1 Step 1.6 catches any leakage.

Cluster B uses an analogous query against the stability patterns.

---

## Trigger-Case Identification

A subset of cluster A is tagged as "trigger cases" — jobs whose old DB grade contains specific Q1-friction language that the new rubric is explicitly designed to correct. The trigger-case correction rate is the test's headline metric for "did the rubric fix what it was meant to fix?".

**Heuristic for trigger-case identification (run during selection):**

```python
TRIGGER_PHRASES = [
    "stretch", "lottery", "sub-1%", "headwind", "prestige-trap",
    "stretch-A", "A-stretch", "narrow-funnel", "brutal selectivity",
    "lottery ticket", "lottery band",
]
# A job is a "trigger case" iff its old fit_assessment contains
# any TRIGGER_PHRASES token AND its db_grade is in {SS, S, A}.
```

The script counts trigger cases in cluster A and writes the count to stdout. If the count is below ~5, the trigger-case correction rate analysis becomes statistically thin — note this in the report's Limitations section.

The list is not exhaustive — when the rubric grows new failure-mode framings, add their canonical phrasings to TRIGGER_PHRASES.

---

## Coverage Matrix

The script writes `/tmp/test-grade-jobs-<run-id>/coverage-matrix.json` mapping each job to the list of agent IDs that will grade it. Shape:

```json
{
  "2447": ["agent-cluster-a-10job-1", "agent-cluster-a-15job-1", "agent-cluster-a-30job", "agent-cross-cluster-1", "agent-full-60-1", "agent-anchor-injected"],
  "3334": ["agent-cluster-a-10job-2", "agent-cluster-a-15job-1", "agent-cluster-a-30job", "agent-cross-cluster-1", "agent-full-60-1", "agent-rubric-blind"],
  ...
}
```

Every job in cluster A appears in: one of the three 10-job agents + one of the two 15-job agents + the 30-job agent + (some) cross-cluster + (some) full-60 + (some) rubric-blind + (some) anchor-injected. Minimum coverage = 6 agents per job (depending on which cross-cluster split + which full-60 + whether the rubric-blind / anchor-injected pick the job).

The coverage matrix is the analysis script's input — without it, the per-job grade distribution cannot be assembled.

---

## Anti-Patterns

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| **Random sample of 60 jobs** | Dilutes the stress-test signal. Most random samples are dominated by mid-band B/C jobs where the rubric's behaviour is unremarkable. | Stratify into cluster A + cluster B per the patterns above. |
| **Cluster A composed entirely of A-grade-in-DB jobs** | Only tests downward correction. The rubric must also preserve correct grades. | Include some B/C/F jobs in cluster A — these are the "unchanged" cases the test confirms didn't break. |
| **Trigger case selection by company name only** | Misses jobs from prestige firms that aren't in the canonical-name list. Catches non-stretch jobs from listed firms. | Add the `fit_assessment` keyword check as well — trigger cases must satisfy both selectivity-axis criteria. |
| **Selection seeded deterministically across runs** | Two runs against the same DB pick the same 60 jobs. Each run becomes a re-test, not an independent test. | Seed by current timestamp (the default). For reproducibility (rare), the user can pass an explicit seed. |
| **Hard-coding trigger-phrase list inline in the script** | The list rots as the rubric introduces new failure-mode framings. | Keep TRIGGER_PHRASES in this reference file (where iteration of grade-jobs surfaces new phrasings) and have the script import / copy from here at run time. |

---

## Additive-Freedom Permission for Prescribed Lists in This File

The lists in this file are non-exhaustive and may be extended:

- **The stress-pattern catalogue (8 patterns)** — when a new failure-mode class is observed in production grading, add a row to Cluster A.
- **The stability-pattern catalogue (6 patterns)** — when a new clearly-correct case class is observed, add a row to Cluster B.
- **The TRIGGER_PHRASES list (11 phrases)** — when the rubric introduces a new failure-mode framing, add its canonical phrasings.
- **The anti-pattern catalogue** — when a new selection failure is observed, add a row.

Additions must be purely additive — they may not weaken or replace any existing pattern.
