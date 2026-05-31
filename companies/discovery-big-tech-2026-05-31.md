# Big-Tech Discovery — 2026-05-31

Generalist software engineering at major tech employers; standard SWE ladder. Established large UK/UK-presence engineering shops not already in `/tmp/cernio-universe.txt`. UK Skilled Worker sponsor verified via sponsor registers, careers pages, or Hunt-UK-Visa-Sponsors mirror of the Home Office register. BT, Vodafone, John Lewis explicitly excluded — they do not sponsor for grad/SWE roles per careers-page FAQs.

---

### Atlassian (UK) Operations Limited
- **Website**: https://www.atlassian.com
- **Location**: HQ Sydney; UK office London (33 Cannon Street)
- **What they do**: Developer-collaboration SaaS — Jira, Confluence, Bitbucket, Compass, Rovo. Backend Java/Kotlin services at scale on AWS, ML-driven search and agents on top.
- **Why relevant**: Stack overlap with Open Source Contributions (Rust + ML infra mindset) and Tessarix (collaborative editor primitives via MDX widget). Jira/Bitbucket-class search tooling parallels Image Browser's RRF retrieval problem at index scale.
- **Source**: https://immigrationgpt.co.uk/company/Atlassian-(UK)-Operations-Limited
- **Sponsor**: yes (https://immigrationgpt.co.uk/company/Atlassian-(UK)-Operations-Limited)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Shopify UK Limited
- **Website**: https://www.shopify.com
- **Location**: HQ Ottawa; UK presence London
- **What they do**: Multi-tenant commerce platform — Rails monolith + Go services on Kubernetes, sharded MySQL, edge runtime (Oxygen) for storefronts.
- **Why relevant**: Performance-engineering-on-constrained-hardware mindset (Image Browser's 22-second freeze fix) directly applies to Shopify's storefront-TTFB obsession; Rust + WASM edge-runtime work matches Cernio's Rust depth.
- **Source**: https://companyjobs.co.uk/blog/acadp_listings/shopify-uk-limited/
- **Sponsor**: yes (https://companyjobs.co.uk/blog/acadp_listings/shopify-uk-limited/)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### GitHub UK Limited
- **Website**: https://github.com
- **Location**: HQ San Francisco; UK office London
- **What they do**: Source-hosting, Actions CI, Codespaces, Copilot. Ruby + Go monorepo, MySQL+Vitess, custom Git infrastructure, agentic developer-tools surface on Copilot.
- **Why relevant**: Open Source Contributions project is GitHub-native; skill / agent prompt authoring directly maps to Copilot Chat agent surface. Tessarix's MDX-widget surface and GitHub Issues / discussions form structurally adjacent.
- **Source**: https://huntukvisasponsors.com/ (sponsor register mirror) + https://github.com/about/careers
- **Sponsor**: yes (https://huntukvisasponsors.com/)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Snap Group Limited
- **Website**: https://snap.com
- **Location**: HQ Santa Monica; UK office London (King's Cross)
- **What they do**: Snapchat backend + AR engineering (Lens Studio, Spectacles). Heavy GPU shader work, on-device ML for AR effects, distributed messaging.
- **Why relevant**: Image Browser's CLIP + DINOv2 + SigLIP-2 on-device ONNX inference is the same problem shape as Snap's Lens ML pipeline. Determinism/reproducibility from NeuroDrive applies to AR-effect QA at scale.
- **Source**: https://careers.snap.com/jobs?location=London
- **Sponsor**: yes (snap.com listings declare visa support; mirror at huntukvisasponsors.com)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Reddit UK
- **Website**: https://www.redditinc.com
- **Location**: HQ San Francisco; UK office London
- **What they do**: Social-platform backend — Go + Python services, GraphQL federation, ML-driven feed ranking and content safety.
- **Why relevant**: Reddit's ranking-and-ML-search problem mirrors Image Browser's multi-encoder fusion at scale; Cernio's profile-aware grading rubric design parallels Reddit's content-policy classification.
- **Source**: https://www.linkedin.com/jobs/reddit-jobs-london (67 open roles surfaced)
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Pinterest UK
- **Website**: https://www.pinterestcareers.com
- **Location**: HQ San Francisco; UK office London
- **What they do**: Visual-discovery engine — multi-encoder image embeddings, retrieval-at-scale, recommendation systems.
- **Why relevant**: Direct topical match for Image Browser — Pinterest's visual-search infrastructure (multi-encoder CV + retrieval) is the production-scale version of the same problem Caner solved locally. Strongest profile-fit on this list.
- **Source**: https://www.pinterestcareers.com/jobs/?country=United+Kingdom
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Discord UK
- **Website**: https://discord.com
- **Location**: HQ San Francisco; UK presence London (remote-friendly)
- **What they do**: Realtime messaging + voice/video infrastructure at scale. Elixir + Rust + Python stack; the public Rust + io_uring posts on their engineering blog signal serious low-level work.
- **Why relevant**: Rust at scale (Cernio + Nyquestro + Image Browser anchor); async I/O + bounded-concurrency depth (Cernio's `tokio::Semaphore` + Image Browser's single-flight `AtomicBool`) matches Discord's voice-gateway concurrency profile.
- **Source**: https://discord.com/careers
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Dropbox UK
- **Website**: https://www.dropbox.com
- **Location**: HQ San Francisco; UK office London
- **What they do**: File-sync + collaboration infrastructure — historically Go and Rust at the storage layer (Magic Pocket), Python at the application layer.
- **Why relevant**: Local-first / privacy-by-construction (Image Browser, Tessarix, Aurix all share this anchor) parallels Dropbox's offline-first sync model; Rust at the storage layer matches Nyquestro's deterministic-engine discipline.
- **Source**: https://jobs.dropbox.com/all-jobs?location=London%2C%20United%20Kingdom
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Workday Limited
- **Website**: https://www.workday.com
- **Location**: HQ Pleasanton CA; UK office London (already in DB as "Workday|workday.com" — SKIPPED, removed from this file)

### ServiceNow UK Limited
- **Website**: https://www.servicenow.com (ALREADY IN DB — SKIPPED)

### HubSpot
- **Website**: https://www.hubspot.com
- **Location**: HQ Cambridge MA; UK office London
- **What they do**: CRM + marketing SaaS — Java microservices, MySQL/Vitess, ML for lead scoring and content generation.
- **Why relevant**: Living-system architecture (Cernio's profile-as-runtime-read pattern, populate-from-lifeos one-way sync) maps directly to HubSpot's CRM sync substrate; CMS-content surfaces parallel Tessarix's MDX authoring.
- **Source**: https://www.hubspot.com/careers/jobs?country=United+Kingdom
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Zendesk UK
- **Website**: https://www.zendesk.com
- **Location**: HQ San Francisco; UK office London
- **What they do**: Customer-support SaaS — multi-tenant Rails, Go services, ML-driven ticket routing and AI-agent automation.
- **Why relevant**: Multi-LLM orchestration (Consilium) and prompt-engineering depth (Tessarix) align with Zendesk's recent AI-agent push.
- **Source**: https://jobs.zendesk.com/us/en/search-results?keywords=%22London%22
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### DocuSign UK
- **Website**: https://www.docusign.com
- **Location**: HQ San Francisco; UK office London (Holborn)
- **What they do**: E-signature + agreement-cloud SaaS — Java + Go services, contract-AI on LLM extraction.
- **Why relevant**: Idempotency-as-load-bearing-invariant (Cernio's `format` idempotency tests, Image Browser's idempotent migrations) is the same discipline DocuSign needs for legally-binding signature workflows.
- **Source**: https://careers.docusign.com/jobs?location=London%2C+United+Kingdom
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Adobe Systems Europe Limited
- **Website**: https://www.adobe.com (Adobe ALREADY IN DB as "Adobe|adobe.com" — SKIPPED)

### Autodesk
- **Website**: https://www.autodesk.com (ALREADY IN DB — SKIPPED)

### Frontier Developments plc
- **Website**: https://www.frontier.co.uk
- **Location**: Cambridge UK (single studio)
- **What they do**: AAA game studio behind Elite Dangerous, Planet Coaster/Zoo, Jurassic World Evolution. In-house COBRA engine in C++.
- **Why relevant**: Bevy ECS depth (NeuroDrive's deterministic fixed-timestep + 4-stage `SimSet` ordering) directly maps to game-engine simulation discipline; concurrent-ECS + fixed-timestep is a Frontier-COBRA-engine problem.
- **Source**: https://www.frontier.co.uk/careers (relocation support advertised) + https://uk.linkedin.com/company/frontier-developments
- **Sponsor**: unknown (relocation support strongly implies sponsor licence)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### SEGA Europe / Creative Assembly
- **Website**: https://careers.sega.co.uk
- **Location**: Brentford (SEGA Europe HQ), Horsham (Creative Assembly), Sofia
- **What they do**: AAA strategy + action games — Total War series, Alien Isolation, Hyenas. Custom C++ engines.
- **Why relevant**: NeuroDrive's brain-inspired RL substrate is directly applicable to Total War AI behaviour modelling; performance engineering on constrained hardware (NeuroDrive's 426→2 stutter fix) is the daily problem of a strategy-game engine.
- **Source**: https://careers.sega.co.uk/ + https://www.creative-assembly.com/careers
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Sumo Digital
- **Website**: https://www.sumo-digital.com
- **Location**: Sheffield HQ, Nottingham, Leamington Spa, Newcastle, Brighton
- **What they do**: Co-dev + original-IP game studio — Sonic, LittleBigPlanet, Hood: Outlaws & Legends. Multi-engine (Unreal + bespoke).
- **Why relevant**: Trait-based modular Rust design (Vynapse, Cernio ATS providers) maps to multi-engine porting work — clean abstractions over heterogeneous engines.
- **Source**: https://www.sumo-digital.com/careers/
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Playground Games
- **Website**: https://www.playground-games.com
- **Location**: Leamington Spa (Microsoft-owned)
- **What they do**: AAA studio behind Forza Horizon series and upcoming Fable. ForzaTech engine (heavily modified) and Unreal Engine 5.
- **Why relevant**: NeuroDrive is a deterministic racing simulator with brain-inspired RL — the same domain Playground works in at AAA scale. Strongest profile-fit among UK game studios.
- **Source**: https://www.playground-games.com/careers
- **Sponsor**: unknown (Microsoft subsidiary; Microsoft holds sponsor licence)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Rebellion Developments
- **Website**: https://www.rebellion.com
- **Location**: Oxford (HQ), Liverpool, Warwick — Oxford is Tier 1 commute
- **What they do**: AAA + indie game studio — Sniper Elite, Zombie Army. Custom Asura engine.
- **Why relevant**: Performance engineering on constrained hardware (Image Browser thumbnail-phase 150s→30s) maps to Asura engine optimisation discipline.
- **Source**: https://www.rebellion.com/careers/
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Splash Damage
- **Website**: https://www.splashdamage.com
- **Location**: Bromley, London
- **What they do**: AAA co-dev studio — Halo, Gears Tactics, Quake Wars. Unreal Engine specialists with custom networking layer.
- **Why relevant**: HFT-style observability + tail-latency tracking (Nyquestro HDR-histogram p50–p9999) is directly applicable to multiplayer-shooter netcode profiling.
- **Source**: https://www.splashdamage.com/careers/
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Tesco Technology
- **Website**: https://www.tesco-careers.com/technology-uk/
- **Location**: Welwyn Garden City (HQ), London
- **What they do**: One of the UK's largest in-house tech orgs — Kotlin/Android, Kubernetes-on-GCP, real-time inventory, ML-driven personalisation.
- **Why relevant**: Cernio's SQLite-as-single-source-of-truth + ATS-integration discipline parallels Tesco's order-orchestration substrate; idempotency-as-load-bearing matches retail order workflows.
- **Source**: https://careers.tesco.com/en_GB/careers/JobDetail/Technology-Software-Engineering-Graduate-Scheme/149808 (£40k grad scheme, sponsorship threshold £54,700 referenced)
- **Sponsor**: yes (https://huntukvisasponsors.com/job/software-development-engineer-ii-kotlin-android-at-tesco-technology-pmk0jjcrbc3q)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Sainsbury's Digital, Tech & Data
- **Website**: https://sainsburys.jobs/teams/technology
- **Location**: Holborn, London + Coventry
- **What they do**: Java/Kotlin services, ML for grocery substitution and demand-forecasting, AWS-heavy. Sainsbury's Tech runs the .com platform + Nectar360 ad-tech (note: Nectar is loyalty-marketing, NOT consumer-ad-targeting per the ethical exclusions rule — B2B retail-media).
- **Why relevant**: Performance engineering + idempotency disciplines from Cernio map to retail-checkout systems; profile-aware grading parallels demand-forecasting personalisation.
- **Source**: https://sainsburys.jobs/teams/technology
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Marks & Spencer Digital & Tech
- **Website**: https://jobs.marksandspencer.com/our-teams/digital-and-tech
- **Location**: Paddington, London
- **What they do**: M&S.com platform rebuild on AWS + microservices, ML personalisation, in-store IoT, food-supply-chain analytics.
- **Why relevant**: Local-first + observability discipline (Aurix + Tessarix telemetry pipelines) maps to retail-edge analytics; Image Browser's encoder-fusion work parallels M&S clothing visual-search.
- **Source**: https://jobs.marksandspencer.com/job-search?team=Digital+
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### AVEVA Group plc
- **Website**: https://www.aveva.com (ALREADY IN DB as "Aveva|https://aveva.com" — SKIPPED)

### Sage Group plc
- **Website**: https://www.sage.com
- **Location**: Newcastle HQ + London (Cobalt Park + 105 Victoria Street)
- **What they do**: SMB accounting + payroll SaaS — multi-region multi-tenant, .NET + Java services, ML for invoice-extraction (Sage Copilot AI agent).
- **Why relevant**: Trait-based modular Rust design + idempotency discipline maps to Sage's multi-tenant accounting-period workflows; agentic LLM work (Consilium + Tessarix) aligns with Sage Copilot.
- **Source**: https://www.sage.com/en-gb/company/careers/
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Kainos Group plc
- **Website**: https://www.kainos.com
- **Location**: Belfast HQ + London, Birmingham, Bristol
- **What they do**: Listed UK digital-services + Workday-implementation firm; ships Smart products (Smart Test, Smart Compliance) on top.
- **Why relevant**: ATS / job-search infrastructure integration (Cernio's 6 production ATS fetchers) parallels Kainos's Workday-platform-integration practice; code-health audit discipline matches their Smart Test product.
- **Source**: https://www.kainos.com/careers/
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Endava plc
- **Website**: https://www.endava.com
- **Location**: London HQ + global delivery centres
- **What they do**: Listed UK digital-engineering firm — financial-services + payments-platform delivery, Java/Python/.NET stacks at scale.
- **Why relevant**: Cernio's profile-aware-grading + ATS-integration discipline + code-health audits all map to Endava's bank-strategist engagement model.
- **Source**: https://www.endava.com/careers/jobs
- **Sponsor**: unknown
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Sky Group (Comcast)
- **Website**: https://careers.sky.com
- **Location**: Isleworth (HQ), Leeds, Livingston
- **What they do**: Pay-TV + streaming engineering — Sky Go, NOW, Sky Stream. Heavy Java/Kotlin services, video-pipeline + DRM work, Android TV at scale.
- **Why relevant**: Performance engineering (Image Browser 22s→fixed) + observability pipeline design (Nyquestro telemetry) map to video-streaming tail-latency engineering. Note: sponsorship is case-by-case per justanswer.co.uk evidence; flag as conditional.
- **Source**: https://careers.sky.com/job/15235839/software-engineer-isleworth-gb/
- **Sponsor**: unknown (case-by-case per public anecdote; verify per-role)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### AstraZeneca (Digital R&D + Cambridge AI)
- **Website**: https://careers.astrazeneca.com
- **Location**: Cambridge Biomedical Campus (HQ), Macclesfield, London
- **What they do**: Pharma with very substantial in-house engineering — AI/ML for drug discovery, real-world-evidence data platforms, lab automation. Senior AI Engineer roles in Cambridge confirmed open.
- **Why relevant**: NeuroDrive's biology-first ML substrate (three-factor rule, STDP, Continual Backprop) is uniquely positioned for AZ's neuroscience-adjacent computational work; biology-inspired-ML is the rarest profile axis and AZ is one of the few employers where it directly applies.
- **Source**: https://careers.astrazeneca.com/location/cambridge-jobs/
- **Sponsor**: yes (FTSE-100 pharma; sponsor licence confirmed via huntukvisasponsors.com mirror)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### GSK (GlaxoSmithKline) — Tech
- **Website**: https://www.gsk.com/en-gb/careers/
- **Location**: Brentford (HQ), Stevenage, Ware
- **What they do**: Pharma with significant tech org — ML-driven drug discovery, computational chemistry platforms, clinical-trial data infrastructure.
- **Why relevant**: Same biology-first-ML profile-fit logic as AstraZeneca; SQL + WAL + reproducibility discipline (Aurix, Image Browser) maps to clinical-trial data integrity needs.
- **Source**: https://www.gsk.com/en-gb/careers/search-jobs/
- **Sponsor**: yes (FTSE-100 pharma; sponsor register entry)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

### Rolls-Royce (Digital + Aerospace Engineering)
- **Website**: https://careers.rolls-royce.com
- **Location**: Derby (HQ), Bristol, London
- **What they do**: Aero-engine + power-systems engineering with large software org — engine health-monitoring platforms (IntelligentEngine), simulation, optimisation.
- **Why relevant**: Determinism / reproducibility engineering (NeuroDrive's 1200-step replay test; Nyquestro's byte-deterministic engine output; Aurix's `config_hash`) is core to safety-critical aero software. Bristol is Tier 2.
- **Source**: https://careers.rolls-royce.com/united-kingdom/digital-and-technology
- **Sponsor**: yes (confirmed sponsor licence per RKY 2026 list)
- **Lane**: big-tech
- **Discovered**: 2026-05-31

---

## Notes for the orchestrator

- **Hard excluded after research**: BT Group, Vodafone, John Lewis Partnership — public FAQs explicitly state no Skilled Worker sponsorship for the grad/SWE tracks Caner would target. Listing them would just dilute the universe.
- **Already-in-DB SKIPS noted inline** (Adobe, Workday, ServiceNow, Aveva, Autodesk, King, Amazon, Microsoft, Apple, Google, Meta, Salesforce, Spotify, Trainline, Just Eat, Wise, Revolut, FNZ, Ocado Technology, Tesco-presence-via-Just-Eat? no, Tesco is genuinely new).
- **Sponsor=unknown entries** are kept because the company-shape and presence-in-UK make sponsor licence plausible; the `populate-db` skill's `cernio resolve` + portal-resolution stage will verify or downgrade.
- **Game-studio cluster** intentionally weighted heavily — NeuroDrive (Bevy ECS, deterministic racing-sim with RL) and Image Browser (on-device ML inference) are unusually-rare profile anchors for studios specifically; AI-Engineer-at-game-studio is a genuinely undervalued lane within big-tech for Caner.
- **No ethical-exclusion entries surfaced**: Sainsbury's Nectar360 noted as B2B retail-media not consumer-ad-targeting; if grade-companies disagrees on the line, drop Sainsbury's and keep the rest.
