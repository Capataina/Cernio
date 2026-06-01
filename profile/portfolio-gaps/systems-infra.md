---
title: Portfolio Gaps — Systems / Infra
lane: systems-infra
last_updated: 2026-06-01
pool_size: 400
distribution: { SS: 6, S: 14, A: 38, B: 71, C: 91, F: 180 }
seed_source: grade-jobs 2026-06-01-121446 Phase 3
---

# Portfolio Gaps — Systems / Infra lane

> Regenerated from the 400-job systems-infra Phase 3 batch. This is the candidate's strongest lane: 58 roles graded A or above (14.5% of the pool), six SS anchors (Graphcore Cambridge, Parity Polkadot Runtime, Apple London Compiler Teams ×2, Cumberland/DRW Infrastructure, Myrtle.ai). Gap shape below is the difference between the current A/S ceiling and the pinnacle-SS ceiling — what would push borderline-S into SS and unlock the closed-off SS roles (Cloudflare core systems, HashiCorp core, Anthropic infra, AWS internals, MongoDB / ScyllaDB / Datadog distributed-systems IC roles, Linux kernel teams).

---

## Tier distribution and read

| Tier | Count | % of lane | Read |
|---|---:|---:|---|
| SS | 6 | 1.5% | Niche brand + portfolio-exact alignment (Graphcore systems / Apple compilers / Parity Rust runtime / Myrtle FPGA-Rust / Cumberland infra). All cleared by stack-concentration carveout, not by professional-tenure depth. |
| S | 14 | 3.5% | Strong sponsor-capable systems-infra IC roles where the Rust + production-systems portfolio substitutes meaningfully (Palantir Infra/Edge/Apollo, Confluent, PostHog Ingestion, Proton Linux, HashiCorp Terraform, Wayve Runtime Platform, DRW Exchange-Data, Arm Grad, Ground Truth Labs Platform). |
| A | 38 | 9.5% | Real applications with one named friction — usually cloud-ops surface (K8s / Terraform), GPU / CUDA fluency, or operational-tenure language (SRE, on-call, incident response). |
| B | 71 | 17.75% | Sponsor-friction, security-clearance (defence work), or pure DevOps-flavoured roles that are off the build-engineering axis. |
| C | 91 | 22.75% | Senior / Staff IC bands the 1-year tenure cannot realistically clear; HFT systems specialist roles (HRT GPU / HPC Storage / Distributed Compute) where the niche-pedigree filter dominates. |
| F | 180 | 45% | Hard-clearance (UK Security Clearance, Anduril / SC-cleared defence) + Senior / Staff / Principal floors + IT-help-desk titles + RTL / silicon-design specialism without RTL portfolio. |

The S/A boundary is exactly where this gaps file should bite: 38 A-tier roles are one closed gap away from S, and the closures cluster into a small number of named surfaces.

---

## Open gaps

Ranked by frequency in the 400-job pool and load-bearing-ness for the S/SS ceiling.

### 1. Kubernetes operator depth — the single most-cited gap (78 JD mentions)

Across the 400-job pool, "Kubernetes" appears in 78 JDs and dominates the A-tier friction commentary on Confluent (Go + K8s operators is the primary stack), Northflank (multi-tenant K8s orchestration), Nscale Storage / Systems Integration, Optiver Cloud Infrastructure Digital Assets, Luminance Platform, Hiverge Platform, Quantifi Platform, Cohere SRE Inference Infrastructure, Thought Machine SRE, Wayve Runtime Platform. The current portfolio has zero K8s exposure — no operator, no CRD, no controller-runtime work.

The specific shape that bites: K8s consumer use (running a container) is taught in days and is not what the JDs want. The JDs want operator authorship — `controller-runtime` reconcile loops, CRD design, leader election, finalisers, status sub-resources, webhook admission. That's a 4-6 week project, not a checkbox.

**Closure prescription.** A single substantive Rust or Go K8s operator project — pick a real problem (a CRD that materialises Cernio's "discovery → resolve → search → grade" pipeline as Jobs / CronJobs, or a controller for a deterministic-replay test harness for Nyquestro). Public repo, README, demonstrable reconcile loop, status reporting, leader election, finaliser handling. This is the single highest-leverage portfolio addition for the lane.

### 2. Go production tenure (43 JD mentions of "Terraform" with Go-adjacency; recurring across Confluent / Thought Machine / HashiCorp / Northflank / PostHog)

Confluent's S-tier role explicitly states "Golang software engineering will be your primary focus". HashiCorp's two S-tier roles are Go-native (Terraform ecosystem, Vault, Consul). Thought Machine is Go + gRPC + K8s. PostHog Ingestion bridges Python + Go. The candidate has Go listed at zero — no Go project in the 15-entry portfolio.

The current Rust-concentration carveout rescues these roles to S, but it does so by relying on cross-language concept-fit. Direct Go tenure would lift them to SS-borderline.

**Closure prescription.** One non-trivial Go project — most natural fit: a Go re-implementation of the Cernio `cernio resolve` ATS-probe pipeline (different language for the same problem makes the cross-language transferability explicit on the public repo), or contributing meaningfully (3+ merged PRs) to a Go OSS project on the existing engagement umbrella (Terraform providers, Vault plugins, a Datadog OSS repo). 6-8 weeks of substantive Go work makes "Go [Comfortable]" on `skills.md` defensible.

### 3. Distributed-tracing / observability stack implementation (10 Prometheus mentions, recurring OpenTelemetry / Datadog / Grafana / Honeycomb)

Cohere SRE Inference Infrastructure, Thought Machine SRE, PostHog ClickHouse Operations, Grafana Backend Platform, Confluent, Optiver, all name Prometheus + Grafana + OpenTelemetry stacks. The portfolio has *concept*-fit on observability (Nyquestro HDR histograms, Image Browser PerfLayer + on-exit `report.md`, Cernio per-step timing on the pipeline) but no *production tooling* — no OTel SDK integration, no Prometheus exporter, no Grafana dashboard, no PromQL.

**Closure prescription.** Instrument Cernio (or Nyquestro) with OpenTelemetry SDK + Prometheus exporter + a Grafana dashboard checked into the repo as `observability/dashboards/cernio.json`. Two-week project; produces a concrete artefact (the dashboard JSON) reviewers can open. Particularly load-bearing for Datadog / Grafana / Honeycomb job applications where the dashboard *is* the engineering blog.

### 4. Kernel / driver-level work (61 "kernel" mentions, dominated by Graphcore SS + S + multiple A roles)

Graphcore's drivers-and-utilities team — the literal SS anchor — wants "low level kernel drivers and user space driver library code". Graphcore's six A-tier graduate roles (Drivers ×2, Build Engineering, DevOps, Firmware, Neuro Engine Modelling) all sit in the host-kernel / firmware boundary. lowRISC OpenTitan firmware. Apple LLDB debugger work. The current portfolio touches Apple AMX FFI in NeuroDrive (`status:active` — FFI dual-backend GEMM via Accelerate) but no actual kernel module, no `ioctl`, no character device, no `/dev/` interaction.

**Closure prescription.** A small Linux kernel module — a character device driver that exposes a synthetic device backing one of Nyquestro's deterministic-replay surfaces, or a kprobe-based syscall observer that feeds Cernio's profiling layer. Out-of-tree, MIT-licensed, readable in a single sitting. The point is the kernel-boundary tenure on the CV, not the size of the artefact. 3-4 weeks.

### 5. eBPF specifics (cross-cutting with #4)

Cloudflare and Datadog roles especially weight eBPF programmability. Honeycomb, Grafana Pyroscope, Cilium. Current portfolio: zero. Closure: an `aya`-based (Rust) eBPF program — a per-syscall latency profiler attached to Nyquestro, output piped to the existing HDR-histogram pipeline. This compounds with #3 (observability) and #4 (kernel-boundary). Two-week project, opens Cloudflare / Datadog / Isovalent doors.

### 6. CRDTs / consensus algorithm implementation (10 "consensus" mentions, 24 "raft" mentions)

ScyllaDB, MongoDB, CockroachDB, TiDB, Confluent (the Kora engine), Parity Polkadot (BABE / GRANDPA consensus). The candidate consumes consensus (SQLite WAL, Cernio's idempotent migrations) but has not implemented it. Parity's SS-tier role rests on the stack-concentration carveout — direct consensus implementation would lift it from "in the realistic pool" to "differentiated applicant".

**Closure prescription.** A from-scratch Raft implementation in Rust as a fourth distinct Rust project on the `Capataina` portfolio. Public, ~3-5k LOC, with Jepsen-style linearisability tests. The artefact title literally answers the JD's "do you understand consensus?" question. 8-10 weeks; very high leverage for the database-internals SS ceiling (ScyllaDB, Cockroach, MongoDB core).

### 7. Hypervisor / VMM specifics (Antithesis, lowRISC, defence)

Antithesis's B-tier FDE role names "software runs inside a hypervisor that records every nondeterministic input". The portfolio has Tectra's virtual-clock abstraction (dormant, C++20) but no actual KVM / Firecracker / WASM-runtime work. Closure prescription is lower-priority than #1-6 but would specifically unlock Antithesis (B → A), Wasmer (currently SS-class brand not in this batch), and the deterministic-testing niche.

### 8. Formal verification (10 mentions, mostly Apple compilers / Riverlane / lowRISC / Antithesis)

The portfolio's Xyntra (typed compiler IR, `status:dormant`) is the closest existing anchor. Closure: take Xyntra past IR-primitives to a soundness-proof on a small subset (TLA+ spec for one Cernio invariant + Apalache model-check, or Kani proofs on Nyquestro's matching invariants). This is the highest-prestige closure listed here and the lowest-cost in time — Kani on existing Rust is a 1-week project.

### 9. Service-mesh internals (Cilium / Linkerd / Istio data-plane)

Sporadic across the pool but load-bearing at Cloudflare and Datadog-adjacent SS roles. Lower-priority than #1-8.

### 10. Hardware-software co-design (compiler / runtime adjacent)

Surfaces specifically in Graphcore SS (Poplar SDK / silicon-software boundary), Apple compilers SS, Riverlane (quantum control), Matta Labs Edge Systems. NeuroDrive's Apple AMX FFI is the existing anchor and is genuinely on-point; the gap is that AMX is undocumented Apple-internal and the JDs would prefer published-ISA targets (LLVM backends, OpenCL, SYCL). Closure: a small LLVM custom backend lab repo or an SYCL kernel for one of NeuroDrive's hot paths.

---

## Confirmed strengths (from the 400-job grade evidence)

The S and SS reasoning across the batch repeatedly cites the same anchors. These are the load-bearing assets:

| Anchor | Status | Demonstrated paradigms | Roles it carries |
|---|---|---|---|
| **Cernio** (~14k LOC Rust) | active | Six ATS provider fetchers + `tokio::Semaphore`-bounded parallel HTTP + shared retry layer; SQLite WAL with 5 tables + idempotent migrations + schema-rebuild pattern; 346 tests (273 inline + 73 integration + 21 build-time invariants surfacing 3 silent production bugs); 26-file modular Ratatui TUI; embedded axum web UI; nine native Claude Code skills | The base for *every* S/A-tier rescue across Confluent, Palantir Infrastructure, PostHog Ingestion, Northflank, Luminance, Nscale Systems Integration |
| **Nyquestro** (~6.5k LOC Rust) | active | Safe-Rust (zero `unsafe`) deterministic multi-instrument LOB matching engine; ChaCha8Rng-pinned byte-deterministic output (`run_twice_identical_sequence_identical_output` integration test); HDR-histogram p50–p9999 latency tracking; live Coinbase Advanced Trade L2 WebSocket → virtual-order translation; per-op latency observability | DRW Exchange-Data S, Cumberland Infrastructure SS, Myrtle.ai SS, Optiver A, Hyperexponential A, Squarepoint A; the lock-free / deterministic / sub-µs anchor |
| **Image Browser** (28 backend files Rust + Tauri 2) | active | Writer/reader dual-connection SQLite WAL topology closing a real 22-second `ipc.get_images` freeze; three ONNX-Runtime image encoders (CLIP ViT-B/32, DINOv2-Base, SigLIP-2 Base 256); audit-removed three `unsafe` sites via `bytemuck::cast_slice`; `select_nth_unstable_by` partial-sort delivering 2.53× speedup; PerfLayer + on-exit `report.md` self-observability | Anthropic infra adjacency, Nscale Storage, every desktop / local-first / on-device-ML role |
| **Aurix** (~10.5k LOC Rust + Tauri 2 + React 19) | active | Writer-thread + r2d2 reader-pool over `tokio-rusqlite`; refinery migrations + 60s `checkpoint_truncate` task; clean-room Q64.96 tick mathematics; three-tier ingest fallback (Subgraph → Alchemy → public-RPC); regime-conditional capital-allocation verdict | Fintech crossover roles, Palantir Apollo Platform S, Thought Machine SRE A |
| **NeuroDrive** (Rust + Bevy 2D, handwritten PPO) | active | Apple AMX FFI dual-backend GEMM via Accelerate; batched mat-mat for PPO updates; M4 dual-GEMM + batched-actor delivering 21× frame-time improvement; 426-stutter problem at 17.3ms → 2 stutters at 9.0ms via flat row-major weight storage; handwritten PPO baseline with no external ML frameworks | Graphcore SS, Apple compilers SS, Isomorphic Labs GPU infra A, Wayve A |
| **Xyntra** (Rust typed compiler IR primitives, zero deps) | dormant | Typed IR, graph container, error taxonomy, config validator scaffold | Apple compilers SS, lowRISC A, Graphcore A |
| **Zyphos** (`std`-only HTTP/1.1 from raw TCP) | dormant | `std::net` bottom-up, `panic::catch_unwind` isolation, atomic connection counter | Proton Linux S, network-systems adjacency |
| **Open Source Contributions** umbrella | active | burn PR #4894 approved (+1864 LOC CLIP-ViT image-quality metric), burn #4938 draft, tinygrad #16119, alloy engagement; `Capataina/OpenSourceContributions` umbrella with per-repo `contribution-culture.md` notes | All OSS-aligned employers — Cloudflare, HashiCorp, Grafana, MongoDB, PostHog (open-core), Proton (open-source-by-default) |

### Concept-domain summary (per `skills.md` §Concepts and Domains, cited across 400 fits)

- Lock-free / deterministic engineering [Proficient] — Nyquestro
- Async I/O with bounded concurrency [Proficient] — Cernio
- Production SQLite WAL topology [Proficient] — Cernio + Image Browser + Aurix
- Performance engineering on constrained hardware [Comfortable] — NeuroDrive
- Reproducibility / determinism engineering [Comfortable] — Nyquestro + Tectra
- HFT-style observability / tail latency [Comfortable] — Nyquestro
- Reinforcement learning from first principles [Proficient] — NeuroDrive
- Compiler engineering / low-level systems [Proficient] — Xyntra
- Modular subsystem design / single-file extension points [Comfortable] — Cernio + Image Browser

---

## Pinnacle anchors — what's needed for SS-grade entry at each

| Company | Lane signal | What candidate has | What's missing for SS-tier conversion |
|---|---|---|---|
| **Cloudflare** | Pinnacle systems-infra; Rust-heavy edge; OSS-aligned | Rust depth, OSS umbrella, Zyphos raw-TCP HTTP/1.1, Nyquestro async observability | eBPF (#5), Workers/V8 isolate familiarity, one merged Cloudflare OSS PR (`workerd`, `pingora`, or `quiche`) |
| **HashiCorp** (post-IBM) | Pinnacle; Go-native; OSS-aligned | Rust systems portfolio; concept-fit on multi-provider orchestration (Cernio) | Go tenure (#2), one Terraform-provider OSS PR, K8s operator (#1) |
| **Datadog** | Pinnacle observability | HDR-histogram tail-latency work (Nyquestro), Image Browser PerfLayer | OpenTelemetry SDK integration (#3), eBPF (#5), Go tenure (#2) |
| **Grafana Labs** | Pinnacle observability OSS | Same as Datadog | Grafana dashboard artefact (#3), one Grafana / Mimir / Loki / Pyroscope OSS PR, Prometheus exporter on Cernio |
| **MongoDB** | Pinnacle distributed-database | Production SQLite WAL discipline, idempotent migrations | Raft / consensus implementation (#6), B-tree / LSM internals project, one MongoDB / WiredTiger / mongo-rust-driver OSS PR |
| **ScyllaDB** | Pinnacle high-performance DB | Rust + lock-free (Nyquestro) | Seastar-style thread-per-core / shared-nothing implementation, Raft (#6), C++ tenure (Chrona paused — needs resurrection) |
| **Wasmer** | WASM runtime | Xyntra IR primitives | WASM runtime contribution, hypervisor / sandboxing (#7), one Wasmer / wasmtime / wasmer-js OSS PR |
| **Apple London Compiler Teams** | Pinnacle compilers SS | Xyntra (dormant), NeuroDrive AMX FFI | Xyntra past IR-primitives to working interpreter; LLVM patch (any size); compiler-internals blog post |
| **Linux kernel teams** (Kernel Recipes, Collabora-style) | Pinnacle systems | None directly | Out-of-tree kernel module (#4), one kernel-mailing-list patch (typo / minor fix is acceptable as first step), `linux-next` build tenure |
| **Parity (Polkadot)** | Pinnacle Rust systems | SS already (#702) | Direct Substrate / Polkadot SDK OSS contribution would lift conversion-probability from sub-1% to credible primary-target |
| **Anthropic infra** | Pinnacle AI-infra | Image Browser local-first ONNX, Cernio production-grade Rust | Distributed-systems consensus (#6), GPU / CUDA tenure (Isomorphic-style), one Anthropic OSS adjacent contribution (the `claude-code` extension ecosystem is the lowest-friction entry) |
| **AWS internals** (S3 / EC2 / Lambda) | Pinnacle distributed-systems | None directly accessible (UK office Senior-IC-only) | Senior-IC tenure barrier — out of reach this cycle; revisit after 18-24 months of professional |

---

## Closure prescriptions — ranked sequence

Reasoned ordering by `(leverage × portfolio-coherence) / time-cost`. The numbering is suggestive, not strict; pick the artefact whose problem the candidate currently *cares* about, because care is what produces depth.

1. **Kubernetes operator** (4-6 weeks). Closes #1. Single highest-leverage addition. Unlocks Confluent SS, HashiCorp SS, Nscale S, Northflank S, Cohere borderline, every AI-infra cloud role.
2. **OpenTelemetry + Prometheus exporter on Cernio + Grafana dashboard** (2 weeks). Closes #3. Compounds with #1 for Cohere / Thought Machine / Datadog. The Grafana dashboard JSON is itself a portfolio artefact.
3. **Kani proofs on Nyquestro's matching invariants** (1 week). Closes #8 cheaply. Demonstrates formal-methods literacy on an *existing* project — no new project needed. High prestige-to-cost ratio for Apple compilers / Riverlane / lowRISC.
4. **`aya` eBPF latency profiler attached to Nyquestro** (2 weeks). Closes #5. Compounds with #2 (feeds the same observability surface). Opens Cloudflare / Datadog / Isovalent.
5. **Linux kernel module (character device for Nyquestro replay)** (3-4 weeks). Closes #4. Compounds with #5 (same kernel-boundary tenure). Opens Graphcore drivers, lowRISC firmware, Apple LLDB.
6. **Go re-implementation of `cernio resolve` ATS pipeline** (6-8 weeks). Closes #2. Cross-language transferability becomes provable, not assertion. Opens HashiCorp / Confluent / Thought Machine to SS-borderline.
7. **From-scratch Raft in Rust** (8-10 weeks). Closes #6. Highest-leverage closure for the database-internals SS ceiling (MongoDB / Cockroach / ScyllaDB / Confluent Kora) but also the highest-cost. Park until 1-5 are shipped.
8. **OSS contribution velocity** (continuous, target 1 merged substantive PR / month). Closes the meta-gap. The OSS umbrella is *itself* the application material for OSS-aligned employers (Cloudflare, HashiCorp, Grafana, MongoDB, Proton, PostHog).

Closures 1-5 are within an 8-12 week window and would move the lane's S-count from 14 to a credible 20-25, and convert 2-3 of the borderline SS-class roles (HashiCorp, Confluent, Cloudflare-if-it-appears) from "stack-concentration carveout rescue" to "direct portfolio match".

---

## Lane-internal calibration notes

- **400-pool ceiling.** The 1.5% SS rate (6 / 400) and 5% combined SS+S rate (20 / 400) are *already* the strongest of any lane. The gap below is not "lift the lane to be competitive" — it's "convert the borderline-A roles to S and the borderline-S roles to SS within an already-strongest lane".
- **Sponsorship is solved at the SS level.** Every SS anchor sponsors. The blockers are not sponsor-shape; they are stack-shape (#1-6) and pedigree-shape (the Senior / Staff IC roles in the C/F tail, which are out-of-scope for this cycle).
- **Rust-concentration is doing 80% of the lifting.** The grade-jobs rubric's stack-concentration carveout fires across nearly every S/SS reasoning — Cernio + Nyquestro + Aurix + Image Browser + NeuroDrive + Vynapse (paused) + Xyntra (dormant) + Zyphos (dormant) clears the "≥3 active projects in role's primary stack" floor for any Rust-named role. This is the rubric reading the portfolio's true shape correctly. The flip side is that the moment the role is *not* Rust-primary (Go-native Confluent, Java-primary Palantir backend, Python-primary AI-infra), the carveout weakens — closure #2 (Go) and continued OSS engagement (#8) are what compound when the stack-concentration argument has nothing to grip.
- **OSS umbrella is undervalued by the absolute-frame grader.** The `Capataina/OpenSourceContributions` umbrella architecture + the +1864-LOC burn PR (approved) + tinygrad / alloy engagement is *itself* a meta-signal that the grader still treats as a single bullet. Cloudflare / HashiCorp / Grafana / MongoDB / PostHog / Proton specifically value visible-GitHub-portfolio engagement at a higher multiplier than the rubric currently encodes — for these companies the OSS umbrella IS the application material.

---

**Summary:** 78 K8s + 61 kernel + 43 Terraform + 24 Raft + 23 operator + 13 ClickHouse + 10 Prometheus / consensus / formal-verification JD mentions name the closeable surface. Strengths cluster on Cernio + Nyquestro + Image Browser + Aurix + NeuroDrive + OSS umbrella — the lane's S/SS results lean almost entirely on these. **Key recommendation: ship a Kubernetes operator + OpenTelemetry/Grafana instrumentation + Kani proofs over the next 6-8 weeks (combined cost ~7 weeks of evenings); this single bundle converts the most A-tier roles to S and the most S-tier to SS-borderline of any closure ordering.**
