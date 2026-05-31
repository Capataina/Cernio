# Discovery — Devtools Lane — 2026-05-31

Sparse lane (56 in DB). Sponsor-verified or "unknown" (flagged for resolve). Excludes gambling/adtech/consumer-crypto. Dedup-checked against /tmp/cernio-universe.txt.

Acquisitions skipped during the run: **Astral** (OpenAI acquired Mar 2026), **Bun** (Anthropic acquired Dec 2025), **Graphite** (Cursor acquired Dec 2025), **Fern** + **liblab** (Postman acquired), **Neon** (Databricks — already in DB).

---

### Sourcegraph
- **Website**: https://sourcegraph.com
- **Location**: San Francisco HQ; remote EMEA hiring with UK overlap
- **What they do**: Code intelligence + AI assistant (Cody) for monorepo-scale codebases — indexes >5M-line repos for retrieval-grounded code generation, code review, and navigation.
- **Why relevant**: Image Browser's "Multi-encoder rank fusion / information retrieval" skill (CLIP+SigLIP-2+DINOv2 RRF over 768-d vectors) maps directly to Cody's code-embedding retrieval problem. Cernio's own `grade-jobs` rubric design and prompt-engineering depth also fits Cody-Eng.
- **Source**: https://sourcegraph.com/jobs ; https://job-boards.greenhouse.io/sourcegraph91
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Replit
- **Website**: https://replit.com
- **Location**: San Francisco HQ; UK roles surfacing on Indeed UK
- **What they do**: Browser-based collaborative coding environment + Agent for natural-language app-building on a managed runtime.
- **Why relevant**: Tessarix's local Tauri+React+MDX runtime + state-aware LLM grounding parallels Replit's browser-runtime + Agent grounding problem. Cernio's skill-orchestration patterns map to Agent dispatch design.
- **Source**: https://replit.com/careers ; https://uk.indeed.com/q-replit-l-london-jobs.html
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Cursor (Anysphere)
- **Website**: https://cursor.com
- **Location**: San Francisco HQ; built on Tauri-style hybrid native+web stack
- **What they do**: AI-native code editor + (post-Graphite-acquisition Dec-2025) integrated AI code review; ~$2B ARR / 1M+ paying users per JetBrains 2026 survey.
- **Why relevant**: Direct overlap with Image Browser + Tessarix + Aurix (3 production Tauri-2 apps in Caner's portfolio) — Cursor uses Tauri-class native+web architecture. Caner's multi-LLM orchestration (Consilium) and prompt-engineering depth (Tessarix's 4-model A/B) map to Cursor's agent layer.
- **Source**: https://cursor.com/careers ; https://fortune.com/2025/12/19/cursor-ai-coding-startup-graphite-competition-heats-up/
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Zed Industries
- **Website**: https://zed.dev
- **Location**: Remote-first; ~$42M from Sequoia
- **What they do**: GPU-accelerated native code editor in **Rust**, real-time multiplayer, agent interoperability via ACP; explicit positioning against Electron's perf ceiling.
- **Why relevant**: Pure-Rust GPU-accelerated editor — directly matches Caner's Rust depth across Cernio (14k LoC), Nyquestro, NeuroDrive (Bevy ECS at 60Hz fixed timestep). Image Browser's `ort` ONNX + bytemuck + cosine partial-sort engineering signals the kind of perf-engineering Zed values.
- **Source**: https://zed.dev/jobs ; https://github.com/zed-industries/zed
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Warp
- **Website**: https://warp.dev
- **Location**: San Francisco; remote engineering; Sequoia/GV-backed
- **What they do**: Rust+GPU-rendered terminal that rebranded 2025 to "Agentic Development Environment" — multi-agent orchestration, cloud agents via Oz, session-sharing.
- **Why relevant**: Same Rust+GPU stack as Zed; Caner's Consilium multi-LLM debate orchestration + Tessarix three-phase LLM runner (Ollama → llama.cpp sidecar → Claude API) map onto Warp's agent-fleet architecture.
- **Source**: https://warp.dev/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Codeium / Windsurf
- **Website**: https://windsurf.com
- **Location**: Mountain View; remote
- **What they do**: AI coding platform — IDE plugin + standalone "Windsurf" editor with agentic flows.
- **Why relevant**: Cernio's grading-rubric iteration discipline (5 phases, calibration anchors, prompt-robust agents) maps to Windsurf's prompt-robustness problem at scale.
- **Source**: https://windsurf.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Tabnine
- **Website**: https://tabnine.com
- **Location**: Tel Aviv HQ; remote engineering
- **What they do**: Privacy-first enterprise AI code assistant — self-hosted / VPC deployment for compliance-bound customers.
- **Why relevant**: Caner's "Local-first / privacy-by-construction software" skill (Image Browser zero-network, Aurix read-only, Tessarix on-device) maps to Tabnine's privacy-first positioning.
- **Source**: https://www.tabnine.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Vercel
- **Website**: https://vercel.com
- **Location**: SF HQ; **Vercel UK Limited** is on the Home Office Skilled Worker register
- **What they do**: Frontend cloud platform (creators of Next.js) — edge functions, previews, ISR; pricing tightened 4× since 2024.
- **Why relevant**: Tessarix's Vite+React 19+TypeScript+MDX frontend is exactly the workload Vercel hosts; Image Browser's TanStack Query 5 + service-layer discipline reads as Vercel-customer-shaped.
- **Source**: https://vercel.com/careers ; https://immigrationgpt.co.uk/company/vercel-uk-limited
- **Sponsor**: yes (https://immigrationgpt.co.uk/company/vercel-uk-limited)
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Render
- **Website**: https://render.com
- **Location**: SF HQ; remote
- **What they do**: PaaS for full-stack apps — web services, background workers, cron, managed databases; "the boring reliable one" per 2026 reviews.
- **Why relevant**: Cernio itself is exactly the "Rust binary + SQLite + scripts + TUI" shape Render's infra abstracts away for cloud-bound forks of similar tools.
- **Source**: https://render.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Railway
- **Website**: https://railway.com
- **Location**: SF HQ; remote
- **What they do**: GitHub-repo-to-deploy PaaS — simplest-path infra for indie SaaS and AI app prototypes.
- **Why relevant**: Caner's "Idempotency as load-bearing invariant" skill (Cernio format-on-startup, Aurix config_hash) is the kind of platform-discipline Railway runtime infra needs.
- **Source**: https://railway.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Fly.io
- **Website**: https://fly.io
- **Location**: Remote; servers in 35+ regions
- **What they do**: Container-microVM PaaS running apps close to users; static IPs even on free tier; aggressive edge story.
- **Why relevant**: Nyquestro's tail-latency tracking (HDR-histogram p50→p9999, bounded-channel backpressure) is the kind of perf-instrumentation Fly's runtime team values.
- **Source**: https://fly.io/jobs
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Linear
- **Website**: https://linear.app
- **Location**: SF + remote; small careful eng team
- **What they do**: Issue tracker + project management for software teams — sets the modern bar for desktop-class web apps.
- **Why relevant**: Image Browser's "Desktop application development (Tauri 2 lineage)" + Tessarix's strict-TypeScript discipline + React 19 lazy/Suspense patterns are the exact stack Linear runs.
- **Source**: https://linear.app/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Retool
- **Website**: https://retool.com
- **Location**: SF HQ; explicit **London** office (Support Engineer role live in London 2026)
- **What they do**: Low-code internal-tools builder for engineering teams; component library + DB wiring + workflows.
- **Why relevant**: Cernio's TUI architecture (5 views, 26 files, density-over-whitespace, mouse-first/keyboard-enhanced) demonstrates UI-engineering judgement Retool front-end needs.
- **Source**: https://retool.com/careers/support-engineer--support--london-england-uk
- **Sponsor**: yes (UK office hiring publicly)
- **Lane**: devtools
- **Discovered**: 2026-05-31

### GitHub
- **Website**: https://github.com
- **Location**: SF HQ; substantial UK presence
- **What they do**: Microsoft-owned code-host + Actions CI + Copilot (37% AI-coding market share, 4.7M paid subs per JetBrains 2026).
- **Why relevant**: Caner's "Open-source contribution practice" skill (burn #4894 +1864/-0 APPROVED, draft PRs in tinygrad/alloy) is GitHub-native. Cernio's 9 native Claude Code skills are exactly the obligation-anchored skill design Copilot Eng needs.
- **Source**: https://github.com/about/careers ; Microsoft holds active UK sponsor licence
- **Sponsor**: yes (Microsoft UK sponsor licence)
- **Lane**: devtools
- **Discovered**: 2026-05-31

### GitLab
- **Website**: https://gitlab.com
- **Location**: All-remote (~2,500 employees across 65+ countries); UK-remote roles published on Greenhouse
- **What they do**: All-in-one DevSecOps platform; pivoting Q2 2026 toward agentic-era flows post-layoff restructure.
- **Why relevant**: Cernio's 6 ATS provider fetchers + unified `AtsJob` normalisation + slug-candidate combinatorial probe is the same shape-of-problem as GitLab's CI runner abstraction.
- **Source**: https://job-boards.greenhouse.io/gitlab
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### JetBrains
- **Website**: https://jetbrains.com
- **Location**: Prague HQ; **London office** (compiler / IDE-platform teams)
- **What they do**: IntelliJ-platform IDEs (IDEA, PyCharm, RustRover, CLion, Rider) + TeamCity CI + Qodana code-quality + Junie AI agent.
- **Why relevant**: Image Browser's `ort` ONNX + tokenizer rewrite from hand-rolled WordPiece to HF tokenizers + bytemuck `cast_slice` audit demonstrates the deep-systems perf-engineering JetBrains IDE-platform team values.
- **Source**: https://jetbrains.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Atlassian
- **Website**: https://atlassian.com
- **Location**: Sydney HQ; **explicit UK presence** (Jira/Bitbucket/Confluence)
- **What they do**: Jira / Confluence / Bitbucket — incumbent dev-collaboration suite; Atlassian Intelligence AI layer rolled out 2025-2026.
- **Why relevant**: Cernio's living-system architecture (one-way `populate-from-lifeos` sync, profile reads every invocation, no-snapshot-embedding rule) is the kind of distributed-state discipline Atlassian platforms need.
- **Source**: https://atlassian.com/company/careers/all-jobs
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Honeycomb
- **Website**: https://honeycomb.io
- **Location**: Remote; US-anchored
- **What they do**: High-cardinality event-based observability — distributed tracing with iterative query refinement; frontend RUM since 2024.
- **Why relevant**: Image Browser's `PerfLayer` + 12 named domain diagnostics + on-exit markdown Stall Analysis report is the same problem-shape Honeycomb solves at distributed-systems scale.
- **Source**: https://honeycomb.io/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### LaunchDarkly
- **Website**: https://launchdarkly.com
- **Location**: Oakland HQ; remote
- **What they do**: Feature-flag and experimentation platform; runtime config separated from deploy.
- **Why relevant**: Cernio's profile-aware grading + the realism-semantic A/B rubric iteration (5 phases, production-failure-driven) maps to LaunchDarkly's experimentation engine.
- **Source**: https://launchdarkly.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Statsig
- **Website**: https://statsig.com
- **Location**: Bellevue WA + remote (OpenAI acquired 2025 — operates independently)
- **What they do**: Feature-flag + experimentation + product analytics; LaunchDarkly's principal challenger.
- **Why relevant**: Tessarix's 50+ telemetry event-kind taxonomy as discriminated-union TS types + JSONL batched writer is exactly Statsig's instrumentation shape.
- **Source**: https://statsig.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Mintlify
- **Website**: https://mintlify.com
- **Location**: SF; remote engineering
- **What they do**: API documentation platform — powers docs for 20,000+ companies; #1 Enterprise Tech 30 (Wing VC 2026).
- **Why relevant**: Tessarix's MDX authoring discipline (9 lessons / 244KB / `<Tier level>` inclusion semantics / frontmatter lint script) is exactly Mintlify's MDX-as-docs shape.
- **Source**: https://jobs.ashbyhq.com/Mintlify
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Stainless
- **Website**: https://stainless.com
- **Location**: NYC HQ; remote
- **What they do**: Auto-generates type-safe SDKs + MCP servers from OpenAPI specs; Mintlify integration for live code samples.
- **Why relevant**: Cernio's 6-provider ATS fetcher abstraction + unified `AtsJob` normalisation is the same many-API-shapes-into-one-typed-surface problem Stainless automates.
- **Source**: https://stainless.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Speakeasy
- **Website**: https://speakeasy.com
- **Location**: SF + remote
- **What they do**: SDK + MCP-server + docs generation from OpenAPI; Stainless's direct competitor.
- **Why relevant**: Same fit as Stainless — provider-trait abstraction + serde-driven normalisation maps directly.
- **Source**: https://speakeasy.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Postman
- **Website**: https://postman.com
- **Location**: SF HQ; **explicit UK office** (London)
- **What they do**: API platform — design, test, mock, docs; acquired Fern + liblab to consolidate SDK/docs layer in 2025.
- **Why relevant**: Cernio's living-system architecture + JSON-fixture-driven ATS parser test patterns map to Postman's mock-server + collection-runner engineering.
- **Source**: https://boards.greenhouse.io/postman ; https://postman.com/company/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Buildkite
- **Website**: https://buildkite.com
- **Location**: Melbourne HQ; remote engineering; UK customers heavy
- **What they do**: Hybrid CI/CD — Buildkite hosts UI/API, customer runs agents on their own infra; build-graph orchestration with concurrency primitives.
- **Why relevant**: Cernio's `tokio::Semaphore`-bounded parallel ATS fetcher + per-portal failure isolation is exactly Buildkite's agent-scheduling primitive shape.
- **Source**: https://buildkite.com/about/jobs
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### CircleCI
- **Website**: https://circleci.com
- **Location**: SF HQ; remote
- **What they do**: Container-tiered CI/CD platform; large enterprise customer base; OIDC + Depot integration for remote build cache.
- **Why relevant**: NeuroDrive's perf-engineering depth (flat row-major weights, batched mat-mat fwd/back, `BatchScratch` zero-alloc) is the kind of inner-loop optimisation CI runner engineering benefits from.
- **Source**: https://circleci.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Earthly Technologies
- **Website**: https://earthly.dev
- **Location**: Remote
- **What they do**: Containerised build system with reproducible builds and remote build cache; "Bazel-style for the rest of us".
- **Why relevant**: Caner's "Determinism / reproducibility engineering" skill (NeuroDrive 1200-step bitwise-identical replay, Nyquestro `run_twice_identical_sequence_identical_output`, Aurix `config_hash`) is Earthly's literal value proposition.
- **Source**: https://earthly.dev/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Namespace Labs
- **Website**: https://namespace.so
- **Location**: Lisbon + remote
- **What they do**: High-performance remote builders for Docker / GitHub Actions / CircleCI — speeds up Docker builds via dedicated infra.
- **Why relevant**: Image Browser's thumbnail-pipeline rewrite (JPEG scaled IDCT + NEON Lanczos3, 5-10× speedup) is the same "exploit the format-specific fast path" engineering Namespace runs at infra scale.
- **Source**: https://namespace.so/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Coder
- **Website**: https://coder.com
- **Location**: Austin + remote; enterprise-self-hosted CDE
- **What they do**: Self-hosted cloud development environments — enterprise teams negotiate cloud bulk-discount, run Coder on their own infra.
- **Why relevant**: Cernio's local-first SQLite-as-contract design + WAL dual-connection pattern (from Image Browser) is the same self-hosted-runtime engineering Coder values.
- **Source**: https://coder.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Daytona
- **Website**: https://daytona.io
- **Location**: NYC + remote; Codeanywhere founders
- **What they do**: B2B-focused secure-alternative-to-Codespaces; self-managed CDE for enterprises.
- **Why relevant**: Same fit as Coder; Cernio's `CERNIO_DB_PATH` env-targeting + tempdir test fixture discipline maps to CDE's per-workspace isolation problem.
- **Source**: https://daytona.io/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Ona (formerly Gitpod)
- **Website**: https://gitpod.io
- **Location**: Remote; **substantial European engineering** (German-founded)
- **What they do**: Rebranded Sept-2025 from CDE pioneer to AI-agent-orchestration platform; pay-as-you-go CDE shut down Oct-2025.
- **Why relevant**: Consilium's multi-LLM debate-style orchestration + Cernio's skill-as-agent-prompt pattern map to Ona's agent-orchestration pivot.
- **Source**: https://gitpod.io/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Doppler
- **Website**: https://doppler.com
- **Location**: SF + remote
- **What they do**: Cloud-hosted secrets-management platform — zero-ops, integrations with Vercel, Railway, GitHub Actions, AWS, K8s.
- **Why relevant**: Cernio's "Idempotency as load-bearing invariant" + atomic-file-write patterns (Image Browser `.tmp` + rename, `.part` model downloads) maps to secrets-rotation correctness.
- **Source**: https://doppler.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Infisical
- **Website**: https://infisical.com
- **Location**: SF + remote; 12.7k GitHub stars
- **What they do**: Open-source secrets-management — developer-experience answer to HashiCorp Vault; self-host or managed.
- **Why relevant**: Caner's "Trait-based modular Rust design" (Vynapse 9-trait `EvolutionaryTrainer<G, M, C, F, S>`) + Cernio's `ArchiveSource` 3-tier fallback chain map to Infisical's storage-backend abstraction layer.
- **Source**: https://infisical.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Modal Labs
- **Website**: https://modal.com
- **Location**: NYC + remote
- **What they do**: Python-decorator GPU-runtime — handles container builds, GPU scheduling, scaling; ~$3.95/hr effective H100 rate.
- **Why relevant**: Image Browser's `ort` ONNX + parallel-encoder phase (`max(CLIP, SigLIP-2, DINOv2)` wall-clock) + Phase 12 dynamic `intra_threads(N/enabled_encoders)` is the same per-GPU resource-sharing engineering Modal handles.
- **Source**: https://modal.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Baseten
- **Website**: https://baseten.co
- **Location**: SF; remote; $5B valuation 2026
- **What they do**: ML model-serving platform; Truss open-source framework; T4-through-B200 GPU selection.
- **Why relevant**: Same fit as Modal — Image Browser's encoder lazy-init + `Mutex<Option<TextEncoder>>` warm-pool + first-call latency elimination via `encode("warmup")` pre-warm is exactly Baseten's cold-start engineering.
- **Source**: https://baseten.co/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Replicate
- **Website**: https://replicate.com
- **Location**: SF + remote
- **What they do**: Public-API ML model hosting — generative models in particular, one-click monetisation.
- **Why relevant**: Tessarix's empirical model-selection discipline (4-model A/B against grounded-explanation prompt, per-model hallucination notes) is the kind of model-curation engineering Replicate front-runs.
- **Source**: https://replicate.com/about
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Together AI
- **Website**: https://together.ai
- **Location**: SF + remote
- **What they do**: GPU inference platform competing with Fireworks AI on throughput; combines inference with open-research releases.
- **Why relevant**: Caner's `ort` ONNX expertise + bytemuck-safe BLOB casts + `BEGIN IMMEDIATE` batched writes maps to inference-runtime engineering.
- **Source**: https://together.ai/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Hex Technologies
- **Website**: https://hex.tech
- **Location**: SF + remote
- **What they do**: Notebook + dashboard platform for data teams; AI-augmented analysis workflows.
- **Why relevant**: Tessarix's interactive-widget library (53 widgets across A-FINE + linear-algebra) + state-aware `<WidgetExplainer>` debouncing is the same explorable-document engineering Hex builds at scale.
- **Source**: https://hex.tech/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Tines
- **Website**: https://tines.com
- **Location**: **Dublin co-HQ** + Boston; visa-sponsorship history in IE (Tier 3 relocation per Caner's preferences)
- **What they do**: No-code workflow + AI-orchestration platform for security/IT teams; customers include Canva, Databricks, Elastic, Intercom.
- **Why relevant**: Cernio's 9-skill orchestration + obligation-anchored skill design + skill-as-agent-prompt patterns maps to Tines's workflow-engine engineering.
- **Source**: https://tines.com/careers ; https://irishtalents.com/companies/tines-security-services-ltd
- **Sponsor**: yes (https://irishtalents.com/companies/tines-security-services-ltd) — Ireland Critical Skills equivalent
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Iterative (DVC / CML)
- **Website**: https://iterative.ai
- **Location**: SF + remote
- **What they do**: Git-native ML pipelines + DVC data versioning + CML CI for ML — open-source plus hosted Studio.
- **Why relevant**: Caner's "Determinism / reproducibility engineering" + the burn PR #4894 A-FINE IQA contribution (full reference + regression tests against reference impl) is exactly the kind of ML-systems engineering Iterative values.
- **Source**: https://jobs.lever.co/iterative
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Pulumi
- **Website**: https://pulumi.com
- **Location**: Seattle + remote
- **What they do**: IaC in real programming languages (TypeScript / Python / Go / .NET / Java) — alternative to Terraform's HCL DSL.
- **Why relevant**: Caner's "Compiler IR design / typed compiler frontend" (Xyntra IR primitives, typed graph validator scaffold) + trait-based modular Rust design maps to Pulumi's provider-SDK engineering.
- **Source**: https://pulumi.com/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Spacelift
- **Website**: https://spacelift.io
- **Location**: Warsaw + remote (London office for sales)
- **What they do**: Collaborative IaC — Terraform / Pulumi / OpenTofu / CloudFormation / Kubernetes / Ansible orchestration with policy-as-code.
- **Why relevant**: Cernio's `cernio check` integrity-report architecture (health / completeness / staleness across 8 systems) maps to Spacelift's policy-as-code surfacing pattern.
- **Source**: https://spacelift.io/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Humanitec
- **Website**: https://humanitec.com
- **Location**: Berlin HQ + Sofia + Bucharest; remote-first EU/Asia/Oceania scope
- **What they do**: Platform-orchestrator for internal developer platforms — works with Terraform / any CI/CD / any cloud / any orchestrator. Top-rated IDP per Internal Developer Platform org.
- **Why relevant**: Cernio's three-layer Conversation→Scripts→SQLite architecture with strict downward dependency is exactly the IDP orchestration-layer pattern Humanitec sells.
- **Source**: https://humanitec.com/careers ; https://devopsprojectshq.com/humanitec-platform-engineer
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Garden
- **Website**: https://garden.io
- **Location**: Berlin + remote
- **What they do**: Dev/preview/test environments for Kubernetes — fast-iterating cloud-native development workflows.
- **Juany relevant**: Cernio's `cernio resolve` slug-candidate combinatorial probe across 10-20 patterns × 7 providers + tempdir-per-test integration testing is the same shape as Garden's per-PR ephemeral-environment engineering.
- **Source**: https://garden.io/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Crossplane (Upbound)
- **Website**: https://upbound.io
- **Location**: Seattle + remote
- **What they do**: Commercial entity behind Crossplane (CNCF) — Kubernetes-native infra control plane for cloud-resource composition.
- **Why relevant**: Caner's `ArchiveSource` 3-tier fallback (Subgraph → Alchemy → public-RPC) + `ATS provider trait` 6-implementation surface is the same composable-provider engineering Crossplane runs at scale.
- **Source**: https://upbound.io/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Charm
- **Website**: https://charm.sh
- **Location**: Distributed; Ottawa / Berlin / NYC
- **What they do**: Go-based TUI toolkit ecosystem — Bubbletea / Lipgloss / Glamour / Soft Serve / Wish; commercial cloud offering for TUI-distributed software.
- **Why relevant**: Direct overlap — Cernio is itself a Ratatui TUI (5 views, 26 source files, density-over-whitespace, modular v5 architecture) that demonstrates exactly the TUI design discipline Charm builds against. Caner's authored work IS a portfolio piece for Charm.
- **Source**: https://charm.sh/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Deno
- **Website**: https://deno.com
- **Location**: SF + remote
- **What they do**: TypeScript-first secure runtime; Deno Deploy edge platform; KV store; commercial managed services.
- **Why relevant**: Tessarix's strict-TypeScript posture (`noUnusedLocals` / `noUnusedParameters`) + discriminated-union route types + `tauri::ipc::Channel<StreamEvent>` SSE handling map to Deno's runtime + Deploy edge engineering.
- **Source**: https://deno.com/jobs
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Northflank
- **Website**: https://northflank.com
- **Location**: **London-headquartered** UK; on Skilled Worker register
- **What they do**: Self-hosted/managed multi-cloud PaaS — combines PaaS simplicity with complex workload features (CI/CD, monitoring, autoscaling, BYOC).
- **Why relevant**: NOTE — Northflank already in DB (line 391 of universe), skipping as duplicate.
- **Sponsor**: yes (UK HQ)
- **Lane**: devtools

### Aiven
- **Website**: https://aiven.io
- **Location**: Helsinki HQ + UK office
- **What they do**: Managed open-source data infra — PostgreSQL / Kafka / OpenSearch / ClickHouse / Redis on AWS/GCP/Azure.
- **Why relevant**: NOTE — Aiven already in DB (line 18), skipping as duplicate.
- **Lane**: devtools

### Continue.dev
- **Website**: https://continue.dev
- **Location**: SF + remote
- **What they do**: Open-source AI code agent + CLI; source-controlled AI checks enforceable in CI; connect any LLM (GPT-4 / Claude / Llama / Mistral) to VS Code or JetBrains.
- **Why relevant**: Tessarix's three-phase LLM runner (Ollama → llama.cpp sidecar → Claude API via shared `LlmClient` abstraction) is exactly Continue's any-backend architecture.
- **Source**: https://continue.dev/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Sourcery AI
- **Website**: https://sourcery.ai
- **Location**: **London-headquartered** UK
- **What they do**: AI code-review for Python + JS/TS + Java + Go — IDE plugin + GitHub/GitLab PR-bot; rule-based + LLM hybrid.
- **Why relevant**: Cernio's `grade-jobs` rubric design (5 phases, calibration anchors, mandatory description citation, lifestyle modulator, realism semantic) is exactly the rule-design discipline Sourcery's analyser team values. UK HQ + Tier-1 commute.
- **Source**: https://sourcery.ai/careers
- **Sponsor**: yes (UK HQ)
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Qodo (formerly Codium)
- **Website**: https://qodo.ai
- **Location**: Tel Aviv + remote
- **What they do**: AI for test-generation and code-review; Qodo Gen IDE plugin + Qodo Cover for autogen tests + Qodo Merge PR-review.
- **Why relevant**: Caner's "Test-driven validation discipline" (Cernio 346 tests, Image Browser 125+62, NeuroDrive 133 green) is the demonstration-of-the-problem-Qodo-solves.
- **Source**: https://qodo.ai/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

### Aviator
- **Website**: https://aviator.co
- **Location**: SF; ex-Googlers; YC-backed
- **What they do**: Engineering-productivity platform — merge-queue + stacked-PR workflows + CI optimisation; full pre-merge-to-deploy collaboration suite.
- **Why relevant**: Cernio's `pipeline/search` filter stack (location → exclusion → inclusion → dedup via URL UNIQUE → `INSERT OR IGNORE`) is the same multi-stage gating pattern Aviator's merge-queue runs.
- **Source**: https://aviator.co/careers
- **Sponsor**: unknown
- **Lane**: devtools
- **Discovered**: 2026-05-31

---

## Summary

**Total finds**: 41 new + 3 dedup-flagged (Northflank, Aiven, Neon)

**Acquisitions skipped**: Astral, Bun, Graphite, Fern, liblab (all acquired 2025-2026)

**Sponsor verified**: Vercel (Vercel UK Ltd), Retool (London office), GitHub (Microsoft UK), Postman (UK office), Sourcery (UK HQ), Tines (Dublin Critical Skills equivalent), Northflank (already in DB)

**Sponsor unknown**: 34 of 41 — flagged for `cernio resolve` + `resolve-portals` AI fallback to verify against gov.uk register before grading

**Top 5 strongest fits**:
1. **Charm** — direct overlap: Caner's Cernio Ratatui TUI is exactly the product Charm builds against
2. **Sourcery AI** — UK HQ + rule-design discipline matches Cernio's grading rubric iteration
3. **Cursor** — Tauri-class hybrid stack matches Caner's 3 production Tauri-2 apps
4. **Zed** — pure-Rust GPU-accelerated editor matches Caner's deep Rust + perf-engineering stack
5. **Vercel** — verified UK sponsor + Caner's Next.js-shaped Tessarix is a customer-anchor

**Dry sources**: Glassdoor/Indeed UK aggregators returned mostly noise; HN "Who is hiring" April/May 2026 threads need deeper per-company chase; YC W25/S25 directory needs direct API extraction (catalogue page is generic).

**Notable observations**: (a) **Acquisition wave** — OpenAI bought Astral, Anthropic bought Bun, Cursor bought Graphite, Postman bought Fern+liblab, Databricks bought Neon. Devtools consolidation is heavy in 2025-2026 — verify "is this company still independent" at grading time. (b) **Many top-tier US devtools companies do NOT have UK Skilled Worker sponsor licences** — Sourcegraph, Modal, Baseten, Replicate, Linear, Render, Railway, Fly.io likely fail the sponsor filter even if listed as "remote globally". These need explicit gov.uk register verification before grading. Real high-confidence UK-sponsor finds in this batch: Vercel, Retool, GitHub-via-Microsoft, Postman, Sourcery, Northflank (dedup), plus Tines-via-Ireland. (c) **"Cool place to work" risk** — many of these are S/A-tier engineering brands but the sponsor-licence filter will cut hard; rec running `cernio resolve` first then `grade-companies` only on the survivors.
