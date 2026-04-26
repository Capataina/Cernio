# Sync Summary and WIDND Format

## Table of Contents

- [Purpose](#purpose)
- [When to Read This File](#when-to-read-this-file)
- [Output Path](#output-path)
- [Summary File Template](#summary-file-template)
- [WIDND Categories (Required Coverage)](#widnd-categories-required-coverage)
- [Diff Summary Table Conventions](#diff-summary-table-conventions)
- [Evidence Block Reproduction](#evidence-block-reproduction)
- [Worked Summary Excerpt](#worked-summary-excerpt)

---

## Purpose

This file is the contract for the Phase 8 summary write. The summary file is the user-facing audit artefact — after the skill runs autonomously, the user opens this file to know exactly what changed, what was skipped, and what needs their attention. Without a substantive summary, the autonomous-run design has no audit surface.

The summary is structured per-phase so the user can trace any change to the phase that produced it. The WIDND section consolidates the structured admissions across all phases.

---

## When to Read This File

Phase 8 only. The orchestrator reads this file when composing `cernio/profile/sync-summary.md` after Phases 0-7 complete. Phase-3 and Phase-5 subagents do not read this file — their outputs feed the summary, but the orchestrator owns the final composition.

---

## Output Path

`cernio/profile/sync-summary.md`. The file is overwritten on every sync run (no history accumulation — each run produces the current snapshot). Git history captures prior versions if the user wants to compare across runs.

The filename is intentionally `sync-summary.md` not `sync-log.md` or `last-sync.md` — it is a per-run summary, not a log file. The most recent run's summary is the only one persisted.

---

## Summary File Template

```markdown
---
sync_run: <YYYY-MM-DD HH:MM:SS UTC>
skill_version: populate-from-lifeos
files_changed: <int>
files_added: <int>
files_deleted: <int>
files_unchanged: <int>
projects_synthesised: <int>
projects_skipped: <int>
agents_dispatched: <int>
agents_returned_ok: <int>
agents_failed: <int>
---

# Sync Summary — <YYYY-MM-DD HH:MM>

This file is the audit artefact for the most recent `populate-from-lifeos` run. Every change made by the skill is documented per-phase below. The WIDND section consolidates structured admissions of skipped or deferred work.

The skill ran autonomously. No mid-run user prompts occurred. If anything below is unexpected or wrong, the source for the change is named so it can be inspected directly.

---

## Diff Summary

| Action | Count | Files |
|---|---|---|
| Replaced | <int> | <comma-separated paths> |
| Added | <int> | <comma-separated paths> |
| Deleted | <int> | <comma-separated paths> |
| Unchanged | <int> | <comma-separated paths or "see Phase 2 unchanged list"> |
| Cernio-native preserved | 2 | preferences.toml, portfolio-gaps.md |

---

## Phase 0 — Pre-flight

- Working directory: <cwd>
- `gh auth status`: <output summary>
- `cernio/profile/preferences.toml` present: <yes/no>
- Pre-run modification timestamps captured for Cernio-native files:
  - `preferences.toml`: <timestamp>
  - `portfolio-gaps.md`: <timestamp>
- Reference files loaded: lifeos-source-map.md, project-synthesis-schema.md, skills-derivation-rubric.md, summary-and-widnd-format.md (this file).

## Phase 1 — README parse and allow-list

- README fetched from: `https://github.com/Capataina/Capataina/blob/main/README.md`
- Parse outcome: <success | failure with reason>
- Section counts:
  - Active Projects: <N>
  - Other Projects: <M>
  - Open Source Contributions: <K>
  - Private Projects: skipped (excluded by design)
- Project allow-list (Active + Other):
  1. <project name> — <github URL>
  2. <project name> — <github URL>
  ... (full list)
- OSS allow-list:
  1. <upstream> — <contribution URL>
  2. ...

## Phase 2 — Professional/ direct copies

| LifeOS source | Cernio target | Verdict |
|---|---|---|
| Profile/Professional/Personal.md | profile/personal.md | unchanged \| replaced \| added |
| Profile/Professional/Experience.md | profile/experience.md | ... |
| ... | ... | ... |

Total: <N> unchanged, <M> replaced, <K> added.

## Phase 3 — Per-project synthesis (parallel agents)

| Project | Output file | LifeOS files read | Agent verdict |
|---|---|---|---|
| <project> | profile/projects/<name>.md | <count> | success \| partial \| failed |
| ... | ... | ... | ... |

Total dispatched: <N>. Returned OK: <M>. Partial: <K>. Failed: <J>.

For each agent that returned, the per-source-file evidence block (path + line count + verbatim last line) is reproduced in [Evidence Blocks](#evidence-blocks-by-agent) below.

## Phase 4 — OSS aggregation

- Source folder: `LifeOS/Projects/Open Source Contributions/`
- Files read: <count>
- Output file: `profile/projects/open-source-contributions.md` (<line count>)
- Per-source-file evidence: see Evidence Blocks below.

## Phase 5 — Skills derivation

- Agent dispatched: 1 (single agent, cross-project synthesis)
- Project files consumed: <count>
- Output file: `profile/skills.md` (<line count>)
- Per-category band distribution from agent return:
  - Programming Languages: <P proficient, C comfortable, F familiar, B beginner>
  - Frameworks: ...
  - Libraries: ...
  - Engines and Runtimes: ...
  - Tools and Platforms: ...
  - Concepts and Domains: ...

## Phase 6 — Index generation

- Output file: `profile/projects/index.md` (<line count>)
- Projects indexed: <count>

## Phase 7 — Cleanup

- `profile/projects.md` (legacy flat): <deleted | absent — no action>
- `profile/volunteering.md` (legacy): <deleted | absent — no action>
- Orphan files detected: <list with reasons, or "none">
- Cernio-native preservation check:
  - `preferences.toml`: <pre-timestamp> → <post-timestamp> — <unchanged ✓ | CHANGED — BUG>
  - `portfolio-gaps.md`: <pre-timestamp> → <post-timestamp> — <unchanged ✓ | CHANGED — BUG>

## Phase 8 — Summary write

- Output file: `profile/sync-summary.md` (this file)
- Run completed: <YYYY-MM-DD HH:MM:SS UTC>

---

## What I Did Not Do

This section enumerates structured admissions per the canonical WIDND categories. Silence on a category is not equivalent to "nothing to declare for that category" — every category appears, with either a specific entry or an explicit nothing-to-declare line.

### Projects on README but absent from LifeOS

- <project name from README> — README section: <Active | Other>. LifeOS folder `Projects/<Name>/` returned 404. Per-project file not generated.
- (or: "Nothing to declare for this category — every README-listed project had a corresponding LifeOS folder.")

### Projects in LifeOS but excluded from the README

- <LifeOS folder name> — present in `LifeOS/Projects/` but not on the README's Active, Other, or OSS sections. Intentional skip per the gatekeeper rule.
- (or: "Nothing to declare for this category — every LifeOS project folder appeared in the README.")

### LifeOS files unreadable due to API errors

- `Projects/<Name>/<file>` — error: <error message>. Corresponding section in `profile/projects/<name>.md` may be incomplete.
- (or: "Nothing to declare for this category — every fetched file returned successfully.")

### Orphan files in cernio/profile/

- `<path>` — last modified <timestamp>. Suggested action: <move into schema | add to legacy-cleanup list | extend schema>.
- (or: "Nothing to declare for this category — no orphans detected.")

### Cernio-native files preserved untouched

- `preferences.toml`: confirmed unchanged (pre/post timestamps match).
- `portfolio-gaps.md`: confirmed unchanged (pre/post timestamps match).

### Agents that returned partial evidence

- Per-project agent for `<project>`: evidence block missing verbatim last lines for <list of files>. Re-dispatch recommended.
- Skills-derivation agent: evidence block complete <or specific gap noted>.
- (or: "Nothing to declare for this category — every agent returned a complete evidence block.")

### Sections of the schema with no LifeOS source evidence

- For project `<name>`: <section> in the per-project file states "no source evidence in LifeOS" — consider adding a `<corresponding LifeOS file>` to the LifeOS folder to strengthen future syncs.
- (or: "Nothing to declare for this category — every per-project file had source evidence for every schema section.")

---

## Evidence Blocks by Agent

The per-source-file evidence from each Phase 3 agent and the Phase 5 agent is reproduced verbatim below. Each block lists every LifeOS file (or per-project file, for Phase 5) consumed, with line count and verbatim last line.

### Phase 3 — Per-project agent: <project name>

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/<Name>/Overview.md | <N> | "<exact text>" |
| ... | ... | ... |

(Repeat for every per-project agent.)

### Phase 4 — OSS aggregation

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Open Source Contributions/<file> | <N> | "<exact text>" |
| ... | ... | ... |

### Phase 5 — Skills derivation

| Path | Lines | Verbatim last line |
|---|---|---|
| profile/projects/cernio.md | <N> | "<exact text>" |
| profile/projects/image-browser.md | <N> | "<exact text>" |
| ... | ... | ... |
```

---

## WIDND Categories (Required Coverage)

The seven categories in the WIDND section are non-negotiable. Every sync run's summary contains every category, even if the entry is "nothing to declare". The categories:

1. **Projects on README but absent from LifeOS** — broken upstream link from the gatekeeper.
2. **Projects in LifeOS but excluded from the README** — confirms the exclusion was deliberate, not a parsing failure.
3. **LifeOS files unreadable due to API errors** — partial-read flagging.
4. **Orphan files in cernio/profile/** — files outside the schema, surfaced for user review.
5. **Cernio-native files preserved untouched** — explicit confirmation that `preferences.toml` and `portfolio-gaps.md` are unchanged.
6. **Agents that returned partial evidence** — Tier-3 anchor enforcement: any agent whose evidence block missed verbatim last lines is named here for re-dispatch.
7. **Sections of the schema with no LifeOS source evidence** — feedback loop into LifeOS, identifying where the canonical source has gaps the user can fill on the LifeOS side.

Skipping a category — even when the answer is "nothing happened in this category" — is the silent-omission failure mode the WIDND structure exists to prevent.

---

## Diff Summary Table Conventions

The Diff Summary table at the top of the file is a high-level scan. The conventions:

- **Replaced** = the target file existed before the sync and was overwritten with new content.
- **Added** = the target file did not exist before the sync and was created.
- **Deleted** = the target file existed before the sync and was removed (legacy cleanup).
- **Unchanged** = the target file existed and the sync determined the new content was byte-identical to the existing content; no write occurred.

A file may not appear in multiple rows. Counts must reconcile against the per-phase tables.

---

## Evidence Block Reproduction

The Phase 3 and Phase 5 agents return evidence blocks listing every source file they consumed with line counts and verbatim last lines. The orchestrator reproduces these blocks verbatim in the summary's "Evidence Blocks by Agent" section.

The reproduction is non-negotiable — it is the user-facing surface of the Tier-3 evidence anchor. A summary that elides evidence blocks ("see agent return for evidence") fails the audit purpose because the agent return is ephemeral; the summary is the durable artefact.

If an evidence block is large (a project with 20+ source files), reproduce the full table — do not truncate. The summary's job is durability of the audit signal, not brevity.

---

## Worked Summary Excerpt

A condensed example showing the shape (illustrative — actual summaries are longer):

```markdown
---
sync_run: 2026-04-26 14:30:12 UTC
skill_version: populate-from-lifeos
files_changed: 4
files_added: 14
files_deleted: 2
files_unchanged: 7
projects_synthesised: 12
projects_skipped: 0
agents_dispatched: 13
agents_returned_ok: 13
agents_failed: 0
---

# Sync Summary — 2026-04-26 14:30

[...]

## Phase 1 — README parse and allow-list

- README fetched: success
- Active: 5, Other: 7, OSS: 2, Private: skipped
- Project allow-list (Active + Other):
  1. Cernio — github.com/Capataina/Cernio
  2. Image Browser — github.com/Capataina/PinterestStyleImageBrowser
  3. Aurix — github.com/Capataina/Aurix
  4. NeuroDrive — github.com/Capataina/NeuroDrive
  5. Nyquestro — github.com/Capataina/Nyquestro
  6. Vynapse — github.com/Capataina/Vynapse
  7. AsteroidsAI — github.com/Capataina/Asteroids-AI
  8. Consilium — github.com/Capataina/Consilium
  9. Chrona — github.com/Capataina/Chrona
  10. Xyntra — github.com/Capataina/Xyntra
  11. Zyphos — github.com/Capataina/Zyphos
  12. Tectra — github.com/Capataina/Tectra

[...]

## What I Did Not Do

### Projects on README but absent from LifeOS
Nothing to declare for this category — every README-listed project had a corresponding LifeOS folder.

### Projects in LifeOS but excluded from the README
- Flat Browser — present in `Projects/Flat Browser/` but not on the README. Intentional skip per the gatekeeper rule.
- LifeOS — present in `Projects/LifeOS/` but appears on the README's Private Projects section. Intentional skip.
- Claude Config — present in `Projects/Claude Config/` but appears on the README's Private Projects section. Intentional skip.

### LifeOS files unreadable due to API errors
Nothing to declare for this category — every fetched file returned successfully.

### Orphan files in cernio/profile/
Nothing to declare for this category — no orphans detected.

### Cernio-native files preserved untouched
- preferences.toml: confirmed unchanged (pre 2026-04-25 12:14:08, post 2026-04-25 12:14:08).
- portfolio-gaps.md: confirmed unchanged (pre 2026-04-21 17:42:51, post 2026-04-21 17:42:51).

### Agents that returned partial evidence
Nothing to declare for this category — every agent returned a complete evidence block.

### Sections of the schema with no LifeOS source evidence
- For project `Tectra`: "Subsystems and components" section in the per-project file states "no source evidence in LifeOS — Systems/ folder absent". Consider adding component breakdown notes to LifeOS to strengthen future syncs.
```

This excerpt shows the level of specificity required: per-project URLs, per-file timestamps for Cernio-native preservation, named exclusions with reasons, named gaps with feedback into the LifeOS source. The user reads the summary once and knows exactly what changed and what needs follow-up.
