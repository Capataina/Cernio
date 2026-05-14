# Application Voice: Idea Sheet

This file is an **inspiration source**, not a hard rule book. When generating
essays or factual answers for a job application, draw from it. Use the bits
that fit the role and the question. Skip the bits that don't. A specific
suggestion below absent from the final answer is fine; a specific suggestion
mechanically inserted everywhere defeats the point.

This file is deliberately **portfolio-agnostic**. It does not name specific
projects, technologies, or OSS contributions, because the portfolio evolves.
The agent reading this file alongside `profile/projects/`, `profile/skills.md`,
and `profile/experience.md` is responsible for choosing which projects, which
contributions, and which skills best match the role being applied to. The
guidance here is about *how to apply that judgment*, not about *which project
to pick* for a given role.

Three things to keep in mind while reading this file:

1. The "narrative hooks" below are screenshot-worthy moves a recruiter might
   actually mention to a teammate. Use one per application at most. Two hooks
   in one essay reads as ornament rather than substance.
2. The "common phrasings" are defaults. Per-job context can override any of
   them; when the form or the company calls for a different shape, prefer
   the shape that fits.
3. The "style notes" are preferences, not constraints. The Answer Generation
   Standard in `prepare-applications/SKILL.md` is the constraint layer; this
   file is the texture layer that runs on top.

---

## Conciseness

A recruiter skimming 20 applications an hour will not read a four-paragraph
"why our company" answer. Aim short. The discipline is not "fewer words for
the sake of fewer words"; it is "every word load-bearing, no filler, no
jargon-stretching".

Heuristics that produce concise prose without losing information:

- **One project per essay, not a list.** Pick the strongest project for this
  role (see "How to Pick What to Lead With") and describe it in depth. Three
  half-described projects read weaker than one fully-described one.
- **Specifics replace adjectives.** "Highly performant" becomes a latency
  number. "Robust" becomes a test count. "Extensive experience" becomes a
  named project with a named outcome. The specifics are usually shorter than
  the adjectives they replace.
- **Cut warm-up sentences.** Every essay's first draft has a sentence that
  introduces what the next sentence will say. Delete it. Start at the point.
- **No jargon padding.** Naming an architectural decision in plain words is
  shorter than stacking buzzwords. The reader is technical; they don't need
  the buzzword scaffolding.
- **Resist the urge to oversell impressive things.** An impressive project
  becomes less impressive when surrounded by superlatives. Let the facts
  carry the weight. If the work is strong, naming it plainly is enough.
- **Trust the reader.** If a sentence explains what the previous sentence
  already implied, cut it. The reader filled in the connection faster than
  the explaining sentence reaches them.

Target lengths as a rough guide (per essay, not per paragraph):

| Essay | Default target | When to go longer |
|---|---|---|
| `why_interested` | 100-150 words | Job description has multiple distinct hooks worth tying to specific projects |
| `why_company` | 100-150 words | The company's product or culture is unusually specific and the alignment deserves more room |
| `technical_project` | 200-300 words | The project has multiple notable architectural decisions and a measurable outcome worth detailing |
| `cover_letter` | 250-350 words | Rarely; a long cover letter is almost always a sign the prose can be cut |

These are guides, not limits. The bar is "every word load-bearing"; if the
essay is at the bar and still longer than the guide, that's fine. If the
essay is at the guide but full of filler, it is too long.

---

## Style Notes

| Preference | Why | What to do instead |
|---|---|---|
| Avoid em-dashes (—) | Recognisable AI tell; reads as machine cadence even when the prose is otherwise good | Commas, semicolons, sentence breaks, or parentheses |
| British English | Caner's spoken voice; CVs and prior cover letters use it | "organisation", "behaviour", "analyse", "centre" |
| Avoid "passionate about", "excited to", "thrilled" | Generic enthusiasm phrases that recruiters skim past | Lead with the technical hook; let the alignment speak |
| Avoid "Dear Hiring Manager" boilerplate | Form-filler tell | Open with the company name or the team name when known |
| Concrete over abstract | Numbers, project names, specific techniques | Replace "extensive experience" with a count, a project name, a measurable outcome |
| Active verbs for solo projects | Avoids overclaiming "led" / "managed" on solo work | Use "built", "designed", "shipped", "maintained" |
| Honest, not modest | Don't fabricate; don't undersell | If a project hit a measurable outcome, name the number |
| No filler or jargon padding | Recruiters skim; padding hides the substance | See the Conciseness section above for the discipline |

---

## Common Factual Phrasings

These are the defaults the prepare-applications skill should reach for when
filling factual answers. Per-job overrides at invocation time supersede them.
Where a phrasing depends on a fact that lives in `profile/` (visa expiry,
graduation year, current employment status), source the fact fresh from those
files; the phrasings below show the *shape*, not the values.

| Question class | Default phrasing pattern | Notes |
|---|---|---|
| Start date | "As soon as possible" | Default while job-hunting full-time with no notice to serve. If a calendar date is required, use today's date or the next Monday. |
| Notice period | "N/A (no current employment)" when independent; the actual notice when employed | Match `profile/experience.md` and `profile/applications.md` |
| Current employer | "Independent" or a fuller form like "Independent; full-time on portfolio engineering and OSS contributions" | Depending on whether the form wants a short or long answer |
| Relocation | "Yes, open to relocation for the right role and city" when open | Lifestyle constraints (city criteria, public-culture preferences, walkability, etc.) live in `profile/lifestyle-preferences.md`; don't repeat them in the answer |
| Sponsorship (now) | Cite the current visa and the date it expires | The "now" framing matters; some forms ask about "now" separately from "in the future" |
| Sponsorship (future) | Cite the specific date sponsorship becomes necessary | Opaque "eventually" reads as evasive |
| Visa status (free text) | Visa name plus the date the visa expires, in a single phrase | Specificity is the load-bearing detail |
| Visa status (dropdown) | Match the form's option exactly | Common variants surface during the autofill schema fetch |
| GPA / degree class | Class plus degree and institution, all in one phrase | Honest; the portfolio is the load-bearing evidence, not the class |
| How heard | "LinkedIn" or "Company careers page" | Pick whichever the form's options list includes |
| US work auth | "Yes" or "No" based on `profile/visa.md` | Only ever asked on US-HQ company forms |
| Interviewed at company before | "No" by default; "Yes" only if explicitly tracked | |
| Data protection / GDPR consent | "Yes" / "Acknowledge" | Always; no exceptions |

---

## Narrative Hooks

Use **one** per application at most. These are designed to land hard once;
repetition dilutes them. Each hook below is a *pattern*, not a script;
instantiate it against the specific role and the current portfolio state.

### The loop-closure hook

When the tool used to find or curate the application is itself a portfolio
project, close the loop explicitly: the agent that discovered this role, the
grader that scored the company, and the system that drafted this answer are
the same artefact. One concrete pointer to the project is enough; the
reader's recognition that "the thing you're reading about is the thing that
surfaced its own application" carries the rest.

When it lands hardest:

- The role is at a privacy, infra, dev-tools, or systems-software company
  where "I built a tool that solves a real problem in my own life" reads
  as cultural fit, not as ornament.
- The cover letter or `technical_project` already has room for one
  long-form project description, and the looped-back tool is the strongest
  match.
- The application form has a "tell us about a time you built something
  for yourself" question.

When **not** to use it:

- The role is at a non-engineering-led firm where the meta-narrative reads
  as showing off rather than as proof.
- The looped-back tool is being mentioned only in passing. The hook needs
  space to land; a half-mention with no follow-through reads as
  name-dropping.
- The essay already includes another strong hook. One per application.

### The architectural-alignment hook

When the company's product commitments (privacy by construction, local-first,
on-prem, deterministic execution, zero-telemetry-by-default, end-to-end
encryption, etc.) match an architectural commitment made in a portfolio
project for reasons of personal discipline rather than external requirement,
name the match directly. The hook is "I made the same commitment without
anyone asking me to". This reads as cultural alignment, not as paid-job
experience, which is often what entry-level applications need to do.

### The "rebuild the legacy app" hook

When the role is explicitly a rewrite-in-Rust (or rewrite-in-Go, rewrite-in-X)
play, lead with a project whose architectural shape ports cleanly to the
migration. The hook is: "I have the discipline that produced [project], and
I have it without the constraint of an existing codebase to migrate; that
discipline ports to the migration, with the added rigour of behavioural-parity
checks against the existing implementation."

### The "I solved this for myself" hook

When the role involves observability, internal tooling, or developer
experience, lead with a project built for personal use. The hook is the
authenticity of the motivation: built because the builder needed it, used
daily, evolves with the user's actual workflow. The story should be tight:
the problem, the build, the daily use.

### The OSS-as-credibility hook

When the role asks for "open-source experience" or names specific upstream
projects, surface the relevant OSS engagements with specifics: the project
name, the PR number, the LOC, the merged status, the verification artefact.
The hook lands when the contribution is non-trivial; a one-line typo fix
weakens the signal rather than strengthens it.

---

## How to Pick What to Lead With

The Answer Generation Standard requires every essay to be profile-grounded;
this section is the *judgment layer* on top, on how to choose which element of
the profile carries the weight in a given essay.

The agent picks projects, contributions, and skills using these tests:

1. **Architectural-shape match.** Find the project whose architectural
   commitments most directly mirror what the role asks for. If the role
   requires concurrency primitives, find the project that solved a problem
   with custom concurrency. If the role requires data-pipeline work, find
   the project that ingests and processes meaningful volumes of data.
   Architectural-shape match beats stack match: identical languages with
   misaligned problem shapes are weaker than divergent stacks tackling the
   same problem.

2. **Stack overlap.** Once architectural-shape candidates are identified,
   prefer the project whose stack overlaps most with the role's stack. A
   role that names specific technologies (a specific framework, a specific
   database, a specific runtime) gets a project that uses those specific
   technologies wherever the profile has one.

3. **Measurable outcome.** Among shape-matched, stack-matched candidates,
   prefer the project with the strongest measurable outcome (a latency
   number, a throughput number, a memory-footprint number, a test-count
   number, a freeze-eliminated number). Measurable outcomes survive
   skimming; adjective-driven outcomes don't.

4. **Recency.** Prefer the most recently-updated project among shape-matched
   candidates. Recency signals current craft; the same architecture written
   18 months ago and not maintained reads as a one-off rather than as
   evidence of standing practice.

5. **Honest depth.** Discard candidates the agent cannot speak about in
   depth from `profile/projects/`. A project named without the supporting
   architectural detail reads as a CV bullet rather than as evidence; if
   the project file doesn't have the depth to support a paragraph, lead
   with a different project.

Run these tests against the current state of `profile/projects/`,
`profile/skills.md`, and `profile/experience.md`. The selection naturally
changes as the portfolio evolves; the tests don't.

---

## Honest Gap Framing

How to handle questions where the role asks for something the profile
doesn't have. The goal is honest, not modest: name the adjacent experience,
don't claim what isn't there.

The pattern for every gap:

1. **Name the gap directly.** "I have not done X in a paid role."
2. **Cite the closest adjacent experience.** Find the project or
   contribution whose work touches the same concepts even if it doesn't
   touch the same tooling.
3. **Frame the gap as small and specific.** Quantify what you do have so
   the gap is contained. The reader should finish the paragraph thinking
   "this person has the foundation to learn this", not "this person is
   missing the foundation".
4. **Name the closure plan if relevant.** Sometimes "I would treat this as
   a focused first-month learning target" reads as honest planning rather
   than as evasion. Use sparingly.

Apply this pattern to whatever the gap actually is, against the current
profile state. Cloud experience, framework-specific experience, team-size
experience, production on-call experience, years-of-experience: the
framing pattern is the same; the specific adjacent experience is whatever
`profile/projects/` and `profile/experience.md` actually contain.

The boundary: do not invent experience. If the role asks for a technology
the profile genuinely has zero exposure to, say so plainly and lead with a
different strength. A fabricated claim that gets probed in an interview is
catastrophic; an honest gap is normal.

---

## Cover-Letter Rhythm

Defaults for structuring the cover-letter body. Per-job context can break
any of these when the role calls for a different shape.

- **Opening**: technical hook tied to the specific role. The first sentence
  should make it clear *why this role specifically*, not a generic opener.
  Pull a verbatim phrase from the job description and tie it to whatever
  element of the profile best matches.
- **Second paragraph**: the strongest project for this role (chosen via the
  tests in "How to Pick What to Lead With"), described with specific
  architectural decisions and a measurable outcome. Name the project. Name
  the number.
- **Third paragraph**: why this company specifically. Cite what the company
  builds (from `companies.what_they_do`), not generic praise. If a narrative
  hook applies, this is usually where it lands.
- **Closing**: short. Visa / availability if relevant in one sentence. No
  "thank you for your consideration" filler; a clean sign-off is stronger.

Three to four paragraphs total. Dense beats long.

---

## Things Not Covered Here

These are deliberately not in this file:

- **Salary expectations.** Live decision per-job; never written into a
  template or autofilled.
- **Custom answers to one-off questions.** When a form asks something
  genuinely unique ("describe a time you had to fire a customer"), generate
  the answer fresh against the profile; this file's defaults won't help.
- **References.** Handled separately when needed.
- **Project-specific deep-dives.** The per-project content lives in
  `profile/projects/`. This file references the *patterns* of how projects
  get used in applications; the project content itself is sourced fresh
  from `profile/projects/` on every invocation.
- **Specific project names.** By design. Any project named here would go
  stale; the agent picks projects from the live portfolio using the tests
  in "How to Pick What to Lead With".
