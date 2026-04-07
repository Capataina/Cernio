# Lever API Reference

> How to fetch and parse job listings from companies using Lever as their ATS.

---

## Endpoints

### List all postings

```
GET https://api.lever.co/v0/postings/{slug}
```

Returns a JSON array of every open position at the company. No authentication required.

**Pagination:** Supports `?limit=N&skip=N` parameters. Default returns all postings for most companies (under 1000). If a company has more, paginate with limit=100.

**Response shape:**

```json
[
  {
    "id": "f542541d-3427-41fd-b7da-bf272bd8b755",
    "text": "Backend Engineer - Personalization",
    "categories": {
      "commitment": "Permanent",
      "department": "Engineering",
      "location": "London / Stockholm",
      "team": "Personalization"
    },
    "hostedUrl": "https://jobs.lever.co/spotify/f542541d-...",
    "applyUrl": "https://jobs.lever.co/spotify/f542541d-.../apply",
    "createdAt": 1715000000000,
    "workplaceType": "unspecified"
  }
]
```

**Key fields for triage:**

| Field | Path | Notes |
|-------|------|-------|
| Title | `text` | The job title. No seniority structure — companies write whatever they want. |
| Location | `categories.location` | Freetext. Could be "London", "London / Stockholm", "London, UK", "United Kingdom", "EMEA", "Remote", or anything else. Companies fill this in themselves. |
| Department | `categories.department` | Freetext. Usually "Engineering", "Data and Analytics", "Product", etc. but not standardised. |
| Team | `categories.team` | Freetext. Sub-department like "Platform", "Personalization", "Music". |
| Commitment | `categories.commitment` | "Permanent", "Short Term", "Full Time Contractor", "N/A", etc. |
| Workplace type | `workplaceType` | "unspecified", "remote", "onsite", "hybrid". Often "unspecified" even for remote roles. |
| Created date | `createdAt` | Unix timestamp in milliseconds. |
| ID | `id` | UUID. Used to fetch the full posting. |

### Fetch a single posting's full details

```
GET https://api.lever.co/v0/postings/{slug}/{id}
```

Returns the complete posting with full description text.

**Response shape (additional fields beyond list):**

```json
{
  "id": "f542541d-...",
  "text": "Backend Engineer - Personalization",
  "description": "<div>HTML description...</div>",
  "descriptionPlain": "Plain text description...",
  "lists": [
    {
      "text": "What You'll Do",
      "content": "<li>Own the backend systems...</li>"
    },
    {
      "text": "Who You Are",
      "content": "<li>Experienced in building...</li>"
    }
  ],
  "additional": "Additional plain text...",
  "additionalPlain": "Additional plain text..."
}
```

**Key fields for evaluation:**

| Field | Path | Notes |
|-------|------|-------|
| Description (HTML) | `description` | The main job description body. HTML formatted. |
| Description (plain) | `descriptionPlain` | Same content, plain text. Easier to read. |
| Requirement lists | `lists[]` | Array of named sections ("What You'll Do", "Who You Are", "Requirements", etc.). Each has `text` (section name) and `content` (HTML). |
| Additional info | `additionalPlain` | Extra text that doesn't fit the main description. Sometimes contains salary, benefits, or legal notices. |

---

## Location quirks on Lever

Location is the most unreliable field. Common patterns seen across companies:

| What they write | What it means |
|----------------|---------------|
| `London` | London office |
| `London, UK` | London office |
| `London, United Kingdom` | London office |
| `London / Stockholm` | Either office, probably with flexibility |
| `UK` | UK-based, could be any office |
| `United Kingdom` | Same as UK |
| `EMEA` | Could be remote from UK, but could mean specific EMEA offices |
| `Europe` | Might include UK remote, might not |
| `Remote` | Remote, but which countries? Check the description. |
| `Home Mix` | Spotify's internal term for remote/hybrid |
| Empty/null | Location not specified — don't drop these, check the description |

**Rule:** When triaging, keep anything that could plausibly include London or UK. Only drop locations that are explicitly and exclusively non-UK with no remote mention. When in doubt, keep it.

---

## Rate limiting

Lever's public API has no documented rate limits for read access. Be reasonable — don't hammer the same endpoint in a tight loop. A 100ms delay between requests is plenty for bulk operations.

---

## Common slug patterns

Lever slugs are usually the company name lowercased. Common patterns:
- `spotify` → Spotify
- `twitch` → Twitch
- `netlify` → Netlify

Some companies use variations. If the obvious slug returns 404, try:
- The domain name without TLD: `stripe` for stripe.com
- Hyphenated: `jane-street` for Jane Street
- Abbreviated: common abbreviations of the company name

The slug is confirmed during the `populate-db` phase, so by the time `search-jobs` runs, the correct slug is already in the database.
