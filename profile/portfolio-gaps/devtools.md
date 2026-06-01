---
title: Portfolio Gaps — Devtools / DX
lane: devtools
last_updated: 2026-06-01
graded_pool: 97 jobs (S=4, A=3, B=14, C=13, F=63)
seed_source: grade-jobs Phase 3 regeneration
---

# Portfolio Gaps — Devtools / DX lane

Lane-internal calibration after 97 graded jobs. The lane reads the candidate as **a devtools builder already** — the F count is overwhelmingly visa-driven (Vercel × 22, Cursor × 4 SF-only, LaunchDarkly × 6 US, Twilio × 11 US) rather than fit-driven. Among the realistically landable subset (S+A+B = 21 jobs), the binding gaps are stack-specific languages, build-system depth, and platform/edge-runtime exposure; the binding strengths are Rust + multi-surface product shipping + Tauri-class desktop deployment + Cernio itself as a meta-devtool.

---

## Open gaps

### Stack-language gaps that cap real applications

The lane's realistically-landable seats name specific primary stacks the portfolio does not hold. Each one cost a Q3a deduction in the graded pool:

| Stack | Cost in pool | Where it bit |
|---|---|---|
| **Go** | A→B-borderline | Paddle DX Go (id=680, "Go engineer to join our team" — Caner has zero Go projects); GitHub UK SWE II (id=1216, Ruby + Go monorepo); incident.io Product Engineer / Platform (id=1160/1162, Go backend) |
| **Kotlin** | C-fix | JetBrains Ktor (id=546, zero Kotlin in `skills/languages.md`); Hyperexponential Full Stack Kotlin/React (id=479); incident.io Mobile (id=1161, Kotlin/Android half) |
| **Bazel** | B-cap | Fractile Bazel/Build Systems (id=342/343, "3+ years with build systems, Strong experience with Bazel" — Caner has Cargo workspace + CMake but no Bazel) |
| **Kubernetes / Terraform / GCP** | A→B (incident.io 1162), C (Cursor 182) | incident.io Platform Engineer explicitly: "owns the infrastructure, deployment systems, and CI/CD pipelines"; flagged in Phase 2 relativity as "the no-Kubernetes / no-Docker / no-cloud cluster" |
| **Ruby** | A-friction | GitHub UK SWE II (id=1216, Ruby + Go monorepo) |
| **Clojure** | B-cap | CircleCI Software Engineer (id=144, historic Clojure core) |
| **Java / JVM** | B-cap | Dataiku Core (id=227/229, Java + TypeScript + React on JVM core) |
| **C#** | C-fix | Reincubate Camo (id=1450, "Software Engineer (C#, remote GMT)") |
| **Next.js production** | B-cap | Vercel FDE (id=1067, "complex Next.js migrations" — incidental React only, no Next.js shipped) |

### Lane-canonical capability gaps

- **Public, installable open-source devtool.** Cernio is itself a devtool but private. Every pinnacle in the lane (HashiCorp, Vercel, PostHog, Sentry, Linear, Replicate) hires on the public-OSS signal as load-bearing; the GitHub II row (id=1216) and the Bloop Rust/Compiler row (id=1285) explicitly cited `merged tinygrad PR` and `burn A-FINE PR (1864 lines)` as the strongest signal types available. The gap is not "shipping software" — it is "shipping software developers can `cargo install` and depend on."
- **IDE / LSP / editor-plugin internals.** Cursor's S-tier roles (177 Bugbot, 178 Enterprise Platform) and JetBrains' core platform roles all assume editor-internals fluency — VS Code extension APIs, LSP wire protocol, Tree-sitter incremental parsing, tsserver-class language servers. Nothing in the portfolio touches editor internals; Image Browser's Tauri shell is closest but is an app, not an editor.
- **Build-system depth at scale.** Bazel (Fractile), Nx / Turbo monorepo tooling, custom Cargo build-script orchestration past the trivial. Cernio's `build.rs` integrity guard is a positive signal (Phase 2 cited "build-time integrity guard with 21 tests") but is single-crate; pinnacle monorepo build-tool roles want multi-language graph reasoning.
- **Edge-runtime knowledge.** Cloudflare Workers, Vercel Edge Functions, Deno Deploy, Bun. Vercel's CDN / Workflows / AI Gateway rows (id=1075, 1082, 1072) and Cloudflare-class edge-compute roles all assume isolate-runtime mental models, V8 isolate sandboxing, and the cold-start/warm-start tradeoff space. Cernio's axum embedded server is a long-running process model — adjacent but not edge.
- **DevEx telemetry / engineering-productivity metrics.** Optiver's DevX role (id=670, A) explicitly named "Track emerging trends in Developer Experience and AI"; Honeycomb / DataDog / Sentry / PostHog all hire on the "instrument the dev loop and act on the numbers" axis. Cernio's `cernio check` integrity surface is the closest analogue; production DevEx telemetry (PR-cycle time, build-failure attribution, test-flake detection, p95 CI wait) is unevidenced.
- **CLI-tool packaging at scale.** `cargo install`-ready distribution, GitHub Releases binary builds, Homebrew formulas, npm-installable Node CLIs, signed Windows binaries, notarised macOS bundles, cross-architecture release matrices. Cernio runs as a local binary but is not published. Pinnacle CLIs (`gh`, `vercel`, `posthog-cli`, `wrangler`, `turbo`) ship through these pipelines.
- **Package-manager internals.** npm / yarn / pnpm resolution algorithms, Cargo lockfile semantics, Python wheel resolution, Bun's npm-compatible install graph. Anysphere, Vercel, GitHub Codespaces, JetBrains Junie all touch this surface. Cernio uses package managers; it does not implement one.
- **Mobile devtools surface.** React Native / Swift / Kotlin Android. incident.io Mobile (id=1161, B), Vercel Mobile (id=1069, F), Granola Windows / iOS (id=374) all flagged "zero mobile surface in the portfolio" as the Q3a kill.

### Function-exclusion gaps (not closable by portfolio, by design)

These appeared 16× in the F pool and are documented as wrong-function-type per `career-goals.md` role-truth-at-hire — keep visible so future runs don't waste an application:

- Developer Advocate / Developer Marketer / DevRel (Paddle 679, ServiceNow/MoveWorks 881, TurinTech 1337, PostHog 756, Lightdash 597/598, Twilio 1027, Vercel 1083/1084, JetBrains 535/537/538/539/540, Parity 704).
- QA / Automation / Demo Engineer (BVNK 103 location-also, TaiNa 994, Synthesized 1414, Improbable 485, Cursor 181).
- Forward-Deployed / Developer Success / Customer-embedded (Vercel 1065/1066/1067/1068, Cursor 179, SingleStore 882/883).

---

## Confirmed strengths

### Cernio itself is the lane-strongest anchor

Cernio cited as the **devtools pinnacle** in 14 separate Phase 1 fit assessments in this pool. Specific cite shapes that landed:

- **id=205 (DRW S):** "Tools engineering for traders maps directly to projects/cernio.md... 290KB of skill reference documentation, obligation-anchored skill design"
- **id=751 (PostHog S):** "Cernio (active, ~14k LOC Rust, 5 TUI views + axum web UI + 9 native Claude Code skills, 346 tests) is exactly the kind of multi-surface end-to-end product-engineering evidence PostHog hires on. Realistically landable on the strength of the portfolio."
- **id=1160 (incident.io S):** "Cernio (~14k LOC Rust + axum web embed) and Aurix (Tauri 2 + React 19) demonstrate full-stack shipping across the same shape incident.io values."
- **id=670 (Optiver DevX A):** "Caner's portfolio... is exactly the kind of developer-tooling engineering DevX teams value. Cernio CLI surface + build-time integrity guard + 346 tests + 21 schema tests demonstrates the tooling discipline DevX teams want."

Specific Cernio surfaces the lane reads as devtools-canonical: the 6-command CLI (`resolve` / `search` / `check` / `clean` / `import` / `format`), the trait-based 6-fetcher ATS provider abstraction with shared retry + backoff, the 9 native Claude Code skills (obligation-anchored skill-design), the 5-view Ratatui TUI, the embedded axum web UI with `cernio snap`, the build-time integrity guard, the SQLite WAL + 346-test discipline.

### Image Browser as the multi-source ML-tooling anchor

- id=1285 (Bloop S, Rust/Compiler): not directly cited but the "Tree-sitter parsing, on-device MiniLM embeddings" stack pattern matches Image Browser's "ONNX inference + multi-source data + 33 .ts/.tsx + 28 backend .rs" exactly.
- Phase 1 cites: "local-first / privacy-by-construction concept Caner has built across image-browser.md / cernio.md" (id=1450 Reincubate Camo).
- Image Browser demonstrates the on-device-ML-with-real-UI pattern Granola, Bloop, and PostHog AI all hire on.

### Aurix as DeFi-meets-Tauri tooling anchor

- Cited in id=1160 (incident.io): "Aurix (Tauri 2 + React 19) demonstrate full-stack shipping."
- 19 IPC handlers + standardised CommandError envelope + manual DTO mirror — exactly the kind of typed-contract devtools engineering DX teams value.
- 45 .tsx + 18 .ts ~9k LOC frontend shows React work that is genuinely production-class, not toy.

### Performance Profiler — overlooked DevX anchor

- id=670 (Optiver DevX A): "projects/performance-profiler.md (status:active — live IL injection + browser SPA dashboard + LiteDB persistence with four-layer crash safety) is DevX work by domain — engineering productivity tooling for other engineers."
- This project is **the cleanest DevX anchor in the entire portfolio** and is undersold. IL injection + per-tick CPU/allocation attribution is the literal shape Optiver / Bloomberg / Jane Street DevX teams build internally.

### Rust as the rarest-keyword lever

- id=1285 (Bloop S): "**Rust, Compiler** is exactly the rare double-keyword combination... Rust roles are rare and Caner's Rust [Proficient] portfolio is the single strongest evidence base in the universe. Bloop is portfolio-readable and the role-specificity narrows the applicant pool sharply."
- Direct anchors cited: Cernio ~14k LoC Rust, Nyquestro deterministic engine zero-unsafe, Vynapse 4-crate Cargo workspace, Xyntra (compiler), the merged tinygrad PR (LSTM in `onnx.py`), the burn A-FINE PR (1864 lines, CLIP-ViT backbone, maintainer-approved).
- Rust + compiler / Rust + IDE-tooling / Rust + build-systems is the highest-leverage application pattern in the lane.

### CLI-design discipline

- Cernio's 6-subcommand surface (`resolve` / `search` / `check` / `clean` / `import` / `format`) demonstrated clean subcommand framing.
- `cernio snap` for self-driven visual debug (per `feedback_self_driven_visual_verification.md` memory).
- Cited as devtools-canonical engineering pattern in 5 separate fit assessments.

### Multi-surface product-engineering shipping

PostHog (id=751) and incident.io (id=1160) both explicitly hire on this; the cite was: "5 TUI views + axum web UI + 9 native Claude Code skills + 346 tests... that PostHog hires on." Caner has shipped backend + frontend + storage + AI-orchestration + TUI + native skills across one repo — pinnacle PE rows read this as the signal.

---

## Closure prescriptions

Ordered by leverage (highest first), with the specific lane row that would shift on closing each gap:

1. **Publish one Cernio component as a `cargo install`-able crate.** The 6-fetcher ATS-provider abstraction (Greenhouse / Lever / Ashby / Workable / SmartRecruiters / Workday) with shared retry + slug normalisation is the most self-contained piece. Crate name `ats-fetch` or similar. Closes: PostHog 751 (lane S, public-OSS signal), Bloop 1285 (Rust crate-author signal), Optiver DevX 670 (CLI distribution discipline). **Highest single-action leverage in the lane.**
2. **Learn Go to comfort, anchor one project in it.** A small Go rewrite of one Cernio fetcher (Lever or Ashby — both have clean JSON APIs) gives the GitHub UK (1216), Paddle DX (680), incident.io Product Engineer (1160), and incident.io Platform (1162) rows a real Q3a anchor. Go-from-Rust is a 2-3 week investment.
3. **Ship one Bazel rule contribution upstream OR a Bazel-driven monorepo demo.** Fractile (342/343) explicitly asks "Contributing upstream to Bazel rules" — a `rules_rust` PR or a public `polyglot-monorepo-demo` repo would convert both Fractile rows from B to A and open the Spotify / Snowflake / Pinterest internal-platform lane.
4. **Ship a Cloudflare Workers or Vercel Edge Function-backed sub-project.** Even a small one: a Cernio status-page edge worker, an isolate-runtime experiment. Closes the Vercel CDN (1075), Workflows (1082), AI Gateway (1072) rows (currently F on location only) and unlocks Cloudflare UK as a future lane.
5. **Anchor Performance Profiler more visibly in `projects/index.md` and the public GitHub README.** The Optiver DevX cite proves it is undersold. A one-paragraph DevX-framing in the project's `_Overview.md` + a screenshot of the SPA dashboard would shift the framing from "dormant gaming tool" to "production DevX artefact" — exactly what Bloomberg / Jane Street / Optiver hire on.
6. **Edit the existing `cargo install` discipline into the visible README.** Even if Cernio stays private, the README can show the binary-distribution shape (notarised macOS, signed Windows, cross-arch). Closes the CLI-distribution-at-scale gap without requiring a public package.
7. **One LSP / Tree-sitter touchpoint.** A small Tree-sitter grammar contribution, an LSP-protocol experiment, or a VS Code extension that exposes Cernio's TUI views via webview. Closes the editor-internals gap that Cursor and JetBrains' core platform rows assume.
8. **Kotlin is a low-priority closure.** JetBrains Ktor (546) failed on location not stack; Hyperexponential Kotlin (479) is one row of many. Defer.
9. **Mobile is a deliberate non-closure.** incident.io Mobile (1161) and Vercel Mobile (1069) name the gap; the portfolio's identity is desktop/CLI/web, and mobile would dilute that. Mark these rows as permanent C and move on.

---

## Pinnacle anchors

Lane-canonical pinnacles and how the portfolio reads against each:

| Pinnacle | Lane | What Caner has | What's missing | Realistic landability |
|---|---|---|---|---|
| **PostHog** | Devtools-OSS-analytics | id=751 graded S explicitly. Cernio's "5 TUI views + axum web UI + 9 native Claude Code skills, 346 tests" cited verbatim as the hiring signal. London HQ, remote-first, portfolio-hires. | Public OSS devtool; ClickHouse/columnar-DB exposure | **High** — already S-graded, apply directly |
| **incident.io** | Devtools-SRE | id=1160 S; Cernio + Aurix cited as the multi-surface evidence | Go for the backend half; Kubernetes / Terraform / GCP for the Platform-engineer row | **High** for Product Engineer; medium for Platform |
| **GitHub UK** | Devtools-mega | id=1216 A; the systems-engineering and Cernio's 6 ATS fetchers cited as integration-engineering anchor | Ruby + Go monorepo stack; Copilot-class AI surface (Caner has the AI-orchestration side via the 9 native skills) | **Medium** — SWE II is portfolio-friendly but Ruby/Go gap real |
| **Vercel** | Devtools-frontend-cloud | id=1067 B (FDE-shape friction); other Vercel rows F on US location | Production Next.js shipped; edge-runtime knowledge | **Low currently** (US-anchored), high if UK-Vercel surface ever opens |
| **HashiCorp** | Devtools-infra | Not in the graded pool but lane-canonical. Cernio's CLI design + trait-based provider abstraction is on-shape | Go (every HashiCorp tool is Go); production cloud-infra distribution discipline | **Medium** with Go closure |
| **Sentry** | Devtools-observability | Not in pool. Performance Profiler is the closest anchor — IL injection + per-tick attribution is exactly Sentry's depth | Production observability tooling at scale; Python/Rust SDK authorship | **Medium** with Performance Profiler resurfacing |
| **Grafana Labs** | Devtools-observability | Not in pool. Cernio's TUI + dashboard pattern is on-shape | Go; PromQL/time-series query authorship | **Medium** with Go closure |
| **Linear** | Devtools-product | Not in pool. Aurix + Image Browser + Tessarix three-Tauri stack reads as the multi-surface product-engineering Linear hires on | Real-time collaboration (CRDTs, Y.js-class sync); shipped consumer product polish | **Medium-high** on PE shape |
| **Replicate** | Devtools-ML-inference | Not in pool. Image Browser ONNX inference is on-shape; Cernio's 9-skill AI orchestration is adjacent | Production ML-serving infra (Cog, Triton, vLLM); Python ML-engineering tenure | **Medium** with one ML-serving anchor |
| **Modal** | Devtools-ML-compute | Not in pool. Python + Rust + Cernio orchestration reads on-shape | Kubernetes; container/sandbox internals; serverless GPU scheduling | **Medium** with K8s closure |
| **Replit** | Devtools-IDE | Not in pool. Cernio's TUI + skill-system + Aurix's typed IPC are on-shape | Editor internals; container/sandbox; multi-user collaboration | **Low-medium** without LSP/editor anchor |
| **JetBrains** | Devtools-IDE-incumbent | id=546 + 534/535/537/538/539/540/544/545/547 — 10 rows, all F on UK-location-absence (Belgrade/Munich/Prague/Amsterdam) | UK office doesn't hire core platform; Kotlin for Ktor; .NET for Rider; would need relocation | **Low** without relocation; revisit if UK office opens core seats |
| **Bloop** | Devtools-niche-Rust | id=1285 S. Rust + Compiler keyword combo cited as "single strongest evidence base in the universe" | Specifically a Compiler / IR engineering anchor (Xyntra is dormant; resurfacing it would convert this from "stretch S" to "actively in the pool") | **High** — apply immediately, resurface Xyntra if possible |
| **HashiCorp / Anysphere / Cursor** | Devtools-AI-IDE | Cursor 177-182 all F or C on US visa, not on fit | US sponsorship pathway; otherwise the AI-orchestration + Rust shape lands | **Low currently** on visa; high on fit if UK-remote opens |

---

## Lane-internal calibration — 97-pool placement

| Tier | Count | What the band represents | Floor row | Ceiling row |
|---|---|---|---|---|
| **S** | 4 | Portfolio-perfect — Cernio / Image Browser / Aurix cited verbatim as the hiring signal in the fit assessment | DRW Cumberland (205) — devtools-for-trading dual-pinnacle bet | Bloop (1285) — Rust + Compiler rarest-combo |
| **A** | 3 | Strong axis fit, one closable gap | Paddle DX Go (680) — Go is the gap | GitHub UK SWE II (1216) — Ruby+Go gap, brand offsets |
| **B** | 14 | Real signal blocked by one specific stack/specialism gap or Q3b off-axis pattern | Reincubate (1450) — C# gap + small-brand | Vercel FDE (1067) — Vercel brand offsets FDE shape |
| **C** | 13 | Wrong function (DevRel/QA), wrong-stack, or hard-floor (Beacon 5y) | Various DevRel | Cursor Bugbot (177) — portfolio-perfect on fit, US-visa kill |
| **F** | 63 | 60+ visa-driven location fails (Vercel × 22 US, Twilio × 11 US, LaunchDarkly × 6 US, Cursor × 4 SF). Pool inflates F count without reflecting fit. | — | — |

**Key calibration insight:** The F count is misleading. If the visa-driven F bucket were filtered out, the lane distribution would be roughly **S:4 / A:3 / B:14 / C:8 / F:5** out of ~34 in-scope jobs — a far healthier distribution that correctly reads the candidate as a credible devtools applicant. Future discovery should deprioritise US-anchored devtools companies (Vercel, LaunchDarkly, Twilio, Cursor) until either US-visa pathway or UK-office expansion shifts.

**Cross-tier signal:** Rust roles (Bloop S, DRW S Tools Engineering Rust-friendly, Fractile B with Rust mentioned) consistently outperform Kotlin / Ruby / Clojure roles in the pool. Future discovery should target Rust-heavy UK devtools shops (Materialize-UK, Cloudflare-UK Rust teams, Mozilla-UK, Embark Studios) as the highest-yield discovery surface.

---

**Summary line:** Lane reads the candidate as a credible devtools builder anchored on Cernio + Image Browser + Aurix + Performance Profiler; binding open gaps are public OSS distribution, Go fluency, build-system depth (Bazel), and edge-runtime knowledge; the single highest-leverage closure is publishing one Cernio component as a public `cargo install` crate, which would convert PostHog (S), Bloop (S), GitHub UK (A), and Optiver DevX (A) from "applies-on-portfolio" to "applies-with-the-canonical-lane-signal."
