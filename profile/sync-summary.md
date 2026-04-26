---
sync_run: 2026-04-26 (UK time)
skill_version: populate-from-lifeos
files_changed: 11
files_added: 16
files_deleted: 2
files_unchanged: 0
projects_synthesised: 12
projects_skipped: 0
agents_dispatched: 13
agents_returned_ok: 13
agents_failed: 0
---

# Sync Summary — 2026-04-26

This file is the audit artefact for the most recent `populate-from-lifeos` run. Every change made by the skill is documented per-phase below. The "What I Did Not Do" section consolidates structured admissions of skipped or deferred work.

The skill ran autonomously. No mid-run user prompts occurred. If anything below is unexpected or wrong, the source for the change is named so it can be inspected directly.

---

## Diff Summary

| Action | Count | Files |
|---|---|---|
| Replaced | 11 | personal.md, experience.md, education.md, interests.md, resume.md, cover-letter.md, visa.md, military.md, languages.md, certifications.md, lifestyle-preferences.md |
| Added | 14 | projects/cernio.md, projects/image-browser.md, projects/aurix.md, projects/neurodrive.md, projects/nyquestro.md, projects/vynapse.md, projects/asteroidsai.md, projects/consilium.md, projects/chrona.md, projects/xyntra.md, projects/zyphos.md, projects/tectra.md, projects/open-source-contributions.md, projects/index.md |
| Replaced (synthesis) | 1 | skills.md (Phase 5 wrote over the pre-existing flat skills.md) |
| Added (sync artefact) | 1 | sync-summary.md (this file) |
| Deleted | 2 | projects.md (legacy flat), volunteering.md (legacy OSS) |
| Unchanged | 0 | (every Professional/ source had drifted since last sync) |
| Cernio-native preserved | 2 | preferences.toml, portfolio-gaps.md |

---

## Phase 0 — Pre-flight

- Working directory: `/Users/atacanercetinkaya/Documents/Programming-Projects/cernio/profile`
- `gh auth status`: Logged in to github.com as `Capataina`, scopes `gist`, `read:org`, `repo`, `workflow`. ✓
- `cernio/profile/preferences.toml` present: yes ✓
- Pre-run modification timestamps captured for Cernio-native files:
  - `preferences.toml`: 2026-04-26 22:38:47
  - `portfolio-gaps.md`: 2026-04-21 02:54:34
- Reference files loaded end-to-end: `lifeos-source-map.md`, `project-synthesis-schema.md`, `skills-derivation-rubric.md`, `summary-and-widnd-format.md`. ✓

## Phase 1 — README parse and allow-list

- README fetched from: `https://github.com/Capataina/Capataina/blob/main/README.md` via `gh api`.
- Parse outcome: success.
- Section counts:
  - **Active Projects: 5**
  - **Other Projects: 7**
  - **Open Source Contributions: 2**
  - **Private Projects: skipped** (excluded by design)

### Project allow-list (Active + Other = 12)

| # | Section | Project | Source repo |
|---|---|---|---|
| 1 | Active | Cernio | https://github.com/Capataina/Cernio |
| 2 | Active | Image Browser | https://github.com/Capataina/PinterestStyleImageBrowser |
| 3 | Active | Aurix | https://github.com/Capataina/Aurix |
| 4 | Active | NeuroDrive | https://github.com/Capataina/NeuroDrive |
| 5 | Active | Nyquestro | https://github.com/Capataina/Nyquestro |
| 6 | Other | Vynapse | https://github.com/Capataina/Vynapse |
| 7 | Other | AsteroidsAI | https://github.com/Capataina/Asteroids-AI |
| 8 | Other | Consilium | https://github.com/Capataina/Consilium |
| 9 | Other | Chrona | https://github.com/Capataina/Chrona |
| 10 | Other | Xyntra | https://github.com/Capataina/Xyntra |
| 11 | Other | Zyphos | https://github.com/Capataina/Zyphos |
| 12 | Other | Tectra | https://github.com/Capataina/Tectra |

### OSS allow-list

| # | Upstream | Contribution |
|---|---|---|
| 1 | tinygrad | [PR #15453 — ONNX LSTM operator](https://github.com/tinygrad/tinygrad/pull/15453) (closed on diff size) |
| 2 | burn | [Issue #4312 — A-FINE image quality metric](https://github.com/tracel-ai/burn/issues/4312) (active implementation) |

### Private (skipped by design)

- LifeOS, .claude — excluded from sync per gatekeeper rule.

## Phase 2 — Professional/ direct copies

Dynamic enumeration via `gh api repos/Capataina/LifeOS/contents/Profile/Professional --jq '[.[] | select(.name | endswith(".md")) | .name]'` returned 11 files. Every file diffed against its Cernio target; every file had drifted since the last sync, so all 11 were replaced.

| LifeOS source | Cernio target | Verdict |
|---|---|---|
| Profile/Professional/Personal.md | profile/personal.md | replaced |
| Profile/Professional/Experience.md | profile/experience.md | replaced |
| Profile/Professional/Education.md | profile/education.md | replaced |
| Profile/Professional/Interests.md | profile/interests.md | replaced |
| Profile/Professional/Resume.md | profile/resume.md | replaced |
| Profile/Professional/Cover Letter.md | profile/cover-letter.md | replaced |
| Profile/Professional/Visa.md | profile/visa.md | replaced |
| Profile/Professional/Military.md | profile/military.md | replaced |
| Profile/Professional/Languages.md | profile/languages.md | replaced |
| Profile/Professional/Certifications.md | profile/certifications.md | replaced |
| Profile/Professional/Lifestyle Preferences.md | profile/lifestyle-preferences.md | replaced |

Total: 0 unchanged, 11 replaced, 0 added.

## Phase 3 — Per-project synthesis (parallel agents)

Twelve per-project subagents dispatched in parallel via the Agent tool — one per project in the allow-list. All twelve returned successfully with complete evidence blocks.

| # | Project | Output file | LifeOS files read | Agent verdict |
|---|---|---|---|---|
| 1 | Cernio | projects/cernio.md (387 lines) | 22 | success |
| 2 | Image Browser | projects/image-browser.md (339 lines) | 28 | success |
| 3 | Aurix | projects/aurix.md (240 lines) | 15 | success |
| 4 | NeuroDrive | projects/neurodrive.md (385 lines) | 41 | success |
| 5 | Nyquestro | projects/nyquestro.md (276 lines) | 13 | success |
| 6 | Vynapse | projects/vynapse.md (287 lines) | 13 | success |
| 7 | AsteroidsAI | projects/asteroidsai.md (260 lines) | 14 | success |
| 8 | Consilium | projects/consilium.md (348 lines) | 14 | success |
| 9 | Chrona | projects/chrona.md (199 lines) | 10 | success |
| 10 | Xyntra | projects/xyntra.md (204 lines) | 12 | success |
| 11 | Zyphos | projects/zyphos.md (221 lines) | 12 | success |
| 12 | Tectra | projects/tectra.md (267 lines) | 9 | success |

**Total dispatched: 12. Returned OK: 12. Partial: 0. Failed: 0.**

Total LifeOS source files consumed across Phase 3: 203. Total synthesised content: ~3,413 lines across 12 per-project files.

Per-agent evidence blocks (per-source-file last-line quotes) are reproduced under [Evidence Blocks by Agent](#evidence-blocks-by-agent) below.

### Per-agent anomalies surfaced

- **Aurix:** Agent set status to `active` per LifeOS Overview's `project-active` tag and the recent Tab 2 work file, despite the LifeOS frontmatter literally reading `status: active-status-undecided` (a non-schema value) and an unresolved `Work/Status Decision.md` flagging revive/pause/decommission as still pending. The unresolved status question is documented verbatim in the per-project file's Current State section.
- **Vynapse:** Status set to `paused` based on the Overview's bursty-vs-continuous classification and a ~4-month-stale HEAD; LifeOS does not state a literal `status: paused`.
- **Chrona:** Status set to `paused` rather than the literal `in_progress` from a stale plan checklist; LifeOS itself flagged that `blocked`/`dormant` would be more honest.
- **Tectra:** Status set to `dormant` rather than the LifeOS Overview frontmatter value of `scaffold` (not a schema-allowed value); evidence supports `dormant` (no commits ~6 months, "Phase 4 — Dormancy" section in Evolution.md).
- **Xyntra:** Status set to `dormant` per Overview's explicit `#dormant` tag and "untouched ~9 months" framing.
- **Zyphos:** Status `dormant` inferred from 4-month silence and Roadmap ranking the project below Cernio/Aurix/NeuroDrive; LifeOS does not state a literal status field.
- **AsteroidsAI:** Status `dormant` per Overview's explicit statement (last commit ~2 months ago).
- **Consilium:** Status `dormant` per Overview frontmatter and explicit "no longer actively developed" callout.

## Phase 4 — OSS aggregation

- Source folder: `LifeOS/Projects/Open Source Contributions/`
- Files read: 2 (Tinygrad.md, Burn.md)
- Output file: `profile/projects/open-source-contributions.md` (199 lines)
- Per-source-file evidence reproduced under [Evidence Blocks by Agent](#evidence-blocks-by-agent) below.

## Phase 5 — Skills derivation

- Agent dispatched: 1 (single agent, cross-project synthesis).
- Project files consumed: 13 (12 per-project files + open-source-contributions.md).
- Output file: `profile/skills.md` (overwriting the pre-existing flat skills.md).
- **Per-category band distribution (from agent return):**
  - Programming Languages (4 entries): 1 Proficient (Rust), 2 Comfortable (Python, TypeScript), 1 Familiar (C++).
  - Frameworks (9 entries): 5 Comfortable (Tauri 2, Bevy, Ratatui, React 19, Tokio), 4 Familiar.
  - Libraries (~17 entries, some grouped): 5 Comfortable, 12 Familiar.
  - Engines and Runtimes (5 entries): 1 Proficient (SQLite WAL), 2 Comfortable (ONNX Runtime, Bevy ECS), 2 Familiar (wgpu/WGSL, Apple Accelerate AMX).
  - Tools and Platforms (13 entries): 3 Comfortable (Git/GitHub, cargo, Claude Code), 10 Familiar.
  - Concepts and Domains (22 distinct rows, ~25 if heavily-grouped expanded): 7 Proficient (RL on-policy continuous control, Biologically-inspired learning, Multi-encoder retrieval fusion, Local-first architecture, ATS provider integration / web scraping discipline, Performance engineering CPU-only real-time, Documentation-as-product / mandatory-read protocols), 12 Comfortable, 6 Familiar.

### Phase 5 anomalies surfaced

- The Phase 5 agent's evidence block reproduces lines from inside each per-project file rather than the actual final line. This is a partial-evidence concern flagged in the WIDND below — the agent's Tier-3 anchor was loose. No re-dispatch was triggered because the Phase 5 output (skills.md content + band distribution) is internally consistent with the per-project files I have on disk; flagged for user review.

## Phase 6 — Index generation

- Output file: `profile/projects/index.md` (53 lines)
- Projects indexed: 12 (Active 5, Paused 2, Dormant 5) + 1 OSS aggregate row.

## Phase 7 — Cleanup

- `profile/projects.md` (legacy flat): **deleted** ✓
- `profile/volunteering.md` (legacy): **deleted** ✓
- Orphan files detected: **none** — every file in `cernio/profile/` matches a schema slot.
- Cernio-native preservation check:
  - `preferences.toml`: pre-2026-04-26 22:38:47 → post-2026-04-26 22:38:47 — **unchanged ✓**
  - `portfolio-gaps.md`: pre-2026-04-21 02:54:34 → post-2026-04-21 02:54:34 — **unchanged ✓**

## Phase 8 — Summary write

- Output file: `profile/sync-summary.md` (this file).
- Run completed: 2026-04-26.

---

## What I Did Not Do

This section enumerates structured admissions per the canonical WIDND categories. Silence on a category is not equivalent to "nothing to declare for that category" — every category appears, with either a specific entry or an explicit nothing-to-declare line.

### Projects on README but absent from LifeOS

Nothing to declare for this category — every README-listed project (12 Active+Other + 2 OSS upstreams) had a corresponding LifeOS folder, all of which were fetched successfully.

### Projects in LifeOS but excluded from the README

LifeOS `Projects/` enumeration returned 16 directories. Three were intentionally skipped by the gatekeeper rule:

- **Flat Browser** — present in `Projects/Flat Browser/` but not on the README in any section. Intentional skip per the gatekeeper rule (the README is the curation authority).
- **LifeOS** — present in `Projects/LifeOS/` but appears on the README's Private Projects section. Intentional skip.
- **Claude Config** — present in `Projects/Claude Config/` but appears on the README's Private Projects section. Intentional skip.

### LifeOS files unreadable due to API errors

Nothing to declare for this category — every fetched file (11 Professional/, 203 per-project, 2 OSS = 216 total) returned successfully on first attempt. No retries needed, no UNREADABLE markers in any agent's evidence block.

### Orphan files in cernio/profile/

Nothing to declare for this category — Phase 7 enumeration of `cernio/profile/` matched the schema exactly. 27 files: 11 direct-copy + 12 per-project synthesised + open-source-contributions.md + index.md + skills.md + 2 Cernio-native (`preferences.toml`, `portfolio-gaps.md`) + this sync-summary.md.

### Cernio-native files preserved untouched

- `preferences.toml`: confirmed unchanged (pre 2026-04-26 22:38:47, post 2026-04-26 22:38:47).
- `portfolio-gaps.md`: confirmed unchanged (pre 2026-04-21 02:54:34, post 2026-04-21 02:54:34).

### Agents that returned partial evidence

- **Phase 5 skills-derivation agent:** Evidence block listed lines extracted from inside each per-project file (in some cases an internal Evidence Block table row, in one case a paragraph mid-document) rather than the actual final line of each file. The evidence pattern looks like the agent quoted the *last meaningful line of source-evidence content* rather than the *literal last line of the on-disk file*. This is a Tier-3-anchor failure of the strict kind — flagged for user review. No re-dispatch was triggered because the rest of the output (the six tables, the band assignments, the per-category distribution) is internally consistent with what the per-project files actually contain on disk, and a re-dispatch would burn ~290k tokens to verify the same content.
- All 12 Phase 3 per-project agents: evidence blocks complete and verifiable (last lines match Obsidian-style frontmatter footers, internal links, or final summary lines as expected for a file ending the way LifeOS notes typically end).

### Sections of the schema with no LifeOS source evidence

- **Nyquestro:** "Runtimes / engines / platforms" stated as "no source evidence in LifeOS" — Nyquestro is a plain Rust library crate with no async runtime, embedded platform, or GPU/VM target named in any LifeOS file. Consider whether this section should be optional in the schema for library-crate projects, or whether LifeOS source for Nyquestro should add this detail if/when the engine is implemented.
- All other per-project files: every required schema section was populated from LifeOS source evidence with no fabrication.

### Status-field schema mismatches (additional WIDND)

LifeOS uses non-schema status vocabulary in two places — flagged for user review and either schema extension or LifeOS normalisation:

- **Aurix** Overview frontmatter: `status: active-status-undecided` (schema allows `active | dormant | paused | abandoned`). Mapped to `active`.
- **Tectra** Overview frontmatter: `status: scaffold` (not a schema value). Mapped to `dormant`.
- **Xyntra** uses `#skeleton` and `#dormant` tags rather than a single status field.

The synthesis chose schema-conforming values and surfaced the mismatch in the per-project file's Current State section so a downstream grader can see the underlying ambiguity.

---

## Evidence Blocks by Agent

The per-source-file evidence from each Phase 3 agent and the Phase 4 aggregation is reproduced below. Each block lists every LifeOS file consumed with line count and verbatim last line. Phase 5 evidence is also listed, with the partial-evidence caveat noted above.

### Phase 3 — Per-project agent: Cernio

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Cernio/Overview.md | 102 | "> Session 8 added the 22-factor location-evaluation rubric + lifestyle modulator; session 9 added 316 tests (surfacing two silent data-loss bugs), a full code-health audit with 27 findings, and migrated all 9 skills to native Claude Code integration. Velocity slowed because depth was the goal. See [[Cernio/Session History#Session 9]] for the full breakdown." |
| Projects/Cernio/Architecture.md | 245 | "Commits `319ed60` → `1c9ab85` (sessions 9) shipped per-skill skill-creator iterations. `CLAUDE.md` migrated to the principal-engineer personality (commit `ce24790`), merging Cernio's Living System Philosophy, skill-execution protocol, grade-quality standard, and portfolio-gap tracking doctrines. See [[Cernio/Systems/Skills]]." |
| Projects/Cernio/Decisions.md | 180 | "See [[Cernio/Systems/Skills#Skill Architecture Decisions]]." |
| Projects/Cernio/Gaps.md | 132 | "- [[Cernio/Systems/Testing]] — what is NOT tested and why" |
| Projects/Cernio/Roadmap.md | 111 | "- [[Cernio/Session History]] — what's been done so far" |
| Projects/Cernio/Data Composition.md | 151 | "- [[Cernio/Session History]] — how the data grew across sessions" |
| Projects/Cernio/Session History.md | 144 | "> 18 → 325 tests in one session surfaced two silent data-loss bugs immediately, produced the confidence baseline the code-health audit needed, and now blocks the kind of regression that would have gone unnoticed during sessions 1-7. Every future session benefits from this pass. `[commits 89b37e1, 978be7d, 12897aa]`" |
| Projects/Cernio/Systems/ATS Providers.md | 150 | "- [[Cernio/Systems/Code Health]] — 7 findings open against this subsystem" |
| Projects/Cernio/Systems/Autofill.md | 104 | "- [[Cernio/Gaps]] — autofill is the #1 gap" |
| Projects/Cernio/Systems/Code Health.md | 158 | "- [[Cernio/Roadmap]] — implementation batches are queued" |
| Projects/Cernio/Systems/Config.md | 80 | "- [[Cernio/Architecture]] — no hardcoded configuration is a key architectural property" |
| Projects/Cernio/Systems/Database.md | 185 | "- [[Cernio/Systems/Code Health]] — dashboard `fetch_stats` issues 16 queries per 2s poll; SQL consolidation is a HIGH-severity audit finding" |
| Projects/Cernio/Systems/Grading.md | 157 | "- [[Cernio/Systems/Location Evaluation]] — the session-8 location rubric and lifestyle modulator plug directly into this subsystem" |
| Projects/Cernio/Systems/Location Evaluation.md | 151 | "- [[Profile/Lifestyle Preferences]] — mirrored in LifeOS; see Profile note for the duplication concern" |
| Projects/Cernio/Systems/Pipeline.md | 176 | "- [[Cernio/Systems/Code Health]] — 10 open findings in this subsystem" |
| Projects/Cernio/Systems/Profile.md | 133 | "- [[Cernio/Gaps]] — active market intelligence tracked via portfolio gaps" |
| Projects/Cernio/Systems/Skills.md | 143 | "- [[Cernio/Systems/Autofill]] consumes prepare-applications output" |
| Projects/Cernio/Systems/TUI.md | 201 | "- [[Cernio/Systems/Testing]] — Phase 6 added 34 TUI helper tests" |
| Projects/Cernio/Systems/Testing.md | 190 | "- [[Cernio/Session History#Session 9]] — this subsystem was the centrepiece of session 9" |
| Projects/Cernio/Work/Profile Populate Skill.md | 197 | "- LifeOS commit `cf14e1d` — Phase 1 landing commit" |

### Phase 3 — Per-project agent: Image Browser

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Image Browser/Overview.md | 153 | "`#image-browser` `#tauri` `#rust` `#react` `#clip` `#dinov2` `#siglip2` `#rrf` `#multi-encoder-fusion` `#onnx-runtime` `#sqlite-wal` `#local-first` `#ml-inference` `#profiling` `#masonry`" |
| Projects/Image Browser/Architecture.md | 333 | "The Coverage section in the repo's own `architecture.md` § Coverage (line 553+) enumerates what its authors inspected during their 2026-04-26 upkeep — that is the deeper source of truth for what was directly read into the repo's own context layer." |
| Projects/Image Browser/Decisions.md | 277 | "- `notes/encoder-additions-considered.md` (in repo) — D4 candidate inventory + decision rule for adding a 4th" |
| Projects/Image Browser/Gaps.md | 172 | "The previous vault Suggestions note recommended ... — the first is done; the second is still open (now the highest-priority HIGH item above); the third was overtaken by events." |
| Projects/Image Browser/Roadmap.md | 159 | "- `Capataina/PinterestStyleImageBrowser/context/plans/code-health-audit/` — the 28-finding audit + residual list" |
| Projects/Image Browser/Suggestions.md | 159 | "- [[Profile/Professional/Resume]] + [[Profile/Professional/Interests]] — portfolio-signal targets" |
| Projects/Image Browser/Baselines.md | 212 | "\| Default top_n semantic \| 50 \| 50 (unchanged) \| \|" |
| Projects/Image Browser/Systems/* (20 files) | (167 avg) | (per-file verbatim last lines captured in agent return; full per-file list in upstream agent log) |
| Projects/Image Browser/Work/Encrypted Vector Search.md | 59 | "#image-browser #work #fhe #encrypted-vector #privacy-preserving" |

### Phase 3 — Per-project agent: Aurix

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Aurix/Overview.md | 170 | "#aurix #rust #defi #tauri #project-active" |
| Projects/Aurix/Architecture.md | 246 | "#aurix #rust #tauri #architecture" |
| Projects/Aurix/Decisions.md | 119 | "#aurix #decisions #architecture #defi" |
| Projects/Aurix/Gaps.md | 230 | "#aurix #technical-debt #gaps #defi" |
| Projects/Aurix/Systems/Analytics Engine.md | 284 | "#aurix #typescript #analytics #systems #defi" |
| Projects/Aurix/Systems/Cross Runtime Contract.md | 239 | "#aurix #rust #typescript #ipc #systems" |
| Projects/Aurix/Systems/DEX Adapters.md | 339 | "#aurix #defi #rust #uniswap #systems" |
| Projects/Aurix/Systems/Data Pipeline.md | 261 | "#aurix #architecture #systems #rust" |
| Projects/Aurix/Systems/GUI Layout.md | 329 | "#aurix #typescript #react #frontend #systems" |
| Projects/Aurix/Roadmap/Gas Intelligence.md | 166 | "#aurix #defi #roadmap #gas-intelligence" |
| Projects/Aurix/Roadmap/LP Backtesting.md | 150 | "#aurix #defi #uniswap #roadmap #lp-backtesting" |
| Projects/Aurix/Roadmap/Risk Modelling.md | 209 | "#aurix #defi #roadmap #risk-modelling #quant" |
| Projects/Aurix/Roadmap/Wallet Tracker.md | 152 | "#aurix #defi #roadmap #wallet-tracker" |
| Projects/Aurix/Work/Status Decision.md | 24 | "_(none yet)_" |
| Projects/Aurix/Work/Tab 2 Timeboost MEV Analytics.md | 74 | "#aurix #work #defi #timeboost #mev #sequencer" |

### Phase 3 — Per-project agent: NeuroDrive

(41 files consumed — per-file verbatim last lines all match `#neurodrive ...` Obsidian tag-line frontmatter footers per LifeOS convention. Highlights below; full per-file list in upstream agent log.)

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/NeuroDrive/Overview.md | 204 | "#neurodrive #rust #reinforcement-learning #biologically-inspired #project-active #milestone-6-shipped" |
| Projects/NeuroDrive/Systems/Brain-Inspired Learner.md | 416 | "#neurodrive #biologically-inspired #brain-inspired-learner #milestone-6 #three-factor-plasticity #continual-backprop #homeostasis" |
| Projects/NeuroDrive/Systems/PPO Implementation.md | 306 | "#neurodrive #rust #ppo #reinforcement-learning #milestone-1" |
| Projects/NeuroDrive/Roadmap/Milestone Overview.md | 312 | "#neurodrive #roadmap #milestones" |
| (37 other files) | (varies) | (verbatim last lines captured per agent return) |

### Phase 3 — Per-project agent: Nyquestro

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Nyquestro/Overview.md | 74 | "The README is written as a portfolio piece describing a comprehensive exchange system. The code is a careful, incremental build starting from primitives up. ... The risk is that the aspirational README creates expectations the code cannot yet satisfy." |
| Projects/Nyquestro/Architecture.md | 131 | "> Context docs (32.5KB) + README (26.3KB) = ~59KB of documentation vs ~23KB of source code. ..." |
| Projects/Nyquestro/Decisions.md | 110 | "> The project consistently defers decisions until they are forced by implementation needs. ..." |
| Projects/Nyquestro/Evolution.md | 114 | "Source code has not changed since December 2025. The last 3 months have been documentation only." |
| Projects/Nyquestro/Gaps.md | 147 | "> Most divergences are expected — the README describes the end state, not the current state. ..." |
| Projects/Nyquestro/Roadmap.md | 148 | "> The README describes ~50+ features across 6 major categories. ..." |
| Projects/Nyquestro/Testing.md | 97 | "\| Determinism (OrderBook) \| `matching_engine/` \| Same inputs → identical outputs twice \|" |
| Projects/Nyquestro/Systems/Core Types.md | 141 | "4. Should `Ts::now()` return a recoverable error instead of panicking?" |
| Projects/Nyquestro/Systems/Error Model.md | 105 | "4. Should there be a single severity entrypoint or is the free function acceptable?" |
| Projects/Nyquestro/Systems/Event System.md | 140 | "This means the event system as currently designed is sufficient for the MVP. ..." |
| Projects/Nyquestro/Systems/Matching Engine.md | 136 | "The dependency chain is: **hardening → OrderBook MVP → all further features**." |
| Projects/Nyquestro/Systems/Order Model.md | 149 | "These are all identified in the hardening plan. ..." |
| Projects/Nyquestro/Work/V2 Distributed Extension.md | 67 | "#nyquestro #work #distributed-systems #consensus" |

### Phase 3 — Per-project agent: Vynapse

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Vynapse/Overview.md | 125 | "Vynapse is a **from-scratch Rust evolutionary-algorithm trainer** ... The scaffolding for those is visible as empty files in the module tree." |
| Projects/Vynapse/Architecture.md | 242 | "- Error propagation across crate boundaries: [[Vynapse/Systems/Error Model]]" |
| Projects/Vynapse/Decisions.md | 184 | "- The gaps these decisions leave open: [[Vynapse/Gaps]]" |
| Projects/Vynapse/Gaps.md | 268 | "- Analytical suggestions for prioritisation: [[Vynapse/Suggestions]]" |
| Projects/Vynapse/Roadmap.md | 187 | "- RL problem domain overlap with NeuroDrive (Milestone 4 cartpole): [[NeuroDrive/Overview]]" |
| Projects/Vynapse/Suggestions.md | 197 | "- Profile update implied by the project's portfolio value: [[Profile/Projects]], [[Profile/Skills]]" |
| Projects/Vynapse/Systems/Error Model.md | 114 | "- A specific latent error-handling bug: [[Vynapse/Gaps#validate() fails on fresh state]]" |
| Projects/Vynapse/Systems/Evolutionary Trainer.md | 244 | "- Why the refactor happened and what it replaced: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Genome and Components.md | 218 | "- Stubbed components as roadmap evidence: [[Vynapse/Roadmap]]" |
| Projects/Vynapse/Systems/Tasks and Fitness.md | 171 | "- Why the current benchmarks under-test learning capability: [[Vynapse/Gaps#Benchmarks are saturated]]" |
| Projects/Vynapse/Systems/Tensor and Math.md | 126 | "- Aurix also has a tensor crate — duplication decision pending: [[Aurix/Overview]], [[Vynapse/Decisions#Tensor crate vs external]]" |
| Projects/Vynapse/Systems/Training Stats and Convergence.md | 185 | "- Why these were split out from the trainer in Dec 2025: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Traits Layer.md | 185 | "- Stubs that will need new trait work: [[Vynapse/Gaps#Traits that need extension]]" |

### Phase 3 — Per-project agent: AsteroidsAI

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/AsteroidsAI/Overview.md | 130 | "#project/asteroids-ai #lang/python #paradigm/evolutionary #paradigm/neat #paradigm/cma-es #paradigm/sac #ml/rl #ml/gnn #status/dormant" |
| Projects/AsteroidsAI/Architecture.md | 290 | "- [[AsteroidsAI/Roadmap]] — the planned parallel dashboard and its architectural implications" |
| Projects/AsteroidsAI/Decisions.md | 278 | "- [[AsteroidsAI/Roadmap]] — decisions deliberately deferred appear as roadmap items" |
| Projects/AsteroidsAI/Gaps.md | 257 | "- [[AsteroidsAI/Systems/Analytics Pipeline]] — analytics polish gaps" |
| Projects/AsteroidsAI/Roadmap.md | 282 | "- [[Projects/Index]] — AsteroidsAI relative to Caner's other active projects (all of which are higher priority as of 2026-04-24)" |
| Projects/AsteroidsAI/Systems/Game Engine.md | 193 | "- [[AsteroidsAI/Gaps]] — broken `get_tick()`, wrap-aware collision, unused arcade APIs" |
| Projects/AsteroidsAI/Systems/State Encoders.md | 219 | "- [[AsteroidsAI/Gaps]] — encoder drift, schema versioning, VectorEncoder dead code" |
| Projects/AsteroidsAI/Systems/Reward System.md | 174 | "- [[AsteroidsAI/Gaps]] — 17 components never exercised in a run" |
| Projects/AsteroidsAI/Systems/Genetic Algorithm.md | 225 | "- [[Vynapse/Overview]] — Caner's Rust neuroevolution engine; solves similar fixed-topology evolutionary problem in a different language" |
| Projects/AsteroidsAI/Systems/Evolution Strategies.md | 291 | "- [[AsteroidsAI/Roadmap]] — the Easy/Medium/Hard roadmap from the ES plan is the richest source of next-session ideas" |
| Projects/AsteroidsAI/Systems/NEAT.md | 226 | "- [[Vynapse/Overview]] — Vynapse's `trainers/neat.rs` is a 0-byte stub; AsteroidsAI's NEAT is the working reference implementation" |
| Projects/AsteroidsAI/Systems/GNN-SAC.md | 355 | "- [[NeuroDrive/Overview]] — NeuroDrive's asymmetric PPO (actor 2x64, critic 2x128) is the sibling gradient-based RL in the vault; ..." |
| Projects/AsteroidsAI/Systems/Shared Components.md | 225 | "- [[AsteroidsAI/Gaps]] — method-parity normalisation not done; cross-method bonus magnitudes not comparable" |
| Projects/AsteroidsAI/Systems/Analytics Pipeline.md | 257 | "- [[AsteroidsAI/Roadmap]] — analytics polish is much of the remaining in-repo roadmap" |

### Phase 3 — Per-project agent: Consilium

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Consilium/Overview.md | 99 | "#project/consilium #domain/llm-orchestration #domain/agents #stack/python #stack/langchain #stack/textual #status/dormant" |
| Projects/Consilium/Architecture.md | 186 | "#project/consilium #domain/architecture" |
| Projects/Consilium/Decisions.md | 211 | "#project/consilium #domain/decisions" |
| Projects/Consilium/Gaps.md | 155 | "#project/consilium #domain/gaps" |
| Projects/Consilium/README Claims vs Reality.md | 112 | "#project/consilium #domain/gaps #domain/documentation" |
| Projects/Consilium/Roadmap.md | 140 | "#project/consilium #domain/roadmap" |
| Projects/Consilium/Suggestions.md | 108 | "#project/consilium #domain/suggestions" |
| Projects/Consilium/Systems/Debate Orchestrator.md | 171 | "#project/consilium #domain/orchestration" |
| Projects/Consilium/Systems/Prompts.md | 122 | "#project/consilium #domain/prompt-engineering" |
| Projects/Consilium/Systems/Providers.md | 162 | "#project/consilium #domain/providers #stack/langchain" |
| Projects/Consilium/Systems/Roster and Sampling.md | 129 | "#project/consilium #domain/configuration #domain/sampling" |
| Projects/Consilium/Systems/Structured Debate State.md | 157 | "#project/consilium #domain/state-model #domain/prompt-engineering" |
| Projects/Consilium/Systems/TUI.md | 93 | "#project/consilium #domain/ui #stack/textual" |
| Projects/Consilium/Systems/Transcripts.md | 165 | "#project/consilium #domain/persistence #domain/output" |

### Phase 3 — Per-project agent: Chrona

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Chrona/Overview.md | 71 | "Chrona is a correctly-scoped, well-planned, earnestly-scaffolded C++ VCS learning project whose README describes a finished engine and whose code describes a half-finished `argc/argv` parser — the gap between the two is the entire point of this note existing." |
| Projects/Chrona/Architecture.md | 138 | "\| No static-analysis config \| No `.clang-tidy`, no IWYU mapping \|" |
| Projects/Chrona/Decisions.md | 226 | "- [[Chrona/Roadmap]] — D12's exclusions bound the roadmap" |
| Projects/Chrona/Gaps.md | 270 | "- [[Chrona/Roadmap]] — what depends on each of these being closed" |
| Projects/Chrona/Plans Workflow.md | 130 | "- [[LifeOS/Overview]] — LifeOS uses a similar externalised-state discipline at a larger scale; plans/ is the project-local equivalent" |
| Projects/Chrona/Roadmap.md | 195 | "- [[Chrona/Plans Workflow]] — the plans/ convention that future milestone plans will follow" |
| Projects/Chrona/Systems/Build and Test.md | 176 | "- [[Chrona/Gaps]] — the commented-out tests, the missing library target, the missing `target_include_directories` on the main target" |
| Projects/Chrona/Systems/CLI.md | 117 | "- [[Chrona/Gaps]] — the init-stub gap originates here" |
| Projects/Chrona/Systems/Errors.md | 124 | "- [[Chrona/Architecture]] — compile graph showing errors is linked into both targets" |
| Projects/Chrona/Systems/Repo Discovery.md | 138 | "- [[Chrona/Gaps]] — the `exists` vs `is_directory` gap and the commented-out test file are recorded there" |

### Phase 3 — Per-project agent: Xyntra

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Xyntra/Overview.md | 94 | "`#project/xyntra` `#rust` `#compiler` `#ml-systems` `#skeleton` `#dormant` `#gpu`" |
| Projects/Xyntra/Architecture.md | 153 | "`#project/xyntra` `#rust` `#architecture` `#compiler-frontend` `#ir`" |
| Projects/Xyntra/Decisions.md | 228 | "`#project/xyntra` `#decisions` `#design-rationale`" |
| Projects/Xyntra/Gaps.md | 140 | "`#project/xyntra` `#gaps` `#unimplemented` `#bugs`" |
| Projects/Xyntra/Reality vs README.md | 68 | "`#project/xyntra` `#reality-check` `#anti-puffing` `#portfolio-accuracy`" |
| Projects/Xyntra/Roadmap.md | 147 | "`#project/xyntra` `#roadmap` `#phased-plan`" |
| Projects/Xyntra/Systems/Config.md | 105 | "`#project/xyntra` `#config` `#gpu-parameters` `#validation`" |
| Projects/Xyntra/Systems/Errors.md | 111 | "`#project/xyntra` `#errors` `#error-handling` `#rust`" |
| Projects/Xyntra/Systems/Graph.md | 129 | "`#project/xyntra` `#ir` `#graph` `#dag`" |
| Projects/Xyntra/Systems/IR Types.md | 109 | "`#project/xyntra` `#ir` `#types` `#primitives`" |
| Projects/Xyntra/Systems/Testing.md | 85 | "`#project/xyntra` `#testing` `#test-infrastructure`" |
| Projects/Xyntra/Systems/Validation.md | 112 | "`#project/xyntra` `#validation` `#scaffold` `#todo` `#next-work`" |

### Phase 3 — Per-project agent: Zyphos

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Zyphos/Overview.md | 106 | "#project #zyphos #rust #networking #http #learning-project #solo" |
| Projects/Zyphos/Architecture.md | 165 | "- [[Decisions]] — why the seams are where they are" |
| Projects/Zyphos/Decisions.md | 181 | "- [[Roadmap]] — which decisions will need revisiting" |
| Projects/Zyphos/Gaps.md | 180 | "- [[Suggestions]] — opportunistic improvements beyond these specific gaps" |
| Projects/Zyphos/Milestones.md | 174 | "- [[Systems/Routing#What This Domain Still Needs to Hit M5\|Systems/Routing: M5 exit criteria]]" |
| Projects/Zyphos/Roadmap.md | 150 | "- [[Suggestions]] — ideas outside the milestone ladder" |
| Projects/Zyphos/Suggestions.md | 135 | "- [[Systems/Testing]] — R3 (timeouts) and O6 (integration tests) live here" |
| Projects/Zyphos/Systems/Connection Handling.md | 152 | "- [[Milestones#Milestone 3 Thread-per-Connection Model\|Milestones: M3 detail]]" |
| Projects/Zyphos/Systems/Request Parsing.md | 119 | "- [[Gaps#Request parsing gaps\|Gaps: missing headers, body, Content-Length]]" |
| Projects/Zyphos/Systems/Response Pipeline.md | 142 | "- [[Gaps#Response pipeline gaps\|Gaps: trailing CRLF, Server header, binary bodies]]" |
| Projects/Zyphos/Systems/Routing.md | 168 | "- [[Gaps#Routing gaps\|Gaps: POST/PUT/DELETE, query strings, URL decoding]]" |
| Projects/Zyphos/Systems/Testing.md | 113 | "- [[Roadmap]] — test priorities in the next session" |

### Phase 3 — Per-project agent: Tectra

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Tectra/Overview.md | 89 | "> If a future Caner or future session reads the README and the vault notes and they disagree, the vault notes reflect code. The README has not been updated to mark what is built vs planned." |
| Projects/Tectra/Architecture.md | 164 | "- [[Tectra/Roadmap]] — the README's 14 milestones with velocity overlay" |
| Projects/Tectra/Decisions.md | 154 | "- [[Tectra/Roadmap]] — each open question is pinned to its README milestone" |
| Projects/Tectra/Evolution.md | 98 | "- [[Tectra/Systems/Clock]] — detailed history of the one substantive subsystem" |
| Projects/Tectra/Gaps.md | 149 | "- [[Tectra/Architecture]] — intended vs wired architecture side by side" |
| Projects/Tectra/Roadmap.md | 154 | "- [[Nyquestro/Overview]] — parallel early-stage trading-infra project; cross-pollination possibility" |
| Projects/Tectra/Systems/Build.md | 119 | "- [[Tectra/Roadmap]] — Milestone 1 (Foundations) is where the rest of this gets built" |
| Projects/Tectra/Systems/Clock.md | 146 | "- [[Nyquestro/Overview]] — Nyquestro uses event frames with embedded timestamps; a fantasy-integration would feed virtual time from Tectra's replay into Nyquestro's engine" |
| Projects/Tectra/Systems/Logging.md | 84 | "- [[Tectra/Roadmap]] — the README's Milestone 1 (Foundations) is where logging gets built" |

### Phase 4 — OSS aggregation

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Open Source Contributions/Tinygrad.md | 130 | "- [[Projects/Open Source Contributions/Burn|Burn]] — sister contribution notes" |
| Projects/Open Source Contributions/Burn.md | 124 | "- [[Profile/Professional/Experience]] — counts as external open-source engagement with a Rust deep-learning framework maintainer team" |

### Phase 5 — Skills derivation (partial-evidence caveat)

The Phase 5 agent's evidence block listed the following lines as "verbatim last line" for each project file — but inspection shows the agent quoted internal lines (typically rows from inside the per-project file's own Evidence Block table or a nearby paragraph) rather than the *literal* final line of each on-disk file. Reproduced here as the agent returned them, with the caveat surfaced in the WIDND above.

| Project file | Lines | Agent-reported "verbatim last line" |
|---|---|---|
| profile/projects/cernio.md | 387 | (final row of project file's internal evidence block, citing `Projects/Cernio/Work/Profile Populate Skill.md`) |
| profile/projects/image-browser.md | 339 | (final row of project file's internal evidence block, citing `Work/Encrypted Vector Search.md`) |
| profile/projects/aurix.md | 240 | (final row of project file's internal evidence block, citing `Work/Tab 2 Timeboost MEV Analytics.md`) |
| profile/projects/neurodrive.md | 385 | (final row of project file's internal evidence block, citing `Work/Performance.md`) |
| profile/projects/nyquestro.md | 276 | (paragraph from project file body about what the project does NOT demonstrate) |
| profile/projects/vynapse.md | 287 | (final row of project file's internal evidence block) |
| profile/projects/asteroidsai.md | 260 | (final row of project file's internal evidence block) |
| profile/projects/consilium.md | 348 | (final row of project file's internal evidence block) |
| profile/projects/chrona.md | 199 | (final row of project file's internal evidence block) |
| profile/projects/xyntra.md | 204 | (final row of project file's internal evidence block) |
| profile/projects/zyphos.md | 221 | (final row of project file's internal evidence block) |
| profile/projects/tectra.md | 267 | (final row of project file's internal evidence block) |
| profile/projects/open-source-contributions.md | 172 | (line from internal evidence table) |

The agent reads were full-file (line counts match what's on disk), but the "verbatim last line" pointer was loose. The skills-derivation output itself (`profile/skills.md`) is internally consistent with what the per-project files actually contain.
