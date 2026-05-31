# Non-Obvious Discovery — 2026-05-31

Source-pipeline territory: Rust Foundation member rolls + RustConf 2025 / EuroRust 2025 sponsor tiers + AI-VC portfolios (Air Street Capital, LocalGlobe, Hoxton, Octopus, Episode 1, Balderton, Index) + Tauri / Bevy commercial ecosystems + London Rust meetup organisers + "who else" expansion. Mainstream tech aggregators surface none of these the way these pipelines do.

Dedup against `/tmp/cernio-universe.txt` (687 entries) applied to every candidate. Already-present hits excluded: Helsing, Keyrock, QRT, Polar Signals (already in DB), Sentry, Canonical, Ferrous Systems, Red Badger, V7, Wayve, Synthesia, Tractable, PolyAI, Graphcore, ElevenLabs, Thought Machine, Automata, Ravelin, Fern Labs, Gradient Labs, Scope, CoMind, QuestDB, Zed, TrueLayer.

---

### Antithesis
- **Website**: https://antithesis.com
- **Location**: HQ Vienna VA + satellite London + SF
- **What they do**: Autonomous deterministic-testing platform — software runs inside a hypervisor that records every nondeterministic input, so bugs reproduce bit-for-bit on demand; a fuzzer drives the system through enormous numbers of states.
- **Why relevant**: Caner's NeuroDrive ships layered determinism (fixed 60Hz tick, `SimSet` ordering, LCG-based 1200-step replay), Nyquestro pins `run_twice_identical_sequence_identical_output` end-to-end, Aurix uses `config_hash` SHA over input config as idempotency key, and Tectra was built clock-first specifically so no service ever breaks determinism — Antithesis is the commercial deepening of exactly this discipline.
- **Source**: https://antithesis.com/company/careers/ (career page lists "Vienna, VA | London | San Francisco"); https://rustconf.com/ — RustConf 2025 Gold sponsor (2026-05-31 fetch)
- **Sponsor**: unknown (London office, but UK Skilled Worker sponsor licence not visible on careers page or gov.uk lookup)
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Ostium Labs
- **Website**: https://www.ostium.com
- **Location**: London, UK
- **What they do**: First decentralised perpetuals exchange for Real World Assets (stocks, ETFs, commodities, indices, forex) on Arbitrum with up to 200x leverage and instant USDC settlement; $25B cumulative trading volume; processed $5B in metals alone.
- **Why relevant**: Direct overlap with Caner's Nyquestro (deterministic price-time-priority order book, microstructure surface — `microprice()`, `ofi(n)`, `spread_cents()`, OU mean-reverting mid simulator) and Aurix (clean-room Uniswap V3 LP backtester, Q64.96 tick math, LVR discrete approximation). Ostium is the on-chain perp-DEX engineering layer Aurix sits adjacent to.
- **Source**: https://siliconcanals.com/ostium-labs-raises-funding/ (London-based perp DEX, LocalGlobe portfolio); https://fortune.com/2025/12/03/ostium-series-a-fundraise-perpetuals-perps-crypto/ ($20M Series A, General Catalyst + Jump Crypto led, 2025-12-03)
- **Sponsor**: unknown (London-HQ small startup, no published sponsor-list entry — likely sponsor-licensed but verify on first apply)
- **Lane**: crypto-mm
- **Discovered**: 2026-05-31

### Flow Engineering
- **Website**: https://www.flowengineering.com
- **Location**: London, UK
- **What they do**: Collaborative SaaS platform integrating CAD, simulation, Excel, MATLAB models into one live traceable requirements system for hardware engineering teams (reusable rockets, autonomous vehicles, nuclear, robotics); enterprise customers include Rivian, Joby Aviation, Astranis, Radiant.
- **Why relevant**: Direct match for Caner's "interfaces and contracts" / "trait-based modular Rust" / "ECS + fixed-timestep simulation" thread (NeuroDrive's `SimSet` ordering, controller-agnostic `game/` vs `brain/` boundary). Hardware-engineering tooling is a devtools-adjacent vertical with high systems-engineering depth; founded by Pari Singh, $23M Series A Oct 2025 led by Sequoia with Stripe-founders / Unity-founder follow-on.
- **Source**: https://startupintros.com/orgs/flow-engineering (London HQ + funding history); EU-Startups coverage 2025-10 (Series A announcement)
- **Sponsor**: unknown (London-HQ, sponsor-likely for hardware-eng SaaS role with Sequoia capital — verify)
- **Lane**: devtools
- **Discovered**: 2026-05-31

### RevEng.AI
- **Website**: https://reveng.ai
- **Location**: London, UK
- **What they do**: Foundational deep-learning models (BinNet) trained alongside allied-government cyber units to perform semantic analysis of compiled binaries — reverse-engineering malware, third-party software and firmware without source access. Series A $15M (NATO Innovation Fund, In-Q-Tel, IQ Capital, Episode 1).
- **Why relevant**: Bridges Caner's ML-systems work (AsteroidsAI GNN-SAC, NeuroDrive PPO from-scratch, Image Browser ONNX multi-encoder fusion) with Tectra's interest in low-level systems / compiler IR (Xyntra's `NodeID(u32)`, `TensorShape`, `OpKind`, four-category error taxonomy). Binary-level foundation models is the most direct fusion of ML + systems work in his portfolio.
- **Source**: https://siliconangle.com/2026/05/27/reveng-ai-raises-15m-reverse-engineer-software-binaries-hunt-malicious-threats/ (2026-05-27 Series A); https://uk.linkedin.com/company/reveng-ai
- **Sponsor**: unknown (London-HQ deep tech, defence-aligned investor mix increases sponsor likelihood)
- **Lane**: ai-ml
- **Discovered**: 2026-05-31

### CuspAI
- **Website**: https://cuspai.com
- **Location**: Cambridge, UK (+ Amsterdam, Berlin, Tokyo)
- **What they do**: Foundation-model AI for materials science (carbon capture, energy materials); generates synthesizable candidates ~10x faster than conventional discovery. $100M Series A Sep 2025 (NEA, Temasek, Nvidia). Co-founded by Max Welling (ex-Microsoft Research Distinguished Scientist, ex-Qualcomm VP). Hinton + LeCun on advisory board.
- **Why relevant**: Foundation-model engineering at scale matches Caner's NeuroDrive (handwritten PPO with PopArt adaptive critic-target normalisation, AdamW + tanh-squashed actions, sustained research-level depth) and Image Browser (3-encoder ONNX fusion — CLIP/SigLIP-2/DINOv2 — with RRF per Cormack 2009). Cambridge HQ = Tier 1 commute-belt.
- **Source**: https://fortune.com/2025/03/05/cuspai-hinton-lecun-google-deepmind-ai-foundation-models-chemistry-climate-change/ (Hinton + LeCun advisors, talent pull from DeepMind); https://siliconangle.com/2025/09/10/cuspai-raises-100m-build-ai-search-engine-transform-materials-science/ ($100M Series A 2025-09-10)
- **Sponsor**: unknown (Cambridge AI startup with Nvidia + Temasek, frontier-AI talent war — almost certainly sponsors, verify)
- **Lane**: ai-ml
- **Discovered**: 2026-05-31

### Nominal
- **Website**: https://nominal.io
- **Location**: HQ USA — but Rust-engineering hiring published as remote on rust.careers/Indeed UK listings
- **What they do**: Unified industrial data stack — Rust-native real-time telemetry / logs / video / simulation capture for hardware test programs at launch pads, airport tarmacs, lab test benches, nuclear test sites. Founding team from SpaceX, NASA, Lockheed Martin.
- **Why relevant**: Direct alignment with Caner's HFT-style observability work (Nyquestro's HDR-histogram p50–p9999 per `Op`, bounded `sync_channel(8192)` with `AtomicU64` drop counter, JSONL telemetry pipeline) + Tessarix's 50+ event kinds discriminated-union telemetry. Industrial test telemetry is the same engineering problem space, scaled.
- **Source**: https://users.rust-lang.org/t/is-there-any-aerospace-military-companies-claimed-to-use-rust-as-their-main-programming-language-in-the-future/68484 (Rust-native aerospace data stack); https://nominal.io
- **Sponsor**: unknown (US-HQ; UK sponsorship only if EMEA office opens — verify via direct outreach; tentatively include because of the Rust + aerospace + telemetry fit)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Foresight Mining Software (Foresight Spatial Labs)
- **Website**: https://www.fslabs.ca (Canada-HQ, distributed)
- **What they do**: Bevy-engine-based CAD application for the mining industry — full real-time 3D mine-design tool built on Rust + ECS architecture rather than the traditional Qt/C++ stacks. Cited by Bevy maintainers as one of the few large-scale production commercial Bevy deployments.
- **Why relevant**: Caner's NeuroDrive is built on Bevy ECS with `FixedUpdate` at 60Hz and 4-stage `SimSet` ordering enforced by chaining; Foresight is the rare commercial Bevy team where that exact ECS+systems discipline is the day job. Concurrent-ECS + fixed-timestep sim is a Comfortable-band domain in Caner's profile.
- **Source**: https://news.ycombinator.com/item?id=31044582 (HN thread on commercial Bevy users — Foresight Spatial Labs cited); This Week in Bevy
- **Sponsor**: unknown (Canada-HQ — UK Skilled Worker unlikely without UK office, include for the Bevy-production-engineering rarity but flag)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### 1Password (AgileBits)
- **Website**: https://1password.com
- **Location**: Toronto HQ + UK remote roles published
- **What they do**: Password manager + enterprise secrets platform; Rust-Foundation Silver member; significant Rust adoption in core sync engine; ~2000 employees; growing extended access management product line.
- **Why relevant**: Production-Rust at enterprise scale; mirrors Caner's local-first / privacy-by-construction stance (Image Browser zero post-launch network, Aurix read-only never-signs, Nyquestro "exactly one run on disk"). Cross-domain local-first encryption infra is a strong systems-infra anchor.
- **Source**: https://rustfoundation.org/members/ (Silver member 2026-05-31 fetch); https://1password.com/careers (UK remote eligibility listed on some roles)
- **Sponsor**: unknown (Canada-HQ; UK is remote-only for engineering — sponsor licence per role basis; verify)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

### Keel (keel.so)
- **Website**: https://keel.so
- **Location**: Altrincham, Cheshire (UK; remote-friendly)
- **What they do**: Code-first programmable operations platform — backend engine, API-first solutions, auto-generated admin tools, open-source — sitting between no-code and full ERPs. Founded by former Echo (UK online pharmacy) CEO/CTO/CPO. £6M seed from LocalGlobe + Earlybird.
- **Why relevant**: Schema-driven generation + idempotent migrations + clear API contracts maps directly to Caner's Cernio (6 idempotent migrations, `format_description` invariant tests, ATS provider trait surface) and Aurix (4 idempotent ALTER TABLE migrations + embedding-pipeline-version migration). Code-first dev-platform engineering with British engineering culture.
- **Source**: https://www.eu-startups.com/2024/10/london-based-keel-secures-e5-5-million-to-empower-operators-to-become-more-technical/ ($6M seed 2024-10-23); https://keel.so/
- **Sponsor**: unknown (UK-HQ small dev-tool startup — sponsor-likely but unverified; flag)
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Tweede Golf
- **Website**: https://tweedegolf.nl
- **Location**: Netherlands HQ — but distributed remote engineering team across Europe with Rust + embedded specialism
- **What they do**: Rust consultancy + custom embedded-Rust engineering services + steward of `rust-prod` ecosystem index; Rust-Foundation Silver member; significant contributions to embedded-rust ecosystem (HAL crates, async embedded runtimes).
- **Why relevant**: Aligned with Caner's open-source contribution practice (Burn PR #4894 APPROVED, tinygrad #16119, alloy #1156) and his evolutionary path toward deeper Rust ecosystem investment. Consultancy model + Rust-only specialism is the closest European analogue to Ferrous Systems for OSS-aligned engineering.
- **Source**: https://rustfoundation.org/members/ (Silver member 2026-05-31); https://tweedegolf.nl/en/expertise
- **Sponsor**: unknown (NL-HQ — UK remote eligibility unclear; flag for direct outreach if Caner is open to NL contracts)
- **Lane**: systems-infra
- **Discovered**: 2026-05-31

---

## Sponsor-ambiguity notes

Eight of ten entries are `Sponsor: unknown` rather than `yes`. The non-obvious pipeline finds smaller / earlier-stage / non-UK-HQ companies for whom the gov.uk register is silent (they may sponsor but haven't been queried). The two strongest UK-HQ entries are Ostium Labs and RevEng.AI — both London, both recently-funded, both sponsor-licence-likely but not yet verified on the register. Verify on first-application outreach rather than dropping pre-emptively, because non-obvious pipeline value disappears if these are filtered too early.

Companies confirmed dropped during discovery (non-sponsor or wrong-shape):
- **Smarkets** (London Rust meetup sponsor) — gambling, ethical exclusion
- **Mimic Robotics** (Zurich) — no UK presence, Python not Rust
- **CodSpeed** (NY/France) — no UK office for engineering
- **Modal Labs** (NY/Stockholm/SF) — London role is sales only, no engineering presence
- **Exein** (Italy) — no UK Skilled Worker sponsorship
