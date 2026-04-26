# LifeOS Source Map

## Table of Contents

- [Purpose](#purpose)
- [The Three-Way Distinction](#the-three-way-distinction)
- [Direct Copy Mapping (Phase 2)](#direct-copy-mapping-phase-2)
- [Synthesis Targets (Phases 3, 4, 5, 6)](#synthesis-targets-phases-3-4-5-6)
- [Cernio-Native Files (Untouchable)](#cernio-native-files-untouchable)
- [Legacy Files for Cleanup (Phase 7)](#legacy-files-for-cleanup-phase-7)
- [Orphan Detection Rules](#orphan-detection-rules)
- [Path Normalisation](#path-normalisation)
- [LifeOS Personal/ — Why It Is Skipped Entirely](#lifeos-personal--why-it-is-skipped-entirely)

---

## Purpose

This is the structural source-of-truth for what `populate-from-lifeos` reads, writes, leaves alone, deletes, and surfaces. Every file in `cernio/profile/` falls into exactly one of four categories: **direct copy from LifeOS**, **synthesised from LifeOS or derived from Cernio synthesis output**, **Cernio-native (untouchable)**, or **legacy/orphan (deleted or surfaced)**. Adding a file to one category requires removing it from all others — no double-classification.

This file is loaded mandatory-core because the orchestrator references it in Phase 2 (which mappings to apply), Phase 7 (orphan detection rules), and the Quality Checklist (verifying preserved files). The Phase 3 and Phase 5 subagents do not read this file directly — they receive the relevant subset embedded in their dispatch prompts.

---

## The Three-Way Distinction

```
┌────────────────────────────────────────────────────────────────┐
│  cernio/profile/                                                │
│                                                                 │
│  ┌──────────────────────┐  ┌────────────────────────────────┐  │
│  │  DIRECT COPY (1:1)   │  │  SYNTHESISED                   │  │
│  │  from LifeOS         │  │  from LifeOS folders + Cernio  │  │
│  │  Profile/Professional│  │  synthesis chain                │  │
│  │                      │  │                                 │  │
│  │  • personal.md       │  │  • projects/<name>.md × 12     │  │
│  │  • experience.md     │  │  • projects/open-source-...md  │  │
│  │  • education.md      │  │  • projects/index.md           │  │
│  │  • interests.md      │  │  • skills.md                   │  │
│  │  • resume.md         │  │                                 │  │
│  │  • cover-letter.md   │  └────────────────────────────────┘  │
│  │  • visa.md           │                                       │
│  │  • military.md       │  ┌────────────────────────────────┐  │
│  │  • languages.md      │  │  CERNIO-NATIVE                 │  │
│  │  • certifications.md │  │  Never read, written, deleted  │  │
│  │  • lifestyle-...md   │  │                                 │  │
│  │                      │  │  • preferences.toml            │  │
│  │  (11 files)          │  │  • portfolio-gaps.md           │  │
│  └──────────────────────┘  └────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

The skill's invariant: every file under `cernio/profile/` belongs to exactly one of the three boxes. After a sync run, no other categories exist. Phase 7 enforces this by surfacing orphans.

---

## Direct Copy Mapping (Phase 2)

Phase 2 is **dynamic-enumeration-driven**, not list-driven. The skill enumerates `Profile/Professional/` at run time via `gh api repos/Capataina/LifeOS/contents/Profile/Professional --jq '[.[] | select(.name | endswith(".md")) | .name]'` and copies every `.md` file the listing returns. Adding a new file to LifeOS `Profile/Professional/` makes it sync automatically on the next run; renaming or removing a file makes it stop syncing automatically.

**Why dynamic, not hardcoded:** a hardcoded list goes stale silently the moment the user adds a new file in LifeOS. The user has explicitly named universal-naming as a structural requirement — the skill must not break when the LifeOS schema is extended.

For each `.md` file the listing returns, the skill: fetches the LifeOS source via `gh api`, base64-decodes, normalises the filename, diffs against the Cernio target, and writes only if different. Filename normalisation rule: see §"Path Normalisation" below.

The table below is **illustrative documentation** of the typical sync — what LifeOS `Profile/Professional/` contains at the time of writing this reference. It is not the contract; the contract is whatever `gh api` returns at run time.

| LifeOS source path (typical) | Normalised Cernio target path | Diff verdict actions |
|---|---|---|
| `Profile/Professional/Personal.md` | `cernio/profile/personal.md` | unchanged → skip; replaced → write; absent → write (added) |
| `Profile/Professional/Experience.md` | `cernio/profile/experience.md` | same |
| `Profile/Professional/Education.md` | `cernio/profile/education.md` | same |
| `Profile/Professional/Interests.md` | `cernio/profile/interests.md` | same |
| `Profile/Professional/Resume.md` | `cernio/profile/resume.md` | same |
| `Profile/Professional/Cover Letter.md` | `cernio/profile/cover-letter.md` | same |
| `Profile/Professional/Visa.md` | `cernio/profile/visa.md` | same |
| `Profile/Professional/Military.md` | `cernio/profile/military.md` | same |
| `Profile/Professional/Languages.md` | `cernio/profile/languages.md` | same |
| `Profile/Professional/Certifications.md` | `cernio/profile/certifications.md` | same |
| `Profile/Professional/Lifestyle Preferences.md` | `cernio/profile/lifestyle-preferences.md` | same — present once moved from `Personal/` per §"LifeOS Personal/" |
| *(any future file added to `Profile/Professional/`)* | *(normalised path)* | same |

If `Profile/Professional/` returns empty or the API call fails entirely: abort the skill with the specific failure. An empty Professional/ would suggest a fundamental LifeOS access problem, not a legitimate state.

If the Cernio target file is absent and the LifeOS source is present: write as a new file (recorded as `added` in the diff summary).

A Cernio profile file that exists but no longer has a corresponding `Profile/Professional/` source (file removed or renamed in LifeOS) is detected in Phase 7 as an orphan — surfaced for user review, not auto-deleted.

---

## Synthesis Targets (Phases 3, 4, 5, 6)

These files do not have a 1:1 LifeOS source. They are produced by Phase 3 (per-project), Phase 4 (OSS aggregation), Phase 5 (skills derivation), and Phase 6 (index).

| Cernio target | Source(s) | Phase | Notes |
|---|---|---|---|
| `projects/<lowercase-dashes>.md` × 12 | `Projects/<Name>/` (every file in folder) | 3 | One file per project in README's Active + Other sections; one subagent per project |
| `projects/open-source-contributions.md` | `Projects/Open Source Contributions/` (every file) | 4 | Aggregated — one Cernio file covering all upstreams |
| `skills.md` | All `cernio/profile/projects/*.md` files just generated | 5 | Single subagent reads cross-project, applies multi-table rubric |
| `projects/index.md` | All `cernio/profile/projects/*.md` (excluding `index.md` itself) | 6 | Mechanical — name + status + primary tech + 1-line summary per project |

**The 12 per-project files** (READ-derived allow-list, lowercase-dashes filenames):

- `projects/cernio.md`
- `projects/image-browser.md`
- `projects/aurix.md`
- `projects/neurodrive.md`
- `projects/nyquestro.md`
- `projects/vynapse.md`
- `projects/asteroidsai.md`
- `projects/consilium.md`
- `projects/chrona.md`
- `projects/xyntra.md`
- `projects/zyphos.md`
- `projects/tectra.md`

If the README is updated to add or remove projects, this list updates automatically — Phase 1's parse is the single source for which projects get synthesised. The list above is illustrative of the current README state and is not the gatekeeper; the README is.

---

## Cernio-Native Files (Untouchable)

These files exist in `cernio/profile/` but have **no LifeOS source** and are **never** touched by this skill. Their content is authored elsewhere (the Rust pipeline, the `check-integrity` skill).

| File | Author | Why off-limits |
|---|---|---|
| `preferences.toml` | Hand-edited; machine-read by `src/config.rs` | Pipeline configuration. Cernio's filter chain, location patterns, cleanup thresholds, and search keywords are all read from this file at every CLI invocation. A sync skill writing to it (or leaving it in an inconsistent state) breaks the entire job-search pipeline silently. |
| `portfolio-gaps.md` | `check-integrity` skill, written from job-grading data | The career-coaching loop output. It contains market-pattern findings derived from the Cernio jobs database — data that LifeOS does not have access to. Overwriting it from LifeOS would erase the coaching signal. |

**Phase 7 verifies preservation** by capturing modification timestamps before Phase 0 begins and comparing after Phase 8 writes. If either timestamp changed, the skill has a bug — abort with explicit error citing the changed file.

---

## Legacy Files for Cleanup (Phase 7)

Files that existed in the pre-skill `cernio/profile/` schema but no longer fit the new structure. Phase 7 deletes these.

| File | Why deleted | Replaced by |
|---|---|---|
| `cernio/profile/projects.md` | The pre-Phase-2 flat projects file. Old schema collapsed all projects into one document. | `cernio/profile/projects/<name>.md` files + `projects/index.md` |
| `cernio/profile/volunteering.md` | The OSS contribution record (currently a single tinygrad entry). Migrated under the new schema. | `cernio/profile/projects/open-source-contributions.md` |

If either file is absent at Phase 7 (already cleaned up in a prior run, or never existed): record *"Legacy file already absent — no cleanup needed"* in the ledger. Not an error.

---

## Orphan Detection Rules

After Phases 2-6 have written every expected file, Phase 7 scans `cernio/profile/` for files that match no schema slot. Orphans are surfaced in WIDND for user review — never auto-deleted.

The schema slots:

- The 11 direct-copy targets (Section: Direct Copy Mapping).
- `projects/` directory containing only: `index.md`, `open-source-contributions.md`, the per-project files matching the README allow-list.
- `skills.md`.
- The 2 Cernio-native files (`preferences.toml`, `portfolio-gaps.md`).
- Sub-directories of `cernio/profile/` other than `projects/` are unexpected (none currently planned).

Anything else is an orphan. Common orphan sources:

- Files manually added by the user that the schema does not know about.
- Files left over from a previous schema version not covered by the legacy-cleanup list.
- Renamed files where the old name persists.

The orphan WIDND entry includes: file path, last-modification timestamp, and one of three suggested actions: *"Move into the schema (rename or relocate)"* / *"Add to the legacy-cleanup list in this file"* / *"Add to the schema by extending one of the categories above"*. The user decides; the skill never deletes.

---

## Path Normalisation

LifeOS uses `Title Case With Spaces.md`; Cernio uses `lowercase-dashes.md`. The skill normalises every output filename to the Cernio convention.

| LifeOS form | Normalised Cernio form |
|---|---|
| `Cover Letter.md` | `cover-letter.md` |
| `Lifestyle Preferences.md` | `lifestyle-preferences.md` |
| `Image Browser/` | `image-browser.md` |
| `Open Source Contributions/` | `open-source-contributions.md` |
| `AsteroidsAI/` | `asteroidsai.md` |
| `NeuroDrive/` | `neurodrive.md` |
| `Cernio/` | `cernio.md` |

Normalisation rule: lowercase the entire string, replace spaces with dashes, strip non-alphanumeric characters except dashes. Camel-case folder names (`AsteroidsAI`, `NeuroDrive`) become flat lowercase (`asteroidsai`, `neurodrive`) — matches the existing `cernio/profile/` convention.

Path-encoding for `gh api` calls: spaces become `%20` in the URL path. `gh api repos/Capataina/LifeOS/contents/Projects/Image%20Browser` not `Projects/Image Browser`.

---

## LifeOS Personal/ — Why It Is Skipped Entirely

`LifeOS/Profile/Personal/` contains three files at the time of writing: `Dream Partner.md`, `Lifestyle Preferences.md`, `Values.md`.

The brief's Phase 1 split designated `Personal/` as LifeOS-only — content not meant for downstream career tooling. This skill respects that classification with **one historical exception**: `Lifestyle Preferences.md` was originally placed in `Personal/` but is load-bearing for Cernio's grading rubric (Cernio uses lifestyle fit as a same-tier modulator on company and job grades). The decision (recorded in the design conversation that authored this skill): move `Lifestyle Preferences.md` from `Personal/` to `Professional/` in LifeOS, so the principle "Personal/ is never touched" stays clean.

Once the move is performed in LifeOS, the Direct Copy Mapping above includes `Lifestyle Preferences.md` straightforwardly. If the move has not yet been performed when this skill is invoked, Phase 2's diff for `lifestyle-preferences.md` will record *"Source file absent from Profile/Professional/ — sync target not updated"* — a benign WIDND entry until the LifeOS-side move is done.

`Dream Partner.md` and `Values.md` remain in `Personal/` and are never read by this skill.
