---
name: populate-from-lifeos
description: "Synchronises Cernio's profile/ folder from LifeOS as the canonical source — copies LifeOS Profile/Professional/ files 1:1, synthesises one comprehensive file per GitHub-README-allow-listed project from every file in the matching LifeOS/Projects/ folder via parallel subagents, derives a multi-table skills.md from the synthesised projects, writes projects/index.md, and emits a sync-summary.md documenting every change. Triggers on 'sync profile from lifeos', 'pull profile from lifeos', 'refresh profile from lifeos', 'sync from lifeos', 'update profile from lifeos', 'pull from lifeos', 'reload profile from lifeos'. Never writes to LifeOS — one-way flow. Never touches preferences.toml or portfolio-gaps.md (Cernio-native files preserved untouched). Not for grading companies or jobs (use grade-companies, grade-jobs), not for scraping a single GitHub repo for project data (use the per-project synthesis here, which reads LifeOS not GitHub source). Use whenever the profile may be out of sync with LifeOS or the GitHub README, even if not named explicitly."
---

# Populate from LifeOS

> [!important] Read this entire file before starting any work
> Cernio doctrine requires reading every reference file in `references/` and every file in `profile/` (when referenced for evaluation tasks). For this skill specifically, also read `references/lifeos-source-map.md`, `references/project-synthesis-schema.md`, and `references/skills-derivation-rubric.md` end-to-end before Phase 0. The skill orchestrates parallel subagents and a final synthesis agent — incomplete reading means the dispatch prompts will be incomplete, which means the subagents work blind.

This skill maintains Cernio's `profile/` folder by treating LifeOS (`Capataina/LifeOS` on GitHub) as the canonical source of truth. The user's `Capataina/Capataina` GitHub README is the gatekeeper for which projects are in scope — only projects appearing in the README's *Active Projects*, *Other Projects*, and *Open Source Contributions* sections are synced. The *Private Projects* section is excluded by design.

The skill runs autonomously from invocation to summary write — no mid-run approval gates. Three subagent dispatches do the heavy lifting: one parallel fan-out (one agent per project for synthesis), and one single agent for skills derivation. The orchestrator handles direct copies, OSS aggregation, index generation, cleanup, and the final summary.

The non-negotiable claims that structure this skill:

1. **One-way flow.** Reads from LifeOS and the GitHub README. Writes only to `cernio/profile/`. Never writes to LifeOS.
2. **README curation is load-bearing.** Projects not on the README are deliberately excluded — the user maintains the README to choose what to showcase. Bypassing it imports work the user has chosen not to surface.
3. **Cernio-native files are off-limits.** `preferences.toml` (Rust pipeline config) and `portfolio-gaps.md` (`check-integrity` output) are never read, written, deleted, or moved by this skill.
4. **Anti-puffing.** Synthesised content describes what the project demonstrates, not what its README pitches. LifeOS folder content is the evidence; the skill structures it, never inflates it.

---

## When This Skill Triggers

Single autonomous mode. Activates on user phrases like *"sync profile from lifeos"*, *"pull profile from lifeos"*, *"refresh profile from lifeos"*, or any phrasing that asks for the Cernio profile to be updated against the LifeOS canonical source.

No mode-question. No depth selection. The skill always runs the full 8-phase workflow.

---

## Workflow

The skill runs 8 phases sequentially. Phases 3 and 5 dispatch subagents — see *Subagent Context Requirements* below for what each dispatch prompt must contain verbatim.

### Phase 0 — Pre-flight

- Read `references/lifeos-source-map.md`, `references/project-synthesis-schema.md`, `references/skills-derivation-rubric.md` in full. The skill cannot dispatch the Phase 3 and Phase 5 subagents without embedding these references verbatim — reading them is the prerequisite.
- Verify `gh` CLI authentication: `gh auth status` succeeds.
- Verify the working directory: `cernio/profile/` exists and contains `preferences.toml`. If either fails, the skill aborts with the specific failure.
- Initialise an in-memory ledger that tracks every action (file written, file skipped, agent dispatched, agent returned). This ledger feeds Phase 8.

### Phase 1 — Allow-list determination (the README parse)

- Fetch `https://github.com/Capataina/Capataina/blob/main/README.md` via `gh api repos/Capataina/Capataina/contents/README.md --jq '.content' | base64 -d`.
- Parse three sections: `## 🚀 Active Projects`, `## 🗂️ Other Projects`, `## 🌱 Open Source Contributions`.
- Extract per-row: project name (from the **bold** link text), GitHub URL.
- Build:
  - `project_allow_list` = active rows + other rows (each becomes a `projects/<name>.md` file).
  - `oss_list` = open-source rows (aggregated into a single `projects/open-source-contributions.md`).
- **Excluded:** `## 🔒 Private Projects` section — never read, never synced.
- Record in the ledger: count per section + the full project list.
- If parsing fails (sections missing, table structure unexpected): abort the skill with the specific parse failure. Do not silently skip projects — the README is the gatekeeper, and a failed parse means the gatekeeper is unreachable.

### Phase 2 — Professional/ direct copies

- Enumerate `Profile/Professional/` dynamically: `gh api repos/Capataina/LifeOS/contents/Profile/Professional --jq '[.[] | select(.name | endswith(".md")) | .name]'`. The list of files to copy is whatever LifeOS currently has — not a hardcoded inventory. New files added to `Profile/Professional/` in LifeOS are picked up automatically on the next sync; renamed or removed files stop syncing automatically.
- For each `.md` file in the listing:
  - Fetch source content: `gh api repos/Capataina/LifeOS/contents/Profile/Professional/<URL-encoded-name> --jq '.content' | base64 -d`.
  - Compute the Cernio target path: apply the path-normalisation rule from `references/lifeos-source-map.md` §"Path Normalisation" (lowercase, spaces → dashes).
  - Read the corresponding Cernio target file if it exists. Diff. If identical: record `unchanged`, skip. If different (or target absent): write the source content to the Cernio target path, record `replaced` or `added`.
- `references/lifeos-source-map.md` §"Direct Copy Mapping" lists the files currently in `Profile/Professional/` as illustrative documentation of the typical sync — not the source of truth. The source of truth is what `gh api` returns at run time.
- Files in `cernio/profile/` not produced by Phase 2 AND not in the synthesised set AND not Cernio-native (`preferences.toml`, `portfolio-gaps.md`) are detected as orphans in Phase 7 — not deleted, surfaced for user review.

### Phase 3 — Per-project synthesis (parallel subagent fan-out)

For each project in `project_allow_list`, dispatch one subagent in parallel via the Agent tool. All dispatches go in a single message — the agents are independent (each reads its own LifeOS folder and writes its own Cernio file with no shared state).

Per-agent prompt content is enumerated verbatim in *Subagent Context Requirements* below. Each agent returns:

- The output file path written.
- Per-source-file evidence: `path | line count | verbatim last line` for every file the agent read from `LifeOS/Projects/<Name>/`. This is the Tier-3 evidence anchor — partial reads cannot produce the verbatim last line.
- Any files in the LifeOS folder that could not be read (with reason).

If an agent returns a missing-folder error (project on README but no `LifeOS/Projects/<Name>/` exists): record in the ledger as a Phase-8 WIDND entry. Do not fabricate a synthesised file from the README description alone — the README is curation, not source content.

### Phase 4 — OSS aggregation

- `gh api repos/Capataina/LifeOS/contents/Projects/Open%20Source%20Contributions --jq '[.[] | .name]'` to list every file.
- Read every file in the folder.
- Synthesise a single aggregated file `cernio/profile/projects/open-source-contributions.md` covering every contribution. Per-upstream subsections in the same file — one file, not one file per upstream.
- Per-source evidence (path + line count + verbatim last line) recorded in the ledger.

### Phase 5 — Skills derivation (single subagent)

Dispatch a single subagent (not parallel — the work is cross-project synthesis that requires holistic view).

Per-agent prompt content is enumerated verbatim in *Subagent Context Requirements* below. The agent reads every `cernio/profile/projects/*.md` file just generated by Phases 3 and 4, applies the multi-table rubric in `references/skills-derivation-rubric.md`, and writes `cernio/profile/skills.md`.

The agent returns:

- The output file path written.
- Per-project evidence: `path | line count | verbatim last line` for every project file consumed.
- Per-category band counts: how many entries in Languages, Frameworks, Libraries, Engines, Tools, Concepts/Domains, with band distribution per category.

### Phase 6 — Index generation

- Read every `cernio/profile/projects/*.md` file (excluding `index.md` and `open-source-contributions.md` if treating it separately).
- Extract from each: project name, status, primary tech, one-line summary (from the file's first non-frontmatter paragraph or a designated summary field).
- Write `cernio/profile/projects/index.md` as a navigation table.
- Record file path and project count in the ledger.

### Phase 7 — Cleanup

- If `cernio/profile/projects.md` exists (legacy flat file, pre-Phase-2 reorg): delete it. Record in ledger.
- If `cernio/profile/volunteering.md` exists: delete it (OSS now lives in `projects/open-source-contributions.md`). Record in ledger.
- Detect orphans: any file in `cernio/profile/` that is not in the schema (mapped Professional/, `lifestyle-preferences.md`, `projects/`, `skills.md`, `preferences.toml`, `portfolio-gaps.md`). Record in ledger for WIDND surfacing — do **not** auto-delete.
- Verify Cernio-native preservation: `preferences.toml` and `portfolio-gaps.md` modification timestamps unchanged from before Phase 0. If either changed, the skill has a bug — abort with explicit error.

### Phase 8 — Summary write

- Write `cernio/profile/sync-summary.md` per the template in `references/summary-and-widnd-format.md`.
- The summary contains:
  - Run timestamp.
  - Per-phase summary: what happened, what changed, what was skipped, with specific counts.
  - Diff summary table: files changed, added, deleted, unchanged.
  - WIDND structured section per the canonical categories.
  - Verbatim cite of the README sections parsed in Phase 1, so the user can audit the gatekeeper.
- The summary contains all 8 phase sections plus the WIDND covering all 7 canonical categories per `references/summary-and-widnd-format.md`. A summary missing any phase section or WIDND category fails the Phase 8 contract.

---

## Subagent Context Requirements

Per Cernio doctrine: subagents have no access to the skill directory, LifeOS, the profile folder, or any external file unless the orchestrator embeds it in the dispatch prompt. Paraphrased context produces useless output.

### Per-project synthesis agent (Phase 3, dispatched once per project)

Each per-project agent prompt embeds verbatim:

- **The full content of `references/project-synthesis-schema.md`** — the per-project file template, required sections, depth standards, evidence-anchoring requirements, and anti-puffing principle. Verbatim, not summarised.
- **The project name** as it appears on the README (e.g. `Image Browser`) and as the LifeOS folder name (which may match exactly or differ by URL-encoding).
- **The output target path:** `cernio/profile/projects/<lowercase-dashes>.md` — with the exact filename calculated.
- **The fetch protocol:** `gh api repos/Capataina/LifeOS/contents/Projects/<URL-encoded-name> --jq '[.[] | .name]'` to list files; per file, `gh api repos/Capataina/LifeOS/contents/Projects/<URL-encoded-name>/<File> --jq '.content' | base64 -d`.
- **Output contract:** the agent returns the written file path AND a structured evidence block listing every LifeOS file consumed with `path | line count | verbatim last line` for each. The verbatim last line is non-substitutable — paraphrases or "(end of file)" placeholders fail this contract.
- **The anti-puffing imperative reproduced verbatim:** *"Describe what the project demonstrates from its LifeOS folder content. Do not interpolate from the GitHub README's pitch language. Do not invent technologies, integrations, or scale numbers not present in the LifeOS source. If a section the schema asks for has no source evidence, state 'no source evidence in LifeOS' for that section rather than fabricating content."*

### Skills-derivation agent (Phase 5, single dispatch)

The skills-derivation agent prompt embeds verbatim:

- **The full content of `references/skills-derivation-rubric.md`** — the multi-table approach (Languages, Frameworks, Libraries, Engines, Tools, Concepts/Domains), the dimensional rubric (depth-of-work, conceptual complexity, completion stage, cross-domain transfer, evidence specificity), the calibration anchors. Verbatim.
- **The list of every project file path** just generated in Phases 3 and 4: `cernio/profile/projects/<name>.md` for each project, plus `cernio/profile/projects/open-source-contributions.md`.
- **The output target path:** `cernio/profile/skills.md`.
- **The read protocol:** the agent reads every project file in full. No `head -N`, no `limit`, no offset. Per-file: line count and verbatim last line in the returned evidence block.
- **Output contract:** the agent returns the written file path AND a structured evidence block listing every project file consumed with `path | line count | verbatim last line`, AND per-category band distribution counts.
- **The anti-puffing reminder reproduced verbatim:** *"Proficiency reflects what the projects demonstrate, not what the user has been exposed to. A language used in one abandoned-early project is Familiar at most. A library named in passing is not a skill. Calibrate against the anchors in the rubric, not against intuition or generosity."*

The failure mode this section defends against is dispatch prompts that embed a one-paragraph summary of the schema or rubric instead of the verbatim text. Summarised-context subagents produce content shaped by the summary's emphasis rather than the rubric's full structure — observed in prior production runs across other skills' subagents.

---

## Reference Loading Instructions

**Mandatory-core** — read at skill invocation, every time. The skill cannot dispatch subagents without embedding these verbatim:

- `references/lifeos-source-map.md` — the file-handling matrix (1:1 copies, synthesis targets, untouchable Cernio-native files, orphan detection rules).
- `references/project-synthesis-schema.md` — the per-project file template and depth standards. Embedded verbatim in every Phase 3 agent prompt.
- `references/skills-derivation-rubric.md` — the multi-table proficiency rubric. Embedded verbatim in the Phase 5 agent prompt.

**Task-based conditional** — read in Phase 8 only:

- `references/summary-and-widnd-format.md` — the sync-summary.md template + WIDND categories. Read when composing the Phase 8 summary write.

---

## Inviolable Rules (Structural Constraints)

1. **One-way flow.** The skill never writes to LifeOS. All `gh api` calls against `Capataina/LifeOS` use only read endpoints (`contents/...`). Any write call is a bug — abort the skill.
2. **Cernio-native files are off-limits.** `preferences.toml` and `portfolio-gaps.md` are never read, written, deleted, or moved by this skill or any of its subagents. Phase 7 verifies their modification timestamps are unchanged from pre-Phase-0; deviation aborts the skill with explicit error.
3. **The README is the gatekeeper.** Projects synthesised = exactly the union of README Active + Other + Open Source sections. Private Projects section is never read. Projects in LifeOS but not in the README are deliberately skipped (recorded in WIDND). Projects on the README but absent from LifeOS are recorded in WIDND, never fabricated from README pitch text.
4. **No mid-run user prompts.** The skill runs autonomously from invocation to summary write. No "approve this plan" gates, no "should I delete this file" questions. The summary is the post-hoc audit.
5. **Every synthesised file evidences its sources.** Phase 3 and Phase 5 agent return blocks include verbatim last-line quotes for every source file consumed. The orchestrator's Phase 8 summary reproduces or links these evidence blocks. Partial reads cannot satisfy this rule.
6. **Lockless on the cernio/profile/ directory.** The skill assumes nothing else is writing to `cernio/profile/` during its run. Concurrent runs are not supported.

---

## Declare What Was Skipped

Close every run of this skill with a "What I Did Not Do" section in `cernio/profile/sync-summary.md`. The canonical categories:

- **Projects on README but absent from LifeOS** — name each, state which README section it appeared in. Their per-project file is not generated.
- **Projects in LifeOS but excluded from the README's Active/Other/OSS sections** — name each (these are intentional skips, but enumeration confirms the exclusion was deliberate, not a parsing failure).
- **LifeOS files unreadable due to API errors** — per file, the path attempted and the error returned. The corresponding sync target was not updated.
- **Orphan files in cernio/profile/** that match no schema slot — listed for user review. Not deleted.
- **Cernio-native files preserved untouched** — `preferences.toml` and `portfolio-gaps.md` confirmed unchanged.
- **Agents that returned partial evidence** — any Phase 3 agent whose evidence block lacked verbatim last lines for one or more source files (signals a partial read; flag for re-run).

If a category genuinely has nothing to declare, state that explicitly per category (*"Projects on README but absent from LifeOS: none — every README-listed project had a corresponding LifeOS folder"*). Silence on a category is not equivalent to "nothing to declare for that category". Per `research/17-obligations-vs-exhortations.md`, admission is preferred over fabrication — and the admission itself must be specific, not a blanket "everything ran".

---

## Quality Checklist

- [ ] **Pre-flight verified** — cite `gh auth status` output and `ls cernio/profile/` confirming `preferences.toml` is present.
- [ ] **README parsed and counts logged** — cite the URL fetched, the count of projects in each section ("Active: N, Other: M, OSS: K, Private: skipped"), and the full extracted project list reproduced in the summary.
- [ ] **Professional/ files diffed** — for each of the 11 mapped files in `references/lifeos-source-map.md` §"Direct copy mapping", cite source path, target path, and verdict (unchanged / replaced / added).
- [ ] **Per-project agents dispatched in parallel** — cite the count dispatched (one per `project_allow_list` entry), the count returned successfully, and the count failed with reason. Each agent's evidence block (per-source-file last-line quote) reproduced or linked from the summary.
- [ ] **OSS aggregated** — cite the count of files read from `Projects/Open Source Contributions/`, the line count of the resulting `projects/open-source-contributions.md`, and per-source-file evidence.
- [ ] **Skills agent dispatched and returned** — cite the count of project files the agent consumed with per-file last-line evidence, and the per-category band distribution from the agent's return.
- [ ] **Index generated** — cite `projects/index.md` line count and the project count it indexes.
- [ ] **Cleanup performed** — cite each deleted legacy file (`projects.md`, `volunteering.md` if present) and each orphan flagged (path + reason for orphan status).
- [ ] **Cernio-native files preserved** — cite `preferences.toml` and `portfolio-gaps.md` modification timestamps from before and after the run, confirming both are unchanged.
- [ ] **Summary written** — cite `profile/sync-summary.md` path, line count, and confirm it contains all 8 phase sections plus the WIDND.
- [ ] **WIDND complete** — every canonical category in the WIDND has either a specific entry or an explicit "nothing to declare for this category" line. Silence on a category fails this item.
