---
name: populate-db
description: "Validates discovered companies, runs the `cernio resolve` Rust script to probe ATS slug candidates across Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday, and Eightfold, and handles AI fallback for companies the script cannot match — either by finding the correct slug on the careers page or marking the company as bespoke. Invoke when the user says 'populate the database', 'populate db', 'research these companies', 'add these companies', 'resolve the portals', 'process potential.md', or after a discovery run has produced entries in `companies/potential.md` that still need processing. Not for discovering new companies (use discover-companies), grading companies (use grade-companies), searching jobs on already-resolved companies (use search-jobs), or handling pure AI-only portal resolution (use resolve-portals — this skill orchestrates the full discovery-to-DB pipeline including the script + the AI fallback). Use this skill whenever there are unprocessed companies that need to reach the database as either resolved or bespoke, even if the user does not name it explicitly."
---

# Populate DB

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

### 6. Present results for user review

Before any DB write, produce a summary table:

```
| Company        | Result   | ATS        | Slug                     | Source                                |
|----------------|----------|------------|--------------------------|---------------------------------------|
| Cloudflare     | resolved | greenhouse | cloudflare               | cernio resolve (mechanical)           |
| XTX Markets    | resolved | greenhouse | xtxmarketstechnologies   | AI fallback — careers page outbound   |
| SurrealDB      | bespoke  | pinpoint   | —                        | surrealdb.pinpointhq.com (unsupported)|
| Vypercore      | removed  | —          | —                        | Companies House: in liquidation       |
```

Wait for user approval. This is a review gate — the user can correct mistakes, skip entries, or request re-research. Writing to the DB only after confirmation.

### 7. Commit and clean up

After approval, insert resolved and bespoke companies into the DB, then remove them from `companies/potential.md`. `potential.md` is a landing zone — its steady state after a population run is "only contains unprocessed entries."

---

## Subagent Context Requirements

When delegating fallback work to parallel subagents (one per 3–5 unresolved companies), each subagent prompt includes:

- The **full content of `references/ats-providers.md`** — verbatim, not summarised. Subagents cannot read the skill's files.
- The **full content of every file in `profile/`** — needed for the relevance judgment in step 2.
- The list of assigned companies with their discovery-source metadata.
- Explicit instruction to output per-company resolution records in the summary-table format, not a narrative summary.

Under-contextualising a subagent produces generic output that mislabels providers. Over-share.

---

## Reference Loading

**Mandatory-core — read at skill invocation:**

- `references/ats-providers.md` — the complete ATS provider reference: supported providers with API details + verification pattern, unsupported providers with recognition markers, careers-page identification playbook, common slug patterns that defeat mechanical resolution.

---

## Inviolable Rules

1. **No DB write without user approval of the summary table.** Writes are reversible but noisy; the review gate catches mislabelled providers and mis-slugged companies before they reach the DB.
2. **Each resolution must be verified against the provider's JSON API** (except bespoke). For SmartRecruiters, verification means `totalFound > 0` — 200 alone is not evidence.
3. **Bespoke companies carry a working careers URL**, not a homepage. The bespoke-search workflow downstream needs a page that actually shows jobs.
4. **Do not grade during this skill.** Grading is `grade-companies`. Mixing the two contaminates both passes.
5. **Profile is read fresh** — no caching, no embedded snapshots, no reliance on earlier-session memory of the profile.

---

## Quality Checklist

- [ ] Every processed company was validated for activity, independence, and engineering fit before probing
- [ ] `cernio resolve` was run before any manual research — the skill does not duplicate the script's work
- [ ] Failed-resolve companies were investigated via AI fallback, not abandoned
- [ ] Every resolved portal was verified against the provider's JSON API (SmartRecruiters: `totalFound > 0`)
- [ ] Each slug belongs to the correct company (not a similarly-named different company)
- [ ] No duplicate inserts — existing DB entries were checked before writing
- [ ] Bespoke entries point to a working careers URL, not a homepage
- [ ] Dead / dissolved companies have evidence cited (Companies House status, 404 website, redirect to parent)
- [ ] Results summary table was presented and user-approved before any DB write
- [ ] `companies/potential.md` was cleaned up after successful insertion
