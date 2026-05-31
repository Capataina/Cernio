---
name: populate-db
description: "Validates discovered companies, runs the `cernio resolve` Rust script to probe ATS slug candidates across the seven supported providers, and handles AI fallback for companies the script cannot match — either finding the correct slug on the careers page or marking the company as bespoke. Invoke when the user says 'populate the database', 'populate db', 'research these companies', 'add these companies', 'resolve the portals', 'process potential.md', or after a discovery run leaves entries in `companies/potential.md` that still need processing. Not for discovering new companies (use discover-companies), grading (use grade-companies), searching jobs on resolved companies (use search-jobs), or pure AI-only portal resolution (use resolve-portals — this skill orchestrates the full discovery-to-DB pipeline, script plus fallback). Use this skill whenever unprocessed companies need to reach the database as resolved or bespoke, even if the user does not name it explicitly."
---

# Populate DB

> [!warning] LANE-BASED RELATIVITY REFACTOR IN-PROGRESS (2026-05-28)
> Per `context/plans/cernio-full-refactor.md §5.2`. Add a sponsor-verification precondition before any ATS resolution:
>
> 1. **Phase 1 — Sponsor verification precondition**: confirm `sponsors_uk = 'yes'` for each company before running `cernio resolve`. Companies without verified sponsorship are rejected — they don't reach the mechanical probe.
> 2. **Phase 4 — Hand off to `grade-companies`** for initial `lanes` assignment + `pinnacle_status_per_lane` + employer grade after ATS resolution succeeds.

## Phased Structure (canonical, post-refactor)

### Phase 1 — Sponsor verification precondition

- For each company in companies/potential.md (or discovery-*.md):
  - Cross-check Gov.uk Skilled Worker sponsor register
  - Verify careers page mentions visa support OR check Hunt-UK-Visa-Sponsors-style database
  - Mark `sponsors_uk = 'yes' | 'unknown'`; non-sponsors rejected entirely
- Companies with `sponsors_uk = 'no'` never proceed to Phase 2

### Phase 2 — Mechanical ATS resolution (cernio resolve)

- Invoke `cernio resolve` Rust script against companies with sponsors_uk in (yes, unknown)
- Script probes 7 ATS providers for slug matches
- Output: companies with status='resolved' (ATS slug found) or status='potential' (no match)

### Phase 3 — AI fallback for unmatched (resolve-portals)

- For companies still status='potential' after Phase 2:
  - Dispatch resolve-portals subagent
  - Subagent reads careers page + searches for ATS link
  - Either finds the real slug or marks status='bespoke' (unsupported ATS)

### Phase 4 — Handoff to grade-companies

- Cascade resolved + bespoke companies to grade-companies Phase 1
- grade-companies assigns lanes, pinnacle_status_per_lane, employer grade

Bridge between company discovery and job search. Discovery produces a list of company names and websites; the search pipeline needs those companies resolved to a specific ATS provider and slug, or marked as bespoke. This skill drives that transition — it validates each company, runs `cernio resolve` for the mechanical slug probing, and applies AI judgment to the cases the script cannot handle.

The skill is the orchestration layer. The Rust script handles 10,000+ HTTP requests against supported ATS providers in seconds; this skill handles the ~20% of cases where the mechanical probing fails — non-obvious slugs (XTX Markets → `xtxmarketstechnologies`), unsupported ATS providers (iCIMS, Taleo, Personio), and companies that turned out to be dead.

---

## Mandatory Reads Before Populating

| # | What | Evidence |
|---|---|---|
| 1 | **Every file in `profile/`** | You can name the profile-relevant reason this company was in the discovery set (e.g. "Cloudflare — targets systems / networking, matches profile's Rust + Nyquestro infrastructure focus") |
| 2 | **`references/ats-providers.md`** in this skill | You can name which of the 7 supported providers a given careers page uses and cite the specific detection signal (outbound-link domain, script-tag source, redirect chain) |
| 3 | **`companies/potential.md`** (the discovery output) | The input to this skill is this file; you know which entries are unprocessed |

If the profile is not read, the relevance judgment in step 2 below cannot be grounded. If `ats-providers.md` is not read, the AI fallback phase misclassifies providers. Both are preconditions, not optional deepening reads.

---

## Scripts

`cernio resolve` is the Rust binary at the heart of this skill. It probes generated slug candidates against the JSON APIs of seven supported ATS providers in parallel and persists results directly to the SQLite database. It is high-throughput mechanical work the agent must not duplicate in prose. It does not replace reasoning — the AI fallback in step 5 handles everything the mechanical probe cannot match.

The command's concrete form is *novel to pretraining* — it is a project-specific binary on the user's PATH, not `pytest`, `npm`, or `cargo` the model has seen at training-scale frequency. Without explicit invocation obligations and evidence citations, the pretraining-frequency bias (F12 Tool-Action Asymmetry, F15 Script-Obligation Asymmetry) will push the agent toward substituting `WebSearch` and prose-reasoning in its place.

**ATS coverage:** Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday, Eightfold. Any company whose careers page points elsewhere (iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, Recruitee, Breezy HR, Wellfound, and any unknown provider) is out of the script's coverage and must be handled through step 5's bespoke bucket. Fallback for out-of-coverage: inspect the careers page (WebSearch + page source) and mark `bespoke` with the careers URL.

### Script inventory

| Command | Purpose | When to invoke |
|---------|---------|----------------|
| `cernio resolve --dry-run` | Emits the company list that will be probed and the slug candidates generated for each, without making HTTP calls. | Step 3, before `cernio resolve`. Always first. |
| `cernio resolve` | Executes ATS resolution against every company in the DB whose `ats_slug` is unset. Writes `resolved` or `failed` to each row. | Step 3, after dry-run is reviewed. |
| `cernio resolve --company "Name"` | Re-probes a single company by name. | Ad-hoc retries after an AI-fallback slug discovery, or when re-resolving a previously failed company. |

### Script invocation obligations

- **Dry-run first.** `cernio resolve --dry-run` is always run before `cernio resolve`. The summary output — which companies are queued, how many slug candidates each has — is pasted into the agent's chat response before the real run. No exception for "small batches" or "obvious cases."
- **Real run second.** `cernio resolve` is invoked only after the dry-run output has been reviewed (by the agent, for surprises; by the user, if any surprise surfaces). Per-company result rows from stdout are preserved for step 4 categorisation.
- **Ad-hoc re-probe after fallback slug discovery.** When step 5 bucket 1 finds a non-obvious slug (e.g. XTX Markets → `xtxmarketstechnologies`), the correct action is usually to update the company's probe hint and re-run `cernio resolve --company "Name"` so the persistence path matches the mechanical one. Do not manually INSERT slugs the script could have confirmed — the script writes richer metadata (ats_extra, verified_at).
- **No substitution.** Do not replace `cernio resolve` with manual WebSearch + curl probing except for the out-of-coverage cases named in the ATS coverage clause above. If the pre-run company list is empty (all companies already have `ats_slug`), skip the invocation entirely and note "no unresolved companies; cernio resolve skipped" in the summary. Emptiness is a valid outcome; it is not the agent's cue to invent work.

### Evidence destination

- The pasted `cernio resolve --dry-run` summary block in the chat response.
- The pasted `cernio resolve` per-company result block (resolved count, failed count, per-company provider + slug for resolved).
- In step 6's summary table, the `Source` column for every Resolved row reads "cernio resolve (mechanical)" with the provider and slug coming directly from the script's output, not the agent's reasoning.

---

## Workflow

### 1. Read the input and deduplicate

Input sources, in order:

- `companies/potential.md` — structured discovery output. Each entry has `name`, `website`, and `why_relevant`.
- Companies the user names directly in conversation.
- Per-agent discovery files in `companies/` if discovery produced them instead of `potential.md`.

Before processing any company, query the `companies` table for existing entries with the same `website`. Duplicates come from running the skill against overlapping discovery outputs and from companies that were previously processed and archived. Skip any company already in the DB.

### 2. Validate each company is real and worth tracking

The script does not validate — it only probes. Validation is the judgment step that filters out dead or irrelevant entries before wasting probe budget on them.

**Activity and viability checks:**

- Does the website load? Dead website → remove from `potential.md` with a reason, do not insert.
- Is there evidence of activity in the last 6–12 months — recent blog posts, press, funding, social, product updates?
- Has the company been acquired, rebranded, or dissolved? Check redirects to a parent; check Companies House if UK.

**Engineering-fit checks (grounded in profile):**

- Does the company have an engineering team large enough to hire at entry-to-mid level?
- Does their actual technical work align with the profile's direction — systems, trading, AI infrastructure, Rust-in-production, modern developer tooling? A "fintech" that does marketing automation does not count, even if it is in the sector.
- Any signal of current hiring or growth — active careers page, listed roles, recent headcount expansion?

Companies that fail validation: remove from `potential.md` with a short note (dead website, acquired by X, too small, not relevant to profile). Companies that pass: continue to step 3. Grading happens later in `grade-companies`; this skill does not grade.

### 3. Run `cernio resolve` for mechanical probing

The script probes every generated slug candidate against every supported ATS provider. No judgment, high throughput.

| Command | Purpose |
|---------|---------|
| `cernio resolve --dry-run` | Preview which companies will be probed and which slug candidates are generated. Run first. |
| `cernio resolve` | Execute resolution against all unresolved companies in the DB. |
| `cernio resolve --company "Palantir"` | Resolve a single company by name. Useful for ad-hoc additions or re-tries. |

Workflow: dry-run first, confirm the company list looks right, then execute. Review the script's per-company output — each row reports resolved, failed, or error.

### 4. Categorise the script's output

| Outcome | Meaning | Next step |
|---|---|---|
| **Resolved** | Slug found and verified on a supported ATS | Ready for `cernio search` — no further work in this skill |
| **Failed** | No slug candidate matched any supported ATS | Step 5 (AI fallback) |
| **Error** | HTTP timeout, rate limit, network error | Re-try once; if still failing, treat as Failed and go to step 5 |

### 5. AI fallback for unresolved companies

Companies that the script could not resolve fall into three buckets. Route each to the correct outcome:

**Bucket 1 — Non-obvious slug on a supported ATS.** The company uses Greenhouse, Lever, Ashby, etc., but the script's slug candidates missed the real one. Common causes: legal entity name (XTX Markets → `xtxmarketstechnologies`), former name (Wise → `transferwise`), abbreviation, or domain-based slug. The careers page is the authoritative source — find the outbound "Apply" link, extract the slug from the ATS URL, verify via the provider's JSON API per `references/ats-providers.md`.

**Bucket 2 — Unsupported ATS provider.** iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, Recruitee, Breezy HR, Wellfound. Mark as `bespoke` with the careers URL preserved. The search pipeline will handle these through the bespoke-search workflow, not the ATS script.

**Bucket 3 — Dead or dissolved company.** No loading website, no careers page, Companies House shows dissolved. Report with evidence and remove from `potential.md`; do not insert.

**Fallback tooling:**

- `WebSearch` for `"{company name}" careers` or `"{company name}" jobs`
- Visit the resulting careers page; inspect outbound links, page source, iframe sources, and redirect chains for ATS identifiers (full detection playbook in `references/ats-providers.md`)
- For a supported-ATS match: verify the candidate slug against the provider's JSON API before writing
- For SmartRecruiters specifically: `totalFound > 0` is the only valid confirmation — a 200 response with `totalFound: 0` is ambiguous and possibly a fabricated slug

**Per-company evidence obligation.** `WebSearch` is a low-pretraining-support tool and will be silently substituted by reasoning-about-the-slug unless evidence is required. For each company reaching this step, the summary table row in step 6 cites:

- The exact `WebSearch` query used (or an explanation of why no search was needed, e.g. direct careers URL already known from `potential.md`).
- The careers-page URL visited (one or more).
- The specific ATS signal observed — the outbound link domain, script-tag source, or redirect chain target. Quote the signal, do not paraphrase it.
- For bucket-1 supported-ATS: the API endpoint hit plus the `total` / `totalFound` / array-length from the JSON response.
- For bucket-3 dead companies: the specific evidence (Companies House URL with status text, HTTP response code from the website, or redirect-destination URL).

A bucket-3 declaration without a citable source is not evidence — it is a silent-skip draped in prose. If the evidence cannot be gathered, the company stays in `potential.md` marked as "needs human review" and is surfaced in the "What I did not do" section at the end.

### 5.5. UK sponsor register entity lookup (every company)

Every company being inserted or resolved in this run must have its `uk_sponsor_entity_name` column populated. This runs alongside the ATS resolution and is independent of resolved/bespoke status. The register is the UK Home Office's Register of Licensed Sponsors: Workers (Skilled Worker route), downloaded fresh each populate run from `https://www.gov.uk/government/publications/register-of-licensed-sponsors-workers` as a CSV.

Per-company workflow:

1. **Normalise the cernio company name** (lowercase, strip Ltd/Limited/PLC/LLC/LLP/Inc/Corp/UK/Europe/Holdings/Group, replace spaces with single space).
2. **Exact-match against the register's normalised entity names.** If a single hit on Skilled Worker route, write the raw register name to `uk_sponsor_entity_name`.
3. **Token-prefix substring match** — if the cernio name's tokens are the first N tokens of a register entity's tokens (e.g. cernio `apple` matches register `apple europe limited`), and there is exactly one such match, accept it.
4. **AI fallback** — for unmatched companies, search the register for likely entity variants (parent company, legal entity, trading-as name). For known acquisitions/rebrands (Meta → Facebook UK, HashiCorp → IBM UK Ltd, Cantab Capital → GAM UK), use the post-acquisition entity. If no register entry exists under any plausible name, write `unknown`.
5. **Cite the source.** The summary table row's `Sponsor Entity` column records either the exact register name or `unknown`. For AI-fallback matches, the agent's per-company evidence section quotes the register line that confirmed the match (grep result or CSV row).

Companies that are confirmed on the register but only via non-Skilled-Worker routes (Global Business Mobility: Senior or Specialist Worker, Graduate Trainee, Charity Worker, etc.) DO NOT qualify — those routes don't permit direct UK hire from outside the company. Such companies should be flagged for archival in step 6, not inserted.

### 6. Assemble end-of-skill summary

After mechanical verification (Subagent Context Requirements §Scratch-path contract) passes and SQL has been applied, the orchestrator concatenates agent-written summary rows under this header for the end-of-skill report. No mid-run approval gate; the skill is autonomous.

```
| Company        | Result   | ATS        | Slug                     | Sponsor Entity                       | Source                                |
|----------------|----------|------------|--------------------------|--------------------------------------|---------------------------------------|
| Cloudflare     | resolved | greenhouse | cloudflare               | Cloudflare Limited                   | cernio resolve (mechanical)           |
| XTX Markets    | resolved | greenhouse | xtxmarketstechnologies   | XTX Research LLP                     | AI fallback — careers page outbound   |
| SurrealDB      | bespoke  | pinpoint   | —                        | unknown                              | surrealdb.pinpointhq.com (unsupported)|
| Vypercore      | removed  | —          | —                        | n/a                                  | Companies House: in liquidation       |
```

The orchestrator does not wait for approval — verification of scratch artefacts (file presence, frontmatter row counts, SQL parses) is the safety net. Anomalies surface in the post-apply summary for the user to act on if needed.

### 7. Commit and clean up

After approval, insert resolved and bespoke companies into the DB, then remove them from `companies/potential.md`. `potential.md` is a landing zone — its steady state after a population run is "only contains unprocessed entries."

### 8. Declare what was skipped

Close the run with a "What I did not do" section covering: companies left in `potential.md` with a "needs human review" note, bucket-3 dead-company claims that lacked a citable source, SmartRecruiters cases where `totalFound = 0` made the slug ambiguous, script errors that did not recover on retry, and any validation-step company rejected on engineering-fit grounds that might be worth revisiting after a profile change. If nothing was skipped, say so explicitly — silence on this section is a signal the declaration was omitted, not a signal that everything was processed cleanly. Silent skipping is the documented Claude abstention pattern (see `research/18-claude-specific-calibration.md`); the declaration is the structural counter.

---

## Subagent Context Requirements

When delegating fallback work to parallel subagents (one per 3–5 unresolved companies), each subagent prompt includes every item below. Subagents run in isolated contexts and cannot read the skill directory or `profile/` themselves; anything not embedded in the prompt is invisible to them.

- The **full content of `references/ats-providers.md`** — verbatim, not summarised.
- The **full content of every file in `profile/`** — needed for the relevance judgment in step 2.
- The list of assigned companies with their discovery-source metadata (name, website, `why_relevant` from `potential.md`).
- The **scratch-path file assignments** — `/tmp/populate-db-<run-id>/<batch>.summary.md` (and `<batch>.sql` for any SQL the agent generates, e.g. UPDATE for AI-fallback resolved slugs) — that the agent must WRITE TO (not emit inline in chat).
- The per-company evidence obligation from step 5 reproduced verbatim, so the summary-file rows carry query + page URL + ATS signal quote + API response fields (not a narrative summary).
- The summary-table column format from step 6, again verbatim, so per-batch summary files are concatenable.

**Scratch-path contract.** Each fallback subagent writes:

```
/tmp/populate-db-<run-id>/
├── <batch-id>.summary.md  ← markdown table rows (Company / Result / ATS / Slug / Sponsor Entity / Source) per Rule §6 format
└── <batch-id>.sql         ← any SQL the agent produces (e.g. UPDATE companies SET uk_sponsor_entity_name = '...' for AI-fallback matches)
```

For bucket-1 slugs the canonical persistence path remains `cernio resolve --company "Name"` after the agent surfaces the slug — the agent's `<batch>.sql` is for sponsor-entity backfill and bespoke-status updates, NOT for INSERTing portal rows (the script writes richer metadata). The orchestrator concatenates the summary files for the §6 user-review block and applies the SQL via `cat .../*.sql | sqlite3 state/cernio.db` after approval.

The failure mode this section defends against is subagent prompts that embed a summary of the reference material rather than the full text. Under-contextualised subagents produce generic output that mislabels providers — verified cause of prior F12-class silent misclassifications on this skill.

---

## Reference Loading

**Mandatory-core — read at skill invocation:**

- `references/ats-providers.md` — the complete ATS provider reference: supported providers with API details + verification pattern, unsupported providers with recognition markers, careers-page identification playbook, common slug patterns that defeat mechanical resolution.

---

## Inviolable Rules

1. **The skill is autonomous start-to-finish.** Mechanical verification of scratch artefacts (file presence, frontmatter row counts, SQL parses, dedup query) is the safety net before `sqlite3` applies the batch. No mid-run user-approval gate. Anomalies surface in the post-apply summary, not as pause-points.
2. **Each resolution must be verified against the provider's JSON API** (except bespoke). For SmartRecruiters, verification means `totalFound > 0` — 200 alone is not evidence.
3. **Bespoke companies carry a working careers URL**, not a homepage. The bespoke-search workflow downstream needs a page that actually shows jobs.
4. **Do not grade during this skill.** Grading is `grade-companies`. Mixing the two contaminates both passes.
5. **Profile is read fresh** — no caching, no embedded snapshots, no reliance on earlier-session memory of the profile.
6. **Every AI-fallback decision is cited.** Bucket 1 cites the careers URL and the API response fields proving the slug; bucket 2 cites the careers URL and the unsupported-ATS signal; bucket 3 cites a specific status source (Companies House URL, HTTP response, redirect target). A fallback row without a citable source is not a decision, it is a guess — surface it under "What I did not do" instead.
7. **The cernio resolve step cannot be substituted.** For companies whose careers pages are on supported ATS providers, `cernio resolve` is the only path into the database. Manual INSERT statements for an in-coverage company are a failure, not a shortcut.
8. **Every company processed must have `uk_sponsor_entity_name` populated.** The column is set to the exact register entity name (Skilled Worker route only — non-SW routes do not qualify) or to the literal string `unknown` if no match is found under any plausible variant. Inserting a row with `uk_sponsor_entity_name` left as NULL is a failure of step 5.5.
9. **Fallback subagents write summary + SQL to assigned scratch path; orchestrator never transcribes per-company rows from chat.** Per the Subagent Context Requirements section, each agent writes `<batch>.summary.md` and (where SQL is produced) `<batch>.sql` to `/tmp/populate-db-<run-id>/`. The §6 review table is assembled by concatenating summary files. Bucket-1 slug surfaces still re-run `cernio resolve --company "Name"` as the canonical persistence path — agent-written SQL handles sponsor-entity backfill and bespoke-status updates, not portal INSERTs.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. Items that cannot be evidenced in the agent's own output are either skipped and declared under "What I did not do," or the skill has not finished.

- [ ] **Profile read fresh this invocation** — cite the tool call that read each file under `profile/` (not "I remember the profile from a prior session").
- [ ] **Reference file read fresh** — cite the tool call that read `references/ats-providers.md` in full this invocation.
- [ ] **Input deduplicated against existing DB** — cite the SQL query run against `companies` and the set of websites that were already present (even if zero).
- [ ] **Validation judgement recorded per company** — for every company processed, the summary table (step 6) names at least one profile-relevant reason the company belongs in the universe, tied to a specific profile element by name (e.g. *"Rust in production, matches `profile/skills.md` Rust focus"*). Generic rationales ("looks interesting") fail this item.
- [ ] **`cernio resolve --dry-run` output pasted verbatim** — the company list and slug-candidate summary appear in chat before the real run. Absent output means the dry-run step was skipped.
- [ ] **`cernio resolve` output pasted verbatim** — per-company result lines (resolved / failed / error) appear in chat; the step 4 categorisation reads directly from these lines, not from the agent's reasoning about what probably happened.
- [ ] **Each supported-ATS resolution has an API-response evidence line** — provider endpoint hit, response `total` / `totalFound` / array length, quoted slug-match evidence. SmartRecruiters rows cite `totalFound > 0` specifically.
- [ ] **Each bucket-1 AI-fallback row cites query + page URL + ATS signal quote + API response fields** — per the step 5 per-company evidence obligation.
- [ ] **Each bucket-2 bespoke row cites careers URL + unsupported-ATS signal** — the specific domain or script source that placed the provider in the unsupported list.
- [ ] **Each bucket-3 dead-company row cites a source** — Companies House URL with status text, HTTP error code, or redirect-target URL. Prose assertion of deadness without a source belongs under "What I did not do."
- [ ] **Scratch-path file contract honoured** — every dispatched fallback batch has `/tmp/populate-db-<run-id>/<batch>.summary.md` on disk (plus `<batch>.sql` where applicable); missing files redispatched, not silently dropped.
- [ ] **Orchestrator did not hand-write per-company rows or SQL** — the §6 table is assembled from agent-written summary files; any DB writes that bypassed `cernio resolve` were applied via `cat /tmp/populate-db-<run-id>/*.sql | sqlite3 state/cernio.db`.
- [ ] **Mechanical verification ran before DB apply** — per-batch file presence, summary frontmatter counts, SQL line count, and SQL parse-check all passed; failing batches were redispatched, not silently applied.
- [ ] **`uk_sponsor_entity_name` populated for every inserted/resolved company** — cite the per-company source: exact register entity name (with the register row quoted) or the literal string `unknown` with the reason no match was found. NULLs in this column post-insert are a step-5.5 failure.
- [ ] **`companies/potential.md` cleaned after successful insert** — the file's content after the run contains only companies that were neither resolved nor bespoke (or is empty). Diff or after-state reference cited.
- [ ] **"What I did not do" declaration emitted** — at the end of the run, a section names every company that was skipped, deferred, left in `potential.md` for human review, or partially processed, with the reason. If nothing was skipped, the section says so explicitly; it is not absent.
