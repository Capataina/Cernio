# Profile System Decisions

The profile is the source of truth for who the user is, what they can do, and what they're looking for. Every grading skill, every fit assessment, every career-coaching observation routes through it.

This note captures the *why* behind the profile system's current shape. The mechanics — what files exist where, what populates them, what reads them — are in `architecture.md` and `.claude/skills/populate-from-lifeos/SKILL.md`.

---

## Read on every startup

Claude reads every file in `profile/` at the start of every session, no exceptions. Without the profile loaded, Claude cannot assess job fit, recommend companies, or give career advice accurately. The user never has to re-explain their background.

This is enforced in three places: the project `CLAUDE.md` Mandatory Startup Behaviour section, the global `CLAUDE.md` skill execution protocol, and every individual skill's mandatory-read block.

---

## LifeOS is canonical, Cernio is downstream

The `profile/` folder is not authored inside Cernio. It is **synced from LifeOS** — the user's private Obsidian vault at `Capataina/LifeOS` — via the `populate-from-lifeos` skill.

**Data flow is one-way.** LifeOS is the canonical source. The skill reads from `LifeOS/Profile/Professional/` and `LifeOS/Projects/<allow-listed>/`, writes to `cernio/profile/`, and never writes back. Editing a profile file directly inside Cernio without updating LifeOS is a path to silent drift.

**The GitHub README is the project allow-list gatekeeper.** `populate-from-lifeos` parses the user's `Capataina/Capataina` profile README's *Active Projects*, *Other Projects*, and *Open Source Contributions* sections to determine which projects get synthesised. The *Private Projects* section is excluded by design — the README is the curation authority. A LifeOS folder that exists for a private or in-flight project will be deliberately skipped.

**Why this design.** The previous architecture (commit history pre-`9f19f73`) had a `profile-scrape` skill that scraped individual GitHub repos for profile data. That responsibility now sits upstream in LifeOS's `extract-project` skill. Keeping both would duplicate the work and let the two skills drift apart. Cernio is now strictly the consumer.

---

## The schema (post-2026-04-26 migration)

Three categories of files in `profile/`:

| Category | Files | Authored by |
|----------|-------|-------------|
| **Direct copy** from LifeOS | `personal.md`, `experience.md`, `education.md`, `interests.md`, `visa.md`, `military.md`, `languages.md`, `certifications.md`, `lifestyle-preferences.md`, `resume.md`, `cover-letter.md` (11 files, dynamic) | LifeOS `Profile/Professional/` — synced 1:1 with filename normalisation |
| **Synthesised** by `populate-from-lifeos` | `projects/<name>.md` (one per README-listed project), `projects/index.md`, `projects/open-source-contributions.md`, `skills.md`, `sync-summary.md` | Per-project subagents (Phase 3), OSS aggregation (Phase 4), skills-derivation subagent (Phase 5), index generator (Phase 6), summary writer (Phase 8) |
| **Cernio-native, off-limits** | `preferences.toml`, `portfolio-gaps.md` | `preferences.toml` is hand-edited and machine-read by `src/config.rs`; `portfolio-gaps.md` is written by the `check-integrity` skill |

**Cernio-native files are never touched** by the sync skill or its subagents. Phase 7 of `populate-from-lifeos` verifies preservation by capturing pre-run modification timestamps and comparing them post-run; deviation aborts with explicit error. This is the safety mechanism that keeps the Rust pipeline config (`preferences.toml`) and the career-coaching output (`portfolio-gaps.md`) safe from accidental overwrite by an LLM agent.

The `preferences.toml` file additionally has a build-time integrity guard at `tests/preferences_integrity.rs` (21 tests) covering required sections, valid grade letters, per-provider location-filter coverage, UK-pattern presence, and the `every_supported_ats_provider_has_a_location_subtable` invariant that catches missing subtables when a new ATS provider is added in `src/ats/`.

---

## Per-project files use status, not tiers

Each `profile/projects/<name>.md` has a `status` frontmatter field with one of: `active | dormant | paused | abandoned`.

**The Tier system (Flagship / Notable / Minor) is retired.** It was introduced session 5 (2026-04-09) when there was a single flat `projects.md` file and grading agents needed to know which projects represented the candidate's strongest work. With per-project files, each one is its own canonical home and carries its own status. Active and complete projects are primary evidence in grading. Paused and dormant projects are secondary evidence. Abandoned projects are background context — they show what was tried, but the grader does not weight them against role requirements the same way as active work.

**LifeOS uses richer status vocabulary** than the schema's enum (`scaffold`, `active-status-undecided`, `#dormant`, `#skeleton`). The Phase 3 subagents map LifeOS values to the schema enum and surface every mismatch in `sync-summary.md`'s WIDND. If the same mismatch keeps appearing across runs, the schema enum should be expanded rather than continuing to map.

---

## Describe what's built, not what's planned

Profile entries lead with what the code actually demonstrates today. Future goals belong in the per-project file's *Direction (in-flight, not wishlist)* and *Planned* sections, but technical highlights and skills derivations come from implemented reality.

This is the **anti-puffing principle** baked into both the per-project synthesis schema and the skills derivation rubric:

- *"Describe what the project demonstrates from its LifeOS folder content. Do not interpolate from the GitHub README's pitch language. Do not invent technologies, integrations, or scale numbers not present in the LifeOS source. If a section the schema asks for has no source evidence, state 'no source evidence in LifeOS' for that section rather than fabricating content."*

**Why.** The original NeuroDrive scrape (under the retired `profile-scrape` flow) revealed the old entry described biological plasticity as if it was built, while the actual codebase — handwritten PPO, entertainment-constrained reward engineering, comprehensive analytics — was undersold. Leading with reality produced a stronger entry and avoids interview landmines. The new schema enforces the same principle structurally: per-project synthesis sections name "What is currently built" and "Direction (in-flight, not wishlist)" as separate concerns so the distinction cannot be quietly elided.

---

## Skills are derived, not hand-maintained

`profile/skills.md` is written by the Phase 5 subagent of `populate-from-lifeos`. It reads every file in `profile/projects/` and produces six tables (Programming Languages, Frameworks, Libraries, Engines and Runtimes, Tools and Platforms, Concepts and Domains) × four bands (Proficient, Comfortable, Familiar, Beginner).

**The rubric explicitly rejects** lines-of-code count, project count, and library count as proficiency proxies. Proficiency reflects:

1. depth of work demonstrated,
2. conceptual complexity of what was built,
3. completion stage (active/complete > paused/dormant > abandoned),
4. cross-domain transfer (a tool spanning ≥3 domains earns a one-band lift),
5. evidence specificity in the per-project files.

**Why this matters.** Bad code is long. Five shallow projects don't beat one deep one. Library-free projects are sometimes deliberately library-free (NeuroDrive's no-ML-framework choice). LOC-based or count-based proxies misread all of these. The five-dimension model lets the rubric give the same Rust → Proficient rating for evidence drawn from different shapes of work.

**Edits to `skills.md` should not be made by hand.** Re-run `populate-from-lifeos` instead. A hand-edit will be overwritten on the next sync, and the source-of-truth chain (project files → derived skills) becomes asymmetric.

---

## Career coaching and portfolio gap tracking

`profile/portfolio-gaps.md` is **Cernio-native** — written by the `check-integrity` skill from job-grading data. It tracks both gaps (what the market wants that the profile lacks) and confirmed strengths (what the profile has that the market clearly values).

Gap closure recommendations should be specific and prioritised: *"add a Dockerfile and CI pipeline to Nyquestro"* beats *"learn Docker."*

**Why this file is Cernio-native, not LifeOS-sourced.** The career-coaching loop output is derived from the Cernio jobs database — data LifeOS does not have. Overwriting this file from LifeOS would erase the coaching signal. The sync skill's Phase 7 explicitly verifies that `portfolio-gaps.md`'s timestamp is unchanged at run end.

---

## Everything is dynamic — the Living System Philosophy

Every artefact in the profile system changes over time:

- The profile evolves as LifeOS gets updated (new projects added, existing projects deepened, status changes).
- Skills.md re-derives on every sync — a project gaining new technique work shifts skill bands automatically.
- Company grades shift when the profile shifts (a project that closes a known gap may upgrade companies that previously required that skill).
- Job evaluations shift when preferences shift (e.g. Manchester roles that scored poorly under London-only become viable when the user opens up to relocation).

**Skills must never embed profile snapshots.** Every skill and agent instruction must read all files in `profile/` at runtime. Hardcoded profile data — visa expiry dates, project statuses, degree classifications, location preferences — goes stale silently and produces grading errors that are difficult to detect.

This rule is enforced by deletion: the hardcoded project-name list in `grade-companies/grading-rubric.md` was removed in commit `d907ee8` because the per-project files are now authoritative. The grading rubrics describe *how to weight evidence* (status-based weighting), not *what evidence exists* (the per-project files are the inventory).

---

## The audit artefact: sync-summary.md

After every `populate-from-lifeos` run, `profile/sync-summary.md` captures the per-phase actions: README parse output, files replaced/added/deleted, per-project agent verdicts, evidence blocks reproduced verbatim, and the WIDND covering seven canonical categories (projects on README but absent from LifeOS, projects in LifeOS but excluded from the README, files unreadable due to API errors, orphan files in `cernio/profile/`, Cernio-native files preserved, agents that returned partial evidence, schema sections with no LifeOS source evidence).

The skill runs autonomously from invocation to summary write — no mid-run user prompts. The summary is the post-hoc audit. Read it when you need to know what changed in the most recent sync.

---

## Resume + cover letter pipeline

`profile/resume.md` is a LaTeX document direct-copied from LifeOS. The current resume leads with the tinygrad open-source contribution, then four projects: Image Browser, Aurix, NeuroDrive, and Nyquestro. The five LifeOS projects in the README's *Active Projects* section define the resume scope; the *Other Projects* section is broader portfolio inventory but not all of them appear on the resume.

`profile/cover-letter.md` is similarly direct-copied. Both are the public-facing artefacts the user controls directly via LifeOS edits.

When updating the resume or cover letter, the workflow is: edit `LifeOS/Profile/Professional/Resume.md` or `Cover Letter.md` → run `populate-from-lifeos` → both Cernio copies update.
