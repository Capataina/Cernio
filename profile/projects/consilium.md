---
name: Consilium
status: dormant
source_repo: https://github.com/Capataina/Consilium
lifeos_folder: Projects/Consilium
last_synced: 2026-04-29
sources_read: 15
---

# Consilium

## One-line summary

Multi-LLM debate and knowledge-synthesis CLI/TUI in Python — runs a question through multiple model slots (Ollama + Google Gemini/Gemma via LangChain), compresses each round into an 8-key structured-state JSON object instead of free-form prose, feeds state forward across rounds, and produces a thesis-style synthesis transcript; reached Milestone 3 of 7 then stopped.

## What it is

Consilium is a multi-LLM debate and knowledge-synthesis CLI/TUI written in Python 3.11+. It runs the same question through multiple model slots, compresses each round into a structured shared state object rather than free-form prose, feeds that state forward so later rounds build on it, and produces a final thesis-style synthesis written to a Markdown transcript. Two provider adapters are wired through LangChain: local **Ollama** and hosted **Google** (Gemini + hosted Gemma). The README describes a broader system than the code implements — claims of "Claude, GPT, Gemini, and local models" are not accurate (only Ollama + Google adapters exist), "MCP tool access" is not implemented (Milestone 5 unchecked, zero MCP imports anywhere), "convergence and divergence tracking" is Milestone 4 unchecked, "rolling summarisation" is implemented but architecturally different from the README framing (it is a *structured-state emitter* outputting an 8-key JSON object). Project moved from Active to Other in 2026-04 vault-sync. Last commit `c592b34` on 2026-03-15. Reached Milestone 3 of a seven-milestone roadmap, then stopped.

## Architecture

```
consilium/                      # 21 files, ~89KB
├── cli.py                     # 13KB — entry point + slot wiring
├── tui/
│   └── app.py                 # 12.4KB — minimal compose → run → result Textual TUI
├── services/
│   ├── single_agent.py        # `ask` command (single-LLM)
│   └── multi_agent_debate.py  # `debate` command (multi-round)
├── agents/
│   └── definitions.py         # DEFAULT_LOCAL_ROSTER + DEFAULT_SAMPLING_PROFILES
├── debate/
│   ├── orchestrator.py        # Round driver, structured-state loop
│   ├── models.py              # 12.6KB — parse_summary_response, build_fallback_snapshot
│   └── transcript.py          # Markdown transcript with structured-state blocks
├── providers/
│   ├── ollama.py              # langchain_ollama wrapper
│   └── google.py              # langchain_google_genai wrapper (Gemini + Gemma)
├── prompts.py                 # 12.9KB — anti-evaluative prompt contract
└── tests/
    └── test_smoke.py          # 16.3KB — 14 test functions
```

The orchestrator runs N rounds; each round each slot produces a turn, the model compresses the round to an 8-key JSON object via `parse_summary_response`, fallback path (`build_fallback_snapshot`) constructs state from raw turns if parsing fails, then a final synthesis pass produces the thesis. Transcript artefacts written to a per-run folder.

## Subsystems and components

| Subsystem | Responsibility |
|---|---|
| **Debate Orchestrator** | Round driver, structured-state loop, slot dispatch |
| **Structured Debate State** | 8-key JSON contract + fallback path from raw turns when LLM-produced JSON is malformed |
| **Prompts** | Anti-evaluative prompt contract — no winner-picking, no comparative competence language |
| **Providers** | Ollama (local) + Google (Gemini + Gemma); LangChain wrappers |
| **Roster and Sampling** | DEFAULT_LOCAL_ROSTER + DEFAULT_SAMPLING_PROFILES; per-slot temp/top_p overrides |
| **TUI** | Textual; minimal compose → run → result flow |
| **Transcripts** | Markdown transcripts with embedded structured-state blocks per round |

## Technologies and concepts demonstrated

### Languages
- **Python 3.11+** — entire codebase, 23 files, ~106KB.

### Frameworks
- **LangChain** — provider-adapter framework via `langchain_ollama` and `langchain_google_genai`.
- **Textual** — TUI framework (compose / run / result flow).

### Libraries
- **`langchain_ollama`** — local Ollama wrapper.
- **`langchain_google_genai`** — Gemini + hosted Gemma access.

### Domains and concepts
- **Multi-LLM orchestration** — running the same question through heterogeneous model slots; the premise being that structured disagreement between models with different training data + objectives + blind spots produces a more useful artefact than any single model's answer.
- **Structured-state debate emission** — instead of free-form prose summaries between rounds, an 8-key JSON object is emitted; later rounds receive structured state, not free text. Sidesteps prompt-context bloat.
- **Anti-evaluative prompt contract** — prompts deliberately do not ask models to evaluate each other's competence, pick winners, or rank arguments. Same design instinct as the principal-engineer personality's evidence-anchored framing.
- **Fallback synthesis from raw turns** — when LLM-produced JSON is malformed, `build_fallback_snapshot` constructs state from raw turns rather than failing the round.
- **Per-slot sampling profiles** — different temperature/top-p per slot for diversity of responses.
- **TUI design via Textual** — compose-run-result minimal flow.

## Key technical decisions

- **Structured state, not free-form prose** — 8-key JSON between rounds.
- **Anti-evaluative prompt contract** — no winner-picking, no comparative competence language.
- **Documentation-first iteration** — `IMPLEMENT_NOW_*` execution playbooks (one archived, one active when the project stopped). Pattern echoes Cernio's plan-driven development.
- **Two providers (Ollama + Google) instead of universal LLM gateway** — kept the surface manageable; Claude / OpenAI listed in README but not wired.
- **Fallback path mandatory** — `build_fallback_snapshot` runs whenever JSON parsing fails so a round never silently drops.
- **Transcripts as the artefact** — Markdown transcripts with embedded state blocks are the durable output; runtime debate state is throwaway.

## What is currently built

- 4 commits on master (2026-03-04 to 2026-03-15).
- 23 Python source files (~106KB), `consilium/` package 21 files (~89KB).
- 2 provider adapters (Ollama + Google).
- Single-agent `ask` command + headless `debate` command + Textual TUI.
- Structured-state emitter (8-key JSON) + raw-turns fallback.
- Final narrative synthesis.
- Markdown transcript with structured-state blocks.
- 18 transcript artefacts (567KB total — the project generated more transcript data than source code).
- 1 test file (`test_smoke.py`, 16.3KB, 14 test functions).
- 11 context docs (61KB) — documentation density unusually high relative to 4 commits.

## Current state

Dormant. Project moved from Active to Other in 2026-04 vault-sync. Last commit 2026-03-15. Reached Milestone 3 of 7 and stopped. `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` was the last open work item.

## Gaps and known limitations

- **No Claude or OpenAI provider adapters** — README claims them; not wired.
- **No MCP tool access** — Milestone 5 unchecked; zero `mcp` imports anywhere in source.
- **No convergence / divergence tracking** — Milestone 4 unchecked; no alignment scoring.
- **Structured-state emitter is unreliable** — `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` was the open work item when the project stopped; fallback path masks frequent JSON-parse failures.
- **Local-model polish (Milestone 6)** + **synthesis polish (Milestone 7)** unimplemented.
- **README claims diverge from code** — the dedicated `README Claims vs Reality.md` LifeOS file enumerates them.

## Direction (in-flight, not wishlist)

Dormant. If revived, the next concrete step is `IMPLEMENT_NOW_STRUCTURED_STATE_RELIABILITY` (the last open plan); after that, Milestone 4 (convergence tracking) is the highest-leverage feature for the comparison thesis.

## Demonstrated skills

- **Multi-LLM orchestration** — running heterogeneous models through a single debate driver; structured state shared across rounds; thesis-style synthesis.
- **LangChain provider adapter pattern** — wrapping `langchain_ollama` and `langchain_google_genai` behind a uniform provider interface; per-slot sampling profiles.
- **Structured prompt engineering** — 8-key JSON state schema as the inter-round contract; anti-evaluative prompt design that avoids the LLM-as-judge failure mode.
- **Textual TUI design** — minimal compose-run-result flow, per-round state visualisation.
- **Documentation-first iteration discipline** — `IMPLEMENT_NOW_*` execution playbooks; design rounds drive feature work.
- **Fallback path mandatory** — robust degradation when LLM-produced JSON is malformed.
- **Reasoning about comparative-LLM evaluation traps** — anti-evaluative prompt contract avoids asking models to rank each other.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Consilium/_Overview.md | 109 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
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
