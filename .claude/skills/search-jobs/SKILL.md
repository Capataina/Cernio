---
name: search-jobs
description: "Orchestrates full Cernio job-search — runs `cernio search` across every resolved-ATS company (Greenhouse / Lever / Ashby / Workable / SmartRecruiters / Workday fetchers) AND dispatches subagents to search every bespoke company (FAANG, HFT, Bloomberg, Arm, Mastercard, plus everything on iCIMS / Taleo / Personio / Pinpoint HQ / Eightfold) whose jobs the automated pipeline cannot touch. Bespoke subagents visit careers pages + aggregators (LinkedIn, Indeed, Glassdoor, BuiltIn); orchestrator inserts every match via `INSERT OR IGNORE INTO jobs`. Does not grade — hands pending queue to `grade-jobs`. Invoke on 'search for jobs', 'scan everything', 'find me roles', 'run a job search', 'check openings', 'what's available at [company]', 'scan all S-tier', 'refresh the jobs'. Not for discovering companies (discover-companies), resolving ATS (populate-db / resolve-portals), grading jobs (grade-jobs), or preparing applications (prepare-applications). Use whenever fresh jobs are needed."
---

# Search Jobs

End-to-end job-search orchestration. The Cernio company universe contains two structurally different classes of companies, and a complete job search covers both:

1. **Resolved-ATS companies** (`status = 'resolved'` in the `companies` table) — companies with Greenhouse / Lever / Ashby / Workable / SmartRecruiters / Workday portals. These are handled by `cernio search`, the Rust pipeline's bulk fetcher. Mechanical: thousands of raw jobs fetched, filtered, deduplicated, inserted — in seconds.
2. **Bespoke companies** (`status = 'bespoke'`) — companies on unsupported ATS platforms (iCIMS, Taleo, Personio, Pinpoint HQ, Eightfold — the last is recognised by Cernio but has no working fetcher and is treated as bespoke) or running custom portals (most FAANG, most HFT, many large public companies). These are invisible to `cernio search`. Finding their jobs requires visiting careers pages, searching aggregators, and inserting roles manually.

Run `SELECT status, COUNT(*) FROM companies GROUP BY status` at the top of the search to record current counts in the run output — counts shift as `discover-companies` and `resolve-portals` run between sessions.

**Bespoke search is not an afterthought.** The bespoke list includes the highest-signal employers in the universe — Apple, Google, Meta, Amazon, Microsoft, Citadel, Bloomberg, Jane Street, Arm, Mastercard. Running `cernio search` without a bespoke pass is a partial search. A session-7 audit found ~13 strong leads missed because bespoke search was skipped. The two halves carry roughly equal weight; the script is faster per company, but the bespoke pass covers the brand names that open doors.

This skill stops at insertion. Grading is `grade-jobs`'s job — this skill hands off the pending queue.

---

## Mandatory Reads Before Searching

| # | What | Evidence |
|---|---|---|
| 1 | **Every file in `profile/`** including the per-project files under `profile/projects/` (each carries an `active` / `paused` / `dormant` / `abandoned` status in its frontmatter; `index.md` is the inventory). Skip `profile/sync-summary.md` — it is operational metadata written by `populate-from-lifeos`, not profile data, and reading it for grading purposes wastes tokens. | The bespoke-search title-matching decisions (which roles to insert, which to skip as obvious senior-only) are grounded in profile target areas from `profile/preferences.toml`, technologies from `profile/skills.md`, and area sensitivities from `profile/lifestyle-preferences.md` (informational — formal lifestyle assessment lives in `grade-jobs`, not here) |
| 2 | **`references/bespoke-search-playbook.md`** | The bespoke-search workflow, which aggregators to try in which order, the INSERT-OR-IGNORE contract, the session-7 failure mode that drove the "results must be inserted, not just reported" rule |

The profile is not cached. Every invocation reads `profile/` fresh.

---

## Workflow

### 1. Run `cernio search` for the resolved-ATS half

The script handles the mechanical volume work across 287 resolved-ATS companies. It fetches job JSON from all supported ATS APIs, applies the location filter, applies the exclusion / inclusion keyword filters from `profile/preferences.toml`, deduplicates against the existing DB (URL as unique key), and inserts new jobs with `evaluation_status = 'pending'`.

| Command | Purpose |
|---------|---------|
| `cargo run -- search --dry-run` | Preview the batch: which companies will be scanned, expected job counts, filter statistics. Run first. |
| `cargo run -- search` | Execute the full pass across all resolved companies |
| `cargo run -- search --company "<name>"` | Scan a single company (ad-hoc check, or retry after a fix) |
| `cargo run -- search --grade S` | Scan only S-tier resolved companies — useful for a fast high-signal refresh |

Workflow: dry-run first, confirm scope, execute, review the per-company output. Note companies that failed to fetch (API errors, timeouts) for retry.

### 2. Report the script's output

After the script completes:

- Companies searched (resolved-ATS)
- Total jobs fetched from ATS APIs
- Jobs that survived the filter chain (location / exclusion / inclusion)
- Jobs that are new (not already in the DB)
- Companies that errored — names + reason

This gives the user a sense of scale before the bespoke pass.

### 3. Run the bespoke search pass — parallel subagents by company

This is the half that the script cannot touch. Query the bespoke list:

```sql
SELECT id, name, careers_url, grade
FROM companies
WHERE status = 'bespoke'
  AND grade IN ('S', 'A', 'B')
ORDER BY CASE grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 END;
```

(`status` is a single enum where `'bespoke'` and `'archived'` are mutually exclusive, so no separate archived filter is needed — the `companies` schema enforces this via CHECK constraint.)

Dispatch parallel subagents — one per batch of 3–5 bespoke companies. Each subagent prompt embeds verbatim:

- The **full text of `references/bespoke-search-playbook.md`** — subagents cannot read the skill's references
- The **relevant profile slice** — target role types, technologies, location preferences, visa status, plus the full text of `profile/lifestyle-preferences.md` (informational area awareness, not a filter — subagents do not skip roles for area reasons; `grade-jobs` performs the formal lifestyle assessment) — all pulled from `profile/` by the orchestrator
- The **list of assigned companies** (name + careers_url + grade)
- **Explicit instruction** to use `WebFetch` on the careers URL first, fall through to `WebSearch` on aggregators (LinkedIn, Indeed, Glassdoor, BuiltIn) per the playbook's priority order
- **Output-format obligation** — per-company structured findings (title, URL, location for each match), not narrative summaries
- **Explicit rule** — results are to be *returned* to the orchestrator as structured findings; the orchestrator inserts into the DB. Subagents do not write SQL.

S-tier bespoke companies go first (highest expected yield per minute). A-tier companies next. B-tier if time permits.

### 4. Insert bespoke results into the DB

After subagent results arrive, the orchestrator:

1. Maps each returned company name to its `company_id` via `SELECT id, name FROM companies WHERE status = 'bespoke';`. Flag any name that does not resolve.
2. Produces `INSERT OR IGNORE INTO jobs` statements for each returned role:

   ```sql
   INSERT OR IGNORE INTO jobs (company_id, title, url, location, evaluation_status, discovered_at)
   VALUES (?, ?, ?, ?, 'pending', datetime('now'));
   ```

3. Executes the inserts. The `OR IGNORE` clause relies on the `url` unique constraint — re-runs do not duplicate.
4. Reports: companies searched, roles found, roles inserted (vs roles that were already in the DB as duplicates).

**This step is load-bearing.** The session-7 failure mode was that subagent-found roles were reported conversationally and never inserted — they remained invisible to the TUI, the grading pipeline, and the application tracker. Insert, then report.

### 5. Confirm the pending queue

Summarise what is now pending:

```sql
SELECT COUNT(*) FROM jobs WHERE evaluation_status = 'pending';
```

Break down by source if useful: how many came from the script run, how many from bespoke search. Present the count grouped by company grade for handoff context.

### 6. Hand off to grade-jobs

This skill ends at insertion. Grading is `grade-jobs`'s job. Recommend the user invoke `grade-jobs` next — it will pick up the pending queue, prioritise by company grade × title promise, fetch missing descriptions via WebFetch, and assign grades with profile-grounded fit assessments.

Do not grade inline in this skill. The old version of this skill attempted inline grading and produced inconsistent results because the full rubric lived elsewhere. The current split — search-jobs discovers, grade-jobs grades — keeps each skill's purpose clean and the quality anchored to the right reference files.

### 7. Declare what was skipped

Close the run with a "What I did not do" section covering: resolved-ATS companies that errored during the script run and were not retried; bespoke companies whose careers pages failed to load and whose aggregator fallbacks found nothing; subagent-returned company names that did not resolve to a `company_id` and were flagged rather than inserted; known-bespoke companies that were deferred for this session (e.g. C-tier not worth the manual-search cost). If the search was clean — all ATS companies succeeded, all bespoke companies either returned roles or produced zero-result records, every returned name mapped — say so explicitly. Silent omission is not the same as nothing-to-declare.

---

## Reference Loading

**Mandatory-core — read at skill invocation every time:**

- `references/bespoke-search-playbook.md` — the bespoke-search workflow: which companies count as bespoke, where to look (careers page → LinkedIn → Indeed → Glassdoor → sector boards → site-specific search), per-company procedure, subagent return format, orchestrator insertion contract, known limitations (JavaScript-walled portals, SSO, LinkedIn rate limits).

---

## Inviolable Rules

1. **Every bespoke role found by a subagent is inserted into the `jobs` table.** Conversational reports without `INSERT OR IGNORE INTO jobs` statements are the session-7 failure mode — jobs become invisible to the TUI and the grading pipeline. Insert, then report.
2. **This skill does not grade.** Grading is `grade-jobs`. Inline grading in this skill produces inconsistent results because the rubric lives elsewhere.
3. **Bespoke search is part of every full search pass** — not an optional tail step. Running `cernio search` alone covers 287 of the 408 companies. The other 121 include the highest-signal employers in the universe; silently skipping them is a partial job search mislabelled as complete.
4. **Profile is read fresh every invocation.** No cached profile data in this skill or any subagent prompt.
5. **Subagents receive the full bespoke-search-playbook + the relevant profile slice embedded verbatim** in their prompts. Agents cannot read the skill's references.
6. **Subagents return structured findings, not SQL.** The orchestrator maps names to `company_id` and constructs the insert statements.
7. **The URL unique constraint drives dedup** — `INSERT OR IGNORE` is the insert form. Hand-coded "check if exists first" queries are unnecessary and error-prone.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. An item that cannot be evidenced in the agent's output is either unmet and surfaced under step 7 "What I did not do," or the skill has not finished.

- [ ] **`profile/` read fresh this invocation** — cite the tool call per file.
- [ ] **`references/bespoke-search-playbook.md` read fresh this invocation** — cite the tool call.
- [ ] **`cargo run -- search --dry-run` output pasted verbatim** — company list and expected counts appear in chat before the real run.
- [ ] **`cargo run -- search` output pasted verbatim** — per-company fetch results visible; error rows identified by company name.
- [ ] **Bespoke SQL query run and row count reported** — the `SELECT ... WHERE status = 'bespoke' AND grade IN ('S','A','B')` result is identifiable in the transcript.
- [ ] **Bespoke pass ran in priority order** — S-tier first, A-tier second, B-tier third; per-tier subagent dispatch counts cited.
- [ ] **Every subagent prompt embedded the full playbook + profile slice** — verified by inspecting the prompt contents in the transcript.
- [ ] **Subagent returns are structured per-company findings** — titles, URLs, locations in the per-company blocks; no narrative-only returns accepted.
- [ ] **Every returned role was inserted via `INSERT OR IGNORE INTO jobs`** — cite the generated SQL batch; every returned role appears in the SQL. A role reported but not inserted is the session-7 failure and fails this item.
- [ ] **Unresolved company-name flags surfaced** — any subagent-returned name that did not map to a `company_id` is named in step 7's declaration (not silently dropped).
- [ ] **Per-source insert vs already-duplicate counts cited** — the count of jobs the OR IGNORE silently skipped is reported alongside the new-insert count.
- [ ] **Pending queue count after insert cited** — `SELECT COUNT(*) FROM jobs WHERE evaluation_status = 'pending'` result appears in the handoff.
- [ ] **Handoff to `grade-jobs` stated** — the final report names `grade-jobs` as the next step.
- [ ] **No grading happened** — every new job is at `evaluation_status = 'pending'`; no `UPDATE jobs SET grade = ...` statements executed by this skill.
- [ ] **Step 7 "What I did not do" declaration emitted** — names errored ATS companies, bespoke-pass failures, unresolved names, deferred companies, or explicitly states "clean run".
