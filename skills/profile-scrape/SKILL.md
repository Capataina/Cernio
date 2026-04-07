# profile-scrape

Scrapes a GitHub repository and updates the user's structured profile with accurate, evidence-based entries. Use when the user provides a repo link, asks to add or refresh a project, says "scrape my repo", "check my profile against the repo", "update my projects", or wants to verify that profile entries match current repo reality. Not for company discovery, job evaluation, or ATS resolution.

---

## Why this skill exists

The profile is the source of truth for job evaluation. Stale or inaccurate entries lead to bad fit assessments — a project that has grown significantly since its last profile entry will be undersold in evaluations, and skills gained from recent work won't be matched against job requirements. Scraping the actual repo ensures the profile reflects reality, not a snapshot from whenever someone last edited it manually.

---

## Input

A GitHub repository identifier in any reasonable format — full URL, `owner/repo`, or just the repo name when the owner is already known. When the user says "scrape all my repos" or similar, iterate over their known projects.

---

## Gathering information

The goal is to understand the project well enough to write an accurate, specific profile entry. How you get there depends on what the repo offers.

**Start with the richest sources.** A repo with a well-maintained `context/` folder (particularly `context/architecture.md` and `context/notes.md`) gives you the full system shape, tech stack, design decisions, and current state in one read — you may not need much else. A repo without `context/` needs more exploration: the README, dependency manifests (`Cargo.toml`, `package.json`, `pyproject.toml`), and source structure fill the gap.

**Use the GitHub API for metadata and file listings.** The API gives you the description, language, topics, stars, last push date, and directory contents without reading individual files. Use this to orient before deciding what to read in detail.

**Read dependency manifests for ground truth on tech stack.** READMEs sometimes aspirationally list technologies. The actual dependency file tells you what the code uses. When the two disagree, trust the dependency file.

**Inspect key source files only when the above sources leave gaps.** If you can't assess technical depth or architectural decisions from the README and context folder, read a few core files — entry points, lib files, core modules. The goal is targeted assessment, not exhaustive code review.

**Gauge project status honestly.** A repo with commits from last week and an active README is "In Progress." A repo whose last commit was eight months ago with a half-finished README is "Abandoned" or "Paused." Use what you observe, not what the README claims.

---

## Updating the profile

### `profile/projects.md`

For an existing project, compare the current entry against what you found. Update any fields that have drifted — tech stack may have expanded, status may have changed, technical highlights may have grown. Preserve the user's voice where the existing entry is already good; improve where it's stale or thin.

For a new project, add an entry following the format already established in `projects.md`. Every field should be grounded in what you actually observed in the repo.

The **technical highlights** field carries the most weight. This is what differentiates "I built a web app" from "I built a lock-free order book with slab allocation and HDR latency histograms." Be specific about architecture decisions, interesting problems solved, and engineering depth demonstrated. Draw these from the code and docs, not from generic descriptions of the project type.

### `profile/skills.md`

If the project uses technologies, frameworks, or demonstrates domain knowledge not already captured in `skills.md`, add them. Assess proficiency honestly based on what the code demonstrates:

- **Proficient** — the project shows deep, repeated, confident use. Multiple modules, idiomatic patterns, non-trivial applications.
- **Comfortable** — the project shows real usage beyond tutorials. Working code that solves a real problem.
- **Familiar** — the project shows initial exploration. A single use case, potentially following guides.

When updating an existing skill's proficiency level upward, the new project should demonstrate meaningfully deeper usage than what was already recorded.

### `profile/portfolio-gaps.md`

If you notice something during the scrape — a technology the project uses that isn't listed in skills, a project that could easily be extended to cover a common market gap, or a strength that should be highlighted — note it in the relevant section.

### `context/notes.md`

If the scrape reveals something worth remembering for future sessions — a project more complete than expected, a technology choice that's unusual, or something that affects how Cernio should evaluate jobs — add a brief note.

### `profile/resume.md`

The resume is a curated artefact the user controls directly. Suggest changes conversationally if the scrape reveals something the resume should reflect, but leave the file untouched.

---

## After scraping

Report back to the user with:
1. **What you found** — brief summary of the repo state and anything notable
2. **What you changed** — which profile files were updated and the substance of each change
3. **Suggestions** — gaps spotted, inconsistencies found, or improvements worth considering

---

## Quality checklist

Before presenting results, verify:

- [ ] Every factual claim in the profile entry (tech stack, status, architecture) is grounded in something you actually read in the repo, not inferred or assumed
- [ ] Technical highlights are specific and concrete, not generic descriptions of the project type
- [ ] Proficiency levels in `skills.md` are justified by what the code demonstrates, not inflated
- [ ] The project status reflects the repo's actual activity, not its aspirational README
- [ ] If the project already had an entry, changes are proportionate — you updated what drifted without rewriting what was already accurate
- [ ] The user's voice and framing is preserved where the existing entry was already good
