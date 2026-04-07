# Profile System Decisions

The profile is the source of truth for who the user is, what they can do, and what they're looking for.

---

## Read on every startup

Claude reads every file in `profile/` at the start of every session, no exceptions. Without the profile loaded, Claude cannot assess job fit, recommend companies, or give career advice accurately. The user never has to re-explain their background.

---

## Auto-updatable from repos

Given a GitHub repo link, the `profile-scrape` skill reads the repo (context/ folder, README, dependency manifests, source structure) and produces or updates profile entries automatically. This means:

- New projects can be added by giving Claude a link
- Existing entries stay current as projects evolve
- Skills and proficiency assessments are grounded in actual code, not self-reported
- Re-running on an existing project updates the entry rather than duplicating it

**Why:** Manual profile maintenance is friction. The projects are the source of truth for what the user can do.

---

## Describe what's built, not what's planned

Profile entries lead with what the code actually demonstrates today. Future goals belong in the summary and project staging description, but technical highlights must be grounded in implemented reality.

**Why:** The NeuroDrive scrape revealed the old entry described biological plasticity as if it was built, while the actual codebase — handwritten PPO, entertainment-constrained reward engineering, comprehensive analytics — was undersold. Leading with reality produced a stronger entry and avoids interview landmines.

**How to apply:** Always distinguish "built and working" from "designed and planned." Both belong in the entry, but technical highlights come from code, not READMEs.

---

## Career coaching and portfolio gap tracking

Claude actively tracks patterns across job descriptions to identify what's strong and what's missing in the profile. `profile/portfolio-gaps.md` tracks strengths, known gaps, and concrete closure opportunities.

Gap closure recommendations should be specific and prioritised: "add a Dockerfile and CI pipeline to Nyquestro" beats "learn Docker."

**Why:** The better the profile fits the market, the more jobs pass evaluation. Entry-level candidates especially benefit because small additions can open entire categories of roles.

---

## Everything is dynamic

The profile, resume, preferences, and company universe all change as projects grow, new contributions land, new skills are acquired, and market patterns shift. Claude should note anything that looks stale and proactively suggest updates.

---

## Preferences are intentionally flexible

Hard filters like `exclude_sectors` and `tech_must_have` are kept loose because the user is an entry-level engineer exploring options rather than prematurely narrowing the search.

---

## Resume uses four main projects

The current resume (LaTeX, stored as `profile/resume.md`) leads with the tinygrad open source contribution, then four projects: Image Browser, Aurix, NeuroDrive, and Nyquestro. Other projects in `projects.md` exist but are incomplete or lower priority. The resume is the public-facing artefact the user controls directly; `projects.md` is the full inventory.

---

## Main projects have readable context/ folders

The four main projects each have a `context/` folder maintained by the same skill system. When updating profile entries or evaluating job fit, Claude can fetch these for up-to-date architectural and implementation details.
