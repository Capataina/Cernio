# Agent Prompts

## Table of Contents

- [Purpose](#purpose)
- [The two agent types](#the-two-agent-types)
- [Common isolation block (every agent gets this)](#common-isolation-block-every-agent-gets-this)
- [Type 1: Core grading agent (11 agents)](#type-1-core-grading-agent-11-agents)
- [Type 2: Pairwise-ranking agent (2 agents)](#type-2-pairwise-ranking-agent-2-agents)
- [Per-agent assignment table](#per-agent-assignment-table)
- [Anti-patterns in agent prompts](#anti-patterns-in-agent-prompts)

---

## Purpose

This reference holds the verbatim prompt templates for the 13 background agents the skill dispatches. Each prompt is a **thin shell** that points at the copy of `grade-jobs/SKILL.md` and the rubric files placed in the run directory at Phase 1 Step 1.4. The agents run grade-jobs' actual workflow at runtime by reading those files — the dispatch prompts do not duplicate grade-jobs' workflow or paraphrase its steps. Paraphrased prompts produce paraphrased grades; thin-shell prompts produce grade-jobs.

The skill embeds the agent prompts here so the dispatch prompts are written once, audited once, and reproduced into Agent tool calls without paraphrase.

---

## The two agent types

| Type | Count | Role |
|---|---|---|
| 1 — Core grading | 11 | Run grade-jobs' workflow against the manifest. Different batch sizes / cluster compositions. |
| 2 — Pairwise-ranking | 2 | Rank pairs of jobs by relative landability. Different elicitation method — sidesteps absolute-letter calibration. |

13 agents total. Each spawned in parallel as a background Agent tool call with `subagent_type: "general-purpose"`, `model: "opus"`, `run_in_background: true`.

There is no rubric-blind agent. There is no anchor-injected agent. There is no cross-family agent in the standard inventory. Each of those introduces a configuration external to production grade-jobs; the resulting comparison frames one configuration as the "right" one, violating the no-bias rule. The test measures grade-jobs against grade-jobs — same workflow, same rubric, same profile reads — and reports inter-agent variance as variance, never as one agent being right and another wrong.

---

## Common isolation block (every agent gets this)

Every agent's prompt begins with this isolation block. Reproduced verbatim per agent — do not paraphrase, do not summarise, do not adjust wording.

```
You are a grading agent for a job-discovery system called Cernio. You are running in deliberate isolation as part of a multi-agent grading consistency test. Your job is to apply grade-jobs' actual workflow verbatim, then write the results to a file path the test specifies. The test will analyse what you produce; you do NOT need to know what the test measures, only that you follow grade-jobs' workflow faithfully.

## Hard isolation constraints (per the test's integrity requirements)

1. DO NOT read `state/cernio.db` or run any sqlite3 / sqlite commands. The DB contains existing grades that would bias your output.
2. DO NOT read any file under `.claude/skills/grade-jobs/`. Your grade-jobs source is the copy at `/tmp/test-grade-jobs-<RUN_ID>/`.
3. DO NOT read `profile/portfolio-gaps.md`. It contains historical grading patterns that would bias your output.
4. DO NOT read `context/notes/` or any other notes folder. May reference specific job IDs and their grades.
5. DO NOT read `context/test-runs/`. May reference prior test outputs and bias this one.
6. DO NOT search for the company names or job IDs anywhere outside the inputs explicitly listed below.
7. DO NOT WebFetch / WebSearch about specific companies or jobs in this manifest. The grading must be from the rubric + profile + description text alone.
8. DO NOT write to `state/cernio.db`. Your output goes to a file path the test specifies, not the DB.

You may read freely:
- The per-agent manifest at `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-<N>.json` — your assigned jobs in your randomised order.
- Any file under `profile/` EXCEPT `portfolio-gaps.md` and `resume.pdf` (binary).
- `/tmp/test-grade-jobs-<RUN_ID>/grade-jobs-SKILL.md` — grade-jobs' actual workflow document. Read this end-to-end before grading any job.
- `/tmp/test-grade-jobs-<RUN_ID>/grading-rubric.md`, `profile-context.md`, `prioritisation-guide.md` — grade-jobs' reference files.

## Why isolation matters

This is a blind test. You are one of many agents running the same workflow on the same jobs. The test measures how internally consistent grade-jobs' output is. If you peek at existing grades, the experiment is contaminated. Be ruthlessly honest about isolation — if you can't grade a job because the description is missing or unparseable, follow grade-jobs' semantic-reasoning or insufficient-evidence path as documented in `grade-jobs-SKILL.md`. Do not guess.

## Why thin-shell

This prompt does NOT duplicate grade-jobs' workflow. It points you at `grade-jobs-SKILL.md` and tells you to run that workflow. The reason: the test is measuring grade-jobs, not a paraphrased shadow of it. If this prompt described a different workflow, the agents would diverge from production behaviour and the test would measure that divergence rather than grade-jobs itself. Read grade-jobs-SKILL.md and apply it verbatim.
```

The `<RUN_ID>` placeholder is substituted with the actual run ID at dispatch time. The `<N>` placeholder is substituted with the agent number.

---

## Type 1: Core grading agent (11 agents)

Each of the 11 core-grading agents receives the common isolation block above, then:

```
## Your task

Run grade-jobs' workflow against the jobs in `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-<N>.json`.

## Mandatory protocol

1. Read `/tmp/test-grade-jobs-<RUN_ID>/grade-jobs-SKILL.md` end-to-end. This is grade-jobs' actual workflow document. Apply it verbatim.

2. Read the three reference files in `/tmp/test-grade-jobs-<RUN_ID>/` in full:
   - `grading-rubric.md` — the rubric
   - `profile-context.md` — how to read the profile for grading
   - `prioritisation-guide.md` — batch ordering

3. Read every file under `profile/` in full EXCEPT `portfolio-gaps.md` and `resume.pdf`. This includes profile/projects/*.md, personal.md, experience.md, education.md, skills.md, preferences.toml, visa.md, lifestyle-preferences.md, interests.md, military.md, languages.md, certifications.md, linkedin.md, leetcode.md, cover-letter.md, resume.md, _overview.md, application-voice.md, sync-summary.md.

4. Read `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-<N>.json` to load your assigned jobs. The jobs in the manifest are in the order you should grade them. The order is randomised per agent — do not re-sort.

5. Grade each job per grade-jobs-SKILL.md's workflow. Produce the structured prose fit_assessment as grade-jobs specifies — Q1, Q2, Q3a, Q3b, Q4, Q5, Verdict slots, all prose, no verdict-enum labels in slot text, JD quotes where the rubric requires, named profile projects with file references, evidence_basis classification ('jd' / 'semantic' / 'insufficient').

6. Do NOT run grade-jobs' end-of-batch Relativity Pass (the test isolation forbids reading DB grades, which Relativity Pass requires). Skip step 11 of grade-jobs' workflow and note this skip in your output's protocol note.

7. Do NOT write to `state/cernio.db`. Write your full output to `/tmp/test-grade-jobs-<RUN_ID>/agent-<ROLE>-<N>.md` where `<ROLE>` is one of `cluster-a-10job`, `cluster-a-15job`, `cluster-a-30job`, `cluster-b-10job`, `cluster-b-30job`, `cross-cluster` (the dispatch specifies which).

## Output structure

```
# Test Grade Jobs — Agent <ROLE>-<N>

(1-paragraph protocol-followed note: which files you read, in what order, plus
the seed used for the manifest order. State explicitly that you skipped the
Relativity Pass per the test isolation constraints.)

---

## Job <id>: <company> — <title>

(full per-job structured-prose fit_assessment per grade-jobs' workflow:
Q1 prose / Q2 prose / Q3a prose / Q3b prose / Q4 prose / Q5 prose / Verdict prose)

evidence_basis: <jd | semantic | insufficient>
Grade: <SS | S | A | B | C | F or NULL if insufficient_evidence>

---

## Job <id>: ...

...

---

## Summary table

| job_id | company | title | grade | evidence_basis | one-sentence reasoning |
| --- | --- | --- | --- | --- | --- |
| ... | ... | ... | ... | ... | ... |
```

Be ruthless and honest. The test measures grade-jobs' coherence and consistency. If you find yourself reaching for a verdict-enum label inside Q-slot prose, that's a violation of grade-jobs' rubric; rewrite in prose. The test will detect such violations as format-adherence failures — you are not graded on the test's measurements, but the rubric is, and the test reports back what the rubric produced.

Write the file. Do not summarise back to me — the orchestrator reads the file directly.
```

The output file ends with grade-jobs' usual structure: per-job sections + summary table. The test extracts the Q-slots and Q1-verdict-readings via prose parsing at analysis time; the agent does NOT emit a separate `Q1: <tag>` metadata line (this was the prior version's contradiction with the new rubric, removed in this iteration).

---

## Type 2: Pairwise-ranking agent (2 agents)

The two pairwise-ranking agents receive the common isolation block but a different task block. Each ranks ~20 job pairs.

```
## Your task

You will rank pairs of jobs by which is more landable for the candidate, using your judgement of the rubric + profile. You are NOT producing letter grades — you are producing pairwise rankings.

The point of this agent is to measure relative-landability stability via a different elicitation method than absolute letter grading. Pairwise rankings are more robust than absolute letter grades because they don't require a global calibration — you just decide which of two specific jobs is more landable.

## Mandatory protocol

1. Read `/tmp/test-grade-jobs-<RUN_ID>/grade-jobs-SKILL.md` end-to-end and the three reference files in `/tmp/test-grade-jobs-<RUN_ID>/` in full.
2. Read every file under `profile/` in full EXCEPT `portfolio-gaps.md` and `resume.pdf`.
3. Read `/tmp/test-grade-jobs-<RUN_ID>/manifest-agent-pairwise-<N>.json` to load your assigned pairs. Each pair is `{pair_id, job_a, job_b}` with full job records for each.
4. For each pair:
   - Read both jobs' descriptions in full.
   - Apply grade-jobs' rubric reasoning to each (Q1 primary lens for landability per the rubric).
   - Decide: which job is more landable for THIS specific candidate?
   - "More landable" means higher realistic probability of conversion through the hiring pipeline given the candidate's profile, not "the better job in isolation".
   - State your decision: `winner: a` or `winner: b` or `tie` (rare — only when the two are genuinely indistinguishable on every Q).

5. For each pair, produce a paragraph (~150-300 words) explaining:
   - Which won and why
   - The decisive Q (Q1 selectivity, Q2 motivation, Q3a stack, Q3b career-axis, Q4 domain, Q5 logistics)
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

Be ruthless and honest. If neither job is genuinely landable for the candidate, the winner is the one *less* unlandable — that's still useful signal. If both are clearly landable, the winner is the one with higher CV value / better life fit. Don't equivocate; ties propagate noise.

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
| cluster-b-30job | `manifest-agent-cluster-b-30job.json` | `agent-cluster-b-30job.md` | 30 (full cluster B) |
| cross-cluster | `manifest-agent-cross-cluster.json` | `agent-cross-cluster.md` | 30 (15 A + 15 B mixed) |
| pairwise-1 | `manifest-agent-pairwise-1.json` | `agent-pairwise-1.md` | ~20 pairs |
| pairwise-2 | `manifest-agent-pairwise-2.json` | `agent-pairwise-2.md` | ~20 pairs |

Each manifest is a JSON array of full job records (the per-agent shape). The orchestrator validates each manifest's expected job count against the table above before dispatching.

---

## Anti-patterns in agent prompts

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| **Paraphrasing the rubric in the prompt** | Subagents grade against the paraphrase, not the rubric. Paraphrased rubrics drift toward agent-default-judgement. | Point the agent at the file path; have it read the rubric in full. |
| **Paraphrasing grade-jobs' workflow in the prompt** | Same shape — subagents follow the paraphrase, not the actual workflow. The test then measures the paraphrase. | Point at `grade-jobs-SKILL.md` copied to the run directory; have the agent read it end-to-end and apply it. |
| **Embedding profile snapshots in the prompt** | Profile evolves. Snapshot rots. | Point at `profile/` directory; have the agent read fresh on every invocation. |
| **Omitting the isolation block** | Agent might query the DB or read the grade-jobs skill files and leak grades into its judgement. | The isolation block is in the common scaffolding — every agent gets it. |
| **Same prompt for all agents** | All agents see the same job order, anchor on the same first-job grade. | Per-agent randomised order via the seed in the manifest. |
| **Asking the agent to "be thorough" or "go above and beyond"** | Sycophancy bait. | Use verifiable obligations only: "Read `grade-jobs-SKILL.md` end-to-end before grading any job." |
| **Allowing the agent to skip jobs** | Agents will pick the easy cases and skip the hard ones, biasing the test. | Require grading every job in the manifest; if a description is unparseable, follow grade-jobs' semantic-reasoning or insufficient-evidence path and continue. |
| **Asking the agent to emit verdict-enum metadata tags** | Contradicts the rubric's no-verdict-enums rule. Pulls agents back toward label-thinking. | The test parses Q1 verdicts from Q1-slot prose at analysis time. Agents emit only the prose; the test extracts the signal. |

---

## Additive-Freedom Permission for Prescribed Lists in This File

The lists in this file are non-exhaustive and may be extended:

- **The two agent types (1, 2)** are the current shape. If a future test dimension warrants a new agent class (e.g. a confidence-elicitation agent that asks for explicit confidence intervals), add Type 3 with its prompt template. Existing types remain mandatory.
- **The per-agent assignment table (13 rows)** is the current minimum agent set. New agent rows may be added; the existing 13 remain mandatory.
- **The anti-pattern catalogue** — when a new prompt failure is observed, add a row.

Additions must be purely additive — they may not weaken or replace any existing item.
