---
name: Tessarix
status: active
source_repo: https://github.com/Capataina/Tessarix
lifeos_folder: Projects/Tessarix
last_synced: 2026-05-31
sources_read: 17
---

# Tessarix

## One-line summary

Local-first Tauri 2 + React 19 + MDX desktop application for teaching abstract technical concepts (image quality, linear algebra, eventually ML and quant finance) through narrative lessons fused with embedded interactive widgets, LLM-integrated assessment surfaces, and step-throughable visualisations.

## What it is

Tessarix is an interactive learning substrate: a desktop application that pairs `.mdx` lessons with React widgets, KaTeX math, tier-gated complexity, and locally-running LLM features so that abstract concepts can be manipulated rather than only read. The premise is that prose alone loses the dimension that makes visualisable concepts understandable, and the substrate is the bet on restoring that dimension. The product is designed around three URL-shaped pillars over the same content + same question bank — `/<topic>` (Teach), `/<topic>/quiz` (Quiz), `/<topic>/interview` (Interview) — but only the Teach pillar currently exists in code; Quiz and Interview are routing-and-persistence-blocked on SQLite that has not landed yet. Primary audience is the author himself; commercial-product framing is explicitly deferred until the substrate is proven on self.

## Architecture

Two runtime processes plus a build pipeline, all in one git repo:

```text
                ┌────────────────────────────────────────┐
                │            build-pipeline              │
                │   (pnpm + Vite + Cargo + tauri-cli)    │
                └─────────────┬──────────────┬───────────┘
                              │              │
                produces      │              │ produces
                              ▼              ▼
                      ┌─────────────┐   ┌────────────────┐
                      │  frontend-  │   │   tauri-host   │
                      │   shell     │   │  (Rust process │
                      │ (Vite+React │◄──┤   + Tauri 2)   │
                      │  + MDX)     │   │                │
                      └──────┬──────┘   └────────┬───────┘
                             │                   │
                             │  Tauri IPC        │
                             │  + Channel        │
                             └───────────────────┘
                              (3 LLM commands +
                               telemetry commands
                               currently registered)
```

- **Tauri host** (`src-tauri/`): Rust 2 + Tauri 2 binary that wraps the WebView, registers IPC handlers, owns the LLM client (`reqwest` + `tokio` + SSE streaming to Ollama at `http://localhost:11434/v1/chat/completions`), and owns the telemetry writer (session log directory).
- **Frontend shell** (`src/`): Vite + React 19 + TypeScript app mounted at `#root`. `App.tsx` is a hash router (`#/catalog`, `#/lesson/<slug>`) wrapped in `ErrorBoundary` + `TierProvider` + `SettingsProvider` + `MDXProvider`. `Layout.tsx` composes topbar + lesson column with `LessonTOC` (IntersectionObserver-driven active section) and `ReadingProgress`.
- **Content layer**: `@mdx-js/rollup` plugin chain (`enforce: "pre"`, then `remark-math` → `rehype-katex` → `providerImportSource: "@mdx-js/react"`); lessons live as `.mdx` files in `src/lessons/`; `src/lessons/registry.ts` declares the lesson set with `Component: lazy(importer)` plus an eager `frontmatter: Promise<LessonFrontmatter>` for catalog rendering.
- **State providers**: `TierContext` (essential/standard/complete with inclusion semantics + localStorage persistence), `SettingsContext` (fontSize / contentWidth / density / reducedMotion → CSS variables + localStorage).
- **IPC surface**: `llm_chat_complete`, `llm_chat_stream`, `llm_chat_json`, plus telemetry write commands. Streaming uses `tauri::ipc::Channel<StreamEvent>` with `Token | Done | Error` variants.

Dependency direction: build-pipeline is upstream of both runtime subsystems; frontend-shell and tauri-host are peers connected by Tauri IPC; the `tauri-plugin-opener` plugin is installed on both sides but currently unused (kept as cheap optionality).

Coupling points worth knowing about:

- Shared literal port `1420` between `vite.config.ts::server.port` and `tauri.conf.json::build.devUrl`; `strictPort: true` is deliberate so port collisions fail hard rather than rendering a connection error.
- `Cargo.toml::[lib].name = "tessarix_lib"` (suffixed) is required to avoid a Windows lib/bin name collision (rust-lang/cargo#8519); `main.rs::tessarix_lib::run()` matches.
- MDX plugin `enforce: "pre"` ordering is load-bearing — `.mdx` must transform to JSX before `@vitejs/plugin-react` picks it up.
- Lesson context for the LLM chatbot is **scraped from the rendered DOM** at request time via `src/lib/llm/dom.ts` (cap ~8KB), not injected from MDX source — hidden tier sections are caught because the scrape walks the whole tree.

## Subsystems and components

### Tauri host (`src-tauri/src/`)

`main.rs` (Windows console suppression + `tessarix_lib::run()`), `lib.rs` (`tauri::Builder` with opener plugin and `generate_handler![...]` for LLM + telemetry IPC), `llm/` (4 files — `client.rs` with `reqwest` + SSE, `commands.rs`, `types.rs`, `mod.rs`), `telemetry/` (3 files — `commands.rs`, `writer.rs`, `mod.rs`), `capabilities/default.json` (`core:default` + `opener:default`), `build.rs` invoking `tauri_build::build()`. `Cargo.lock` is 127.9KB.

### Frontend shell (`src/`)

22 named `.tsx` files plus 53 widgets across `components/widgets/{afine,linear-algebra,shared}/`. `assessments/` carries 7 assessment widgets plus `AnswerThread`; `chatbot/` carries `AskAboutLesson`. `lib/llm/` carries the client, hooks (`useLLM`, `useLLMStream`, `useLLMJson`), prompts, types, DOM scraper, and prewarm. `lib/telemetry/` carries the client, the `events.ts` taxonomy, and `useWidgetTelemetry`. `lib/imaging/` carries distortions plus PSNR/SSIM plus render helpers for image-quality widgets.

### Content layer

9 lesson MDX files (`afine.mdx` ~50KB; `linear-algebra.mdx` foundations primer ~26KB; six further `linear-algebra-*.mdx` files: matrices, dot-product, span, matrix-operations, matrix-inverse, basis) totalling ~244KB. `glossary.mdx` carries cross-page-hyperlink targets (CLIP, ViT, LPIPS, ...). `scripts/lint-lesson-frontmatter.ts` cross-checks `widgets_used` vs imports vs JSX usage.

### Widget library

53 widgets total (afine 10, linear-algebra 41, shared 4). Shared widgets include `FunctionGrapher`, `LineChart`, `Misconception`, `WidgetExplainer`. A-FINE-specific widgets include `AFinePipeline`, `MetricComparison`, `RatioCollapseDemo`, `AdapterHeatmap`, `GeluComparison`, `EmbeddingHeatmap`, `TranslationVsBlurPlot`, `FidelityHeadCalculator`, `CalibratorComparison`. Library accretes opportunistically per lesson rather than pre-built speculatively.

### Assessment system

7 widget shapes shipped today. Includes `<MultipleChoice>` with optional `llmThread` for wrong-answer micro-conversations, `<ClickableHotspot>`, `<GoalDrivenWrapper>` (which hosts the tiered-hints LLM call), `<GoalChain>` for multi-step goal expansion (shipped 2026-05-12 as Q2/Q8/Q9 on A-FINE), `<PredictThenVerify>`, `<KnowledgeCheck>`, plus inline-thread component `<AnswerThread>`. Bigger pillars (Quiz, Interview) deferred until SQLite + Claude API land.

### LLM integration

Three IPC commands (`llm_chat_complete`, `llm_chat_stream`, `llm_chat_json`) bridged through three React hooks (`useLLM`, `useLLMStream`, `useLLMJson`). All three currently-shipped LLM features (wrong-answer thread, right-pane chatbot, tiered hints) run on `llama3.2:3b` via Ollama. JSON-schema mode used for tiered hints. Streaming via Tauri's `Channel<StreamEvent>`. The base URL is hardcoded today; a settings UI override is a flagged enhancement.

### Telemetry

Rust-side writer to a session log directory; frontend `lib/telemetry/client.ts` invokes `telemetry_write_*`. Event taxonomy covers `session_heartbeat`, `lesson_open`, `lesson_close` (with dwell time), `widget_mount`, `widget_interact`, `widget_unmount`, `llm_request`, `llm_response`, `llm_error`, `focus_change`, `idle_start`, `idle_end`, `route_change`. Failures are swallowed by design — never block reader experience.

### Build pipeline

`pnpm` workspace (single package) running Vite for the WebView and `cargo` for the Rust host, orchestrated by `tauri-cli`. Production build via `pnpm tauri build`. Lesson frontmatter lint script runnable independently. No CI configured.

## Technologies and concepts demonstrated

### Languages

- **Rust** — `src-tauri/` (10 `.rs` files): Tauri 2 host process, LLM HTTP client with streaming, telemetry writer, IPC command registration.
- **TypeScript** — 79 `.tsx` + 20 `.ts` files: React 19 frontend, hooks, state providers, MDX components, lesson registry, telemetry client, LLM hooks.
- **MDX** — 9 lesson files (~244KB) mixing markdown prose with React widget components and KaTeX math.
- **CSS** — 72 `.css` files; plain CSS chosen (no CSS-in-JS); dark-luxe palette tokens in `src/theme.css` (159 lines).

### Frameworks and libraries

- **Tauri 2** — both `tauri` (Rust) and `@tauri-apps/api` (JS) sides; `tauri-plugin-opener` / `@tauri-apps/plugin-opener` installed but currently unused.
- **React 19** + `@vitejs/plugin-react` — WebView frontend; lazy-loaded lesson components via `React.lazy`.
- **Vite** — dev server (`strictPort: true` on 1420) + production build for the WebView.
- **`@mdx-js/rollup`** + `@mdx-js/react` (`providerImportSource`) — lesson compilation; plugin runs `enforce: "pre"`.
- **`remark-math`** + **`rehype-katex`** — `$..$` / `$$..$$` math rendering at build time.
- **`reqwest` 0.12** with `rustls-tls` (chosen over `native-tls` to avoid OpenSSL build dependencies cross-platform).
- **`tokio` 1** with `rt-multi-thread`, `macros`, `sync` features — Rust async runtime, also covers planned async SQLite + Claude API workloads.
- **`futures-util` 0.3** (`StreamExt`) — chunk-by-chunk SSE iteration in `LlmClient::chat_stream`.

### Runtimes / engines / platforms

- **Ollama at `http://localhost:11434/v1/chat/completions`** — OpenAI-compatible local model runner; production model `llama3.2:3b`.
- **Tauri 2 host process** wrapping a system WebView; native window 800×600 declared in `tauri.conf.json`.
- **KaTeX** for math typesetting (chosen over MathJax for speed + dependency-free + server-side renderable).

### Tools

- **pnpm 11+** workspace; lockfile 94KB.
- **`tauri-cli`** orchestrating `beforeDevCommand: "pnpm dev"` + cargo build.
- **`tauri_build::build()`** in `build.rs` generates `gen/schemas/{capabilities,acl-manifests,*-schema}.json` from `capabilities/default.json`.
- **Lesson frontmatter lint** — `scripts/lint-lesson-frontmatter.ts` cross-checks `widgets_used` vs imports vs JSX usage.

### Domains and concepts

- **Local-first LLM integration with empirical model selection.** Tested `llama3.2:3b` (IFEval 77.4, production default), `llama3.2:1b` (dev fallback), `qwen2.5:3b` (A/B alternative, hallucinated factor-of-4 in long prompts, Chinese-token leakage), and `gemma2:2b` (removed — 5× slower per word and inverted dominance relationship on technical content per a documented architectural symbolic-hallucination issue).
- **Prompt discipline for small-model reliability.** System/user prompt split; persona priming; bounded output length ("answer in 2-3 sentences"); explicit grounding rule in system prompt ("Use ONLY the lesson context. Never invent technical details..."); temperature 0.2 / top_p 0.9; JSON-schema mode for structured output; token caps (~250 chatbot, ~150 per wrong-answer turn, ~80 per hint).
- **DOM-based context injection.** Lesson context for the chatbot is scraped from the rendered DOM at request time (cap ~8KB), rather than passing MDX source. Catches hidden tier sections; fits trivially in `llama3.2:3b`'s 128K window at current lesson sizes.
- **Tier inclusion semantics.** `<Tier level="essential|standard|complete">` renders at its declared minimum tier *and* all tiers above; single source of truth per lesson with discrete gating.
- **Lessons as living documents.** Backward-compatible widget API discipline; lessons declare `prerequisites`, `widgets_used`, `last_updated` in frontmatter; "adaptive staleness" formula `(now − file.last_updated) − (now − median(all_files.last_updated))` specced.
- **Parallel agent dispatch for content production.** 44-widget linear-algebra push on 2026-05-12 used 7 Opus background agents in worktree isolation, one per lesson; pattern captured durably and intended for reuse on eigenvalue / decompositions / applied-LA tracks.
- **Visualisation-over-prose discipline with explicit acid test.** "If sentences are naming phenomena the reader should be able to feel but no widget is anchoring those phenomena, the section is incomplete regardless of how good the prose reads."
- **Four-tier content architecture** (lesson / cross-page hyperlink / glossary / chat) gated by *what shape of artefact does this concept deserve?*

## Key technical decisions

- **D1. Tauri 2 over web, TUI, or Electron.** Rejected pure web (cannot reliably tap CPU/GPU for heavy visualisations), TUI (product is fundamentally visual + interactive), Electron (heavier binaries, slower startup, Node baggage). Tauri 2 sits in the middle: native Rust host + WebView; experience transfers directly from the Image Browser project.
- **D2. MDX as the content layer.** Rejected plain markdown (loses embedded-widget capability), raw TSX (loses prose ergonomics), custom DSL (reinventing a worse MDX).
- **D3. Three pillars as branches, not steps.** Teach / Quiz / Interview are alternative URL-addressed views over the same content + same question bank, covering three real intents ("learn", "drill", "explain under pressure"). Rejected linear flow + pillars-as-separate-products + pillars-as-modes-inside-Teach.
- **D4. Tier inclusion semantics** (not three separate MDX files, not exact-match tiers, not continuous depth slider). Tags mark the *minimum* tier at which a section becomes relevant.
- **D5. Local LLM via Ollama for high-volume features; Claude API reserved for quality-critical paths.** Phase 1 (Ollama) shipped; Phase 2 (`llama-server` bundled as Tauri sidecar at port 8080) and Phase 3 (Claude API via same `LlmClient` abstraction) deferred until self-distribution + quality-critical features are needed.
- **D6. Prompt discipline as load-bearing.** Every rule (system/user split, persona priming, bounded length, system-prompt grounding rule, temperature/top_p, JSON schema, token caps) was validated empirically against `llama3.2:3b`; model-shaped not universal.
- **D7. Semi-generic playground engine — not fully generic.** `<StepController>` + typed visualisers (`<ArrayVisualiser>`, `<TreeVisualiser>`, `<GraphVisualiser>`, `<MatrixVisualiser>`, `<NeuralNetVisualiser>`) over fully-generic runtime-type-inference + AST instrumentation. Defer the abstraction until the pattern repeats; three concrete reasons is a stronger justification than imagining the fourth.
- **D8. Authoring discipline.** Drafts from the future M3 sync-learning agent never auto-applied (emit to `lessons/_drafts/`); widgets stay manual (LLM may eventually compose existing primitives but never implement new ones); component library accretes opportunistically from real lessons.
- **D9. Visualisation-over-prose discipline with acid test and three sibling disciplines** (D9a state-aware `<WidgetExplainer>` captions, D9b 12-pattern metaphor library + mandatory two-draft rule before building any widget, D9c canonical Spivak-Strang-Feynman-Strogatz-Yudkowsky-Tao-MacKay voice blend codified via 20-sample voice audit).
- **D10. Four-tier content architecture** (lesson / cross-page hyperlink / glossary / chat) gated by per-concept artefact-shape question.
- **D11. Lessons are living documents with backward-compatible widget APIs.** Frontmatter is queryable for "lessons using widget X" or "lessons that depend on lesson Y"; adaptive staleness, not absolute staleness.
- **D12. Stack-specific picks consolidated** — KaTeX (not MathJax), Monaco (not CodeMirror, when shipped), react-flow / xyflow (when shipped), SQLite WAL via `rusqlite` or `sqlx` (when shipped), Satoshi body font (replaced Inter 2026-05-12), JetBrains Mono Nerd Font, dark-luxe palette, `reqwest` 0.12 with `rustls-tls`, `tokio` 1.
- **D13. Eight-month-style deferral on Quiz + Interview pillars** until SQLite persistence lands. Router currently only handles `#/catalog` and `#/lesson/<slug>`.
- **D14. Tessarix-as-Caner's-pedagogy-tooling, not a commercial product.** No tests, no CI, no release pipeline; verification is reading + using the lesson; README's commercial-positioning language is aspirational pacing rather than statement of audience.

## What is currently built

Honest current state, distinct from design ambition:

| Subsystem | Status |
|---|---|
| Tauri 2 + Vite + React 19 + TS shell, hash router, ErrorBoundary | Shipped |
| MDX content layer (`@mdx-js/rollup` + `remark-math` + `rehype-katex`); 9 lessons render | Shipped |
| Lesson registry + Catalog UI; 8 lessons registered | Shipped |
| Complexity tier system (`<Tier>` + inclusion semantics + localStorage + `TierControl` UI) | Shipped (was not in original M1 plan) |
| Settings panel (fontSize / contentWidth / density / reducedMotion) | Shipped |
| `LessonTOC` + `ReadingProgress` (IntersectionObserver-based) | Shipped |
| Local LLM integration (3 IPC commands + 3 hooks + JSON-schema mode + SSE streaming via Channel) | Shipped |
| Wrong-answer thread on `<MultipleChoice>` / `<ClickableHotspot>` | Shipped |
| Right-pane chatbot (`AskAboutLesson`, section-scoped LLM Q&A) | Shipped |
| Tiered LLM hints inside `<GoalDrivenWrapper>` via JSON-schema | Shipped |
| Telemetry (session heartbeat, idle detection, lesson_open/close + dwell, widget_mount/interact/unmount, llm_*) | Shipped |
| Lesson frontmatter lint script | Shipped |
| A-FINE lesson (50KB MDX, 9 sections, 10 widget types, 11+ inline assessments) | Shipped |
| Linear-algebra track | 7 of 28 lessons shipped (44 widgets) |
| Inspirations catalogue (~73 tools surveyed + 15 recurring patterns extracted) | Reference shipped |
| SQLite persistence | Not started |
| Claude API client | Not started |
| Quiz pillar routing + SR scheduler | Not started |
| Interview pillar routing + Claude grader | Not started |
| Sync-learning authoring agent (M3) | Not started |
| Code playgrounds (Monaco + StepController) | Not started (originally M1, deliberately skipped) |
| Tests | Zero across the entire codebase |
| CI | Not configured |

Repo scale at last verification: 272 tracked files, 200 source files (.tsx 79, .ts 20, .rs 10, .mdx 9, .css 72), 62 docs/context files, 0 tests. 75 commits across two days (24 on 2026-05-11, 51 on 2026-05-12).

## Current state

Active. Two-day burst created the substrate plus a 9-lesson library plus three LLM features; subsequent cadence will reveal whether the burst pattern is sustained or one-off. The most recent LifeOS verification snapshot is `9cd4f40` from 2026-05-12, last-verified 2026-05-13. In-flight per LifeOS: M2 widget-library expansion plus continued linear-algebra curriculum buildout; M3-M5 (sync-learning agent, SQLite persistence, Claude-graded interview) pending and explicitly sequenced behind SQLite.

## Gaps and known limitations

- **G1. No SQLite persistence layer.** Blocks Quiz pillar SR state, Interview transcripts, per-topic mastery, persistent chat history, AnswerThread persistence, lesson-completion state, sync-learning agent's incremental-deltas detection.
- **G2. No Claude API integration.** Blocks conversational interview, LLM-graded free-response, sync-learning authoring agent, per-feature local/cloud toggle.
- **G3 / G4. Quiz and Interview pillars do not route.** `App.tsx::parseHash()` only matches `lesson/<slug>`. No `<QuizView>` or `<InterviewView>`.
- **G5. Sync-learning authoring agent not started.** No `.claude/skills/sync-learning-app/` in either Tessarix or LifeOS.
- **G6. Code playgrounds (Monaco + StepController + typed visualisers) not started** — most significant divergence between README intent and code reality.
- **G7. Zero tests across the entire codebase; no `.github/workflows/`.** Stance is "verification is reading + using the lesson; automation comes when audience widens." Reasonable first additions flagged: frontmatter consistency CI promotion, Vitest smoke importing every lesson, `cargo test` smoke on `LlmClient` request-building.
- **G8. No web-build branch.** No `pnpm build:web` script; no static-deployment story for portfolio sharing.
- **G9. Cross-page lesson routing not wired up** — inline cross-page hyperlinks would render as inert text or external links today.
- **G10. Chatbot context is whole-lesson DOM-scrape**; section-scoped variant unimplemented.
- **G11. Telemetry → revision-signal pipeline unbuilt.** Rich event stream is captured; no aggregation surface, no staleness-detection consumer, no "lessons due for revisit" dashboard. Most under-leveraged shipped capability.
- **G12. A-FINE lesson has 8 open improvements** (`<MetricLandscape>`, `<NaturalnessVsFidelityScatter>`, "before vs after CLIP" section, failure-mode panel restructure, failed-experiment sidebars, quiz-mode rendering, telemetry aggregation, A-FINE Playwright regression).
- **G13. `enrich-lesson` skill not yet built** despite being specified at 10 audit dimensions in `context/notes/enrich-lesson-skill.md`; now unblocked by `<GoalChain>` shipping.
- **G14. 21 of 28 linear-algebra lessons missing.** Layers 4-6 (eigenstructure, decompositions, applications) entirely empty.
- **G15. M2-framed lessons outside linear algebra (Hebbian plasticity, CNN, options theta, algorithm playground) not started.**
- **G16-G29.** Decision-shaped open questions (assessment selection from question bank, cross-view state sharing, "stuck" path for `<QuestionGroup>`, completion definition, LLM-grading cost calibration, interview model choice, "I don't know" handling, cloud-API privacy, cost visibility, lesson splitting, glossary-to-lesson promotion criteria, "Where to read next" format, M3 compose-rights threshold, editorial pass workflow).
- **R1-R5. Risks worth flagging:** burst-cadence sustainability unknown; single-author single-machine validation only; lesson-renderer error opacity (generic `<ErrorBoundary>` UI); telemetry fire-and-forget silent data loss potential; DOM-scrape staleness during rapid section navigation.

## Direction (in-flight, not wishlist)

- **Linear-algebra curriculum continuing.** 7 of 28 lessons built; priority 3 is Layer-3 buildout (null-space, rank, four-fundamental-subspaces), likely needing `NullSpaceVisualiser`. Layer-4 eigenstructure is priority 4+, needing `EigenvectorExplorer` ("vectors that don't rotate under A").
- **A-FINE further improvements.** 11 items shipped, 8 still open; tracked in `context/plans/afine-further-improvements.md`.
- **M2 widget-library expansion in flight.**
- **Sequencing recommendation captured in Roadmap.md** (recommendation, not commitment): SQLite layer → Quiz pillar routing + SR → Layer-3 LA lessons → Interview pillar routing + Claude API → M3 sync-learning agent → `enrich-lesson` skill.

## Demonstrated skills

- **Tauri 2 desktop-app delivery in production-shaped form.** End-to-end build pipeline (`pnpm` + Vite + Cargo + `tauri-cli`), capabilities ACL, build-hooks coupling, Windows-specific `[lib].name` workaround for cargo#8519, port-coupling discipline with `strictPort: true` deliberate hard-fail.
- **Local-first LLM integration including streaming.** Rust `LlmClient` over `reqwest` 0.12 + `rustls-tls` + `tokio` 1 + `futures-util` SSE iteration; OpenAI-compatible chat-completions; JSON-schema-mode responses; Tauri `Channel<StreamEvent>` token streaming surfaced through React hooks (`useLLM`, `useLLMStream`, `useLLMJson`).
- **Empirical model selection with documented criteria.** Tested 4 small local models against a real workload, recorded per-model failure modes (qwen2.5:3b factor-of-4 hallucinations + Chinese-token leakage; gemma2:2b architectural symbolic-hallucination issue), picked `llama3.2:3b` with `llama3.2:1b` as dev fallback.
- **Prompt-engineering discipline for small-model reliability.** System/user split, persona priming, bounded output length, system-prompt grounding rule, temperature/top_p tuning, JSON-schema mode, token caps — all validated empirically against the chosen model rather than assumed universal.
- **React 19 + Vite + TypeScript application architecture.** Hash routing, lazy-loaded route components, ErrorBoundary, multi-provider composition (Tier + Settings + MDX), `IntersectionObserver`-driven active-section tracking, localStorage-backed state with CSS-variable application.
- **MDX content pipeline.** `@mdx-js/rollup` plugin chain ordering (`enforce: "pre"`), `remark-math` + `rehype-katex` math at build time, `providerImportSource: "@mdx-js/react"` for component overrides, two-phase lesson registry (lazy component + eager frontmatter) for tree-shakeable catalog rendering.
- **DOM-as-LLM-context-source pattern.** Lesson context for the chatbot is scraped from the rendered DOM at request time rather than passed in as source — catches dynamically-hidden tier sections without bespoke MDX-walking code.
- **Tier inclusion semantics for single-source-of-truth multi-level content.** `<Tier level="X">` renders at X and above; alternative approaches (three files per lesson, exact-match tiers, continuous slider) explicitly considered and rejected.
- **Parallel agent dispatch for content production at scale.** 44 widgets across 7 lessons in a single day via 7 Opus background agents in worktree isolation; pattern captured durably for reuse.
- **Decision archaeology discipline.** 14 named, numbered, alternatives-and-rejection-reasons load-bearing decisions captured in `Decisions.md`, plus a register of "decisions deferred (not yet decided)" so open questions don't masquerade as choices.
- **Honest current-state framing.** README versus code reality reconciliation called out explicitly; "what is not a gap (despite appearances)" register prevents false-positive technical debt accounting.
- **Telemetry as a first-class subsystem despite no consumer yet.** Fire-and-forget write path, swallowed-failure UX-protection stance, comprehensive event taxonomy (`session_heartbeat`, `lesson_open/close`, `widget_*`, `llm_*`, `focus_change`, `idle_*`, `route_change`).
- **Authoring discipline as written craft.** Four-note quartet (visualisation-over-prose, explanations-must-adapt-to-state, widget-creativity, lesson-voice) plus 20-sample voice audit codifying authorial register; "the product's whole differentiator is interactivity" as the load-bearing acid test for whether a section is complete.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Tessarix/_Overview.md | 225 | "That gives it a distinctive shape against the rest of the set: it has the strongest *cross-project* dependency surface (every other project is a content source for at least one planned lesson), but the smallest *production-readiness* demand (the substrate works when it works for Caner; widening the audience is a deliberate future choice, not a forcing function)." |
| Projects/Tessarix/Architecture.md | 270 | "- \"What is missing or broken?\" → [[Projects/Tessarix/Gaps]]" |
| Projects/Tessarix/Authoring Discipline.md | 375 | "- [[Projects/Tessarix/Gaps#G13]] - the enrich-lesson skill's specification + readiness status" |
| Projects/Tessarix/Decisions.md | 352 | "- [[Projects/Tessarix/Systems/LLM Integration]] - D5 + D6 in depth" |
| Projects/Tessarix/Gaps.md | 302 | "- [[Projects/Tessarix/Systems/Linear Algebra Track]] - G14 in depth" |
| Projects/Tessarix/Inspirations Catalogue.md | 257 | "- [[Projects/Tessarix/Roadmap]] - which planned lessons want which patterns" |
| Projects/Tessarix/Roadmap.md | 153 | "- [[Projects/Tessarix/Decisions]] - the rationale behind the deferral choices" |
| Projects/Tessarix/Systems/A-FINE Lesson.md | 211 | "- [[Projects/Tessarix/Decisions#D14]] - Tessarix-as-Caner's-pedagogy-tooling framing (A-FINE is the canonical example)" |
| Projects/Tessarix/Systems/Assessment System.md | 237 | "- [[Projects/Tessarix/Inspirations Catalogue]] - the 12-shape question taxonomy in context of the broader inspirations" |
| Projects/Tessarix/Systems/Build Pipeline.md | 193 | "- [[Projects/Tessarix/Decisions#D12]] - stack-specific picks (KaTeX, Monaco, react-flow, SQLite)" |
| Projects/Tessarix/Systems/Content Layer.md | 284 | "- [[Projects/Tessarix/Decisions#D11]] - lessons-as-living-documents" |
| Projects/Tessarix/Systems/Frontend Shell.md | 359 | "- [[Projects/Tessarix/Systems/Telemetry]] - full event taxonomy" |
| Projects/Tessarix/Systems/LLM Integration.md | 310 | "- [[Projects/Tessarix/Decisions#D6]] - prompt discipline as load-bearing" |
| Projects/Tessarix/Systems/Linear Algebra Track.md | 332 | "- [[Projects/Tessarix/Inspirations Catalogue]] - recurring patterns informing each widget's design" |
| Projects/Tessarix/Systems/Tauri Host.md | 237 | "- [[Projects/Tessarix/Decisions#D5]] - local LLM via Ollama + Phase 1/2/3 runner architecture" |
| Projects/Tessarix/Systems/Telemetry.md | 234 | "- [[Projects/Tessarix/Gaps#G11]] - the aggregation-consumer gap" |
| Projects/Tessarix/Systems/Widget Library.md | 268 | "- [[Projects/Tessarix/Inspirations Catalogue]] - the 15 recurring patterns the metaphor library descends from" |
