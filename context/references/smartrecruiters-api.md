# SmartRecruiters Posting API Reference

> Last verified: 2026-04-07 against Wise (369 jobs), Visa (915 jobs).

---

## Endpoints

| Endpoint | Purpose | Returns description? |
|----------|---------|---------------------|
| `GET /v1/companies/{slug}/postings` | List postings (summary) | No |
| `GET /v1/companies/{slug}/postings/{id}` | Single posting detail | Yes (full HTML) |

Base URL: `https://api.smartrecruiters.com`

No authentication required for the Posting API. This is the public-facing API for career sites. The authenticated Customer API (`/configuration/company`) requires an API key and is separate.

---

## Critical: Slug Verification

**SmartRecruiters returns HTTP 200 with `{"totalFound": 0, "content": []}` for ANY slug, including completely fake ones.** You cannot use HTTP status codes to verify a slug exists.

**The only reliable verification is `totalFound > 0`.** A company with zero current openings is indistinguishable from a non-existent slug via this API.

This is a known gotcha documented in `context/notes/populate-db-lessons.md`.

---

## Pagination

SmartRecruiters supports proper pagination via `limit` and `offset` query parameters.

| Parameter | Type | Default | Max | Notes |
|-----------|------|---------|-----|-------|
| `limit` | integer | 100 | 100 | Number of results per page |
| `offset` | integer | 0 | -- | Starting position in result set |

The response envelope always includes:

```json
{
  "offset": 0,
  "limit": 100,
  "totalFound": 369,
  "content": [ ... ]
}
```

**To fetch all jobs:** Loop with `offset += limit` until `offset >= totalFound`.

Example: Wise has 369 jobs. First request returns 100 (offset 0), second returns 100 (offset 100), third returns 100 (offset 200), fourth returns 69 (offset 300).

---

## Filtering

The list endpoint supports server-side filtering via query parameters:

| Parameter | Type | Example | Notes |
|-----------|------|---------|-------|
| `q` | string | `?q=engineer` | Free-text keyword search |
| `country` | string | `?country=gb` | ISO 3166-1 alpha-2, lowercase |
| `region` | string | `?region=England` | |
| `city` | string | `?city=London` | |
| `department` | string | `?department=Engineering` | |
| `language` | string | `?language=en` | Posting language |

**Verified:** `?country=gb` on Wise reduces results from 369 to 138 (London + UK jobs only). Filtering happens server-side, so responses are smaller and faster.

**Recommendation for Cernio:** Always pass `?country=gb` for UK-only searches to reduce response size and avoid processing irrelevant international roles. Can combine with `?q=engineer` for further narrowing.

---

## Response Structure

### List Response Envelope

```json
{
  "offset": 0,
  "limit": 100,
  "totalFound": 369,
  "content": [ ... posting objects ... ]
}
```

### Posting Object (List Endpoint)

```json
{
  "id": "744000119316797",
  "name": "Engineering Lead II - Database - Platform",
  "uuid": "062ba899-c813-4bb1-909d-dc3069557da0",
  "jobAdId": "258b924a-5aac-4a4f-83fa-ee0af3f0b30a",
  "defaultJobAd": true,
  "refNumber": "R8557",
  "company": {
    "identifier": "Wise",
    "name": "Wise"
  },
  "releasedDate": "2026-04-08T09:20:59.461Z",
  "location": {
    "city": "London",
    "region": "England",
    "country": "gb",
    "address": "Worship Street",
    "postalCode": "EC2A 4JE",
    "remote": false,
    "hybrid": true,
    "hybridDescription": "Hybrid - 3 days a week in office",
    "latitude": "51.5220563",
    "longitude": "-0.0830745",
    "fullLocation": "London, England, United Kingdom"
  },
  "industry": {
    "id": "financial_services",
    "label": "Financial Services"
  },
  "department": {
    "id": "12345",
    "label": "Engineering"
  },
  "function": {
    "id": "engineering",
    "label": "Engineering"
  },
  "typeOfEmployment": {
    "id": "permanent",
    "label": "Full-time"
  },
  "experienceLevel": {
    "id": "mid_senior_level",
    "label": "Mid-Senior Level"
  },
  "customField": [ ... ],
  "visibility": "PUBLIC",
  "ref": "https://api.smartrecruiters.com/v1/companies/wise/postings/744000119316797",
  "creator": {
    "name": "Recruiter Name"
  },
  "language": {
    "code": "en",
    "label": "English",
    "labelNative": "English (US)"
  }
}
```

### Posting Object (Detail Endpoint)

The detail endpoint (`/postings/{id}`) returns the same fields as the list, plus the full job description as rendered HTML. The WebFetch of the detail endpoint for Wise's Engineering Lead II role returned a complete description with responsibilities, requirements, compensation, and company context.

The description is returned as the body content of the posting (not a separate JSON field name -- the detail endpoint appears to serve a richer representation that includes the full description text).

---

## Field Reference

| Field | Type | Always present? | Notes |
|-------|------|----------------|-------|
| `id` | string | Yes | Numeric string, unique posting ID |
| `name` | string | Yes | Job title |
| `uuid` | string | Yes | UUID v4 |
| `jobAdId` | string | Yes | UUID of the specific job ad variant |
| `defaultJobAd` | bool | Yes | Whether this is the default ad for the role |
| `refNumber` | string | Yes | Internal reference number (e.g. `"R8557"`) |
| `company` | object | Yes | `{identifier, name}` |
| `releasedDate` | string | Yes | ISO 8601 datetime with timezone: `"2026-04-08T09:20:59.461Z"` |
| `location` | object | Yes | Rich location object (see below) |
| `industry` | object | Yes | `{id, label}` -- company industry |
| `department` | object | Sometimes | `{id, label}` -- can be empty `{}` |
| `function` | object | Yes | `{id, label}` -- job function category |
| `typeOfEmployment` | object | Yes | `{id, label}` -- e.g. `permanent`/`Full-time` |
| `experienceLevel` | object | Yes | `{id, label}` -- e.g. `mid_senior_level`, `entry_level`, `director` |
| `customField` | array | Yes | Company-specific custom fields (variable) |
| `visibility` | string | Yes | Always `"PUBLIC"` on the public API |
| `ref` | string | Yes | Self-link URL to this posting |
| `creator` | object | Sometimes | `{name}` -- recruiter name, not always present |
| `language` | object | Yes | `{code, label, labelNative}` |

---

## Location Object (Detailed)

```json
{
  "city": "London",
  "region": "England",
  "country": "gb",
  "address": "Worship Square, 65 Clifton Street",
  "postalCode": "EC2A 4JE",
  "remote": false,
  "hybrid": true,
  "hybridDescription": "Hybrid - 3 days a week in office",
  "latitude": "51.5220563",
  "longitude": "-0.0830745",
  "fullLocation": "London, England, United Kingdom"
}
```

| Field | Type | Always present? | Notes |
|-------|------|----------------|-------|
| `city` | string | Yes | City name |
| `region` | string | Sometimes | State/region -- can be missing entirely (not just empty) |
| `country` | string | Yes | ISO 3166-1 alpha-2, **lowercase** (`"gb"`, `"us"`, `"ee"`) |
| `address` | string | Sometimes | Street address -- not always present |
| `postalCode` | string | Sometimes | Postal/zip code -- not always present |
| `remote` | bool | Yes | `true` if fully remote |
| `hybrid` | bool | Yes | `true` if hybrid working |
| `hybridDescription` | string | Sometimes | Only present when `hybrid: true`; human-readable (e.g. `"Hybrid - 3 days a week in office"`) |
| `latitude` | string | Sometimes | GPS latitude as string -- not always present |
| `longitude` | string | Sometimes | GPS longitude as string -- not always present |
| `fullLocation` | string | Yes | Pre-formatted: `"London, England, United Kingdom"` or `"Tallinn, , Estonia"` (note double comma when region missing) |

### Key Observations

1. **Single location per posting.** Unlike Workable (which has a `locations` array), SmartRecruiters has a single `location` object per posting. A job posted in multiple locations appears as multiple separate postings (each with its own `id`).

2. **Country is lowercase alpha-2.** Unlike Workable (uppercase `"GB"`), SmartRecruiters uses lowercase (`"gb"`). Normalise on parsing.

3. **`region` can be entirely absent.** Not null, not empty string -- the key itself is missing from the JSON. The `fullLocation` field shows this as a double comma: `"Tallinn, , Estonia"`.

4. **Remote policy is explicit.** Two booleans: `remote` (fully remote) and `hybrid` (hybrid). Both `false` means on-site. When `hybrid: true`, there may be a `hybridDescription` explaining the split.

5. **Location is never null/missing.** The `location` object is always present (never null). At minimum it contains `city`, `country`, `remote`, `hybrid`, and `fullLocation`.

### Location Format Examples

| Scenario | `city` | `region` | `country` | `remote` | `hybrid` | `fullLocation` |
|----------|--------|----------|-----------|----------|----------|----------------|
| London hybrid | `"London"` | `"England"` | `"gb"` | `false` | `true` | `"London, England, United Kingdom"` |
| London on-site | `"London"` | (absent) | `"gb"` | `false` | `false` | `"London, , United Kingdom"` |
| Tallinn on-site | `"Tallinn"` | (absent) | `"ee"` | `false` | `false` | `"Tallinn, , Estonia"` |
| Doha on-site | `"Doha"` | `"QATAR"` | `"qa"` | `false` | `false` | `"Doha, QATAR, Qatar"` |
| Foster City hybrid | `"Foster City"` | `"CA"` | `"us"` | `false` | `true` | `"Foster City, CA, United States"` |

---

## Experience Level Values

Observed `experienceLevel.id` values from real data:

| `id` | `label` | Notes |
|------|---------|-------|
| `entry_level` | `"Entry Level"` | Target for Cernio |
| `mid_senior_level` | `"Mid-Senior Level"` | Most common in Wise/Visa data |
| `director` | `"Director"` | |
| `associate` | `"Associate"` | |
| `executive` | `"Executive"` | |

---

## Custom Fields

The `customField` array contains company-specific metadata. Structure is consistent but content varies wildly between companies.

```json
{
  "fieldId": "65d90823f2e6436af25814c2",
  "fieldLabel": "Job Ad Salary Range",
  "valueId": "...",
  "valueLabel": "125000 - 153000 GBP Annual"
}
```

Notable fields observed in Wise data:
- `Job Ad Salary Range` -- compensation with currency and period
- `WD Target Hire Date` -- target hire date
- `Job Family` / `Job Family Group` -- organisational taxonomy
- `Squad` / `Tribe` -- team structure
- `Management Level` -- seniority number
- `Worker Type` / `Worker Sub Type` -- employment classification
- `Cost Center` -- internal cost allocation

**These are not standardised.** Different companies use different custom fields. Do not rely on any specific custom field existing across companies. Useful for enrichment when present, but not for filtering logic.

---

## URL Patterns

| Purpose | Pattern | Example |
|---------|---------|---------|
| List postings | `https://api.smartrecruiters.com/v1/companies/{slug}/postings` | `...companies/wise/postings` |
| Single posting | `https://api.smartrecruiters.com/v1/companies/{slug}/postings/{id}` | `...companies/wise/postings/744000119316797` |
| Career page (human) | `https://careers.smartrecruiters.com/{Slug}/` | `https://careers.smartrecruiters.com/Wise/` |
| Apply link | `https://jobs.smartrecruiters.com/{Slug}/{id}` | `https://jobs.smartrecruiters.com/Wise/744000119316797` |

**Slug case sensitivity:** The API slug appears case-insensitive (`wise` and `Wise` both work). The human-facing career page uses title case.

---

## Cernio Integration Notes

### Fetcher Design

```
1. Probe slug:   GET /v1/companies/{slug}/postings?limit=1
                 Check totalFound > 0 (NOT HTTP status -- always 200)
                 totalFound > 0 → slug valid
                 totalFound == 0 → slug invalid OR company has no openings

2. List jobs:    GET /v1/companies/{slug}/postings?country=gb&limit=100
                 Loop with offset += 100 until offset >= totalFound
                 Filter by experienceLevel, function, name keywords

3. Get detail:   GET /v1/companies/{slug}/postings/{id}
                 Returns full description for evaluation

4. Construct URLs:
                 View: https://jobs.smartrecruiters.com/{Slug}/{id}
                 API detail: ref field in listing response
```

### Mapping to Cernio Schema

| SmartRecruiters field | Cernio `jobs` column | Notes |
|----------------------|---------------------|-------|
| `name` | `title` | Direct |
| Constructed URL | `url` | `https://jobs.smartrecruiters.com/{Slug}/{id}` |
| `location.city + location.country` | `location` | Use `fullLocation` or build from parts |
| `location.remote` / `location.hybrid` | `remote_policy` | `remote: true` → `"remote"`, `hybrid: true` → `"hybrid"`, both false → `"on-site"` |
| `releasedDate` | `posted_date` | ISO 8601, parse datetime |
| Detail endpoint body | `raw_description` | Fetch separately per job |
| `id` | (not stored separately) | Embedded in URL |

### Advantages Over Other Providers

1. **Server-side filtering** -- reduce response size with `?country=gb` before parsing
2. **Proper pagination** -- handle large companies (Visa: 915 jobs) without OOM
3. **Explicit remote/hybrid booleans** -- no need to guess from description
4. **Experience level** -- can filter `entry_level` server-side or post-fetch
5. **Salary in custom fields** -- some companies expose compensation
