# ATS Fetchers

> Cernio's job-board integration layer — six provider-specific fetchers that normalise heterogeneous ATS APIs into a single `AtsJob` shape.

---

## Scope / Purpose

Cernio's company universe is hosted across many different applicant tracking systems. Each provider has its own API shape, its own URL conventions, its own quirks (rate limits, optional fields, region-specific endpoints). This subsystem hides all of that behind a uniform interface so the pipeline never has to know which provider a job came from.

The ATS layer owns:
- per-provider HTTP clients that know how to list and detail jobs,
- response deserialisation against provider schemas (Serde),
- normalisation into the shared `AtsJob` / `SlugProbeResult` shapes,
- retry-on-transient-failure logic (`get_with_retry` / `post_json_with_retry`),
- provider-specific hazards (SmartRecruiters false positives, Lever EU domain, Workday variable subdomains).

It does **not** own:
- which companies to search or resolve — that is the pipeline layer's job,
- location / exclusion / inclusion filtering — that is `config` + `pipeline/search`,
- persistence — that is `db/`.

---

## Boundaries / Ownership

### Module layout

```text
src/ats/
├── mod.rs             # Module declarations (per-provider pub mod)
├── common.rs          # AtsJob, SlugProbeResult, retry helpers
├── lever.rs           # Lever — US + EU API domains
├── greenhouse.rs      # Greenhouse — semicolon locations, Remote/Hybrid inference
├── ashby.rs           # Ashby — postalAddress decomposition
├── workable.rs        # Workable — country-code locations array
├── smartrecruiters.rs # SmartRecruiters — requires totalFound > 0 check
└── workday.rs         # Workday — variable subdomain + site in ats_extra
```

Each provider is a flat module with:
- provider-specific request types (`#[derive(Deserialize)]` shapes that mirror the API),
- a `fetch_jobs(slug, …) -> Vec<AtsJob>` or equivalent public entry,
- a `probe(client, slug) -> Option<SlugProbeResult>` that tests whether a slug is valid,
- a `normalise_*` function converting the raw response into `AtsJob`,
- an inline `#[cfg(test)] mod tests` block with 8–16 tests per provider (72 total — see `notes/testing-strategy.md`).

### What a seventh provider looks like

New providers follow the same contract:
1. Add `pub mod newprovider;` to `src/ats/mod.rs`.
2. Define request types in `src/ats/newprovider.rs`.
3. Implement `fetch_jobs` + `probe` + `normalise_*`.
4. Wire the provider into `pipeline::resolve` (probe path) and `pipeline::search` (fetch path).
5. Add a location pattern entry to `profile/preferences.toml`'s `[search_filters.locations.newprovider]`.
6. Add inline tests mirroring the existing six.

Eightfold.ai has been partially probed against but has no fetcher module yet — companies on Eightfold are recorded in `company_portals` with `ats_extra` (subdomain + domain) but not actively searched until the fetcher lands.

---

## Current Implemented Reality

### Shared types (`common.rs`)

```rust
pub struct AtsJob {
    external_id: String,            // Provider-specific job ID
    title: String,
    url: String,                    // Direct application URL
    location: Option<String>,       // Single display location
    all_locations: Vec<String>,     // Primary + secondary, used for filtering
    remote_policy: Option<String>,
    posted_date: Option<String>,    // ISO date string
    description: Option<String>,    // Plain text, HTML pre-stripped where possible
}

pub struct SlugProbeResult {
    provider: &'static str,
    slug: String,
    job_count: usize,
}
```

The pipeline layer works entirely with these two shapes. Nothing downstream ever touches a provider-specific type.

### Retry helpers

`get_with_retry(client, url, max_retries)` and `post_json_with_retry(...)` live in `common.rs`. Both retry on timeout / connection / request errors with linear backoff (500 ms × attempt number), and return immediately on non-retryable errors (4xx). Every provider uses these instead of raw `client.get(...)` so transient failures at the 200+ company scale don't kill a whole resolve run. The rationale is recorded in `notes/populate-db-lessons.md` (2026-04-09 discovery).

### Per-provider reality

| Provider | API base (list) | Slug complexity | `ats_extra` shape | Fetcher status | Tests |
|----------|----------------|-----------------|-------------------|----------------|-------|
| Greenhouse | `boards-api.greenhouse.io/v1/boards/{slug}/jobs` | Simple | — | In use | 13 |
| Lever | `api.lever.co/v0/postings/{slug}` **+** `api.eu.lever.co/v0/postings/{slug}` | Simple, two domains probed | — | In use | 16 |
| Ashby | `api.ashbyhq.com/posting-api/job-board/{slug}` | Simple | — | In use | 8 |
| Workable | `apply.workable.com/api/v1/widget/accounts/{slug}` | Simple | — | In use | 10 |
| SmartRecruiters | `api.smartrecruiters.com/v1/companies/{slug}/postings` | Simple, but requires `totalFound > 0` check | — | In use | 12 |
| Workday | `{company}.{wd1-12}.myworkdayjobs.com/wday/cxs/{company}/{site}/jobs` | Complex, variable subdomain + site | `{"subdomain":"wd3","site":"External"}` | In use | 13 |
| Eightfold | `{subdomain}/api/apply/v2/jobs?domain={domain}` | Complex, company-specific subdomain | `{"subdomain":"…","domain":"…"}` | Not yet implemented — companies recorded as bespoke | 0 |

All six implemented providers went through the Phase 4 test push (72 tests, offline JSON fixtures, 2026-04-10) — see `notes/testing-strategy.md` §Phase 4.

### Normalisation quirks (per provider)

These are the per-provider hazards the tests are designed to catch. Every one of them was a real-world data corruption or resolution failure found while building or running the pipeline.

- **Lever**: posted-at arrives as a millisecond epoch and must be converted to `YYYY-MM-DD`; description is built by concatenating multiple `lists` sections; locations arrive as an array that may be empty.
- **Greenhouse**: locations are often semicolon-separated strings; `Remote`/`Hybrid` must be inferred from the location text when the `offices` array is empty; `posted_date` falls through `first_published` → `updated_at`.
- **Ashby**: primary `location` is built by decomposing `postalAddress` fields; `secondaryLocations` merged into `all_locations`; `workplaceType` takes precedence over the legacy `isRemote` flag.
- **Workable**: location is composed from `city`, `state`, `countryCode`; empty strings skipped; `telecommuting` flag maps to remote policy.
- **SmartRecruiters**: the list endpoint does not return descriptions — they require a separate detail fetch; **the probe function must check `totalFound > 0`** because the API returns HTTP 200 with an empty list for any slug (documented in `notes/populate-db-lessons.md`); `ref_url` is used as URL when `id` is absent.
- **Workday**: `ats_extra` stores `{subdomain, site}` as JSON; `parse_extra` must gracefully handle missing, invalid, or non-string inputs; URLs built from `build_base_url` + `build_posting_url`; locations are pipe-separated; bullet fields concatenated into description.

---

## Key Interfaces / Data Flow

### Who calls what

```
┌─ pipeline/resolve ─────────────────────┐        ┌─ pipeline/search ─────────────────┐
│                                        │        │                                    │
│  for each slug candidate:              │        │  for each resolved portal:         │
│    for each provider:                  │        │    provider::fetch_jobs(slug, …)   │
│      provider::probe(slug) ──┐         │        │                       │            │
│                              ▼         │        │                       ▼            │
│           records to company_portals   │        │             Vec<AtsJob> → filters  │
└────────────────────────────────────────┘        └────────────────────────────────────┘
                        │                                        │
                        │  both use get_with_retry()             │
                        ▼                                        ▼
                  ┌─────────────────────────────────────────────────┐
                  │              reqwest::Client + retry              │
                  └─────────────────────────────────────────────────┘
```

`pipeline::resolve::run` iterates `slug_candidates(company_name)` (in `resolve.rs`) × every provider, calling `probe` for each. `pipeline::search::run` reads existing rows from `company_portals`, dispatches `fetch_jobs` per portal in parallel under a Tokio `Semaphore`, collects `Vec<AtsJob>`, and passes them into the filter stack.

### Contract with `config`

`SearchFilters::passes_location(provider, locations)` takes the provider name (`"lever"`, `"greenhouse"`, ...) as a string and looks up that provider's location patterns from `profile/preferences.toml`. This means **provider names are a shared identifier** between `ats/`, `config`, and `preferences.toml`. If a new provider is added, `preferences.toml` needs a matching `[search_filters.locations.<provider>]` entry or every job from that provider will fail the location filter.

### Contract with `db`

Providers never touch the database. The pipeline writes probe results into `company_portals` (composite unique `(company_id, ats_provider, ats_slug)`) and writes jobs into `jobs` keyed by `url`. The ATS layer produces plain values; the pipeline decides what to persist. See `systems/database.md` for the full schema.

---

## Implemented Outputs / Artifacts

- `src/ats/common.rs` — `AtsJob`, `SlugProbeResult`, `get_with_retry`, `post_json_with_retry`
- `src/ats/{lever,greenhouse,ashby,workable,smartrecruiters,workday}.rs` — per-provider fetchers + probes
- 72 inline tests (15–18 per provider in Phase 4) covering parsing, normalisation, URL construction, and the quirks listed above
- `context/references/greenhouse-api.md`, `smartrecruiters-api.md`, `workable-api.md` — external API reference docs consulted during fetcher authorship

---

## Known Issues / Active Risks

- **SmartRecruiters false positives.** The list endpoint returns HTTP 200 with `{totalFound: 0, content: []}` for any slug string, so a naive "200 OK means company exists" probe would inflate SmartRecruiters resolutions. The mandatory `totalFound > 0` check is enforced in `probe`, but any future refactor that replaces the check with a status-code-only test reintroduces the bug. A dedicated test (`probe_rejects_zero_total`) guards against this.
- **Lever EU companies missed without dual-domain probe.** `api.lever.co` and `api.eu.lever.co` are two independent job boards; a UK/EU company living exclusively on the EU domain will 404 on the US endpoint. Both must be probed for every resolve. Guarded by tests + documented in `notes/populate-db-lessons.md`.
- **Workday `ats_extra` is structurally load-bearing.** Unlike simple-slug providers, Workday cannot be fetched from `ats_slug` alone — `subdomain` (wd1–wd12) and `site` (External / CareerSite / …) are required to construct the URL. If `ats_extra` becomes null for a Workday portal, that portal silently stops producing jobs. The schema allows null but `pipeline::search` skips null-extra Workday portals with a log line. Similar risk for Eightfold.
- **Eightfold has no fetcher.** Companies on Eightfold get a `company_portals` row but no job fetches run against them. They effectively behave as bespoke companies for the search pipeline until an Eightfold fetcher lands.
- **Provider API drift.** Any of the six providers can change response shapes without notice. The inline test suite is the early-warning system — tests fixture-based so they do not detect live drift, but a failed deserialize in production will show up as "0 jobs fetched" for that provider.

Downstream impact: if fetchers misbehave, `pipeline::search` returns fewer jobs, the Jobs view looks stale, and the grading pipeline sees a stream of duplicate re-discoveries when the same jobs get picked up again after the API recovers. There is no cascading data corruption — missing jobs just fail to enter the DB.

---

## Partial / In Progress

None. The six implemented providers are stable. Eightfold is deferred.

---

## Planned / Missing / Likely Changes

- **Eightfold fetcher** — Atlassian, Nvidia, Cisco, a handful of others currently resolved as bespoke could move onto a proper searchable portal.
- **Pinpoint HQ fetcher** — surfaced in `notes/populate-db-lessons.md`; low priority until more companies on Pinpoint accumulate.
- **iCIMS / Taleo / BambooHR / Jobvite / Personio** — each has enough companies on it to justify a fetcher eventually, but all remain bespoke for now.
- **Rate-limit-aware backoff** — current retry is linear per-request. At very large scale (500+ companies) some providers return 429 and the right response is exponential backoff with jitter, not linear retry. Not hit in production yet.

---

## Durable Notes / Discarded Approaches

- **Early-termination slug probing was removed.** `pipeline::resolve` originally stopped at the first provider that accepted a slug. This missed multi-ATS companies (ClearBank on Ashby + Workable). The current resolver probes every provider for every slug candidate; the multi-portal schema in `db/` lets a company legitimately own multiple portals with `is_primary` flagging the one to search.
- **Mockito/HTTP mocking was rejected for tests.** Phase 4 went with offline JSON fixtures instead. Fixtures double as living documentation of each provider's response shape, run in well under a second, and are deterministic. See `notes/testing-strategy.md` §Decision 5.
- **Per-request retry beat per-batch retry.** Original design retried the whole resolve batch on any failure. At 200+ companies this wasted minutes for a single transient 502. Per-request retry with short linear backoff is the current model and recovers transparently.

---

## Obsolete / No Longer Relevant

None.
