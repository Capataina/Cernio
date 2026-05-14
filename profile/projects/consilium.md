---
name: Consilium
status: dormant
source_repo: https://github.com/Capataina/Consilium
lifeos_folder: Projects/Consilium
last_synced: 2026-05-13
sources_read: 15
---

# Consilium

## One-line summary

A Python 3.11+ CLI/TUI that runs the same question through a heterogeneous roster of LLMs across multiple debate rounds, compressing each round's output into a strict 8-key JSON state snapshot that is fed forward in place of raw peer text, with a final thesis-style synthesis written to a Markdown transcript.

## What it is

Consilium is a multi-LLM debate and knowledge-synthesis tool whose stated premise is that structured disagreement between heterogeneous models, compressed through a canonical shared-state object rather than free-form prose, produces a more useful knowledge artefact than any single model's answer. It is implemented as a Python package (`consilium/`) with three entry points — the default Textual TUI, a headless `debate` subcommand for scripting, and a single-agent `ask` subcommand — all sharing the same orchestration core. LifeOS records the project as having reached Milestone 3 of a seven-milestone README roadmap (Single-Agent CLI, Multi-Agent Debate Engine, Rolling Summarisation) and then stopping; the last commit is `c592b34` on 2026-03-15, with four commits total spanning 11 days from 2026-03-04, and the GitHub status was moved from "Active" to "Other" in the 2026-04 vault sync. The project is best read as a Milestone-3-complete prototype with an active but unfinished `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` work item documenting the open structured-state work that was queued when iteration paused. What it *demonstrates* is therefore narrower than what the README *describes*: README claims around Claude/GPT provider parity, MCP shared tool access, and programmatic convergence tracking are explicitly flagged in LifeOS as aspirational with zero source evidence.

## Architecture

The codebase has a strict top-down dependency direction enforced by LifeOS Architecture.md as an invariant: CLI/TUI → services → debate core (orchestrator + prompts + models) → providers, with the providers layer wrapping LangChain. The `consilium/debate/` package never imports from CLI, TUI, or services, and `consilium/providers/` never imports from `consilium/debate/`, so the reasoning core and provider adapters are independently swappable.

```
┌─────────────────────────────────────────────────────────────────────┐
│  Entry points                                                        │
│  ├── consilium (script from pyproject)  ──► consilium.cli:main       │
│  ├── python -m consilium                ──► consilium.__main__       │
│  └── python main.py (legacy shim)       ──► consilium.cli:main       │
└────────────────────────────┬─────────────────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────────────────┐
│  consilium/cli.py    (argparse: tui | debate | ask, default = tui)    │
│  • builds ProviderSettings, DebateSettings, roster                    │
│  • merges env / .env / CLI overrides                                  │
│  • dispatches to TUI or service                                       │
└──────┬────────────────────────┬────────────────────────┬──────────────┘
       │ (no args / tui)        │ (debate)               │ (ask)
       ▼                        ▼                        ▼
┌─────────────┐      ┌─────────────────────┐      ┌───────────────────┐
│ tui/app.py  │      │ services/           │      │ services/         │
│ Textual     │──────▶ multi_agent_debate  │      │ single_agent      │
│ compose →   │      │     .py             │      │     .py           │
│ run → result│      └──────────┬──────────┘      └────────┬──────────┘
└─────────────┘                 │                          │
                                ▼                          │
                     ┌──────────────────────┐              │
                     │ debate/orchestrator  │              │
                     │ round loop +         │              │
                     │ parse-or-fallback    │              │
                     └──┬───────────────┬───┘              │
                        ▼               ▼                  │
              ┌──────────────┐  ┌───────────────┐          │
              │ debate/      │  │ debate/       │          │
              │ prompts.py   │  │ models.py     │          │
              │ (3 builders) │  │ (snapshot +   │          │
              │              │  │  parse +      │          │
              │              │  │  fallback +   │          │
              │              │  │  render)      │          │
              └──────────────┘  └───────────────┘          │
                        │                                  │
                        ▼                                  ▼
              ┌──────────────────────────────────────────────────┐
              │ providers/factory.py → OllamaChatClient          │
              │                      → GoogleChatClient          │
              │ (thin wrappers around LangChain invoke)          │
              └──────────────────────────────────────────────────┘
                        │
                        ▼
              ┌──────────────────────────────┐
              │ debate/transcript.py         │
              │ MarkdownTranscriptWriter     │
              │ → artifacts/<ts>_<slug>.md   │
              └──────────────────────────────┘
```

The TUI is explicitly described in LifeOS as "a presentation layer over the same service used by headless debate — zero orchestration logic in the TUI layer", which is verified by `tui/app.py` importing `MultiAgentDebateService` rather than reimplementing any round logic.

Per-round, the `DebateOrchestrator.run` loop does this:

1. For each agent in roster order, build per-slot `ProviderSettings` (agent's sampling profile overrides base settings), construct a fresh client via the factory, build the round prompt (`build_round_prompt`), call `client.ask(prompt)`, record the response as the agent's `previous_self_response`, and fire `on_turn_complete`.
2. Once all agents have spoken, build the summary prompt (`build_summary_prompt`) and call the summariser client.
3. Attempt `parse_summary_response(raw_text, expected_agents)` — on `ValueError`, fall through to `build_fallback_snapshot(debate_round, summary_text)`.
4. Wrap the resulting snapshot as a `DebateSummary` (with `raw_response` captured but never persisted), fire `on_summary_complete`, and let the snapshot become the input to the next round.

After the loop, `MultiAgentDebateService._build_final_synthesis` calls `build_final_synthesis_prompt(topic, transcript)` on a freshly-constructed summariser client and wraps the result as `FinalSynthesis`. The transcript writer then renders the whole thing to a Markdown file under `artifacts/`.

Key cross-cutting properties captured in LifeOS:

- **Agents never see raw peer outputs.** In round N+1, each agent sees only its own previous response and the structured state. Peer content flows exclusively through the summariser's structured emission, by design.
- **One model does two jobs.** The same `CONSILIUM_SUMMARIZER_PROVIDER`/`CONSILIUM_SUMMARIZER_MODEL` pair is used both for the per-round structured JSON emission and for the final flowing thesis — LifeOS flags these as rewarding different model behaviours (rigid schema compliance vs fluid prose) and the IMPLEMENT_NOW doc raises splitting them as an open decision.
- **Strict parsing, tolerant coercion.** Top-level key presence and exact slot-name coverage are hard checks; within fields, the parser coerces lists/dicts/numbers/bools to strings rather than rejecting.
- **TUI stage updates fire on completion only.** The pipeline tracker flips stages from pending to completed when a provider call returns; there is no per-stage start event, so the UI looks frozen during long generations. LifeOS records this as a known imperfection.

## Subsystems and components

### Debate Orchestrator (`consilium/debate/orchestrator.py`, 5.2KB)

The single point in the codebase that decides when a model is called, in what order, with what prompt, and what happens when parsing fails. Per round it sequences agent turns, calls the summariser, and runs the `try parse_summary_response / except ValueError → build_fallback_snapshot` decision. Exposes `on_turn_complete` and `on_summary_complete` callbacks; the service layer adds `on_final_synthesis_complete`. The orchestrator does not produce the final synthesis itself — that is composed by `MultiAgentDebateService` after the loop completes.

### Structured Debate State (`consilium/debate/models.py`, 12.6KB)

The core architectural artefact replacing the original free-form prose summary. `DebateStateSnapshot` is a frozen dataclass with 8 top-level keys: `shared_context_snapshot` (str), `per_model_current_position` (tuple of `PerModelDebateState`), `agreements`, `disagreements`, `assumptions_or_scope_differences`, `open_issues`, `changes_this_round`, `concepts_to_preserve_next_round` (all `tuple[str, ...]`). The nested per-model schema has 6 keys: `agent_name`, `current_claim`, `key_supporting_reasoning_or_mechanisms`, `key_objections_raised_against_claim`, `response_to_objections_this_round`, `changes_from_prior_round`.

`parse_summary_response` extracts a JSON payload (fenced code block first, then outermost-brace match), validates the dict has all 8 required keys, requires the per-model list to contain exactly the expected slot names (no missing, no extras), sorts entries by slot index, then runs forgiving text/list coercion (`_coerce_text`, `_coerce_text_list`) on field contents. `build_fallback_snapshot` extracts first-sentence/first-paragraph content from raw turns when parsing fails, populating per-model fields but leaving cross-model fields (agreements, disagreements, assumptions, concepts-to-preserve) empty. `serialise_debate_state` produces deterministic JSON with `ensure_ascii=True` because prior runs with non-ASCII model output caused encoding issues when fed back into prompts.

### Prompts (`consilium/debate/prompts.py`, 12.9KB)

LifeOS Prompts.md states the project's behavioural specification lives in this file as literal string text, not in the orchestrator or schema. Three builders:

- `build_round_prompt` — 12 numbered obligations including isolation reminder ("no access to other models' raw responses"), round-1-vs-later-round branching, structured-state injection, mandatory open-issue handling, concept carry-forward ("Do not casually drop valuable concepts once they have appeared"), and a depth clause preferring mechanisms over generic statements.
- `build_summary_prompt` — carries the project's most heavily-worded prompt content: an anti-evaluative block explicitly banning comparative competence language (`dominant`, `robust`, `precise`, `failed to`, `struggled to`, `better`, `worse`, `superior`, `inferior`, etc.), an extraction-not-summarisation contract, a JSON-only output instruction with no Markdown wrapping, and the full 8-key + 6-key schema spec.
- `build_final_synthesis_prompt` — thesis-only output (no bullets, no field labels), anti-narration ("don't narrate the debate as a sequence of speakers"), generic-topic clause to prevent neuroscience-specific phrasing, takes both raw turns and rendered structured state per round as dual input.

LifeOS records the prompts as "purely declarative — no few-shot examples, no chain-of-thought cue, no length limits". The structured-state schema is described as "the thinking structure" that replaces an explicit step-by-step cue.

### Providers (`consilium/providers/`, 4 files, ~5KB total)

Factory + thin-adapter pattern. `factory.py` (1.0KB) dispatches on `settings.provider` to `OllamaChatClient` or `GoogleChatClient`. Both adapters expose exactly one method, `ask(prompt: str) -> str`, and both use the same defensive duck-typed return handling for LangChain responses (`getattr(response, "content", "")`, then string-or-stringify, then strip).

- `OllamaChatClient` (1.9KB) wraps `langchain_ollama.ChatOllama` with `model`, `base_url` (from `ProviderSettings.host`, default `http://localhost:11434`), and optional `temperature`/`top_p`/`top_k`/`repeat_penalty`.
- `GoogleChatClient` (1.9KB) wraps `langchain_google_genai.ChatGoogleGenerativeAI` with `model`, `temperature`, `google_api_key`, and optional `top_p`/`top_k`. `repeat_penalty` is silently dropped because Google's SDK does not expose it — a documented asymmetry.

LifeOS Providers.md records the full recipe for adding a Claude or OpenAI adapter as a six-file change (`pyproject.toml` dependency, new adapter file, factory branch, `__init__` re-export, `SUPPORTED_PROVIDERS` entry in `config.py`, CLI's `_default_model_for_provider`). The hosted-Google adapter handles both Gemini and hosted Gemma model families through the same client, with `gemma-3-27b-it` recorded as the preferred hosted benchmark due to free-tier quota behaviour on the lighter Gemini models.

### Roster and Sampling (`consilium/agents/definitions.py`, 2.9KB)

A "roster" is an ordered list of `DebateAgentDefinition` instances, each carrying a slot name (`Model A`, `Model B`, …), provider, model, and sampling profile. The default local roster is heterogeneous: slot A is `llama3.2` at temp=0.3/top_p=0.85/top_k=30/repeat=1.05 (strict anchor); slot B is `qwen3.5:4b` at temp=0.7/top_p=0.92/top_k=60/repeat=1.00 (balanced); slot C is `gemma3:4b` at temp=1.0/top_p=0.98/top_k=100/repeat=0.98 (exploratory).

Overrides stack in three layers: defaults → env-var overrides (`CONSILIUM_AGENT_PROVIDERS`, `CONSILIUM_AGENT_MODELS` as comma lists, applied by slot index) → CLI overrides (`--agent-provider`, `--agent-model` flags, applied positionally). Sampling profile index wraps via modulo for rosters beyond three slots; there is no fourth profile. Per-slot `ProviderSettings` are built fresh each round, with agent sampling fields overriding base settings via `None`-check fallthrough. Host and API key are *shared* across slots — no per-slot host configuration, so two Ollama slots cannot target two different local daemons in the same debate.

LifeOS Roster and Sampling.md is explicit that model-family diversity is "a reasoning-quality claim, not just optimisation" because same-family small models share vocabulary basins and failure modes; sampling alone cannot break out of the basin.

### TUI (`consilium/tui/app.py`, 12.4KB)

A Textual `App` running a three-state flow: COMPOSE (centred topic input + minimal guidance), RUN (active roster in compact form + pipeline tracker with stage nodes for agent turns, round summaries, and final thesis), RESULT (final thesis in scrollable reading view + transcript file path). An earlier multi-panel live-transcript version with per-model cards was explicitly discarded in favour of this minimal compose→run→result flow because "the value artefact of the project is the final thesis, not the intermediate model chatter". The TUI uses Textual's `@work` background worker pattern and subscribes to the three `MultiAgentDebateService` callbacks. Stage transitions are completion-only — pending flips straight to completed when a provider call returns, so the UI looks frozen for the duration of long generations.

### Transcripts (`consilium/debate/transcript.py`, 2.9KB)

`MarkdownTranscriptWriter` produces one Markdown file per debate run with filename pattern `YYYYMMDD_HHMMSS_<slug>.md` where slug is the topic lowercased with non-alphanumeric runs collapsed to `-`. Body sections: topic, ISO timestamp, per-round agent turns verbatim plus the rendered structured state block (8-key headed format with empty lists shown as `* None`), then a `Final Synthesis` section with the summariser's `provider:model` label.

Captured but excluded from the written file: token counts, runtime, per-agent sampling settings, an explicit `was_fallback` flag, and the summariser's `raw_response` (held in `DebateSummary` in memory but never persisted). No machine-readable JSON companion export exists. Write is non-atomic (direct `write_text`, no `os.replace` from a tempfile), and filename collision is possible if two runs of the same topic start in the same second.

LifeOS records the live `artifacts/` folder as 18 transcripts totalling 567KB across roughly four topics, with "how does the human brain learn" dominating (11/18) because that question reliably triggers mechanisms, causal chains, examples, and competing explanatory frameworks — unlike "what is consciousness" which the LifeOS REASONING_QUALITY doc notes "reliably triggers shallow philosophical language". LifeOS Transcripts.md calls the folder "a small but real benchmark dataset" for evaluating new summariser models or prompt variants.

## Technologies and concepts demonstrated

### Languages
- **Python 3.11+** — sole language across the package; 23 Python source files totalling ~106KB, with `consilium/` package containing 21 files at ~89KB. Largest files are `cli.py` (13KB), `prompts.py` (12.9KB), `models.py` (12.6KB), and `tui/app.py` (12.4KB) — LifeOS notes that nearly all complexity is concentrated in CLI wiring, prompt text, state models, and the TUI.

### Frameworks and libraries
- **LangChain** (`langchain>=1.2.12`) — the provider-agnostic LLM abstraction. LifeOS Providers.md notes that LangChain's 1.x line "still ships breaking changes between minor versions" and the project pins only lower bounds, which is flagged as a revival-time concern.
- **langchain-ollama** (`>=1.0.1`) — `ChatOllama` wrapper used by `OllamaChatClient`.
- **langchain-google-genai** (`>=4.2.1`) — `ChatGoogleGenerativeAI` wrapper used by `GoogleChatClient`, handling both Gemini and hosted Gemma model families.
- **Textual** (`>=8.1.1`) — the TUI framework. Used as a thin presentation layer over `MultiAgentDebateService` with the `@work` background-worker pattern for non-blocking provider calls.
- **Python `dataclasses`** — frozen dataclasses are used throughout `consilium/debate/models.py` for `DebateTurn`, `DebateRound`, `DebateSummary`, `DebateStateSnapshot`, `PerModelDebateState`, `FinalSynthesis`, `DebateTranscript`.
- **`argparse`** — CLI subcommand dispatch (`tui | debate | ask`, default = tui).
- **`unittest`** — single test module `tests/test_smoke.py` (16.3KB, 14 test functions) covering config load, service orchestration, CLI routing, structured-summary parsing, and TUI dispatch. Run via `uv run python -m unittest discover`.

### Runtimes / engines / platforms
- **Ollama** — local model runtime targeted via HTTP at `http://localhost:11434` (configurable). Default-roster local models named are `llama3.2`, `qwen3.5:4b`, `gemma3:4b`. LifeOS Suggestions.md flags Ollama model-name drift as a revival-time risk because the default roster hardcodes these identifiers.
- **Google Generative AI (Gemini API)** — hosted provider, accessed via either `GOOGLE_API_KEY` or `GEMINI_API_KEY` (first-set-wins hedge for Google's branding shift). Default hosted model is `gemini-2.5-flash-lite`; hosted Gemma (`gemma-3-27b-it`) is the LifeOS-recorded preferred hosted benchmark.

### Tools
- **uv** — the package manager / runner used for `uv run python -m consilium ...` and `uv sync`. Pyproject is the canonical declaration.
- **`pyproject.toml` scripts** — declares the `consilium` console script entry pointing at `consilium.cli:main`.
- **`.env` / dotenv-style configuration** — `consilium/config.py` reads `.env` plus `os.environ` to build `ProviderSettings` and `DebateSettings`. LifeOS Architecture.md tabulates the full 14-variable env surface, including a documented mismatch between `.env.example` (says `CONSILIUM_SUMMARIZER_MODEL=llama3.2`) and the code-default tested in `test_load_debate_settings_uses_defaults` (asserts `gemma3:4b`).

### Domains and concepts
- **Multi-LLM orchestration / debate-style prompting** — per-round per-slot client invocation, sequenced turns, shared-state injection in round 2+.
- **Anti-evaluative prompt contract** — explicit list of banned comparative-competence words in both summary and final-synthesis prompts (`dominant`, `robust`, `precise`, `failed to`, `struggled to`, `better`, `worse`, `superior`, `inferior`, `more convincing`). LifeOS Decisions.md frames this as an instance of the broader "verifiable obligations beat vague exhortations" pattern.
- **Schema-driven structured output with parse-or-fallback path** — 8-key JSON state with strict key/slot validation but tolerant per-field coercion; deterministic fallback that preserves slot content while losing cross-model reasoning fields.
- **Heterogeneous-roster reasoning-quality design** — combining different model families (`llama3.2` / `qwen3.5:4b` / `gemma3:4b`) plus layered sampling profiles (strict / balanced / exploratory) as a deliberate alternative to same-family pseudo-diversity.
- **Provider-agnostic factory + thin adapter pattern** — factory dispatch on `settings.provider`, both adapters expose a single `ask(prompt) -> str` method, errors translated into `ConsiliumProviderError` for the CLI's stderr-exit-1 path.
- **Documentation-first iteration with `IMPLEMENT_NOW_*` execution playbooks** — LifeOS records this as a recognisable Caner-workflow pattern across projects. The Consilium repo has 61KB of context docs against 89KB of source for a 4-commit repo, with two `IMPLEMENT_NOW_*` playbooks (one archived covering summary-neutrality work, one active covering structured-state reliability) following an identical Header/Modules/Function-inventory/Tasks/Verify/Invariants/Tests schema.
- **TUI-over-service composition** — Textual presentation layer with zero orchestration logic, sharing the exact same `MultiAgentDebateService` used by headless `debate`, driven via three completion callbacks.

## Key technical decisions

### Replace prose summary with structured state
LifeOS Decisions.md records the round-to-round memory was once a free-form prose summary. The prompt-only neutrality fix (banning evaluative vocabulary) was observed to fail in live runs — `gemma3:4b` stopped using the banned words but still behaved like an adjudicator. The architectural response was to replace prose with an 8-key JSON `DebateStateSnapshot`. The Decisions doc quotes the supersession rationale: *"the real architectural problem was that one free-form prose summary was carrying too much responsibility inside the loop"*. Rejected alternatives: keep prose but ban evaluative language via prompt engineering (the archived approach), no shared memory at all, raw peer outputs as shared memory.

### Agents never see raw peer outputs
A core architectural invariant: in round N+1, no agent sees what any other agent wrote in round N verbatim. They see only their own previous response and the structured state. Rationale per LifeOS: raw peer outputs contaminate independent reasoning. The fallback path's first-sentence/first-paragraph extraction is acknowledged as "inferior to the clean structured path but still better than raw contamination".

### Slot personalities removed
Earlier versions had named "Analyst" / "Builder" agent personalities. These were deliberately discarded. LifeOS Decisions.md quotes the rationale: *"Fixed personalities are no longer part of the default product model... Treating `Analyst` and `Builder` style personalities as the final product model is obsolete."* The insight is that personalities are prompt-engineering tricks that collapse into each other on small models, creating the appearance of role diversity without the substance. Real diversity comes from genuinely different model families and sampling regimes. Slots are now generic `Model A`/`Model B`/`Model C`.

### Anti-evaluative prompt contract
Both the summariser prompt and final-synthesis prompt carry an explicit banned-word list (`dominant`, `robust`, `more correct`, `failed to`, `struggled to`, etc.). Final synthesis is told not to mention participant labels at all in normal cases. The Decisions doc records a progression — soft guidance failed, explicit banned-word list alone failed, combined with the structural change (structured state) the two reinforce each other.

### Default local roster is heterogeneous
`llama3.2` + `qwen3.5:4b` + `gemma3:4b` — three different model families, not three copies of one model with sampling variation. LifeOS explicitly notes same-family small models share vocabulary basins and failure modes that sampling cannot break out of. Rejected alternatives: three copies of `llama3.2` with tiny sampling differences (the original approach), three copies of a single model with large sampling-profile gaps.

### Default summariser is `gemma3:4b`, not `llama3.2`
Live run observations are quoted in LifeOS: *"The move from `llama3.2` summarisation to `gemma3:4b` summarisation was motivated by observed summary weakness in live runs."* `llama3.2` produced summary-shaped output reliably but the summaries were repetitive and lossy. `gemma3:4b` emits better structured state in typical cases but can drift evaluative under weak prompts. `qwen3.5:4b` as summariser produced blank-output failures. Note: `.env.example` disagrees with the code default — a known mismatch.

### Structured-state emitter uses JSON not line-oriented schema
JSON was chosen for universal model support and cheap `json.loads` parsing. But LifeOS Decisions.md records this is *under active review*: the active `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY.md` Task 1 says *"Decide whether the state emitter should stay as JSON or move to a simpler line-oriented schema if local compliance remains poor."* The exit criterion is two-round local debates *usually* producing parseable structured state without fallback.

### Markdown-only transcripts
No JSON companion file, no alternative formats. Priority at Milestone 3 was auditability — a human reading a transcript can tell whether a round used clean structured state or fallback. Machine-readable export was deferred pending convergence tracking (Milestone 4), which would actually need it for alignment scoring.

### Minimal compose→run→result TUI (not a live dashboard)
The earlier multi-panel operator console with per-model live cards was replaced. The current minimal flow is an explicit UX bet: a dashboard frames the *debate* as the product, a minimal UI frames the *thesis* as the product. This matches the project's stated intent of producing a "structured knowledge artefact".

### Documentation-first iteration
The `context/` folder is 61KB against 89KB of source for a 4-commit repo, including two `IMPLEMENT_NOW_*` execution playbooks. LifeOS records this as a consistent personal-workflow pattern across Caner's vault projects, not a Consilium-specific decision — the context-doc functions as a working-memory artefact that outlives the active coding session and forces open questions to surface before code is written.

### Test-embedded canonical example
`tests/test_smoke.py::_structured_summary_response` contains a hand-written valid structured-summary string used as the response of a fake summariser client in service tests. LifeOS calls out that the fixture is doing double duty as the canonical specification of what a compliant structured-state emission looks like — more precise than any docstring or prompt text.

## What is currently built

Working features per LifeOS _Overview.md's verification table:

| Feature | State | Evidence (per LifeOS) |
|---------|-------|-----------------------|
| Single-agent `ask` command | Working | `consilium/services/single_agent.py` |
| Headless `debate` command (multi-round) | Working | `consilium/services/multi_agent_debate.py` |
| Textual TUI (compose → run → result) | Working | `consilium/tui/app.py`, 12.4KB |
| Ollama provider adapter | Working | `consilium/providers/ollama.py` via `langchain_ollama` |
| Google provider adapter (Gemini + Gemma) | Working | `consilium/providers/google.py` via `langchain_google_genai` |
| Heterogeneous default local roster | Working | `DEFAULT_LOCAL_ROSTER` in `agents/definitions.py` |
| Per-slot sampling profiles | Working | `DEFAULT_SAMPLING_PROFILES` in `agents/definitions.py` |
| Per-slot provider/model overrides (env + CLI) | Working | `cli.py _build_debate_agents` |
| Structured-state emitter (8-key JSON schema) | Working but unreliable | `parse_summary_response` in `debate/models.py` |
| Fallback state from raw turns | Working | `build_fallback_snapshot` in `debate/models.py` |
| Final narrative synthesis | Working | `MultiAgentDebateService._build_final_synthesis` |
| Markdown transcript with structured state blocks | Working | `consilium/debate/transcript.py` |
| MCP tool access | Not implemented | No MCP imports anywhere in `consilium/` |
| Convergence / divergence tracking | Not implemented | No such module exists |
| Claude / OpenAI provider adapters | Not implemented | `providers/__init__.py` |

Scale markers from LifeOS:
- 23 Python source files, ~106KB total; `consilium/` package = 21 files, ~89KB.
- 11 context-markdown files, ~61KB — described in LifeOS as "unusually thorough for a 4-commit repo".
- 1 test file (`tests/test_smoke.py`, 16.3KB, 14 test functions).
- 18 transcript Markdown files, 567KB total in `artifacts/` — the project generated more transcript data than source code.
- 4 commits total, all by Caner, spanning 2026-03-04 to 2026-03-15.

## Current state

Status: dormant. The last commit is `c592b34` on 2026-03-15, with four commits total spanning 11 days. The project was moved from "Active" to "Other" in the GitHub project list during the 2026-04 vault sync. The most recent activity in the LifeOS folder is vault-lint passes (2026-04-24 and 2026-04-28) updating frontmatter and structure on the documentation, not on the code itself. The IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY plan is recorded as Status: Active, meaning it represents queued work that was not completed before iteration paused. LifeOS Roadmap.md characterises the pause as the project hitting "a point (reliable structured state) where the next step required a clearer decision (JSON vs line-schema; MCP vs simpler tools) and the project stepped off the gas".

## Gaps and known limitations

- **README–reality drift on three top-level capabilities.** The README features Claude/GPT/Gemini/local model parity, MCP shared tool access, and programmatic convergence/divergence tracking as headline features. None of these are implemented. `grep mcp` returns zero matches in source. Only Ollama and Google adapters exist (no `langchain-anthropic` or `langchain-openai` dependencies). LifeOS contains a dedicated `README Claims vs Reality.md` file auditing every README claim against code.
- **Structured-state reliability work is unfinished.** The active `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` playbook has three tasks — prompt-compliance redesign, parser strictness with fallback annotation, transcript visibility for fallback-derived rounds — none marked complete.
- **Strict parser plus weak local-model compliance produces frequent fallback.** The parser rejects perfectly-valid JSON if any expected slot name is missing or any unknown slot name appears. Small local models drift slot names (`Agent A` vs `Model A`) and merge entries, triggering fallback.
- **Fallback path is ambient, not explicit.** `DebateSummary` does not carry a `was_fallback: bool` flag. Readers infer fallback from literal placeholder strings ("No reliable ... was available in fallback mode.") appearing in per-model entries.
- **Fallback loses cross-model reasoning entirely.** When parsing fails, the four cross-model list fields (agreements, disagreements, assumptions, concepts-to-preserve) become empty tuples; the loop continues but reasoning quality collapses to parallel monologues.
- **`raw_response` captured but discarded at write time.** The summariser's raw output is held in `DebateSummary.raw_response` but never serialised to the transcript — debugging fallback triggers requires capturing live stderr/stdout.
- **Schema field name leans knowledge-heavy.** `key_supporting_reasoning_or_mechanisms` biases the schema toward science/knowledge questions; the IMPLEMENT_NOW invariant explicitly warns against this drift and the field name escaped the guard.
- **`.env.example` disagrees with code default.** `.env.example` sets `CONSILIUM_SUMMARIZER_MODEL=llama3.2`; tests assert `gemma3:4b` as the default with no env file.
- **TUI stage updates are completion-only.** No per-stage start events from the orchestrator, so the UI appears frozen during long generations.
- **One summariser does two jobs.** The same model emits strict JSON state after each round and writes the flowing thesis at the end — LifeOS Suggestions.md raises splitting these into separate `CONSILIUM_STATE_EMITTER_*` and `CONSILIUM_SYNTHESISER_*` configs as the low-risk answer to the JSON-vs-prose conflict.
- **Host and API key shared across slots.** Cannot run two Ollama slots against two different local hosts in the same debate.
- **No per-slot CLI override for sampling.** `--agent-temperature` / `--agent-top-p` etc. do not exist; sampling is only settable via the hardcoded `DEFAULT_SAMPLING_PROFILES` or globally via env vars.
- **`repeat_penalty` silently dropped for Google slots.** Google's SDK does not expose it as a first-class kwarg; mixed rosters adjusting repeat penalties get no effect on Google slots.
- **No machine-readable transcript export.** Markdown only, no JSON companion.
- **No atomic transcript write.** Direct `write_text` with no temp-file staging.
- **Filename collision risk on sub-second starts.** `YYYYMMDD_HHMMSS_<slug>.md` would collide for two runs of the same topic starting in the same second.
- **No artefact rotation.** 18 transcripts at 567KB already; sustained use would grow unboundedly.
- **No token-usage / cost / runtime metadata in transcripts.** Easy capture from LangChain responses, flagged as planned but not done.
- **No bounds check on `agent_count`.** Accepts any positive integer; practical scaling is VRAM/context-bounded, not app-bounded.
- **No streaming.** Both adapters use `.invoke` not `.stream`.
- **No CI configuration.** No `.github/workflows`, no pre-commit hook, no `tox.ini`.
- **Tests cover orchestration but not TUI rendering.** Trust-the-framework approach for Textual.

## Direction (in-flight, not wishlist)

The only actively-queued item per LifeOS is the `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` playbook (Status: Active). Its three tasks and exit criterion are:

1. Make the state-emitter prompt easier for local models to satisfy — decide JSON vs line-oriented, reduce schema ambiguity, consider fewer required fields while staying generic.
2. Tighten parsing without over-aggressive coercion — treat missing per-model slot coverage as a fallback condition, preserve useful coercions for minor drift, add explicit fallback marker.
3. Make transcripts and synthesis clearly reflect clean state vs fallback-derived state.

Exit criterion: *"Two-round local debates usually produce parseable structured state without fallback. Fallback usage is visible and clearly secondary when it occurs."*

Everything else in LifeOS Roadmap.md is conditional on the project being resumed; the project is currently dormant, so those items are aspirational rather than in-flight.

## Demonstrated skills

- Building a multi-LLM orchestration pipeline from scratch in Python with strict layered dependencies (CLI → services → debate core → providers) and verified module-level isolation that allows the reasoning core to be swapped without touching providers and vice versa.
- Designing a schema-driven shared-state alternative to free-form prose summarisation, including a strict-key/strict-slot validator with tolerant per-field type coercion, a deterministic fallback path that preserves continuity when parsing fails, and JSON-via-ASCII serialisation chosen because non-ASCII model output caused encoding issues when round-tripped.
- Prompt engineering at production-spec-level scope: three large prompt builders (12.9KB of declarative prompt text), an explicit anti-evaluative banned-word contract for both summariser and final-synthesis prompts, an extraction-vs-summarisation contract, structured-state injection in place of raw peer history, generic-topic clauses to prevent neuroscience-specific drift.
- Building a provider-agnostic adapter layer in LangChain with factory dispatch, single-method (`ask(prompt) -> str`) thin adapters wrapping `ChatOllama` and `ChatGoogleGenerativeAI`, defensive duck-typed return handling for `AIMessage.content` shape variance, and documented provider-asymmetry handling (e.g. `repeat_penalty` silently dropped for Google because the SDK does not expose it).
- Designing a heterogeneous-roster reasoning-quality strategy combining model-family diversity (`llama3.2` + `qwen3.5:4b` + `gemma3:4b`) and layered sampling profiles (strict / balanced / exploratory) with a documented rationale that same-family small models share vocabulary basins.
- Building a Textual TUI as a thin presentation layer over a service: three-state compose→run→result flow, `@work` background-worker pattern for non-blocking provider calls, three completion callbacks driving a pipeline tracker, with a deliberate UX choice to hide intermediate model output in favour of the thesis as the visible product.
- Building a self-aware honest-state documentation discipline: a dedicated `README Claims vs Reality` audit file, an Overview that begins with a status warning callout when the project is dormant, a Gaps file that organises limitations by severity, an `IMPLEMENT_NOW_*` execution playbook pattern that captures Status/Scope/Exit-rule/Modules/Function-inventory/Tasks/Verify/Invariants/Tests in a reusable schema.
- Producing a small benchmark artefact set during development (18 transcripts across 4 topics totalling 567KB) that doubles as a reasoning-quality regression dataset for evaluating new summariser models or prompt-design variants.
- Practical knowledge of local-model behaviour at the 4B class: documented failure modes per model (`qwen3.5:4b` blank summaries, `gemma3:4b` adjudicator-drift, `llama3.2` repetitive-lossy summaries), prompt-only-fix versus architectural-fix iteration, and the design-space choice between per-round-strict-JSON-emission and per-round-prose-summary.
- Designing test-fixture-as-specification: the `_structured_summary_response` hand-written summary in `tests/test_smoke.py` serves as the canonical worked example of a compliant emission, more precise than any prompt-text or docstring.

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
