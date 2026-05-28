---
title: Cernio Full Refactor — Lane-Based Relative Grading
status: design-locked-pending-implementation
date: 2026-05-28
session_context: full design discussion in chat 2026-05-28; supersedes the now-deleted lane-based-redesign.md and location-expansion.md
---

# Cernio Full Refactor — Lane-Based Relative Grading

A complete redesign of Cernio's grading semantic, discovery, search, and supporting skills. The system shifts from **global single-scale grading anchored to hardcoded examples** to **lane-relative grading where all calibration emerges from comparison within the dataset**.

Every skill in `.claude/skills/`, every Rust script, the database schema, the TUI, and the profile layer are affected.

This plan captures **design decisions only**. Execution is described as an **agent-orchestration model** at the end. No implementation has started.

---

## Table of contents

1. [Core principles](#1-core-principles)
2. [Strategic foundation — lanes, goals, ceilings](#2-strategic-foundation)
3. [Database schema changes](#3-database-schema-changes)
4. [Profile layer — what's dynamic, what isn't](#4-profile-layer)
5. [The skill redesigns — phased orchestration](#5-the-skill-redesigns)
6. [The Rust scripts](#6-the-rust-scripts)
7. [TUI changes](#7-tui-changes)
8. [Context docs and README](#8-context-docs-and-readme)
9. [Migration — jobs reset, companies preserved](#9-migration)
10. [Explicitly out of scope](#10-explicitly-out-of-scope)
11. [Execution model — parallel background-agent orchestration](#11-execution-model)

---

## 1. Core principles

These principles govern every decision in this refactor. When implementation choices conflict, defer to these:

### Don't hardcode anything that can be derived

No file says "Anthropic is SS in AI/ML". No rubric says "Amazon SDE-1 Grad is the worked-example anchor for SS". No skill embeds a profile snapshot. Calibration emerges from comparison within the dataset.

Concrete implications:
- **No `profile/lanes.md`** — the lane list lives in `career-goals.md` (just the 8 names + brief descriptions); the per-lane pinnacle ranking lives in the database (`pinnacle_status_per_lane` per company, derived by `grade-companies`)
- **No hardcoded calibration anchors** — relative grading inside `grade-jobs` and `grade-companies` derives ranking from within-lane comparison phases
- **No hardcoded "what does HFT want"** — the per-lane portfolio gaps emerge from grading runs, not from authored content
- Lanes themselves can change over time (a 9th lane added; one folded) — change them in `career-goals.md` and the rest of the system reads from there

### All companies must sponsor UK Skilled Worker visas

Cernio's universe is **sponsor-only**. Companies that don't sponsor can't be applied to anyway, so they should never enter the DB. `sponsors_uk` is a mandatory verified field at company-creation time. Non-sponsoring companies are rejected at discovery; they don't reach `grade-companies`.

### Skill-owned artefact maintenance

Folders and files that Cernio uses as durable memory — `profile/skills/`, `profile/portfolio-gaps/` — are **maintained by skills, not by humans editing markdown**. `populate-from-lifeos` maintains `profile/skills/`. `grade-jobs` maintains `profile/portfolio-gaps/`. The skills are responsible for the curation pass that rewrites these files in place at the end of each invocation.

This means the appropriate way to "update skills" is to invoke `populate-from-lifeos`, not to edit `profile/skills/languages.md` by hand. Manual edits are valid (the user can always do them) but the canonical maintenance path is the skill.

### Lane-relative everything

A company is graded against its lane peers, not globally. A job is graded against its lane peers, not globally. Cross-lane comparisons (Monzo vs Anthropic) happen at the user-decision level, not at the rubric level.

### Realism semantic preserved (anti-prestige-trap)

The 2026-04-29 realism semantic — decoupling reputation from selectivity to prevent prestige-trap inflation — **carries forward into lane-relative grading**. The reframe is purely additive:

- Previously: "best accessible job overall"
- Now: "best accessible job within its lane, for each lane"

The "accessibility" half doesn't change. SS in a lane still requires Q1 cleared decisively (the user can actually clear the bar) AND strong Q2–Q5 (reputation, role-truth, profile-fit, visa). Brand-name pinnacle-of-lane employers with narrow pipelines still cap at A-stretch within their lane, exactly as they did globally pre-refactor. HRT grad SWE = SS-in-hft (wide-funnel + reputation); Jane Street Tech = S-in-hft (narrow selectivity vs reputation).

Implementation guard: the Phase 2 consistency pass MUST compare on the combined Q1–Q5 evidence within a lane, not on Q3 (pinnacle position) alone. Pinnacle status influences but never determines grade. The risk being guarded against is "Jane Street is HFT-pinnacle, therefore every Jane Street role is SS-in-hft" — that's the prestige trap in lane form, and it's prohibited.

### Role-truth-at-hire is a first-class rubric dimension

The role function (engineering / quant / research / strats) must be locked at day one. Vertical climb within function is fine. Cross-function transitions hoping for a lateral hop are auto-downgraded. This applies in `grade-jobs` and is the rubric Q-slot that handles it.

---

## 2. Strategic foundation

### The eight active lanes

Declared in `profile/career-goals.md`. Just names + brief role-shape descriptions — no hardcoded pinnacle lists.

| Lane key | Description (one line) |
|---|---|
| `big-tech` | Generalist software engineering at major tech employers; standard SWE ladder |
| `ai-ml` | ML research and engineering — model training, inference systems, applied ML |
| `hft` | Low-latency systems, market data, exchange connectivity, lock-free engineering at proprietary trading firms |
| `crypto-mm` | Market-making systems and DeFi engineering for digital-asset firms |
| `bank-strats` | Engineering function inside bank S&T divisions — e-trading platforms, strategist tooling, electronic execution |
| `systems-infra` | Low-level systems, compilers, databases, distributed infrastructure, OSS-aligned work |
| `devtools` | Developer tooling and developer-experience products |
| `fintech` | Payments, neobanks, financial-product software at non-bank fintech employers |

### Strategy

**Strategy A — Prestige exit → independent contracting.** Build brand asset at pinnacle employers for 5–8 years, then exit to remote independent contracting. Implicit because several lanes (HFT, Bank Strats) aren't remote-during-career and only make sense under prestige-exit.

**No lane priority during junior phase.** Any of the 8 landing is a major win.

### Role-truth-at-hire

The role function must be locked at day 1. Specifically excluded:
- IBD analyst path (wrong function for engineering profile, and entirely cross-function for promotion)
- Solutions Architect / Technical Programme Manager / Sales Engineer / Product Manager / Operations Analyst / Data Analyst → SWE lateral hopes
- "Global Markets Graduate" rotation lotteries
- Any role where the JD describes a junior function with implied promotion into a different function

### Sponsor-only universe

Every company in the database must sponsor UK Skilled Worker visas. Companies that don't sponsor are rejected at discovery and never enter the DB.

### Locations (commute-belt + relocation candidates)

Ceiling: **~2h 45 door-to-door from Highams Park / Stratford** based on verified Bristol ≈ 2h 40 acceptable vs Liverpool ≈ 3h unacceptable.

| Tier | Cities | Office-frequency rule |
|---|---|---|
| **Tier 1 (daily-commute)** | London, Reading, Cambridge, Oxford, Milton Keynes, Brighton, Guildford, Watford, Stevenage, Luton, Newbury | Any frequency acceptable |
| **Tier 2 (stretch hybrid)** | Bristol, Bath, Birmingham, Coventry, Northampton, Sheffield, Nottingham, Derby | ≤2 days/wk acceptable; 3+ days/wk downgrade |
| **Tier 3 (relocation)** | Edinburgh, Glasgow | Treated as relocation-candidates; graded normally with relocation flag |
| **Remote** | Any city if role is fully UK-remote with sponsorship | Graded normally |
| **Excluded** | Manchester, Leeds, Liverpool, Newcastle, Cardiff (verified over ceiling or thin lane density) | — |

Hybrid-unspecified JDs at Tier 2 default to optimistic-accept with a risk flag; `grade-jobs` agent reads the JD carefully for office-day signals and downgrades if 3+ days surface.

---

## 3. Database schema changes

### `companies` table additions

| Column | Type | Purpose |
|---|---|---|
| `lanes` | JSON array of lane keys | Which of the 8 lanes the company has teams in; multi-tag |
| `pinnacle_status_per_lane` | JSON map `{lane → pinnacle/strong/adjacent/borderline}` | Within-lane positioning, derived by grade-companies |
| `sponsors_uk` | enum `yes/no/unknown` (with `no` causing deletion) | Verified visa sponsorship; mandatory verified-yes for retention |

### `companies` existing fields

`grade` stays as a single generalised employer-quality grade (SS/S/A/B/C/F). This is *not* a per-lane grade — it's the overall "what kind of employer is this".

### `jobs` table changes

| Column | Change | Purpose |
|---|---|---|
| `lanes` | NEW — JSON array of lane keys | Multi-tag, primary lane is first |
| `grade` | Keep but redefine — lane-relative SS/S/A/B/C/F | Graded against primary lane's peers |
| `fit_assessment` | Q-slot structure updated (see §5 grade-jobs) | New 5-slot rubric |
| `evaluation_status` | Keep | Coarser bucketing |
| `evidence_basis` | Keep | jd / semantic / insufficient |

### `application_packages` table

Deleted entirely. `prepare-applications` skill is being killed; its DB table goes with it.

### Schema migration

- ALTER companies ADD COLUMN lanes, pinnacle_status_per_lane, sponsors_uk
- ALTER jobs ADD COLUMN lanes; redefine grade semantic via documentation, no schema change
- DELETE FROM application_packages; DROP TABLE application_packages
- DELETE FROM jobs (full reset — see §9 migration)
- Existing companies preserved; backfill lanes + pinnacle_status_per_lane + sponsors_uk via `grade-companies` re-run

---

## 4. Profile layer

### What's static (human-maintained)

| File | Content |
|---|---|
| `profile/career-goals.md` | The 8 active lane keys + descriptions; Strategy A statement; hard rules (role-truth, no IBD, sponsor-required); soft preferences (locations, office frequency); time horizon |
| `profile/preferences.toml` | Operational config — locations by tier, ATS provider patterns, search filters |
| `profile/Personal.md`, `profile/Experience.md`, etc. | Existing LifeOS-synced files — maintained via `populate-from-lifeos` |

### What's skill-maintained (dynamic)

| Folder | Owner skill | Content |
|---|---|---|
| `profile/skills/` | `populate-from-lifeos` | Per-group skill files: `_Overview.md`, `languages.md`, `systems-low-level.md`, `ml-ai.md`, `infra-tooling.md`, `data.md`, `web-frontend.md`, `finance-domain.md`. Project anchors live here. Groups chosen because per-lane would duplicate (Rust appears in 5 lanes) |
| `profile/portfolio-gaps/` | `grade-jobs` | Per-lane gap files: `_Overview.md`, `<lane-key>.md` per active lane, `closed.md`. Rewritten in place at end of each grade-jobs batch. Not append-only. |

### Folder shapes

```
profile/
  career-goals.md                — strategy, lanes, rules, preferences
  preferences.toml               — operational config
  Personal.md                    — LifeOS-synced
  Experience.md                  — LifeOS-synced
  Education.md                   — LifeOS-synced
  ... (other LifeOS-synced files) ...
  skills/                        — maintained by populate-from-lifeos
    _Overview.md
    languages.md
    systems-low-level.md
    ml-ai.md
    infra-tooling.md
    data.md
    web-frontend.md
    finance-domain.md
  portfolio-gaps/                — maintained by grade-jobs
    _Overview.md
    big-tech.md
    ai-ml.md
    hft.md
    crypto-mm.md
    bank-strats.md
    systems-infra.md
    devtools.md
    fintech.md
    closed.md
```

The exact group split inside `profile/skills/` is determined by `populate-from-lifeos` based on the LifeOS profile content — the skill chooses the appropriate groups, not the user editing markdown.

---

## 5. The skill redesigns

Every skill in `.claude/skills/` is touched. Each gets a phased structure where appropriate — initial pass + cross-check/consistency pass + curation pass.

### 5.1 `discover-companies` — lane-aware, sponsor-filtered

**New phase structure:**

- **Phase 1 — Per-lane parallel discovery.** ~8 parallel agents (one per active lane). Each agent uses lane-specific source heuristics:
  - HFT: prop-trading registries, low-latency conference attendee/sponsor lists, kdb+/q job board crawls
  - Crypto MM: on-chain DEX maker addresses backtraced to entities, crypto-conference sponsors, GitHub crypto-org star patterns
  - AI/ML: AI Safety org lists, arXiv-affiliated startups, accelerator demo days
  - Bank Strats: bank careers pages, e-trading vendor partnerships, alumni LinkedIn patterns
  - Systems / Infra: OSS-foundation sponsor lists (Rust, Linux, CNCF), DB-vendor directories
  - Devtools: HN Show HN tracking, YC devtools cohorts, dev-tooling directories
  - Fintech: UK fintech accelerator graduates, FCA-authorised firm lists
  - Big-Tech: established sources (already largely covered)
  - Each agent writes to `companies/discovery-<lane>-<date>.md`
- **Phase 2 — Sponsor verification + dedup.** For each newly-discovered company: verify it sponsors UK Skilled Worker (check Gov.uk Skilled Worker sponsor register OR direct careers-page evidence). Deduplicate against existing DB. Non-sponsors are dropped here, not later.
- **Phase 3 — Import via `cernio import`.** Validated companies enter the DB as `potential`. Cascade into `populate-db` / `resolve-portals` for ATS resolution.

### 5.2 `populate-db` — sponsor-gated, lane-tagged at resolve

Existing mechanical script (`cernio resolve`) probes ATS slugs. Skill wraps it with:

- **Phase 1 — Sponsor verification (precondition).** Confirm `sponsors_uk = yes` before any ATS work. Reject otherwise.
- **Phase 2 — `cernio resolve` mechanical probe** (existing).
- **Phase 3 — AI fallback for unmatched** (existing `resolve-portals` behaviour, folded in here).
- **Phase 4 — Hand off to `grade-companies`** for initial lane assignment + pinnacle status + employer grade.

### 5.3 `resolve-portals` — minor changes

AI fallback for the mechanical resolver. Mostly unchanged. Add sponsor-verification gate (don't AI-resolve a non-sponsor company).

### 5.4 `populate-from-lifeos` — owns `profile/skills/` maintenance

**New phase structure:**

- **Phase 1 — Read LifeOS canonical files** via `gh api` (existing).
- **Phase 2 — Sync flat profile files** 1:1 from LifeOS (existing).
- **Phase 3 — Synthesise per-project files** via parallel subagents (existing).
- **Phase 4 — NEW: Maintain `profile/skills/` folder.** Skill chooses appropriate groups (languages, systems-low-level, ml-ai, infra-tooling, etc.) based on the synthesised profile content. Rewrites each group file in place. Project anchors live inside group files alongside the skills they demonstrate.
- **Phase 5 — Sync summary** (existing).

The skill owns the group taxonomy decision — chooses groups that fit the actual skill content rather than a hardcoded list.

### 5.5 `grade-companies` — phased, lane-relative, sponsor-gated

**New phase structure:**

- **Phase 1 — Initial pass.** Parallel agents (~20 batched across ungraded companies). Each agent:
  - Verifies `sponsors_uk` (if not already verified)
  - Assigns `lanes` array from research (which of the 8 lanes does this company have teams in?)
  - Initial `pinnacle_status_per_lane` per lane the company tags (pinnacle / strong / adjacent / borderline)
  - Initial generalised `grade` (employer quality, SS/S/A/B/C/F)
  - Writes prose reasoning per assignment
- **Phase 2 — Relative consistency pass within lanes.** ~10 parallel agents. Each agent gets a random sample of ~50 companies (potentially spanning multiple lanes); compares within-lane positioning for consistency; adjusts `pinnacle_status_per_lane` where similar companies have inconsistent classifications. This is the "no hardcoded anchor" mechanism — calibration emerges from comparison.
- **Phase 3 — No-lane deletion sweep.** Companies that ended Phase 2 with empty `lanes` array AND no overwhelming reason to expand the lane list → deleted entirely (cascading delete on associated jobs).

### 5.6 `grade-jobs` — phased, lane-relative, role-truth-aware, gap-curating

**New phase structure:**

- **Phase 1 — Initial pass.** ~20 parallel agents (each ~50 jobs). Each agent:
  - Reads JD + company context (including company's `lanes` and `pinnacle_status_per_lane`)
  - Assigns job `lanes` (primary + secondary tags)
  - Q1 — Lane assignment with prose justification
  - Q2 — Role-truth-at-hire check (function locked at day 1?)
  - Q3 — Within-lane pinnacle position (using company's `pinnacle_status_per_lane`)
  - Q4 — Profile fit within lane (cite specific project anchors from `profile/skills/`)
  - Q5 — Visa + soft-preference compatibility
  - Initial grade (SS/S/A/B/C/F, lane-relative to primary lane)
- **Phase 2 — Relative consistency pass.** ~10 parallel agents. Each gets 100 random jobs; compares grades within-lane for consistency; adjusts where peers diverge without justified reason. Same mechanism as grade-companies Phase 2 — calibration from comparison.
- **Phase 3 — Per-lane portfolio-gaps regeneration.** ~8 parallel agents (one per active lane). Each reads all jobs in its lane; extracts patterns (what the lane wants that the profile lacks; what the profile has that the lane values); rewrites `profile/portfolio-gaps/<lane>.md` in place. Then a single coordinator rewrites `_Overview.md`. The `closed.md` file accumulates gaps that have been closed (those move from per-lane file to closed.md when the closure is confirmed).

### 5.7 `search-jobs` — sponsor-only, lane-aware

**New phase structure:**

- **Phase 1 — Pre-filter** to sponsor-verified companies only (existing DB constraint; verify mechanically).
- **Phase 2 — Per-provider mechanical search** via `cernio search` for resolved-ATS companies (existing).
- **Phase 3 — Bespoke search via subagents** for unresolved (existing). Subagents are now lane-aware — the search query includes lane context for better targeting.
- **Phase 4 — Insert into DB** as pending jobs. Hand off to `grade-jobs`.

### 5.8 `check-integrity` — lane + sponsor coherence added

**New phase structure:**

- **Phase 1 — Mechanical** (`cernio check` + `cernio format`) (existing).
- **Phase 2 — Profile-driven staleness** (existing — compare profile mtime vs `graded_at`).
- **Phase 3 — NEW: Lane coherence.** Every company has at least one lane tag. Every job's lane(s) is a subset of its company's lane(s). Every active lane in `career-goals.md` has at least N companies tagged to it (sanity).
- **Phase 4 — NEW: Sponsor coherence.** Every company in the DB has `sponsors_uk = yes` (no `unknown` should persist past discovery; `no` shouldn't exist).
- **Phase 5 — Portfolio-gaps freshness.** Each per-lane gap file's last-updated timestamp is checked against the most recent grade-jobs run for that lane; surface stale ones.

### 5.9 `prepare-applications` — deleted

Skill removed entirely. The `application_packages` DB table is dropped. The `p` key in the TUI that autofills from this table is removed.

### 5.10 `test-grade-jobs` — lane-aware axes

The 7 existing axes are restructured to test the new lane-aware grading:

- Axis A — Lane assignment correctness
- Axis B — Role-truth-at-hire detection (does the rubric correctly auto-downgrade cross-function-transition roles?)
- Axis C — Within-lane relative consistency (do agents grade similar-pinnacle-position roles similarly within a lane?)
- Axis D — Cross-lane independence (does grading in HFT influence grading in Fintech? It shouldn't)
- Axis E — Sponsor-status accuracy
- Axis F — Phase 2 consistency-pass effectiveness (does Phase 2 actually correct Phase 1 drift?)
- Axis G — Q-slot structure adherence (no hardcoded anchor language, no banned-tokens leak)

Baseline + iteration tracking continues as before.

---

## 6. The Rust scripts

### `cernio resolve` — sponsor gate

Add a pre-flight check: company must have `sponsors_uk = yes` before ATS-slug probing.

### `cernio search` — sponsor + lane filters

- Filter source companies to `sponsors_uk = yes` only
- Lane filter (optional CLI flag `--lane hft` to search within one lane)
- Insert resulting jobs with `lanes` populated from company's `lanes` array (Phase 1 of grade-jobs will refine)

### `cernio check` — lane + sponsor coherence

Extend mechanical checks:
- Every company has non-empty `lanes`
- Every company has `sponsors_uk = yes`
- Every job's `lanes` ⊆ its company's `lanes`
- Every active lane in `career-goals.md` is tagged to ≥ N companies

### `cernio format` — unchanged

Mechanical formatting. No semantic change.

### `cernio import` — sponsor gate at import

Reject imports where `sponsors_uk` isn't verified-yes. Surface the rejection rather than silently dropping.

### `cernio clean` — extended

Add lane-aware cleanup: delete companies with empty `lanes` after a grade-companies run.

### `cernio unarchive` — unchanged

Mechanical un-archiving. No semantic change.

### `cernio stats` / `cernio pending` — lane filters

Extend with optional lane filters for per-lane reporting.

---

## 7. TUI changes

The TUI's 5 views need lane awareness:

- **Companies view** — show `lanes` column (comma-separated lane keys); show `pinnacle_status_per_lane` summary; show `sponsors_uk` badge; lane filter (`L` key cycles lanes)
- **Jobs view** — show `lanes` column (primary lane first); lane filter (`L` key); office-tier badge for location (T1/T2/T3)
- **Pending view** — show `lanes` even if grade isn't set yet (lane assigned at Phase 1 of grade-jobs)
- **Saved view** — same
- **Application view** — `prepare-applications` skill is gone; the `p` key removed; application packages table dropped; this view either gets repurposed or removed

NEW: **Lane summary view** — per active lane, show graded count, SS/S count, top open SS/S jobs. Helps the user see lane-by-lane progress at a glance.

The TUI lane-filter UX needs design — probably cycle through `all → big-tech → ai-ml → hft → crypto-mm → bank-strats → systems-infra → devtools → fintech → all`.

---

## 8. Context docs and README

### `context/architecture.md`

Rewrite the grading section. Document:
- Lane-based relative grading semantic
- Company `lanes` + `pinnacle_status_per_lane` + `sponsors_uk` fields
- Job `lanes` field + lane-relative grade semantic
- The phased structure of `grade-jobs` and `grade-companies`
- The skill-owned artefact maintenance (skills/, portfolio-gaps/)

### `context/notes/grading-rubric.md`

Supersede with iter4 notes:
- Tier-retirement language removed
- Wide-funnel-anchor semantic removed
- Single-anchor (Amazon SDE-1) calibration removed
- New: lane-relative emergent calibration via Phase 2 consistency pass

### `context/notes/lanes-taxonomy.md` (NEW)

Brief stable reference for the 8 lane keys + descriptions. Mirrors `profile/career-goals.md` content for the lane keys (intentional small duplication so context-readers don't need to fetch profile).

### `context/notes/sponsor-only-policy.md` (NEW)

Document the sponsor-only universe policy: discovery rejects non-sponsors, schema enforces sponsors_uk = yes, check-integrity verifies.

### `README.md`

Update the project description:
- "Lane-aware job discovery and curation engine" (instead of generic)
- Document the 8 lanes
- Document the sponsor-only constraint
- Document the relative-grading semantic
- Remove references to `prepare-applications` skill

---

## 9. Migration

### What gets reset

- **Jobs table — FULL RESET.** All 1,075 currently-graded jobs are deleted. They were graded under iter1 + global scale; re-grading them is more work than re-discovering them fresh. After lane-aware `search-jobs` runs, the new jobs land with lane tags and get graded under iter4 directly.
- **Application packages — DELETED.** Whole table dropped.

### What gets preserved

- **Companies — PRESERVED.** Existing companies stay but get:
  - `lanes` backfilled via `grade-companies` re-run
  - `pinnacle_status_per_lane` derived
  - `sponsors_uk` verified per company (those failing verification are deleted)
  - Generalised `grade` re-computed under lane-aware semantic (likely shifts for some)
- **Profile data — PRESERVED.** LifeOS-sourced canonical files. `populate-from-lifeos` will regenerate `profile/skills/` on next invocation.
- **portfolio-gaps.md (legacy single file) — DELETED.** Will be replaced by `profile/portfolio-gaps/` folder, generated fresh by `grade-jobs` on first lane-aware run.
- **Skill logs in LifeOS — PRESERVED.** Historical record.

### Migration order

1. Schema migration (companies columns added; jobs table truncated; application_packages dropped)
2. `populate-from-lifeos` run → regenerates `profile/skills/` folder
3. `grade-companies` Phase 1+2 run on all existing companies → assigns lanes, pinnacle status, sponsor verification, employer grade
4. `cernio clean` no-lane deletion sweep
5. `discover-companies` run (lane-aware) → expands universe via lane-targeted discovery
6. `cernio resolve` + `resolve-portals` for new companies
7. `search-jobs` run → re-discovers jobs across the lane-aware universe
8. `grade-jobs` Phase 1+2+3 run → grades fresh jobs, generates `profile/portfolio-gaps/`
9. `check-integrity` to verify lane + sponsor coherence

This pipeline runs end-to-end as part of the refactor's "first full activation".

---

## 10. Explicitly out of scope

- **Interview prep per lane** — handled by a separate project, not Cernio
- **`prepare-applications`** — being killed entirely
- **Time windows / deadline tracking** — external concern
- **Application throughput feedback loop** — not in this refactor (could be future work)
- **International locations** beyond UK (Berlin, Amsterdam, Dublin) — separate decision
- **Lane priority weighting in preferences** — explicitly deferred (junior phase doesn't need it)
- **Hardcoded calibration anchors** — explicitly prohibited

---

## 11. Autonomous execution playbook

This section operationalises the refactor for **fire-and-forget execution**: no user confirmation gates. Safety is enforced mechanically — pre-flight backups, per-wave commits, automated verification, contract checks between waves, halt-with-diagnostic on unrecoverable failure, and a status dashboard the user reads on return.

### 11.1 Pre-flight phase (Phase A) — ~5 min, sequential

Runs before any skill iteration. Bootstraps the safety net.

1. **Create refactor branch.** `git checkout -b refactor/cernio-full-relativity` from current `main`.
2. **Back up the database.** Copy SQLite DB to `/tmp/cernio-pre-refactor-<timestamp>.sqlite.bak`. Also save SHA256.
3. **Back up `profile/portfolio-gaps.md`** (legacy file about to be deleted) to `/tmp/cernio-pre-refactor-portfolio-gaps.md`.
4. **Verify prerequisites:**
   - LifeOS reachable: `gh api repos/Capataina/LifeOS` returns 200
   - All 10 skills present in `.claude/skills/` (discover-companies, populate-db, resolve-portals, populate-from-lifeos, grade-companies, grade-jobs, search-jobs, check-integrity, prepare-applications, test-grade-jobs)
   - `skill-creator` available globally at `~/.claude/skills/skill-creator/`
   - `cargo build --release` passes on current code (baseline must compile before we start changing things)
   - `cargo test` passes on current code (baseline must be green)
5. **Initialise status dashboard** at `/tmp/cernio-refactor-status.md` (format in §11.7).
6. **Commit checkpoint:** `git commit -am "refactor: pre-flight checkpoint"`. Tag commit hash in status dashboard.

If any pre-flight step fails: halt, write diagnostic, do not proceed.

### 11.2 Skill iteration waves (Phase B) — ~2.5 hours

Each wave runs its skills in parallel as background agents; waves run sequential.

#### Wave 1 — Profile foundation

| Skill | What it iterates per plan §5.4 |
|---|---|
| `populate-from-lifeos` | Adds Phase 4: maintains `profile/skills/` folder; chooses group taxonomy from actual LifeOS profile content |

Run as single background agent (no parallelism within wave). After completion, run the iterated skill to actually produce `profile/skills/` folder content (otherwise Wave 2 has no skills data to grade against).

#### Wave 2 — Grading skills (parallel)

| Skill | What it iterates per plan §5.5, §5.6 |
|---|---|
| `grade-companies` | Phased structure (Phase 1 initial + Phase 2 relative consistency + Phase 3 no-lane deletion); lane-relative semantic; sponsor-gating; pinnacle_status_per_lane |
| `grade-jobs` | Phased structure (Phase 1 initial + Phase 2 relative consistency + Phase 3 portfolio-gaps regeneration); new 5-slot Q-structure (lane assignment, role-truth-at-hire, within-lane pinnacle, profile fit, visa); realism semantic preservation guard |

Both run as parallel background agents. Both must complete before Wave 3 starts.

**Critical:** the two skills share `context/notes/grading-rubric.md` if it's referenced. To avoid concurrent edits, the orchestrator assigns one agent to own that file (grade-jobs owns it as the more rubric-heavy skill); the other reads only.

#### Wave 3 — Discovery, search, ATS resolution (parallel)

| Skill | What it iterates per plan §5.1, §5.2, §5.3, §5.7 |
|---|---|
| `discover-companies` | Per-lane parallel discovery with lane-specific source heuristics; sponsor verification + dedup phase |
| `populate-db` | Sponsor-gating precondition; existing mechanical probe + AI fallback flow |
| `resolve-portals` | Sponsor-verification gate before AI-resolving |
| `search-jobs` | Sponsor + lane filters; lane-aware bespoke search subagents |

All four run as parallel background agents.

#### Wave 4 — Integrity + testing (parallel)

| Skill | What it iterates per plan §5.8, §5.10 |
|---|---|
| `check-integrity` | Lane coherence checks + sponsor coherence + portfolio-gaps freshness |
| `test-grade-jobs` | New 7-axis structure for lane-aware testing |

Both run as parallel background agents.

#### Wave 5 — Deletion (sequential)

| Action | What |
|---|---|
| Delete `.claude/skills/prepare-applications/` entirely | `rm -rf` |
| Remove from `settings.json` if listed | Edit |
| Note for TUI track: `p` key removal + view removal | Logged for code track |

No skill iteration; direct file operations. Commit.

### 11.3 Rust code track (Phase C) — parallel with Waves 3–5 — ~45 min

Runs as its own track, started in parallel with Wave 3. Three sub-tracks:

#### 11.3a Schema migration

Sequential, runs first in Phase C:
1. Run schema migration against `/tmp/cernio-pre-refactor-<timestamp>.sqlite.bak` first; verify success
2. Apply same migration to live DB
3. `cargo test` — must pass (verify nothing else broke)

Migration SQL:
```sql
ALTER TABLE companies ADD COLUMN lanes TEXT;             -- JSON array
ALTER TABLE companies ADD COLUMN pinnacle_status_per_lane TEXT;  -- JSON map
ALTER TABLE companies ADD COLUMN sponsors_uk TEXT;       -- 'yes' / 'no' / 'unknown'
ALTER TABLE jobs ADD COLUMN lanes TEXT;                  -- JSON array
DELETE FROM jobs;                                        -- Full reset
DROP TABLE IF EXISTS application_packages;
```

#### 11.3b Rust script updates

Parallel sub-tasks after schema migration:
- `cernio resolve` — sponsor-gate at start
- `cernio search` — sponsor + lane filters
- `cernio check` — lane + sponsor coherence
- `cernio clean` — no-lane deletion sweep
- `cernio import` — sponsor-gate at import
- `cernio stats`, `cernio pending` — lane filter flags

Each change → `cargo build` → `cargo test`. If build/test fails, retry with adjusted approach up to 3 times; halt with diagnostic on third failure.

#### 11.3c TUI updates

Parallel sub-track:
- Companies view: lanes column + pinnacle_status display + sponsors_uk badge + `L` key lane filter
- Jobs view: lanes column + office-tier badge + `L` key lane filter
- Application view: removed entirely
- New lane summary view: per-lane graded count + SS/S count + top open jobs
- `p` key handler: removed (prepare-applications gone)

Each change → `cargo build` → `cargo test`. Same retry policy.

After 11.3a + 11.3b + 11.3c all complete: commit `refactor: rust + schema + tui updates`.

### 11.4 Activation pipeline (Phase D) — ~90 min, sequential

Runs after all skill waves + Rust track complete. The 9-step pipeline from §9:

| Step | Action | Status-write |
|---|---|---|
| 1 | Schema migration already applied in 11.3a | `[X] Schema migration` |
| 2 | Run `populate-from-lifeos` — regenerates `profile/skills/` | `[X] LifeOS sync` |
| 3 | Run `grade-companies` Phase 1+2 on existing companies | `[X] Companies graded (N companies)` |
| 4 | Run `cernio clean` no-lane deletion sweep | `[X] Deleted N no-lane companies` |
| 5 | Run `discover-companies` (per-lane parallel) | `[X] Discovered N new companies` |
| 6 | Run `cernio resolve` + `resolve-portals` for new | `[X] Resolved N companies` |
| 7 | Run `search-jobs` | `[X] Discovered N jobs` |
| 8 | Run `grade-jobs` Phase 1+2+3 | `[X] Graded N jobs; portfolio-gaps regenerated` |
| 9 | Run `check-integrity` | `[X] Integrity verified` or `[!] N issues` |

Halt-with-diagnostic on any step failure. Commit after each successful step: `refactor activation: step N — <description>`.

### 11.5 Verification gates

**Per-skill verification (after each background agent completes, before commit):**

Mechanical checks on the iterated skill:
1. Skill structure intact — `SKILL.md` exists, `references/` exists if it existed before
2. No `lanes.md` created anywhere — `find . -name lanes.md` returns empty
3. No hardcoded company-name calibration in rubric prose — `grep` for company names from the lane definitions in the rubric body (Anthropic, Google, HRT, Jane Street, Wintermute, etc.) — these should appear only in description/comments, not as "X is SS" anchors
4. Phased structure present where required — `grep` for "Phase 1", "Phase 2" in SKILL.md for grading skills
5. Skill log file written if skill was invoked during test
6. Plan section §5.N references match actual skill behaviour

Failure → halt wave, write diagnostic, do not commit.

**Inter-wave contract checks:**

| Boundary | Check |
|---|---|
| After Wave 1 → before Wave 2 | `profile/skills/_Overview.md` exists; at least 3 group files exist |
| After Wave 2 → before Wave 3 | Both grading skills' `SKILL.md` reference `lanes` column, `pinnacle_status_per_lane`, `sponsors_uk`; both have Phase 1/Phase 2/Phase 3 structure; realism-semantic guard text present in grade-jobs |
| After Wave 3 → before Wave 4 | All four discovery/search skills reference `sponsors_uk = yes` filter and lane-aware logic |
| After Wave 4 → before Wave 5 | `check-integrity` references lane + sponsor coherence checks; `test-grade-jobs` references lane-aware axes |
| After Wave 5 → before Phase D | `.claude/skills/prepare-applications/` does not exist |

Failure → halt, write diagnostic.

**Code track gates:**

After every Rust change: `cargo build` + `cargo test` must pass. Retry up to 3× with adjustments; halt on 3rd failure.

### 11.6 Halt-with-diagnostic protocol

On any unrecoverable failure, the orchestrator:

1. Writes detailed diagnostic to status dashboard:
   - Current phase + wave + step
   - What was attempted
   - What failed and why (full error text)
   - Last clean commit hash
   - Files affected since last clean commit
2. Tags a halt commit: `refactor: HALT at phase X wave Y step Z`
3. Stops execution. Does not attempt to "fix" by drifting from plan.

User on return reads the status dashboard, understands what happened, decides whether to retry that step manually or revert to last clean commit.

### 11.7 Status dashboard format

Located at `/tmp/cernio-refactor-status.md`. Updated after every commit and on every halt.

```markdown
# Cernio Full Refactor — Live Status

**Started:** 2026-MM-DD HH:MM
**Last update:** 2026-MM-DD HH:MM
**Branch:** refactor/cernio-full-relativity
**Current state:** Running Phase B Wave 2 | Activation step 5 | Complete | HALTED

## Pre-flight (Phase A)
- [X] Branch created (commit abc1234)
- [X] DB backup at /tmp/cernio-pre-refactor-<ts>.sqlite.bak (sha256: ...)
- [X] Prerequisites verified
- [X] Baseline cargo build + test green

## Skill iteration waves (Phase B)
- [X] Wave 1 — populate-from-lifeos (commit def5678, completed HH:MM)
- [X] Wave 2 — grade-companies + grade-jobs (commit ..., completed HH:MM)
- [ ] Wave 3 — discover/populate/resolve/search (in progress)
- [ ] Wave 4 — check-integrity + test-grade-jobs
- [ ] Wave 5 — prepare-applications deletion

## Code track (Phase C)
- [ ] Schema migration
- [ ] Rust script updates
- [ ] TUI updates

## Activation pipeline (Phase D)
- [ ] Step 1 — Schema migration
- [ ] Step 2 — populate-from-lifeos
- [ ] Step 3 — grade-companies (N companies graded)
- [ ] Step 4 — cernio clean (N companies deleted)
- [ ] Step 5 — discover-companies (N new companies)
- [ ] Step 6 — resolve (N companies resolved)
- [ ] Step 7 — search-jobs (N jobs discovered)
- [ ] Step 8 — grade-jobs (N jobs graded)
- [ ] Step 9 — check-integrity

## Halts / issues
(none)

## Files changed
(populated as we go — list of changed files since pre-flight)

## Final summary
(populated on completion: total wall-time, companies/jobs graded, grade distribution per lane, any deferred items)
```

### 11.8 Parallel-agent isolation rules

Concurrent background agents cannot conflict on shared files:

| Rule | Enforcement |
|---|---|
| One agent owns each shared file | Wave 2: `grade-jobs` owns `context/notes/grading-rubric.md`; `grade-companies` reads only |
| Skills in same wave touch only their own skill directory | Each agent's scope is `.claude/skills/<skill-name>/` plus skill-log directory |
| All commits go through orchestrator | Background agents return changes; orchestrator commits after verification |
| Status dashboard writes go through orchestrator | Single writer; agents return status delta, orchestrator writes |
| DB writes during activation pipeline are sequential | Phase D is single-threaded |
| Within-skill parallelism is the skill's own concern | When grade-jobs spawns 20 parallel grading agents internally, that's inside the skill — the orchestrator just invokes the skill once |

### 11.9 Retry + timeout policy

| Failure mode | Retry policy |
|---|---|
| `cargo build` fails | Retry with adjusted code up to 3× (e.g., diagnose error, attempt fix); halt on 3rd failure |
| `cargo test` fails | Same as build |
| Skill iteration produces invalid output (missing files, forbidden patterns) | Re-invoke skill-creator with diagnostic context up to 2×; halt on 2nd failure |
| Background agent takes > 45 min wall-time | Halt that agent, halt the wave, diagnose |
| Network failure (LifeOS unreachable, gh API rate limit) | Backoff retry up to 3× over ~5 min; halt on persistent failure |
| Skill invocation during activation (grade-jobs run) fails | Single retry; halt on second failure |

### 11.10 What's NOT autonomous

Per user authorisation on 2026-05-28: merge + push are now part of the autonomous flow (see §11.16 Phase F). The only thing left non-autonomous is:

1. **Decision on any deferred items** — if the activation pipeline surfaces issues that aren't unrecoverable but are policy decisions (e.g., a borderline lane-assignment that needs user judgement), they're logged for user review, not auto-resolved. The user reads these in the wrap-up dashboard's "deferred items" pane on return.

Everything else — including merge, push, and DB backup cleanup — runs to completion in Phase E + F + G. The user on return finds:
- `main` branch updated with the full refactor
- Remote `origin/main` pushed
- Working tree clean
- Wrap-up dashboard in chat + at `/tmp/cernio-refactor-final-dashboard.txt`
- No backups, no scratch files, no stray temp artefacts

### 11.11 Estimated total wall-time

| Phase | Duration |
|---|---|
| A — Pre-flight | ~5 min |
| B — Skill iteration waves (5 waves sequential, parallelism within) | ~2.5 hours |
| C — Rust code track (parallel with Waves 3–5, sequential internally) | ~45 min (overlaps with Phase B end) |
| D — Activation pipeline | ~90 min |
| **Total wall-time** | **~4–5 hours** |

The user on return after a typical evening out should find the refactor complete or halted-with-clear-diagnostic. No mid-execution decisions left dangling.

### 11.12 Critical orchestration rules summary

1. **No `prepare-applications` reactivation.** Background agents must not be told to iterate it.
2. **Pass the plan context to every background agent.** Each agent gets §1 (Core principles) + the relevant §5.N + the verification gates from §11.5. Not just §5.N alone.
3. **Skill-creator handles iteration mechanics.** Background agents invoke skill-creator with derived specs; they don't directly edit skill files.
4. **Verify mechanically, not via summary.** Every skill iteration is verified against §11.5 mechanical checks before commit. Background agent summary is a pointer to what to inspect, never proof of correctness.
5. **No `lanes.md` ever materialises.** Mechanical check in §11.5 catches this; halt if found.
6. **No hardcoded company-name calibration in rubric prose.** Mechanical check catches this; halt if found.
7. **Commit at every wave boundary + every code-track milestone + every activation step.** Granular commits enable revert if needed.
8. **Halt loud, halt clean.** On unrecoverable failure, diagnostic to status dashboard + commit a halt marker. Never silently drift from plan to "fix" something.
9. **Status dashboard is the user's window.** Update after every commit and every halt. User on return reads this first.
10. **Merge + push are autonomous.** Per user authorisation 2026-05-28, the refactor branch is merged to main with `--no-ff` and pushed to `origin/main` as Phase F. The user does not need to merge or push manually. The refactor branch is pushed to origin first for audit trail before merge. Conflicts on merge or non-fast-forward errors → halt with diagnostic; never auto-resolve.
11. **DB backup is deleted only after merge + push succeed.** The backup is the safety net through Phase F; Phase G.2 deletes it once main is shipped and verified.

### 11.13 Orchestration affordances — maximising adherence

The autonomous run uses every available structural affordance to anchor itself to the plan. The more granular the tracking, the less drift.

#### TaskCreate at the start

Before Phase A begins, the orchestrator creates a top-level task list:

| Task ID | Task | Status |
|---|---|---|
| 1 | Phase A — Pre-flight (backup, branch, baseline verify) | pending → in_progress → completed |
| 2 | Wave 1 — populate-from-lifeos iteration + run | pending |
| 3 | Wave 2 — grade-companies + grade-jobs iteration | pending |
| 4 | Wave 3 — discover + populate-db + resolve + search iteration | pending |
| 5 | Wave 4 — check-integrity + test-grade-jobs iteration | pending |
| 6 | Wave 5 — prepare-applications deletion | pending |
| 7 | Phase C — Schema migration | pending |
| 8 | Phase C — Rust script updates | pending |
| 9 | Phase C — TUI updates | pending |
| 10 | Activation step 1 — Schema migration applied | pending |
| 11 | Activation step 2 — populate-from-lifeos run | pending |
| 12 | Activation step 3 — grade-companies Phase 1+2 | pending |
| 13 | Activation step 4 — cernio clean no-lane sweep | pending |
| 14 | Activation step 5 — discover-companies | pending |
| 15 | Activation step 6 — resolve + resolve-portals | pending |
| 16 | Activation step 7 — search-jobs | pending |
| 17 | Activation step 8 — grade-jobs Phase 1+2+3 | pending |
| 18 | Activation step 9 — check-integrity | pending |
| 19 | Phase E.1 — Pre-merge verification (cargo, check, sample-grade) | pending |
| 20 | Phase E.2 — Intermediate cleanup (wave logs, specs, pre-state files) | pending |
| 21 | Phase F.1 — Push refactor branch to origin (audit trail) | pending |
| 22 | Phase F.2 — Switch to main + pull origin/main | pending |
| 23 | Phase F.3 — Merge refactor → main with --no-ff | pending |
| 24 | Phase F.4 — Push main to origin | pending |
| 25 | Phase F.5 — Delete local refactor branch | pending |
| 26 | Phase F.6 — Verify integration | pending |
| 27 | Phase G.1 — Render wrap-up dashboard via render.py | pending |
| 28 | Phase G.2 — Final cleanup (DB backup, render script) | pending |
| 29 | Phase G.3 — Final git status verification | pending |

Each task transitions in-progress → completed as the orchestrator works. Status dashboard mirrors this. The TaskCreate state is the source-of-truth for "where are we right now".

#### Per-wave scratch files

Each wave has a scratch file at `/tmp/cernio-refactor-wave-<N>-log.md` capturing:

- Wave start timestamp
- For each skill in the wave: pre-iteration state (current SKILL.md hash, line count), spec passed to skill-creator (which §5.N section), background agent ID, completion timestamp, post-iteration verification results
- Inter-wave contract check results
- Wave commit hash
- Wave end timestamp + wall-time

These are working files for the orchestrator — they make the wave's reasoning visible. Deleted in Phase E cleanup.

#### Per-skill-iteration spec files

Before invoking each skill-creator iteration, the orchestrator writes the spec to `/tmp/cernio-skill-<skill-name>-spec.md`:

- The full content of §1 (Core principles)
- The full content of the relevant §5.N section
- The §11.5 mechanical verification checks specific to this skill
- A "what to add", "what to keep", "what to remove" summary derived from the plan

The background agent reads this spec file (passed as argument to its prompt) before invoking skill-creator. This means the agent never has to re-derive the spec from the long plan file — it has a curated, skill-specific brief.

Spec files are deleted in Phase E cleanup.

#### Per-activation-step pre-state files

Before each activation step that modifies the DB significantly, write `/tmp/cernio-activation-step-<N>-pre-state.md`:

- DB row counts (companies, jobs, application_packages) before
- Expected delta after this step
- Anything that should be true after (e.g., "every company should now have non-empty lanes")

After the step, verify the expected delta matches actual. Mismatch → halt with diagnostic.

Deleted in Phase E cleanup.

#### Background agents per discrete chunk of work

Maximise parallelism within the rules of §11.8 (no shared-file conflicts):

- **Wave 1:** 1 background agent (populate-from-lifeos) — no parallelism possible
- **Wave 2:** 2 background agents in parallel (grade-companies + grade-jobs)
- **Wave 3:** 4 background agents in parallel (discover-companies + populate-db + resolve-portals + search-jobs)
- **Wave 4:** 2 background agents in parallel (check-integrity + test-grade-jobs)
- **Wave 5:** 1 background agent (prepare-applications deletion — sequential mechanical work)
- **Phase C:** 3 background agents in parallel after schema migration (Rust scripts updates split into 3 sub-tracks: `resolve+search+import`, `check+clean`, `stats+pending+unarchive`); plus 1 background agent for TUI updates
- **Phase D activation:** sequential by necessity (each step depends on prior); but within `discover-companies` and `grade-jobs`, the skills themselves spawn internal parallel agents per lane

Peak parallel-agent count: ~4 concurrent in Wave 3 and Phase C overlap.

#### Commit message format

Every commit during the refactor follows this format for grep-ability later:

```
refactor(<scope>): <one-line summary>

Phase: <A|B|C|D|E>
Wave/Step: <wave N | activation step M | code track sub-track>
Files changed: <list>
Verification: <pass | <count> failures>

<optional detail body>

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

### 11.14 Phase E — Pre-merge verification + intermediate cleanup

After Phase D step 9 succeeds, but BEFORE merging to main, run final verifications and intermediate cleanup.

#### 11.14.1 Pre-merge verification

| Check | Action |
|---|---|
| `cargo build --release` passes | Hard gate; halt if fails |
| `cargo test` passes (all suites) | Hard gate; halt if fails |
| `cernio check` (mechanical integrity) | Hard gate; halt if reports errors |
| `cernio format` | Run; commit any formatting changes |
| Sample-grade sanity check | Random-sample 20 graded jobs; verify each has lane(s) + lane-relative grade + Q1-Q5 prose. Visual inspection by orchestrator |
| `git status` is clean on refactor branch | Hard gate; if uncommitted changes exist, commit them |
| Branch is ahead of main by N commits, behind by 0 | Verify ancestry; if main has diverged in a problematic way, halt with diagnostic |

If any check fails: halt with diagnostic in `/tmp/cernio-refactor-status.md`. The refactor branch is preserved; user on return decides remediation.

#### 11.14.2 Intermediate cleanup

Delete scratch files that are no longer needed for diagnosis (KEEP DB backup until after merge succeeds — it's the safety net if merge somehow corrupts state):

| File | Action |
|---|---|
| `/tmp/cernio-refactor-wave-*-log.md` | Delete — info captured in wave commits |
| `/tmp/cernio-skill-*-spec.md` | Delete — info captured in skill iteration commits |
| `/tmp/cernio-activation-step-*-pre-state.md` | Delete — info captured in activation commits + status dashboard |

KEPT (for diagnostic purposes if Phase F or G fails):
- `/tmp/cernio-pre-refactor-<timestamp>.sqlite.bak` — DB backup, retained until Phase G.4
- `/tmp/cernio-pre-refactor-portfolio-gaps.md` — legacy gaps backup, retained until Phase G.4
- `/tmp/cernio-refactor-status.md` — phase-by-phase log, retained for user-visibility

Commit: `refactor(verify): pre-merge verification + intermediate cleanup complete`.

### 11.15 Phase F — Merge to main + push to origin

The autonomous integration. After §11.14 succeeds:

#### 11.15.1 Push refactor branch first (audit trail)

```
git push origin refactor/cernio-full-relativity
```

Pushes the granular wave-by-wave history to the remote. Preserves audit trail even after the branch is later deleted locally. If push fails (network, auth) → retry per §11.9 policy; halt on persistent failure.

#### 11.15.2 Switch to main + pull remote

```
git checkout main
git pull origin main
```

If `git pull` reports merge conflicts or non-fast-forward errors: halt with diagnostic. Do not attempt to auto-resolve. The user on return makes the call (refactor branch is safe on origin already).

#### 11.15.3 Merge refactor → main

```
git merge refactor/cernio-full-relativity --no-ff -m "$(cat <<'EOF'
refactor: complete lane-based relativity refactor

Full Cernio refactor per context/plans/cernio-full-refactor.md.
Shifts grading from global single-scale to lane-relative across 8 lanes.
All 10 skills iterated. Schema + Rust scripts + TUI updated.
Jobs table reset; companies preserved with lane backfill.

See wrap-up dashboard at /tmp/cernio-refactor-final-dashboard.txt for
per-lane statistics.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

`--no-ff` preserves the refactor branch's commit history as a merge bubble. If merge produces conflicts (shouldn't, since main has been untouched during the refactor, but defensively): halt with diagnostic. Do not auto-resolve.

#### 11.15.4 Push main

```
git push origin main
```

If push fails (network) → retry per §11.9. If push is rejected (remote moved despite the pre-merge pull — race condition): halt with diagnostic; user on return resolves.

#### 11.15.5 Delete refactor branch locally

Once main is successfully pushed:

```
git branch -d refactor/cernio-full-relativity
```

`-d` not `-D` — refuses to delete if branch isn't merged, which it now is. Defensive.

Remote refactor branch stays at `origin/refactor/cernio-full-relativity` for historical audit. User can delete it later from GitHub UI if they want; not part of the autonomous flow.

#### 11.15.6 Verify integration

| Check | Expected |
|---|---|
| `git branch --show-current` | `main` |
| `git log -1 --format='%s'` | "refactor: complete lane-based relativity refactor" |
| `git log -1 --format='%P' \| wc -w` | 2 (two parents, merge commit) |
| `git status -sb` | `## main...origin/main` clean |
| `git rev-parse HEAD` | matches `origin/main` |

Status dashboard update: Phase F complete, main shipped.

### 11.16 Phase G — Wrap-up dashboard + final cleanup

#### 11.16.1 Render wrap-up dashboard via `~/.claude/scripts/render.py`

Now reflects the truly final state: shipped to main, pushed to origin, refactor branch merged.

Width 200, double-bordered header + status bar, single-bordered content panes. Per-run script at `/tmp/cernio_wrap_up_render_<YYYY-MM-DD>_<HHMMSS>.py`. Output goes to stdout (emitted verbatim in chat) AND written to `/tmp/cernio-refactor-final-dashboard.txt`.

#### Per-run script pattern

```python
import sys
sys.path.insert(0, '/Users/atacanercetinkaya/.claude/scripts')
from render import pane, paired, split_line, assemble, verify_all

W = 200
PW = (W - 2) // 2

# ── Section data (populated from refactor outcomes) ──
# ... pane data ...

# ── Assemble + verify ──
out = assemble([...])
drift = verify_all(out, expected_width=W)
assert len(drift) == 0, f'render drift: {drift}'
print('\n'.join(out))
```

#### Dashboard sections (top-to-bottom render order)

| # | Section | Border | Content |
|---|---|---|---|
| 1 | **HEADER** | double | "CERNIO REFACTOR SHIPPED" / main updated / total wall-time / final state token (SHIPPED or HALTED-AT-X) |
| 2 | **Phase timeline arc** | none | Horizontal arc A → B → C → D → E → F → G with timestamps |
| 3 | **THE GIST** | single | 3-5 sentences: what was accomplished, what shifted, that main is updated + pushed |
| 4 | **WAVE SUMMARY** \| **CODE TRACK SUMMARY** | paired single | Per wave: skills iterated, commits, wall-time \| Schema migration, scripts, TUI changes |
| 5 | **LANE TAGGING SUMMARY** | single full-width | Per lane (8 rows): # companies tagged, pinnacle/strong/adjacent counts, # jobs graded |
| 6 | **GRADE DISTRIBUTION PER LANE** | single full-width | 8 lanes × 6 grade buckets matrix with totals |
| 7 | **TOP SS/S JOBS BY LANE** \| **PORTFOLIO GAPS REGENERATED** | paired single | Top SS/S jobs per lane (3 per lane) \| Per-lane gaps file mtime + open-gap count |
| 8 | **FILES CHANGED** \| **TEMP FILES CLEANED** | paired single | Files committed in refactor (categorised) \| Temp files deleted |
| 9 | **SKILL ITERATION STATUS** | single full-width | Per skill: before → after, key changes, verification result |
| 10 | **GIT INTEGRATION SUMMARY** | single full-width | NEW: refactor branch pushed to origin, merged to main with --no-ff, main pushed, local refactor branch deleted, audit-trail commit hashes |
| 11 | **DEFERRED ITEMS (if any)** | single full-width | Policy decisions the orchestrator flagged for user review. If none: "Nothing deferred — refactor complete." |
| 12 | **INTEGRATION COMPLETE** | single full-width | Brief celebratory closing — no action required; refactor is fully shipped. List the commit on main + the refactor branch ref on origin |
| 13 | **STATUS BAR** | double | `[branch: main]` `[origin: pushed]` `[refactor merged: ✓]` `[backup: deleted]` `[tree: clean]` `[commits: N]` `[wall-time: Xh Ym]` `[STATE: SHIPPED]` |

Tone: peer-to-peer, matches orient/wrap-up convention. No formal-briefing phrases. The user reads this in 30 seconds and knows the refactor is done.

#### 11.16.2 Final cleanup — return tree to truly clean

After dashboard renders successfully (stdout captured, `/tmp/cernio-refactor-final-dashboard.txt` written, no AssertionError on `verify_all`):

| File | Action |
|---|---|
| `/tmp/cernio-pre-refactor-<timestamp>.sqlite.bak` | DELETE — main has the changes, backup no longer needed |
| `/tmp/cernio-pre-refactor-portfolio-gaps.md` | DELETE — superseded by `profile/portfolio-gaps/` folder on main |
| `/tmp/cernio_wrap_up_render_<date>_<time>.py` | DELETE — output captured in chat + dashboard file |

KEPT in `/tmp/`:
- `/tmp/cernio-refactor-status.md` — phase-by-phase log, user may want to re-read
- `/tmp/cernio-refactor-final-dashboard.txt` — wrap-up dashboard for re-viewing

These live in `/tmp/` outside the git tree, will be cleaned up by OS naturally.

#### 11.16.3 Final git verification

| Check | Expected |
|---|---|
| `git status` | Clean, on `main` |
| `git status -sb` | `## main...origin/main` clean |
| `git rev-parse HEAD == git rev-parse origin/main` | true |
| `git branch --list` | no `refactor/cernio-full-relativity` (deleted) |
| `git branch -r --list` | `origin/refactor/cernio-full-relativity` still present (audit trail) |

If any check fails: surface in dashboard, log to status dashboard, but don't halt (the refactor is already shipped; verification failure here is informational).

### 11.17 Phase E + F + G flow summary

| Step | Phase | Action |
|---|---|---|
| 1 | E.1 | Pre-merge verification (cargo, check, sample-grade) |
| 2 | E.2 | Intermediate cleanup (delete wave logs, specs, pre-state) |
| 3 | E.3 | Commit verify checkpoint |
| 4 | F.1 | Push refactor branch to origin (audit trail) |
| 5 | F.2 | Switch to main + pull origin |
| 6 | F.3 | Merge refactor with --no-ff |
| 7 | F.4 | Push main |
| 8 | F.5 | Delete local refactor branch |
| 9 | F.6 | Verify integration (branch / commit / status) |
| 10 | G.1.a | Compose dashboard data from full refactor outcomes |
| 11 | G.1.b | Write per-run render script |
| 12 | G.1.c | Run script, capture stdout, write `/tmp/cernio-refactor-final-dashboard.txt` |
| 13 | G.1.d | Emit dashboard verbatim in final chat message |
| 14 | G.2 | Delete final temp files (DB backup, portfolio-gaps backup, render script) |
| 15 | G.3 | Final git status verification |
| 16 | G.4 | Mark task #20 completed; final status dashboard update: STATE = SHIPPED |

### 11.18 Updated wall-time estimate

| Phase | Duration |
|---|---|
| A — Pre-flight | ~5 min |
| B — Skill iteration waves | ~2.5 hours |
| C — Rust code track (parallel with Waves 3–5) | ~45 min |
| D — Activation pipeline | ~90 min |
| E — Pre-merge verification + intermediate cleanup | ~10 min |
| F — Merge + push to origin/main | ~5 min |
| G — Wrap-up dashboard + final cleanup | ~10 min |
| **Total wall-time** | **~5–6 hours** |

The dashboard is the user's read-on-return artefact. Renders to stdout in the final chat message AND writes to `/tmp/cernio-refactor-final-dashboard.txt` for re-viewing.

#### Per-run script pattern

Write per-run script at `/tmp/cernio_wrap_up_render_<YYYY-MM-DD>_<HHMMSS>.py`. Pattern matches orient's `/tmp/orient_render_...py` convention:

```python
import sys
sys.path.insert(0, '/Users/atacanercetinkaya/.claude/scripts')
from render import pane, paired, split_line, assemble, verify_all

W = 200
PW = (W - 2) // 2

# ── Section data (populated from refactor outcomes) ──
# ... pane data ...

# ── Assemble + verify ──
out = assemble([...])
drift = verify_all(out, expected_width=W)
assert len(drift) == 0, f'render drift: {drift}'
print('\n'.join(out))
```

Run via `python3 /tmp/cernio_wrap_up_render_...py`; capture stdout. Per-run script deleted in Phase E.2 cleanup.

#### Dashboard sections (top-to-bottom render order)

| # | Section | Border | Content |
|---|---|---|---|
| 1 | **HEADER** | double | "CERNIO REFACTOR COMPLETE" / branch name / duration / final state token (COMPLETE or HALTED-AT-X) |
| 2 | **Phase timeline arc** | none | Horizontal arc showing Phase A → B → C → D → E with timestamps |
| 3 | **THE GIST** | single | 3-5 sentences summarising what the refactor accomplished, what shifted in the system, any halts encountered |
| 4 | **WAVE SUMMARY** \| **CODE TRACK SUMMARY** | paired single | Wave-by-wave: skills iterated, commit hashes, wall-time per wave \| Code track: schema migration applied, scripts updated, TUI updates |
| 5 | **LANE TAGGING SUMMARY** | single full-width | Per lane (8 rows): # companies tagged, # pinnacle / # strong / # adjacent, # jobs graded in lane |
| 6 | **GRADE DISTRIBUTION PER LANE** | single full-width | 8 lanes × 6 grade buckets (SS/S/A/B/C/F) matrix with totals |
| 7 | **TOP SS/S JOBS BY LANE** \| **PORTFOLIO GAPS REGENERATED** | paired single | Top SS/S jobs per lane (3 per lane) \| Per-lane gaps file mtime + open-gap count |
| 8 | **FILES CHANGED** \| **TEMP FILES CLEANED** | paired single | Files committed in refactor \| Temp files deleted in Phase E cleanup |
| 9 | **SKILL ITERATION STATUS** | single full-width | Per skill: before-version → after-version, key changes, verification result |
| 10 | **ANY HALTS / DEFERRED ITEMS** | single full-width | If complete: "none". If halted: full diagnostic. If deferred: list of policy items the orchestrator flagged for user |
| 11 | **WHAT'S NEXT FOR USER** | single full-width | Concrete next actions: review the diff, sanity-check N graded jobs, merge `refactor/cernio-full-relativity` → main, push when ready |
| 12 | **STATUS BAR** | double | `[branch: refactor/cernio-full-relativity]` `[commits: N]` `[wall-time: Xh Ym]` `[companies: N]` `[jobs: M]` `[backup: deleted]` `[tree: clean]` `[STATE: COMPLETE]` |

#### Tone

Peer-to-peer, matches orient/wrap-up convention. No formal-briefing phrases. Direct observations. The user reads this in 30 seconds and knows everything.

#### Failure mode

If `verify_all()` returns drift > 0, the per-run script halts with AssertionError. Orchestrator falls back to rendering a narrow (80-col) hand-bordered dashboard as backup. Original drift logged to status dashboard.

If `~/.claude/scripts/render.py` itself can't be imported (filesystem error): write a plain-text status summary instead, document the failure.

### 11.15 Cleanup phase (Phase E.2) — return tree to clean

After the dashboard renders successfully (verified by stdout capture), clean up everything that was created during the refactor as scratch / backup / working state.

#### What gets deleted

| File / pattern | Why |
|---|---|
| `/tmp/cernio-pre-refactor-<timestamp>.sqlite.bak` | DB backup — no longer needed once refactor verified successful |
| `/tmp/cernio-pre-refactor-portfolio-gaps.md` | Legacy gaps file backup — superseded by per-lane folder |
| `/tmp/cernio-refactor-wave-*-log.md` | Per-wave scratch logs — info captured in commits |
| `/tmp/cernio-skill-*-spec.md` | Per-skill iteration specs — info captured in committed skill changes |
| `/tmp/cernio-activation-step-*-pre-state.md` | Pre-state snapshots — info captured in commits + dashboard |
| `/tmp/cernio_wrap_up_render_*.py` | Per-run dashboard render script — output captured in chat + dashboard file |

#### What gets kept

| File | Why |
|---|---|
| `/tmp/cernio-refactor-status.md` | Phase-by-phase live status log — user may want to re-read history |
| `/tmp/cernio-refactor-final-dashboard.txt` | Wrap-up dashboard for re-viewing |

These two persist in `/tmp` and will be cleaned up by the OS naturally; they're not in the git tree.

#### Working tree verification

After cleanup:
1. Run `git status` — expected output: clean working tree on `refactor/cernio-full-relativity` branch
2. If anything is untracked (e.g., backups accidentally landed in repo) or modified — diagnose; either commit (if it should have been) or delete (if it's leftover scratch)
3. Run `git status -sb` and confirm no `??`, no `M`, no `D` lines
4. Update the status dashboard with final state: tree-clean confirmed

#### Cleanup commit

If any cleanup action affected the git tree (shouldn't happen if isolation rules were followed, but as a safety net), commit it: `refactor(cleanup): return tree to clean state`.

### 11.16 Phase E flow summary

| Step | Action |
|---|---|
| E.1.a | Compose dashboard data from Phase D outcomes (lane stats, grade distribution, files changed, etc.) |
| E.1.b | Write per-run `/tmp/cernio_wrap_up_render_<date>_<time>.py` |
| E.1.c | Run script, capture stdout, write to `/tmp/cernio-refactor-final-dashboard.txt` |
| E.1.d | Emit dashboard verbatim in final chat message |
| E.2.a | Delete all temp files per §11.15 deletion list |
| E.2.b | Run `git status` verification |
| E.2.c | Mark task #20 completed in TaskCreate state |
| E.2.d | Final status dashboard update: STATE = COMPLETE |

### 11.17 Updated wall-time estimate

| Phase | Duration |
|---|---|
| A — Pre-flight | ~5 min |
| B — Skill iteration waves | ~2.5 hours |
| C — Rust code track (parallel with Waves 3–5) | ~45 min |
| D — Activation pipeline | ~90 min |
| E — Wrap-up dashboard + cleanup | ~10 min |
| **Total wall-time** | **~4.5–5.5 hours** |

---

## Open questions — resolved with defaults

- **Group split for `profile/skills/`** — `populate-from-lifeos` decides based on actual LifeOS profile content; no fixed group list. This is the load-bearing decision; everything else is default.
- **Cambridge MA / Birmingham AL false-positive disambiguation** — default rule: ATS location patterns prefer UK-anchored forms (`Cambridge, UK`, `Cambridge, Cambridgeshire`, `Birmingham, UK`, `Birmingham, West Midlands`) over bare city names. Pattern lists encoded per-provider in `preferences.toml`.
- **Hybrid-unspecified at Tier 2 heuristic** — default: optimistic-accept (treat as ≤2 days/wk); the `grade-jobs` agent reads the JD for office-day signals and downgrades if 3+ days mentioned.
- **TUI lane-filter UX** — default: `L` key cycles `all → big-tech → ai-ml → hft → crypto-mm → bank-strats → systems-infra → devtools → fintech → all`.
- **Application view in TUI** — default: remove the view entirely along with `prepare-applications` skill deletion.

---

**Status:** design locked. Implementation begins via the wave-orchestration model described in §11 when triggered.
