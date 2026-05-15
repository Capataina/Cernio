---
name: test-grade-jobs
description: "Runs a multi-agent overlap-matrix consistency test against the current grade-jobs rubric to detect grade inflation, calibration drift, aggregation-rule fuzziness, batch-size satisficing, and trigger-case correction. Triggers on 'test the grading', 'regrade audit', 'rubric consistency test', 'grade-jobs CI', 'validate the rubric', 'blind grading test', 'rubric calibration check', 'check rubric for inflation', 'how well does the rubric work', 'audit grading quality', 'overlap test grades'. Selects a stratified 60-job sample from state/cernio.db (stress tests + stability anchors), dispatches 20 parallel Opus agents at varying batch sizes (10/15/30/60) with no DB grades leaked, then aggregates per-job grade distributions and writes a timestamped report to context/test-runs/. Not for re-grading the production DB, not for grading individual jobs, not for evaluating a single rubric change in isolation."
---

# Test Grade Jobs

> [!important] Read this entire file before starting any work
> The inviolable rules and the quality checklist are at the bottom. Do not begin Phase 1 (Setup) until every reference file in this skill has been read and every file under `.claude/skills/grade-jobs/` and `profile/` (except `portfolio-gaps.md` and `resume.pdf`) has been read end-to-end.

This skill is the rubric's CI suite. It spawns many isolated Opus agents to grade the same 60 jobs against the current `grade-jobs` rubric in varying batch-size and cluster compositions, then measures per-job grade variance, inter-agent agreement, batch-size effects, and trigger-case correction. The output is a timestamped report at `context/test-runs/test-grade-jobs-<YYYY-MM-DD-HHMM>.md` plus a chat-visible summary. Letter grades are quiet failure surfaces — wrong grades only surface when a user notices a specific job. This skill exposes systemic grading behaviour by sampling many independent reads of the same surface.

The skill is autonomous start-to-finish per the autonomy contract for this skill ecosystem. Once invoked, it runs Setup → Dispatch → Wait → Analyse → Report without requesting user input. The user MAY type unprompted at any point; the skill reacts to the input but never pauses for it.

---

## When This Skill Triggers

Single-mode skill — no mode selection. Trigger phrases include "test the grading", "regrade audit", "rubric consistency test", "grade-jobs CI", "validate the rubric". When the user invokes with one of these phrases, the skill runs the full 4-phase workflow against the current rubric and a fresh stratified job sample.

---

## The 4-Phase Workflow

| Phase | What runs | Output artefact |
|---|---|---|
| **1. Setup** | Read every mandatory file. Run `scripts/select-jobs.py` to pick 60 jobs stratified into clusters A and B. Write per-agent manifest JSONs (no grades) to `/tmp/test-grade-jobs-<run-id>/`. Copy rubric + profile-context + prioritisation-guide. | `/tmp/test-grade-jobs-<run-id>/jobs-all.json`, per-agent manifests, copied rubric files |
| **2. Dispatch** | Spawn 20 background Opus agents via the Agent tool with the prompts from `references/agent-prompts.md` reproduced verbatim per agent. Job-order randomisation per agent. No grades leaked. Run all 20 in parallel as background tasks. | 20 agent ID references + per-agent expected output paths |
| **3. Wait** | Wait for completion notifications from all 20 background agents. The user MAY type unprompted; handle reactively. Do not poll, do not sleep, do not pre-check; the runtime delivers notifications. | All 20 per-agent output files present at `/tmp/test-grade-jobs-<run-id>/agent-*.md` |
| **4. Analyse + Report** | Run `scripts/analyse.py` to ingest per-agent outputs and compute the 10 analysis sections per `references/analysis-protocol.md`. Write the report to `context/test-runs/test-grade-jobs-<run-id>.md`. Print chat-visible summary. | Timestamped report file + chat summary |

The four phases run in order. Phase 2's agent dispatch is the heart of the skill — the rest exists to support it (Setup) or interpret its output (Analyse).

---

## Phase 1 — Setup (Tier 3 evidence-anchored)

The setup phase is mechanical but has Tier-3 evidence obligations because each step's omission silently corrupts the test.

**Step 1.1 — Set the run ID.** Run `date +%Y-%m-%d-%H%M` and use the output as the run ID. Create the working directory `/tmp/test-grade-jobs-<run-id>/`. Evidence: cite the run-id string in the chat.

**Step 1.2 — Read the rubric being tested.** Read every file under `.claude/skills/grade-jobs/` end-to-end: the SKILL.md plus every file under `references/`. Evidence: list each filename with its line count.

**Step 1.3 — Read the candidate profile.** Read every file under `profile/` except `portfolio-gaps.md` and `resume.pdf`. Evidence: list each filename with its line count.

**Step 1.4 — Copy the rubric to the run directory.** Copy `grading-rubric.md`, `profile-context.md`, `prioritisation-guide.md` from `.claude/skills/grade-jobs/references/` to `/tmp/test-grade-jobs-<run-id>/`. Evidence: `ls -la` output for the run directory showing the three copied files with non-zero sizes.

**Step 1.5 — Run the selection script.** Invoke `python3 .claude/skills/test-grade-jobs/scripts/select-jobs.py <run-id>`. The script writes:

- `/tmp/test-grade-jobs-<run-id>/jobs-all.json` — all 60 selected jobs with their full job-record fields (NO grades or fit_assessment)
- `/tmp/test-grade-jobs-<run-id>/cluster-a.json` — cluster A (stress tests) job IDs
- `/tmp/test-grade-jobs-<run-id>/cluster-b.json` — cluster B (stability anchors) job IDs
- `/tmp/test-grade-jobs-<run-id>/coverage-matrix.json` — per-job → list-of-agent-ids the job appears in
- per-agent manifests at `/tmp/test-grade-jobs-<run-id>/manifest-agent-<n>.json` (one per agent, with the job order randomised per agent's seed)

Evidence: cite the script's stdout (which prints the run-id, job counts per cluster, the trigger-case identification result, and a confirmation that no grades appear in any output).

**Step 1.6 — Verify zero grade leakage in manifests.** Run `grep -E '"grade"|"fit_assessment"|"grade_reasoning"' /tmp/test-grade-jobs-<run-id>/manifest-agent-*.json` and confirm zero matches. Evidence: cite the grep command and the empty result.

If Step 1.6 finds any matches, the manifests are contaminated. Stop, fix the selection script, re-run.

---

## Phase 2 — Dispatch (Tier 3 evidence-anchored)

This phase is where the skill spends its compute budget. The full agent inventory is in `references/agent-prompts.md` and reproduced verbatim into each spawn's prompt — subagents cannot read the skill directory, the database, the rubric files, or the profile, so anything not in the dispatch prompt is invisible to them.

**The 20-agent inventory:**

| # | Count | Batch | Cluster | Role |
|---|---|---|---|---|
| 1 | 3 | 10 jobs | A disjoint | Core grading, small batch on stress tests |
| 2 | 2 | 15 jobs | A disjoint | Core grading, mid batch on stress tests |
| 3 | 1 | 30 jobs | A full | Core grading, full cluster A |
| 4 | 3 | 10 jobs | B disjoint | Core grading, small batch on stability anchors |
| 5 | 2 | 15 jobs | B disjoint | Core grading, mid batch on stability anchors |
| 6 | 1 | 30 jobs | B full | Core grading, full cluster B |
| 7 | 2 | 30 jobs | Cross (15 A + 15 B each) | Core grading, mixed composition |
| 8 | 2 | 60 jobs | Full | Core grading, maximum batch |
| 9 | 1 | 60 jobs | Full, no rubric | **Rubric-blind baseline** — sees profile + jobs, no rubric. Null hypothesis. |
| 10 | 1 | 60 jobs | Full + worked-example anchors | **Anchor-injected** — sees rubric worked examples prepended. Tests anchoring bias. |
| 11 | 2 | ~20 pairs each | Cross-cluster pairs | **Pairwise-ranking** — ranks pairs, doesn't grade. Sidesteps SS-bar question. |

16 core grading + 1 rubric-blind + 1 anchor-injected + 2 pairwise = **20 agents total**.

**The dispatch obligation.** For each of the 20 agents:

1. Read the appropriate prompt template from `references/agent-prompts.md`.
2. Substitute the per-agent manifest path, the per-agent output path, and the per-agent seed.
3. Invoke the Agent tool with `subagent_type: "general-purpose"`, `model: "opus"`, `run_in_background: true`.
4. Record the returned agent ID.

All 20 dispatches happen in parallel — issue them in a single response with 20 parallel Agent tool calls.

**No cost bounding.** Every agent uses Opus. No Sonnet fallback. No batch consolidation. The test's validity depends on agents behaving exactly as they would in production grading runs.

**Evidence:** cite the 20 agent IDs returned by the Agent tool, paired with the per-agent role (e.g. "agent-cluster-a-10job-1: <id>").

---

## Phase 3 — Wait

Wait for completion notifications from all 20 background agents. Do not poll. Do not sleep. Do not check on individual agents proactively — the runtime delivers notifications on completion.

**Reactive handling of mid-run user input.** If the user types unprompted while the skill is waiting (e.g. asks a question, requests a status update), respond to the input but do NOT abort the run. The agents continue in the background. The skill remains in Phase 3 until all 20 notifications arrive.

**Evidence:** when all 20 notifications have arrived, cite the per-agent output file at `/tmp/test-grade-jobs-<run-id>/agent-<role>-<n>.md` with its line count, confirming the file is non-empty.

If any agent failed to produce output (returned an error notification, or produced a file that doesn't exist / is empty), note the failure in the analysis report's Limitations section. Do not re-dispatch — the failure is itself a data point about agent reliability under this prompt shape.

---

## Phase 4 — Analyse + Report (Tier 3 evidence-anchored)

**Step 4.1 — Run the analysis script.** Invoke `python3 .claude/skills/test-grade-jobs/scripts/analyse.py <run-id>`. The script parses every per-agent output file, extracts grades + Q1-verdicts + hiring-pattern signals, and emits intermediate computations to `/tmp/test-grade-jobs-<run-id>/computed-*.json`.

Evidence: cite the script's stdout summary listing the number of grades parsed, the number of agents whose output was successfully ingested, and the number of unique jobs covered.

**Step 4.2 — Compose the report.** Following `references/analysis-protocol.md`, produce a markdown report covering 10 mandatory sections (per-job grade distribution, inter-agent agreement, batch-size effect, cluster-position effect, trigger-case correction, Q1-verdict consistency, rubric-blind baseline, anchor-injection, pairwise-ranking, verdict + recommendations). Write the report to `context/test-runs/test-grade-jobs-<run-id>.md`.

Each section uses the formatting prescribed in `references/analysis-protocol.md` — tables where the output is comparison-shaped, not prose.

Evidence: the report file exists with all 10 sections, and each section cites specific evidence (job IDs, agent counts, computed values from `computed-*.json`).

**Step 4.3 — Print the chat summary.** A concise (~30-50 line) chat-visible summary covering: distribution shift vs DB, inter-agent agreement headline, trigger-case correction rate, rubric-blind comparison, and the top 3-5 recommendations.

---

## Declare What Was Skipped

Close every run with a "What I Did Not Do" section in the chat summary covering:

- **Agents that failed to produce output** — cite the agent role, the expected output path, and the failure mode (timeout, runtime error, empty file, malformed output).
- **Analysis sections that lacked data** — if cluster B was structurally smaller than cluster A this run, or if too few jobs were graded by multiple agents to compute Cohen's-kappa, name the section and the cause.
- **Trigger-case-identification limitations** — if `portfolio-gaps.md` patterns are sparse and the trigger-case set is smaller than expected, name the cause and the actual count used.
- **Computation limitations** — if any of the 10 mandatory analysis sections was downgraded from quantitative to qualitative because of data limits, name it and state what would unblock the quantitative form.

If nothing was skipped or limited, state so explicitly per category. Silence on a category is not equivalent to "nothing to declare for that category".

---

## Reference Loading Instructions

**Mandatory-core** — read at skill invocation, every time:

- `references/cluster-design.md` — stratification rules, selection SQL templates, trigger-case heuristics. Read before invoking `scripts/select-jobs.py` so the script's output is interpretable.
- `references/agent-prompts.md` — verbatim prompt templates for all four agent types (core grading, rubric-blind baseline, anchor-injected, pairwise-ranking). Each template is reproduced verbatim into the matching Agent tool call.
- `references/analysis-protocol.md` — the 10 mandatory analysis sections with computation snippets and worked-example output shapes.

**Task-based conditional** — none. All three references are required on every invocation; the skill has no conditional branches that would change which reference is read.

---

## Scripts

This skill ships two verification-anchored scripts. They produce deterministic artefacts the agent then interprets. They do not replace the agent's reasoning about agent dispatch or report composition.

**Language coverage:** Python 3 (stdlib + sqlite3 only — no external dependencies). Both scripts run on macOS and Linux. Fallback for any failure: the agent surfaces the script error in the chat, attempts a one-step diagnostic (e.g. check that `state/cernio.db` exists and the schema matches), and stops the run with a Step-failure notification. Silent skipping is not permitted.

### Script inventory

| Script | Purpose | When to invoke |
|---|---|---|
| `scripts/select-jobs.py` | Stratified selection of 60 jobs (cluster A: stress tests; cluster B: stability anchors). Writes per-agent manifests with job-order randomised per agent. No grades in any output. | Phase 1 Step 1.5 |
| `scripts/analyse.py` | Ingests per-agent output markdown files. Parses grades, Q1-verdicts, hiring-pattern signals. Computes inter-agent agreement, batch-size effect, cluster-position effect, trigger-case correction, Q1-verdict consistency, rubric-blind comparison, anchor-injection effect, pairwise-ranking consistency. Writes intermediate JSONs the report consumes. | Phase 4 Step 4.1 |

### Script invocation obligations

- **`select-jobs.py`**: run `python3 .claude/skills/test-grade-jobs/scripts/select-jobs.py <run-id>` once at Phase 1 Step 1.5. Cite the stdout in the chat (run-id confirmation + cluster sizes + trigger-case count + zero-grade-leakage assertion).
- **`analyse.py`**: run `python3 .claude/skills/test-grade-jobs/scripts/analyse.py <run-id>` once at Phase 4 Step 4.1. Cite the stdout summary in the chat (grades parsed + agents ingested + unique jobs covered).

Both scripts read from `state/cernio.db` and the `/tmp/test-grade-jobs-<run-id>/` working directory. Both fail loudly on missing inputs; the agent surfaces the failure rather than papering over it.

Script output alone is not evidence of the analysis being correct — the agent's interpretation of the script output in the report is the final artefact.

---

## Inviolable Rules (Structural Constraints)

1. **All 20 dispatched agents use Opus.** No Sonnet, no Haiku, no model downgrades. The test's validity rests on agent behaviour being identical to production grading runs.
2. **No grade leakage to agents.** Per-agent manifests contain title, url, location, remote_policy, raw_description, company_name, what_they_do. They MUST NOT contain job.grade, job.fit_assessment, job.fit_score, company.grade, company.grade_reasoning. Phase 1 Step 1.6 grep verifies this; the run halts if leakage is detected.
3. **No mid-run user intervention.** The skill runs Setup → Dispatch → Wait → Analyse → Report autonomously. The user may type unprompted at any point; the skill reacts but never pauses for input. No "ask the user to confirm", no "request approval", no "pause to verify". Per the autonomy contract for this ecosystem.
4. **Every dispatched agent's prompt embeds the rubric files verbatim, not as paraphrase.** Subagents cannot read the skill directory or any external files; the prompt must contain the full text. See `references/agent-prompts.md`.
5. **Job-order randomisation per agent.** Each agent receives the jobs in a different randomised order (different seed per agent). Sequential-anchoring drift is broken structurally, not relied on prose.
6. **Report writes to `context/test-runs/`.** Create the folder if absent. The report is human-facing (the user reads it on its own merits) so use tight signal density, no agent-process language, reading-grade prose; per the audience-routing principle.

---

## Quality Checklist (Recency Anchor)

- [ ] **Phase 0 reading completed** — every file under `.claude/skills/test-grade-jobs/references/`, every file under `.claude/skills/grade-jobs/`, every file under `profile/` (except `portfolio-gaps.md` and `resume.pdf`) read end-to-end. Evidence: per-file `wc -l` cited at Phase 1 Step 1.2 and 1.3.
- [ ] **Run ID set** — `date +%Y-%m-%d-%H%M` output cited; `/tmp/test-grade-jobs-<run-id>/` created.
- [ ] **Rubric copied** — `ls -la` on the run directory shows `grading-rubric.md`, `profile-context.md`, `prioritisation-guide.md` with non-zero sizes.
- [ ] **Selection script ran** — stdout cited; cluster A and B sizes confirmed; trigger-case count cited.
- [ ] **Zero grade leakage** — `grep -E '"grade"|"fit_assessment"|"grade_reasoning"' /tmp/test-grade-jobs-<run-id>/manifest-agent-*.json` returns zero matches. The grep command + empty result are cited.
- [ ] **20 agents dispatched in parallel** — 20 agent IDs cited paired with role. All `run_in_background: true`, all `model: opus`.
- [ ] **All 20 completion notifications received** — per-agent output file at `/tmp/test-grade-jobs-<run-id>/agent-<role>-<n>.md` exists with non-zero line count; cite the `wc -l` per file.
- [ ] **Analysis script ran** — stdout summary cited; grades parsed, agents ingested, jobs covered all stated.
- [ ] **Report file written** — `context/test-runs/test-grade-jobs-<run-id>.md` exists with all 10 mandatory sections; each section is non-empty and cites specific evidence.
- [ ] **Chat summary printed** — distribution shift, inter-agent agreement, trigger-case correction rate, rubric-blind comparison, top recommendations all stated in chat.
- [ ] **Skipped-work declaration emitted** — every category named (failed agents, sparse analysis sections, trigger-case limits, computation limits); silence on any category fails this item.

---

## Additive-Freedom Permission for Prescribed Lists in This File

Per the additive-freedom permission pattern. The lists in this SKILL.md that are non-exhaustive and may be extended:

- **The 4-phase workflow (Setup / Dispatch / Wait / Analyse + Report)** is the current sequential structure. If a future invocation requires a new phase (e.g. a "Phase 5 — Drift Check" comparing the current run against a prior run's report), add it; existing phases remain mandatory.
- **The 20-agent inventory** is the current minimum agent set. If a future analysis dimension warrants an additional agent class (e.g. a confidence-calibration agent that grades each job with explicit confidence rather than just a letter), add it; the existing 20 remain mandatory.
- **The 10 mandatory analysis sections** listed under Phase 4 are the minimum. If the data this run produces warrants additional sections (e.g. a "Stack-Concentration Sensitivity" section if the run included substantial portfolio-shape variation), add them; the existing 10 must each be populated.
- **The trigger-phrase list in the description** is dense but not exhaustive. New trigger phrases may be added when a real user invocation surfaces a phrasing that did not activate; the existing phrases remain.
- **The Inviolable Rules (1-6)** are the current structural constraints. If a future iteration surfaces a new constraint warranting Rule 7+, add it; existing rules remain inviolable.

For all five lists above, additions must be **purely additive** — they may not weaken, conditional-ise, or escape-hatch any existing item. Document additions in the next commit's message.
