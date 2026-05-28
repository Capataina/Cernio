# Career Goals

> **Source of truth** for the strategic frame Cernio's grading skills should reason against. Manually maintained; read by `grade-companies`, `grade-jobs`, and `check-integrity` on every invocation. NOT touched by `populate-from-lifeos`.

---

## Long-term goal

Location-independent / contracting / villa-life via **prestige-exit at year 5–8**. Build brand asset at pinnacle employers across the active lanes during the early career window, then exit to independent contracting at £1.5k–£3k/day rates while living anywhere.

This is **Strategy A** of the two strategies discussed in the refactor plan:

- **Strategy A — Prestige-exit then contract.** Accept hybrid/office life during the first 5–8 years; cash the brand asset later.
- **Strategy B — Remote-first from day one.** Skip the prestige lanes; lower comp ceiling; lifestyle available throughout.

Strategy A is the active strategy. Strategy B is documented for reference, not active.

---

## Time horizon

5–10 years before remote-trajectory becomes the primary optimisation. During this window, the lane priority is "land any pinnacle role across the active lanes" — not "optimise for location flexibility".

---

## The eight active lanes

No priority weighting during the junior phase. Any of these eight landing is a major win.

| Lane key | Role-shape (one line) |
|---|---|
| `big-tech` | Generalist software engineering at major tech employers; standard SWE ladder |
| `ai-ml` | ML research and engineering — model training, inference systems, applied ML |
| `hft` | Low-latency systems, market data, exchange connectivity, lock-free engineering at proprietary trading firms |
| `crypto-mm` | Market-making systems and DeFi engineering for digital-asset firms |
| `bank-strats` | Engineering function inside bank S&T divisions — e-trading platforms, strategist tooling, electronic execution |
| `systems-infra` | Low-level systems, compilers, databases, distributed infrastructure, OSS-aligned work |
| `devtools` | Developer tooling and developer-experience products |
| `fintech` | Payments, neobanks, financial-product software at non-bank fintech employers |

**Folded out** (documented so the grading agent doesn't surprise-tag jobs into them):

- AI/ML Research as a separate lane — folded into `ai-ml` (Anthropic Research and Anthropic Eng both live in `ai-ml`)
- Exchange Tech — folded into `hft`-adjacent (LMAX, LSEG appear under `hft`)
- Autonomous Systems / Robotics — companies like Wayve appear under `ai-ml`
- Security / SecOps — declassified entirely (no portfolio anchor; not a target)
- Investment Banking Division (IBD) — not a lane at all (wrong function for engineering profile)

---

## Hard rules

**Role-truth-at-hire (load-bearing).** The role function (engineering / quant / research / strats) must be locked at day one. Vertical climb within function is expected. Cross-function transitions hoping for a lateral hop are auto-downgraded.

Passes:
- Junior SWE → Senior SWE
- Junior Quant Developer (with desk rotations) → Senior Quant Dev
- GS Strats Analyst (title says Analyst; function is engineering from day 1)
- Microsoft AGP / Apple ASE / Google grad SWE (function-locked grad schemes)

Fails (auto-downgrade or auto-reject in `grade-jobs` Q2):
- Solutions Architect hoping to lateral to SWE
- Technical Programme Manager hoping to lateral to SWE
- Sales Engineer hoping to lateral to SWE
- Product Manager hoping to lateral to Eng
- Operations Analyst hoping to lateral to Trader-Tech
- Data Analyst hoping to lateral to Data Engineer
- "Global Markets Graduate" rotation lotteries (function-uncertain)
- IBD Analyst (wrong function for the engineering profile entirely)

**Sponsor-only universe.** Every company in the Cernio DB must sponsor UK Skilled Worker visas. Non-sponsors are rejected at discovery, never enter the DB. `sponsors_uk` is mandatory verified-yes for retention.

**No hardcoded calibration anchors.** No file says "X company is SS in Y lane". Lane-pinnacle positioning emerges from the relativity pass in `grade-companies` Phase 2. `lanes.md` does NOT exist as a separate file — the lane LIST lives here, the lane CONTENT lives in the DB.

---

## Soft preferences

**Location.** London + UK commute-belt within ~2h 45 door-to-door from Highams Park. Specific cities encoded in `preferences.toml`:

| Tier | Cities | Office-frequency rule |
|---|---|---|
| Tier 1 (daily-commute) | London, Reading, Cambridge, Oxford, Milton Keynes, Brighton, Guildford, Watford, Stevenage, Luton, Newbury | Any frequency acceptable |
| Tier 2 (stretch hybrid) | Bristol, Bath, Birmingham, Coventry, Northampton, Sheffield, Nottingham, Derby | ≤2 days/wk acceptable; 3+ downgrade |
| Tier 3 (relocation candidates) | Edinburgh, Glasgow | Treat as relocation-candidate; grade normally with relocation flag |
| Excluded | Manchester, Leeds, Liverpool, Newcastle, Cardiff | Verified over ceiling or thin lane density |

**Remote.** Acceptable for any lane if the role is fully UK-remote with sponsorship.

**Hybrid-unspecified at Tier 2** → optimistic-accept (treat as ≤2 days/wk); `grade-jobs` agent reads the JD for office-day signals and downgrades if 3+ days mentioned.

---

## What the grading rubric reads from this file

- The 8 active lanes (the universe of valid lane assignments)
- The role-truth-at-hire hard rule (Q2 of grade-jobs rubric)
- The sponsor-only universe rule (filter at discovery + Q5 of grade-jobs)
- The location tiers and office-frequency rule (Q5 of grade-jobs)
- The strategy frame (informs Q3 within-lane pinnacle position via post-exit-leverage)

---

## What is NOT in this file

- Specific company pinnacle anchors — those live in the DB (`companies.pinnacle_status_per_lane`) and emerge from relativity grading
- Lane priorities / weights — junior phase has none
- Time-window-bound deadlines (Arm Cambridge SS, etc.) — handled by skills external to Cernio
- Interview prep per lane — handled by a separate project, not Cernio

---

**Last updated:** 2026-05-28 (lane-based-relativity refactor)
