---
title: Portfolio Gaps — Big-Tech SWE
lane: big-tech
last_updated: 2026-06-01
seed_source: grade-jobs Phase 3 (run 2026-06-01-121446, 85 jobs)
distribution: { SS: 8, S: 7, A: 19, B: 11, C: 8, F: 32 }
---

# Portfolio Gaps — big-tech

> Lane snapshot generated 2026-06-01 from grade-jobs Phase 3 across 85 graded jobs.
> Distribution: SS 8 / S 7 / A 19 / B 11 / C 8 / F 32. The S/SS pool is dominated
> by canonical FAANG / FAANG-adjacent grad pipelines (Amazon SDE 2026 x4, Apple ASE, Google Early-Career,
> Meta University Grad, Microsoft AGP, plus Apple Cloud Object Store, Apple Developer Foundations,
> Meta Infra/Product, Microsoft Intern, TikTok Recommendation Infra). The F tail is
> overwhelmingly geographic (Dropbox 12 remote-US/CA/MX roles, Cisco Slovakia,
> Reddit UK, Pinterest UK) plus seniority floors (M&S Staff / Senior / Manager) and
> function-misalignment (SDET, Data Scientist routing, HubSpot Deal Desk).

## Open Gaps

- **Java / JVM stack-fluency** — appeared as primary or co-primary stack in 8 of 85 graded roles, including S/A targets that would otherwise sit one tier higher.
  Roles: id=685 Palantir Backend SWE (Foundry — Java), id=693 Palantir Full Stack (Java backend + TS), id=908 Spotify Backend Data Platform (Java), id=910 Spotify C++ / CI mentions Bazel + Java, id=912 Spotify Full Stack Audiobooks (Java backend), id=1176 Apple Java Mercury Platform, id=1180 Apple JDK, id=909 Spotify Backend Release.
  Profile status: education-only — `profile/education.md` Software 2 (OO DS&A in Java) + Engineering 1 libGDX/Java 8-person team project as lead developer; `profile/skills.md` does not list Java in the Proficient or Comfortable band.
  Impact: id=1176 (Apple Java Mercury) and id=685 (Palantir Backend) explicitly downgraded from S to A on stack-mismatch despite excellent Q2/Q5. id=1180 (Apple JDK) reasoning calls out "would be S if Rust-primary".
  Closure opportunity: build one substantive Java service — Spring Boot + Postgres + a real concurrency primitive (lock-free queue, lock-striped cache) — sized at 3-5k LOC, deployed somewhere reachable, documented in `profile/projects/`. The portfolio shape change is the goal: take Java from "education-only" to "Comfortable, anchored on `projects/<jvm-project>.md` status:active". Alternative: refactor one of the smaller dormant Rust projects into a Kotlin/JVM equivalent with feature parity, surfacing the bilingual systems-thinking.

- **Cloud-platform production experience (AWS / Azure / GCP)** — appeared as expected baseline in 14 of 85 roles; explicit gate in 4.
  Roles: id=907/908/909/910/912 Spotify (GCP), id=908 Spotify Data Platform (BigQuery / Spark / Flink / Beam, "experienced with cloud infrastructure, containerized application"), id=895 Sony Data Engineer (Snowflake/dbt/cloud warehouse), id=1416 Tesco (Kubernetes-on-GCP), id=1441 M&S Loyalty (AWS + microservices), Microsoft AGP/Azure-team placements (id=1237/1238/1239), Amazon AWS internals (id=1170-1173).
  Profile status: `profile/skills.md` shows Kubernetes at Familiar; no AWS / Azure / GCP project anchor in `profile/projects/index.md`; Cernio is explicitly local-first, the antithesis of cloud-shape evidence. `portfolio-gaps.md` (root) already flags "no-K8s gap".
  Impact: lifted id=148 Cisco DevOps from B to A only through brand offset; id=908 Spotify Data Platform held at A despite excellent Q4 because of missing BigQuery / Spark / Flink / Beam evidence. Cloud-naïveté is the single most cited gap in mid-band reasoning.
  Closure opportunity: stand up a minimal three-node Kubernetes cluster (k3s on a £20/month VPS or local kind) running ONE of Cernio's existing fetchers as a stateless microservice with a horizontal-pod-autoscaler driven off ATS queue depth. Document the manifest, the ingress, the secret-management pattern, the observability hookup (Prometheus + a Grafana board sourced from the existing histogram metrics in Nyquestro). 4-6 weeks of evening work, one new `projects/k8s-cernio-edge.md` artefact — closes the gap and seeds the systems-infra lane simultaneously.

- **C++ at production band (above Familiar)** — appeared in 6 of 85 roles, gated 1 from S to A.
  Roles: id=910 Spotify C++ Platform Engineering (CI Experience squad — Python + C++ + Bazel + gRPC/Protobuf), id=898 Sony Graphics Engineer (PS5 graphics APIs, C++), id=146/148 Cisco core networking, id=1230 Meta Infrastructure (C++ adjacent), id=1180 Apple JDK (HotSpot internals — C++ shop).
  Profile status: `profile/skills.md` lists C++ at Familiar (single university coursework anchor); no active C++ project. Concept-fit (lock-free, HFT-observability) was substituted under the rubric's cross-language concept-fit rule, but only kept id=910 at A, not S.
  Impact: prevents promoting Apple JDK, Meta Infra, and Spotify C++ Platform from A toward S.
  Closure opportunity: port the Nyquestro matching-engine hot loop (already `unsafe`-free, deterministic, HDR-instrumented) to a C++23 implementation with feature parity and a parity-test harness comparing tick-for-tick outputs between the Rust and C++ versions. The dual-implementation artefact is the signal: it demonstrates C++ ability AND the engineering discipline of equivalence testing.

- **Production scale / billion-user systems experience** — appeared as the recurring "the portfolio is single-machine deep" friction in 11 A/B roles.
  Roles: id=907 Spotify Core Infrastructure ("infrastructure systems that power Spotify's backend, supporting thousands of developers and over a billion users"), id=1230 Meta Infrastructure, id=1333 TikTok Recommendation Infra ("ByteDance recommendation infra stack is typically Go + C++ + large-scale distributed systems (Hadoop/Spark/custom in-house). Caner has zero of these"), id=908 Spotify Data Platform (Spark / Flink / Beam at scale).
  Profile status: no project anchors multi-region / multi-replica / horizontally-sharded production deployment. Cernio is single-binary local-first by design; Image Browser is desktop-only; Nyquestro is in-memory single-process. Every project demonstrates engineering depth on a single machine.
  Impact: the rubric's "single-machine deep" pushback is what holds id=907 Spotify Core Infra and id=1230 Meta Infrastructure at S/A rather than promoting to SS; it shows up verbatim in id=1333 TikTok reasoning.
  Closure opportunity: not portfolio-closeable in the BEng-fresh window — this is the gap that resolves itself the day you land an SDE-I role at one of the SS targets. The closure plan is application-side, not portfolio-side: in interview behaviourals, frame the gap explicitly ("I have not operated systems at billion-user scale; here is the systems-thinking transfer from Nyquestro's lock-free design and Cernio's six-fetcher pipeline").

- **Mobile development (iOS / Android / Swift / Kotlin)** — appeared as primary gate in 3 roles, decisive in C-tier.
  Roles: id=1022 Trainline Junior iOS, id=898 Sony Graphics (PS5 platform — Objective-C / Swift adjacent), id=1416 Tesco (Kotlin Android).
  Profile status: zero mobile-platform projects in `profile/projects/index.md`. No Swift, no Kotlin in `profile/skills.md`. Bevy/NeuroDrive runs desktop only.
  Impact: id=1022 Trainline Junior iOS hard-dropped to C on stack-mismatch despite "Junior" framing.
  Closure opportunity: low priority — mobile is off the systems-infra / hft / ai-ml axes the candidate is targeting per `profile/career-goals.md`. Not worth a portfolio investment unless Apple FaceTime / iCloud iOS-side teams become primary targets. Mark this gap "intentional".

- **CI/CD platform engineering / release tooling at scale** — appeared in 5 roles, all clustered at Spotify Platform.
  Roles: id=910 Spotify C++ CI Experience, id=909 Spotify Backend Release, id=908 Spotify Data Platform, id=145 Cisco Early-Career SRE, id=148 Cisco DevOps SWE.
  Profile status: `profile/skills/methodologies-soft.md` "Solo-contributor master-only workflow" — no multi-engineer release-train evidence; no GitHub Actions / Jenkins / Bazel deep work in active projects.
  Impact: id=909 Spotify Release held to B explicitly ("commit-per-checkpoint discipline... but no project demonstrates production release-engineering tooling specifically"). Cisco DevOps (id=148) explicitly cites no-K8s + no production CI as the offset against brand.
  Closure opportunity: add a real CI/CD layer to Nyquestro AND Cernio — GitHub Actions running the full test suite + benchmark guard (regression detection on Nyquestro's p99 latency) + Cargo-deny / cargo-audit. Document the pipeline architecture in each project's `_Overview.md`. Time cost: one weekend. The free-win: every active project now demonstrates the CI engineering Spotify Release / Cisco DevOps are screening for.

- **Algorithmic-interview / LeetCode-medium grindable consistency** — universal background gate at every SS/S/A FAANG-tier role.
  Roles: every SS row (id=1170-1239 cluster), explicitly named in id=1230 Meta Infra ("the algorithmic-interview screen is the dominant filter rather than competition-pedigree"), id=1232 Meta Product ("viable with friction (algorithmic-interview ladder)").
  Profile status: `profile/leetcode.md` documents the preparation framework but no consistent daily-cadence evidence. This is execution risk, not signal risk.
  Impact: the SS/S pool will collapse if the interview ladder is failed; no portfolio depth substitutes.
  Closure opportunity: 30 min/day LC-medium habit minimum 4 weeks before any SS/S screen. Track in `profile/leetcode.md` weekly. This is the single most leverage-dense closure available — the SS targets are all wide-funnel grad pipelines where the algorithmic screen is the dominant filter.

- **2:2 degree class friction at narrow-funnel firms** — appeared as low-grade friction at 3 firms.
  Roles: implicit at id=685/691/693 Palantir (Palantir historically prefers 2:1+), explicit at id=1416 Tesco (broad on degree class but the rubric flagged 2:1+ as typical).
  Profile status: `profile/education.md` BEng York 2:2.
  Impact: not decisive at wide-funnel SS targets (Amazon, Google, Microsoft, Meta, Apple all confirmed wide-funnel), but adds friction at Palantir and bespoke-pipeline employers.
  Closure opportunity: degree class is unalterable. The closure is portfolio-side: 8 active projects + 14k LOC Rust Cernio + production-shipped tModLoader C# project explicitly outweighs the 2:2 in every SS reasoning chain. The structural mitigation is already done; the application-side closure is to lead every cover letter and CV with the project depth, never with the degree.

- **Behavioural / STAR-format interview preparation** — universal gate, not yet operationalised.
  Roles: every SS / S target with structured grad-pipeline interview loop.
  Profile status: `profile/portfolio-gaps.md` (root) already flags this; no STAR drafts in `profile/`.
  Impact: behaviourals weight ~30-40% of FAANG grad loops; under-prepared behaviourals collapse the SS pool independently of technical depth.
  Closure opportunity: pre-draft 8-10 STAR stories sourced from Image Browser optimisation (22-second freeze fix via dual-connection split), Cernio architecture (six-fetcher resilience design, lane-relativity refactor), NeuroDrive (PPO determinism debug), Nyquestro (HDR-histogram observability rollout), Performance Profiler (production C# crash-bug catches). Store at `profile/behavioural-stories.md`. Time cost: one focused day.

## Confirmed Strengths

- **Multi-domain Rust depth at production-shape** — appeared as the decisive Q3a anchor in 21 of 26 SS/S/A roles.
  Profile anchor: `profile/projects/cernio.md` (status:active, 14k LOC, 346 tests, 6 ATS provider fetchers) + `profile/projects/nyquestro.md` (status:active, deterministic price-time-priority matching engine, zero-`unsafe`, HDR-histogram p50/p95/p99/p999/p9999/max/mean) + `profile/projects/image-browser.md` (status:active, dual-connection SQLite, ONNX inference, RRF retrieval) + `profile/projects/aurix.md` (status:active, ~10.5k LOC Rust analytics platform).
  Lane signal: Apple, Amazon, Microsoft, and Meta all have growing Rust adoption (S3, Firecracker, EC2 Nitro, Azure SDK, Edge, Compiler Performance teams); Rust is the differentiator across the SS pool. The portfolio's Rust depth is the single strongest big-tech-lane asset.

- **End-to-end systems-engineering breadth** — appeared as Q3a anchor in 18 roles, the lane-affinity supporting line "shipping eight production-shaped systems across distinct domains demonstrates the generalist's range".
  Profile anchor: `profile/projects/index.md` 8 active projects (Cernio + Nyquestro + Image Browser + Aurix + Tessarix + Xyntra + NeuroDrive + Performance Profiler) spanning systems, ML, frontend, compiler IR, game-mod IL-injection.
  Lane signal: matches Bending Spoons graduate-rotation framing (id=1425-1429), Microsoft AGP team-flexibility (id=1239), Google grad SWE breadth, Amazon SDE-I rotation. Breadth is exactly what wide-funnel grad pipelines select for.

- **Observability and tail-latency instinct** — appeared as Q3a anchor in 8 roles, decisive at Apple SRE.
  Profile anchor: `profile/skills/concepts-domains.md` "HFT-style observability / tail latency" (Comfortable) and "observability" (Proficient); `profile/projects/nyquestro.md` HDR-histogram p50/p95/p99/p999/p9999/max/mean; `profile/projects/image-browser.md` tracing + tracing-subscriber with PerfLayer + 1Hz sysinfo RSS/CPU sampler; `profile/projects/cernio.md` Ratatui v5 dashboard + activity heatmap.
  Lane signal: drives the A grades on id=1174/1175 Apple SRE iCloud; supports Spotify Platform / Core Infra (id=907/910); maps directly to Amazon's SDE-I observability bar.

- **TypeScript / React 19 / Tauri 2 desktop bilingualism** — appeared as Q3a anchor in 5 roles, secondary anchor in 8.
  Profile anchor: `profile/skills/languages.md` TypeScript Comfortable + React 19 Comfortable + Tauri 2 Proficient; three active production-shaped frontends — Image Browser (33 .ts/.tsx strict mode), Aurix (45 .tsx + 18 .ts + 9k LoC + hand-rolled IPC), Tessarix (79 .tsx + 20 .ts strict mode + three LLM commands + SSE streaming).
  Lane signal: covers Microsoft M365 / VS Code / GitHub team placements (id=1237/1238/1239), Meta Product (id=1232), Palantir Full Stack (id=693), Spotify Audiobooks (id=912), Nothing Technology (id=648). Bilingual systems + frontend is rarer than either alone.

- **Production-shipped C# with IL-manipulation depth** — surprisingly strong, 1 role anchored.
  Profile anchor: `profile/projects/performance-profiler.md` status:active — live IL injection via MonoMod / Cecil, production-shipped tModLoader mod with four-layer crash safety, xUnit harness catching two production-crash bugs before they shipped.
  Lane signal: anchored Sony C#/Windows (id=901) directly; would anchor any Microsoft .NET / C# team placement at AGP. This is the strongest single-language signal outside Rust.

- **Database engineering / SQLite mastery** — appeared as Q3a anchor in 5 roles.
  Profile anchor: `profile/skills/concepts-domains.md` "database engineering" anchored on `projects/image-browser.md` (dual-connection SQLite + writer Mutex<Connection> + read-only secondary R2 via OnceLock<Mutex<Connection>>, closed perf-1777212369 22-second freeze) + `projects/cernio.md` (SQLite WAL + tiered archival lifecycle).
  Lane signal: Apple Cloud Object Store (id=1178) anchors here directly; Spotify Data Platform (id=908) partially. Storage-layer thinking is well-evidenced.

## Pinnacle Anchors (FAANG-grad gate)

**What the candidate has that maps to wide-funnel FAANG-grad screening:**

- Amazon SDE-I 2026 (id=1170-1173) — wide-funnel UK Graduate SDE intake; Caner is in the realistic primary-target pool per the rubric's SS calibration anchor. Rust portfolio is differentiator for AWS Rust-adopting teams (S3 / Firecracker / EC2 Nitro). 4 listings = high apply-cost-redundancy.
- Apple ASE Intern (id=1181) — wide-funnel Apple Software Engineering intern intake; Rust portfolio matches Apple Compiler Performance team's growing Rust adoption.
- Google Early-Career Campus 2026 (id=1223) — wide-funnel UK Graduate SWE; broad C++ / Go / Python / Java / TypeScript stack-flex makes Rust-primary OK if interview ladder clears.
- Meta University Grad (id=1231) — FAANG-tier brand on CV; Caner clears Q1 decisively per the rubric's post-graduation candidate-detection rule.
- Microsoft AGP (id=1239) + Microsoft Intern (id=1237/1238) — three doors into the same pipeline; intern-to-AGP conversion is the highest-yield CV trajectory available.

**What is currently MISSING for FAANG-grad screens specifically:**

- **LeetCode-medium throughput at consistent daily cadence**. The single-largest leverage-point. Without 4-6 weeks of 30-min/day before each screen, the SS pool collapses on the algorithmic ladder alone.
- **System-design portfolio for the FAANG-grad SD round**. Microsoft AGP / Google / Meta loops include a system-design round at the grad band. Caner has the systems-engineering thinking via Nyquestro + Cernio + Image Browser but has no written-up system-design exercises. Closure: write 6-8 system-design walkthroughs (URL shortener, distributed cache, rate limiter, message queue, recommendation system, photo-sharing service) and cross-reference them to portfolio anchors so the answer in the loop has lived backing.
- **Behavioural STAR stories pre-drafted** — see Open Gap above.
- **Mock-interview hours** — at minimum 4-6 mock loops before the first SS application lands. Pramp / interviewing.io / a peer rotation. The technical depth is there; the interview-loop performance is the gate.

## Lane Internal Calibration

**Current pool placement:** strong primary-target pool. Of 85 graded big-tech jobs, 15 land at S or above (8 SS + 7 S = 18%), with another 19 at A. The SS cluster is exclusively canonical FAANG / FAANG-adjacent grad pipelines, which is the correct shape — Caner is precisely the wide-funnel grad-pipeline candidate these are built around. The F tail (32 of 85) is overwhelmingly geographic exclusion or function-misalignment (SDET, Data Scientist routing, Manager seniority) — not signal failures.

**Top 5 primary-target rows (S/SS with strongest portfolio fit):**

1. **id=1239 Microsoft — Software Engineering Fulltime Opportunities for University Graduates (AGP)** — SS. Wide-funnel + Rust-portfolio matches Azure SDK / Edge / Compiler / VS Code teams + sponsorship via AGP solves visa.md Aug 2027 expiry + multi-stack portfolio (Cernio Rust + Image Browser TS + Consilium Python) gives Microsoft team-routing flexibility. Highest-yield single application in the lane.
2. **id=1181 Apple — Intern Software Engineer London ASE** — SS. Wide-funnel intern intake + Apple London Compiler Performance team Rust adoption + Caner's compiler-IR work (`projects/xyntra.md`) and matching-engine systems-correctness (`projects/nyquestro.md`) both map directly. Intern-to-FT conversion is the second-highest-yield pathway.
3. **id=1170 Amazon — Software Development Engineer 2026 UK** — SS. Wide-funnel UK Graduate SDE + AWS Rust-adopting teams (S3 / Firecracker / EC2 Nitro) + sponsorship-reliable + Strategy A prestige-exit framing per `profile/career-goals.md`. Apply once across the 4 listings (id=1170-1173 are duplicate-shape, apply to one).
4. **id=1223 Google — Software Engineer Early Career Campus 2026** — SS. Wide-funnel UK Graduate SWE at largest UK engineering hub; KGX1 London office; broad stack-flex covers Rust-primary candidate; lane-affinity adjacency to ai-ml and systems-infra.
5. **id=1178 Apple — Software Engineer Apple Cloud Object Store** — S. Direct portfolio anchor on database engineering + distributed systems via Image Browser dual-connection SQLite + Cernio SQLite WAL tiered archival. London + sponsorship + on-axis. Strongest non-grad-pipeline S in the lane.

**What single addition would meaningfully shift the lane shape:** the k3s / k8s mini-cluster portfolio piece (cloud-platform gap closure above). Closing the cloud gap would lift Spotify Core Infra (id=907) from A toward S, Tesco Grad (id=1416) from A toward S, Microsoft AGP into a stronger position on Azure-team routing, and Meta Infra (id=1230) toward firmer S. Net effect: +3-4 S-tier roles and meaningful upgrades across the A tier. No other single closure has equivalent lane-shape leverage in the BEng-fresh window.

**Secondary additions, ranked:**

1. Java service (id=1176 Apple Java Mercury, id=1180 Apple JDK, id=685 Palantir Backend, id=908 Spotify Data Platform all lift if Java moves from education-only to Comfortable).
2. CI/CD layer on Cernio + Nyquestro (closes Spotify Release id=909, Cisco DevOps id=148, signals release-engineering at every SS target).
3. C++23 port of Nyquestro hot loop with parity test (id=910 Spotify C++ Platform from A toward S; id=1180 Apple JDK lifts; id=1230 Meta Infra closes the C++ gap).

## Summary

Lane is in a strong primary-target posture. 15 of 85 roles at S+, 19 at A, the SS cluster cleanly mapped to canonical FAANG grad pipelines. Open gaps are tractable: cloud-platform exposure is the highest-leverage portfolio closure, Java is second, C++ third. Algorithmic-interview cadence and STAR pre-draft are the highest-leverage application-side closures. The F tail is mostly geography and seniority, not signal failure — re-running this lane in 6 months with the cloud + Java closures landed should shift 3-4 A rows to S and tighten the SS pool's strike rate.

---
**Gaps tracked:** 9. **Confirmed strengths:** 6. **Key recommendation:** stand up a k3s + Cernio-fetcher microservice + Prometheus / Grafana observability artefact (4-6 weeks) — single highest-leverage portfolio closure for the lane.
