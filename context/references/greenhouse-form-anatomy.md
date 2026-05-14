# Greenhouse Form Anatomy — Field Survey

**Purpose:** catalogue real Greenhouse application forms across companies and URL patterns to inform the autofill rewrite in `src/autofill/greenhouse.rs`.

**Method:** curl the listing page, extract embedded JSON or DOM markup, document field shape. One company at a time, sequential, no parallel fetches.

**Why this file exists:** prior survey (2026-05-14) had 9 of 10 agents finish but a memory crash lost all findings. This file is the durable replacement — it's appended to after every inspection so a crash never loses more than the in-flight company.

---

## Survey Targets

| # | Tier | Company | Job ID | URL | Result |
|---|---|---|---|---|---|
| 1 | SS | Cloudflare | 2301 → 2299 | legacy `boards.greenhouse.io` 301s to `job-boards.greenhouse.io` | ✓ schema captured |
| 2 | S | B2C2 | 2260 | `job-boards.eu.greenhouse.io/b2c2/...` | ✓ schema captured |
| 3 | A | GSA Capital | 2383 | `www.gsacapital.com/careers/gh/?gh_jid=X` (custom-domain wrapper → real GH embed) | ✓ schema captured |
| 4 | B | Monzo | 2816 | `job-boards.greenhouse.io/monzo/...` (first EEOC + demographic_questions sample) | ✓ schema captured |
| 5 | C | BlueCrest | 2999 | `job-boards.greenhouse.io/bluecrestcapitalmanagement/...` | ✓ schema captured |
| 6a | SS | Hudson River Trading | 2447 | `www.hudsonrivertrading.com/careers/job/?gh_jid=X` | ✗ **off-platform** — see data-quality finding |
| 6b | SS | Squarepoint Capital | 2617 | `www.squarepoint-capital.com/...` → `embed/job_app?for=squarepointcapital` | ✓ schema captured (substitute for HRT) |
| 7 | B | Anthropic | 2972 | `job-boards.greenhouse.io/anthropic/...` (essay-heavy: "Why Anthropic?" + length-hint descriptions) | ✓ schema captured |
| 8 | A | XTX Markets | 3334 | `job-boards.greenhouse.io/xtxmarketstechnologies/...` (essay-heavy: 2 required essays) | ✓ schema captured |

**URL pattern coverage achieved:**
- `boards.greenhouse.io/{slug}/jobs/{id}` — legacy subdomain (now 301s to job-boards.)
- `job-boards.greenhouse.io/{slug}/jobs/{id}` — current Remix (Cloudflare, Monzo, BlueCrest, Anthropic, XTX)
- `job-boards.eu.greenhouse.io/{slug}/jobs/{id}` — EU Remix variant (B2C2)
- `boards.greenhouse.io/embed/job_app?for=X&token=Y` — embedded form, fetched and injected by custom-domain wrapper (GSA, Squarepoint)
- `{company}.com/...?gh_jid=X` — custom-domain wrapper; can be either genuine GH embed (GSA, Squarepoint) OR off-platform with vestigial gh_jid (HRT)

---

## Per-Company Findings

### 1. Cloudflare — Security Engineer Intern (Summer 2026) [SS]

**Source URL:** `https://boards.greenhouse.io/cloudflare/jobs/7582169`
**Redirects to:** `https://job-boards.greenhouse.io/cloudflare/jobs/7582169` (301 — legacy `boards.greenhouse.io` is being consolidated into `job-boards.greenhouse.io`)
**Originally surveyed job (DB job 2301, job_id 7296923):** DEAD — redirects to generic `/careers/jobs/` page. Two of three sampled Cloudflare job IDs are pulled. **Stale jobs in DB.** Worth a sweep.
**App URL kind:** standalone Remix page (not embedded)

#### Schema location

The entire form schema is **server-rendered into the HTML** in a `<script>` block:

```html
<script>window.__remixContext = { ...22.9 KB of JSON... };</script>
```

Schema path within: `state.loaderData['routes/$url_token_.jobs_.$job_post_id'].jobPost`

Other top-level fields available:
- `submitPath` — the POST endpoint (here: `https://boards.greenhouse.io/cloudflare/jobs/7582169`)
- `confirmationPath` — post-submit redirect
- `quickApply` — whether the quick-apply flow is enabled
- `seek` — sub-object for Apply-with-Seek integration

There is also `window.ENV` (1.2 KB) carrying runtime config: asset CDN URL, email-validator host, snowplow analytics endpoint, captcha enabled flag.

#### jobPost-level config

| Key | Value | Meaning |
|---|---|---|
| `enable_eeoc` | `false` | EEOC demographic block disabled for Cloudflare |
| `language` | `"en"` | Localisation key |
| `disable_captcha` | `false` (from job-board config) | reCAPTCHA active |
| `education_config` | object | Separate schema for the multi-row education widget |
| `pay_ranges` | `[]` (empty) | Per-job pay band, populated when present |

#### Question schema (universal shape)

Every entry in `jobPost.questions` has:

```jsonc
{
  "required": true|false,
  "label": "First Name",          // human-readable — STABLE KEY for autofill
  "description": "...",            // optional help text
  "fields": [                      // array because resume can have multiple input modes
    {
      "name": "first_name",        // form field name on POST
      "type": "input_text",        // discrete enum
      "values": []                 // populated only for selects
    }
  ]
}
```

**Field-type enum observed:**
- `input_text` — single-line `<input type="text">`
- `textarea` — `<textarea>`
- `input_file` — `<input type="file">`
- `multi_value_single_select` — combobox; pick one of `values[]`
- `multi_value_multi_select` — checkbox list; pick any subset of `values[]`

**For selects, each option is `{value: "623958729", label: "Yes"}`** — `value` is the option_id (unstable across companies and over time), `label` is the human text (stable). **Autofill must match by label.**

#### Cloudflare's 19 questions

| # | Req | Label | Field name | Type | Opts |
|---|---|---|---|---|---|
| 0 | ✓ | First Name | `first_name` | text | — |
| 1 | ✓ | Last Name | `last_name` | text | — |
| 2 | ✓ | Email | `email` | text | — |
| 3 | ✓ | Phone | `phone` | text | — |
| 4 | ✓ | Candidate Location | `candidate_location` | text¹ | — |
| 5 | ✓ | Resume/CV | `resume` + `resume_text` | file OR textarea | — |
| 6 |   | Cover Letter | `cover_letter` + `cover_letter_text` | file OR textarea | — |
| 7 |   | Legal Name (if different) | `question_63179973` | text | — |
| 8 |   | LinkedIn Profile | `question_63179974` | text | — |
| 9 |   | GitHub Profile URL | `question_63179975` | text | — |
| 10 | ✓ | How did you hear about this job? | `question_63179976` | single_select | 9 |
| 11 | ✓ | Do you have permission to work in the UK? | `question_63229415` | single_select | 2 |
| 12 | ✓ | Do you now or will you require immigration sponsorship? | `question_63179977` | single_select | 2 |
| 13 | ✓ | Acknowledge Cloudflare's Candidate Privacy Policy | `question_63179978[]` | multi_select | 1 |
| 14 | ✓ | Currently enrolled in a university? | `question_63179979` | single_select | 2 |
| 15 | ✓ | When do you expect to graduate? | `question_63179980` | single_select | 11 |
| 16 | ✓ | What degree are you pursuing? | `question_63179981` | single_select | 6 |
| 17 | ✓ | Residing in London or have plans to be? | `question_63179982` | single_select | 2 |
| 18 | ✓ | When could you start full-time? | `question_63179983` | single_select | 2 |

¹ Candidate Location is rendered as a combobox (typeahead) in the DOM despite being `input_text` in the schema — it has `role="combobox"` and a location-validator service. Likely text-with-autocomplete.

#### DOM rendering insights (when running via browser)

Even though we have the JSON, the DOM is what a browser-driven autofill writes into. Each select/combobox renders as:

```html
<label id="question_NNN-label" for="question_NNN">...</label>
<input id="question_NNN" role="combobox" aria-haspopup="true"
       aria-required="true" aria-labelledby="question_NNN-label"
       class="select__input" value=""/>
<button class="icon-button" aria-label="Toggle flyout">▼</button>
<!-- The actual validation gate is this hidden mirror: -->
<input required="" tabindex="-1" aria-hidden="true"
       class="remix-css-1a0ro4n-requiredInput" value=""/>
```

**The hidden `requiredInput` is the form-validation gate.** Setting `.value` on the visible combobox input does nothing for submission; the React state has to be updated so the hidden input gets the option's `value`. This is exactly the React-synthetic-event problem from `project_autofill.md`.

Checkboxes render as `<fieldset class="checkbox" id="question_NNN[]" aria-required="true">` with one `<input type="checkbox" value="OPTION_ID">` per option.

Standard fields use predictable `id`s (`first_name`, `last_name`, `email`, `phone`, `resume`, etc.) — those CAN be filled by selector if needed.

Resume + Cover Letter have a tri-mode UI: file upload (`data-testid="resume-dropbox"`), URL paste (no testid found, third pill button), text paste (`data-testid="resume-text"` switches to `<textarea id="resume_text">`). The `fields` array in the JSON reflects only the file + textarea modes.

#### Anti-bot

- reCAPTCHA enabled (`disable_captcha: false` in job board config)
- Specifically v3 (invisible, scores in the background) — no challenge widget rendered
- `window.ENV` carries the email-validator host suggesting server-side validation of email syntax/disposable detection

#### Implications

1. **Don't drive the DOM by CSS selector. Drive by the JSON.** Parse `window.__remixContext` → walk `jobPost.questions` → match by `label` against the prepared application package. The DOM is the rendering; the JSON is the schema.
2. **Two execution strategies are viable:**
   - **POST directly to `submitPath`** with the encoded form data, bypassing the React UI entirely. Cleanest but reCAPTCHA v3 token must come from somewhere (the script generates one on `grecaptcha.execute`; we'd need to call that via CDP `evaluate` before posting).
   - **CDP `Input.insertText` typing** into the visible combobox/text inputs, then click the matching `<li>` in the dropdown for selects. Slower but indistinguishable from a real user; reCAPTCHA gets its signals naturally.
3. **Match by label, not by `question_NNN` ID.** Question IDs are per-job; labels are stable enough across most company forms. Where labels differ ("Phone Number" vs "Phone" vs "Mobile"), a small label-normalisation table on the autofill side handles it.
4. **Stale-job sweep needed.** Two of three Cloudflare jobs sampled were dead. The job-pulling pattern (302 to generic /careers/jobs/) is detectable — `cernio check` could flag this and auto-archive.

---

### 2. B2C2 — Graduate Quant Developer [S]

**Source URL:** `https://job-boards.eu.greenhouse.io/b2c2/jobs/4745320101`
**App URL kind:** standalone Remix page on the EU subdomain
**Submit path:** `https://boards.eu.greenhouse.io/b2c2/jobs/4745320101` (EU twin of `boards.greenhouse.io`)

#### What's the same as Cloudflare

- `window.__remixContext` JSON in initial HTML — identical structure (`state.loaderData['routes/$url_token_.jobs_.$job_post_id'].jobPost.questions`)
- Question shape: `{required, label, description, fields:[{name, type, values}]}` — byte-for-byte the same schema contract
- Same five field types: `input_text`, `textarea`, `input_file`, `multi_value_single_select`, `multi_value_multi_select`
- Same standard field names (`first_name`, `last_name`, `email`, `phone`, `resume`, `resume_text`, `cover_letter`, `cover_letter_text`) — the autofill name-mapping for standard fields is universal
- `enable_eeoc: false` — no EEOC block

#### What differs

- **Submit-path host follows the region.** US Greenhouse posts to `boards.greenhouse.io/...`; EU Greenhouse posts to `boards.eu.greenhouse.io/...`. Inferred from URL prefix; rendering host is `job-boards.{eu.,}greenhouse.io`.
- **Question ID length differs by region.** US uses ~8-digit IDs (`63179973`); EU uses ~10-digit IDs (`7982971101`). Cosmetic but confirms the IDs are unstable across regions, never use as keys.
- **`Phone` is optional** (B2C2) where Cloudflare made it required. Required-ness is per-question, per-company config — never assume.
- **Free-text date question** (`Date of graduation (Month/Year)`) is `input_text`, not a picker. Same for Cloudflare's question_63179975 (Github URL) — Greenhouse has no `input_url` or `input_date` types in the schema, every freeform value is `input_text`.
- **`textarea` for short answers.** "Website/Github account link" is a `textarea` on B2C2 even though the answer is short. The field type is set by the recruiter on Greenhouse's admin side and has nothing to do with expected answer length.
- **Numeric-scale questions ("rate your Java expertise 1–6") are `multi_value_single_select`** with options literally `"1"`, `"2"`, ..., `"6"`. No special scale widget. The label of the option *is* the numeric string. This is the autofill discriminator — answers stored as integers won't match; we need the string form.

#### B2C2's 15 questions

| # | Req | Label | Field name | Type | Opts |
|---|---|---|---|---|---|
| 0 | ✓ | First Name | `first_name` | text | — |
| 1 | ✓ | Last Name | `last_name` | text | — |
| 2 | ✓ | Email | `email` | text | — |
| 3 |   | Phone | `phone` | text | — |
| 4 |   | Resume/CV | `resume` + `resume_text` | file OR textarea | — |
| 5 |   | Cover Letter | `cover_letter` + `cover_letter_text` | file OR textarea | — |
| 6 |   | LinkedIn Profile | `question_7982971101` | text | — |
| 7 |   | Website/Github account link | `question_7982972101` | textarea | — |
| 8 | ✓ | Eligible for employment (citizen, PR, etc.)? | `question_7982973101` | single_select | 2 |
| 9 | ✓ | Current legal right to work in UK | `question_8208361101` | single_select | 9 |
| 10 | ✓ | Date of graduation (Month/Year) | `question_7982975101` | text | — |
| 11 | ✓ | Java expertise (1–6 scale) | `question_8208361101` | single_select | 6 |
| 12 | ✓ | Python expertise (1–6 scale) | `question_8208362101` | single_select | 6 |
| 13 |   | Activities to increase knowledge beyond uni | `question_7982976101` | textarea | — |
| 14 |   | Skills/experience that align with this job | `question_7982977101` | textarea | — |

#### Implications added

5. **Generalised submit-host rule:** if rendering host starts with `job-boards.eu.`, submit host is `boards.eu.greenhouse.io`; if `job-boards.`, submit is `boards.greenhouse.io`. The submitPath is in the JSON anyway — use that directly, don't reconstruct.
6. **Schema is region-invariant.** Same JSON shape, same field-type enum, same standard-field names. The autofill code path is one for both regions.
7. **Numeric scales need string-form answers.** Storing "Java rating: 5" as int 5 won't match the option whose label is `"5"`. Coerce on autofill.

---

### 3. GSA Capital — Software Developer [A]

**Source URL (custom-domain entry point):** `https://www.gsacapital.com/careers/gh/?gh_jid=7555839002`
**Real Greenhouse embed URL:** `https://boards.greenhouse.io/embed/job_app?for=gsacapital&token=7555839002` (301s to `job-boards.greenhouse.io/embed/job_app?...`)
**Submit path:** `https://boards.greenhouse.io/embed/gsacapital/jobs/7555839002` (note: *different shape* from the GET path)

#### Embed mechanism — critical finding

`www.gsacapital.com/careers/gh/?gh_jid=7555839002` returns a soft 404, but the HTML still contains references to:

```
https://boards.greenhouse.io/embed/job_app?for=gsacapital&token=4010431002
https://boards.greenhouse.io/embed/job_app?for=gsacapital&token=7555839002
https://boards.greenhouse.io/embed/job_app?for=gsacapital&token=8077636002
```

These are the **canonical Greenhouse embed endpoints** that GSA's careers page fetches (either via iframe in older deployments or, here, via JS-driven DOM injection). **No iframes were found in the page**, confirming GSA injects the embed markup into their own DOM.

Fetching the embed URL directly returns the SAME Remix app — `window.__remixContext` is present, route key is `routes/embed.job_app`, schema shape is byte-for-byte identical to the standalone Remix page.

**Autofill implication:** if Cernio's autofill detects a custom-domain page that doesn't directly contain the application form, it should:
1. Probe the page HTML for `boards.greenhouse.io/embed/job_app?for=X&token=Y` references
2. Extract `X` (company slug) and `Y` (job token)
3. Navigate directly to that embed URL instead of the custom-domain wrapper
4. Parse `window.__remixContext` from there

This bypasses any wrapper-page bugs and gives us the same JSON schema regardless of where the form is embedded.

#### What's the same as Cloudflare/B2C2

- `window.__remixContext` JSON, same shape, same nesting
- Same five field-type enum
- Same standard field names (`first_name`, `last_name`, `email`, `phone`, `resume`, `cover_letter`)
- `enable_eeoc: false`
- Schema is server-rendered into initial HTML — no JS execution required to extract

#### What's new on GSA

- **`preferred_name` as a standard field** (not custom, has the dedicated `preferred_name` ID). Cloudflare and B2C2 didn't expose this. Greenhouse's standard-fields set is a per-job recruiter toggle.
- **Submit path is `/embed/{slug}/jobs/{id}`** not `/embed/job_app?for=...&token=...`. **The GET-page URL and POST-target URL have different URL shapes for embedded forms.** Standalone forms have GET and POST on the same path. Autofill must read `submitPath` from the JSON rather than reconstructing from the page URL.
- **Question IDs are 11-digit** (`28531208002`). With Cloudflare at 8-digit and B2C2 at 10-digit, IDs are clearly unstable across companies and over time.
- **9 questions only** — short forms are common for hedge-fund mid-level roles; the application is a CV-screening step before interview rounds, not a complete intake.

#### GSA's 9 questions

| # | Req | Label | Field name | Type | Opts |
|---|---|---|---|---|---|
| 0 | ✓ | First Name | `first_name` | text | — |
| 1 | ✓ | Last Name | `last_name` | text | — |
| 2 |   | Preferred First Name | `preferred_name` | text | — |
| 3 | ✓ | Email | `email` | text | — |
| 4 | ✓ | Phone | `phone` | text | — |
| 5 | ✓ | Resume/CV | `resume` + `resume_text` | file OR textarea | — |
| 6 |   | Cover Letter | `cover_letter` + `cover_letter_text` | file OR textarea | — |
| 7 | ✓ | How did you hear about GSA Capital? | `question_28531208002` | single_select | 12 |
| 8 | ✓ | Data Protection | `question_28531209002[]` | multi_select | 1 |

#### Implications added

8. **The "embedded vs standalone" axis is irrelevant for autofill.** Both render the same Remix app on the same `job-boards.greenhouse.io` host with the same `window.__remixContext` schema. The only difference is the URL shape. Treat them uniformly.
9. **Custom-domain wrapper pages may 404 or otherwise misbehave** (GSA's careers page returned 404 on this URL), but the underlying Greenhouse embed URL still works. Cernio should prefer the embed URL over the wrapper URL when both are available. The wrapper URL is brittle; the embed URL is canonical.
10. **`submitPath` differs from page URL for embedded forms.** Always read `submitPath` from the JSON; don't reconstruct it from `window.location`.
11. **Standard-field set is opt-in per job.** `preferred_name`, `candidate_location`, country are all standard fields but only appear when the recruiter enables them. Autofill must tolerate any subset.

---

### 4. Monzo — Data Scientist [B]

**Source URL:** `https://job-boards.greenhouse.io/monzo/jobs/5636930`
**Submit path:** `https://boards.greenhouse.io/monzo/jobs/5636930` (US Greenhouse despite Monzo being UK; Greenhouse account region is independent of job location)
**Job location:** London

#### The big finding — EEOC and demographic surveys are SEPARATE schemas

Cloudflare/B2C2/GSA all had `enable_eeoc: false`. Monzo is the first with `enable_eeoc: true`, which surfaces TWO additional question buckets beyond `jobPost.questions`:

```
jobPost.questions[]            ← main application form (universal schema)
jobPost.eeoc_sections[]        ← US-government EEOC self-identification
jobPost.demographic_questions  ← company-defined diversity survey (DIFFERENT schema)
```

Three distinct shapes, three distinct code paths needed in the autofill.

#### Schema A — main `questions` (same as Cloudflare/B2C2/GSA)

```jsonc
{ "required": bool, "label": str, "description": str?, "fields": [{"name": str, "type": str, "values": [{"value": str, "label": str}]}] }
```

Field types: `input_text` | `textarea` | `input_file` | `multi_value_single_select` | `multi_value_multi_select`.

#### Schema B — `eeoc_sections[]` (US EEOC reporting)

```jsonc
{
  "description": "<html for legal preamble>",
  "questions": [
    { "required": false, "label": "Gender", "fields": [{"name": "gender", "type": "multi_value_single_select", "values": [...]}] }
  ]
}
```

**Shape of each question matches Schema A** — autofill can reuse the same parser. Only difference: nested under sections, each section has its own description HTML.

**Standard EEOC field names (fixed Greenhouse vocabulary):**
- `gender` — Male / Female / Decline To Self Identify
- `veteran_status` — I am not a protected veteran / I identify as one or more / I don't wish to answer
- `disability_status` — Yes / No / I do not want to answer
- (race/ethnicity sometimes — not present on Monzo's job)

Monzo had 4 sections; section 3 contained zero questions — it was just the Paperwork Reduction Act legal preamble.

**All EEOC questions have `required: false`.** Decline-to-answer is always permitted.

#### Schema C — `demographic_questions` (company-defined)

```jsonc
{
  "title": "👤 Identity survey",
  "description": "<html>...</html>",
  "questions": [
    {
      "id": 627,
      "name": "How would you describe your gender identity?",
      "required": false,
      "answer_type": { "id": null, "key": "MULTI_SELECT" },
      "answer_options": [
        { "id": 3315, "name": "Prefer not to say", "free_form": false, "decline_to_answer": false },
        { "id": 3312, "name": "Non-binary", "free_form": false, "decline_to_answer": false },
        { "id": 3315, "name": "I identify in another way (please share)", "free_form": true, "decline_to_answer": false }
      ]
    }
  ]
}
```

**Different vocabulary from Schemas A/B:**
- `name` (not `label`)
- `id` numeric (not the `question_NNN` string)
- `answer_type: { key: "MULTI_SELECT" }` (not `type: "multi_value_single_select"`)
- `answer_options[]` (not `fields[0].values[]`)
- Option-level `free_form: bool` — if true, picking the option opens a text input ("I identify in another way (please share)")
- Option-level `decline_to_answer: bool` — **misleading: not always set on the actual decline option.** Monzo's "Prefer not to say" had `decline_to_answer: false` despite being the obvious opt-out. **Do not rely on this flag; label-match instead.**

Monzo asked 6 demographic questions (gender identity, transgender, sexual orientation, disability, neurodivergence, ethnicity). All optional. Each had a "Prefer not to say" first option.

#### Monzo's main 11 questions

| # | Req | Label | Field name | Type | Opts |
|---|---|---|---|---|---|
| 0 | ✓ | First Name | `first_name` | text | — |
| 1 | ✓ | Last Name | `last_name` | text | — |
| 2 | ✓ | Email | `email` | text | — |
| 3 |   | Phone | `phone` | text | — |
| 4 |   | Resume/CV | `resume` + `resume_text` | file OR textarea | — |
| 5 |   | LinkedIn Profile | `question_56634077` | text | — |
| 6 | ✓ | Are you a US person? | `question_56634078` | single_select | 2 |
| 7 | ✓ | 🛂 UK Right to Work | `question_56634079` | single_select | 6 |
| 8 | ✓ | 🔐 Candidate data acknowledgement | `question_56634082` | single_select | 2 |
| 9 |   | What is your current job title? | `question_58598753` | text | — |
| 10 |   | Discussed internal move with manager? | `question_58598754` | text | — |

Note: question labels can contain emoji (🛂, 🔐). Label matching must be unicode-aware.

#### Implications added

12. **Three separate question buckets.** Autofill must extract all three from `jobPost`:
    - `questions[]` — required + custom main fields (Schema A)
    - `eeoc_sections[].questions[]` — Schema A shape, nested under sections, **only present when `enable_eeoc: true`**
    - `demographic_questions.questions[]` — **completely different Schema C**, present when the company has configured a diversity survey (independent of `enable_eeoc`)
13. **Strategy for demographic/EEOC blocks: decline all by default.** All are optional; safe behaviour is to leave them blank or pick "Prefer not to say" / "Decline to Self Identify" / "I don't wish to answer". Let the user opt-in to answering them via a preferences flag if they want.
14. **`decline_to_answer` flag is unreliable.** Monzo's "Prefer not to say" had `decline_to_answer: false`. Label-match against a known opt-out vocabulary instead: `"Prefer not to say"`, `"Decline To Self Identify"`, `"I don't wish to answer"`, `"I do not want to answer"`, `"Not stated"`.
15. **Free-form option in demographic surveys.** Some options have `free_form: true`, meaning picking them opens a text-input. Autofill should avoid these by default (they require user input). The opt-out label-match handles this naturally.
16. **EEOC content includes legal HTML.** Each `eeoc_sections[].description` is government-mandated text. Autofill should never modify or skip rendering these — they're for the user's information.
17. **Label normalisation must be unicode-aware.** Question labels include emoji (🛂, 🔐). Strip leading non-letter characters when matching, or use a unicode-letter-aware regex.

---

### 5. BlueCrest Capital Management — AI Cloud Platform Engineer [C]

**Source URL:** `https://job-boards.greenhouse.io/bluecrestcapitalmanagement/jobs/7731837003`
**Submit path:** `https://boards.greenhouse.io/bluecrestcapitalmanagement/jobs/7731837003`
**Notable:** `enable_eeoc: false`, no `demographic_questions` block. Pure Schema A.

This entry adds no schema discoveries; it confirms uniformity. Logging mainly for the question-content patterns it shows for finance roles.

#### BlueCrest's 11 questions

| # | Req | Label | Field name | Type | Opts |
|---|---|---|---|---|---|
| 0 | ✓ | First Name | `first_name` | text | — |
| 1 | ✓ | Last Name | `last_name` | text | — |
| 2 | ✓ | Email | `email` | text | — |
| 3 |   | Phone | `phone` | text | — |
| 4 | ✓ | Candidate Location | `candidate_location` | text | — |
| 5 | ✓ | Resume/CV | `resume` + `resume_text` | file OR textarea | — |
| 6 | ✓ | Right to Work status | `question_30715962003` | single_select | 5 |
| 7 | ✓ | Current notice period | `question_30715963003` | text | — |
| 8 | ✓ | Current total compensation | `question_30715964003` | text | — |
| 9 | ✓ | Expected total annual compensation | `question_30715965003` | text | — |
| 10 | ✓ | Commit to 4 days/week in office? | `question_30715966003` | single_select | 2 |

#### Right-to-Work option labels (worth saving as a label-matching corpus)

```
"UK / Irish National"
"EU / EEA or Swiss National with Settled or Pre-Settled Status"
"Skilled Worker Visa"
"Spousal Visa"
"I do not have the Right to Work in the UK and require sponsorship"
```

These labels recur across UK-based companies with minor variations. Worth gathering a normalised mapping in the autofill (e.g. user's `visa_status: "Skilled Worker"` → match against option labels containing "Skilled Worker").

#### Implications added

18. **Free-form compensation/notice-period questions are common.** They're `input_text` and the recruiter expects a sentence or a number — autofill should write the user's stored answer verbatim. Where missing, leave blank rather than guess.
19. **Right-to-Work label vocabularies are partially standardised.** Building a label-matching table for visa statuses gives high coverage across UK Greenhouse forms with one small per-company tweak surface.
20. **`candidate_location` is sometimes required.** Cloudflare and BlueCrest require it; B2C2 and Monzo don't expose it. Autofill must populate when present.

---

### 6a. Hudson River Trading — Software Engineer (C++ or Python) 2026 Grads [SS]

**Source URL (DB):** `https://www.hudsonrivertrading.com/careers/job/?gh_jid=7053514`
**Redirects to:** `https://www.hudsonrivertrading.com/hrt-job/software-engineer-c-or-python-2026-grads-3/?gh_src=`

> **Correction (post-implementation, see §16):** the "HRT is off-platform" claim below was a survey methodology error — I probed the slug `hudsonrivertrading` (inferred from company name) when HRT's actual Greenhouse slug, correctly stored in `company_portals.ats_slug`, is `wehrtyou`. With the correct slug, `boards-api.greenhouse.io/v1/boards/wehrtyou/jobs/7053514?content=true&questions=true` returns a full 19-question schema. **HRT is on Greenhouse.** The wrapper-page HTML opacity (no greenhouse references visible in static HTML) is a JS-fetch-only pattern, not an off-platform signal. The autofill code reads the slug from the DB, so this case works correctly.

**Original (incorrect) finding:** The DB has them as `ats_provider='greenhouse'`, but:

| Probe | Result |
|---|---|
| `boards.greenhouse.io/hudsonrivertrading/jobs/7053514` | 404 |
| `boards.greenhouse.io/embed/job_app?for=hrt&token=7053514` | 404 |
| `boards.greenhouse.io/embed/job_app?for=hudsonrivertrading&token=7053514` | 404 |
| `boards.greenhouse.io/embed/job_app?for=hudsonrivertradingllc&token=7053514` | 404 |
| `api.greenhouse.io/v1/boards/hrt/jobs` | 404 |
| HRT careers page contains greenhouse references | None (only `gh_src` redirect tracking) |
| HRT page's static HTML contains an application form | No |

HRT runs WordPress + a bespoke `hrt-jobs` plugin (`/wp-content/plugins/hrt-jobs/scripts/frontend-bundle.min.js`) and `Gravity Forms`. The `gh_jid` in DB URLs is a **vestigial tracking parameter** — either HRT migrated off Greenhouse since being added to Cernio, or `gh_jid` was always cosmetic and they were always on their own stack.

**Cernio-level implications (not autofill):**

a. The `cernio resolve` / `populate-db` pipelines that marked HRT as Greenhouse were wrong. A re-validation pass should verify the Greenhouse slug is real by hitting `boards.greenhouse.io/embed/job_app?for=SLUG&token=ANY_KNOWN_JOB_ID` once, not just by URL-pattern-matching `gh_jid=`.

b. Other companies in the DB might have the same problem. A `check-integrity` step could probe every `ats_provider='greenhouse'` company's embed URL with one of its known job IDs and reclassify those that 404.

c. HRT specifically should be reclassified as `bespoke` (own ATS) and re-searched via the `search-jobs` skill's bespoke subagent path, not the mechanical Greenhouse fetcher.

This survey entry counts as a **data-quality finding**, not a Greenhouse schema discovery. No new field/schema info from HRT.

---

### 6b. Squarepoint Capital — Graduate Software Developer [SS] (HRT substitute)

**Source URL (wrapper):** `https://www.squarepoint-capital.com/open-opportunities?id=6040910&gh_jid=6040910` — thin SPA shell (7.7 KB), fetches and injects form client-side
**Greenhouse embed (canonical):** `https://boards.greenhouse.io/embed/job_app?for=squarepointcapital&token=6040910`
**Submit path:** `https://boards.greenhouse.io/embed/squarepointcapital/jobs/6040910` (same `/embed/{slug}/jobs/{id}` shape as GSA, confirming the GET vs POST asymmetry rule for embedded forms is universal)

#### Two distinct custom-domain integration patterns confirmed

| Pattern | Example | Mechanism | Autofill approach |
|---|---|---|---|
| **Genuine GH embed** | GSA, Squarepoint | Wrapper SPA fetches `boards.greenhouse.io/embed/job_app?for=X&token=Y` and injects markup | Skip wrapper; hit the embed URL directly |
| **Off-platform** | HRT | `gh_jid` is vestigial; form is on company's own infra | Falls outside Greenhouse autofill — Cernio should classify as `bespoke` |

The discriminator: probe `boards.greenhouse.io/embed/job_app?for=SLUG&token=JID` once. 200 → genuine; 404 → off-platform.

#### Squarepoint's 22 questions (compact)

Standard + custom mix; the noteworthy additions:

- **`question_48606662` "Why does this track interest you?"** — required textarea → essay question. This is where autofill should plug `prepare-applications` output.
- **`question_49214018` "Please only apply to the one job…"** — optional textarea, but mostly instructional. Probably should NOT be autofilled (it's a "please read this notice" disguised as a question).
- **`question_48606658` "Number of internships completed"** — 11-option single_select (0, 1, 2, … 10+). Numeric scale rendered as combobox.
- **`question_48606660` "Programming language you are most comfortable with"** — single_select with `["C++", "Python", "KDB+/q", "React"]`. **React is a UI framework, not a language.** Mismatched semantics — recruiter conflated. Autofill should still match user's `primary_language: "C++"` to the literal option string.
- **`question_48606661[]` "Career track interest"** — multi_value_multi_select with 4 long-form labels (Python/C++/Frontend/Quant). Multi-pick. Autofill needs an explicit array of preferences.
- **`question_49905875[]` "Preferred Office Location"** — 18-option multi_value_multi_select. Long checkbox lists are common.
- **`question_60449429` "Interview recording opt in/opt out"** — single_select with `["Opt in", "Opt out"]`. Not `["Yes", "No"]` — vocabulary varies.
- **`question_61030782` Singapore government scholarship** — irrelevant for UK user; should be left blank if optional. Autofill must handle "irrelevant question" gracefully.

#### Implications added

21. **Essay-style required questions (textarea, no options) are the highest-value autofill targets.** `prepare-applications` already produces structured JSON answers for these; the autofill just needs to map by label.
22. **"Instructional textarea" questions exist** (Squarepoint's "Please only apply to one job..."). The label is a paragraph of instructions, not a real question. Autofill heuristic: if the label is > 200 characters and contains words like "please" / "note" / "if our team..." then leave blank by default and surface to user.
23. **Discriminate custom-domain patterns by probing the embed URL.** One probe per company (not per job) at population time is enough to distinguish "genuine Greenhouse with custom wrapper" from "stale `gh_jid` tracking on off-platform ATS". Saves the autofill from running blind against companies that aren't actually on Greenhouse.
24. **Yes/No vocabulary is not universal.** Some forms use `Opt in / Opt out`; others use `Yes / No`; consent questions use `Acknowledge / Confirm`. Autofill must label-match against the option's `label`, not against a hardcoded `Yes`/`No`.

---

### 7. Anthropic — Research Engineer, ML (RL Velocity) [B] — essay-heavy

**Source URL:** `https://job-boards.greenhouse.io/anthropic/jobs/5198074008`
**Submit:** `https://boards.greenhouse.io/anthropic/jobs/5198074008`
**Why included:** rich open-ended questions — these are the autofill targets that `prepare-applications` exists to feed.

#### Essay textarea questions

| # | Req | Label | Field name | Description (hint to user) |
|---|---|---|---|---|
| 11 | ✓ | **Why Anthropic?** | `question_15796681008` | "Why do you want to work at Anthropic? (We value this response highly — great answers are often 200-400 words.)" |
| 14 |   | Additional Information | `question_15796684008` | "Add a cover letter or anything else you want to share." |

The description carries **target length** ("200–400 words") in HTML. **Autofill must surface descriptions to the user OR pass them to `prepare-applications` as guidance.** A 50-word answer where the recruiter wanted 200–400 is a soft fail.

#### Other notable Anthropic questions

- **Q10 "AI Policy for Application" (REQ, Yes/No)** — description is a 400-character HTML paragraph asking the candidate to acknowledge Anthropic's AI partnership guidelines. Autofill must surface this to the user (it's a meaningful consent, not a checkbox). The answer is binary.
- **Q12 + Q13 sponsorship questions are duplicated** — Q12 "Do you require visa sponsorship?" and Q13 "Will you now or in the future require employment visa sponsorship…". Same answer needed, two questions. Pattern: when two questions both mention sponsorship/visa, the same boolean answers both.
- **Q16/Q17 paired relocation Qs** — Q16 boolean "open to relocation?", Q17 text "What address are you working from? (type 'relocating' if you'd need to)". Pattern: a boolean primary plus a conditional text follow-up.
- **Q5 "(Optional) Personal Preferences"** — `input_text` with no other guidance. Open-ended text fields with no description are a recurring autofill challenge; the safe default is to leave them blank.

---

### 8. XTX Markets — Software Developer, Research Tech [A] — essay-heavy

**Source URL:** `https://job-boards.greenhouse.io/xtxmarketstechnologies/jobs/7407705003`
**Submit:** `https://boards.greenhouse.io/xtxmarketstechnologies/jobs/7407705003`

#### Essay textarea questions

| # | Req | Label (full) |
|---|---|---|
| 8 | ✓ | "Please briefly highlight **why you have applied for this role specifically at XTX**. Please note this role does not interface with traders, this role focuses on research technology infrastructure." (rest truncated) |
| 9 | ✓ | "Please briefly outline **a highly scalable system that you have played a significant role in developing**, which has exceeded conventional limits…" |

Both required, no description hints (the prompt is entirely in the label, including a clarifying caveat about the role's scope). XTX is at one end of the spectrum: just two essay questions, no description fluff, recruiter expects the candidate to write the entire context themselves.

#### Pattern comparison: Anthropic vs XTX

| Dimension | Anthropic | XTX |
|---|---|---|
| Essay-question shape | Short label + rich description | Long label, no description |
| Essay count | 1 required + 1 optional | 2 required |
| Length hint | "200-400 words" in description | None |
| Topic specificity | Open ("Why Anthropic?") | Highly specific ("a system you built that exceeded conventional limits") |

#### Implications added (essay-handling)

25. **`description` is HTML and carries hidden context.** Length hints, examples, definitions, consent text. Autofill must:
    - Parse the HTML (strip tags) to extract plain text
    - Pass description AND label to `prepare-applications` as the prompt context
    - Surface descriptions to the user as the form is rendered so they see the recruiter's hints
26. **Essay-question labels can themselves carry context** (XTX's "Please note this role does not interface with traders"). Pass the full label, not a truncated version, to the answer-generation pipeline.
27. **Sponsorship-class questions often appear twice** in slightly different phrasings. The same answer applies. Autofill can detect "visa" + "sponsor" + ("now" OR "future") and treat them as one logical question.
28. **Paired binary + conditional-text pattern** (relocation Yes/No + "address" text). Autofill should answer both consistently: if "open to relocation" = Yes, the text answer is the user's preferred address; if No, the text answer is the current address. Pattern is common enough to special-case.
29. **Open-ended `input_text` with no description is the worst autofill target.** Anthropic's "(Optional) Personal Preferences" is a single text input with no guidance about what to write. Leave blank by default; flag for user attention rather than guessing.

---

### 9. Anthropic — Safeguards Foundations [C] — pay range exposed

**Source URL:** `https://job-boards.greenhouse.io/anthropic/jobs/5191433008`
**Location:** London, UK (despite Anthropic being US-HQ; their Greenhouse covers all regions)

#### What's new

- **`jobPost.pay_ranges` populated for the first time** in our sample:
  ```jsonc
  [{
    "title": "Annual Salary:",
    "description": "<p>The annual compensation range for this role is listed below…</p>",
    "min": "£255,000",
    "max": "£325,000",
    "currency_type": "GBP"
  }]
  ```
  Salaries are pre-formatted strings (e.g. `"£255,000"`), not raw numbers. Cernio could surface these in the TUI job-detail view.

- **`multi_select_style` is an extra key on every question in the Remix HTML schema** — a UI rendering hint (probably controls dropdown-vs-list rendering). **Not present in the public API response.** Autofill should ignore it; it's Remix-renderer-only metadata.

Otherwise identical pattern to job 5198074008 (Research Engineer): Anthropic uses a per-team question template, so two roles at the company have ~95% identical forms.

---

### 10. DRW — DevOps Engineer (London) [C] — first full US-style EEOC sample

**Source URL:** `https://job-boards.greenhouse.io/drweng/jobs/7918653`
**Submit:** `https://boards.greenhouse.io/drweng/jobs/7918653`
**Critical finding:** UK-located role, but **`enable_eeoc: true` with 4 EEOC sections** because DRW is a US firm (Chicago HQ) and EEOC config lives at company level, not job level.

#### EEOC sections on DRW

| Section | Question | Field name | Options |
|---|---|---|---|
| 0 | Gender | `gender` | 3 |
| 0 | **Race** | `race` | 8 — **first sample with race/ethnicity** |
| 1 | Veteran Status | `veteran_status` | 3 |
| 2 | Disability Status | `disability_status` | 3 |
| 3 | (PRA legal preamble, 0 questions) | — | — |

**Race options on DRW:**
(8 options; standard US EEOC categories — White, Black or African American, Asian, Hispanic or Latino, American Indian or Alaska Native, Native Hawaiian or Pacific Islander, Two or More Races, Decline To Self Identify)

#### Other unusual DRW patterns

- **Q5 "Candidate Location"** required (matching BlueCrest)
- **Q12 "Are you legally authorized to work in the United States?"** — **on a London role.** Boilerplate copy-paste from a US template. Autofill should answer literally based on question text, not assume relevance.
- **Q15/Q16 "Legal First Name" / "Legal Last Name" as `textarea` types** — unusual; multi-line textarea for short single-line answers. Confirms the type is purely recruiter-set.
- **Q10 "How did you hear about this job?" with 16 options** — long enough to warrant a typeahead in the UI.

#### Implication: company-vs-job EEOC config

**EEOC sections appear based on COMPANY config, not the job's location.** UK roles at US firms (DRW, Anthropic depending on team) can still surface US-EEOC. Autofill design must:

- Render EEOC questions whenever `enable_eeoc: true`, regardless of job location
- Decline-all by default (per Implication 13)
- Surface to the user: "this is US-EEOC reporting, not optional in the US legal sense for the employer, but always optional for the candidate"

---

### 11. Proton — Rust Software Engineer (London/Geneva) [A] — soft-conditional pattern

**Source URL:** `https://job-boards.eu.greenhouse.io/proton/jobs/4585460101`
**Location:** London + Geneva

#### Confirms the screenshot's "combobox + free-text-explain" pattern

Q15 + Q16 are a **soft-conditional pair**:
```
[15] REQ (single_select [Yes/No])  "Do you have an eligibility/working permit to work in this particular location?"
[16]     (textarea)                 "If 'yes' can you specify the type of working permit you posses (citizenship, permanent residency, work visa)..."
```

**Both render unconditionally.** The conditional logic is encoded in the label text, not in the schema. Autofill heuristic:

```
IF question.fields[0].type == 'textarea'
AND question.label starts with "If yes," / "If no," / "If applicable,"
AND a preceding select question has a Yes/No answer
THEN: only fill if the trigger answer matches; otherwise leave blank
```

#### Other Proton patterns

- **Q11 "Salary expectations — select the right choice"** (4 options: ranges) + **Q12 "Salary currency"** (7 options: GBP, EUR, USD, CHF, etc.) — composite question rendered as two separate selects + Q10 free-text number. Three coupled questions for one logical value.
- **Q10 "Salary expectations — number"** is `input_text` with description "Please include your salary expectations (numbers only please)". Annoying composite form.
- Required textarea Q8 **"What is it about Proton that excites you?"** — essay pattern matches Anthropic/XTX.
- **Geneva listed alongside London** — Swiss + UK locations on same job. Affects visa-status answer.

---

### 12. Jane Street — ML Performance Engineer [A] — biggest form in the sample (26 questions)

**Source URL:** `https://job-boards.greenhouse.io/janestreet/jobs/7449252002`

#### Notable patterns

- **26 questions** — biggest in our sample. Includes student-targeted Qs even though the role is full-time (template reuse).
- **128-option `multi_value_single_select`** for "How did you hear about us?" — long enough that the rendered DOM has 128 `<li>` elements. Performance consideration for the autofill (don't load all options upfront if not needed; click-to-open).
- **Q11 "Enter your college/university"** + **Q12 "Candidate: college/university"** — two text fields for the same data. Likely a CRM-side legacy + user-facing duplicate. Autofill should fill both with the same value.
- **Q22 "Languages Spoken" `multi_value_multi_select` (11 options)** + **Q23 `input_text` "Additional Information For Languages"** — multi-select + free-text follow-up pattern; same shape as Monzo's free-form demographic options.
- **Q24 "What year did you graduate high school?"** — open-ended `input_text` for year. Could be a number picker but Greenhouse offers no such type.
- **Q9 "Pronouns" as optional `input_text`** — explicit Greenhouse pattern for inclusive forms.
- **Q21 "Area of Study" (20-option single_select)** + **Q16 "Current education level" (7-option)** — coupled academic questions.
- **`Website UID`** — strange field name (Q18). Possibly Greenhouse-internal for portfolio URL.

Demonstrates that **long forms accumulate "soft-duplicates"** through CRM/template drift over time. Autofill must tolerate this — answering the same data into multiple fields with similar labels.

---

### 13. Schonfeld — 2027 Summer Internships [A] — short essay-style internship form

**Source URL:** `https://job-boards.greenhouse.io/schonfeld/jobs/7635430`
**Location:** Dubai, HK, London, Miami, NYC (multi-region single job; visa answers depend on which the candidate picks at interview)

10 questions; short, internship-flavoured form:
- **Q8 "What is your current cumulative GPA?"** — required `input_text`. Freeform numeric (no validation type, recruiter expects "3.8/4.0" or similar).
- **Q9 "What degree are you currently pursuing?"** — 4-option single_select (presumably Bachelor's, Master's, PhD, MBA).
- No `enable_eeoc`, no demographic, no essay textareas (Schonfeld's full intern application is presumably elsewhere; this is express-interest).

Pattern: **short "express interest" forms** are screened-then-redirected to the full application separately. Cernio's autofill should be prepared for the case where submitting `prepare-applications` output produces a confirmation email asking the candidate to complete a separate detailed application.

---

### 14. Tower Research — Quantitative Developer (London) [A] — `embed/job_board/js` pattern

**Source URL:** `https://www.tower-research.com/open-positions/?gh_jid=7080219`
**Wrapper mechanism:** JS-based embed pattern — `<script src="https://boards.greenhouse.io/embed/job_board/js?for=towerresearchcapital">`

This is a **DIFFERENT** embed family from `embed/job_app`:

| Variant | Returns | Use case |
|---|---|---|
| `boards.greenhouse.io/embed/job_app?for=X&token=Y` | HTML form for one job | What GSA/Squarepoint use |
| `boards.greenhouse.io/embed/job_board/js?for=X` | JS bundle that fetches and renders entire job board | What Tower/Jump use |

Tower's wrapper page contains the `embed/job_board/js` script tag but no inline Remix HTML — the JS fetches data client-side. The underlying schema is still on the public API: `boards-api.greenhouse.io/v1/boards/towerresearchcapital/jobs/7080219?content=true&questions=true` returns the form schema cleanly.

---

### 15. Jump Trading — DevOps/HPT System Engineer (London) [A] — public API discovery

**Source URL:** `https://www.jumptrading.com/hr/job?gh_jid=6004172`
**Wrapper mechanism:** custom JS that fetches **`boards-api.greenhouse.io/v1/boards/jumptrading/jobs/${jobId}?content=true`** directly.

**This is the critical architectural discovery.** Jump's wrapper page contained explicit JS:
```javascript
const url = `https://boards-api.greenhouse.io/v1/boards/jumptrading/jobs/${jobId}?content=true`;
const res = await fetch(url);
const data = await res.json();
```

This reveals the **public Greenhouse Job Board API**: `boards-api.greenhouse.io/v1/boards/{slug}/jobs/{id}?content=true&questions=true`. Verified working across **all 13 surveyed companies** including EU subdomain ones (b2c2 resolves on the US `boards-api.` host — single global host, no regional split for the API).

#### What the API returns (richer than HTML)

| Field | Notes |
|---|---|
| `id`, `internal_job_id`, `requisition_id` | Clean job identifiers |
| `title`, `company_name`, `location` (structured `{name}`), `departments`, `offices` | For display |
| `absolute_url` | Canonical URL |
| `first_published`, `updated_at`, `application_deadline` | Staleness signals |
| `content` | Job description HTML |
| `questions[]` | Schema A — main form (no `multi_select_style` clutter) |
| `location_questions[]` | **New bucket** — geolocation block, see below |
| `demographic_questions` | Schema C |
| `compliance` / `data_compliance` | GDPR consent config |
| `metadata[]` | Custom job-level fields (e.g. `"Location Type": "On-Site"`) |

#### New question bucket: `location_questions[]` (Schema D)

```jsonc
[
  { "label": "Longitude", "required": false, "fields": [{"name": "longitude", "type": "input_hidden", "values": []}] },
  { "label": "Latitude",  "required": false, "fields": [{"name": "latitude",  "type": "input_hidden", "values": []}] },
  { "label": "Location",  "required": false, "fields": [{"name": "location",  "type": "input_text",   "values": []}] }
]
```

- **New field type discovered: `input_hidden`** — for lat/long capture. Autofill should ignore these (they're for geolocation, not user-entered data).
- Schema shape is identical to Schema A — same `{required, label, fields:[{name, type, values}]}` contract. Just a different bucket.

---

### 16. Stripe — Full Stack Engineer, Expansion — actually on Greenhouse

**Source URL:** `https://stripe.com/jobs/search?gh_jid=7531158`

**Correction (post-implementation):** I initially marked Stripe and HRT as off-platform because their wrapper-page HTML contained no Greenhouse references. That was misleading. The actual cause:

- **HRT**: DB slug is `wehrtyou` (not `hudsonrivertrading` as the company name suggests). With the correct slug, `boards-api.greenhouse.io/v1/boards/wehrtyou/jobs/7053514?content=true&questions=true` returns HTTP 200 with a full 19-question schema. **HRT is on Greenhouse.** What was wrong was the survey probe inferring the slug from the company name instead of reading it from the DB.
- **Stripe**: DB slug is `stripe`. `boards-api.greenhouse.io/v1/boards/stripe/jobs` returns 481 live jobs. The specific job ID 7531158 returns 404 — that's a **stale job**, not an off-platform company. Stripe is on Greenhouse; the search-jobs flow will replace the stale ID on its next run.

**Reinforces a critical autofill rule:** always source the ATS slug from `company_portals.ats_slug` in the DB, never infer it from the company name or the wrapper-page URL. The new autofill code does exactly this.

The "no greenhouse references in wrapper-page HTML" pattern (Stripe, HRT, partially Squarepoint) means the company fetches the embed entirely client-side from JS. The static HTML is opaque to a non-browser-driven scraper — but the public Greenhouse API (which we now use) sees through this completely.

**Net data-quality findings: 0 confirmed off-platform companies in this 16-form survey.** The earlier claim of "2 off-platform" was a methodology error on my part. The existing `cernio check`'s `verify_ats_slugs` (in `src/pipeline/check.rs`) already probes the correct slugs from the DB and will catch real off-platform cases.


---

## Synthesis

**Sample:** 8 Greenhouse forms (Cloudflare, B2C2, GSA, Monzo, BlueCrest, Squarepoint, Anthropic, XTX) across 5 URL patterns and 4 grading tiers. One company (HRT) turned out to be off-platform — a data-quality finding, not a schema sample.

### What is universal across every Greenhouse form

```
                       window.__remixContext
                              │
                              ▼
                       state.loaderData[<route_key>]
                              │
                              ▼
                            jobPost
                              │
              ┌───────────────┼─────────────────────────────┐
              ▼               ▼                             ▼
        questions[]      eeoc_sections[]           demographic_questions
        (Schema A)    (Schema A, nested,         (Schema C — different
                       only if enable_eeoc)        vocabulary entirely)
```

1. **Schema location.** The entire form schema is embedded as `window.__remixContext` in a `<script>` tag in the initial HTML. **No JS execution required to extract it.** True for standalone Remix pages, EU-region pages, and embedded forms.

2. **Route key shape.**
   - Standalone: `routes/$url_token_.jobs_.$job_post_id`
   - Embedded (via `/embed/job_app`): `routes/embed.job_app`
   - Read it dynamically; don't hardcode.

3. **Schema A — main `questions[]`.** Per-question shape:
   ```jsonc
   {
     "required": bool,
     "label": str,              // human text — STABLE KEY for autofill matching
     "description": str?,        // HTML, may contain length hints / consent text / examples
     "fields": [{
       "name": str,             // form field name on POST
       "type": "input_text" | "textarea" | "input_file" | "multi_value_single_select" | "multi_value_multi_select",
       "values": [{ "value": str, "label": str }]?  // populated for selects only
     }]
   }
   ```

4. **Standard field names are a fixed Greenhouse vocabulary** (opt-in per job):
   - `first_name`, `last_name`, `preferred_name`, `email`, `phone`
   - `candidate_location`, `country`
   - `resume` (file), `resume_text` (textarea fallback)
   - `cover_letter`, `cover_letter_text`
   - Resume + cover-letter render as a single logical question with TWO `fields[]` entries — file OR textarea.

5. **Custom questions use `question_NNN` IDs that are per-job and unstable.** Match by `label`, never by ID.

6. **Select-option values are unstable too.** Each option is `{value: "623958729", label: "Yes"}`; the numeric `value` (option_id) varies across companies and over time. **Match user answers to option `label`, never to `value`.**

7. **Submit-path is in the JSON, not derivable from the page URL.**
   - Standalone: `submitPath` host is `boards.greenhouse.io` (US) or `boards.eu.greenhouse.io` (EU); path matches `/{slug}/jobs/{id}`.
   - Embedded: `submitPath` has the shape `/embed/{slug}/jobs/{id}` (NOT `/embed/job_app?for=...&token=...` — that's the GET URL only).
   - Always read `submitPath` from the JSON. Don't reconstruct.

### What varies (catalogue, in observed frequency)

| Variation | Observed values | Frequency |
|---|---|---|
| URL pattern | `boards.greenhouse.io` (legacy, 301s) / `job-boards.greenhouse.io` / `job-boards.eu.greenhouse.io` / `/embed/job_app?for=X&token=Y` / custom-domain wrapper | 5 distinct |
| Question count | 6 (Anthropic-Fellows redirect) to 22 (Squarepoint) | Median ~12 |
| `enable_eeoc` | `false` on 7/8 forms; `true` only on Monzo | Rare, but materially shifts schema |
| `demographic_questions` present | Only Monzo | Rare |
| Question ID length | 8 / 10 / 11 digits | Per-company |
| Yes/No vocabulary | `Yes/No`, `Opt in/Opt out`, `Acknowledge/Confirm`, `I confirm.../I do not...` | Common variation |
| `phone` required | Required on Cloudflare; optional everywhere else | ~25% |
| `candidate_location` exposed | 2 of 8 | Sparse |
| `preferred_name` exposed | 1 of 8 (GSA) | Rare |
| `description` carries length hints | Only Anthropic | Rare, but valuable when present |
| Resume required | Yes on 4/8; optional on 4/8 | ~50% |
| Essay (textarea) questions | 0 on most; 2 on XTX and Anthropic; 1 on B2C2 + Squarepoint | Tier-correlated |

### The three question buckets — comparison

| Bucket | Lives at | Schema | When present | Autofill default |
|---|---|---|---|---|
| Main `questions[]` | `jobPost.questions` | Schema A | Always | Fill from package |
| `eeoc_sections[]` | `jobPost.eeoc_sections[].questions` | Schema A (nested) | When `enable_eeoc: true` | Pick opt-out option |
| `demographic_questions` | `jobPost.demographic_questions.questions` | Schema C (entirely different shape) | When company configured a custom survey | Pick opt-out option |

**Schema C** uses `name`/`id`/`answer_type.key`/`answer_options[]` (with `free_form` and `decline_to_answer` flags) — different parser needed.

### Recommended autofill design for `src/autofill/greenhouse.rs`

Three options were considered:

| Strategy | Mechanism | Pros | Cons |
|---|---|---|---|
| **A: Pure POST** | Parse schema → POST multipart directly to `submitPath` | ~10× faster; no browser; no React-event bugs | reCAPTCHA v3 token still needs a real browser to generate; multipart edge cases |
| **B: Pure DOM-driving** | CDP `Input.insertText` + clicks on visible DOM | Captcha passes naturally; submission looks human | Sensitive to DOM changes; the React-event problem `project_autofill.md` already documented |
| **C: Schema-driven DOM filling** (recommended) | Parse `window.__remixContext` for schema → use CDP `Input.insertText`/clicks/`setFileInputFiles` to drive the DOM by *type from JSON*, not by selector | Best of both: schema-based dispatch avoids selector brittleness; real keystrokes fire React natively; captcha gets genuine signals | Per-application timing slower than pure POST |

#### Implementation shape for Strategy C

```
1. Resolve URL
   ├─ If page URL is custom-domain wrapper → probe for boards.greenhouse.io/embed/job_app?for=X&token=Y
   │  references in HTML → navigate there directly
   └─ Otherwise navigate to the listing URL as-is

2. Fetch + parse
   ├─ HTTP GET the page (no browser yet)
   ├─ Regex-extract <script>window.__remixContext = {...};</script>
   ├─ Parse JSON → walk to .state.loaderData[<route_key>]
   └─ Extract jobPost.questions, jobPost.eeoc_sections, jobPost.demographic_questions, submitPath

3. Match answers
   ├─ For each main question:
   │  ├─ If field.name is a standard name → look up in profile + application_package
   │  └─ Else → normalise label (lowercase, strip emoji, collapse whitespace) →
   │            match against package's question_label_index
   ├─ For each EEOC question → pick option whose label is in
   │  {"Decline To Self Identify", "I don't wish to answer",
   │   "I do not want to answer", "Prefer not to say"}
   └─ For each demographic_question → same opt-out heuristic

4. Drive the DOM via CDP
   ├─ Launch Chrome (chromiumoxide) and navigate
   ├─ Wait for form-render (poll until #application-form exists)
   ├─ For each matched answer, dispatch by JSON type (NOT by CSS selector):
   │   input_text/textarea → focus + Input.insertText (real keystrokes; React fires)
   │   input_file          → DOM.setFileInputFiles
   │   multi_value_single_select → click combobox trigger, wait for listbox,
   │                               click <li> matching option label,
   │                               verify hidden requiredInput got option value
   │   multi_value_multi_select  → for each selected label, click corresponding checkbox
   └─ Optionally let grecaptcha.execute run naturally on form-blur events

5. Pre-submit verification
   ├─ Read each visible input's React-state value via Runtime.evaluate
   ├─ Confirm every required question has a value
   ├─ Compare against the original package answers
   └─ Surface diffs to user as a TUI confirmation before submission

6. Submit
   └─ Click the <button type="submit"> — let Greenhouse handle the rest
```

#### Why this fixes the current bug

`project_autofill.md` notes: *"JavaScript `.value =` assignment doesn't trigger React's synthetic event system."* Strategy C replaces `.value =` with CDP `Input.insertText`, which generates real keystroke events that React's synthetic event system listens to natively. **The bug disappears without us needing to special-case React.** This is the standard pattern in CDP-driven automation.

#### What it does NOT solve

- **reCAPTCHA v3 scoring as a bot.** Strategy C improves the signals (real keystrokes, real clicks, real timing), but if Greenhouse's reCAPTCHA Enterprise model flags the session, no amount of CDP can fix it without IP rotation / fingerprint adjustment. **Defer until we observe the failure mode.**
- **File uploads with arbitrary paths.** `DOM.setFileInputFiles` requires absolute paths to local files. Application packages must include resume + cover-letter file paths (or have them generated at autofill time).
- **The "instructional textarea" anti-pattern.** Questions whose label is a paragraph of instructions (Squarepoint's "Please only apply to one job..."). Heuristic-based skip or surface-to-user is the right move.

### Concrete next steps for the implementation

1. **Replace `src/autofill/greenhouse.rs::selectors`** (CSS selector module) with a Remix-context parser and a `Question` enum mapped from the JSON.
2. **Build a `LabelNormaliser`** with a small synonym table (visa-status variants, sponsorship phrasings, Yes/No vocabulary).
3. **Add a `verify_submission_ready()` pass** that re-reads the React state and surfaces mismatches before the user is asked to confirm.
4. **Update `prepare-applications`** to write answers keyed by **normalised label**, with the option to store an optional `description_hint` so the autofill can show the recruiter's "200-400 words" guidance to the user when generating.
5. **Add `cernio check`-level validation** that probes the Greenhouse embed URL for any `ats_provider='greenhouse'` company once, and reclassifies 404s as `bespoke` (catches the HRT-style data drift).
6. **Capture the Schema C path separately** — autofill should never confuse `demographic_questions` with `questions`. They share no field names.

### Open questions

- **How does Greenhouse handle the reCAPTCHA Enterprise token in the submit POST?** Need to inspect the network traffic of a real submission to confirm the field name and where the token comes from. (Not blocking — Strategy C lets the real browser run the captcha naturally.)
- **Are there other ATSes worth surveying first?** Workday, Eightfold, iCIMS — all have radically different schemas and would each need their own anatomy doc. Greenhouse covers ~60% of Cernio's resolved companies; the rest are bespoke.
- **Should the autofill probe the embed URL at populate-db time and store the rendered schema?** Would let us pre-validate that a company's autofill is feasible before any user time is spent on application packages.

---

## Synthesis revision (after batch 2 — 8 more samples)

**Sample now: 16 forms across 14 unique companies** (HRT + Stripe were off-platform, 14 had valid schemas). Two architectural discoveries supersede the original recommendation:

### Discovery 1: The public Greenhouse Job Board API is the canonical source

```
https://boards-api.greenhouse.io/v1/boards/{slug}/jobs/{job_id}?content=true&questions=true
```

- **Universal**: returns 200 for every Greenhouse-resolved company, regardless of region (EU or US)
- **Single host**: `boards-api.greenhouse.io` (no EU twin) — confirmed for B2C2 which renders on `job-boards.eu.greenhouse.io`
- **Cleaner schema**: no `multi_select_style` UI clutter, identical label/required/fields/type structure
- **Richer payload**: exposes `metadata`, `compliance`, `data_compliance`, `pay_ranges`, `application_deadline`, `first_published`, `updated_at`, `internal_job_id`, `requisition_id`, structured `location` and `offices` — most of which are not in the HTML
- **Zero HTML parsing**: no regex, no Remix-route discovery, no script-tag extraction

**This supersedes the "parse `window.__remixContext`" approach as the primary strategy.** Remix-context parsing is now the *fallback* for edge cases (in case the API is ever rate-limited or fails for a specific job).

### Discovery 2: There are FOUR question buckets, not three

| Bucket | Schema | New finding |
|---|---|---|
| `questions[]` | Schema A | Main form (universal) |
| `eeoc_sections[].questions[]` | Schema A (nested) | DRW confirmed full US-EEOC including Race on a UK role — config is company-level |
| `demographic_questions.questions[]` | Schema C (different vocabulary) | Confirmed on Monzo |
| **`location_questions[]`** | Schema A — but with **new `input_hidden` field type** | Geolocation capture: latitude/longitude (hidden) + location text. Autofill should populate `location` (text), leave lat/long blank |

### Discovery 3: New patterns surfaced in batch 2

- **Soft-conditional pairs** (Proton Q15+Q16): both questions always render; the second's label starts with "If yes…" / "If no…" and is filled only when the first's answer matches. Autofill heuristic: detect label-prefix + look up the trigger question.
- **Composite/coupled questions** (Proton salary: text + currency + range; Jane Street: language multi-select + free-text follow-up). Multiple fields encode one logical answer. Autofill needs an explicit "composite question" detector.
- **Soft-duplicate fields** (Jane Street college/university appears in 2 fields with slightly different labels). Fill both with the same value.
- **128-option dropdowns** (Jane Street "How did you hear about us?"). DOM rendering / option scanning needs to be lazy.
- **Boilerplate copy-paste** (DRW asking US work authorization on a London role). Autofill answers literally against the question text, doesn't assume relevance from job location.
- **Pay ranges as pre-formatted strings** (Anthropic): `"£255,000"`, not raw numbers. Don't parse for math; render as-is.
- **`metadata[]` includes "Location Type"** (On-Site / Hybrid / Remote). Useful for Cernio's job-list TUI beyond autofill.
- **`data_compliance.type: "gdpr"`** with consent flags — could surface to user before submission for EU companies.

### Updated recommended autofill design — Strategy C′ (API-driven)

```
1. Fetch schema
   ├─ Resolve company.ats_slug from DB (e.g. "anthropic")
   ├─ Extract job_id from the URL (e.g. 5198074008)
   └─ GET https://boards-api.greenhouse.io/v1/boards/{slug}/jobs/{job_id}?content=true&questions=true
      → parses to JSON in one shot, no HTML scraping

2. Validate
   ├─ HTTP 404 → company is off-platform; flag to Cernio for reclassification to bespoke
   ├─ HTTP 200 → proceed
   └─ Cache the schema in the application_packages row for re-use

3. Match answers (as before)
   ├─ Main questions → label-match against the package
   ├─ EEOC + demographic + location_questions → decline-by-default
   └─ Soft-conditional pairs → fill follow-up only if trigger matches

4. Drive the DOM (browser strategy unchanged)
   ├─ Launch Chrome / chromiumoxide
   ├─ Navigate to the application URL (still need a real browser for reCAPTCHA v3 token generation)
   ├─ For each matched answer, dispatch by JSON type:
   │   input_text/textarea  → focus + Input.insertText
   │   input_file           → DOM.setFileInputFiles
   │   input_hidden         → SKIP (location_questions only)
   │   multi_value_*_select → click combobox trigger, click <li> by label match
   └─ Verify React state mirror via Runtime.evaluate

5. Pre-submit verification + submit (unchanged)
```

**Net change:** Step 1's "fetch HTML + regex + JSON parse" becomes a single API GET. Everything downstream (label matching, type dispatch, DOM driving) is unchanged. The autofill code stays the same shape; the schema source is just cleaner.

### Updated next steps for the implementation

1. **Add a `GreenhouseSchema` fetcher** that hits `boards-api.greenhouse.io/...` and returns a typed Rust struct. This is the new primary source.
2. **Keep the Remix-context parser as a fallback** — only invoke if the API returns non-200.
3. **Validate at `populate-db` time**: probe `boards-api.greenhouse.io/v1/boards/{slug}/jobs/ANY_KNOWN_JOB_ID` once per Greenhouse-resolved company. Reclassify 404s as `bespoke`. Catches HRT/Stripe-style drift cheaply.
4. **Surface `metadata` in TUI**: "Location Type", departments, application_deadline are all valuable enrichment for the job list — not autofill-specific.
5. **Build the soft-conditional detector** based on label-prefix heuristics (`If yes,` / `If no,` / `If applicable,` / `If you answered`).
6. **Define a `composite question` strategy** — when 2-3 adjacent questions share a topic prefix (e.g. all start with "Salary expectations"), treat as one logical answer that fans out.
7. **`location_questions` autofill behaviour**: fill `location` text-field with the candidate's city; leave `latitude`/`longitude` (`input_hidden`) blank — they're for browser-side geolocation, not user input.

### Data-quality findings (Cernio-scope, not autofill)

- **2 of 13 surveyed companies are off-platform** despite DB classification as Greenhouse: HRT and Stripe. Both have `gh_jid` in URL as vestigial tracking.
- **Suggested `cernio check` step**: for every company with `ats_provider='greenhouse'`, probe `boards-api.greenhouse.io/v1/boards/{slug}/jobs/{any_known_job_id}`. 404 → reclassify as bespoke and re-search via the bespoke-companies subagent path.
- **Stale jobs**: Cloudflare had 2 of 3 sampled job IDs returning 302 to generic /careers/jobs. The API would return 404 on those too — same check covers both staleness modes.

