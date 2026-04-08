# Workable Widget API Reference

> Last verified: 2026-04-07 against Board of Innovation (6 jobs), Luminance (30+ jobs).

---

## Endpoints

| Endpoint | Purpose | Returns description? |
|----------|---------|---------------------|
| `GET /api/v1/widget/accounts/{slug}` | List all jobs (summary only) | No |
| `GET /api/v1/widget/accounts/{slug}?details=true` | List all jobs with full HTML description | Yes |

Base URL: `https://apply.workable.com`

No authentication required. Public widget API. No rate-limit headers observed.

**There is no individual job detail endpoint on the public widget API.** The `?details=true` query parameter on the list endpoint is the only way to get descriptions. The v3 API requires authentication (customer-only).

---

## Slug Verification

Non-existent slugs return **HTTP 404** (not 200 with empty data). This means a simple HTTP status check is sufficient for slug probing -- a 200 response confirms the company exists on Workable.

---

## Pagination

**There is no pagination.** The API returns all jobs in a single response. No `page`, `per_page`, `offset`, or `limit` parameters are supported. All jobs come back at once.

**Filtering is not possible.** No query parameters for location, department, title, or keyword search exist on the public widget API.

**Recommendation for Cernio:** Fetch without `?details=true` first (smaller response). Then fetch with `?details=true` only when job descriptions are needed for evaluation. Since there is no pagination, large companies will produce large responses with `?details=true`.

---

## Response Structure

### Top-Level Object

```json
{
  "name": "Company Display Name",
  "description": "<p>HTML company description...</p>",
  "jobs": [ ... ]
}
```

| Field | Type | Notes |
|-------|------|-------|
| `name` | string | Company display name (may differ from slug) |
| `description` | string | HTML-formatted company description |
| `jobs` | array | All published jobs |

### Job Object (without `?details=true`)

```json
{
  "title": "Account Executive",
  "shortcode": "C7389E54B7",
  "code": "",
  "employment_type": "Full-time",
  "telecommuting": false,
  "department": "Commercial",
  "url": "https://apply.workable.com/j/C7389E54B7",
  "shortlink": "https://apply.workable.com/j/C7389E54B7",
  "application_url": "https://apply.workable.com/j/C7389E54B7/apply",
  "published_on": "2025-06-06",
  "created_at": "2025-06-06",
  "country": "United Kingdom",
  "city": "London",
  "state": "England",
  "education": "Bachelor's Degree",
  "experience": "",
  "function": "Sales",
  "industry": "Internet",
  "locations": [
    {
      "country": "United Kingdom",
      "countryCode": "GB",
      "city": "London",
      "region": "England",
      "hidden": false
    }
  ]
}
```

### Job Object (with `?details=true`)

Same fields as above, plus:

```json
{
  "description": "<p>Full HTML job description including responsibilities, requirements, etc.</p>"
}
```

The `description` field contains the complete job posting as HTML. There is no separate `requirements` or `benefits` field -- everything is embedded in the single `description` HTML string.

---

## Field Reference

| Field | Type | Always present? | Notes |
|-------|------|----------------|-------|
| `title` | string | Yes | Job title |
| `shortcode` | string | Yes | Unique job identifier (hex string, e.g. `"C7389E54B7"`) |
| `code` | string | Yes | Internal job code; often empty string `""` |
| `employment_type` | string | Yes | e.g. `"Full-time"`, `"Other"` |
| `telecommuting` | bool | Yes | `true` if fully remote |
| `department` | string | Yes | Can be empty string `""` |
| `url` | string | Yes | Public job page URL |
| `shortlink` | string | Yes | Same as `url` in practice |
| `application_url` | string | Yes | Direct apply link (`url` + `/apply`) |
| `published_on` | string | Yes | ISO date `"YYYY-MM-DD"` |
| `created_at` | string | Yes | ISO date `"YYYY-MM-DD"` |
| `country` | string | Yes | Full country name: `"United Kingdom"`, `"United States"` |
| `city` | string | Yes | City name; can be empty string `""` |
| `state` | string | Yes | Region/state; can be empty string `""` |
| `education` | string | Yes | e.g. `"Bachelor's Degree"`, `"Master's Degree"`, or `""` |
| `experience` | string | Yes | e.g. `"Entry level"`, `"Mid-Senior level"`, `"Director"`, or `""` |
| `function` | string | Yes | e.g. `"Sales"`, `"Consulting"`, `"Human Resources"`, or `""` |
| `industry` | string | Yes | e.g. `"Internet"`, `"Management Consulting"` |
| `locations` | array | Yes | Array of location objects (see below) |
| `description` | string | Only with `?details=true` | Full HTML job description |

### Location Object

```json
{
  "country": "United Kingdom",
  "countryCode": "GB",
  "city": "London",
  "region": "England",
  "hidden": false
}
```

| Field | Type | Notes |
|-------|------|-------|
| `country` | string | Full country name |
| `countryCode` | string | ISO 3166-1 alpha-2 code (uppercase: `"GB"`, `"US"`, `"BE"`, `"AU"`) |
| `city` | string | City name |
| `region` | string | State/region/province |
| `hidden` | bool | Whether location is hidden on the public page |

---

## Location Handling

### Key Observations

1. **Dual location representation.** Location appears both as top-level flat fields (`country`, `city`, `state`) AND as a structured `locations` array. The `locations` array is the canonical source.

2. **Multiple locations supported.** The `locations` array can contain multiple entries when a job is posted to more than one location. The top-level `country`/`city`/`state` fields appear to reflect the primary location only.

3. **Location is always present.** In all observed responses, `country`, `city`, `state`, and `locations` are always present (never null/missing). However, `city` and `state` can be empty strings `""`.

4. **Country is a full name, not a code.** The top-level `country` field uses full names (`"United Kingdom"`, `"Belgium"`). The `locations[].countryCode` field uses ISO alpha-2 codes (`"GB"`, `"BE"`).

5. **Remote is indicated by `telecommuting: true`.** There is no separate remote policy field or `"Remote"` location string. Check the boolean.

### Location Format Examples

| Scenario | `country` | `city` | `state` | `locations[0].countryCode` |
|----------|-----------|--------|---------|---------------------------|
| London, UK | `"United Kingdom"` | `"London"` | `"England"` | `"GB"` |
| New York, US | `"United States"` | `"New York"` | `"New York"` | `"US"` |
| Antwerp, Belgium | `"Belgium"` | `"Antwerp"` | `"Flanders"` | `"BE"` |
| Sydney, Australia | `"Australia"` | `"Sydney"` | `"New South Wales"` | `"AU"` |

---

## URL Patterns

| Purpose | Pattern | Example |
|---------|---------|---------|
| Company board | `https://apply.workable.com/{slug}/` | `https://apply.workable.com/luminance-1/` |
| Job listing API | `https://apply.workable.com/api/v1/widget/accounts/{slug}` | |
| Job page (human) | `https://apply.workable.com/j/{shortcode}` | `https://apply.workable.com/j/C7389E54B7` |
| Apply link | `https://apply.workable.com/j/{shortcode}/apply` | `https://apply.workable.com/j/C7389E54B7/apply` |

Note: the job URL does NOT include the company slug -- just the global shortcode.

---

## Slug Discovery Notes

Common slug patterns observed:
- Direct company name: `luminance-1` (note the `-1` suffix -- likely collision avoidance)
- Hyphenated: `board-of-innovation` maps to `boardofinnovation`
- Lowercase with hyphens: `big-potato`, `neat-method-careers`, `vista-group`
- Sometimes unpredictable: must probe, not guess

---

## Cernio Integration Notes

### Fetcher Design

```
1. Probe slug:  GET /api/v1/widget/accounts/{slug}
               200 → slug valid, parse job list
               404 → slug invalid

2. List jobs:   Parse response.jobs array
               Each job has title, location, shortcode, published_on
               Filter by location (countryCode == "GB") and title keywords

3. Get descriptions (when needed for evaluation):
               GET /api/v1/widget/accounts/{slug}?details=true
               Parse description field from each matching job

4. Construct URLs:
               View: https://apply.workable.com/j/{shortcode}
               Apply: https://apply.workable.com/j/{shortcode}/apply
```

### Mapping to Cernio Schema

| Workable field | Cernio `jobs` column | Notes |
|---------------|---------------------|-------|
| `title` | `title` | Direct |
| `url` | `url` | Use as UNIQUE key |
| `locations[0].city + country` | `location` | Concatenate: `"London, United Kingdom"` |
| `telecommuting` | `remote_policy` | `true` → `"remote"`, `false` → check for hybrid signals in description |
| `published_on` | `posted_date` | Already ISO format |
| `description` | `raw_description` | HTML string, needs `?details=true` |
| `shortcode` | (not stored separately) | Embedded in URL |
