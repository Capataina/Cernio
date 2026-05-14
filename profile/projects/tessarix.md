---
name: Tessarix
status: active
source_repo: https://github.com/Capataina/Tessarix
lifeos_folder: Projects/Tessarix
last_synced: 2026-05-13
sources_read: 18
---

# Tessarix

## One-line summary

Local-first Tauri 2 desktop application that teaches abstract technical concepts (image-quality metrics, linear algebra) through narrative MDX lessons fused with embedded interactive React widgets, LLM-integrated assessments (wrong-answer threads, JSON-schema tiered hints, streaming chatbot), three complexity tiers, and a typed telemetry pipeline — built as a personal pedagogy substrate, not a commercial product.

## What it is

Tessarix is a desktop learning substrate designed by Caner for Caner: a Tauri-2 native binary (Rust host + Vite + React 19 + TypeScript WebView) whose primary unit is a single MDX lesson rendered alongside interactive visualisations, state-aware LLM explanations, and progressively-tiered content gated by a `<Tier level="essential|standard|complete">` inclusion model. The design ambition is a three-pillar product — Teach, Quiz, Interview — over a shared question bank, where each topic gets `/<topic>`, `/<topic>/quiz`, `/<topic>/interview` routes serving the same content under different cognitive demands. Today only the Teach pillar is wired up; Quiz and Interview are SQLite-blocked. The substrate was scaffolded on 2026-05-11 and reached its current state in a 75-commit two-day burst (24 commits day 1 + 51 commits day 2 per LifeOS `_Overview.md`), during which one full A-FINE image-quality lesson plus seven linear-algebra lessons plus 53 widgets plus three LLM-integrated assessment surfaces all shipped. The bet captured in LifeOS `Decisions.md` D14 is that authoring lessons in this richer mode is worth the cost for Caner's own learning of topics he has invested in but does not fully understand (A-FINE, Hebbian plasticity, linear algebra underwriting his ML work); audience widening is a future choice, not a forcing function.

## Architecture

Two runtime processes plus a build pipeline that produces them. Everything else (SQLite, Claude API client, sync-learning agent, Quiz/Interview routing) is planned but not yet implemented — new capabilities land *inside* the two existing processes as new internal modules, not as new top-level subsystems.

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

The bridge between the two halves is no longer infrastructural-only — three LLM commands (`llm_chat_complete`, `llm_chat_stream`, `llm_chat_json`) plus a set of telemetry-write commands flow across it today. The Rust host owns Ollama HTTP traffic (reqwest with `rustls-tls`, tokio multi-thread, futures-util SSE streaming) and filesystem JSONL telemetry writing; the WebView owns hash-based routing, the lesson registry, MDX rendering, the complexity tier system, the settings system, and 53 widgets composed inside lessons.

Three load-bearing coupling points knit the halves together: (a) Vite's `server.port = 1420` with `strictPort: true` must equal `tauri.conf.json::build.devUrl = "http://localhost:1420"`; (b) the MDX plugin in `vite.config.ts` runs with `enforce: "pre"` so `.mdx` files are transformed to JSX before `@vitejs/plugin-react` picks them up (with `remark-math` + `rehype-katex` + `providerImportSource: "@mdx-js/react"`); (c) `Cargo.toml::[lib].name = "tessarix_lib"` is non-cosmetic — dropping the suffix would break Windows builds per rust-lang/cargo#8519.

Dependency direction: build-pipeline is upstream of both runtime subsystems at dev/build time only; frontend-shell and tauri-host are peers connected by Tauri IPC plus `tauri::ipc::Channel<StreamEvent>` for streaming. Watch-exclusion (`server.watch.ignored = ["**/src-tauri/**"]`) is non-optional — without it cargo's writes thrash Vite's HMR.

Repository scale at HEAD `9cd4f40` per LifeOS `_Overview.md`: 272 tracked files, 200 source files (79 .tsx, 20 .ts, 10 .rs, 9 .mdx, 72 .css), 62 docs/context files (51 under `context/` totalling 380.9KB), zero test files, ~244KB of MDX lesson content across 9 files, 53 widgets across 3 subfolders.

## Subsystems and components

### Tauri host (`src-tauri/`)

Maturity: working · Stability: volatile. Native Rust process wrapping the WebView. `main.rs` sets `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` (annotated DO NOT REMOVE) and calls `tessarix_lib::run()`; `lib.rs` builds the Tauri Builder with the `opener` plugin and registers three LLM commands plus telemetry commands. `src-tauri/src/llm/` is 4 files ~345 lines — `types.rs` (OpenAI-compatible request/response, JSON-schema response_format, StreamChunk and StreamEvent enums Token/Done/Error), `client.rs` 7.5KB (`LlmClient` defaulting to `http://localhost:11434/v1/chat/completions`, model `llama3.2:3b`, 180s timeout, three methods chat_complete / chat_stream / chat_json), `commands.rs` 2.7KB (three Tauri commands wiring `tauri::ipc::Channel<StreamEvent>` for per-token streaming), `mod.rs`. `src-tauri/src/telemetry/` is 3 files — `writer.rs` 2.9KB (JSONL append to session log dir, reset on each launch per commit `7979ad8`), `commands.rs` 1.5KB, `mod.rs`. Cargo dependencies: `reqwest 0.12` with `rustls-tls`/`json`/`stream`, `tokio 1` with `rt-multi-thread`/`macros`/`sync`, `futures-util 0.3` for `StreamExt`, `serde 1`, `serde_json 1`, `tauri 2`, `tauri-plugin-opener 2`. Capability set: `core:default` + `opener:default`; `app.security.csp: null` (acceptable for development, needs tightening before release).

### Frontend shell (`src/`)

Maturity: working · Stability: volatile. React 19 + TypeScript application running inside the WebView. `App.tsx` ~280 lines: hash-router with `Route = { kind: "catalog" } | { kind: "lesson"; slug: string }`; five-provider stack `<ErrorBoundary>` → `<SettingsProvider>` → `<TierProvider defaultTier="standard">` → `<MDXProvider>` → `<Suspense>`; lesson-lifecycle telemetry useEffects emitting `focus_change`/`idle_start`/`idle_end`/`session_heartbeat`/`route_change`/`lesson_open`/`lesson_close` events; `prewarmLLM()` and `initTelemetry()` at app start. Strict TS posture: `strict: true`, `noUnusedLocals`, `noUnusedParameters`, `target: "ES2020"`, `jsx: "react-jsx"`. The Layout component renders a topbar (logo + gradient text + lesson chip + three-pillar pill nav), a centred ~760px main column, optional `<LessonTOC>` + `<ReadingProgress>` sidebars at ≥1400px viewports. Settings exposes `fontSize`/`contentWidth`/`density`/`reducedMotion` persisted to localStorage and reflected as CSS custom properties onto the document root. Tier system uses inclusion semantics: `TIER_ORDER = ["essential", "standard", "complete"]`; `shouldRender(level) = TIER_ORDER.indexOf(tier) >= TIER_ORDER.indexOf(level)`.

### Build pipeline

Maturity: working · Stability: stable. pnpm 11 + Vite 6 (frontend) + Cargo (host) + `@tauri-apps/cli` v2 orchestrator. `pnpm tauri dev` reads `tauri.conf.json::build.beforeDevCommand = "pnpm dev"`, spawns Vite on port 1420, in parallel compiles `src-tauri/` via cargo (which invokes `build.rs::tauri_build::build()` to regenerate `src-tauri/gen/schemas/*.json` from `tauri.conf.json` + `capabilities/default.json`), then launches the binary which navigates the WebView to `http://localhost:1420`. `pnpm build` runs `tsc && vite build` (sequential `&&`) — `tsc` as a strict-mode pre-flight that fails the whole build on type errors, then Vite bundles to `dist/`. pnpm 11+ approval policy: `pnpm approve-builds` required once to authorise esbuild's postinstall. `clearScreen: false` in Vite config keeps Rust compile errors visible across HMR cycles.

### Content layer

Maturity: working · Stability: volatile. MDX lessons + KaTeX build-time math rendering + lesson registry + frontmatter discipline. `src/lessons/registry.ts` (175 lines) declares each lesson as `{ slug, domain, summary, Component: lazy(importer), frontmatter: Promise<LessonFrontmatter> }` — the lazy Component defers the JSX bundle until route activation; the eager `frontmatter` Promise resolves at module init so the catalog UI can render cards without bundle download. Both paths share the same `importer` function reference so Vite tree-shaking works. `LessonFrontmatter` carries `title`, `tag`, `tags[]`, `last_updated`, `estimated_time`, `widgets_used?`, `prerequisites?` — the last is declared but not yet enforced. `scripts/lint-lesson-frontmatter.ts` (4.5KB) cross-checks `widgets_used` against actual imports and JSX usage; flags declared-not-imported, imported-not-declared, imported-not-rendered; runs clean on `afine.mdx`. The four-tier content architecture (Decisions D10): full lesson at `src/lessons/<slug>.mdx`; cross-page hyperlink to another lesson; glossary entry in `src/glossary.mdx`; right-pane chatbot as last-resort safety net.

### Widget library

Maturity: working · Stability: volatile. 53 widget components across `src/components/widgets/{afine,linear-algebra,shared}/`: 10 A-FINE-specific widgets (`AFinePipeline`, `MetricComparison`, `RatioCollapseDemo`, `AdapterHeatmap`, `GeluComparison`, `EmbeddingHeatmap`, `CalibratorComparison`, `TranslationVsBlurPlot`, `FidelityHeadCalculator`, plus shared composition via `<GoalChain>`), 41 linear-algebra widgets (9 core widgets shipped 2026-05-11 plus 31 creative-widgets-catalogue widgets shipped 2026-05-12 via 7 parallel Opus background agents in worktree isolation, one per lesson, plus 1 substitute), 4 shared primitives (`FunctionGrapher`, `LineChart`, `Misconception`, `WidgetExplainer`). The 12-pattern metaphor library (iterated operation, projection/shadow, deformation/morph, direct manipulation, dual-state simultaneous display, particle field, composition timeline, physical metaphor, counter-example/regime explorer, constructive build-up, convergence animation, side-by-side regime comparison) anchors widget design; the two-draft rule (prose-describe two qualitatively-different widget designs before building either) is the forcing function against chart-default laziness.

### Assessment system

Maturity: working — 7 widget shapes shipped · Stability: volatile. `src/components/assessments/` contains seven assessment widgets — `<MultipleChoice>`, `<KnowledgeCheck>`, `<GoalDrivenWrapper>`, `<GoalChain>`, `<PredictThenVerify>`, `<ClickableHotspot>` — plus `<AnswerThread>` (10.9KB TSX + 3.8KB CSS), the LLM-driven multi-turn follow-up that opens below any `<MultipleChoice>` or `<ClickableHotspot>` carrying opt-in `llmThread` + `llmContext` props. The discipline (per LifeOS `Authoring Discipline.md`): MCQ is the exception not the default; lessons mix shapes; the widget IS the question whenever the cognitive task is manipulation; LLM-grounded wrong-answer threads have a fixed Turn 1 (auto-LLM explanation + "what were you thinking?") / Turn 2 (reader textarea) / Turn 3 (tailored correction) shape plus ≤2 follow-ups.

### LLM integration

Maturity: working — 3 features shipped on `llama3.2:3b` · Stability: volatile. `src/lib/llm/` 5 files ~430 lines: `client.ts` (non-React invoke wrappers), `hooks.ts` 11.6KB (`useLLM`/`useLLMStream`/`useLLMJson`), `prompts.ts` 16.2KB (shared PERSONA system prompt + per-feature builders for wrong-answer thread turns 1/3/followup, chatbot, tiered hints, plus the JSON schema for tiered hints), `dom.ts` (lesson DOM scrape for context injection, ~8KB cap including hidden tier sections), `prewarm.ts` (fires dummy call to warm Ollama), `types.ts`, `index.ts`. Three shipped features: (1) wrong-answer thread in `<AnswerThread>`; (2) right-pane streaming chatbot `<AskAboutLesson>` 7.0KB TSX + 5.9KB CSS, lesson-DOM-grounded, refuses out-of-scope; (3) tiered LLM hints in `<GoalDrivenWrapper>` — three progressive hints (subtle/named-parameter/near-answer) generated in a single JSON-schema-constrained call. Three-phase runner architecture per Decisions D5: Phase 1 Ollama-on-host shipped; Phase 2 `llama-server` from llama.cpp bundled as Tauri sidecar deferred; Phase 3 Claude API via same `LlmClient` abstraction reserved for quality-critical paths (conversational interview, LLM-graded free-response, sync-learning authoring agent).

### Telemetry

Maturity: working · Stability: volatile. `src/lib/telemetry/` 4 files: `client.ts` 8.3KB (emit() entry, batching with debounced flush, force-flush on `route_change`/`lesson_close`/`beforeunload`), `events.ts` 13.0KB (50+ event kinds as discriminated-union TypeScript types), `useWidgetTelemetry.ts` 2.1KB (React hook for `widget_mount`/`widget_interact`/`widget_unmount`), `index.ts`. Session-level events (`focus_change`, `idle_start`/`idle_end` at IDLE_THRESHOLD_MS = 30_000, `session_heartbeat` every 30s, `route_change`), lesson-lifecycle events (`lesson_open` with `slug`/`title`/`tier_initial`/`widgets_declared`/`prerequisites_declared`/`from_route`; `lesson_close` with `dwell_ms`/`active_ms`/`max_scroll_pct` plus empty arrays for `headings_visited`/`widgets_engaged` — known gap), widget events via `useWidgetTelemetry`, LLM events (`llm_request`/`llm_response`/`llm_error` opt-in per call via `opts.telemetryFeature`). Privacy stance: data stays on-device only; per-session log directory reset on each launch; no opt-out UI yet. The aggregation consumer that would feed "lessons due for revisit", widget engagement audit, section-level attention, chatbot-question-frequency-per-section does not yet exist — the data is being captured, nothing is reading it (Gaps G11).

## Technologies and concepts demonstrated

### Languages

- **Rust** — `src-tauri/` Tauri host. Single binary crate with `[lib].name = "tessarix_lib"` (Windows lib/bin collision avoidance), `[build-dependencies] tauri-build = "2"`. ~345 lines across the LLM module plus the telemetry module. Demonstrates: async networking (reqwest + tokio + rustls-tls), SSE streaming chunk iteration via `futures-util::StreamExt`, Tauri IPC command macros, `tauri::ipc::Channel<StreamEvent>` for per-token frontend updates, JSONL filesystem writer, JSON-schema-constrained API requests via Ollama's `response_format` field, Windows console suppression via `#![cfg_attr(...)]`.
- **TypeScript** — `src/` frontend. 79 .tsx files + 20 .ts files. Strict mode with `noUnusedLocals` / `noUnusedParameters`. Demonstrates: discriminated-union route types, React 19 `lazy()` + `Suspense` lesson loading, eagerly-resolved frontmatter `Promise<LessonFrontmatter>` for catalog rendering, custom hooks for LLM hooks (single-shot, streaming, JSON-schema), localStorage-persisted context providers with CSS-custom-property side effects, IntersectionObserver for TOC active-section tracking, `tauri::ipc::Channel`-backed streaming, branded telemetry event taxonomy as discriminated unions.
- **MDX** — `src/lessons/*.mdx`. 9 files totalling ~244KB (`afine.mdx` 49.3KB; seven linear-algebra lessons 22.1-32.2KB each; `glossary.mdx` 4KB). Demonstrates: YAML frontmatter as queryable metadata, React-component imports inside markdown, `<Tier level="X">` tier-tagged content blocks, KaTeX math via `$..$` and `$$..$$` syntax rendered at build time.

### Frameworks and libraries

- **Tauri 2** — Cross-process desktop application framework. Tauri Builder pattern, capability-permission model in `capabilities/default.json` (`core:default` + `opener:default`), tauri_build::build() compile-time schema regeneration, `tauri::ipc::Channel` streaming, `generate_handler!` macro for IPC.
- **React 19** — Frontend rendering. `lazy()` for code-split lessons, `Suspense` boundaries, hash-router state machine, `createRoot(...).render()` inside `<StrictMode>`.
- **Vite 6** — Dev server + bundler. `@vitejs/plugin-react`, `@mdx-js/rollup` with `enforce: "pre"`, strictPort, watch-exclusion of `src-tauri/`, `clearScreen: false`, env-driven `TAURI_DEV_HOST` switching to ws:// HMR on port 1421 for mobile.
- **MDX (`@mdx-js/rollup` + `@mdx-js/react`)** — Markdown + JSX, with `<MDXProvider components={mdxComponents}>` injecting overrides at the App level.
- **KaTeX (via `remark-math` + `rehype-katex`)** — Build-time math rendering, dark-theme overrides per `.katex` selector.
- **reqwest 0.12 (Rust)** — HTTP client with `rustls-tls` (no OpenSSL build dependency), `json`, `stream` features.
- **tokio 1 (Rust)** — Async runtime with `rt-multi-thread`, `macros`, `sync` features.
- **futures-util 0.3 (Rust)** — `StreamExt` for SSE chunk iteration.
- **`@tauri-apps/api` v2 + `@tauri-apps/plugin-opener` v2** — JS-side Tauri bindings; opener installed but unused, kept as cheap optionality.

### Runtimes / engines / platforms

- **Ollama at `http://localhost:11434`** — Local LLM runner. OpenAI-compatible chat-completions endpoint; user runs `brew services start ollama` and `ollama pull llama3.2:3b` (2.0 GB) once.
- **`llama3.2:3b`** — Production-default local model. Selected after empirical hallucination testing against `llama3.2:1b` (dev fallback), `qwen2.5:3b` (A/B alternative, IFEval 58.2, Chinese-token leakage in some Ollama configs), `gemma2:2b` (rejected — 5× slower per word AND inverted dominance relationship on technical content, architectural symbolic-hallucination per ceur-ws.org 2025 paper).

### Tools

- **pnpm 11** — Package manager. Approve-builds workflow for esbuild postinstall.
- **Cargo + tauri-cli v2** — Rust build + Tauri orchestration.
- **`scripts/lint-lesson-frontmatter.ts`** — Custom lint cross-checking `widgets_used` frontmatter against imports + JSX usage. Not yet wired to CI (no CI exists yet — see Gaps G7).
- **GitHub Actions** — Not yet configured; `.github/workflows/` is absent.

### Domains and concepts

- **Image quality assessment (full-reference)** — A-FINE metric: CLIP-backed adaptive fidelity index with naturalness evaluation; ratio fidelity head with `c1`/`c2` constants (silent failure under `1e-6` vs correct `1e-10`); five-parameter logistic calibrator; adapter blending asymmetric on `(s_nat_d, s_nat_r)`. PSNR + SSIM classical baselines with disagreement under translation. Implementation traps: fused-QKV transposed split, 0-D scalar drops, QuickGELU vs erf-based GELU ~1% activation diff.
- **Linear algebra pedagogy** — 28-lesson curriculum DAG in 6 layers (Foundations / Matrices / Vector-space structure / Eigenstructure / Decompositions / Applications), 7 lessons authored covering Layers 1-3; 44 widgets across the 7 lessons; the prerequisite graph is a DAG, not a linear sequence.
- **Local-LLM prompt discipline** — System+user prompt split, persona priming, bounded output length, explicit grounding rule in system prompt, temperature 0.2 top_p 0.9, JSON-schema mode for structured output, token caps (~250 chatbot / ~150 per wrong-answer turn / ~80 per hint). All validated empirically against the 4 candidate models. Load-bearing finding: short prompts kept all four models grounded; hallucinations emerged only under long technical depth.
- **Empirical model selection** — Direct A/B testing 4 local models on a grounded-explanation prompt (~40-50 line A-FINE fidelity head). Findings captured verbatim per-model in `Decisions.md` D5 and `LLM Integration.md`.
- **State-aware LLM explanations** — `<WidgetExplainer>` debounces state changes 800ms, streams output, aborts in-flight requests on state change, exposes inline "Ask a question" affordance pre-loaded with lesson + widget state. The discipline: hardcoded captions on interactive widgets are a regression.
- **Telemetry-as-future-feedback** — 50+ event kinds across session, lesson, widget, and LLM lifecycles. Aggregation consumer specced (lessons-due-for-revisit adaptive-staleness formula `(now − file.last_updated) − (now − median(all_files.last_updated))`, widget engagement audit, chatbot-question-frequency-per-section) but not built.
- **Two-draft rule** — Mechanical forcing function against chart-defaulted widget design. Author writes two qualitatively-different widget designs in prose before building either; picks one with one-sentence rationale. If both drafts are charts, that's the signal to think harder.
- **Tier inclusion semantics** — `<Tier level="X">` renders at X and all higher tiers (not exact-match); `essential` content shows at all three tiers; `complete` only at complete. Single source of truth, no per-tier file duplication.
- **Four-tier content architecture** — Lesson (full MDX with widgets), cross-page hyperlink, glossary entry, chat safety-net. Gated by "what shape of artefact does this concept deserve?".
- **Adaptive scrollytelling / step-throughable visualisation** — Step-by-step state machine advance (pattern §9), composition timelines (§7), goal-driven manipulation with success-check function, predict-then-verify reveal, click-the-hotspot diagnostic spatial reasoning.
- **Parallel-agent dispatch pattern** — 2026-05-12 wave shipped 31 widgets via 7 Opus background agents in worktree isolation, one per lesson, with self-contained briefs. 7 merge commits visible in commit history. Captured durably as Caner's auto-memory `project_parallel_agent_dispatch_for_wide_scope.md` (per LifeOS `Linear Algebra Track.md`).

## Key technical decisions

LifeOS `Decisions.md` enumerates 14 load-bearing choices plus a register of decisions-deferred. Each below states what was chosen, what was rejected, and the load-bearing reason.

**D1. Tauri 2 — not web, TUI, or Electron.** Rejected: pure web app (cannot reliably tap CPU/GPU for heavier visualisations, browser worker/memory/OffscreenCanvas limits would constrain the substrate), TUI/Ratatui (product is fundamentally visual + interactive; step-throughable visualisations and KaTeX math don't translate to a terminal), Electron (heavier binaries, slower startup, larger memory footprint, Node baggage). Rust host plus WebView gives CPU/GPU access in the host, MDX/TSX in the WebView, SQLite ownership in the host, real desktop-app build pipeline. Image Browser's Tauri 2 lineage transfers directly.

**D2. MDX as the content layer.** Rejected: plain markdown (loses embedded-widget capability), raw TSX (loses prose-authoring ergonomics), custom DSL (reinventing a worse MDX). MDX is well-maintained, has a Vite plugin, and the plugin chain (`enforce: "pre"`, remark-math, rehype-katex, `providerImportSource: "@mdx-js/react"`) all clicked together cleanly.

**D3. Three pillars are branches, not steps.** Rejected: linear "first teach, then quiz, then interview" flow (forces unwanted sequencing), pillars-as-separate-products (3× authoring cost; prevents shared question bank), Quiz/Interview as modes inside Teach (conflates three intents). The same question bank serves Quiz and Interview because the underlying knowledge probes are identical — only the answering surface differs (constrained MC/cloze vs free-response with rubric).

**D4. Tier inclusion semantics, not exact-match.** Rejected: three separate MDX files per lesson (3× authoring cost + drift), exact-match tiers (forces authoring of overlap content multiple times), continuous depth slider (discrete sections need discrete tags). Tagging marks the minimum tier at which a section becomes relevant; higher tiers always include lower-tier content.

**D5. Local LLM via Ollama for high-volume interactive features.** Rejected models: `gemma2:2b` (5× slower per word; inverted dominance relationship on technical content; architectural symbolic-hallucination issue documented in ceur-ws.org 2025 paper, not fixable via prompting). Selected: `llama3.2:3b` as production default (IFEval 77.4, no fabricated numbers, 31s for ~40-line response), `llama3.2:1b` as dev fallback for RAM-pressured environments, `qwen2.5:3b` as A/B alternative. Three-phase runner: Phase 1 Ollama-on-host (shipped); Phase 2 `llama-server` from llama.cpp bundled as Tauri sidecar (deferred); Phase 3 Claude API for quality-critical paths (deferred).

**D6. Prompt discipline as load-bearing levers.** Each rule was empirically validated against the 4 candidate models. System+user prompt split (not concatenated blob), persona priming, bounded output length (the single biggest hallucination prevention — short outputs rephrase context safely; long outputs force the model to fill gaps from weights), explicit grounding rule in SYSTEM prompt only, temperature 0.2/top_p 0.9, JSON-schema mode for structured output, token caps. Critical observation: when the same 4 models were tested with a SHORT prompt (2-3 sentence answer), all stayed grounded; hallucinations only emerged when forced into technical depth.

**D7. Semi-generic playground engine, not fully-generic.** `<StepController>` + a small library of typed visualisers (`<ArrayVisualiser>`, `<TreeVisualiser>`, `<GraphVisualiser>`, `<MatrixVisualiser>`, `<NeuralNetVisualiser>`) + per-algorithm step generators. Rejected: fully-generic (runtime type inference, AST instrumentation, polymorphic rendering, sandbox execution) — months of engine work for variable quality. Defer the abstraction until the pattern repeats three times.

**D8. Authoring discipline — drafts never auto-applied, widgets stay manual, library accretes opportunistically.** The sync-learning agent will emit MDX drafts to `lessons/_drafts/`, never directly to `lessons/`. The LLM authors prose and basic assessment questions, never interactive widget code (a `<FunctionGrapher eq="sin(a*x + b)" sliders={...} />` with wrong axes/domain/slider-ranges actively miseducates). Component library grows from real lessons, not pre-built speculatively.

**D9. Visualisation-over-prose discipline.** If a concept's whole point is "two things disagree" or "output depends continuously on input" or "small parameter change produces qualitative shift", the lesson MUST include a widget making that visible — prose alone is not acceptable. Triggered by the A-FINE shipping retrospective: the lesson had the principle violated in 5 specific places (PSNR-vs-SSIM, c1/c2 silent failure, adapter asymmetry, GELU comparison, CLIP embeddings) — all filled by subsequent widget shipments. Sibling disciplines: D9a state-aware `<WidgetExplainer>` (not hardcoded captions on interactive widgets), D9b widget creativity (two-draft rule + 12-pattern metaphor library), D9c canonical voice (Spivak-Strang-Feynman-Strogatz-Yudkowsky-Tao-MacKay blend with explicit register modulation; 20 voice samples drafted, 10 spoken-lecturer + 7 book-prose + 3 blends).

**D10. Four-tier content architecture.** Lesson / cross-page hyperlink / glossary / chat safety-net. Rejected: monolithic lessons (bloats path with content reader didn't come for), embedding-not-linking (circular content, conflicting tier states, TOC chaos), one file per concept name (VGG-16 doesn't need own file), LLM-generated lesson pages on demand (3B local models can't generate widget-rich content).

**D11. Lessons are living documents with backward-compatible widget APIs.** New widget capabilities added as props with defaults; never reshape call signatures. Adaptive staleness, not absolute — a lesson is stale when meaningfully older than the project's recent edit rhythm, not by fixed wall-clock duration. Concrete realisation: A-FINE shipped 2026-05-11 with 7 `<MultipleChoice>` questions; incrementally replaced with `<GoalDrivenWrapper>`, `<PredictThenVerify>`, `<ClickableHotspot>`, `<GoalChain>` shapes through 2026-05-12.

**D12. Stack-specific picks.** KaTeX (not MathJax) for math typesetting, Monaco (not CodeMirror) for code editor when shipped, react-flow/xyflow for node-graph viz when shipped, SQLite WAL via `rusqlite` or `sqlx` for persistence when shipped, Satoshi body font (replacing Inter post `765d37b` 2026-05-12), JetBrains Mono Nerd Font for code/mono, dark-luxe palette (cyan/magenta/yellow/green neon accents on near-black background; 159 lines of design tokens in `src/theme.css`), `reqwest 0.12` with `rustls-tls` to avoid OpenSSL build dependencies, `tokio 1` with `rt-multi-thread`/`macros`/`sync`, `futures-util 0.3` `StreamExt` for SSE iteration.

**D13. Eight-month deferral on Quiz + Interview pillars.** SQLite-blocked. The router currently only handles `#/catalog` and `#/lesson/<slug>`. Quiz needs SR scheduling state per card per session, persistent through-session storage, per-topic mastery state. Interview needs conversation transcript persistence + rubric storage + Claude API client. Neither fits cleanly in a per-page-load state model.

**D14. Tessarix-as-Caner's-pedagogy-tooling, not a commercial product.** No tests, no CI, no release pipeline yet. Lesson quality verified by Caner reading and using the lesson, not by automation. The README's commercial-positioning language is aspirational pacing, not a statement of audience. Once the substrate is great for Caner, widening the audience is a separate set of decisions made with the substrate already proven.

## What is currently built

What code actually exists and works today as of HEAD `9cd4f40` (2026-05-13), distinct from design ambition:

- **Tauri 2 native binary** — Compiles clean on macOS via `cargo check` (per LifeOS `_staleness-report.md` reference in A-FINE Lesson note); window declared 800×600, title "Tessarix", identifier `com.capataina.tessarix`.
- **Vite + React 19 + TypeScript frontend** — `tsc --noEmit` zero errors, `vite build` 664ms producing 367KB JS / 74KB CSS at the time of the A-FINE shipping verification. Hash-routes `#/catalog` and `#/lesson/<slug>` resolve through `findLesson(slug)` against the lesson registry's 8 entries.
- **MDX content layer** — 9 lesson MDX files (~244KB total) render through `@mdx-js/rollup` + `remark-math` + `rehype-katex`. Math renders at build time as `<span class="katex">` with dark-theme overrides. Lesson registry's two-phase loading pattern works — eager `Promise<LessonFrontmatter>` for catalog cards plus lazy `Component` for route activation.
- **Tier system** — `<Tier level="essential|standard|complete">` with localStorage persistence at `tessarix.tier`. `TierControl.tsx` UI for toggling. Inclusion semantics live across all 8 shipped lessons.
- **Settings system** — `fontSize` / `contentWidth` / `density` / `reducedMotion` with localStorage persistence, CSS custom property application onto document root, `<SettingsPanel>` UI.
- **53 interactive widgets** — Across A-FINE (10), linear-algebra (41), shared (4). Mechanics shipped include drag-to-construct (snap-to-tail), click-to-mark, auto-graded input quizzes at 3 difficulty tiers, drag-to-order cards, drag-to-connect lines, real-time matrix-recognition tetris, race-to-zero target games, spatial-prediction click-the-foot, multi-select identification, rapid-fire judgement quizzes, constraint-satisfaction puzzles, build-the-product algorithmic mini-games, commutativity bingo, min-swap puzzles, predict-one-entry, navigate-under-custom-basis, tournament survival bracket, eigenvector discovery, lattice reconstruction, two-step relay solvers, timed RREF speedrun, rapid 5-second classification, build-A⁻¹-from-cards, root-hunt singularity, and the A-FINE-specific pipeline-walkthrough, ratio-collapse, adapter-asymmetry-with-ghost-twin, calibrator-comparison-with-live-RMSE, fidelity-head 8-dim simplification with draggable bars.
- **Assessment system** — 7 widget shapes shipped (`<MultipleChoice>`, `<KnowledgeCheck>`, `<GoalDrivenWrapper>`, `<GoalChain>`, `<PredictThenVerify>`, `<ClickableHotspot>`, `<AnswerThread>`). 3 multi-step `<GoalChain>` instances live on A-FINE (Q2 MetricComparison 4-step, Q8 RatioCollapseDemo 3-step, Q9 AdapterHeatmap 4-step).
- **LLM integration end-to-end** — Rust `LlmClient` (single-shot / streaming / JSON-schema) → 3 Tauri IPC commands → React hooks (`useLLM`, `useLLMStream`, `useLLMJson`) → 3 user-visible features (wrong-answer thread; right-pane streaming chatbot; tiered LLM hints in `<GoalDrivenWrapper>`). Empirically validated on 4 candidate models.
- **Telemetry pipeline** — 50+ event kinds; session/lesson/widget/LLM lifecycle coverage; JSONL writer with session-log-dir reset on launch. The pipeline is fire-and-forget — never blocks reader experience.
- **A-FINE lesson** — 49.3KB MDX, 9 sections, 17 widgets declared in `widgets_used`, tier-tagged sections, 5 LLM-eligible assessment slots with `AFINE_LLM_CONTEXT` injection. Glossary scaffolding for CLIP, ViT, SSIM, LPIPS, DISTS, FID, GELU, QuickGELU, PyIQA, burn, ImageNet.
- **Linear-algebra track** — 7 of 28 lessons shipped (foundations, matrices, dot-product, span, matrix-operations, matrix-inverse, basis); 44 widgets across the 7.
- **Frontmatter consistency lint script** (`scripts/lint-lesson-frontmatter.ts`, 4.5KB) — runs clean on `afine.mdx`; not yet wired to CI.
- **Inspirations catalogue** — `context/references/inspirations/` ~73 surveyed tools across 3 categories (stem-core / technical-specialised / wildcards) plus `recurring-patterns.md` 28KB synthesis identifying 15 recurring interactive-learning patterns; mirrored into LifeOS as `Inspirations Catalogue.md`.

## Current state

Status: `active`. HEAD commit `9cd4f40` 2026-05-12 10:41 UTC. Last meaningful commit one day before run on 2026-05-13. Commit velocity profile per LifeOS `_Overview.md`: 24 commits 2026-05-11 (scaffold day), 51 commits 2026-05-12 (linear-algebra wave), then run-time silence. Currently in flight per LifeOS `Roadmap.md`: continuing the linear-algebra curriculum (21 of 28 lessons still pending across Layers 3-6), executing the A-FINE further-improvements backlog (8 items still open), preparing for SQLite landing as the next major host-side capability.

## Gaps and known limitations

From LifeOS `Gaps.md`, filtered to what is career-relevant and stated as fact rather than apology:

- **SQLite persistence layer not started** (highest blocking impact). No `rusqlite` or `sqlx` dependency in `Cargo.toml`. Choice between bundled-SQLite simplicity and offline-mode compile-time type rigour not yet committed; `rusqlite` is the likely pick. Blocks Quiz pillar SR scheduling, Interview pillar transcript persistence, persistent chat history, per-topic mastery state, inline AnswerThread persistence across page loads, lesson-completion state, and the sync-learning agent's incremental-deltas detection.
- **Claude API integration not started.** No `anthropic-sdk` crate. Key-storage strategy not chosen (Tauri's secure-storage plugin is the obvious pick). Blocks conversational interview (Interview pillar's primary mode), LLM-graded free-response, sync-learning authoring agent.
- **Quiz pillar not implemented.** `App.tsx::parseHash()` only matches `lesson/<slug>`. No `<QuizView>` component. No `<slug>.questions.ts` typed question-bank file pattern; question content is inline in MDX today. Blocks adaptive within-session difficulty and across-session spaced repetition.
- **Interview pillar not implemented.** No `<InterviewView>` component. No rubric-authoring conventions. No `interview_*` IPC commands. Blocks the deepest product-value moment per the design.
- **Sync-learning authoring agent (M3) not implemented.** No `.claude/skills/sync-learning-app/` folder. No `invoke("sync_learning_run")` command. Sequencing depends on SQLite + question-bank schema + Claude API.
- **Code playgrounds not implemented.** Monaco editor not yet installed. No `<StepController>`, no typed visualisers, no `playgrounds/<slug>/` folder pattern. Listed as M1 in original README framing but explicitly deferred during the 2026-05-11 burst — the most significant divergence between README intent and code reality.
- **Zero tests.** No `*.test.*` files anywhere. No `.github/workflows/`. The stance per Decisions D14: verification is reading + using the lesson; automation comes when the audience widens. Stance creates active risks: silent widget API regressions break old lessons, LLM-feature breakage on Ollama model changes is undetectable until manual use, refactors of `LlmClient`/`TierContext`/lesson-registry have no safety net.
- **Web-build branch not implemented.** README §13 notes the option but no `pnpm build:web` script or static-deployment story exists. Blocks portfolio-public sharing of lessons.
- **`widgets_engaged` and `headings_visited` emitted as empty arrays in `lesson_close`.** Per-widget engagement aggregation is not threaded back to the lesson-level event; data is recoverable from the per-widget stream but the gap is real.
- **Lesson-to-lesson routing not wired.** Inline cross-page hyperlinks from `afine.mdx` to glossary entries depend on lesson-routing being wired up; currently deferred.
- **Telemetry aggregation consumer doesn't exist.** Data is being captured (50+ event kinds, JSONL writer, session log dir); nothing reads it. The "lessons due for revisit" surface, widget engagement audit, section-level attention analysis, chatbot-question-frequency-per-section all specced but not built. This is the most under-leveraged shipped capability today.
- **A-FINE 8 open improvements.** `<MetricLandscape>` 2D disagreement heatmap (precomputed JSON needed), `<NaturalnessVsFidelityScatter>` (needs real dataset), "before vs after CLIP" section (needs re-stylised reference image), failure-mode side-panel restructure, failed-experiment sidebars, quiz-mode rendering (depends on Quiz pillar), telemetry-driven evidence aggregation (depends on aggregation consumer), screenshot regression test (Playwright setup).
- **`enrich-lesson` skill not built.** 10-dimension audit catalogue is specified in `context/notes/enrich-lesson-skill.md` but the skill is not implemented. Author currently has to remember every audit dimension manually.
- **Linear-algebra curriculum 21 of 28 lessons missing.** Layers 4-6 (eigenstructure, decompositions, applications) entirely empty.
- **Single-author, single-machine.** No second machine has run Tessarix yet. First non-Caner build will likely surface platform-specific bugs in Tauri bundle icons, Ollama discovery (only `http://localhost:11434` hardcoded; no fallback), pnpm 11+ approval friction, macOS-specific KaTeX font fallbacks.
- **No CSP.** `tauri.conf.json::app.security.csp: null`. Acceptable for development; needs tightening before release.

## Direction (in-flight, not wishlist)

From LifeOS `Roadmap.md`, items actively being worked on or with concrete near-term plan:

- **Linear-algebra Layer 3 buildout.** Priority 3 per the curriculum doc — null-space, rank, four-fundamental-subspaces. Substantial conceptual leap; needs new widgets (`NullSpaceVisualiser` likely).
- **A-FINE further-improvements backlog execution.** 11 items already shipped (3 GoalChains; composition section; TranslationVsBlurPlot; FidelityHeadCalculator; CalibratorComparison; "what A-FINE doesn't do" section; glossary scaffolding; Q10/Q11 implementation-trap mini-exercises; frontmatter lint script). 8 items still open as enumerated under Gaps above.
- **SQLite layer next.** Likely the next major host-side capability. `rusqlite` vs `sqlx` decision unresolved; `rusqlite` favoured given Image Browser's lineage and zero-ops preference.
- **`enrich-lesson` skill first build unblocked.** The skill's `<GoalChain>` dependency is now satisfied (shipped 2026-05-12 as Q2/Q8/Q9 on A-FINE), so the skill is ready to be built.

Sequencing recommendation per `Roadmap.md`: SQLite → Quiz pillar (stress-tests SQLite + question-bank schema; deterministic, no LLM dependencies) → Layer-3 linear-algebra → Interview pillar + Claude API → M3 sync-learning agent → `enrich-lesson` skill. Recommendation, not commitment.

## Demonstrated skills

What this specific project proves Caner can do, evidence-anchored to LifeOS source:

- **Ships a Tauri 2 desktop application end-to-end** with substantive IPC surface (three LLM commands plus telemetry commands) plus capability-permission configuration plus platform-specific bundle icons plus Windows-collision-avoidance lib naming. Repeats the lineage demonstrated in Image Browser.
- **Writes production Rust under a real concurrency model** — reqwest + tokio + futures-util SSE chunk iteration; `tauri::ipc::Channel<StreamEvent>` for per-token frontend streaming; JSONL filesystem writer; OpenAI-compatible chat-completion request building with JSON-schema response_format mode. Cargo dependency selection with explicit rationale (`rustls-tls` to avoid OpenSSL build dependencies).
- **Writes production TypeScript under strict mode** — `noUnusedLocals` / `noUnusedParameters` / discriminated-union route types / React 19 `lazy()` + `Suspense` / custom hooks for three LLM modes / `IntersectionObserver`-based TOC active-section tracking. ~280-line `App.tsx` with five-provider stack and rich lesson-lifecycle telemetry.
- **Authors interactive learning content at scale** — 53 widgets across 3 subfolders, 9 MDX lessons totalling ~244KB, mechanics inventory spanning 24+ distinct interaction shapes (enumerated under "What is currently built"). Validates a forcing-function authoring discipline (two-draft rule, 12-pattern metaphor library, visualisation-over-prose) and follows through on it.
- **Validates a local-LLM stack empirically before commitment** — 4 candidate models A/B tested against a grounded-explanation prompt; each model's hallucination pattern documented verbatim; the load-bearing critical observation (short prompts kept all models grounded; depth induced hallucination) extracted as the prompt discipline that makes the 3B model reliable in production.
- **Designs and ships LLM-grounded assessment surfaces** — wrong-answer thread with structured Turn-1/Turn-2/Turn-3 + ≤2 follow-ups; tiered hints via JSON-schema-constrained single call; streaming right-pane chatbot grounded in lesson DOM scrape. Composability via opt-in `llmThread` + `llmContext` props on existing widgets.
- **Dispatches parallel agents for wide-scope authoring** — 7 Opus background agents in worktree isolation shipped 31 widgets in a single day, one per lesson, each with self-contained briefs encoding the existing widget library + theme tokens + schema for new widget creation. Pattern captured durably in auto-memory as `project_parallel_agent_dispatch_for_wide_scope.md`.
- **Maintains 380.9KB of `context/` repository memory layer plus the 51-file LifeOS Tessarix folder** including subsystem docs, design-decision rationale, gap registers, plan files. Demonstrates discipline at the level of project memory, not just project code.
- **Designs telemetry pipelines for future feedback loops** — 50+ event kinds, batched JSONL writer, fire-and-forget never-block UX, per-event session/sequence anchoring. Aggregation consumer specced (adaptive-staleness formula, widget engagement audit, section-level attention, chatbot-frequency-per-section) even though not yet built.
- **Distinguishes design ambition from code reality honestly** — LifeOS `Roadmap.md` explicitly reconciles the 2026-05-11 README's milestone framing against the 2026-05-13 code's actual state, naming the divergence (M1 substrate exceeded; code playgrounds deferred; LLM features that lived in M4 territory shipped in M1). The capacity to keep these straight is the kind of project discipline most fast-moving repos lose by week 3.
- **Builds personal projects with explicit framing** — Decisions D14 names Tessarix as Caner's-pedagogy-tooling, not commercial product; rejects premature decisions about pricing/hosting/accounts/multi-user state until the substrate is good enough to widen the audience. The capacity to draw this boundary protects velocity.
