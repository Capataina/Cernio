# Greenhouse Job Board API Reference

> Last verified: 2026-04-07 against Cloudflare (200+ jobs), XTX Markets (7 jobs), Helsing (103 jobs).

---

## Endpoints

| Endpoint | Purpose | Returns description? |
|----------|---------|---------------------|
| `GET /v1/boards/{slug}/jobs` | List all jobs (summary only) | No |
| `GET /v1/boards/{slug}/jobs?content=true` | List all jobs with full HTML description | Yes |
| `GET /v1/boards/{slug}/jobs/{id}` | Single job detail | Yes (always) |

Base URL: `https://boards-api.greenhouse.io`

No authentication required. Public API. No rate-limit headers observed.

---

## Pagination

**There is no pagination.** The API returns all jobs in a single response. The `page` and `per_page` query parameters are silently ignored. Cloudflare with 200+ jobs returns them all at once.

This means:
- No need to loop through pages.
- Response size can be large for companies with many openings.
- Using `?content=true` on the list endpoint for a large company will produce a very large response (each job includes full HTML description).

**Recommendation for Cernio:** Fetch the list endpoint *without* `?content=true` first (small response, fast). Then fetch individual job details only for jobs that pass location/title filtering.

---

## Response Shape

### Top-Level Structure

```json
{
  "jobs": [ ... ],
  "meta": {
    "total": 7
  }
}
```

The `meta.total` field was present for XTX Markets (7 jobs) but **not observed** for Cloudflare (200+ jobs). Treat `meta` as optional — use `jobs.len()` as the authoritative count.

### List Endpoint Fields (without `?content=true`)

```json
{
  "id": 7480799,
  "absolute_url": "https://boards.greenhouse.io/cloudflare/jobs/7480799?gh_jid=7480799",
  "internal_job_id": 3320057,
  "title": "Account Executive Public Sector UKI Ministry Of Defence (MoD)",
  "company_name": "Cloudflare",
  "location": {
    "name": "Hybrid"
  },
  "updated_at": "2026-02-06T10:04:45-05:00",
  "first_published": "2026-01-21T04:46:39-05:00",
  "requisition_id": null,
  "language": "en",
  "metadata": [ ... ] | null,
  "data_compliance": [ ... ]
}
```

### Additional Fields with `?content=true` or Detail Endpoint

These three fields appear **only** when using `?content=true` on the list endpoint or when hitting the detail endpoint `/jobs/{id}`:

```json
{
  "content": "<div>...HTML job description...</div>",
  "departments": [
    {
      "id": 70660,
      "name": "Field Sales",
      "child_ids": [],
      "parent_id": 29066
    }
  ],
  "offices": [
    {
      "id": 19988,
      "name": "London, United Kingdom",
      "location": "London, United Kingdom",
      "child_ids": [],
      "parent_id": 74843
    }
  ]
}
```

---

## Complete Field Reference

| Field | Type | Present in list? | Present in detail? | Notes |
|-------|------|-----------------|-------------------|-------|
| `id` | integer | Yes | Yes | Unique job ID. Use for detail endpoint. |
| `absolute_url` | string | Yes | Yes | Full URL to the job posting page. |
| `internal_job_id` | integer | Yes | Yes | Internal Greenhouse ID (different from `id`). |
| `title` | string | Yes | Yes | Job title as posted. |
| `company_name` | string | Yes | Yes | Display name of the company. |
| `location` | object | Yes | Yes | See location section below. |
| `updated_at` | string (ISO 8601) | Yes | Yes | Last modification timestamp with timezone offset. |
| `first_published` | string (ISO 8601) | Yes | Yes | Original publication date. Useful for filtering stale postings. |
| `requisition_id` | string or null | Yes | Yes | Internal requisition reference. Often null. |
| `language` | string | Yes | Yes | ISO 639-1 language code. Always `"en"` in practice. |
| `metadata` | array or null | Yes | Yes | Company-defined custom fields. **Can be null** (XTX) or an array (Cloudflare, Helsing). |
| `data_compliance` | array | Yes | Yes | GDPR consent configuration. Not useful for job matching. |
| `content` | string (HTML) | Only with `?content=true` | Yes | Full job description as HTML. The primary field for evaluation. |
| `departments` | array of objects | Only with `?content=true` | Yes | Department hierarchy. Each has `id`, `name`, `child_ids`, `parent_id`. |
| `offices` | array of objects | Only with `?content=true` | Yes | Office locations. Each has `id`, `name`, `location` (string), `child_ids`, `parent_id`. |

---

## Location Representation

This is the most complex and inconsistent part of the API. Location data appears in **three different places** with different formats.

### 1. `location.name` (always present)

A single string in the `location` object. Always present, never null in observed data.

```
┌────────────────────────────────────────────────────────────────┐
│  location.name examples from real data                         │
├────────────────────────────────────────────────────────────────┤
│  "Hybrid"                          ← Cloudflare (work policy)  │
│  "Distributed"                     ← Cloudflare (work policy)  │
│  "New York"                        ← XTX Markets (city only)   │
│  "Singapore, Central, Singapore"   ← XTX Markets (city+region) │
│  "Berlin; London; Munich"          ← Helsing (multi-city)      │
│  "Berlin; London; Munich; Paris"   ← Helsing (multi-city)      │
└────────────────────────────────────────────────────────────────┘
```

**Key observations:**
- Format is completely company-dependent. No standard schema.
- Can be a work-policy label ("Hybrid", "Distributed") instead of a geographic location.
- Can contain multiple cities separated by semicolons.
- Can include city only, city+country, city+region+country, or work-style labels.
- There are no separate `city`, `country`, or `region` sub-fields.
- Cannot be relied upon for precise geographic filtering.

### 2. `metadata` (company-configured, optional)

Some companies put structured location in custom metadata fields. Cloudflare uses a `"Job Posting Location"` metadata field:

```json
{
  "id": 8654220,
  "name": "Job Posting Location",
  "value": ["London, UK"],
  "value_type": "multi_select"
}
```

**Key observations:**
- This is company-specific. XTX Markets has `metadata: null`. Helsing has metadata but no location field in it.
- When present, the `value` is an array (multi_select), so a single job can list multiple locations.
- Format varies: `"London, UK"`, `"Austin, US"`, `"US"`, `"Remote"`.
- Cannot be relied upon to exist.

### 3. `offices` (detail endpoint / `?content=true` only)

The most structured location data. An array of office objects:

```json
{
  "id": 19988,
  "name": "London, United Kingdom",
  "location": "London, United Kingdom",
  "child_ids": [],
  "parent_id": 74843
}
```

Helsing example with multiple offices:

```json
[
  {"id": 4007986101, "name": "Berlin",  "location": "Berlin, Germany"},
  {"id": 4007984101, "name": "London",  "location": "London, United Kingdom"},
  {"id": 4007985101, "name": "Munich",  "location": "München, Germany"},
  {"id": 4007987101, "name": "Paris",   "location": "Paris, France"}
]
```

**Key observations:**
- `offices` is an array — a single job can have multiple offices.
- Each office has both `name` (short display name) and `location` (fuller string, but still not decomposed into city/country).
- The `location` string tends to be `"City, Country"` format, which is more parseable than `location.name`.
- `name` can differ from `location` (e.g., `"Munich"` vs `"München, Germany"`).
- Only available via detail endpoint or `?content=true` — not in the default list response.

### Location Filtering Strategy for Cernio

```
Priority 1: offices[].location  — most structured, "City, Country" format
             (requires detail fetch or ?content=true)

Priority 2: location.name       — always available but unreliable format
             Check for "London" substring as a fast pre-filter

Priority 3: metadata             — company-specific, may not exist

Recommended approach:
  1. Fetch list endpoint (no content) for speed
  2. Pre-filter on location.name containing target strings ("London", "UK", "United Kingdom", "Remote")
  3. For candidates that pass, fetch detail to get offices[] for precise matching
  4. Accept jobs where any offices[].location contains target city/country
```

---

## URL Format

The `absolute_url` field provides the direct link to the job posting page. Two formats observed:

| Company | URL format |
|---------|-----------|
| Cloudflare | `https://boards.greenhouse.io/cloudflare/jobs/7480799?gh_jid=7480799` |
| XTX Markets | `https://job-boards.greenhouse.io/xtxmarketstechnologies/jobs/6274458003` |
| Helsing | `https://helsing.ai/jobs/4334849101?gh_jid=4334849101` |

**Key observations:**
- The subdomain varies: `boards.greenhouse.io` vs `job-boards.greenhouse.io` vs custom domain.
- Some companies use their own domain (Helsing uses `helsing.ai`).
- The `gh_jid` query parameter is sometimes appended, sometimes not.
- Always use `absolute_url` as-is — do not construct URLs manually.

---

## Content Field (HTML Description)

The `content` field contains the full job description as an HTML string. This is the primary field for Claude's evaluation.

**Format:** Raw HTML with `<div>`, `<p>`, `<ul>`, `<li>`, `<strong>`, `<h2>`, `<h3>`, `<br>` tags. No markdown.

**Typical sections within the HTML:**
- About the company
- Role overview / what you'll do
- Responsibilities
- Requirements / qualifications
- Nice-to-haves
- Benefits / compensation
- Equal opportunity statement

**For Cernio's Rust fetcher:** Strip HTML tags to extract plain text for storage in `raw_description`. The HTML structure is not consistent enough across companies to parse semantically — treat it as a blob of text for Claude to evaluate.

---

## Metadata Field

The `metadata` field is either `null` or an array of company-defined custom fields.

```json
// Can be null (XTX Markets):
"metadata": null

// Or an array of objects (Cloudflare):
"metadata": [
  {
    "id": 825338,
    "name": "Cost Center",
    "value": "5150 - Field Sales",
    "value_type": "single_select"
  },
  {
    "id": 8654220,
    "name": "Job Posting Location",
    "value": ["London, UK"],
    "value_type": "multi_select"
  }
]

// Or with different field types (Helsing):
"metadata": [
  {
    "id": 7229084101,
    "name": "Confidential role",
    "value": false,
    "value_type": "yes_no"
  }
]
```

**Value types observed:** `single_select` (string value), `multi_select` (array of strings), `yes_no` (boolean).

**For Cernio:** Metadata is not reliable for filtering because field names and presence vary by company. It can provide supplementary signal (e.g., department) but should not be a primary filter.

---

## Departments Field

Only available via detail endpoint or `?content=true`.

```json
"departments": [
  {
    "id": 70660,
    "name": "Field Sales",
    "child_ids": [],
    "parent_id": 29066
  }
]
```

- Array of department objects.
- Each department has `id`, `name`, `child_ids`, `parent_id` (hierarchical).
- Can be used for supplementary filtering (e.g., exclude "Sales", "Marketing" departments).
- Usually a single department, but the array structure allows multiple.

---

## Data Model Mapping for Cernio

```
Greenhouse field          →  Cernio jobs table field
─────────────────────────────────────────────────────
id                        →  (use to construct detail URL)
title                     →  title
absolute_url              →  url
location.name             →  location (fallback)
offices[0].location       →  location (preferred, from detail)
content (HTML stripped)    →  raw_description
first_published           →  posted_date
departments[0].name       →  (store in parsed_tags)
offices (all)             →  (store in parsed_tags for multi-location)
```

---

## Differences from Lever API

For implementers already familiar with the Lever fetcher:

| Aspect | Lever | Greenhouse |
|--------|-------|------------|
| Base URL | `api.lever.co/v0/postings/{slug}` | `boards-api.greenhouse.io/v1/boards/{slug}/jobs` |
| Pagination | Yes (offset-based, 10 per page default) | No (all jobs in one response) |
| Description in list | Always included (`descriptionPlain`, `description`) | Only with `?content=true` (as `content`) |
| Location format | `workplaceType` + `categories.location` | `location.name` + `offices[]` |
| Multiple locations | `categories.allLocations[]` | `offices[]` array |
| Department | `categories.department` | `departments[]` array |
| Job ID format | UUID string | Integer |
| Apply URL | Construct from `{base}/{slug}/{id}` | Use `absolute_url` directly |
