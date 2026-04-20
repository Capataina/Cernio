---
name: profile-scrape
description: "Scrapes a GitHub repository and updates the user's structured profile with accurate, evidence-based entries grounded in what the code actually demonstrates, not what the README aspirationally claims. Reads the repo's `context/`, README, dependency manifests (`Cargo.toml`, `package.json`, `pyproject.toml`), and key source files; updates `profile/projects.md`, `profile/skills.md`, and `profile/portfolio-gaps.md`; notes insights in `context/notes.md`. Invoke when the user provides a GitHub repo link, says 'scrape my repo', 'update my projects', 'refresh the profile', 'check the profile against the code', 'scrape all my repos', 'add this project', or wants to verify that profile entries still match repo reality. Not for company discovery (use discover-companies), job evaluation (use grade-jobs), or ATS resolution (use populate-db / resolve-portals). Use this skill whenever a repo has evolved past its current profile entry or a new project needs an evidence-based entry added."
---

# Profile Scrape

The profile is the source of truth for every fit assessment, every company grade, and every application answer. Stale or inaccurate entries cause silent evaluation errors — a project that has grown since its last profile entry gets undersold in grading, and skills gained from recent work never get matched against job requirements. This skill keeps the profile synced with repo reality.

Every entry written by this skill is grounded in something observed in the repo — dependency manifests, source code, context docs, commit activity. Nothing is paraphrased from the README's aspirational language; technical highlights cite specific techniques, data structures, algorithms, or architecture decisions. The depth test applied to every highlight is: "Could this sentence describe hundreds of other projects, or is it specific to this one?"

---

## Mandatory Reads Before Scraping

| # | What | Why evidence-anchored |
|---|---|---|
| 1 | **Every file in `profile/`** | Without the current state of the profile, you cannot tell what has changed or what already has a good entry worth preserving. |
| 2 | **`references/scraping-methodology.md`** | Defines the source-priority order (context → README → manifests → code), the depth-assessment test, the status-honesty rule, and the multi-repo iteration pattern. |
| 3 | **`references/profile-format.md`** | Defines the exact format and quality standard for profile entries — what good technical highlights look like vs generic filler, proficiency-level definitions, which files to update and which to never touch. |

When delegating scraping to parallel subagents, the full text of both reference files plus the current content of `profile/projects.md` and `profile/skills.md` must be embedded verbatim in each agent's prompt. Agents cannot read the skill's references or the profile directly.

---

## Input

A GitHub repository identifier in any format — full URL, `owner/repo`, bare repo name when the owner is known. Plural inputs ("scrape all my repos") iterate over the known project list in `profile/projects.md`.

---

## Gathering Information

The information-gathering approach is calibrated to what the repo offers. A repo with a well-maintained `context/` folder (particularly `architecture.md` and `notes.md`) gives you system shape, stack, design decisions, and current state in one read — you may not need to read much source. A repo without `context/` needs more exploration: README, dependency manifests, and source structure fill the gap.

**Source-priority order:**

1. **`context/`** if it exists — richest source of architecture and decision history
2. **`README.md`** — framing, scope, purpose (but treat tech claims skeptically)
3. **GitHub API metadata** — description, language, topics, last `pushed_at`, default branch
4. **Dependency manifests** — `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, etc. Ground truth for tech stack when the README disagrees.
5. **Key source files** — entry points, core modules, lib files. Targeted assessment, not exhaustive review.
6. **Commit activity** — `pushed_at` and recent commit messages calibrate the project status claim.

**Status honesty.** Recent commits + active README → *In Progress*. Last commit > 8 months old and half-finished README → *Paused* or *Abandoned*. Use what the repo shows, not what the README claims.

---

## Updating the Profile

### `profile/projects.md`

For an existing project, diff what you found against what is recorded. Update fields that have drifted — expanded stack, changed status, grown technical highlights. Preserve existing voice where the entry is already good; improve where it is stale or thin.

For a new project, add an entry matching the format in `projects.md`. Every field is grounded in observed evidence.

**Technical highlights are the load-bearing field.** This is what differentiates "I built a web app" from "I built a lock-free order book with slab allocation and HDR latency histograms." Each highlight names a specific architectural decision, interesting problem, or engineering technique — drawn from the code and docs, not from generic descriptions of the project type. Aspirational features from the README that do not exist in the code do not belong.

### `profile/skills.md`

If the project uses technologies, frameworks, or demonstrates domain knowledge not already captured, add them. Proficiency levels — per `references/profile-format.md` — are calibrated against what the code actually demonstrates:

- **Proficient** — deep, repeated, confident use across multiple modules; idiomatic patterns; non-trivial applications
- **Comfortable** — real usage beyond tutorials; working code solving a real problem
- **Familiar** — initial exploration; single use case, possibly tutorial-following

Upward revisions of an existing skill require meaningfully deeper usage than what was already recorded. A second small use of the same library does not move Comfortable to Proficient.

### `profile/portfolio-gaps.md`

If the scrape surfaces a strength worth highlighting or a gap-closure opportunity, note it in the relevant section. Example: the project uses Kubernetes, which closed a named gap — move the entry from "Gaps" to "Confirmed strengths" with the specific project as evidence.

### `context/notes.md`

If the scrape reveals something worth remembering for future sessions — a project more complete than expected, an unusual technology choice, something that affects how Cernio should evaluate jobs — add a brief note. This is optional but valuable when the insight would otherwise decay.

### `profile/resume.md`

The resume is user-controlled. Do not edit it. Suggest changes conversationally if the scrape reveals something the resume should reflect.

---

## Reporting

After scraping, report to the user:

1. **What was found** — brief summary of the repo state and anything notable (status mismatch, stack divergence from README, newly-visible strengths)
2. **What was changed** — which profile files were updated and the substance of each change
3. **What was not changed and why** — entries that looked stale on first glance but checked out on closer look; upward proficiency revisions considered and rejected; gaps that the project nearly closes but does not quite
4. **What I did not do** — repos in a multi-repo scrape that were skipped (with reason: already current, archived, inaccessible), proficiency revisions deferred pending a second supporting project, README claims contradicted by manifest that were preserved as a flag for the user rather than silently changed. If nothing was deferred or skipped, say so explicitly — silence is not the same as nothing-to-declare.
5. **Suggestions** — gaps spotted, resume items worth considering, improvements deferred because they exceed scraping scope

---

## Inviolable Rules

1. **Every factual claim is grounded in observed evidence.** If the claim cannot be traced back to a specific file or API response, it does not belong in the profile.
2. **Technical highlights describe what is BUILT, not what is PLANNED.** README roadmaps do not produce highlights.
3. **The dependency manifest is authoritative for tech stack.** When the README and the manifest disagree, trust the manifest.
4. **Project status is observational.** `pushed_at` + recent commits determine status, not the README's framing.
5. **`profile/resume.md` is untouched.** The resume is curated by the user; suggest changes only.
6. **The profile is read fresh every invocation.** Never embed profile content in this skill or its references. The profile is a living document.
7. **Changes to existing entries are proportionate.** Do not rewrite what is already accurate — update what drifted, preserve the user's voice where it is good.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. A checklist item that cannot be evidenced in the agent's output is either unmet and surfaced under "What I did not do" in the report, or the skill has not finished.

- [ ] **Both reference files read fresh this invocation** — cite the tool call for `scraping-methodology.md` and `profile-format.md`.
- [ ] **Every file in `profile/` read fresh this invocation** — cite the tool call for each file under `profile/`.
- [ ] **Dependency manifest(s) read** — cite the exact file path (`Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, etc.) and the dependency list extracted. When the manifest and the README disagree on stack, the manifest wins; the disagreement is named in the report.
- [ ] **GitHub API metadata fetched** — cite the WebFetch or `gh api` call that returned `pushed_at`, the primary language, and the top-level directory listing. "Inferred from source code" does not substitute.
- [ ] **Every factual claim in new / updated entries cites a source** — file path, manifest line, API field, or commit reference. Assertions without source citations fail this item.
- [ ] **Technical highlights pass the depth test, per-highlight** — for each new or updated highlight sentence, name the specific technique, structure, algorithm, or decision it cites; "uses modern patterns"-class phrasings fail and are rewritten.
- [ ] **Built vs planned is distinguished** — every highlight describes something visible in the repo right now; no roadmap items claimed as present. Cite the specific file or module where the built thing lives.
- [ ] **Proficiency upgrades (if any) cite the new evidence** — the second or third project demonstrating deeper usage is named, with the specific patterns that justify the upgrade. Upgrades without new evidence fail this item.
- [ ] **Project status matches repo signal** — cite `pushed_at` or the most recent commit date and the decision that status maps to (Completed / In Progress / In Progress (not enough interest) / Paused / Abandoned).
- [ ] **Existing entries preserved where already good** — diff between old and new entry is surfaced in the report; unchanged sections are explicitly unchanged, not silently rewritten.
- [ ] **Cross-file updates are consistent** — new skills added to `skills.md` are referenced in the relevant `projects.md` entry; gap closures noted in `portfolio-gaps.md` name the project; `context/notes.md` entries (if any) cite the scrape's findings.
- [ ] **`profile/resume.md` untouched** — confirmed by checking the file's modification time (or, if in the same session, by naming which files were edited and noting `resume.md` is not one of them).
- [ ] **Report includes "What I did not do" section** — either names specific deferrals or states "no deferrals or skipped work" explicitly; absence is not permitted.
