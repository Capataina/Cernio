# Application Voice: Idea Sheet

> [!note] What this file is
> **Reasoning for agents, not rules.** Every observation explains *what a skimming reader actually experiences* when they hit a particular shape of prose, and names the *variables* that shape the right call in any given application. The agent reads the profile, the role, the company, and the form, then decides. The patterns here are inputs to that judgment, not constraints on it.

> [!important] Portfolio-agnostic by design
> This file names **zero** specific projects, technologies, or contributions, because the portfolio evolves. The agent picks from the live state of `profile/projects/`, `profile/skills.md`, and `profile/experience.md` using the reasoning here. As projects change, this file stays accurate.

---

## 1. Length, Density, and How Many Projects

Two forces compete in every essay:

| Dimension | What it gives the reader | What it costs |
|---|---|---|
| **Breadth** | Signal of range, versatility, multiple proof points | Each item less vividly rendered |
| **Depth** | Vivid evidence the reader can quote back | Less ground covered |

Neither is the default. The right answer is contextual, and four variables determine it:

### 1.1 Alignment density

The single biggest driver. How densely the profile maps onto the role being applied to:

| Alignment shape | Strongest move | Why |
|---|---|---|
| Many portfolio items each map to different role responsibilities | **Breadth at moderate depth** (3-4 projects, tight) | The range itself is the evidence; covering ground demonstrates versatility one deep project can't |
| One project hits the heart, others are adjacent | **Depth on the one project** | A bull's-eye is more memorable than a constellation; pad it with one or two supporting names at the close |
| Plausible fit, no single project on target | **Shorter and more concise overall** | Reduces the risk of reading as overclaim; honesty about partial fit beats forced enthusiasm |
| The role is a genuinely perfect fit, multiple projects all hit it | **Boast deliberately** | Specificity-stacked confidence is appropriate when earned; the reader can feel when an applicant is genuinely the candidate vs. fishing |

### 1.2 Role framing

The job description usually telegraphs what it wants the candidate to show:

- **Senior-track wording** ("ownership", "drives initiatives", "across the stack") rewards **range**. Show breadth.
- **Entry-level specialist wording** ("strong Rust", "deep familiarity with X") rewards **depth in one thing**. Show focus.
- **Generalist wording** ("comfortable across", "polyglot mindset") rewards **breadth shown lightly**. Touch several things briefly.

### 1.3 Essay's natural length

The form's slot shape itself shapes the answer:

| Slot | Natural fit |
|---|---|
| One-paragraph textarea | One project, in depth |
| Two-or-three paragraph essay | One project deep, one supporting |
| Full cover-letter body | Two or three projects, varied depth |
| Multi-section answer set | Different projects can carry different essays |

Forcing four projects into a one-paragraph slot makes each shallow. Stretching one project across four paragraphs makes the prose feel padded.

### 1.4 Headline strength of each project

Some projects carry an essay on a single fact (a sharp latency number, a striking architectural decision, a measurable outcome stated plainly). Others are individually solid without being individually striking; those projects work better in concert than in isolation.

### 1.5 Sanity-reference word counts

Strong essays in this voice tend to land in the ranges below. The right essay can sit outside them when the content earns it:

| Essay | Typical range | When longer is right | When shorter is right |
|---|---|---|---|
| `why_interested` | 100-200 words | Multiple distinct hooks in the role each tie to different projects | One specific hook does most of the work cleanly |
| `why_company` | 100-200 words | Company has unusually specific commitments worth engaging with | "Good company doing good work" is honestly most of what there is to say |
| `technical_project` | 150-350 words | Project genuinely has multiple architectural decisions and a measurable outcome worth detailing | Padded depth reads worse than honest brevity |
| `cover_letter` | 200-400 words | Multiple projects each load-bearing on different parts of the role | Recruiters skim cover letters fastest; the case for length has to be earned |

---

## 2. What Makes Prose Land or Slide Off

A skimming reader's eye moves through an essay scanning for friction points: places where the prose stops feeling like a specific person saying a specific thing. Each observation below names the friction point, explains the mechanism, and notes when the friction matters most.

### 2.1 Em-dashes

> [!warning] Recognisable AI cadence
> Models produce a particular rhythm (short clause, dash, longer clarifying clause) often enough that human readers register it as a tell, even without consciously naming it.

| Mechanism | When it matters most | When it matters least |
|---|---|---|
| Statistical signature of model-generated text | AI-suspicious audiences, application portals at AI-adjacent companies, anywhere recruiters have screening fatigue | Internal-team-written form where the reader knows the candidate already |

The substitute is usually commas, semicolons, or sentence breaks. The information rarely depends on the dash; the cadence does.

### 2.2 Generic enthusiasm phrases

> [!warning] Filler that pushes evidence out
> "Passionate about", "excited to", "thrilled to be considered". These convey only the writer's emotional posture, which the reader assumes anyway from the fact that the application exists.

**The real cost is what they push out.** A sentence about emotion is a sentence that didn't make the case. The reader's eye lands on it, gets nothing actionable, and moves on slightly more sceptical.

### 2.3 Boilerplate openings

"Dear Hiring Manager" and similar generic salutations signal that the same opening went out to every other application. The cost is small but real: the reader unconsciously categorises the application as templated and reads the rest with that frame.

### 2.4 Adjectives without evidence

"Highly performant", "robust", "production-grade", "extensive experience". These are claims the reader has to take on trust. The same information is usually shorter expressed as evidence:

| Adjective | Evidence equivalent |
|---|---|
| "Highly performant" | A latency or throughput number |
| "Robust" | A test count, an uptime number, a failure-mode handled |
| "Production-grade" | The named users, the named scale, the named system |
| "Extensive experience" | A named project, a named contribution, a named outcome |

Evidence is shorter than the adjective it replaces, and harder to disbelieve.

### 2.5 Solo-work overclaim verbs

"Led", "managed", "directed" applied to a project the candidate built alone can read as overstatement once the rest of the application makes the solo nature clear. Active building verbs ("built", "designed", "shipped", "maintained") read as accurate and stronger for it.

### 2.6 British vs American English

Consistency matters more than the specific choice. Caner's spoken voice and CV are British English; mixing within a single application reads as careless or templated.

### 2.7 Honest, not modest

> [!tip] The asymmetry
> Underselling a real achievement reads as **hedging**. A measurable outcome stated plainly reads as **confidence**. Claiming something that isn't there reads as **fabrication**, and the reader can spot it faster than the writer thinks.

### 2.8 No fawning, no pleading

> [!warning] The single biggest tonal trap in cover letters
> Echoing the company's own marketing language back to them, name-dropping scale or pedigree as if reverent of it, and praising the company as exceptional reads as **performative impressment**. Recruiters see this on every templated application; it is the loudest "this candidate is desperate" signal in the genre.

What the trap looks like in practice (each is a real cover-letter-killer pattern):

| Pattern | Why it lands wrong |
|---|---|
| "X is the rare company where..." / "X is exceptional because..." | Reads as if the writer needs to convince themselves; recruiters know it's their company and don't need to be sold on it |
| Citing user counts, employee counts, valuation, fundraising totals | Numbers the reader already knows. Quoting them back signals nothing except that the writer reads the about page |
| "Founded by [pedigree institution]" or other origin-story name-drops | The reader doesn't need their own backstory recited. It positions the writer below the company, not alongside it |
| "I would be honoured to..." / "Thrilled at the opportunity..." | Begging-coded; even when sincere, it shifts the dynamic from peer-to-peer toward supplicant-to-gatekeeper |
| "X is the production version of what I've been practising" | Self-aggrandising disguised as flattery. Reads as the writer comparing themselves favourably to the company |
| Repeating the company's own marketing phrases ("first principles", "mission-driven", "psychologically safe environment") | The reader wrote those phrases; hearing them back reads as parroting rather than analysis |

The underlying principle: **treat the company as a peer doing interesting work, not a benefactor to win over.** Confidence comes from grounding the application in shared technical interest, not in performative awe. A good test: would the same sentence make sense if the company was a friend's side project rather than a famous brand? If yes, it's grounded. If it depends on the brand's prestige to land, it's fawning.

What replaces fawning:

- **State technical alignment as fact, not as wonder.** "Proton's architecture is the kind I'd build for myself" carries the same information as "Proton is the rare company where..." but lands as a peer-to-peer observation.
- **Acknowledge interesting work without praising the company.** "Cross-platform Rust client preserving end-to-end encryption is the kind of project I'd want to work on regardless of who built it" lands stronger than "Proton's mission inspires me."
- **Skip pedigree references entirely.** If a candidate is genuinely good fit, the application stands without crediting CERN or Stanford or YC. If the candidate isn't a fit, namedropping won't fix it.
- **Avoid claiming you've been "preparing" for this exact role.** Even when true, it reads as backwards-projecting. State the projects, let the alignment speak.

---

## 3. What a Skimming Reader Actually Does

> [!important] The bedrock observation
> A recruiter reading the twentieth application of the morning has finite attention. They start each essay looking for a reason to keep reading. They abandon the moment the prose stops earning that.

```
        ┌──────────────────────────────────────────────┐
        │  Application N of 20 this morning            │
        │  ~8 seconds on the opening                   │
        │  ~30 seconds total if the opening lands      │
        │  Either way: filler costs more than length   │
        └──────────────────────────────────────────────┘
```

What this implies:

- **The first sentence has to land.** Naming the specific role, pulling a verbatim phrase from the description and tying it to evidence, or opening on a concrete detail that signals "this is not templated" all work.
- **Filler costs more than length.** A 400-word essay where every sentence is load-bearing reads faster than a 250-word essay padded with warm-up sentences. The reader's experience is "the prose got me through it", not "the word count was low".
- **Specifics teach faster than generalities.** One specific fact (a number, a PR with merged status, a named architectural decision) teaches the reader more than a paragraph of capability claims.
- **Trust the reader.** If a sentence explains what the previous sentence already implied, the reader felt the explanation coming. Cut it; they filled in the connection faster than the explaining sentence could reach them.

---

## 4. Common Factual Phrasings

Defaults for the factual half of the application. Values are sourced fresh from `profile/` files; the phrasings below are *shapes* that tend to land cleanly. Per-job context can override any of them.

| Question class | Phrasing shape | What's underneath |
|---|---|---|
| **Start date** | "As soon as possible" (or a calendar date when required) | Reads as available and motivated |
| **Notice period** | "N/A (no current employment)" when independent | Exact-match to the form's expected shape; vagueness reads as hiding something |
| **Current employer** | "Independent" or a fuller form when the role values self-direction | Match the field's tone; one-word fields don't want a paragraph |
| **Relocation** | Direct yes/no with a brief qualifier | "Yes, for the right role and city" reads as deliberate; bare "yes" can read as desperate |
| **Sponsorship (now)** | Visa name plus exact expiry date | Specificity is what the reader needs to do hiring math |
| **Sponsorship (future)** | The exact date sponsorship becomes necessary | Opaque "eventually" reads as evasive even when it isn't |
| **Visa status (free text)** | Visa name plus expiry date in one phrase | The visa name is load-bearing; the date confirms specificity |
| **Visa status (dropdown)** | Exact match to the form's option | Custom phrasing in a dropdown breaks downstream filters |
| **GPA / degree class** | Class, degree, and institution in one phrase | The portfolio is the evidence; the class is the receipt |
| **How heard** | "LinkedIn" or "Company careers page" | Match the form's option list |
| **US work auth** | "Yes" or "No" from `profile/visa.md` | Only asked on US-HQ company forms |
| **Interviewed at this company before** | "No" by default | Honest by construction |
| **Data protection / GDPR** | "Yes" / "Acknowledge" | Always; downstream logic depends on it |
| **Salary expectation** | £35,000 by default; a sensible figure anchored at £35k as the floor when the JD signals a higher band; never priced too high | Most salary fields are optional; when the form makes the field mandatory, a concrete figure beats a blank. The £35k floor is the entry-level reference point Caner is comfortable with; pricing too high risks being filtered out by automated screens |

---

## 5. Narrative Hooks

> [!tip] One per application, at most
> A narrative hook is a screenshot-worthy line, something a recruiter might quote back in the first conversation. **Hooks land hard once and dilute fast.** Two hooks in the same application make each one feel less earned.

Each pattern below is a *shape*, not a script. Whether to use it, where to land it, and what specific portfolio item to instantiate it against are all judgment calls.

### 5.1 The loop-closure hook

**The move.** When the tool used to discover or curate the application is itself a portfolio project, naming the loop explicitly turns the project from a CV bullet into demonstrated problem-solving.

**Why it lands.** The reader's recognition does the work: *"the thing you're reading about is the thing that surfaced its own application"*. Satisfying in a way a normal project description isn't.

**When it dilutes.** When the looped-back tool is mentioned only in passing, or when the audience values traditional credentials over self-built tooling.

### 5.2 The architectural-alignment hook

**The move.** When the company's product commitments (privacy by construction, local-first, deterministic execution, zero telemetry, end-to-end encryption) match a commitment made in a portfolio project for *personal* reasons rather than for any external requirement, the alignment itself is the hook.

**Why it lands.** Reads as *"I made the same choice without anyone asking me to"*, which is harder for an entry-level candidate to fake than paid experience.

**When it dilutes.** When the alignment is loose; when the personal-reason framing has to be invented rather than honestly named.

### 5.3 The legacy-rewrite hook

**The move.** When a role is explicitly a rewrite-in-X play, lead with a project whose architectural shape ports cleanly to the migration.

**Why it lands.** Positions the candidate as already at the destination: *"I have the discipline that produced this, without the constraint of an existing codebase to migrate"*.

**When it dilutes.** When the rewrite framing is incidental rather than central to the role.

### 5.4 The "I solved this for myself" hook

**The move.** When the role is in observability, internal tooling, or developer experience, lead with a project built for personal use and lived with daily.

**Why it lands.** Carries authenticity that nothing else does. The reader can feel whether the prose is describing a tool the writer actually uses or one they once shipped and forgot.

**When it dilutes.** When the personal-use framing is fabricated; the reader's authenticity-detector is sharper for this hook than for any other.

### 5.5 The OSS-as-credibility hook

**The move.** When the role names specific upstream projects or asks for "open-source experience", surface relevant OSS engagements with specifics: PR number, LOC, merged status, verification artefact.

**Why it lands.** Specifics turn claim into evidence the reader can verify in 30 seconds.

**When it dilutes.** When the contribution is small enough that naming it feels strained.

---

## 6. How to Pick What to Lead With

> [!note] Five inputs the agent weighs together
> None of these tests is a rule on its own. The right project (or set of projects) emerges from holding them in tension.

| # | Test | What it asks | Why it matters |
|---|---|---|---|
| 1 | **Architectural-shape match** | Which project's commitments most directly mirror the role's? | The reader is hiring for the kind of thinking the project required, not for line-by-line tool overlap |
| 2 | **Stack overlap** | Which project's stack overlaps most with the role's? | Useful when the description leans heavily on specific technologies; weaker signal than shape match |
| 3 | **Measurable outcome** | Which project has the strongest number to anchor on? | Numbers survive skimming; adjectives don't |
| 4 | **Recency** | Which project is most recently maintained? | Recent work signals current craft; year-old projects with no updates read as one-offs |
| 5 | **Honest depth** | Which projects can the agent actually describe in paragraph-depth from `profile/projects/`? | A project named without supporting architectural detail reads as a CV bullet, not as evidence |

The selection naturally changes as the portfolio evolves; the reasoning doesn't.

---

## 7. Honest Gap Framing

> [!warning] The asymmetry of approaches
> The way a gap is handled matters more than the gap itself.

| Approach | Reader's reaction | Upside | Downside |
|---|---|---|---|
| **Named directly + adjacent experience cited** | "Self-aware and grounded" | Small but real | Small |
| **Hedged behind soft language** ("familiar with", "exposure to") | "Evasive, probably worse than they're saying" | Small | Moderate |
| **Fabricated as filled** | "...until probed in interview" | Small | Career-ending |

The variables that shape *how* to frame a particular gap:

| Variable | When the gap is small | When the gap is large |
|---|---|---|
| **Closeness of adjacent experience** | Cite the adjacent work; frame the gap as specific | Honesty has to carry it alone |
| **Importance of the gap to the role** | Acknowledge in passing | Pair with a closure plan ("a focused first-month learning target") |
| **How honestly the role asked** | Wish-list role: gaps weigh light | Tight role asking only what they need: each gap weighs heavier |

---

## 8. Cover-Letter Rhythm

A strong cover letter has a recognisable shape, not because the shape is mandatory but because each beat does specific work the reader processes faster than other arrangements:

```
┌───────────────────────────────────────────────────────────────┐
│  Beat 1: Opening                                              │
│  Why this role specifically, in the first sentence            │
│  ─────────────────────────────────────────────────────────    │
│  Beat 2: The technical middle                                 │
│  Project(s) at depth chosen via §1 reasoning                  │
│  ─────────────────────────────────────────────────────────    │
│  Beat 3: Why this company specifically                        │
│  Cite what they actually build; hook lands here if at all     │
│  ─────────────────────────────────────────────────────────    │
│  Beat 4: The close                                            │
│  Visa / availability in one sentence; clean signoff           │
└───────────────────────────────────────────────────────────────┘
```

**Beat 1 (Opening).** Has to make clear *why this role specifically* within the first line. Generic openings signal templated origin; the reader's investment in the rest drops. Pulling a verbatim phrase from the description and tying it to evidence is one strong move; opening on a concrete detail that signals substantive body is another.

**Beat 2 (Technical middle).** Where projects and the strongest evidence land. §1 determines whether this is one project deep, two at moderate depth, or three or four covered lightly.

**Beat 3 (Why this company).** Often shorter inside the cover letter than as a standalone essay, because the letter has already spent space on evidence. Citing what the company actually builds (from `companies.what_they_do`) carries weight; generic praise doesn't. Narrative hooks usually land here.

**Beat 4 (Close).** Visa or availability or location specifics in one sentence when relevant. Filler thank-yous subtract more than they add; a clean name signoff reads stronger.

The letter is doing its job when the reader finishes it knowing one or two specific things they could quote back, rather than a vague sense of *"competent application"*.

---

## 9. What This File Doesn't Cover

| Out of scope | Why | Where it lives |
|---|---|---|
| Custom one-off form questions | Generate fresh against the profile | Not predictable in advance |
| References | Handled separately when needed | Out-of-band |
| Project architectural detail | Goes stale fast; lives where the truth is | `profile/projects/` (sourced fresh) |
| Specific project / tech / contribution names | Would go stale as the portfolio evolves | The agent picks from `profile/` using §6 |
