---
name: resolve-portals
description: "AI fallback for ATS portal resolution — handles companies the mechanical `cernio resolve` Rust script could not match. Uses WebSearch + careers-page inspection to extract the real ATS slug (non-obvious: legal entity, former name) or to mark the company bespoke when the provider is unsupported (iCIMS, Taleo, Personio, etc.). Every slug is verified against the provider's JSON API before write. Invoke when the user says 'resolve portals', 'find ATS for remaining companies', 'handle unresolved companies', 'resolve the unmatched ones', 'find their job boards', 'the resolver failed on these', or names companies still in `potential` status after `cernio resolve` ran. Not for the initial mechanical probe (that is `cernio resolve` via populate-db), discovering new companies (use discover-companies), grading (use grade-companies), or searching already-resolved companies (use search-jobs). Use this skill whenever companies remain in `potential` after the script has run, even if the user does not name it explicitly."
---

# Resolve Portals

AI fallback for companies the mechanical resolver could not match. Every company reaching this skill has already had its obvious slug variants probed — `{company-name}`, `{domain-without-tld}`, `{hyphenated-name}` — against every supported ATS. None returned valid results. The remaining cases need judgment: find the real careers page, inspect it for the actual ATS link, and verify the slug against the provider's JSON API.

The skill's single output is a lifecycle transition: every target company moves from `potential` to either `resolved` (ATS found and verified) or `bespoke` (no supported ATS; careers URL preserved). Dead companies are surfaced with evidence and left for the user to decide.

---

## Mandatory Read Before Resolving Any Company

| # | What | Evidence it was read |
|---|---|---|
| 1 | **`references/ats-providers.md`** (the full file) | You can cite the exact JSON API endpoint for each of the 7 supported providers and name the SmartRecruiters false-positive rule (`totalFound > 0` required) |

Without the reference, the agent cannot verify slugs correctly, misses the SmartRecruiters false-positive trap, and misclassifies unsupported providers. A resolution session without this read produces silently broken DB entries. When delegating work to subagents, the full reference text is embedded verbatim in each agent's prompt — agents cannot read the skill's references themselves.

---

## Context

The companies reaching this skill fall into three buckets. Categorisation happens during inspection, not upfront:

1. **Non-obvious slug on a supported ATS.** The company uses Greenhouse, Lever, Ashby, etc., but the script's candidate set missed the real slug. Canonical examples: XTX Markets → `xtxmarketstechnologies` (legal entity), Wise → `transferwise` (former name), Cleo → `cleoai` (suffix). The careers page outbound "Apply" link reveals the correct slug directly — no further guessing.
2. **Unsupported ATS provider.** iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, Recruitee, Breezy HR, Wellfound. These become `bespoke` with the careers URL preserved. Recognition markers for each are in `references/ats-providers.md`.
3. **Dead or dissolved company.** No working website, no careers page, Companies House shows dissolved or in liquidation. Report with evidence and stop; do not insert anything.

---

## Workflow

### 0. Run `cernio resolve` first

The mechanical resolver is free to run and may resolve companies that have never been probed — typically newly inserted rows added after the last `populate-db` cycle, or rows whose probe candidate set has been updated since they last failed. Running the script first converts those rows from `potential` to `resolved` at zero cost, shrinking the set this skill has to AI-fallback.

Dry-run first, real run second:

```bash
cernio resolve --dry-run    # preview the candidate list + slug candidates
cernio resolve              # execute the mechanical probe
```

Paste both outputs into the chat response before moving to step 1. If `cernio resolve --dry-run` shows zero unresolved companies, skip the execution and note "no unresolved companies; cernio resolve not run" explicitly — silence on this is the documented abstention pattern, not evidence the step was unnecessary.

**No substitution.** Do not skip this step on the assumption that `populate-db` or a previous `resolve-portals` run has already resolved everything. The skill's input (`WHERE status = 'potential'`) is defined by DB state, and DB state changes between invocations; running the script first is the only way to guarantee the AI fallback handles genuinely-unresolved companies, not ones the script could have resolved mechanically.

### 1. Query unresolved companies

```sql
SELECT id, name, website, careers_url
FROM companies
WHERE status = 'potential';
```

Present the list to the user. For large batches (more than ~10), propose processing in batches so the review gate stays tractable.

### 2. For each company, find the careers page

The mechanical resolver already exhausted slug guessing. Guessing further is wasted effort. Go to the source:

- `WebSearch` for `"{company name}" careers` or `"{company name}" jobs`
- Visit the company's website; find the Careers / Jobs link in navigation or footer
- Inspect the careers page for ATS markers: outbound links, iframe sources, embedded JavaScript widgets, redirect chains

ATS identification signals per provider (full playbook in `references/ats-providers.md`):

- `boards.greenhouse.io` or `job-boards.greenhouse.io` → Greenhouse
- `jobs.lever.co` → Lever
- `jobs.ashbyhq.com` → Ashby
- `apply.workable.com` → Workable
- `jobs.smartrecruiters.com` → SmartRecruiters
- `*.myworkdayjobs.com` → Workday (stores subdomain + site name in `ats_extra`)
- `eightfold.ai` in page source → Eightfold (stores subdomain in `ats_extra`)

**Per-company evidence obligation.** `WebSearch` is a low-pretraining-support tool and page-source inspection is a specific action the agent will quietly substitute with prose reasoning unless evidence is required. For each company, the step-5 summary-table `Evidence` column cites:

- The exact `WebSearch` query used (or the direct careers URL from `companies.careers_url` if already known, identified as such).
- The careers-page URL visited.
- The specific ATS signal observed — outbound link domain, iframe src, embed-script URL, or redirect-chain target. Quote the signal verbatim, not as paraphrase.
- When the signal points at an unsupported provider (bespoke path), quote the URL fragment that matched the unsupported-providers table in `references/ats-providers.md`.
- When the company is dead, cite the specific source — Companies House URL with status text, the HTTP response code of the website, or the parent-company redirect destination.

An evidence-column entry that reads "found via careers page" without the URL and the signal is not evidence; it is narrative. Such rows are not permitted to pass to step 3.

### 3. Verify the portal against the provider's JSON API

Finding an ATS URL on the careers page is evidence; confirming the slug returns valid job data is verification. Extract the slug, hit the provider's JSON API endpoint (exact endpoints in the reference file), and confirm a parseable response:

- **Greenhouse / Lever / Ashby / Workable:** 200 with valid JSON = confirmed; 404 = slug does not exist; valid response with zero jobs = valid slug with no current openings (still confirmed).
- **SmartRecruiters:** 200 is **not** sufficient. The API returns `{"totalFound": 0, "content": []}` for fabricated slugs. Confirmation requires `totalFound > 0` OR a SmartRecruiters link on the company's actual careers page combined with a response that parses cleanly. When ambiguous, mark as bespoke and flag the uncertainty.
- **Workday:** POST `{"limit": 1, "offset": 0}` to the API endpoint. 200 with `jobPostings` = confirmed.

A valid portal with zero current jobs is still a valid resolution — the company may simply not be hiring right now and the slug will light up later.

### 4. Handle the result

**Supported ATS found:**

```sql
INSERT INTO company_portals (company_id, ats_provider, ats_slug, ats_extra, verified_at, is_primary)
VALUES (?, ?, ?, ?, date('now'), 1);

UPDATE companies SET status = 'resolved' WHERE id = ?;
```

If multiple portals are found for one company (active Ashby + residual Workable after a migration), insert both. Mark the active one `is_primary = 1` and the residual one `is_primary = 0`. Multi-portal entries are expected, not errors.

**Unsupported ATS or custom portal:**

```sql
UPDATE companies
SET status = 'bespoke', careers_url = ?
WHERE id = ?;
```

The careers URL points to the actual job listings page, not just the homepage — downstream bespoke search needs a page that shows jobs.

**Dead or dissolved company:**

Do not insert. Report to the user with specific evidence — Companies House status, HTTP 404 on the website, redirect to a parent company, no activity in the last 12 months. The user decides whether to delete the DB entry or leave it for future retry.

### 5. Present the summary table before any DB write

```
| Company        | Result   | ATS        | Slug                     | Evidence                                                                        |
|----------------|----------|------------|--------------------------|---------------------------------------------------------------------------------|
| XTX Markets    | resolved | greenhouse | xtxmarketstechnologies   | `xtxmarkets.com/careers` → outbound `boards.greenhouse.io/xtxmarketstechnologies` · API `total=14` |
| SurrealDB      | bespoke  | pinpoint   | —                        | `surrealdb.com/careers` → iframe src `surrealdb.pinpointhq.com/en/jobs` (unsupported) |
| Vypercore      | dead     | —          | —                        | Companies House `13859477` status "Liquidation" (2025-08-14)                    |
```

Evidence column entries read like the rows above — specific URLs, quoted outbound-link domains, API response fields. Prose assertions ("found via careers page") are not accepted — they do not constitute evidence.

Wait for user approval before executing any SQL. This is a review gate — the user catches misclassified providers, wrong slugs, or premature "dead" calls.

### 6. Declare what was skipped

Close the run with a "What I did not do" section covering: companies left in `potential` status because evidence could not be gathered confidently, SmartRecruiters cases where `totalFound = 0` on an apparent SmartRecruiters careers page made the slug ambiguous, companies where the careers page itself failed to load, and any company where the three-bucket classification (supported / bespoke / dead) did not land cleanly. If nothing was skipped, say so explicitly — silence is the Claude abstention pattern, and the declaration is the structural counter.

---

## Lessons From Production

These are failure modes observed in real resolution runs, not theoretical concerns:

- **Slug guessing keeps failing; the careers page keeps being the answer.** Every non-obvious slug seen in production (XTX, Wise, Cleo, Thought Machine) was trivially extractable from the careers page outbound link. The careers page is the authoritative source.
- **SmartRecruiters returns 200 for fabricated slugs.** `totalFound > 0` is the only trustworthy signal. Without this check, the DB accumulates false-positive SmartRecruiters entries that yield zero jobs forever.
- **Companies migrate ATS providers and leave residues.** ClearBank has an active Ashby board and a zero-job residual Workable board. The schema supports `is_primary` on `company_portals` precisely for this — record both, flag the active one.
- **Companies House is the fastest death verifier for UK companies.** When a website is dead, Companies House resolves "dissolved / in liquidation / active" in seconds and prevents hours of searching for a ghost.
- **Unsupported ATS providers are common, not exotic.** Pinpoint HQ, Personio, iCIMS, Taleo, BambooHR, Jobvite all exist in the current company universe. Bespoke with careers URL preserves the information for later manual handling or future ATS support.

---

## Subagent Context Requirements

When dispatching parallel resolution work (one subagent per 3–5 unresolved companies), each subagent prompt embeds every item below. Subagents run in isolated contexts and cannot read the skill directory or query the database themselves; anything not embedded in the prompt is invisible to them.

- The **full text of `references/ats-providers.md`** — verbatim, not summarised.
- The **list of assigned companies** with their existing `website` and `careers_url` fields.
- The step-2 per-company evidence obligation reproduced verbatim, so the subagent returns rows with query + page URL + ATS signal quote (not narrative).
- The summary-table column format from step 5 reproduced verbatim (company / result / ATS / slug / evidence), so subagent output can be concatenated without reformatting.
- The SmartRecruiters `totalFound > 0` rule stated inline in the prompt, not only linked — tool-use obligations degrade when buried in references (F13).

The failure mode this section defends against is subagent prompts that paraphrase the reference material rather than embed it. Paraphrased-input subagents drop the SmartRecruiters false-positive trap on real companies — this has been observed in practice, not a theoretical concern.

---

## Reference Loading

**Mandatory-core — read at skill invocation:**

- `references/ats-providers.md` — complete reference: 7 supported providers with API endpoints and verification rules, 9+ unsupported providers with recognition markers, careers-page identification playbook, common slug patterns that defeat mechanical resolution.

---

## Inviolable Rules

1. **Every resolved portal is verified against the provider's JSON API before DB write.** Raw careers-page evidence is necessary but not sufficient.
2. **SmartRecruiters requires `totalFound > 0`.** A 200 with zero totalFound is ambiguous and defaults to bespoke when the careers page itself does not link to SmartRecruiters.
3. **No DB write without user approval of the summary table.** The review gate catches mislabelled providers and wrong-slug assignments before they reach production.
4. **Bespoke entries point to a working job listings page.** Not a homepage. Downstream bespoke search depends on this.
5. **Dead-company claims carry concrete evidence.** Companies House status, HTTP 404, redirect-to-parent. No unsubstantiated "seems dead" calls.
6. **Every summary-table row cites specific artefacts.** URLs, quoted outbound-link domains, API response fields, Companies House numbers — not narrative. Prose evidence ("found via careers page") is rejected at step 3.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. Items that cannot be evidenced in the agent's own output are either skipped and declared under "What I did not do" (step 6), or the skill has not finished.

- [ ] **Reference read fresh this invocation** — cite the tool call that read `references/ats-providers.md` in full.
- [ ] **`cernio resolve --dry-run` + `cernio resolve` run at step 0** — both outputs pasted verbatim in chat before step 1. If the dry-run showed zero candidates, the "not run" declaration is stated explicitly; silence fails this item.
- [ ] **SQL query for unresolved companies run and shown** — the `SELECT ... WHERE status = 'potential'` result is visible in the transcript; row count named.
- [ ] **Each company has a per-company evidence row** — summary table Evidence column names a careers-page URL + ATS signal quote (or equivalent for bespoke and dead paths).
- [ ] **Each supported-ATS resolution has a JSON API response line** — endpoint hit, HTTP status, and the response field that confirms it (`total`, `totalFound`, array length).
- [ ] **SmartRecruiters rows cite `totalFound > 0`** explicitly, or are demoted to bespoke with the careers-page link documenting the ambiguity.
- [ ] **Bespoke rows cite the careers URL and the unsupported-ATS signal** — URL fragment matching the unsupported-providers table in `ats-providers.md`.
- [ ] **Dead rows cite a source** — Companies House URL + status text, HTTP error code, or redirect-target URL. Prose-only "seems dead" rows fail this item.
- [ ] **Multi-portal cases have `is_primary` assignments justified in the Evidence column** — active = 1, residual = 0, with a sentence naming which portal is active and why.
- [ ] **Duplicate `company_portals` rows avoided** — cite the pre-insert check (SELECT query or UNIQUE constraint reliance) that catches duplicates.
- [ ] **Summary table presented and approved** — the table was emitted, the user explicitly approved, and the approval turn is identifiable in the transcript.
- [ ] **"What I did not do" declaration emitted** — at the end of the run, a section names every company left in `potential`, every SmartRecruiters ambiguity resolved against confirmation, every careers-page that failed to load. If nothing was skipped, the section says so explicitly; it is not absent.
