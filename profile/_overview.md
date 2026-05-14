---
title: "Profile/Professional — Overview"
tags:
  - profile
  - professional
  - overview
domain: profile
last_updated: 2026-04-28
---

# Profile/Professional — Overview

Career-facing structured facts about Caner. The professional half of `Profile/`. Each file in this folder is a single authoritative slice of who Caner is when interacting with the job market, recruiters, hiring managers, and engineering interviewers.

## What lives here

| File | Owns |
|---|---|
| [[Profile/Professional/Personal\|Personal]] | Demographic + identity facts: name, location, age, languages, contact, citizenship status |
| [[Profile/Professional/Resume - Ata Caner Cetinkaya\|Resume]] | LaTeX source of the canonical 1-page resume. Compiled to PDF for job applications |
| [[Profile/Professional/Cover Letter - Ata Caner Cetinkaya\|Cover Letter]] | Generic cover letter narrative; per-application tailoring is a separate workflow |
| [[Profile/Professional/LinkedIn\|LinkedIn]] | LinkedIn-profile content + update checklist (mirrors Resume; no MCP/API access so this is the canonical source for what to paste into the LinkedIn UI) |
| [[Profile/Professional/Experience\|Experience]] | Structured employment + project experience history |
| [[Profile/Professional/Education\|Education]] | Formal education record |
| [[Profile/Professional/Interests\|Interests]] | Active interests + engineering practices (canonical source for portfolio-strategy interest territories) |
| [[Profile/Professional/Languages\|Languages]] | Spoken languages + proficiency |
| [[Profile/Professional/Lifestyle Preferences\|Lifestyle Preferences]] | Job-search-relevant lifestyle constraints (commute, café-work, gym access, weather, frontier-tech availability). Moved here from Personal/ on 2026-04-26 |
| [[Profile/Professional/Visa\|Visa]] | UK visa status + work eligibility |
| [[Profile/Professional/Military\|Military]] | Turkish military service status |
| [[Profile/Professional/Certifications\|Certifications]] | Formal certifications |
| [[Profile/Professional/LeetCode\|LeetCode]] | Practice profile: solved problems, language mix, topic coverage, calibration data |

## Source-of-truth hierarchy

When a fact about Caner appears in multiple places, the resolution order is:

1. **`Profile/Professional/`** wins on professional-context facts (resume claims, experience, etc.).
2. **`Profile/Personal/`** wins on personal-context facts (values, dream partner, etc.).
3. Working notes elsewhere in the vault carry current state of ongoing work, not Profile-level facts about Caner.

When working notes contradict Profile, Profile is updated (the K12 cross-update propagation pattern in vault-lint).

## Cross-references

- [[Profile/Personal/Values\|Personal/Values]] — non-career-facing identity content
- [[Projects/_Overview]] — projects demonstrate the skills + interests claimed here
- [[Profile/Professional/Resume - Ata Caner Cetinkaya]] — LaTeX source compiled by [[.claude/skills/improve-resume/SKILL]] (improve-resume skill)
- `~/.claude/CLAUDE.md` — global personality consumes the Profile context

## Maintenance

- The `improve-resume` vault-local skill keeps Resume.md + Cover Letter.md in sync with project state via dated proposals files at `Profile/Professional/Resume - Ata Caner Cetinkaya Proposals YYYY-MM-DD.md`
- Profile drift is detected by morning-brew Phase 3 step 4 + vault-lint K12 cross-update propagation check
- Per the K12 pattern: a fact demonstrated in 2+ projects without representation here is surfaced as a propagation candidate

## Current state (2026-04-28)

Recent activity in this folder:

- `5a2ce4d` (2026-04-28) — vault-lint ST12 Phase D: create missing Overview/Index files for Context/, Profile/Profess
- `8dfc16c` (2026-04-28) — profile/interests: add Terminal UI engineering w/ Ratatui + Crossterm (3-day-old K12 propa
- `db6a272` (2026-04-27) — resume: trim 'complete' + 'from scratch' from NeuroDrive b1 to absorb orphan
- `5c56777` (2026-04-27) — resume: restore over-trimmed signal — keep 1 page
- `72bcc8a` (2026-04-27) — resume: refresh projects + add burn OSS contribution + tighten to 1 page
