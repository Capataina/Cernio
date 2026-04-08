# Ashby API Reference

> How to fetch and parse job listings from companies using Ashby as their ATS.

---

## Endpoints

### List all postings (the only public endpoint)

```
GET https://api.ashbyhq.com/posting-api/job-board/{slug}
```

Returns a JSON object containing every published job posting for the company. No authentication required.

**Query parameters:**

| Parameter | Type | Default | Effect |
|-----------|------|---------|--------|
| `includeCompensation` | boolean | `false` | When `true`, adds a `compensation` object to each job |

No other query parameters are documented. There is no search, filtering, or pagination support -- the endpoint returns everything in a single response.

**Pagination:** None. The API returns all published postings in one response. This is fine for Cernio's use case -- even large companies rarely exceed a few hundred postings, and the response is a single JSON array.

---

## Response shape

### Top-level structure

```json
{
  "jobs": [
    { ... },
    { ... }
  ]
}
```

The top-level object has a single key `jobs` containing an array of job objects.

**Note on `apiVersion`:** The Ashby developer docs mention an `apiVersion` field at the top level, but real responses from ClearBank, Sentry, and Cohere do not include it. Treat it as optional/absent.

### Job object -- complete field inventory

Every field observed across ClearBank, Sentry, and Cohere boards:

```json
{
  "id": "667636eb-20eb-4542-9493-9bb665aea833",
  "title": "Senior Account Executive - Europe North",
  "department": "ClearBank Europe",
  "team": "Sales",
  "employmentType": "FullTime",
  "location": "Amsterdam Office - Hybrid",
  "secondaryLocations": [
    {
      "location": "Dubai",
      "address": {
        "postalAddress": {
          "addressCountry": "United Arab Emirates",
          "addressLocality": "Dubai"
        }
      }
    }
  ],
  "publishedAt": "2025-12-27T10:47:13.963+00:00",
  "isListed": true,
  "isRemote": true,
  "workplaceType": "Hybrid",
  "shouldDisplayCompensationOnJobPostings": true,
  "address": {
    "postalAddress": {
      "addressRegion": "North Holland",
      "addressCountry": "Netherlands",
      "addressLocality": "Amsterdam"
    }
  },
  "jobUrl": "https://jobs.ashbyhq.com/clearbank/667636eb-...",
  "applyUrl": "https://jobs.ashbyhq.com/clearbank/667636eb-.../application",
  "descriptionHtml": "<div>Full HTML job description...</div>",
  "descriptionPlain": "Full plain text job description...",
  "compensation": {
    "compensationTierSummary": "$220K – $280K • Offers Equity",
    "scrapeableCompensationSalarySummary": "$220K - $280K",
    "compensationTiers": [
      {
        "id": "db1fbc03-...",
        "tierSummary": "$220K – $280K • Offers Equity",
        "title": "Zone A",
        "additionalInformation": null,
        "components": [
          {
            "id": "1ee60008-...",
            "summary": "$220K – $280K",
            "compensationType": "Salary",
            "interval": "1 YEAR",
            "currencyCode": "USD",
            "minValue": 220000,
            "maxValue": 280000
          },
          {
            "id": "da30c6ae-...",
            "summary": "Offers Equity",
            "compensationType": "EquityPercentage",
            "interval": "NONE",
            "currencyCode": null,
            "minValue": null,
            "maxValue": null
          }
        ]
      }
    ],
    "summaryComponents": [
      {
        "compensationType": "Salary",
        "interval": "1 YEAR",
        "currencyCode": "USD",
        "minValue": 220000,
        "maxValue": 280000
      }
    ]
  }
}
```

### Field reference

| Field | Type | Notes |
|-------|------|-------|
| `id` | UUID string | Unique posting identifier. Used in jobUrl and applyUrl. |
| `title` | string | Job title. No structured seniority -- companies write freetext. |
| `department` | string | Organisational department. Freetext. |
| `team` | string | Sub-department / team name. Freetext. |
| `employmentType` | enum string | `"FullTime"`, `"PartTime"`, `"Intern"`, `"Contract"`, `"Temporary"` |
| `location` | string | Primary location as freetext. See location section below. |
| `secondaryLocations` | array | Additional locations. Each element has `location` (string) and `address` (object). Often empty `[]`. |
| `publishedAt` | ISO 8601 datetime | Last publication timestamp with timezone offset. |
| `isListed` | boolean | Whether the posting is visible on the public job board. Always `true` in practice (unlisted postings are not returned). |
| `isRemote` | boolean or null | `true`, `false`, or `null`. Can be `null` even for hybrid roles. |
| `workplaceType` | enum string or null | `"OnSite"`, `"Remote"`, `"Hybrid"`, or `null`. |
| `shouldDisplayCompensationOnJobPostings` | boolean | Company-level flag. Only present when `includeCompensation=true`. |
| `address` | object | Structured location. See location section. |
| `jobUrl` | URL string | `https://jobs.ashbyhq.com/{slug}/{id}` -- the public posting page. |
| `applyUrl` | URL string | `https://jobs.ashbyhq.com/{slug}/{id}/application` -- direct application link. |
| `descriptionHtml` | string | Full job description in HTML. Always present. |
| `descriptionPlain` | string | Full job description in plain text. Always present. |
| `compensation` | object or absent | Only present when `includeCompensation=true` is passed. See compensation section. |

---

## Key difference from Lever

Ashby returns the **full job description** (both HTML and plain text) in the list endpoint. There is no need for a separate detail fetch per job. This is a significant advantage -- one HTTP request gets everything.

Lever requires a second request (`GET /v0/postings/{slug}/{id}`) to get the description body.

---

## Location representation

Location in Ashby is represented at three levels, all on the same job object:

### 1. `location` (string) -- primary location label

Freetext string set by the company. Observed values:

| What they write | What it means |
|----------------|---------------|
| `"London Office - Hybrid"` | London office, hybrid policy |
| `"San Francisco, California"` | City + state |
| `"Vienna, Austria"` | City + country |
| `"Korea"` | Country-level, no city |
| `"Riyadh"` | City only |
| `"Canada"` | Country-level |
| `"Amsterdam Office - Hybrid"` | Office label with policy |

**Reliability:** More structured than Lever's `categories.location` but still freetext. Companies embed workplace policy in the string ("- Hybrid"), use inconsistent formats, and sometimes use just a country name.

### 2. `address.postalAddress` (object) -- structured location

```json
{
  "postalAddress": {
    "addressLocality": "London - Hybrid",
    "addressRegion": "London",
    "addressCountry": "United Kingdom"
  }
}
```

All three fields are individually optional. Observed variations:

| Scenario | addressLocality | addressRegion | addressCountry |
|----------|----------------|---------------|----------------|
| Full | `"Amsterdam"` | `"North Holland"` | `"Netherlands"` |
| No region | `"Seoul"` | absent | `"South Korea"` |
| Country only | absent | absent | `"Canada"` |
| Locality includes policy | `"London - Hybrid"` | `"London"` | `"United Kingdom"` |
| Region == country | `"Vienna"` | `"Austria"` | `"Austria"` |

**Key quirk:** `addressLocality` sometimes includes the workplace policy appended (e.g. `"London - Hybrid"`). Don't treat it as a clean city name without stripping suffixes.

**Can the entire `address` be null/missing?** Not observed in any response. Even country-level locations have an `address` object with at least `addressCountry`.

### 3. `secondaryLocations` (array) -- additional locations

```json
"secondaryLocations": [
  {
    "location": "Dubai",
    "address": {
      "postalAddress": {
        "addressCountry": "United Arab Emirates",
        "addressLocality": "Dubai"
      }
    }
  },
  {
    "location": "United States",
    "address": {
      "postalAddress": {
        "addressRegion": "California",
        "addressCountry": "United States",
        "addressLocality": "San Francisco"
      }
    }
  }
]
```

- Usually empty `[]` -- most jobs have a single location.
- When populated, each element mirrors the primary location structure: a freetext `location` string and an `address` object.
- Represents genuine multi-location postings (e.g. "Riyadh or Dubai").

### 4. `isRemote` and `workplaceType` -- remote policy

| `isRemote` | `workplaceType` | Meaning |
|------------|----------------|---------|
| `true` | `"Remote"` | Fully remote |
| `true` | `"Hybrid"` | Hybrid (common -- Ashby marks many hybrid roles as isRemote=true) |
| `null` | `null` | Not specified. Check the description. |
| `false` | `"OnSite"` | In-office only |

**Quirk:** `isRemote: true` does **not** mean fully remote. Many hybrid roles have `isRemote: true` with `workplaceType: "Hybrid"`. Use `workplaceType` as the primary signal, falling back to `isRemote` only when `workplaceType` is null.

### Location triage rule for Cernio

Same principle as Lever: keep anything that could plausibly include London or UK. Match against:
- `address.postalAddress.addressCountry` containing "United Kingdom" or "UK"
- `address.postalAddress.addressLocality` containing "London"
- `location` string containing "London", "UK", "United Kingdom", "Remote", "EMEA", "Europe"
- `workplaceType` == "Remote" (might accept UK applicants -- check description)
- Any `secondaryLocations` entry matching the above

Only drop locations that are explicitly and exclusively non-UK with no remote mention.

---

## Compensation

Only present when `?includeCompensation=true` is passed. Many companies set `shouldDisplayCompensationOnJobPostings: false`, in which case the `compensation` object exists but is empty:

```json
"compensation": {
  "compensationTierSummary": null,
  "scrapeableCompensationSalarySummary": null,
  "compensationTiers": [],
  "summaryComponents": []
}
```

When populated (e.g. Sentry):

| Field | Type | Notes |
|-------|------|-------|
| `compensationTierSummary` | string or null | Human-readable summary: `"$220K – $280K • Offers Equity"` |
| `scrapeableCompensationSalarySummary` | string or null | Machine-readable salary range: `"$220K - $280K"`. Can be null for hourly roles. |
| `compensationTiers[]` | array | Tiered compensation (e.g. by location zone). Each tier has `title`, `tierSummary`, and `components[]`. |
| `compensationTiers[].components[]` | array | Individual compensation elements. |
| `summaryComponents[]` | array | Flat aggregate of all components across tiers. |

**Component fields:**

| Field | Values |
|-------|--------|
| `compensationType` | `"Salary"`, `"Bonus"`, `"EquityPercentage"` |
| `interval` | `"1 YEAR"`, `"1 HOUR"`, `"NONE"` |
| `currencyCode` | ISO currency code (`"USD"`, `"EUR"`, `"GBP"`) or `null` |
| `minValue` / `maxValue` | numbers or `null` |

**For Cernio:** Always pass `includeCompensation=true`. The cost is negligible (same endpoint, slightly larger response). Parse `compensationTierSummary` for display and `summaryComponents` for structured comparison.

---

## Error handling

| Scenario | HTTP Status | Response |
|----------|-------------|----------|
| Valid slug, has postings | 200 | `{"jobs": [...]}` |
| Valid slug, no postings | 200 | `{"jobs": []}` (presumed -- not tested with a company that has zero postings) |
| Invalid/nonexistent slug | 404 | Empty body |

**For slug verification:** A 200 response confirms the slug is valid. A 404 means it does not exist on Ashby. This is the same probe pattern used for Lever and Greenhouse.

---

## Rate limiting

No documented rate limits for the public posting API. Same guidance as Lever: be reasonable, use a 100ms delay between requests for bulk operations.

---

## URL formats

| Purpose | Pattern |
|---------|---------|
| Job board API | `https://api.ashbyhq.com/posting-api/job-board/{slug}` |
| Public job page | `https://jobs.ashbyhq.com/{slug}/{job_id}` |
| Application page | `https://jobs.ashbyhq.com/{slug}/{job_id}/application` |
| Hosted careers page | `https://jobs.ashbyhq.com/{slug}` |

**Slug discovery:** The slug is the final path segment of the company's Ashby careers page. Usually the company name lowercased: `clearbank`, `sentry`, `cohere`. Confirmed during `populate-db`, so by search time the correct slug is already in the database.

---

## Common slug patterns

Ashby slugs follow the same conventions as Lever:
- Company name lowercased: `clearbank`, `sentry`, `cohere`
- Hyphenated for multi-word names: `tldr-tech` (though `tldr.tech` also works via redirect)
- Sometimes a short brand name rather than the full legal entity

If the obvious slug returns 404, try:
- The domain name without TLD
- Hyphenated variant
- Common abbreviations

---

## Implementation notes for the Ashby fetcher

1. **Single request per company.** Unlike Lever, there is no need for a detail fetch. One GET to the job board endpoint returns all postings with full descriptions.

2. **Always pass `includeCompensation=true`.** Free data, no extra requests.

3. **Parse `workplaceType` over `isRemote`.** The `isRemote` field is unreliable -- many hybrid roles are marked `isRemote: true`.

4. **Strip policy text from `addressLocality`.** Values like `"London - Hybrid"` need cleaning before use as a location string.

5. **Check `secondaryLocations` for UK matches.** A job with primary location "Riyadh" might have "London" as a secondary location.

6. **Use `descriptionPlain` for evaluation, `descriptionHtml` for display.** Both are always present.

7. **The `id` is a UUID, not a slug.** Job URLs use the full UUID: `https://jobs.ashbyhq.com/{slug}/{uuid}`.
