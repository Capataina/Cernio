---
title: Portfolio Gaps — HFT
lane: hft
last_updated: 2026-06-01
seed_source: grade-jobs Phase 3 run 2026-06-01-121446 (217 hft-lane jobs graded)
distribution_snapshot: "S: 10, A: 9, B: 42, C: 85, F: 71"
---

# Portfolio Gaps — HFT lane

Synthesised from the 217-job hft-lane batch graded on 2026-06-01. The lane is structurally narrow-funnel: the C tier is 85 jobs (39%) and the F tier 71 (33%), almost entirely driven by Q1-headwind reasoning at pinnacle firms (Jane Street, HRT, Citadel, Optiver senior bands) plus quant-research / FPGA / kdb+/q stack-prerequisite walls. The S/A pool of 19 (8.8%) clusters at DRW (incl. Cumberland), QRT internships and SWE seats, Tradeweb internships, Verition's Junior Rust seat, and Vitol commodities desk-dev — i.e. wide-funnel grad/intern pipelines plus Rust-specific seats where Nyquestro is the deciding anchor.

## Open gaps

Ordered by how often the gap bit a grade across the 217-job batch.

### 1. C++ Familiar → Proficient (lane-blocking, recurring on 30+ roles)

The single most-cited friction across the lane. Cited explicitly in C / F verdicts for HRT C++ SWE, Citadel Securities C++ SWE, Tower-equivalent C++ seats, Old Mission C++ SWE, QRT Quant Developer HFT (C++), QRT Low Latency Market Data Developer (C++), QRT Market Access Developer (C++), Aquatic Capital C++ Market Data, Squarepoint Junior C++ SWE, LMAX Software Test Engineer (C++), Cumberland Trading Systems Engineer (named as the residual friction at S). Profile says C++ at **Familiar** anchored on `projects/chrona.md` (status:paused) + `projects/tectra.md` (status:dormant, 6-day C++20 scaffold). The portfolio has no shipped C++ system; Rust-to-C++ translation is undemonstrated. The gap caps every C++-mandated seat at B-or-below regardless of how well the concept-fit reads on Nyquestro.

### 2. FIX protocol fluency — absent

Cited as a structural prerequisite at Tradeweb, LMAX, Squarepoint, Tower, Maven Securities and several QRT market-access roles. The portfolio has no FIX 4.2/4.4/5.0 / SBE / OUCH / ITCH implementation. Nyquestro's Coinbase Advanced Trade WebSocket integration is the closest analogue — it demonstrates *protocol decoding + idempotent state translation* (the `HashMap<(Symbol, Side, Px), OrderID>` virtual-order map), but it is a JSON/WebSocket feed, not a FIX session. Bank-strats and exchange-tech firms (Tradeweb, LMAX, ICE) read this gap as a hard stack prerequisite for market-access seats.

### 3. kdb+/q — absent (hard gate at Squarepoint, Citadel, Jane Street, HRT, GSA quant-research)

Cited verbatim as the deciding C-tier friction on Squarepoint Junior Software Developer (KDB+/Q), Point72 Quant Software Developer Intern (KDB+/Q in some teams), GSA Capital Software Developer (KDB+/Q-or-Python framing), QRT Quantitative Developer HFT. The skill is a domain-specific time-series language used heavily in tick-data and quant-trading shops; conversion at firms that screen on KDB without prior exposure is sub-1%. No KDB exposure in `profile/skills.md` and no per-project anchor in `projects/`.

### 4. Kernel-bypass / low-latency networking — absent

Cited at HRT Trading Systems Engineer ("kernel-level tuning and network optimization", "every single nanosecond counts"), Optiver Senior C++ SWE Digital Assets ("Linux internals, kernel bypass and low-level profiling / tuning"), QRT Low Latency Market Data Developer, IMC Trading C++ SWE. The portfolio has zero io_uring / AF_XDP / DPDK / Solarflare onload / ef_vi / OpenOnload evidence. `lane-affinity.md` hft section names this gap explicitly; the 2026-06-01 batch confirms it bit at ~6 C-tier and 3 F-tier roles.

### 5. FPGA / hardware acceleration — absent (Jane Street, HRT, Optiver senior bands)

Cited at Jane Street FPGA Engineer (C, OCaml + HardCaml + Verilog requirement), HRT Hardware Design Engineer (F-tier, "Engineers build automated trading algorithms in C++ with deep OS, CPU, and networking knowledge ... FPGA and ASIC accelerators for nanosecond-precision trading"), Optiver senior FPGA-adjacent bands. Zero Verilog / HDL / Hardcaml / Vivado evidence. This is closure-unfeasible inside a candidate budget — see Closure prescriptions below for why it stays open as "non-pursued, not non-closed".

### 6. Quant-research credential floor — non-closable in the lane

Cited at every F-tier Quantitative Researcher / Quantitative Analyst row (DRW Quant Researcher Commodities/Equities/ML, HRT Algo Developer & Algorithm Developer (PhDs), IMC Quant Researcher, Capstone Quant Analyst, BlueCrest Quant Researcher, Aquatic Quant Researcher Intern, GSA Quant Researcher, Capula 2027 T&R Internship — 15+ rows). These roles screen on PhD-or-equivalent in maths/stats/physics + competitive-programming pedigree; BEng (2:2) York is structurally outside the realistic primary-target pool. **This is correctly an F across the lane** — not a portfolio gap to close, but a lane-structure to respect when sizing the realistic pool.

### 7. ITCH / OUCH / SBE / FIX-engine exchange protocols beyond FIX

Distinct from #2 (FIX as a session protocol). ITCH (NASDAQ market data) and OUCH (NASDAQ order entry) appear at Tower, LMAX, IMC, HRT Market Structure roles; SBE (Simple Binary Encoding, CME) at QRT C++ Market Access. Zero exchange-protocol implementation in the portfolio. The Coinbase WebSocket bridge is the closest analogue but does not substitute on the protocol-engineering axis.

### 8. Multicast / UDP feed handling — absent

Mentioned at Ansatz Capital ("TCP/multicast handlers"), QRT Low Latency Market Data, several HRT data-production roles. Nyquestro's WebSocket bridge is TCP-only; no UDP multicast feed-handler code. Lower-frequency citation than FIX/kdb+ but consistently appears as a soft prerequisite at C++-mandated market-data seats.

### 9. Options Greeks / vol-surface / pricing-library work — absent

Cited at DRW Equity Index Options SE (A-tier, named as the residual domain-depth gap), Optiver C++ SWE Options, Maven Securities Options-adjacent, several IMC roles. Aurix's Q64.96 fixed-point + LP backtest is *math-adjacent* but is concentrated-liquidity AMM pricing, not Black-Scholes / SABR / local-vol surfaces or first-second-order Greeks. The gap caps options-desk roles at A and excludes the candidate from senior-options seats.

### 10. Microstructure beyond simulator-scale

Nyquestro implements `microprice()`, `ofi(n)`, `spread_cents()`, `depth(n)`, `top_n_bids/asks(n)` — solid junior microstructure surface. But it is single-venue, single-feed (Coinbase), with no cross-venue arbitrage routing, no STP (self-trade prevention), no journal, no risk guard, no market-making strategy. QRT Trading Systems Reliability Engineer, Cumberland Mining, and IMC mid-band roles cite this as the "next-step" gap that separates a strong portfolio anchor from a hireable mid-band candidate.

### 11. Published latency benchmarks — absent

Nyquestro has HDR histograms per `Op::Submit/Match/Cancel` but no `criterion`-based benchmark harness, no claimed sub-µs floor, no published blog post or README graph. HRT Junior Quantitative Latency Engineer (F), Optiver Software Engineer roles, and Cumberland Trading Systems Engineer all read latency-credibility through *published* numbers. This is the single highest-leverage closure item under #12 — see Closure prescriptions.

### 12. OCaml — absent (Jane Street-blocking)

Jane Street builds nearly all software in-house in OCaml; their FPGA work uses Hardcaml (an OCaml HDL DSL). Zero OCaml in the portfolio. Acts as a stack-prerequisite filter at Jane Street's SWE / FPGA / Strats roles, contributing to the 5+ Jane Street rows landing at C/F.

## Confirmed strengths

These are what kept Nyquestro+Aurix+Cernio as the deciding anchors on the S/A roles in this batch.

| Strength | Evidence | Roles where it bit (positively) |
|---|---|---|
| **Deterministic LOB matching engine in safe Rust (zero `unsafe`)** | `projects/nyquestro.md` status:active — `BTreeMap<Px, PriceLevel>` ladders + `VecDeque<Order>` FIFO, four-phase `submit_limit`, byte-deterministic engine output across runs | DRW Market Data SE (S, id=207), DRW Cumberland Trading Systems Engineer (S, id=217), Cumberland Trading Systems Engineer (S, id=1204), QRT SWE intern (S, id=768), QRT Quant Dev intern (S, id=766), Verition Junior Rust (S, id=1343), Ansatz Capital SWE London (B but cleanest hft concept-fit in batch) |
| **Live exchange-feed integration (Coinbase Advanced Trade level2 WebSocket) + idempotent L2-to-virtual-order translation** | `projects/nyquestro.md` — `HashMap<(Symbol, Side, Px), OrderID>` virtual-order map; tokio-tungstenite + native-tls | DRW Market Data SE (S), QRT SWE Distributed Market Data Systems Python (S, id=809), Optiver Digital Assets (A, id=671) |
| **HFT-style observability / tail-latency tracking** | `projects/nyquestro.md` — per-op HDR-histogram p50/p95/p99/p999/p9999/max across `Op::Submit/Match/Cancel`; `lane-affinity.md` hft cross-cutting skill at Comfortable | Cumberland Trading Systems Engineer (S), Tradeweb internships (S/A), Tradeweb SRE/BI Engineer intern (A, id=1257) |
| **Backpressure as structural property** | `SNAPSHOT_LEVEL_CAP=50` + `PER_FRAME_BUDGET=500` two-pronged defence per `lane-affinity.md` hft section; concepts-domains.md Comfortable band | QRT SWE Data Core (S, id=805), Cumberland (S, id=1204) |
| **Microstructure inspection surface** | `microprice()`, `ofi(n)`, `spread_cents()`, `depth(n)`, `top_n_bids/asks(n)` per `projects/nyquestro.md` | DRW Prediction Markets SE Python (A, id=208), DRW Equity Index Options SE (A, id=206) |
| **Crypto-mm domain depth: Q64.96 fixed-point + cross-DEX arbitrage + LP backtest** | `projects/aurix.md` status:active — clean-room V3 math port on `BigUint`, 5 chains × 3 protocols, LVR discrete approx | DRW Cumberland (S, dual hft+crypto-mm pinnacle), Cumberland Trading Systems Engineer (S), Optiver Digital Assets (A) |
| **Production-shaped async Rust + Tokio (proficient)** | `projects/cernio.md` status:active — Tokio runtime + multi-provider parallel fetch + shared retry infrastructure; 14k LOC | QRT SWE Data Core (S), QRT SWE Distributed Market Data Systems (S), Verition Junior Rust (S) |
| **Rust [Proficient] across 9 active projects** | `projects/index.md` — Cernio, Image Browser, Aurix, NeuroDrive, Nyquestro, Tessarix, Performance Profiler all active; Vynapse paused; Xyntra/Zyphos dormant | Verition Junior Rust (S, only Rust-primary HFT seat in 200+ batch — direct hit) |
| **Risk modelling / statistical awareness** | `projects/aurix.md` — adaptive-tercile vol-regime classifier, multi-variant capital-allocation verdict, Bailey/de Prado Deflated Sharpe acknowledged | Tower-equivalent risk-tooling seats, DRW Python Global Delta One (A, id=209), Verition Fixed Income Python (A, id=1344) |
| **Active markets / live data delivery** | Tectra status:dormant (C++ trading-infra scaffold with virtual-clock-first ordering), plus Nyquestro's live Coinbase feed | Cited as supporting evidence on most S/A rows |

## Closure prescriptions

Ranked by hft-lane leverage per unit of work, weighted by the 2026-06-01 batch frictions.

### Priority 1 — Ship a FIX 4.4 exchange adapter on Nyquestro

A minimal FIX 4.4 session-and-application layer feeding Nyquestro's matching engine. Concrete deliverable: a `nyquestro-fix-adapter` sub-crate that:

- Implements FIX 4.4 session layer (logon, heartbeat, sequence-number management, resend request, logout) against a public test server (e.g. QuickFIX/n test acceptor, or CME iLink test environment if reachable).
- Translates NewOrderSingle (D) / OrderCancelRequest (F) / OrderCancelReplaceRequest (G) into Nyquestro's `submit_limit` / `cancel` API.
- Emits ExecutionReport (8) back through the session.
- Adds a README section with a sequence diagram of a full order lifecycle.

**Why this closure**: it closes gaps #2 (FIX), #7 (exchange protocols), and partially #4 (real production-shaped protocol engineering). It would bit at Tradeweb, LMAX, Tower, QRT Market Access, Cumberland — at least 8 roles in this batch would re-grade up by one tier with this anchor present. Higher leverage than the C++ closure because it lands a brand-new portfolio anchor on the dominant gap.

### Priority 2 — Take Tectra past the Clock-interface scaffold into a working feed-handler + matching loop in C++

Tectra is currently 6 days of work; only the `Clock` interface and `RealClock` / `VirtualClock` exist. Push it to:

- A pcap or UDP-multicast feed handler that parses a public market-data format (ITCH 5.0 sample data is freely available from NASDAQ TotalView).
- A minimal price-time matching engine in C++20 that consumes the parsed feed.
- One published `criterion`-equivalent C++ benchmark (Google Benchmark) showing per-op latency percentiles.

**Why this closure**: it directly moves C++ from Familiar → Proficient via demonstrable production-shaped code (closing gap #1, which bit 30+ times in this batch), partially closes gap #7 (ITCH), and produces gap #11 (published latency benchmarks). Combined with Priority 1, this would unlock the entire C++-mandated mid-band cohort (QRT C++ seats, Citadel Securities C++, Tower, Old Mission C++, Aquatic C++ Market Data) — currently the largest single block of B/C downgrades.

### Priority 3 — Publish a lock-free queue / matching-engine benchmark blog post

Direct closure of gap #11 (published latency benchmarks). Concrete deliverable: a public blog post (or README + GitHub Pages site) with:

- `criterion`-based benchmark harness for Nyquestro's `submit_limit` showing p50/p99/p999 latency across realistic load shapes.
- A SPSC / MPSC lock-free queue benchmark (the lane-affinity.md gap "no actual lock-free order book yet" is closed when this lands), with comparison against `crossbeam-queue` and `flume`.
- One graph per metric, raw numbers in a table.

**Why this closure**: builds the latency-credibility credential that HRT Junior Quantitative Latency Engineer, Optiver SWE roles, and Cumberland Trading Systems Engineer all implicitly screen for. Low marginal effort (Nyquestro already has HDR histograms instrumented) and high-leverage signal.

### Priority 4 — kdb+/q minimum-viable exposure

Not "become a KDB developer" — that's a multi-year investment with narrow lane reach beyond the specific Squarepoint / Jane Street / GS quant-research seats. Instead: a small public artefact (e.g. a tick-data analysis notebook against a public dataset, or a 200-line q script implementing a TWAP/VWAP calculator) sufficient to add KDB+/q [Familiar] to skills.md. Closes gap #3 from "hard zero" to "Familiar" for the ~5 KDB-naming roles per batch. Low priority because KDB-mandated roles are a narrow slice; not zero priority because the marginal cost is small and KDB exposure is a known door-opener.

### Priority 5 — Options pricing-library micro-project

Closure of gap #9. A small `options-pricing` crate (Rust or C++) implementing Black-Scholes, first-order Greeks (delta, gamma, vega, theta), and one numerical method (Newton-Raphson for implied vol). Anchors a credible options-desk story when applying to DRW Equity Index Options SE, Optiver C++ Options, Maven options-adjacent seats. Medium leverage — options-desk roles are real but not the bulk of S/A; would move 2-3 batch rows up a tier.

### Non-pursued, by deliberate choice

- **FPGA / Hardcaml / Verilog** (gap #5). Closure cost is months of HDL learning, target hires are Jane Street + HRT senior hardware bands which sit at C / F regardless. Not on the closure path.
- **OCaml** (gap #12). Single-firm-blocker (Jane Street). Closure cost not justified by lane reach.
- **PhD-track quant research credentials** (gap #6). Structurally non-closable in the relevant time horizon; the F-tier placement of all quant-research roles is correct lane structure, not a portfolio gap.

## Pinnacle anchors

How the candidate's portfolio reads against the lane's canonical pinnacles.

| Firm | Realistic stance | What maps | What's specifically missing |
|---|---|---|---|
| **Jane Street** | C across the board (single-digit grad intake from Oxbridge/Imperial + comp-prog pedigree, BEng 2:2 York outside the realistic primary-target pool). 5 rows at C/F in this batch confirm the placement. | Nyquestro's safe-Rust systems discipline + matching-engine concept-fit; deterministic engineering shows the *kind* of mind Jane Street values. | OCaml (firm-wide primary), Hardcaml (FPGA team), competitive-programming pedigree, top-university credential signal. Stretch lottery — does not make the budget of 30. |
| **Citadel / Citadel Securities** | C at the SWE bands (id=1192-1198 all C). Narrow-funnel hiring shape; C++-mandate on every Citadel Securities seat. | Cernio's multi-layer Rust architecture + Nyquestro matching-engine. | C++ at Proficient (currently Familiar — gap #1), Slang / internal cpp DSLs, comp-prog screening readiness. Stretch lottery. |
| **HRT** | C across SWE bands (id=455, 462, 466, 472, 473 all C); F on the quant-research / hardware bands. | Nyquestro's tail-latency observability + safe-Rust matching engine reads strong on the *concept* axis. | Kernel-bypass production code (gap #4), C++ deep (gap #1), published latency numbers (gap #11). Stretch lottery for SWE bands; F on quant-research and hardware bands is correct. |
| **Two Sigma** | Not in this batch (no Two Sigma rows visible in the sample). Historical pattern: narrow-funnel PhD-weighted research + structured SWE intern pipeline; SWE intern is the realistic entry, mid-band SWE is stretch. | Cernio + Nyquestro on the SWE-intern path. | Same as Citadel — C++ depth, comp-prog readiness, credential floor for FTE conversion. |
| **Optiver SE Junior** | A at Digital Assets SWE (id=671) — the strongest pinnacle-tier landing in the batch. Senior-band Optiver lands at F. | Aurix (Q64.96 fixed-point + cross-DEX) + Nyquestro (live Coinbase WebSocket + LOB matching) is **exactly** the Digital Assets desk shape — Caner's portfolio has the highest-coupling fit in the batch for this specific role. | C++ depth at the 5+ year mark, kernel-bypass exposure. Junior / digital-assets band is realistic primary target; broader Optiver senior bands are stretch. |
| **DRW (incl. Cumberland)** | S on the Market Data / Trading Systems / Cumberland Trading Systems seats (id=207, 217, 1204). A on the Python platform / derivatives / prediction-markets seats. | Nyquestro matching engine + live Coinbase feed + Aurix crypto-mm depth — DRW's broader London hiring + portfolio-acceptance reads Caner inside the pool. DRW is the cleanest primary-target pinnacle in the lane. | C++ residual friction at Cumberland; otherwise strong fit. **Top-priority pinnacle application set.** |
| **QRT** | S on the 2026 internship cohort (Data Engineering / Quant Developer / SWE — id=765, 766, 768) and S on Data Core / Distributed Market Data Systems SWE (id=805, 809). B on the Quant Developer HFT (C++) seat. | Internship cohort accepts BEng with strong systems-Rust portfolio; QRT's "growing interest in modern systems languages" framing is the door Caner's portfolio walks through. | C++ depth on the C++-mandated HFT seats. **Internship cohort is the cleanest hft-lane career launch in Europe** at the candidate's profile shape. |

## Lane-internal calibration

### Pool shape

The 217-job hft-lane pool is dominated by C (39%) and F (33%) because:

1. The lane's pinnacle firms (Jane Street, HRT, Citadel, Two Sigma, Optiver senior) hire 10-50 graduates per cycle and screen on comp-prog + top-university credentials. Caner's BEng 2:2 from York places him outside the realistic primary-target pool at these firms, *not* because the portfolio is weak but because the hiring shape doesn't intersect his profile shape. This is correct prestige-trap reasoning — C is the right grade even when Q2-Q4 read strong on the absolute frame.
2. Quant-research roles (DRW Quant Researcher Equities/Commodities/ML, IMC Quant Researcher, BlueCrest, Capstone, GSA Researcher, Aquatic Researcher) are PhD-track functions; F across the board is correct.
3. Senior-band C++ seats (Citadel Securities C++ SWE, Tower-equivalent, Optiver senior C++) have explicit 5+ year floors plus C++-mandate; F is correct.

The B/A/S pool of 61 jobs (28%) is the realistic application universe. Within that:

- **S (10)**: DRW Market Data + DRW Cumberland (×2) + QRT internships (×3) + QRT SWE Data Core + QRT SWE Distributed Market Data Systems Python + Tradeweb 2026 C++/Python intern + Verition Junior Rust. **These are the top-priority axis bets** — apply with energy, treat as the lane's S-tier from this run.
- **A (9)**: DRW Python Trading Platform / Equity Index Options / Prediction Markets / Python Global Delta One; Old Mission Python; Optiver Digital Assets; Vitol Desk Dev; Tradeweb SRE/BI intern; Verition Fixed Income Python.
- **B (42)**: QRT Database Engineer / Market Access C++ / HFT C++ / Data & Lifecycle seats; Squarepoint grad/junior; PDT Partners SWE; Schonfeld 2027 intern; Ansatz Capital (lottery axis bet); Point72 Cubist quant-dev intern. Apply when the S/A pipeline is thin.

### Prestige-trap stance

Most pinnacles (Jane Street, HRT, Citadel, Two Sigma) correctly sit at C across their SWE bands. The rubric's prestige-trap pattern — high CV value + strong technical fit + brutal selectivity = C, not A — is the dominant pattern in this lane. **Do not promote pinnacle roles to A or above to fill an imagined budget**; the S/A pool of 19 from grad-pipeline / Rust-specific / mid-tier-pinnacle seats *is* the realistic shortlist for this candidate's profile shape.

### Realistic primary targets (by application priority)

1. **DRW (incl. Cumberland)** — apply to Market Data SE, Cumberland Trading Systems Engineer, Python Trading Platform, Equity Index Options, Prediction Markets Python, Python Global Delta One. 6+ S/A roles in this batch; DRW is the lane's primary pinnacle for this profile.
2. **QRT 2026 internship cohort** — Data Engineering, Quant Developer, SWE intern. Conversion-to-FTE is the cleanest junior-phase hft launch in Europe.
3. **QRT mid-band SWE seats** — Data Core, Distributed Market Data Systems (Python), Trading Systems Reliability Engineer.
4. **Optiver Digital Assets SWE** — strongest single role-portfolio coupling in the batch (Aurix + Nyquestro fit exactly).
5. **Verition Junior Rust Developer** — only Rust-primary HFT seat in 200+ jobs; structurally narrow applicant pool.
6. **Tradeweb 2026 internship cohort** — wider-funnel public-company entry; C++/Python intern in particular.
7. **AQR engineering side, Cumberland Mining, Vitol Desk Dev** — adjacent pinnacles for the commodities / energy / multi-strat slice.

This list is shaped by the candidate's specific profile, not by lane-pinnacle prestige. Jane Street / HRT / Citadel are correctly *not* on this list — they belong to the stretch-lottery cohort that does not make a budget of 30.

---

**Gaps**: C++ Familiar → Proficient, FIX protocol, kdb+/q, kernel-bypass networking, FPGA (non-pursued), ITCH/OUCH/SBE, multicast, options Greeks, published latency benchmarks, OCaml (Jane Street-specific).
**Strengths**: Nyquestro (deterministic safe-Rust LOB matching engine + live Coinbase WebSocket + HDR-histogram tail-latency + microstructure surface), Aurix (Q64.96 + cross-DEX + LP backtest + risk-statistical awareness), Cernio (production-shaped async Rust + Tokio + multi-layer architecture), Rust Proficient across 9 active projects.
**Key recommendation**: ship a FIX 4.4 exchange adapter on Nyquestro AND push Tectra to a working C++ ITCH feed-handler + matching loop with published `criterion` benchmark numbers — together these close the three dominant lane frictions (C++ depth, FIX, published latency) and would re-grade 8+ B/C rows up a tier without speculative work.
