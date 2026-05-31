---
name: Consilium
status: dormant
source_repo: https://github.com/Capataina/Consilium
lifeos_folder: Projects/Consilium
last_synced: 2026-05-31
sources_read: 15
---

# Consilium

## One-line summary

A multi-LLM debate and knowledge-synthesis CLI/TUI in Python 3.11+ that replaces free-form rolling summaries with an 8-key structured JSON state snapshot parsed into typed dataclasses, then produces a thesis-style Markdown synthesis from heterogeneous Ollama and Google provider slots.

## What it is

Consilium runs the same question through multiple model slots over a configurable number of rounds, compresses each round into a canonical `DebateStateSnapshot` rather than free-form prose, feeds that state forward so later rounds build on it, and produces a final thesis-style synthesis written to a Markdown transcript. Two provider adapters are wired through LangChain: local Ollama and hosted Google (Gemini + hosted Gemma). The system is designed so agents never see each other's raw outputs — peer content only flows through the structured-state bottleneck the summariser emits, which is the project's core architectural bet against winner-picking and vocabulary-basin collapse. Reached Milestone 3 of a seven-milestone roadmap in four commits over 11 days (2026-03-04 to 2026-03-15) and then stopped; the design ambition includes MCP tool integration and convergence tracking, but the implemented scope ends at structured-state-driven rounds plus thesis synthesis.

## Architecture

Single Python package (`consilium/`, 21 files, ~89KB) with three entry points (`consilium` script, `python -m consilium`, legacy `main.py` shim) all routing into `consilium/cli.py` (argparse: `tui` | `debate` | `ask`, default `tui`). Layered top-down:

```
CLI / TUI    →    services/    →    debate/ (orchestrator + prompts + models)
                      │                            │
                      └────────────►   providers/  ◄────┘
                                            │
                                            ▼
                               langchain_ollama
                               langchain_google_genai
```

Key invariants from the import graph:

- `consilium/debate/` does not import from CLI, TUI, or services. It is the pure reasoning core.
- `consilium/providers/` does not import from `debate/`. Provider swap cannot break debate logic.
- TUI is a presentation layer over the same `MultiAgentDebateService` used by headless debate — zero orchestration logic in the TUI layer.

The round loop lives in `DebateOrchestrator.run` (5.2KB): for each round, iterate agents (build per-slot `ProviderSettings`, construct client, call `build_round_prompt`, invoke `client.ask`, emit `on_turn_complete`); then call the summariser with `build_summary_prompt`, try `parse_summary_response` and fall back to `build_fallback_snapshot` on `ValueError`; the resulting `DebateStateSnapshot` becomes the input rolling summary for the next round. After the loop, `MultiAgentDebateService._build_final_synthesis` calls `build_final_synthesis_prompt` on the full transcript (raw turns plus per-round rendered structured state) and `MarkdownTranscriptWriter.write` persists the artefact as `YYYYMMDD_HHMMSS_<slug>.md` under `artifacts/`.

The configuration surface is env-var driven (`.env.example` plus `config.py` defaults): `CONSILIUM_PROVIDER`, `CONSILIUM_MODEL`, `CONSILIUM_OLLAMA_HOST`, `CONSILIUM_TEMPERATURE` / `TOP_P` / `TOP_K` / `REPEAT_PENALTY`, `CONSILIUM_DEBATE_ROUNDS` (default 3), `CONSILIUM_AGENT_COUNT` (default 3), per-slot `CONSILIUM_AGENT_PROVIDERS` / `CONSILIUM_AGENT_MODELS` comma lists, separate `CONSILIUM_SUMMARIZER_PROVIDER` / `CONSILIUM_SUMMARIZER_MODEL`, and `CONSILIUM_OUTPUT_DIR`. CLI flags `--agent-provider` and `--agent-model` override per-slot positionally by index.

## Subsystems and components

### Debate Orchestrator (`consilium/debate/orchestrator.py`, 5.2KB)

Single place in the codebase that decides when a model gets called, in what order, with what prompt, and what happens when the summariser's output doesn't parse. Per round it builds each agent's prompt with `build_round_prompt(topic, agent, round_number, rolling_summary, previous_self[agent])`, calls the provider's `ask`, then runs the summariser and either parses to a `DebateStateSnapshot` or builds a fallback snapshot from raw turns. Exposes two callbacks (`on_turn_complete`, `on_summary_complete`); `MultiAgentDebateService` adds a third (`on_final_synthesis_complete`) for the thesis stage.

### Structured Debate State (`consilium/debate/models.py`, 12.6KB)

The 8-key `DebateStateSnapshot` schema is the most important design choice in the codebase after the decision to hide peer outputs. Top-level keys: `shared_context_snapshot` (str), `per_model_current_position` (tuple of `PerModelDebateState`), `agreements`, `disagreements`, `assumptions_or_scope_differences`, `open_issues`, `changes_this_round`, `concepts_to_preserve_next_round` (each a tuple of str). Nested per-model schema has six keys: `agent_name`, `current_claim`, `key_supporting_reasoning_or_mechanisms`, `key_objections_raised_against_claim`, `response_to_objections_this_round`, `changes_from_prior_round`. `parse_summary_response` raises `ValueError` on missing top-level keys, missing expected agent names, or unexpected agent names; type coercion (`_coerce_text`, `_coerce_text_list`) is deliberately tolerant for text-field shape drift (list → `"; ".join`, dict → `k: v`) but slot-coverage is strict. `_extract_json_payload` tries fenced code-block match first, then outermost-brace match. `build_fallback_snapshot` reconstructs `current_claim` from first sentence and `key_supporting_reasoning_or_mechanisms` from first non-empty paragraph of each raw turn; the four cross-model list fields collapse to empty tuples, which is why two rounds in fallback mode degrade reasoning quality to roughly-parallel-monologues. `serialise_debate_state` uses `ensure_ascii=True` deliberately — prior runs with non-ASCII content (Turkish characters, em-dashes, smart quotes) caused encoding issues when JSON was fed back into prompts.

### Prompts (`consilium/debate/prompts.py`, 12.9KB)

Three builders carrying almost all of the project's behavioural specification as literal string text. `build_round_prompt` is a twelve-section checklist (identity, isolation clause, round-1-vs-later branching, structured-state injection on round 2+, concept carry-forward, open-issue handling, depth clause, format). `build_summary_prompt` carries the heaviest anti-evaluative block in the codebase — an explicit banned-word list (`dominant`, `robust`, `precise`, `failed to`, `struggled to`, `better`, `worse`, `superior`, `inferior`, `more convincing`, etc.) plus an extraction-not-summarisation contract. `build_final_synthesis_prompt` mandates thesis-style prose with no bullets, no Model-A/B/C labels unless attribution is unavoidable, the same banned-comparative-language pattern, and consumes both raw turns and rendered structured state. No few-shot examples; no chain-of-thought cue (the structured-state schema *is* the thinking structure); no token-budget language; no agent personalities.

### Providers (`consilium/providers/`, 4 files, ~5.4KB total)

`factory.py` dispatches on `settings.provider`; `ollama.py` wraps `langchain_ollama.ChatOllama` (accepts `temperature`, `top_p`, `top_k`, `repeat_penalty`); `google.py` wraps `langchain_google_genai.ChatGoogleGenerativeAI` (accepts `temperature`, `top_p`, `top_k` — `repeat_penalty` silently dropped because the SDK does not expose it). Both expose a single `ask(prompt: str) -> str` with defensive duck-typed content extraction (handles `AIMessage.content` whether `str` or list-of-dicts multimodal shape). Errors translate to `ConsiliumProviderError`; the error string is the de-facto Ollama onboarding checklist (`"Confirm Ollama is running, the model '{model}' is pulled, and the host '{host}' is reachable."`). Adding a new provider is six lines across four files — the architectural cleanliness is real.

### Roster and Sampling (`consilium/agents/definitions.py`, 2.9KB)

Default Ollama roster is three heterogeneous slots — Model A (`llama3.2`, temp=0.3, top_p=0.85, top_k=30, repeat=1.05 — strict anchor), Model B (`qwen3.5:4b`, temp=0.7, top_p=0.92, top_k=60, repeat=1.00 — balanced), Model C (`gemma3:4b`, temp=1.0, top_p=0.98, top_k=100, repeat=0.98 — exploratory). Three layers stack with later overriding earlier: defaults → env-var per-slot lists → CLI `--agent-provider`/`--agent-model` positional flags. Sampling profile index wraps via modulo so slot D gets profile 0, etc. Per-slot host and API key are *not* supported — base `ProviderSettings.host` and `.api_key` are shared across slots; each adapter ignores what it doesn't care about, which works for mixed Ollama+Google rosters but blocks two-Ollama-host-in-one-debate setups.

### TUI (`consilium/tui/app.py`, 12.4KB, Textual)

Default entrypoint when `consilium` is invoked with no args. Three-state flow: COMPOSE (centred topic input + minimal guidance) → RUN (active roster, pipeline tracker with stage nodes for agent turns, round summaries, final thesis) → RESULT (thesis-only scrollable reading view, transcript file path). Uses Textual `@work` background worker pattern to avoid blocking the UI on provider calls; subscribes to all three service callbacks. Deliberately hides the debate itself — the value artefact is the synthesis, not the per-model arguments, so a dashboard-style live transcript view was rejected as making the debate feel like the product. Pipeline stages flip from pending to completed only on completion events — there are no per-stage start events, so a minute-long `gemma-3-27b-it` call looks frozen for a minute. Two tests cover routing (`test_cli_launches_tui_by_default`, `test_cli_launches_tui_command`); rendering and event handling are trust-the-framework.

### Transcripts (`consilium/debate/transcript.py`, 2.9KB)

Single-class `MarkdownTranscriptWriter`. Filename `YYYYMMDD_HHMMSS_<slug>.md` where `_slugify` lowercases and collapses non-alphanumeric runs to `-`. Renders `# Consilium Debate Transcript` header, per-round `### Model X (provider:model)` blocks with full raw response, per-round `### Structured Debate State (provider:model)` block with the rendered 8-key snapshot, then `## Final Synthesis` with the thesis. UTF-8 explicit; no atomic write (a crashed process leaves a partial file); no machine-readable JSON companion (the `DebateSummary.raw_response` field is captured in memory but discarded at write time). 18 transcripts totalling 567KB exist in `artifacts/` from four iterated topics — the folder is itself a small reasoning-quality benchmark dataset.

## Technologies and concepts demonstrated

### Languages

- **Python 3.11+** — sole language; 23 files, ~106KB total. Heavy use of frozen dataclasses (`DebateAgentDefinition`, `DebateTurn`, `DebateRound`, `DebateSummary`, `DebateStateSnapshot`, `PerModelDebateState`, `FinalSynthesis`, `DebateTranscript`, `ProviderSettings`, `DebateSettings`) and typed records throughout the debate core. Tolerant JSON parsing in `models.py` exercises pragmatic Python type-coercion idioms (`isinstance` dispatch over `None | str | int | float | bool | list | dict`).

### Frameworks and libraries

- **LangChain (>=1.2.12)** — core provider-agnostic abstraction. Both adapters (`ChatOllama`, `ChatGoogleGenerativeAI`) wrap LangChain clients with a uniform `ask(prompt) -> str` surface.
- **langchain-ollama (>=1.0.1)** — local model adapter passing temperature, top_p, top_k, repeat_penalty plus `base_url` to a local daemon.
- **langchain-google-genai (>=4.2.1)** — hosted Gemini and hosted Gemma adapter; accepts either `GOOGLE_API_KEY` or `GEMINI_API_KEY` (Google's branding shifted between Gemini and general Google Generative AI).
- **Textual (>=8.1.1)** — TUI framework using `@work` background worker pattern for non-blocking provider calls and event-callback-driven stage tracking.
- **argparse** — CLI surface (`tui` | `debate` | `ask`) with positional per-slot override flags.
- **unittest** — single 16.3KB smoke-test file (14 test functions) covering config load, service orchestration, CLI routing, and structured-summary parsing. Test fixture `_structured_summary_response` doubles as the canonical specification of a compliant 8-key emission.

### Runtimes / engines / platforms

- **Ollama local daemon** — `http://localhost:11434` by default; the default local roster targets `llama3.2`, `qwen3.5:4b`, `gemma3:4b` (genuinely different model families).
- **Google Generative AI hosted endpoint** — both `gemini-2.5-flash-lite` (default model) and hosted Gemma (`gemma-3-27b-it`, preferred for frontier-quality benchmarking due to less restrictive free-tier behaviour).
- **uv** — dependency management; `pyproject.toml` defines the `consilium` script entry point.

### Tools

- **`uv` / `pyproject.toml`** — packaging and dependency pinning (lower bounds only).
- **`unittest discover` (via `uv run`)** — test runner; no CI configuration, no pre-commit, no tox.
- **`.env` plus environment variables** — runtime configuration with a documented mismatch between `.env.example` (`CONSILIUM_SUMMARIZER_MODEL=llama3.2`) and code default (`gemma3:4b`).

### Domains and concepts

- **Multi-agent LLM orchestration** — structured round-based debate between heterogeneous models with isolation invariants (agents only see their own previous response plus structured state, never peer raw output).
- **Schema-driven structured output from LLMs** — 8-key top-level / 6-key per-model JSON schema, strict parse with tolerant type coercion plus a deterministic fallback path; explicit choice of JSON over line-oriented schema with `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY.md` documenting it as actively under review.
- **Anti-evaluative prompt engineering** — explicit banned-word lists in summariser and synthesis prompts plus structural anti-judging (structured state schema forces extraction over interpretation); the archived `IMPLEMENT_NOW_SUMMARY_NEUTRALITY.md` records that prompt-only neutrality was insufficient and the architectural shift to structured state was the deeper fix.
- **Model-family diversity as a reasoning-quality strategy** — explicit choice of three different families over three copies of one model with sampling variation; documented reasoning that same-family small models share vocabulary basins and failure modes.
- **Sampling-profile layering** — strict / balanced / exploratory profiles (temperature 0.3 / 0.7 / 1.0; top_p 0.85 / 0.92 / 0.98; top_k 30 / 60 / 100) per slot to add a second diversity axis on top of model-family diversity.
- **Provider-agnostic factory + thin-adapter pattern** — single-method `ask(prompt) -> str` interface, `factory.py` dispatch, domain-error translation. The cleanest surface in the code and the most reusable pattern.
- **TUI as thin presentation layer** — service-layer reuse between TUI and headless modes; opinionated UX framing the thesis (not the debate) as the product.
- **Documentation-first iteration** — 61KB of context docs against 89KB of source in a four-commit repo; `IMPLEMENT_NOW_*.md` execution playbooks (Status / Scope / Exit rule / Modules / Tasks / Invariants / Tests) as a reusable template across active and archived design passes.

## Key technical decisions

- **Replace prose summary with structured state.** Prose summaries failed in live runs — `gemma3:4b` stopped using banned evaluative words but still behaved like an adjudicator. The architectural fix (canonical `DebateStateSnapshot` rather than free-form prose) was deeper than the prompt-only fix and shifted responsibility from prompt engineering to schema enforcement.
- **Agents never see raw peer outputs.** In round N+1 no agent sees what others wrote verbatim — only its own previous response plus the structured state. Removes echo-contamination of independent reasoning. Risk: when fallback fires, peer content reaches next round agents indirectly via the fallback's first-sentence / first-paragraph extraction.
- **Slot personalities removed.** Agents are generic `Model A` / `Model B` / `Model C` — no "Analyst", no "Builder". Real diversity comes from genuinely different model families plus sampling regimes, not from prompt-engineering tricks that collapse into each other on small models.
- **Anti-evaluative prompt contract.** Both summariser and final-synthesis prompts carry explicit banned-comparative-language lists. The combined structural + lexical approach is belt-and-braces; the structural change carries most of the load.
- **Default local roster is heterogeneous.** `llama3.2` + `qwen3.5:4b` + `gemma3:4b` (three families) over the earlier three-copies-of-`llama3.2` approach. Pseudo-diversity from sampling alone cannot escape a shared vocabulary basin.
- **Default summariser is `gemma3:4b`, not `llama3.2`.** `llama3.2` produced summary-shaped output reliably but the summaries were repetitive and lossy; `gemma3:4b` emits better structured state. `.env.example` still says `llama3.2`, code default is `gemma3:4b` — a documented mismatch.
- **Markdown-only transcripts.** No JSON export. At Milestone 3 the priority was auditability; machine-readable export was deferred pending convergence tracking (Milestone 4) which would drive the requirement.
- **Structured-state emitter uses JSON not a line-oriented schema.** Chosen because JSON is universally understood by models and `json.loads` error paths are well-defined. Actively under review in `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY.md` — line-oriented might improve local-model compliance.
- **Minimal compose→run→result TUI rather than a live dashboard.** Earlier multi-panel operator console with per-model live cards rejected because the value artefact is the thesis, not the debate. Opinionated UX bet aligned with project intent.
- **Documentation-first iteration.** Context docs and `IMPLEMENT_NOW_*` execution playbooks written before and during the work, not after. Consistent personal pattern across vault projects, not a per-project decision.
- **Test-embedded canonical example.** `tests/test_smoke.py::_structured_summary_response` is a hand-written valid structured-summary string used as a fake-summariser fixture and doubles as the canonical specification of a compliant emission.

## What is currently built

The headless `consilium debate "<topic>"` and `consilium ask "<topic>"` paths work end-to-end. The default TUI launches when no args are supplied and drives the same `MultiAgentDebateService` through compose → run → result with a stage-tracker pipeline view. The Ollama and Google provider adapters work, including mixed-provider rosters (env or CLI configured); model-only swaps are effectively a one-line config change. The structured-state path (8-key JSON snapshot, strict parse with tolerant type coercion, deterministic fallback to first-sentence / first-paragraph extraction) works but with documented unreliability on small local summarisers — the parser is strict on slot-name coverage, so naming drift (e.g. emitting `"Agent A"` instead of `"Model A"`) triggers fallback often. Final thesis synthesis works and is rendered into the same Markdown transcript alongside per-round raw turns and rendered structured state. 18 transcripts (567KB) across four iterated topics live in `artifacts/`; the largest are hosted-Gemma multi-round brain-learning debates. Test coverage: 1 file, 14 functions covering config load, service orchestration, CLI routing, prompt composition; TUI rendering is untested.

The Markdown structured-state path is implemented, but the "rolling summarisation" framing in the README is misleading — what exists is a structured-state emitter producing an 8-key JSON object, not prose summarisation. MCP tool integration, convergence/divergence tracking, and Claude/OpenAI adapters are all absent from source — README claims for these are aspirational.

## Current state

Status: **dormant**. Moved from Caner's "Active" GitHub projects to "Other" as of the 2026-04 vault sync. Last commit `c592b34` on 2026-03-15. Four commits total spanning 2026-03-04 to 2026-03-15 (rapid prototype cadence, ~2.75 days between commits, message shapes suggesting large changes per commit). Reached Milestone 3 of seven, with Milestones 4-7 unimplemented. Final in-flight work was `context/IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY.md` (status `Active`, exit criteria "two-round local debates *usually* produce parseable structured state without fallback; fallback usage is visible and clearly secondary when it occurs") — none of its three tasks have completion markers.

## Gaps and known limitations

- **No MCP code anywhere in the repository** — `grep` for `mcp|MCP` in `consilium/` returns zero matches; no tool abstraction, no tool registry, no MCP server, no provider-side tool-call plumbing. README's "Shared tool access via MCP" is a Milestone 5 aspiration.
- **No convergence/divergence tracking.** `agreements` and `disagreements` are summariser qualitative judgements, not programmatic round-to-round similarity scores; no early stopping on convergence threshold.
- **No Claude or OpenAI adapters.** `pyproject.toml` depends only on `langchain-ollama` and `langchain-google-genai`. README's "Claude, GPT, Gemini, and local models" is aspirational. The recipe to add Claude is documented (~6 lines across 4 files).
- **Structured-state emitter is unreliable on small local summarisers.** Strict slot-name coverage check rejects valid JSON with naming drift. Fallback path preserves slot content but loses all four cross-model list fields (`agreements`, `disagreements`, `assumptions_or_scope_differences`, `concepts_to_preserve_next_round`) — two rounds in fallback degrade reasoning quality to roughly-parallel-monologues.
- **Fallback is ambient, not explicit.** No `was_fallback: bool` flag on `DebateSummary` or `DebateStateSnapshot`; readers infer fallback from literal placeholder strings appearing in per-model entries. Task 3 of the active IMPLEMENT_NOW doc, not completed.
- **`raw_response` captured but discarded at write time.** Original summariser output is held on `DebateSummary.raw_response` but never persisted — debugging fallback triggers requires inspecting the live run's stderr.
- **Per-model schema field `key_supporting_reasoning_or_mechanisms` biases toward knowledge-heavy questions.** "Mechanisms" framing is a poor fit for strategic or conceptual questions despite the explicit invariant against domain-specific schema.
- **`.env.example` disagrees with code defaults.** `.env.example` sets `CONSILIUM_SUMMARIZER_MODEL=llama3.2`; tests assert `gemma3:4b` is the default when no env is set.
- **TUI stage updates are completion-only.** No per-stage start events; a minute-long hosted call looks frozen.
- **No streaming.** All provider calls use `.invoke`, not `.stream`. The TUI run screen is silent for the full turn duration.
- **Host and API key shared across slots.** Cannot run two Ollama slots against two different local hosts in the same debate.
- **No per-slot CLI override for sampling.** Per-slot sampling only settable via the hardcoded `DEFAULT_SAMPLING_PROFILES` or by env-var globals.
- **Summariser inherits base sampling with no override.** Coupling: lowering debate-agent temperature also forces it on the summariser.
- **`repeat_penalty` silently dropped for Google slots.** `ChatOllama` passes it through; `ChatGoogleGenerativeAI` ignores it.
- **No atomic-write of transcripts.** Mid-write crash leaves a partial file.
- **No token-usage, runtime, or cost metadata in transcripts.** Easy to capture from LangChain responses; not implemented.
- **No bounds check on `agent_count`.** Accepts any positive integer; practically bounded by VRAM/context but not by the app.
- **No CI.** No `.github/workflows`, no pre-commit, no `tox.ini`.

## Direction (in-flight, not wishlist)

The only actively-named work item is `context/IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY.md` (status `Active`) with three tasks: (1) reduce schema ambiguity in the state-emitter prompt and decide JSON vs line-oriented schema; (2) tighten parser strictness (treat missing slot coverage as fallback rather than coerce, add explicit fallback marker); (3) make transcript and synthesis clearly distinguish clean vs fallback-derived state. Project is currently dormant; this work was not in motion when the LifeOS notes were last verified (2026-04-24).

## Demonstrated skills

- Designs and implements a multi-agent LLM debate system from scratch with non-trivial architectural choices (peer-output isolation, structured-state bottleneck) rather than gluing together a framework's defaults.
- Replaces prose-summary rolling context with a canonical 8-key JSON structured-state schema parsed into frozen typed dataclasses (`DebateStateSnapshot`, `PerModelDebateState`), with strict slot-coverage validation and tolerant type coercion for text fields.
- Implements a deterministic fallback path (`build_fallback_snapshot`) that degrades gracefully when small local summarisers fail JSON compliance, including documented honest limits on what fallback can reconstruct (loses all four cross-model list fields).
- Builds an anti-evaluative prompt contract that combines explicit banned-comparative-language lists with structural enforcement via schema choice — and identifies in writing why prompt-only neutrality was insufficient (`gemma3:4b` stopped using banned words but still adjudicated).
- Implements a provider-agnostic factory + thin-adapter pattern over LangChain (`OllamaChatClient`, `GoogleChatClient`, both `ask(prompt) -> str`) with domain-error translation, defensive duck-typed content extraction, and the architectural cleanliness that adding a new provider is six lines across four files.
- Designs a sampling-profile system (strict / balanced / exploratory) layered on top of model-family diversity (`llama3.2` + `qwen3.5:4b` + `gemma3:4b`) and articulates in writing why model-family diversity is a reasoning-quality strategy, not just optimisation.
- Builds a Textual TUI as a thin presentation layer over a shared service (`MultiAgentDebateService`) with `@work` background workers, event-callback stage tracking, and an opinionated three-state compose→run→result flow that frames the thesis as the product.
- Persists rich Markdown transcripts (per-round raw turns + rendered 8-key structured state + final thesis) with documented gaps (no machine-readable export, no `raw_response` persistence, no atomic write).
- Practises documentation-first iteration at scale — 61KB of context docs against 89KB of source in a four-commit repo, with reusable `IMPLEMENT_NOW_*` execution-playbook structure (Status / Scope / Exit / Modules / Tasks / Invariants / Tests) and archived-vs-active superseded files.
- Uses a test fixture (`tests/test_smoke.py::_structured_summary_response`) as a canonical specification artefact more precise than prose docs — hand-written valid structured-summary string used by service tests.
- Honestly audits README claims against code reality and writes a claim-by-claim ledger (`README Claims vs Reality.md`) distinguishing aspirational, partially-true, and verified-true capabilities.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Consilium/_Overview.md | 108 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
| Projects/Consilium/Architecture.md | 186 | "#project/consilium #domain/architecture" |
| Projects/Consilium/Decisions.md | 211 | "#project/consilium #domain/decisions" |
| Projects/Consilium/Gaps.md | 155 | "#project/consilium #domain/gaps" |
| Projects/Consilium/README Claims vs Reality.md | 112 | "#project/consilium #domain/gaps #domain/documentation" |
| Projects/Consilium/Roadmap.md | 140 | "#project/consilium #domain/roadmap" |
| Projects/Consilium/Suggestions.md | 108 | "#project/consilium #domain/suggestions" |
| Projects/Consilium/Systems/_Overview.md | 42 | "- [[Projects/Consilium/Roadmap]] — direction-of-travel" |
| Projects/Consilium/Systems/Debate Orchestrator.md | 171 | "#project/consilium #domain/orchestration" |
| Projects/Consilium/Systems/Prompts.md | 122 | "#project/consilium #domain/prompt-engineering" |
| Projects/Consilium/Systems/Providers.md | 162 | "#project/consilium #domain/providers #stack/langchain" |
| Projects/Consilium/Systems/Roster and Sampling.md | 129 | "#project/consilium #domain/configuration #domain/sampling" |
| Projects/Consilium/Systems/Structured Debate State.md | 157 | "#project/consilium #domain/state-model #domain/prompt-engineering" |
| Projects/Consilium/Systems/TUI.md | 93 | "#project/consilium #domain/ui #stack/textual" |
| Projects/Consilium/Systems/Transcripts.md | 165 | "#project/consilium #domain/persistence #domain/output" |
