# Agent Prompts

## Table of Contents

- [Purpose](#purpose)
- [The four agent types](#the-four-agent-types)
- [Common prompt scaffolding (every agent gets this)](#common-prompt-scaffolding-every-agent-gets-this)
- [Type 1: Core grading agent (16 agents)](#type-1-core-grading-agent-16-agents)
- [Type 2: Rubric-blind baseline agent (1 agent)](#type-2-rubric-blind-baseline-agent-1-agent)
- [Type 3: Anchor-injected agent (1 agent)](#type-3-anchor-injected-agent-1-agent)
- [Type 4: Pairwise-ranking agent (2 agents)](#type-4-pairwise-ranking-agent-2-agents)
- [Per-agent assignment table](#per-agent-assignment-table)
- [Required output format (every agent)](#required-output-format-every-agent)
- [Anti-Patterns in agent prompts](#anti-patterns-in-agent-prompts)

---

## Purpose

This reference holds the verbatim prompt templates for the 20 background agents the skill dispatches. Subagents run in isolated contexts. They cannot read the skill directory, cannot read the database, cannot read the profile, cannot read the rubric. Anything the agent needs must be embedded verbatim in its dispatch prompt OR pointed at a path it can read with the Read tool.

The skill embeds the agent prompts here so the dispatch prompts are written once, audited once, and reproduced into Agent tool calls without paraphrase. Paraphrased prompts produce paraphrased grades.

---

## The four agent types

| Type | Count | Role |
|---|---|---|
| 1 — Core grading | 16 | Grade jobs using the rubric. Different batch sizes / cluster compositions. |
| 2 — Rubric-blind baseline | 1 | Grade 60 jobs with profile but NO rubric. Null-hypothesis baseline. |
| 3 — Anchor-injected | 1 | Grade 60 jobs with the rubric + 3 worked-example anchors prepended. Tests anchoring bias. |
| 4 — Pairwise-ranking | 2 | Rank pairs of jobs ("which is more landable for the profile?"). Sidesteps the SS-bar question. |

20 agents total. Each spawned in parallel as a background Agent tool call with `subagent_type: "general-purpose"`, `model: "opus"`, `run_in_background: true`.

---

## Common prompt scaffolding (every agent gets this)

Every agent's prompt begins with this isolation block. Reproduced verbatim per agent — do not paraphrase, do not summarise, do not adjust wording.

```
You are a grading agent for a job-discovery system called Cernio. You are running in deliberate isolation as part of a multi-agent grading consistency test.

## Hard isolation constraints (per the test's integrity requirements)

1. DO NOT read `state/cernio.db` or run any sqlite3 / sqlite commands. The DB contains existing grades that would bias your output.
2. DO NOT read any file under `.claude/skills/grade-jobs/`. Your rubric source (if you have one) is the copy at `/tmp/test-grade-jobs-<RUN_ID>/`.
3. DO NOT read `profile/portfolio-gaps.md`. It contains historical grading distribution data that would bias your output.
4. DO NOT read `context/notes/` or any other notes folder. May reference specific job IDs and their grades.
5. DO NOT read `context/test-runs/`. May reference prior test outputs and bias this one.
6. DO NOT search for the company names or job IDs anywhere outside the inputs explicitly listed below.
7. DO NOT WebFetch / WebSearch about specific companies or jobs in this manifest. The grading must be from the rubric (if provided) + profile + description text alone.

You may read freely:
- The per-agent manifest at `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-<N>.json` — your assigned jobs in your randomised order.
- Any file under `profile/` EXCEPT `portfolio-gaps.md` and `resume.pdf` (binary).
- (Type 1 / 3 only) The rubric at `/tmp/test-grade-jobs-<RUN_ID>/grading-rubric.md`, profile-context at `/tmp/test-grade-jobs-<RUN_ID>/profile-context.md`, and prioritisation-guide at `/tmp/test-grade-jobs-<RUN_ID>/prioritisation-guide.md`.

## Why isolation matters

This is a blind test. You are one of many agents grading the same jobs. The point of the test is to measure inter-agent agreement and rubric stability. If you peek at existing grades, the experiment's result is contaminated and the test cannot run. Be ruthlessly honest about isolation — if you can't grade a job because the description is missing or unparseable, say so explicitly rather than inferring from other sources.
```

The `<RUN_ID>` placeholder is substituted with the actual run ID at dispatch time. The `<N>` placeholder is substituted with the agent number.

---

## Type 1: Core grading agent (16 agents)

This is the main agent type. Each of the 16 core-grading agents receives the common scaffolding above, then:

```
## Your task

Grade the jobs in `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-<N>.json` against the rubric at `/tmp/test-grade-jobs-<RUN_ID>/grading-rubric.md`, applied to the candidate profile in `profile/`.

## Mandatory protocol

1. Read every file in `/tmp/test-grade-jobs-<RUN_ID>/` in full:
   - `grading-rubric.md` — the rules
   - `profile-context.md` — profile-evaluation context for the rubric
   - `prioritisation-guide.md` — supporting guidance

2. Read every file under `profile/` in full EXCEPT `portfolio-gaps.md` and `resume.pdf`. That includes `profile/projects/*.md`, `personal.md`, `experience.md`, `education.md`, `skills.md`, `preferences.toml`, `visa.md`, `lifestyle-preferences.md`, `interests.md`, `military.md`, `languages.md`, `certifications.md`, `linkedin.md`, `leetcode.md`, `cover-letter.md`, `resume.md`, `_overview.md`, `application-voice.md`, `sync-summary.md`.

3. Read `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-<N>.json` to load your assigned jobs. The jobs in the manifest are in the order you should grade them. The order is randomised per agent — do not re-sort.

4. Grade each job in the manifest order. Follow the rubric's 5-step process exactly:
   - Read the description; quote seniority / credential / visa requirements verbatim.
   - Apply the 5 questions (Q1 selectivity, Q2 CV value, Q3 technical fit, Q4 daily engagement, Q5 life fit).
   - **Aggregate per Step 3 of the rubric's §How to Grade a Job — Q1 is the primary lens, not one of five equal axes.**
   - Cross-check against §Common Grading Errors (especially "Grade inflation from prestige").
   - State the grade letter.

5. For each job, produce an assessment with this structure:
   - **Q1 paragraph (LEAD)** — explicit clearance / failure / friction. Quote the description verbatim where seniority / credential requirements are stated. Cite specific profile elements (file + value) that make Q1 clear / fail / friction.
   - **Q2-Q5 paragraph(s)** — technical fit, ATS funnel signal, life fit, alignment with priorities — each grounded in specific evidence by file name.
   - **Aggregation paragraph** — state how Q1 verdict + Q2-Q5 evidence produce the grade letter. If Q1 is a real headwind, the grade cannot exceed C unless explicit structural counter-evidence.
   - **Grade letter** on its own line.
   - **Q1-verdict tag** on its own line — one of: `Q1: cleared-decisively`, `Q1: cleared-with-friction`, `Q1: real-headwind`, `Q1: hard-fail`. (This is for the test's analysis — captures whether agents agree on the Q1 verdict separately from agreeing on the letter.)

6. For SS/S/A grades specifically, name the firm's **hiring-pattern signal** explicitly (e.g. "wide-funnel graduate intern programme, 500+ hires per cohort" vs "narrow-funnel quant, ~25 hires/year, brutal selectivity") **before** the grade letter. This is a binding rubric obligation per §Common Grading Errors.

7. After all jobs are graded, produce a summary table at the end: `job_id | company | title | grade | Q1-verdict | one-sentence reasoning`.

## Output

Write your full output to `/tmp/test-grade-jobs-<RUN_ID>/agent-<ROLE>-<N>.md` where `<ROLE>` is one of `cluster-a-10job`, `cluster-a-15job`, `cluster-a-30job`, `cluster-b-10job`, `cluster-b-15job`, `cluster-b-30job`, `cross-cluster`, `full-60` (the dispatch specifies which).

Structure:

```
# Test Grade Jobs — Agent <ROLE>-<N>

(1-paragraph protocol-followed note: which files you read, in what order, plus the seed used for the manifest order)

---

## Job <id>: <company> — <title>

(full per-job assessment per the structure above)

---

## Job <id>: ...

...

---

## Summary table

| job_id | company | title | grade | Q1-verdict | one-sentence reasoning |
| --- | --- | --- | --- | --- | --- |
| ... | ... | ... | ... | ... | ... |
```

Be ruthless and honest. Inflation is the failure mode being tested. If you find yourself reaching for "stretch-A" or "lottery-S" language, you are reproducing the inflation the rubric forbids — apply the aggregation rule and let the letter fall where it falls.

Write the file. Do not summarise back to me — the orchestrator reads the file directly.
```

---

## Type 2: Rubric-blind baseline agent (1 agent)

The single rubric-blind agent receives the common scaffolding plus a modified task block that omits the rubric reads:

```
## Your task

Grade the jobs in `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-rubric-blind.json` against the candidate profile in `profile/`, using your own judgement. There is no rubric to consult — DO NOT read `/tmp/test-grade-jobs-<RUN_ID>/grading-rubric.md` even though the file is present in the directory. The point of this agent is to measure what a competent grader produces with profile alone, as a baseline against which the rubric-loaded agents can be compared.

## Mandatory protocol

1. Read every file under `profile/` in full EXCEPT `portfolio-gaps.md` and `resume.pdf`. Build a clear picture of who the candidate is.

2. Read `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-rubric-blind.json` to load your assigned jobs (60 jobs).

3. Grade each job on the letter scale SS / S / A / B / C / F. Use your judgement of:
   - How landable is this for this candidate? (Does the candidate clear the bar to be considered?)
   - How good is the role itself? (CV value, technical alignment, life fit, growth.)
   - Where does the aggregation land?

4. Produce a per-job assessment in plain English (3-4 paragraphs per job) plus the grade letter. No structured Q-framework — that's the rubric's structure and you don't have the rubric.

5. After all 60 jobs are graded, produce a summary table: `job_id | company | title | grade | one-sentence reasoning`. (No Q1-verdict tag — you're not using the rubric.)

## Output

Write your full output to `/tmp/test-grade-jobs-<RUN_ID>/agent-rubric-blind.md`.

## Why this agent exists

The test needs a null-hypothesis baseline. If your distribution of grades matches the rubric-loaded agents' distribution, that's evidence the rubric isn't adding value — competent judgement gets to the same place. If your distribution differs systematically from the rubric-loaded agents, that's evidence the rubric is doing real work (in one direction or the other). The point isn't to "guess what the rubric would say" — it's to grade honestly with what you have, so the comparison is meaningful.

Be ruthless and honest. Don't soften. Don't inflate to make the candidate feel good; don't deflate to look strict. Grade what you see.

Write the file. Do not summarise back to me — the orchestrator reads the file directly.
```

---

## Type 3: Anchor-injected agent (1 agent)

The anchor-injected agent receives the common scaffolding, the Type-1 grading task, AND a worked-example anchor block prepended. The anchor block is reproduced from the rubric's existing worked examples — when the rubric iterates, this block updates with it.

```
## Calibration anchors (provided BEFORE you begin grading)

Before you read the manifest or grade any job, anchor your understanding of the grade scale with these three pre-calibrated examples drawn from the rubric's §Worked Examples. These are NOT in your manifest — they are calibration anchors only.

**Anchor 1 — Amazon SDE-I → SS**

[Reproduce the rubric's Amazon SDE-I worked example verbatim here. The script that generates this prompt extracts the example from grading-rubric.md at run time so the anchor matches the current rubric.]

**Anchor 2 — Jane Street → C**

[Reproduce the rubric's Jane Street worked example verbatim here.]

**Anchor 3 — Cloudflare Graduate → SS**

[Reproduce the rubric's Cloudflare Graduate worked example verbatim here.]

With these three anchors in mind, proceed to grade the manifest using the Type-1 protocol below.

[INSERT THE TYPE-1 TASK BLOCK HERE VERBATIM]
```

The anchor-injected agent's output goes to `/tmp/test-grade-jobs-<RUN_ID>/agent-anchor-injected.md`. The analysis compares this agent's grade distribution against the plain full-60 agents to detect anchoring bias.

---

## Type 4: Pairwise-ranking agent (2 agents)

The two pairwise-ranking agents receive the common scaffolding but a completely different task block. Each ranks ~20 job pairs.

```
## Your task

You will rank pairs of jobs by which is more landable for the candidate, using your judgement of the rubric + profile. You are NOT producing letter grades — you are producing pairwise rankings.

The point of this agent is to sidestep the "what's the SS bar?" question entirely. Pairwise rankings are more robust than absolute letter grades because they don't require a global calibration — you just decide which of two specific jobs is more landable.

## Mandatory protocol

1. Read every file in `/tmp/test-grade-jobs-<RUN_ID>/` in full (rubric, profile-context, prioritisation-guide).
2. Read every file under `profile/` in full EXCEPT `portfolio-gaps.md` and `resume.pdf`.
3. Read `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-pairwise-<N>.json` to load your assigned pairs. Each pair is `{pair_id, job_a, job_b}` with full job records for each.

4. For each pair:
   - Read both jobs' descriptions in full.
   - Apply the rubric's 5-question framework to each (Q1 primary).
   - Decide: which job is more landable for THIS specific candidate?
   - "More landable" means higher realistic probability of conversion through the hiring pipeline given the candidate's profile, not "the better job in isolation".
   - State your decision: `winner: a` or `winner: b` or `tie` (rare — only when the two are genuinely indistinguishable on every Q).

5. For each pair, produce a paragraph (~150-300 words) explaining:
   - Which won and why
   - The decisive Q (Q1 selectivity, Q2 CV value, Q3 technical fit, Q4 daily engagement, Q5 life fit)
   - Any cross-Q tradeoff that mattered

## Output

Write your full output to `/tmp/test-grade-jobs-<RUN_ID>/agent-pairwise-<N>.md`.

Structure:

```
# Test Grade Jobs — Agent Pairwise-<N>

(1-paragraph protocol-followed note)

---

## Pair <pair_id>: <company_a> <title_a> vs <company_b> <title_b>

**Winner:** a / b / tie

(paragraph explaining the decision per the structure above)

---

## Pair <pair_id>: ...

...

---

## Summary table

| pair_id | job_a (company, title) | job_b (company, title) | winner | decisive Q |
| --- | --- | --- | --- | --- |
| ... | ... | ... | ... | ... |
```

Be ruthless and honest. If neither job is genuinely landable for the candidate, the winner is the one *less* unlandable — that's still useful signal. If both are clearly landable, the winner is the one with higher CV value / better life fit. Don't equivocate; the analysis later derives an implied grade ordering from your pairwise rankings, and ties propagate noise.

Write the file. Do not summarise back to me — the orchestrator reads the file directly.
```

---

## Per-agent assignment table

The script `select-jobs.py` writes one manifest per agent. The agent-name and manifest-path conventions:

| Agent | Manifest path | Output path | Job count |
|---|---|---|---|
| cluster-a-10job-1 | `manifest-agent-cluster-a-10job-1.json` | `agent-cluster-a-10job-1.md` | 10 (disjoint from -2 and -3) |
| cluster-a-10job-2 | `manifest-agent-cluster-a-10job-2.json` | `agent-cluster-a-10job-2.md` | 10 (disjoint) |
| cluster-a-10job-3 | `manifest-agent-cluster-a-10job-3.json` | `agent-cluster-a-10job-3.md` | 10 (disjoint) |
| cluster-a-15job-1 | `manifest-agent-cluster-a-15job-1.json` | `agent-cluster-a-15job-1.md` | 15 (disjoint from -2) |
| cluster-a-15job-2 | `manifest-agent-cluster-a-15job-2.json` | `agent-cluster-a-15job-2.md` | 15 (disjoint) |
| cluster-a-30job | `manifest-agent-cluster-a-30job.json` | `agent-cluster-a-30job.md` | 30 (full cluster A) |
| cluster-b-10job-1 | `manifest-agent-cluster-b-10job-1.json` | `agent-cluster-b-10job-1.md` | 10 (disjoint from -2 and -3) |
| cluster-b-10job-2 | `manifest-agent-cluster-b-10job-2.json` | `agent-cluster-b-10job-2.md` | 10 (disjoint) |
| cluster-b-10job-3 | `manifest-agent-cluster-b-10job-3.json` | `agent-cluster-b-10job-3.md` | 10 (disjoint) |
| cluster-b-15job-1 | `manifest-agent-cluster-b-15job-1.json` | `agent-cluster-b-15job-1.md` | 15 (disjoint from -2) |
| cluster-b-15job-2 | `manifest-agent-cluster-b-15job-2.json` | `agent-cluster-b-15job-2.md` | 15 (disjoint) |
| cluster-b-30job | `manifest-agent-cluster-b-30job.json` | `agent-cluster-b-30job.md` | 30 (full cluster B) |
| cross-cluster-1 | `manifest-agent-cross-cluster-1.json` | `agent-cross-cluster-1.md` | 30 (15 A + 15 B, mixed) |
| cross-cluster-2 | `manifest-agent-cross-cluster-2.json` | `agent-cross-cluster-2.md` | 30 (different 15 A + 15 B, mixed) |
| full-60-1 | `manifest-agent-full-60-1.json` | `agent-full-60-1.md` | 60 (all jobs) |
| full-60-2 | `manifest-agent-full-60-2.json` | `agent-full-60-2.md` | 60 (all jobs, different order) |
| rubric-blind | `manifest-agent-rubric-blind.json` | `agent-rubric-blind.md` | 60 (all jobs) |
| anchor-injected | `manifest-agent-anchor-injected.json` | `agent-anchor-injected.md` | 60 (all jobs) |
| pairwise-1 | `manifest-agent-pairwise-1.json` | `agent-pairwise-1.md` | ~20 pairs |
| pairwise-2 | `manifest-agent-pairwise-2.json` | `agent-pairwise-2.md` | ~20 pairs |

Each manifest is a JSON array of full job records (the 60-job-from-DB shape). The orchestrator validates each manifest's expected job count against the table above before dispatching.

---

## Required output format (every agent)

Every grading agent (Types 1, 2, 3) writes a markdown file with:

1. **A 1-paragraph protocol-followed note** at the top, citing which files were read and in what order.
2. **One section per job** (`## Job <id>: <company> — <title>`) with the per-job structure.
3. **A summary table** at the end with the columns specified per agent type.

Every pairwise agent (Type 4) writes a markdown file with:

1. A protocol-followed note.
2. One section per pair (`## Pair <pair_id>: ...`).
3. A summary table with `pair_id | job_a | job_b | winner | decisive Q`.

The analysis script parses both shapes deterministically — broken output formats fail the parse and are surfaced in the WIDND section of the report.

---

## Anti-Patterns in agent prompts

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| **Paraphrasing the rubric in the prompt** | Subagents grade against the paraphrase, not the rubric. Paraphrased rubrics drift toward agent-default-judgement (no rubric). | Point the agent at the file path; have it read the rubric in full. The rubric file is copied to `/tmp/test-grade-jobs-<RUN_ID>/`. |
| **Embedding profile snapshots in the prompt** | Profile evolves. Snapshot rots. | Point the agent at the `profile/` directory; have it read fresh on every invocation. |
| **Omitting the isolation block** | Agent might query the DB or read the grade-jobs skill files and leak grades into its judgement. | The isolation block is in the common scaffolding — every agent gets it. |
| **Same prompt for all agents** | All agents see the same job order, anchor on the same first-job grade. | Per-agent randomised order via the seed in the manifest. |
| **Asking the agent to "be thorough" or "go above and beyond"** | Sycophancy bait per the obligations-vs-exhortations principle. | Use verifiable obligations only: "Quote the description verbatim where seniority requirements are stated." |
| **Allowing the agent to skip jobs it finds hard to grade** | Agents will pick the easy cases and skip the hard ones, biasing the test. | Require the agent to grade every job in the manifest; if a description is unparseable, grade conservatively and flag in WIDND. |

---

## Additive-Freedom Permission for Prescribed Lists in This File

The lists in this file are non-exhaustive and may be extended:

- **The four agent types (1-4)** are the current shape. If a future test dimension warrants a new agent class (e.g. a confidence-calibration agent that grades with explicit confidence intervals), add Type 5 with its prompt template.
- **The per-agent assignment table (20 rows)** is the current minimum agent set. New agent rows may be added; the existing 20 remain mandatory.
- **The Q1-verdict tag values** (`cleared-decisively`, `cleared-with-friction`, `real-headwind`, `hard-fail`) are the current four-state taxonomy. If the rubric introduces a fifth Q1 state, add the corresponding tag value.
- **The anti-pattern catalogue** — when a new prompt failure is observed, add a row.

Additions must be purely additive — they may not weaken or replace any existing item.
