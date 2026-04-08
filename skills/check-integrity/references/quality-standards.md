# Quality Standards for Grades and Assessments

> What good grade reasoning and fit assessments look like vs what bad ones look like. Use this to judge whether existing database entries meet the quality bar.

---

## Company grade reasoning

### Unacceptable (fails quality check)

> "Good company, sponsors visas, relevant tech."

**Why it fails:** No specific profile connection. No evidence cited. No boundary reasoning. A reader cannot understand WHY this grade was assigned or how it relates to this specific candidate.

> "Strong engineering reputation. Growing company. Worth tracking."

**Why it fails:** Generic praise that could apply to any candidate at any company. No mention of which profile elements align, no sponsorship evidence, no growth data.

### Acceptable (passes quality check)

> "S-tier. Core product is a distributed time-series database built in Rust — direct overlap with the candidate's Nyquestro (lock-free matching engine) and systems programming depth. Active OSS presence (12 public repos, regular commits). Engineering blog with substantive posts on storage engine internals. Confirmed Skilled Worker sponsor (licence active on gov.uk register). Series B ($45M, 2025) — actively growing, 8 open engineering roles on Greenhouse board. Clear IC ladder visible on Levels.fyi reaching Staff/Principal. A rather than S only because brand recognition is limited outside the database community, which limits CV signal for a first job."

**Why it passes:** Names specific projects. Cites specific evidence (OSS, blog, sponsor register, funding). Explains the grade boundary. A reader understands exactly why this grade was assigned for this candidate.

---

## Company why_relevant

### Unacceptable

> "Found on fintech list. Interesting tech company."
> "Does relevant engineering work."

### Acceptable

> "Infrastructure-focused payments company building real-time clearing and settlement systems. Core engineering challenges — strict transaction ordering, distributed consensus, fault tolerance — are structurally similar to order matching. The candidate's Nyquestro matching engine and distributed systems experience from NeuroDrive map directly to their problem space. Rust not in stack but the systems thinking transfers."

---

## Job fit assessment

### Unacceptable (fails quality check)

> "Good role at a strong company. Decent fit with the profile. Worth considering."

**Why it fails:** Says nothing about what the job actually requires, which profile elements align, what gaps exist, or whether sponsorship is viable.

> "Strong match. Infrastructure role at a top company. Apply."

**Why it fails:** No specific profile connections. No gap analysis. No sponsorship assessment. No career trajectory analysis.

### Acceptable for SS/S (passes quality check)

> "SS. Graduate Infrastructure Engineer at Cloudflare's London office. The role builds and maintains edge network infrastructure handling millions of requests/second — directly aligned with your systems engineering focus. Your Nyquestro matching engine demonstrates the exact lock-free, performance-critical thinking this team values. NeuroDrive's distributed multi-agent simulation shows you can reason about distributed systems at scale. Rust is mentioned as a 'bonus' language in the listing — your primary language and strongest differentiator vs other graduates. Cloudflare is a confirmed Skilled Worker sponsor with an established graduate programme, addressing your Graduate visa timeline (expires Aug 2027 — 15+ months of buffer). Career ceiling is exceptional — clear IC track to Principal Engineer, compensation in this domain reaches your long-term targets. Gap: no production operations experience, but the graduate programme explicitly provides mentorship and structured on-call onboarding. Application narrative: your matching engine + Rust + systems projects make you a standout among graduates who typically bring web application experience."

### Acceptable for A/B

> "A. Backend Engineer, Payments at Form3. Payment message routing involves distributed consensus and strict ordering — technically adjacent to your matching engine work. Go-heavy stack (not your primary language, but systems paradigm transfers). Confirmed sponsor, Series C, growing. Falls short of S because the role description reads mid-level ('2-3 years preferred') — a stretch application. Worth pursuing if S-tier options are thin; your Nyquestro project provides a credible narrative for the seniority gap."

### Acceptable for C/F

> "F. Senior Staff Platform Engineer — hard seniority mismatch. Description requires 'led platform teams of 5+, 8+ years production experience, principal-level architecture ownership.' Not achievable given current experience level per experience.md."

> "C. QA Automation Engineer at a consumer fintech. Testing-only role with no path to SDET or engineering. Career ceiling is low per preferences.toml targets. Company is fine but the role is a dead-end."

---

## How to use this during integrity checks

When auditing existing grades and assessments:

1. Pull a sample of entries from the database
2. Compare each one against the standards above
3. If an entry matches the "unacceptable" patterns → flag for rewrite
4. If it matches "acceptable" → passes
5. If it's borderline → flag with a note about what's missing

The most common quality failures:
- No specific project names from `profile/projects.md`
- No technology overlap analysis from `profile/skills.md`
- No sponsorship evidence (just "probably sponsors")
- No boundary reasoning (why this grade, not the adjacent one)
- No career trajectory assessment
