---
name: Consilium
status: dormant
source_repo: https://github.com/Capataina/Consilium
lifeos_folder: Projects/Consilium
last_synced: 2026-04-26
sources_read: 14
---

# Consilium

## One-line summary

Multi-LLM debate engine in Python that compresses each round of heterogeneous-model reasoning into an 8-key structured-state JSON snapshot rather than free-form prose, then synthesises a thesis from the round-by-round state stream.

## What it is

Consilium is a multi-LLM debate and knowledge-synthesis CLI/TUI written in Python 3.11+. It runs the same question through multiple model "slots" (default three), each with its own provider/model/sampling profile; after every round, a dedicated summariser model is asked to emit a strict JSON object describing the state of the debate, which becomes the only cross-model context the next round's agents see. After the configured number of rounds, a final synthesis call produces a thesis-style answer written to a Markdown transcript in `artifacts/`.

The project is **dormant**. LifeOS Overview states it was moved from "Active" to "Other" GitHub projects as of 2026-04 vault sync. The git history is four commits over eleven days (2026-03-04 → 2026-03-15, last commit `c592b34`), reaching Milestone 3 of a seven-milestone roadmap before stopping mid-way through the open `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` work item.

The design intent — distinct from current implementation — is that structured disagreement between heterogeneous models, compressed through shared state, produces a more useful knowledge artefact than any single model's answer. The README's framing of that intent claims Claude/GPT/Gemini/local providers and MCP-mediated shared tool access; LifeOS's `README Claims vs Reality` audit confirms only Ollama and Google adapters are implemented and there is zero MCP code in the repository. The per-project file below describes the implemented system, with the aspirational gaps named explicitly.

## Architecture

The implementation is a layered Python package under `consilium/` with a clean dependency direction: CLI/TUI → services → debate-core (orchestrator + prompts + models) → providers → LangChain. The reasoning core (`consilium/debate/`) does not import from CLI, TUI, or services; the providers package does not import from `debate/`; the TUI is a presentation layer over the same `MultiAgentDebateService` the headless `debate` command uses. These invariants are verified file-by-file in LifeOS Architecture.md.

```
Entry points
├── consilium (script)        ──► consilium.cli:main
├── python -m consilium       ──► consilium.__main__
└── python main.py (legacy)   ──► consilium.cli:main
                                       │
                                       ▼
                          consilium/cli.py
                          (argparse: tui | debate | ask, default = tui)
                          • builds ProviderSettings, DebateSettings, roster
                          • merges env / .env / CLI overrides
                                       │
                ┌──────────────────────┼──────────────────────┐
                ▼ (no args / tui)      ▼ (debate)             ▼ (ask)
        tui/app.py             services/                services/
        Textual                multi_agent_debate.py    single_agent.py
        compose → run → result          │
                                        ▼
                          debate/orchestrator.py
                          round loop + parse-or-fallback
                                        │
                          ┌─────────────┴─────────────┐
                          ▼                           ▼
                 debate/prompts.py           debate/models.py
                 (round / summary /          (8-key schema +
                  synthesis prompts)          parse + fallback +
                                              render)
                                        │
                                        ▼
                          providers/factory.py
                          ├── OllamaChatClient  (langchain_ollama)
                          └── GoogleChatClient  (langchain_google_genai)
                                        │
                                        ▼
                          debate/transcript.py
                          MarkdownTranscriptWriter → artifacts/<ts>_<slug>.md
```

Module responsibilities, sourced from LifeOS Architecture.md (file sizes verified against the package):

| Module | Size | Responsibility |
|---|---|---|
| `consilium/__main__.py` | 452B | `python -m consilium` → `cli.main` |
| `consilium/cli.py` | 13.1KB | argparse, env/CLI merging, roster build, dispatch, pretty-print turn/summary |
| `consilium/config.py` | 7.6KB | `ProviderSettings`, `DebateSettings` dataclasses; `.env` reader; typed validation |
| `consilium/errors.py` | 1.8KB | `ConsiliumError` base + `ConsiliumConfigurationError`, `ConsiliumProviderError` |
| `consilium/agents/definitions.py` | 2.9KB | `DebateAgentDefinition` dataclass, `DEFAULT_SAMPLING_PROFILES`, `DEFAULT_LOCAL_ROSTER`, `build_debate_agents` |
| `consilium/debate/models.py` | 12.6KB | Typed records, `parse_summary_response`, `build_fallback_snapshot`, `render_debate_state`, JSON extraction helpers |
| `consilium/debate/orchestrator.py` | 5.2KB | `DebateOrchestrator.run` — round loop, summariser call, parse-or-fallback |
| `consilium/debate/prompts.py` | 12.9KB | Three prompt builders: `build_round_prompt`, `build_summary_prompt`, `build_final_synthesis_prompt` |
| `consilium/debate/transcript.py` | 2.9KB | `MarkdownTranscriptWriter` — slugified filename, Markdown rendering |
| `consilium/providers/factory.py` | 1.0KB | `build_chat_client(settings)` — Ollama or Google |
| `consilium/providers/ollama.py` | 1.9KB | `OllamaChatClient` — wraps `langchain_ollama.ChatOllama` |
| `consilium/providers/google.py` | 1.9KB | `GoogleChatClient` — wraps `langchain_google_genai.ChatGoogleGenerativeAI` |
| `consilium/services/single_agent.py` | 2.7KB | `SingleAgentDebateService.run` |
| `consilium/services/multi_agent_debate.py` | 5.2KB | `MultiAgentDebateService.run` — composes orchestrator + final synthesis + transcript write; three callback hooks |
| `consilium/tui/app.py` | 12.4KB | Textual `App` — compose / run / result screens; pipeline-tracker stage events |

Almost all complexity concentrates in four files: `cli.py`, `prompts.py`, `models.py`, `tui/app.py` — CLI wiring, prompt text, the structured-state model layer, and the TUI.

## Subsystems and components

### Debate Orchestrator (`consilium/debate/orchestrator.py`)

The 5.2KB round loop is the single place in the codebase that decides when a model gets called, in what order, with what prompt, and what happens when the summariser's output does not parse. Per round it iterates the agents (build per-slot `ProviderSettings`, construct client, run `build_round_prompt`, call `client.ask`, emit `on_turn_complete`, record `previous_self_response`); then runs the summariser (build `build_summary_prompt`, `client.ask`, try `parse_summary_response`, on `ValueError` call `build_fallback_snapshot` on the round's raw turns); then emits `on_summary_complete` with the structured `DebateStateSnapshot`. The resulting `rolling_summary` becomes input to the next round.

What each agent sees per round, sourced from `Systems/Debate Orchestrator.md`:

| Input | Always present? | Source |
|---|---|---|
| Topic | Yes | `topic` parameter |
| Slot name | Yes | `DebateAgentDefinition.name` |
| Round number | Yes | Loop index |
| Agent's own previous answer | Round 2+ | `previous_self_responses[agent.name]` — local stance memory |
| Structured state from previous round | Round 2+ | `rolling_summary.snapshot` rendered via `render_debate_state` |
| Other agents' raw responses | **Never** | Deliberately excluded — the structured-state bottleneck is the only cross-model channel |

The parse-or-fallback decision point is strict on agent-name coverage: a JSON object with the right 8 keys is rejected if any expected slot name is missing or any unknown slot name appears. In live local runs, small models frequently drift the slot naming (`"Agent A"` vs `"Model A"`) which triggers the fallback path. This strictness is deliberate per the active `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` work item.

### Structured Debate State (`consilium/debate/models.py`)

The 12.6KB models module defines the 8-key top-level schema and 6-key per-model schema that round-to-round shared memory is canonicalised through. Top-level keys: `shared_context_snapshot`, `per_model_current_position`, `agreements`, `disagreements`, `assumptions_or_scope_differences`, `open_issues`, `changes_this_round`, `concepts_to_preserve_next_round`. Per-model keys: `agent_name`, `current_claim`, `key_supporting_reasoning_or_mechanisms`, `key_objections_raised_against_claim`, `response_to_objections_this_round`, `changes_from_prior_round`.

Parsing is deliberately asymmetric: text fields are tolerant (`_coerce_text` accepts `None`/str/int/float/bool/list/dict and converts everything to a stripped string; lists where strings were asked for get joined with `; `), but top-level key presence and per-model slot coverage are strict. The extraction step tries fenced ```json``` blocks first, then falls back to greedy first-`{`-to-last-`}` brace matching — the latter is a known weakness when models emit prose containing braces around the actual JSON.

`build_fallback_snapshot` builds a `DebateStateSnapshot` directly from the round's raw turns: first sentence per turn → `current_claim`, first non-empty paragraph → `key_supporting_reasoning_or_mechanisms`, three placeholder strings for the other per-model fields, and empty tuples for the four cross-model list fields (`agreements`, `disagreements`, `assumptions_or_scope_differences`, `concepts_to_preserve_next_round`). LifeOS Gaps explicitly notes: *"two rounds in fallback mode give the next round agents almost nothing to build on beyond their own prior answer."*

`render_debate_state` produces human-readable Markdown for prompts and transcripts; `serialise_debate_state` produces deterministic JSON via `asdict` + `json.dumps(indent=2, ensure_ascii=True)` — ASCII enforcement is intentional after prior runs hit encoding issues with non-ASCII model output.

### Prompts (`consilium/debate/prompts.py`)

The 12.9KB prompts module is where the behavioural specification of Consilium lives as literal string text. Three builders:

- **`build_round_prompt`** — twelve concatenated obligations: identity, isolation reminder, goal, first-round-vs-later branching (Round 1 demands core claim → mechanisms/principles/distinctions/examples/implications; Round 2+ injects own previous answer + the rendered structured state), round objective (don't re-answer; build on state; respond to challenges; if no challenge, deepen), disagreement engagement, concept carry-forward (*"do not casually drop valuable concepts once they have appeared"*), open-issue handling, topic discipline, depth clause (*"Prefer mechanisms, causal relationships, distinctions, examples, and consequences over generic statements like 'it is complex'"*), and a paragraph-not-bullets format directive.
- **`build_summary_prompt`** — anti-evaluative pre-block (the most heavily-worded part of any prompt: *"Do not rank models, compare model quality, praise models, criticise models, pick a winner, or imply that one participant is best, strongest, dominant, weakest, more correct, more robust, more precise..."*), extraction-not-summarisation contract, JSON-only output instruction, previous structured state injection, current-round raw turns, post-response schema spec.
- **`build_final_synthesis_prompt`** — anti-format (no bullets, no field labels), anti-narration (don't narrate the debate; don't mention `Model A`/`B`/`C` unless attribution unavoidable), anti-scoreboard, substance preservation, convergence-vs-disagreement framing rules, dual input (structured state as canonical compressed view, raw turns as supporting source material), per-round injection of full raw turns followed by rendered state, organisational directive (core answer first, then explanatory layers, then strongest integrated understanding + most important unresolved tension).

LifeOS notes a recognised risk: twelve-obligation prompts depend on the model attending across the full prompt, and live artefacts show small local models attending to early items and drifting on later ones. There are no few-shot examples, no chain-of-thought cues (the structured-state schema *is* the thinking structure), no length limits, and no agent-personality framing.

### Providers (`consilium/providers/`)

Four files, ~6KB total. The factory dispatches on `settings.provider`:

```python
def build_chat_client(settings: ProviderSettings) -> object:
    if settings.provider == "ollama":
        return OllamaChatClient(settings)
    if settings.provider == "google":
        return GoogleChatClient(settings)
    raise ConsiliumConfigurationError(f"Unsupported provider '{settings.provider}'.")
```

Both adapters expose exactly one method — `ask(prompt: str) -> str` — with a defensive duck-typed return-handling pattern that strips `response.content` to text whether LangChain returns an `AIMessage` or a list-of-dicts multimodal response.

`OllamaChatClient` wraps `langchain_ollama.ChatOllama` and accepts `model`, `base_url` (= `host`), `temperature`, `top_p`, `top_k`, `repeat_penalty`. `GoogleChatClient` wraps `langchain_google_genai.ChatGoogleGenerativeAI` and accepts `model`, `temperature`, `google_api_key`, `top_p`, `top_k` — `repeat_penalty` is silently ignored on Google slots because the SDK does not expose it. The Google adapter handles both Gemini (`gemini-2.5-flash-lite`) and hosted Gemma (`gemma-3-27b-it`) — LangChain's Google client abstracts the model family. Both `GOOGLE_API_KEY` and `GEMINI_API_KEY` env vars are accepted (first set wins), hedging against Google's branding shifts.

LifeOS records that adding Claude or OpenAI is a six-step addition (one `pyproject.toml` line, one new adapter file mirroring `google.py`, one factory branch, one re-export, two `config.py`/`cli.py` updates) — the architectural cleanliness is real, the omission is one of work-not-done rather than design.

### Roster and Sampling (`consilium/agents/definitions.py`)

Default local roster (3 slots), heterogeneous by family with strict→balanced→exploratory sampling:

| Slot | Provider | Model | Sampling profile |
|---|---|---|---|
| A | ollama | llama3.2 | temp=0.3, top_p=0.85, top_k=30, repeat=1.05 |
| B | ollama | qwen3.5:4b | temp=0.7, top_p=0.92, top_k=60, repeat=1.00 |
| C | ollama | gemma3:4b | temp=1.0, top_p=0.98, top_k=100, repeat=0.98 |

Three override layers stack: defaults → env-var (`CONSILIUM_AGENT_PROVIDERS=ollama,google,ollama`, `CONSILIUM_AGENT_MODELS=...`) → CLI (`--agent-provider`/`--agent-model` applied positionally by index). Sampling profiles cycle via modulo for slots beyond three. Per-slot host and API key are *not* supported — all slots inherit `host`/`api_key` from base provider settings, so two Ollama slots cannot point at two different local hosts in the same debate.

The summariser is **outside** the roster — it has separate `CONSILIUM_SUMMARIZER_PROVIDER` / `CONSILIUM_SUMMARIZER_MODEL` knobs and no sampling profile of its own (it inherits `temperature`/`top_p`/`top_k` from base settings). LifeOS flags that wanting `temperature=0` for JSON compliance forces it on the debate agents too — a coupling.

### TUI (`consilium/tui/app.py`)

A Textual app — 12.4KB — with three screens:

```
COMPOSE  ──submit topic──►  RUN  ──debate end──►  RESULT
centred                     pipeline             thesis-only
input bar +                 tracker              scrollable
guidance                    (stages)             reading view
```

Compose: centred input + minimal guidance, no model selector or history. Run: active roster summary, pipeline tracker with stage nodes per agent turn / round summary / final thesis, pending/running/completed/failed markers; no live transcript or per-token streaming. Result: thesis in scrollable reading view, transcript file path; no raw debate exchange or per-round navigation.

Implementation: Textual's `@work` background-worker pattern keeps the UI thread free of provider calls; the worker subscribes to the same three callbacks the CLI uses (`on_turn_complete`, `on_summary_complete`, `on_final_synthesis_complete`). Stage transitions are completion-only — the tracker cannot show "agent currently generating" because there is no per-stage start event, so a long Gemma call appears frozen until it returns.

The TUI is the **default** entrypoint — running `consilium` with no arguments launches it. `consilium debate` and `consilium ask` are headless alternatives. The TUI does no debate logic of its own; all orchestration runs through `MultiAgentDebateService`.

### Transcripts (`consilium/debate/transcript.py`)

A 2.9KB single-class module. Filename format: `YYYYMMDD_HHMMSS_<slug>.md`, slug = topic lowercased with non-alphanumeric runs collapsed to `-`. Output directory created with `mkdir(parents=True, exist_ok=True)`. Body structure: title, topic + UTC creation timestamp, per-round sections (each containing every raw agent turn verbatim followed by the rendered structured state with per-model bullets and the seven cross-model headed lists), final synthesis section with summariser's `provider:model` label.

What is included vs excluded: included — topic, UTC timestamp, all raw turns verbatim, per-round structured state (post-parse or fallback), summariser's `provider:model` label, final narrative synthesis. Excluded — runtime duration, token counts, cost estimates, the raw pre-parse summariser output (`DebateSummary.raw_response` is captured in memory but discarded at write time), an explicit `was_fallback` flag, per-agent sampling settings used, rounds-metadata block.

No machine-readable export, no JSON companion file, no atomic write (mid-write crash leaves a partial file), HH:MM:SS-precision filenames so two runs in the same second on the same topic would collide. The artefacts folder holds 18 transcripts at 567KB across roughly 4 topics — the project generated more transcript data than source code.

## Technologies and concepts demonstrated

### Languages

- **Python 3.11+** — every line of source. Heavy use of typed `@dataclass(frozen=True)` records (`DebateTurn`, `DebateRound`, `DebateSummary`, `DebateStateSnapshot`, `PerModelDebateState`, `FinalSynthesis`, `DebateTranscript`), `tuple[..., ...]` types for immutable state, `argparse` for the CLI surface.

### Frameworks and libraries

- **LangChain (1.x)** — provider-agnostic LLM orchestration layer. `pyproject.toml` pins `langchain>=1.2.12`, `langchain-google-genai>=4.2.1`, `langchain-ollama>=1.0.1`. Both adapters are thin wrappers around LangChain client classes (`ChatOllama`, `ChatGoogleGenerativeAI`).
- **Textual (>=8.1.1)** — the TUI framework. The app uses Textual's `App` subclass, `@work` background worker pattern for non-blocking provider calls, and screen composition for the compose/run/result flow.

### Runtimes / engines / platforms

- **Ollama (local)** — primary local-inference backend. Default models in the roster: `llama3.2`, `qwen3.5:4b`, `gemma3:4b`. Adapter passes `base_url` (default `http://localhost:11434`).
- **Google Generative AI (hosted)** — second provider. Models include `gemini-2.5-flash-lite` (default for the Google provider) and hosted Gemma `gemma-3-27b-it`. LifeOS records the latter as the *"most promising frontier-quality path"* because the lighter Gemini models hit free-tier quota limits during iterative testing.

### Tools

- **uv** — Python project/dependency manager. README invocations and LifeOS examples use `uv run python -m consilium ...` and `uv run python -m unittest discover` for tests.
- **`unittest`** — the standard-library test runner. `tests/test_smoke.py` is the single test file (16.3KB, 14 test functions covering config load, service orchestration, CLI routing, and structured-summary parsing).
- **`.env` files** — `.env.example` documents the full env-var surface. `config.py` reads `.env` plus the OS environment with typed validation.
- No CI configured (no `.github/workflows`, no `pre-commit`, no `tox.ini`).

### Domains and concepts

- **Multi-agent / multi-LLM orchestration** — round-loop architecture where N independent agents each respond to a topic per round, with a dedicated summariser model running between rounds.
- **Structured-state shared memory (anti-prose-summary)** — the central design move: replacing free-form prose round summaries with a strictly-validated JSON snapshot. Documented as a direct response to observed prose summarisers drifting into evaluative "winner-picking" language.
- **Anti-evaluative prompt contract** — explicit banned-comparative-language list (`dominant`, `robust`, `more correct`, `failed to`, `struggled to`, etc.) in both summariser and final-synthesis prompts; final synthesis told not to mention slot labels in normal cases. Documented as an instance of *verifiable obligations beat vague exhortations* — the prompt-only version was insufficient on its own and required the structural change to structured state.
- **Schema-driven structured output with parse-or-fallback** — strict JSON parsing (8 required top-level keys, strict slot-name coverage) with tolerant per-field type coercion (lists → joined strings; numbers/bools → str), and a graceful fallback path that builds a degraded snapshot from raw turns when parsing fails.
- **Heterogeneous-family model diversity** — explicit reasoning-quality strategy (not optimisation): default roster spans three model families because same-family small models share vocabulary basins and failure modes; sampling-profile diversity layers on top.
- **Provider-agnostic adapter pattern** — thin one-method (`ask(prompt) -> str`) adapters behind a factory; adding a new provider is mechanically six small edits.
- **Service-layer separation** — `MultiAgentDebateService` composes orchestrator + final synthesis + transcript write; the TUI is a thin presentation layer over the same service the headless `debate` command uses; zero orchestration logic lives in the TUI.
- **Slugified timestamped artefact filenames** — UTC `YYYYMMDD_HHMMSS_<slug>.md` for transcript output, with `_slugify` collapsing non-alphanumeric runs.
- **Documentation-first iteration with `IMPLEMENT_NOW_*` execution playbooks** — context docs are written before and during the work, not after; the pattern emits archived (superseded) and active playbook files that act as working-memory artefacts.

## Key technical decisions

Drawn from LifeOS `Decisions.md`, ordered by architectural weight.

- **Replace prose summary with structured state.** Round-to-round shared memory is a canonical 8-key JSON snapshot, not free-form prose. *Rejected*: prose with banned-evaluative-language prompt fix (the archived `IMPLEMENT_NOW_SUMMARY_NEUTRALITY` work — explicitly observed to fail in live runs because `gemma3:4b` stopped using the banned words but still adjudicated in substance); no shared memory; raw peer outputs as shared memory. *Why it won*: free prose is inherently editorial, lossy, and evaluative; a rigid schema forces extraction over interpretation. *What would change it*: a much larger local summariser (20B+) reliably producing neutral prose with all mechanisms preserved, or schema rigidity proving wrong-fit for non-knowledge questions.
- **Agents never see raw peer outputs.** In round N+1, no agent sees what other agents wrote in round N verbatim — only its own previous response and the structured state. *Rejected*: full raw peer visibility (Milestone 2 design); compressed prose summary of peers. *Why current won*: raw peer outputs contaminate independent reasoning — agents start echoing each other's wording. *Risk*: when the summariser fails (fallback path), peer content reaches next-round agents indirectly via the fallback's first-sentence/first-paragraph extraction — degraded but still better than raw contamination.
- **Slot personalities removed.** Agents are generic `Model A`/`B`/`C` — no `Analyst`, no `Builder`, no rhetorical personalities in prompts. *Rejected*: fixed personalities as default product model; configurable personalities per-slot. *Why current won*: personalities are prompt-engineering tricks that collapse into each other on small models — they create the *appearance* of role diversity without the substance. Real diversity comes from genuinely different model families and sampling regimes.
- **Anti-evaluative prompt contract.** Both summariser and final-synthesis prompts carry explicit banned-comparative-language lists. *Rejected*: trust-the-model; soft guidance ("try to be neutral"). *Why current won*: documented progression — soft guidance failed, banned-word-list-alone failed, banned-word-list combined with the structural change (structured state) is what works. An instance of *verifiable obligations beat vague exhortations*.
- **Default local roster is heterogeneous.** Default Ollama roster spans `llama3.2` + `qwen3.5:4b` + `gemma3:4b` — three different model families. *Rejected*: three copies of `llama3.2` with sampling differences (original); three copies of one model with large sampling-profile gaps. *Why current won*: same-family small models produce *pseudo-diversity* — they share vocabulary basins and failure modes; sampling alone cannot break out of the basin.
- **Default summariser is `gemma3:4b`, not `llama3.2`.** Although `.env.example` lists `llama3.2`, the built-in fallback in `config.py` and the test suite assert `gemma3:4b`. *Rejected*: `llama3.2` (initial choice — produced summary-shaped output reliably but the summaries were repetitive and lossy); `qwen3.5:4b` (stricter prompt-follower but blank-output failures observed). *Why current won*: live observation that `gemma3:4b` emits better structured state in typical cases, though it can still drift into evaluative substance under weak prompts. **The `.env.example` and code-default disagreement is a known bug** (Gaps `.env.example`-vs-code-default mismatch).
- **Markdown-only transcripts.** No JSON export, no alternative formats. *Why current won*: at Milestone 3 the priority is *auditability* — a human reading a transcript can tell whether a round used clean structured state or fallback state. Machine-readable export was deferred pending Milestone 4 (convergence tracking) which would drive the requirement.
- **Structured-state emitter uses JSON not line-oriented schema.** *Why JSON won for now*: universal model understanding; cheap `json.loads` with well-defined error paths. **This decision is actively under review** — `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` Task 1 explicitly says: *"Decide whether the state emitter should stay as JSON or move to a simpler line-oriented schema if local compliance remains poor."* This was the active implementation work when the project paused.
- **Minimal compose→run→result TUI (not a live dashboard).** *Rejected*: multi-panel operator console with per-model live cards (earlier approach, now obsolete). *Why current won*: the value artefact of Consilium is the *thesis*, not the debate itself; a dashboard frames the debate as the product whereas the minimal flow frames the thesis as the product.
- **Documentation-first iteration.** Context docs written before and during the work, not after; `context/` folder is 61KB against 89KB of source for a 4-commit repo, with `IMPLEMENT_NOW_*` execution playbooks acting as working-memory artefacts that outlive active coding sessions. Cross-project pattern observed across Caner's vault.
- **Test-embedded canonical example.** `tests/test_smoke.py::_structured_summary_response` contains a hand-written valid structured-summary string used as the response of a fake summariser client. The fixture is doing double duty as the canonical specification of what a compliant emission looks like — more precise than any docstring or prompt text.

## What is currently built

The honest implementation surface (distinct from the design ambition documented in the README), drawn from LifeOS Overview's "What is actually working" table:

**Working:**
- Single-agent `ask` command (single prompt → single model, no transcript).
- Headless `debate` command (multi-round, multi-agent, transcript written).
- Textual TUI (default entrypoint; minimal compose → run → result).
- Ollama provider adapter (`langchain_ollama`).
- Google provider adapter (`langchain_google_genai`, supporting both Gemini and hosted Gemma).
- Heterogeneous default local roster (3 slots, 3 model families).
- Per-slot sampling profiles (strict / balanced / exploratory).
- Per-slot provider/model overrides (env-var + CLI, applied positionally by index).
- Structured-state emitter (8-key JSON schema) — works but **unreliable** on small local summarisers.
- Fallback snapshot built from raw turns when parsing fails.
- Final narrative synthesis after all rounds.
- Markdown transcripts with per-round structured-state blocks.

**Not implemented (despite README claims):**
- MCP tool integration — zero MCP code in the repository; `grep MCP` returns README-only matches. Milestone 5 unchecked.
- Convergence / divergence tracking — no module scores round-to-round similarity; `agreements`/`disagreements` are populated by the summariser's qualitative judgement, not programmatic comparison. Milestone 4 unchecked.
- Claude / OpenAI provider adapters — `pyproject.toml` does not depend on `langchain-anthropic` or `langchain-openai`.
- Streaming — every provider call uses `.invoke`, not `.stream`.
- JSON / machine-readable transcript export.

**Scale markers** (from LifeOS Overview, all `[verified]` against repo scans):
- Python source: 23 files, ~106KB.
- `consilium/` package: 21 files, ~89KB.
- Largest source files: `cli.py` (13KB), `prompts.py` (12.9KB), `models.py` (12.6KB), `tui/app.py` (12.4KB) — almost all complexity concentrated in CLI wiring, prompt text, state models, and the TUI.
- Context docs: 11 markdown files, ~61KB — unusually thorough for a 4-commit repo.
- Test suite: 1 file (`tests/test_smoke.py`, 16.3KB, 14 test functions) covering config load, service orchestration, CLI routing, and structured-summary parsing.
- Artefacts: 18 transcript Markdown files, 567KB total — the project generated more transcript data than source code.
- Commits: 4 total, all by Caner, spanning 2026-03-04 to 2026-03-15. The context-docs-to-source ratio (61KB:89KB) is a striking signal that the project was driven by documented design passes more than by feature-first iteration.

## Current state

**Status: dormant.** Last commit `c592b34` on 2026-03-15. The project reached Milestone 3 (rolling summarisation, replaced architecturally with structured state) and stopped mid-way through `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` — the only `IMPLEMENT_NOW_*` file with `Status: Active` at the time the project paused. None of that file's three tasks (prompt-compliance decision JSON-vs-line-oriented, parser-strictness tightening, transcript-and-synthesis fallback visibility) carry completion markers. Per LifeOS Overview, the project was moved from Caner's "Active" to "Other" GitHub projects as of 2026-04 vault sync.

## Gaps and known limitations

Drawn from LifeOS `Gaps.md` and `README Claims vs Reality.md`, filtered to what is technically substantive.

**Critical (README-vs-reality mismatches):**
- **Missing MCP tool integration** — README features MCP prominently, `grep MCP` returns zero matches in `consilium/`. No tool abstraction, no tool registry, no MCP server, no provider-side tool-call plumbing.
- **Missing convergence and divergence tracking** — README claims programmatic alignment scoring; the *summariser* identifies agreements/disagreements qualitatively but the *system* does not track them across rounds. No early-stopping on convergence threshold.
- **Missing Claude and OpenAI adapters** — README lists "Claude, GPT, Gemini, and local models"; only Ollama and Google adapters exist.

**High (active work when project paused):**
- **Summariser reliability** — small local summarisers frequently trigger the fallback path because of strict slot-name coverage; fallback degrades reasoning quality to roughly-parallel-monologues. The unfinished `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` work targets exactly this.
- **Fallback is ambient, not explicit** — `DebateSummary.snapshot` carries no `was_fallback: bool` flag; readers infer from literal placeholder strings (`"No reliable ... was available in fallback mode."`) appearing in per-model entries.
- **`raw_response` captured but discarded at write time** — `DebateSummary.raw_response` is populated by the orchestrator on every summary call, never serialised. Useful for debugging fallback triggers; lost.
- **Schema field name is knowledge-biased** — `key_supporting_reasoning_or_mechanisms` leans neuroscience/science; the `IMPLEMENT_NOW` doc's invariant explicitly guards against domain-specific schema drift but the field name escaped the guard.

**Medium:**
- `.env.example` (`CONSILIUM_SUMMARIZER_MODEL=llama3.2`) disagrees with code defaults and tests (`gemma3:4b`). A user copying the example file gets different behaviour from a user with no `.env`.
- TUI stage updates are completion-only — no per-stage start events, so a long Gemma call appears frozen until the network call returns.
- Host and API key shared across slots — cannot run two Ollama slots against two different local hosts in the same debate.
- No per-slot CLI overrides for sampling — `--agent-temperature` etc. don't exist; per-slot sampling is only settable via the hardcoded `DEFAULT_SAMPLING_PROFILES`.
- Summariser inherits base sampling with no override — wanting `temperature=0` for JSON compliance forces it on the debate agents too.
- Filename collision risk on sub-second concurrent starts (HH:MM:SS precision + same topic).
- No cleanup or rotation of `artifacts/` — already 18 files at 567KB from 4 commits.

**Low:**
- No atomic write of transcripts (mid-write crash leaves partial file).
- No token-usage / runtime / cost metadata in transcripts.
- No bounds check on `agent_count`.
- `repeat_penalty` silently dropped for Google slots (provider asymmetry).
- No streaming.
- Hardcoded error strings in adapters rather than centralised in `errors.py`.
- TUI rendering is not tested (trust-the-framework).
- No CI configuration (`.github/workflows`, `pre-commit`, `tox.ini` all absent).

## Direction (in-flight, not wishlist)

The only genuinely in-flight work at pause was `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` — three open tasks: (1) decide whether the state-emitter prompt stays JSON or moves to a simpler line-oriented schema, reduce schema ambiguity; (2) tighten parsing without over-aggressive coercion, treat missing per-model slot coverage as fallback, add explicit fallback marker; (3) annotate fallback-derived rounds in transcript and let the final synthesis prompt distinguish clean from fallback state. Exit criteria per the doc: *"Two-round local debates usually produce parseable structured state without fallback. Fallback usage is visible and clearly secondary when it occurs."*

Per LifeOS Roadmap, if the project resumes the priority ordering is: (1) finish the IMPLEMENT_NOW work; (2) add Claude and OpenAI adapters (~1 day each, demonstrates the provider-agnostic claim); (3) reconcile README to reality (narrow MCP and convergence claims to "planned"); (4) Milestone 4 convergence tracking (requires JSON transcript export as prerequisite); (5) decide MCP versus a simpler provider-agnostic tool abstraction. Items beyond (1) are not yet in flight — they are the documented next path if the project reactivates.

## Demonstrated skills

Specific provable capabilities the LifeOS source documents this project exercises (regardless of dormant status):

- **LangChain-based provider-agnostic LLM orchestration** — factory + thin one-method (`ask`) adapter pattern across two real providers, with the architecture verified clean enough that a third adapter is genuinely a six-step addition.
- **Multi-LLM debate engine design** — independent-agent round loop, dedicated summariser model between rounds, explicit isolation invariant ("agents never see raw peer outputs").
- **Schema-driven structured output for LLM-mediated state** — 8-key JSON snapshot, strict top-level + per-slot validation, asymmetric tolerance (forgiving per-field coercion paired with strict structure), graceful fallback path that degrades reasoning quality predictably rather than crashing the loop.
- **Prompt engineering with explicit anti-evaluative contracts** — banned-comparative-language lists, extraction-not-summarisation framing, twelve-obligation round prompts, explicit anti-narration in final-synthesis prompts; combined with structural reinforcement when prompt-only fixes proved insufficient.
- **Heterogeneous-family roster design as a reasoning-quality strategy** — documented reasoning that same-family small models share vocabulary basins, with the default roster spanning three model families and three sampling regimes.
- **Textual TUI built as a thin presentation layer over a service** — zero orchestration logic in the TUI layer; `MultiAgentDebateService` is shared between TUI and headless paths; background-worker pattern keeps the UI thread free during provider calls.
- **Documentation-first iteration with `IMPLEMENT_NOW_*` execution playbooks** — a reusable schema (Status/Scope/Exit-rule, modules affected, function inventory, task breakdown with Discovery/Implementation/Verify/Invariants/Tests, integration points, completion criteria) for in-progress design work; consistent across the repo's archived and active playbooks.
- **Test fixtures as canonical specifications** — `_structured_summary_response` in `tests/test_smoke.py` is treated as the precise spec for a compliant structured-state emission, more precise than any docstring or prompt text.
- **Honest gap auditing** — the `README Claims vs Reality.md` artefact explicitly enumerates aspirational-vs-implemented claims line-by-line, with no defensiveness around the unimplemented surface.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Consilium/Overview.md | 99 | "#project/consilium #domain/llm-orchestration #domain/agents #stack/python #stack/langchain #stack/textual #status/dormant" |
| Projects/Consilium/Architecture.md | 186 | "#project/consilium #domain/architecture" |
| Projects/Consilium/Decisions.md | 211 | "#project/consilium #domain/decisions" |
| Projects/Consilium/Gaps.md | 155 | "#project/consilium #domain/gaps" |
| Projects/Consilium/README Claims vs Reality.md | 112 | "#project/consilium #domain/gaps #domain/documentation" |
| Projects/Consilium/Roadmap.md | 140 | "#project/consilium #domain/roadmap" |
| Projects/Consilium/Suggestions.md | 108 | "#project/consilium #domain/suggestions" |
| Projects/Consilium/Systems/Debate Orchestrator.md | 171 | "#project/consilium #domain/orchestration" |
| Projects/Consilium/Systems/Prompts.md | 122 | "#project/consilium #domain/prompt-engineering" |
| Projects/Consilium/Systems/Providers.md | 162 | "#project/consilium #domain/providers #stack/langchain" |
| Projects/Consilium/Systems/Roster and Sampling.md | 129 | "#project/consilium #domain/configuration #domain/sampling" |
| Projects/Consilium/Systems/Structured Debate State.md | 157 | "#project/consilium #domain/state-model #domain/prompt-engineering" |
| Projects/Consilium/Systems/TUI.md | 93 | "#project/consilium #domain/ui #stack/textual" |
| Projects/Consilium/Systems/Transcripts.md | 165 | "#project/consilium #domain/persistence #domain/output" |
