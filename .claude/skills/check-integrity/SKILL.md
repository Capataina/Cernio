---
name: check-integrity
description: "Audits the Cernio SQLite database — runs `cernio check` + `cernio format` (steps 1-2), then applies judgment for profile-driven grade staleness, shallow reasoning, stale `why_relevant`, cross-tier consistency, missing data. Step 8 maintains `profile/portfolio-gaps.md` — samples 10 jobs per tier, extracts market patterns, writes findings. Invoke on 'check integrity', 'audit the database', 'health check', 'verify data quality', 'are my grades up to date', 'check for stale grades', after profile updates that may invalidate grades, before a job search, or after a grading run. Not for grading new companies (grade-companies), new jobs (grade-jobs), populating (populate-db), resolving ATS (resolve-portals), or preparing applications (prepare-applications). Use whenever grade freshness or database coherence is in doubt, even if not named."
---

# Check Integrity

The Cernio database is a living system. Grades, fit assessments, and `why_relevant` fields written yesterday may be wrong today — a new flagship project in the profile, a shifted preference, an expanded skill set can silently invalidate prior judgments without altering any timestamp. Mechanical integrity checks catch schema-level and timestamp-level drift; this skill catches semantic drift — the class of staleness that only shows up when the profile is read alongside the grade reasoning.

The skill runs in two modes. In report mode (default), it produces a prioritised findings list and stops — no DB writes. In remediation mode (triggered by the user saying "fix these" / "update these" / "regrade these"), it works through the findings with the user's explicit approval per fix, using the procedures in `references/remediation-guide.md` and the quality bars in `references/quality-standards.md`.

The output is not just a report. Step 8 — portfolio gap analysis — writes directly to `profile/portfolio-gaps.md`. This is the career-coaching side of Cernio; a check-integrity run that does not update portfolio-gaps is a broken loop.

---

## Mandatory Reads Before Checking

| # | What | Evidence |
|---|---|---|
| 1 | **Every file in `profile/`** | Findings cite specific profile files and the specific element that is now stale relative to a grade (e.g. "Company X's grade_reasoning says 'lacks cloud experience'; `profile/projects.md` now includes an AWS deployment in project Y") |
| 2 | **`references/remediation-guide.md`** (191 lines, TOC present) | Each report-mode finding maps to a named remediation procedure ready for the remediation-mode switch |
| 3 | **`references/quality-standards.md`** (83 lines) | Grade-quality flagging cites the specific "acceptable" / "unacceptable" examples from this file |
| 4 | **`references/profile-context.md`** (52 lines) | Staleness detection focuses on the profile-change-to-grade-reasoning link, not blanket re-evaluation |
| 5 | **`references/cross-checking-guide.md`** (238 lines, TOC present) | Cross-tier consistency checks follow the comprehensive procedure — this is not a spot-check |

The profile is not cached in this skill or in any reference file; every invocation reads `profile/` fresh. Hardcoded profile facts would diverge silently and produce false "still accurate" verdicts on grades that are actually stale.

---

## Workflow

### 1. Run `cernio check`

Mechanical baseline — catches schema-level problems. Captures: dead ATS slug endpoints, stale-by-timestamp entries, orphaned foreign keys, missing required fields.

```bash
cargo run -- check
```

Note the findings. Schema problems feed the "Mechanical Issues" section of the report.

### 2. Run `cernio format`

Cleans raw HTML / entity-encoded descriptions and fit assessments across the database. Idempotent and fast — safe to run every invocation. Without this step, downstream grading agents waste tokens on `data-*` attributes and nested tag soup, producing confused assessments.

```bash
cargo run -- format
```

If any rows were formatted, note the count in the report.

### 3. Read the current profile fresh

Every file in `profile/`, every invocation. Attention heads should land on:

- **Recently modified profile files** — these are the primary drivers of potential staleness
- **Newly-added projects and skills** — expand what the candidate can now match
- **Preference shifts** — reset which companies / roles are relevant
- **Visa status changes** — shift sponsorship weighting

### 4. Detect profile-driven staleness

Query graded companies:

```sql
SELECT id, name, grade, grade_reasoning, graded_at, why_relevant, status
FROM companies
WHERE grade IS NOT NULL
  AND status != 'archived'
ORDER BY graded_at ASC;
```

For each graded company, the question is not "has this grade aged" but "does the grade reasoning still hold given the current profile?" Focus on entries where a specific profile change is directly relevant to the reasoning, not blanket re-evaluation.

Examples of what to flag:

- Grade reasoning cites a skill gap that the profile has since filled ("lacks Kubernetes" → `profile/projects.md` now has a K8s project → recommend upward re-grade)
- Grade reasoning relies on a preference that has changed ("London-only" → `profile/preferences.toml` now accepts Manchester → reasoning is stale)
- Grade reasoning predates a significant new flagship project that strengthens the technical-alignment argument

A new Rust project does not affect a company graded on sponsorship concerns. Relevance of the profile change to the specific reasoning is the filter.

### 5. Audit grade quality

Pull 3–5 graded companies and 3–5 graded jobs. For each, verify against the standards in `references/quality-standards.md`:

- Is the reasoning specific or generic filler?
- Does it cite specific profile elements by name?
- For jobs: does the `fit_assessment` accurately reflect the match between the role's actual requirements and the profile?
- Is the reasoning contradicted by the current profile state?

Flag thin or contradicted entries.

### 6. Cross-check across the full universe

This is the pass that catches errors no individual grading session can see. Follow `references/cross-checking-guide.md` in full.

**Companies:** verify within-tier coherence (do all A-tier companies genuinely belong together), across-tier gradients (is every A genuinely less valuable than every S), and specific red flags (famous employer at C, unknown startup at S, tech-stack-only justification for a grade).

**Jobs:** verify company-grade / job-grade consistency (a graduate role at an S-tier company should rarely be below A), compare all SS / S jobs against each other, check seniority-requirement citations in fit assessments, spot-check description-assessment consistency by reading both for a sample.

**The cardinal rule (from cross-checking-guide.md):** before changing any grade, the complete DB record (what_they_do / raw_description, grade_reasoning / fit_assessment, why_relevant) AND the full candidate profile were read this invocation. A grade change without the full record read is damage, not value.

Present findings as recommendations. The user decides what to act on — no DB writes in report mode.

### 7. Missing-data sweep

```sql
-- Ungraded companies
SELECT id, name, status FROM companies
WHERE grade IS NULL AND status != 'archived';

-- Ungraded jobs
SELECT j.id, j.title, c.name AS company
FROM jobs j
JOIN companies c ON j.company_id = c.id
WHERE j.fit_grade IS NULL AND j.status != 'archived' AND c.status != 'archived';

-- Empty why_relevant
SELECT id, name, grade FROM companies
WHERE (why_relevant IS NULL OR why_relevant = '') AND status != 'archived';

-- Missing job descriptions
SELECT j.id, j.title, c.name AS company
FROM jobs j
JOIN companies c ON j.company_id = c.id
WHERE (j.description IS NULL OR j.description = '') AND j.status != 'archived';
```

Quantify the results.

### 8. Portfolio gap analysis — active maintenance, not passive check

This step writes to `profile/portfolio-gaps.md`. The check without the write is a broken loop.

Sample 10 jobs per grade tier (SS, S, A, B, C, F — fewer if the tier has < 10 jobs), prioritised by company grade (S-tier company jobs first, then A, then B):

```sql
SELECT j.id, j.title, j.grade, j.raw_description, j.fit_assessment,
       c.name AS company, c.grade AS company_grade
FROM jobs j
JOIN companies c ON j.company_id = c.id
WHERE j.grade = ?1 AND j.evaluation_status <> 'archived'
ORDER BY CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END, RANDOM()
LIMIT 10;
```

Read the `raw_description` and `fit_assessment` for each sampled job (~60 total). Across the sample, track:

| Pattern | What to record |
|---|---|
| Technologies appearing repeatedly but absent from `profile/skills.md` | Technology, count, role types + companies where it appeared, sector concentration |
| Domain knowledge the market expects | Topic, count, role types (e.g. "FIX protocol — 4 trading roles") |
| Experience patterns that recur | Pattern, count (e.g. "production incident management — 8 roles") |
| Confirmed strengths the market values | Technology / skill from the profile, count in SS / S roles |

Write findings to `profile/portfolio-gaps.md`:

- **"Patterns from Job Evaluations"** — update with concrete new findings (technology, count, roles / companies, recommended closure if a gap)
- **"Known Gaps"** — update if new gaps found or existing gaps closed by recent profile additions
- **"Current Strengths"** — update if the market clearly values profile strengths

Even a null result deserves a dated note ("No new patterns found in this sample — checked 2026-04-20"). Silent non-updates break the loop.

### 9. Relevance refresh

For companies where `why_relevant` is generic ("interesting tech company", "found on fintech list") or stale (references profile details that have shifted), draft updated relevance statements that connect the company to the current profile specifically. Present as suggestions — no DB writes in report mode.

### 10. Present the prioritised report

Structure the output by severity:

```
## Integrity Report

### Mechanical Issues (from cernio check)
- [list]

### Profile-Driven Staleness
- **[Company Name]** (current grade: B, graded 2026-03-15)
  Grade reasoning: "lacks cloud experience" — profile now includes AWS
  deployment in project X. Recommend re-evaluation; likely A-tier.

### Grade Quality Issues
- [quoted generic reasoning and the specific quality-standards.md example it fails against]

### Cross-Tier Inconsistencies
- [comparisons that failed the within-tier-coherence or cross-tier-gradient check]

### Missing Data
- N companies without grades
- M jobs without evaluations
- K companies with empty why_relevant

### Portfolio Gap Analysis
- [new patterns found in the 60-job sample; updates made to portfolio-gaps.md]

### Relevance Refresh Candidates
- [list with suggested updates]

### Recommendations
1. Re-grade [A, B] — profile changes directly affect evaluations
2. Grade the N ungraded companies
3. Update why_relevant for K companies
```

No DB writes. Every recommendation is for user approval before action.

---

## Remediation Mode

When the user reviews the report and says "fix these" / "update these" / "regrade these", switch to remediation mode.

1. **Confirm scope with the user.** All findings, or a specific subset?
2. **Read `references/remediation-guide.md`** for the exact procedure per issue type.
3. **Execute fixes in priority order:**
   - Profile-driven staleness first (regrade companies / jobs whose reasoning is invalidated by profile changes)
   - Quality issues (rewrite shallow reasoning against the `quality-standards.md` bar)
   - Missing data (grade ungraded entries, fetch missing descriptions, write relevance statements)
   - Mechanical issues (fix orphaned records, update dead URLs, re-verify ATS slugs)
4. **Present each fix for user approval** before writing to the DB. Auto-fix without explicit per-fix confirmation is a rule violation.
5. **Use `references/quality-standards.md`** as the bar for rewritten reasoning.

### Coordinated family references (acceptable cross-skill paths)

Cernio's skills form a coordinated family — the research/03 hard-rule update (2026-04-18) permits path references within such a family. When remediation work requires the full procedure from another Cernio skill, the paths below are load-bearing and not an F9 stale-reference risk so long as all referenced files exist:

- Regrading companies → follow `.claude/skills/grade-companies/SKILL.md` + its references
- Regrading jobs → follow `.claude/skills/grade-jobs/SKILL.md` + its references (fetch the description first via `WebFetch` if missing)
- Resolving ATS → run `cargo run -- resolve --company "Name"`; on failure, follow `.claude/skills/resolve-portals/SKILL.md` + its `references/ats-providers.md`
- Rewriting `why_relevant` → use `references/quality-standards.md` in this skill for the quality bar

---

## When to Recommend This Skill

Trigger conditions for any agent (not only user intent):

- A profile update has materially changed skills, projects, preferences, or visa status
- A substantial period has passed since the last integrity check
- A job search session is about to start and the user wants confidence in grade currency
- A grading run just completed and quality verification is in order
- The user asks about data quality or staleness

---

## Reference Loading

**Mandatory-core — read at skill invocation every time:**

- `references/remediation-guide.md` — per-issue-type fix procedures, SQL, quality requirements
- `references/quality-standards.md` — acceptable vs unacceptable grade reasoning and fit assessment examples
- `references/profile-context.md` — how to read the profile for integrity assessment (not the profile itself)
- `references/cross-checking-guide.md` — comprehensive cross-tier consistency procedure

All four are read at invocation. The remediation guide alone without the quality standards produces generic fixes; the cross-checking guide alone without the profile-context produces decontextualised comparisons.

---

## Inviolable Rules

1. **Report mode writes nothing to the DB.** Every finding is a recommendation awaiting user approval. The one exception: step 8 writes to `profile/portfolio-gaps.md` — this is the career-coaching output, part of the skill's core purpose.
2. **Remediation mode requires per-fix user approval.** Batch auto-fix is a rule violation.
3. **The cardinal rule of grade changes:** the complete DB record + the full candidate profile are read this invocation before any grade change is drafted. A grade change drafted from memory is damage.
4. **Profile is read fresh every invocation.** No caching, no embedded snapshots, no reliance on earlier-session memory.
5. **Step 8 writes to `portfolio-gaps.md`.** A check that does not update it — even with "no new patterns this check" — breaks the career-coaching loop.
6. **`cernio format` runs in step 2.** Step 2 precedes all judgment-based reads; without it, grading agents downstream read HTML-laden descriptions.

---

## Quality Checklist

- [ ] All four reference files were read before the check began
- [ ] All files in `profile/` were read this invocation
- [ ] `cernio check` ran and its output is in the report
- [ ] `cernio format` ran in step 2 — format counts noted if any rows were touched
- [ ] Staleness detection focused on entries where the profile change is directly relevant to the specific reasoning, not blanket re-evaluation
- [ ] Grade-quality spot-check included quoted examples of acceptable and unacceptable reasoning per `quality-standards.md`
- [ ] Cross-checking pass ran per `cross-checking-guide.md`, including within-tier coherence and across-tier gradient checks
- [ ] Step 8 portfolio gap analysis sampled 10 jobs per grade tier (or the maximum available), prioritised by company grade
- [ ] `profile/portfolio-gaps.md` was updated — "Patterns from Job Evaluations", "Known Gaps", and "Current Strengths" sections as applicable, or dated "no new patterns found" note
- [ ] Missing-data queries ran and quantified results
- [ ] Recommendations prioritised — most impactful first
- [ ] Zero DB writes in report mode (except `portfolio-gaps.md`)
- [ ] In remediation mode: each fix presented for user approval before write; no batch auto-fix
