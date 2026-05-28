---
name: test-grade-jobs
description: "Measures grade-jobs' own structural compliance + inter-agent consistency on a fresh job sample. Seven axes: format-adherence (Q-slots populated, JD quotes, project anchors, no verdict-enum strings), reasoning-specificity (citation density vs generic phrases), Q3a/Q3b differentiation, internal-consistency (Verdict↔Grade alignment), inter-agent variance, pairwise consistency, risk-acknowledgment density. Runs grade-jobs verbatim — agent prompts point at grade-jobs' actual workflow, no paraphrased shadow. Never asserts a specific grade is correct; never contains curated answers or per-job expectations. Persists per-axis scores to baseline.json so iterations show regression direction. Triggers on 'test the grading', 'rubric consistency test', 'grade-jobs CI', 'validate the rubric', 'measure the rubric', 'audit grading quality'. Not for re-grading the DB or comparing against external ground truth."
---

# Test Grade Jobs

> [!warning] LANE-BASED RELATIVITY REFACTOR IN-PROGRESS (2026-05-28)
> Per `context/plans/cernio-full-refactor.md §5.10`. The 7-axis structure is being restructured for lane-aware testing:
>
> - **Axis A — Lane assignment correctness**: does the agent correctly identify which lane (or lanes) a job belongs to?
> - **Axis B — Role-truth-at-hire detection**: does the rubric correctly auto-downgrade cross-function-transition roles (Solutions Architect hoping to lateral to SWE)?
> - **Axis C — Within-lane relative consistency**: do agents grade similar-pinnacle-position roles similarly WITHIN a single lane?
> - **Axis D — Cross-lane independence**: grading in HFT must not influence grading in Fintech — they're separate scales.
> - **Axis E — Sponsor-status accuracy**: does the grader respect the company.sponsors_uk flag and not surface non-sponsor companies?
> - **Axis F — Phase 2 consistency-pass effectiveness**: does Phase 2 actually correct Phase 1 drift?
> - **Axis G — Q-slot structure adherence**: no hardcoded calibration anchors in slot prose, no banned-tokens leak from rubric narration.
> 
> Baseline + iteration tracking continues; new baseline established post-refactor.

## Phased Structure (canonical, post-refactor)

### Phase 1 — Sample selection

- Pull stratified sample: N jobs across 8 lanes (e.g., 5 per lane × 8 = 40 jobs)
- Stratification: 1 SS / 1 S / 1 A / 1 B / 1 C/F per lane to test full range
- Cite job IDs + parent company lanes for reproducibility

### Phase 2 — Parallel agent grading

- Dispatch ~3 Opus agents to grade the same sample in parallel (independent prompts)
- Each agent runs grade-jobs verbatim per its current SKILL.md
- Collect 3-agent output per job for inter-agent variance measurement

### Phase 3 — Seven-axis scoring

Score the collected output:

- **Axis A — Lane assignment correctness**: does each agent identify the correct primary lane?
- **Axis B — Role-truth-at-hire detection**: does the rubric correctly auto-downgrade cross-function-transition roles?
- **Axis C — Within-lane relative consistency**: do agents grade similar-pinnacle-position roles similarly within a single lane?
- **Axis D — Cross-lane independence**: do HFT lane grades correlate with Fintech lane grades on shared agent runs? (correlation should be near zero — they're separate scales)
- **Axis E — Sponsor-status accuracy**: does the grader respect company.sponsors_uk?
- **Axis F — Phase 2 consistency-pass effectiveness**: does within-lane Phase 2 actually correct Phase 1 drift?
- **Axis G — Q-slot structure adherence**: no hardcoded calibration anchors in slot prose, no banned-rubric-narration tokens

### Phase 4 — Baseline + delta

- Persist per-axis scores to `.claude/skills/test-grade-jobs/baseline.json`
- Compare to prior baseline; cite regression direction per axis
- If composite score drops > 5 points vs prior baseline: surface as regression alert

### Phase 5 — Output

- Markdown report per-axis with cited evidence
- Baseline.json update
- No DB writes (this is measurement-only)

> [!important] Read this entire file before starting any work
> The inviolable rules and the quality checklist are at the bottom. Do not begin Phase 1 (Setup) until every reference file in this skill has been read and every file under `.claude/skills/grade-jobs/` and `profile/` (except `portfolio-gaps.md` and `resume.pdf`) has been read end-to-end.

This skill measures grade-jobs against grade-jobs' own rubric. It dispatches background Opus agents that run grade-jobs' actual workflow — same protocol, same rubric, same profile reads — against a fresh stratified sample. It then measures the resulting output across seven structural axes. None of the measurements compare the output to any external "correct answer." The test never asserts that a specific job should land at a specific grade. The test never contains curated answer sets, golden labels, or behaviour expectations. If the agents produce different results from each other, that's measurement of grade-jobs' agreement — not failure to be corrected.

The seven axes are detailed in `references/analysis-protocol.md`. Briefly: format-adherence (does the output follow the rubric's slot structure?), reasoning-specificity (citation density vs generic phrases), Q3a/Q3b differentiation (does the stack/career-axis split actually produce distinct content?), internal-consistency (does the Verdict prose support the Grade letter?), inter-agent variance (how stable are grades across agents on the same jobs?), pairwise consistency (how stable is relative ranking?), and risk-acknowledgment density (does Q3b actually name and weigh risks?).

The skill is autonomous start-to-finish per the autonomy contract for this skill ecosystem. Once invoked, it runs Setup → Dispatch → Wait → Analyse → Report without requesting user input. The user MAY type unprompted at any point; the skill reacts to the input but never pauses for it.

---

## When This Skill Triggers

Single-mode skill — no mode selection. Trigger phrases include "test the grading", "rubric consistency test", "grade-jobs CI", "validate the rubric", "measure the rubric", "audit grading quality". When the user invokes with one of these phrases, the skill runs the full 4-phase workflow against the current rubric and a fresh stratified job sample.

---

## The 4-Phase Workflow

| Phase | What runs | Output artefact |
|---|---|---|
| **1. Setup** | Read every mandatory file. Run `scripts/select-jobs.py` to pick 60 jobs stratified into clusters A and B. Write per-agent manifest JSONs (no grades) to `/tmp/test-grade-jobs-<run-id>/`. Copy rubric + profile-context + prioritisation-guide + grade-jobs SKILL.md. | `/tmp/test-grade-jobs-<run-id>/jobs-all.json`, per-agent manifests, copied rubric files |
| **2. Dispatch** | Spawn 13 background Opus agents via the Agent tool with the prompts from `references/agent-prompts.md` reproduced verbatim per agent. Job-order randomisation per agent. No grades leaked. Run all 13 in parallel as background tasks. | 13 agent IDs + per-agent expected output paths |
| **3. Wait** | Wait for completion notifications from all 13 background agents. The user MAY type unprompted; handle reactively. Do not poll, do not sleep, do not pre-check; the runtime delivers notifications. | All 13 per-agent output files present at `/tmp/test-grade-jobs-<run-id>/agent-*.md` |
| **4. Analyse + Report** | Run `scripts/analyse.py` to ingest per-agent outputs and compute the seven structural axes per `references/analysis-protocol.md`. Diff against `context/test-runs/baseline.json` if present. Write the report to `context/test-runs/test-grade-jobs-<run-id>.md`. Print chat-visible summary. Update baseline.json. | Timestamped report file + chat summary + updated baseline.json |

The four phases run in order. Phase 2's agent dispatch is the heart of the skill — the rest exists to support it (Setup) or interpret its output (Analyse).

---

## Phase 1 — Setup (Tier 3 evidence-anchored)

The setup phase is mechanical but has Tier-3 evidence obligations because each step's omission silently corrupts the test.

**Step 1.1 — Set the run ID.** Run `date +%Y-%m-%d-%H%M` and use the output as the run ID. Create the working directory `/tmp/test-grade-jobs-<run-id>/`. Evidence: cite the run-id string in the chat.

**Step 1.2 — Read the rubric being tested.** Read every file under `.claude/skills/grade-jobs/` end-to-end: the SKILL.md plus every file under `references/`. Evidence: list each filename with its line count.

**Step 1.3 — Read the candidate profile.** Read every file under `profile/` except `portfolio-gaps.md` and `resume.pdf`. Evidence: list each filename with its line count.

**Step 1.4 — Copy the rubric AND grade-jobs SKILL.md to the run directory.** Copy `grading-rubric.md`, `profile-context.md`, `prioritisation-guide.md` from `.claude/skills/grade-jobs/references/` AND `SKILL.md` from `.claude/skills/grade-jobs/` to `/tmp/test-grade-jobs-<run-id>/grade-jobs-SKILL.md`. The grade-jobs SKILL.md copy is what makes the agents run grade-jobs' actual workflow — without it, agents would have only the rubric and would have to infer the workflow. Evidence: `ls -la` output for the run directory showing the four copied files with non-zero sizes.

**Step 1.5 — Run the selection script.** Invoke `python3 .claude/skills/test-grade-jobs/scripts/select-jobs.py <run-id>`. The script writes:

- `/tmp/test-grade-jobs-<run-id>/jobs-all.json` — all 60 selected jobs with their full job-record fields (NO grades or fit_assessment)
- `/tmp/test-grade-jobs-<run-id>/cluster-a.json` — cluster A job IDs (diversity-sampled stress patterns)
- `/tmp/test-grade-jobs-<run-id>/cluster-b.json` — cluster B job IDs (diversity-sampled stability patterns)
- `/tmp/test-grade-jobs-<run-id>/coverage-matrix.json` — per-job → list-of-agent-ids the job appears in
- per-agent manifests at `/tmp/test-grade-jobs-<run-id>/manifest-agent-<n>.json` (one per agent, with the job order randomised per agent's seed)

The clusters are diversity samples, NOT expectation labels. The script picks jobs that span the rubric's likely-triggered patterns (narrow-funnel firms, hard-floor descriptions, wide-funnel grad programmes, off-stack roles, etc.) so the test measures grade-jobs across a varied surface — not so the test can assert what grade-jobs "should" produce. The script no longer writes a `trigger-cases.json` file or a `db-grades.json` file; both encoded external expectations the test must not consult.

Evidence: cite the script's stdout (which prints the run-id, job counts per cluster, and a confirmation that no grades appear in any output).

**Step 1.6 — Verify zero grade leakage in manifests.** Run `grep -E '"grade"|"fit_assessment"|"grade_reasoning"' /tmp/test-grade-jobs-<run-id>/manifest-agent-*.json` and confirm zero matches. Evidence: cite the grep command and the empty result.

If Step 1.6 finds any matches, the manifests are contaminated. Stop, fix the selection script, re-run.

---

## Phase 2 — Dispatch (Tier 3 evidence-anchored)

This phase is where the skill spends its compute budget. The full agent inventory is in `references/agent-prompts.md`. Each agent's prompt is a thin shell that points at the copied grade-jobs files — the agents run grade-jobs' actual workflow at runtime by reading those files, not by following a paraphrased version baked into the dispatch prompt.

**The 13-agent inventory:**

| # | Count | Batch | Cluster | Role |
|---|---|---|---|---|
| 1 | 3 | 10 jobs | A disjoint | Core grading, small batch on stress-pattern sample |
| 2 | 2 | 15 jobs | A disjoint | Core grading, mid batch on stress-pattern sample |
| 3 | 1 | 30 jobs | A full | Core grading, full cluster A |
| 4 | 3 | 10 jobs | B disjoint | Core grading, small batch on stability-pattern sample |
| 5 | 1 | 30 jobs | B full | Core grading, full cluster B |
| 6 | 1 | 30 jobs | Cross (15 A + 15 B) | Core grading, mixed composition |
| 7 | 2 | ~20 pairs each | Cross-cluster pairs | Pairwise-ranking |

11 core grading + 2 pairwise = **13 agents total**.

There is no rubric-blind agent (introducing external comparison violates the no-bias rule). There is no anchor-injected agent (same reason — it compares one prompt configuration against another, and the comparison implies one is "correct"). The test measures grade-jobs configured exactly as production grade-jobs is configured.

**The dispatch obligation.** For each of the 13 agents:

1. Read the appropriate prompt template from `references/agent-prompts.md`.
2. Substitute the per-agent manifest path, the per-agent output path, and the per-agent seed.
3. Invoke the Agent tool with `subagent_type: "general-purpose"`, `model: "opus"`, `run_in_background: true`.
4. Record the returned agent ID.

All 13 dispatches happen in parallel — issue them in a single response with 13 parallel Agent tool calls.

**No cost bounding.** Every agent uses Opus. No Sonnet fallback. No batch consolidation. The test's validity depends on agents behaving exactly as they would in production grading runs.

**Evidence:** cite the 13 agent IDs returned by the Agent tool, paired with the per-agent role (e.g. "agent-cluster-a-10job-1: <id>").

---

## Phase 3 — Wait

Wait for completion notifications from all 13 background agents. Do not poll. Do not sleep. Do not check on individual agents proactively — the runtime delivers notifications on completion.

**Reactive handling of mid-run user input.** If the user types unprompted while the skill is waiting (e.g. asks a question, requests a status update), respond to the input but do NOT abort the run. The agents continue in the background. The skill remains in Phase 3 until all 13 notifications arrive.

**Evidence:** when all 13 notifications have arrived, cite the per-agent output file at `/tmp/test-grade-jobs-<run-id>/agent-<role>-<n>.md` with its line count, confirming the file is non-empty.

If any agent failed to produce output (returned an error notification, or produced a file that doesn't exist / is empty), note the failure in the analysis report's Limitations section. Do not re-dispatch — the failure is itself a data point about agent reliability under this prompt shape.

---

## Phase 4 — Analyse + Report (Tier 3 evidence-anchored)

**Step 4.1 — Run the analysis script.** Invoke `python3 .claude/skills/test-grade-jobs/scripts/analyse.py <run-id>`. The script parses every per-agent output file, extracts the structured Q-slots and grade letters via prose parsing (no Q1-tag emission — the Q1 reading is INFERRED from Q1-slot prose using rule-based pattern detection), and computes per-axis scores. It writes intermediate computations to `/tmp/test-grade-jobs-<run-id>/computed-*.json` and updates `context/test-runs/baseline.json` with the new per-axis scores.

Evidence: cite the script's stdout summary listing per-axis scores AND the diff against the prior baseline if one exists.

**Step 4.2 — Compose the report.** Following `references/analysis-protocol.md`, produce a markdown report covering seven mandatory axes plus a regression-diff section if a baseline existed. Write the report to `context/test-runs/test-grade-jobs-<run-id>.md`.

Each section uses the formatting prescribed in `references/analysis-protocol.md` — tables where the output is comparison-shaped, not prose. The report never asserts that a specific grade is "correct." It reports what grade-jobs produced and how internally coherent the output was. The user — the human reader — judges whether the measured behaviour is acceptable.

Evidence: the report file exists with all seven axes plus regression-diff section, and each section cites specific evidence from the per-agent outputs.

**Step 4.3 — Print the chat summary.** A concise (~30-50 line) chat-visible summary covering: per-axis scores with regression-diff if available, inter-agent agreement headline, and the most-disagreed jobs surfaced as informational (not as failures of any specific agent).

---

## Declare What Was Skipped

Close every run with a "What I Did Not Do" section in the chat summary covering:

- **Agents that failed to produce output** — cite the agent role, the expected output path, and the failure mode (timeout, runtime error, empty file, malformed output).
- **Axes that lacked data** — if any axis cannot be computed because of a parser failure or missing field, name it and the cause.
- **Computation limitations** — if any of the seven axes was downgraded from quantitative to qualitative because of data limits, name it and state what would unblock the quantitative form.
- **Baseline diff suppressed** — if `context/test-runs/baseline.json` was missing or unparseable, the regression-diff section reports "first run; no baseline to diff against" and the run becomes the new baseline.

If nothing was skipped or limited, state so explicitly per category. Silence on a category is not equivalent to "nothing to declare for that category".

---

## Reference Loading Instructions

**Mandatory-core** — read at skill invocation, every time:

- `references/cluster-design.md` — stratification rules and selection patterns for the sample. The clusters are diversity samples, NOT expectation labels.
- `references/agent-prompts.md` — verbatim prompt templates for the two agent types (core grading, pairwise-ranking). Each template is reproduced verbatim into the matching Agent tool call.
- `references/analysis-protocol.md` — the seven mandatory axes with computation snippets and worked-example output shapes.

**Task-based conditional** — none. All three references are required on every invocation.

---

## Scripts

This skill ships two verification-anchored scripts. They produce deterministic artefacts the agent then interprets.

**Language coverage:** Python 3 (stdlib + sqlite3 only — no external dependencies). Both scripts run on macOS and Linux. Fallback for any failure: the agent surfaces the script error in the chat, attempts a one-step diagnostic, and stops the run with a Step-failure notification. Silent skipping is not permitted.

### Script inventory

| Script | Purpose | When to invoke |
|---|---|---|
| `scripts/select-jobs.py` | Stratified diversity sampling of 60 jobs (cluster A: stress patterns; cluster B: stability patterns). Writes per-agent manifests with job-order randomised per agent. No grades in any output. No trigger-case identification (the test contains no curated answers). | Phase 1 Step 1.5 |
| `scripts/analyse.py` | Ingests per-agent output markdown files. Parses Q-slots, JD quotes, project anchors, grade letters via prose parsing. Computes the seven structural axes. Updates `context/test-runs/baseline.json` for cross-run regression tracking. | Phase 4 Step 4.1 |

### Script invocation obligations

- **`select-jobs.py`**: run `python3 .claude/skills/test-grade-jobs/scripts/select-jobs.py <run-id>` once at Phase 1 Step 1.5. Cite the stdout in the chat (run-id confirmation + cluster sizes + zero-grade-leakage assertion).
- **`analyse.py`**: run `python3 .claude/skills/test-grade-jobs/scripts/analyse.py <run-id>` once at Phase 4 Step 4.1. Cite the stdout summary in the chat (per-axis scores + regression-diff if available).

Both scripts read from `state/cernio.db` and the `/tmp/test-grade-jobs-<run-id>/` working directory. Both fail loudly on missing inputs.

Script output alone is not evidence of the analysis being correct — the agent's interpretation of the script output in the report is the final artefact.

---

## Inviolable Rules (Structural Constraints)

1. **All 13 dispatched agents use Opus.** No Sonnet, no Haiku, no model downgrades. The test's validity rests on agent behaviour being identical to production grading runs.
2. **No grade leakage to agents.** Per-agent manifests contain title, url, location, remote_policy, raw_description, company_name, what_they_do. They MUST NOT contain job.grade, job.fit_assessment, company.grade, company.grade_reasoning. Phase 1 Step 1.6 grep verifies this; the run halts if leakage is detected.
3. **No mid-run user intervention.** The skill runs Setup → Dispatch → Wait → Analyse → Report autonomously. The user may type unprompted at any point; the skill reacts but never pauses for input.
4. **Agent prompts are thin shells that point at grade-jobs, NOT paraphrased grade-jobs protocols.** Subagents read the copied grade-jobs SKILL.md + rubric files at runtime and apply grade-jobs' actual workflow. The dispatch prompt does not duplicate or simplify grade-jobs' steps; it points at the workflow's source-of-truth and constrains only isolation (no DB reads, no grade leakage) and output path.
5. **The test never asserts a specific grade is "correct" for any specific job.** No reference file contains per-job expectations, golden labels, behaviour expectations, expected patterns, or curated answers. No analysis section claims "this grade is wrong" or "this should have been X." When agents disagree, that's measurement of grade-jobs' agreement — never a failure of any specific agent that the test would correct.
6. **No comparison agents inject external configurations.** The standard 13-agent inventory excludes rubric-blind, anchor-injected, and cross-family agents. Each of those introduces a configuration external to production grade-jobs and the comparison frames one configuration as the "right" one. Additive scope: a future iteration may add cross-family agents for verification purposes if framed as "what does Claude-Sonnet say compared to Claude-Opus" (descriptive), never as "this is what the grade should have been" (prescriptive).
7. **Job-order randomisation per agent.** Each agent receives the jobs in a different randomised order (different seed per agent). Sequential-anchoring drift is broken structurally, not relied on prose.
8. **Report writes to `context/test-runs/`.** Create the folder if absent. The report is human-facing — tight signal density, no agent-process language, reading-grade prose.
9. **Per-axis scores persist to `context/test-runs/baseline.json` for cross-run regression tracking.** New runs diff against the most recent baseline. The diff is reported as `+Δ` or `−Δ` per axis with no claim that any direction is "correct"; the user judges whether the direction reflects grade-jobs' improvement.

---

## Quality Checklist (Recency Anchor)

- [ ] **Phase 0 reading completed** — every file under `.claude/skills/test-grade-jobs/references/`, every file under `.claude/skills/grade-jobs/`, every file under `profile/` (except `portfolio-gaps.md` and `resume.pdf`) read end-to-end. Evidence: per-file `wc -l` cited at Phase 1 Step 1.2 and 1.3.
- [ ] **Run ID set** — `date +%Y-%m-%d-%H%M` output cited; `/tmp/test-grade-jobs-<run-id>/` created.
- [ ] **Rubric + grade-jobs SKILL.md copied** — `ls -la` on the run directory shows `grading-rubric.md`, `profile-context.md`, `prioritisation-guide.md`, AND `grade-jobs-SKILL.md` with non-zero sizes. The grade-jobs SKILL.md copy is what makes agents run grade-jobs' workflow; missing it means agents have to infer the workflow from the rubric alone.
- [ ] **Selection script ran** — stdout cited; cluster A and B sizes confirmed; no trigger-cases.json or db-grades.json written (both removed in the no-curated-answers redesign).
- [ ] **Zero grade leakage** — `grep -E '"grade"|"fit_assessment"|"grade_reasoning"' /tmp/test-grade-jobs-<run-id>/manifest-agent-*.json` returns zero matches. The grep command + empty result are cited.
- [ ] **13 agents dispatched in parallel** — 13 agent IDs cited paired with role. All `run_in_background: true`, all `model: opus`.
- [ ] **All 13 completion notifications received** — per-agent output file exists with non-zero line count; cite the `wc -l` per file.
- [ ] **Analysis script ran** — stdout summary cited; per-axis scores stated; regression-diff stated if baseline existed.
- [ ] **Report file written** — `context/test-runs/test-grade-jobs-<run-id>.md` exists with the seven axes + regression-diff section; each section is non-empty and cites specific evidence from per-agent outputs.
- [ ] **Baseline file updated** — `context/test-runs/baseline.json` contains the per-axis scores from this run; if it didn't exist before, it was created.
- [ ] **Chat summary printed** — per-axis scores, regression-diff, inter-agent agreement headline, and most-disagreed jobs (informational) all stated in chat.
- [ ] **Skipped-work declaration emitted** — every category named (failed agents, sparse axes, computation limits, missing baseline); silence on any category fails this item.
- [ ] **No grade assertions in the report** — verify by re-reading the report and confirming no sentence claims "this grade is wrong" or "this should have been X." The report describes what grade-jobs produced; it does not judge.

---

## Additive-Freedom Permission for Prescribed Lists in This File

Per the additive-freedom permission pattern. The lists in this SKILL.md that are non-exhaustive and may be extended:

- **The 4-phase workflow (Setup / Dispatch / Wait / Analyse + Report)** is the current sequential structure. If a future invocation requires a new phase (e.g. "Phase 5 — Cross-Run Drift Analysis"), add it; existing phases remain mandatory.
- **The 13-agent inventory** is the current minimum agent set. If a future axis dimension warrants an additional agent class (e.g. a confidence-elicitation agent that grades each job with explicit confidence rather than just a letter), add it; the existing 13 remain mandatory.
- **The seven mandatory axes** listed under Phase 4 are the minimum. If a future axis becomes load-bearing (e.g. "stack-concentration sensitivity"), add it; the existing seven must each be populated.
- **The trigger-phrase list in the description** is dense but not exhaustive. New trigger phrases may be added when a real user invocation surfaces a phrasing that did not activate.
- **The Inviolable Rules (1-9)** are the current structural constraints. If a future iteration surfaces a new constraint warranting Rule 10+, add it; existing rules remain inviolable.
- **The Quality Checklist items** are the current verifiable obligations. New items may be added; existing items remain mandatory.

For all six lists above, additions must be **purely additive** — they may not weaken, conditional-ise, or escape-hatch any existing item. Document additions in the next commit's message.
