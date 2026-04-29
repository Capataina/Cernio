---
name: Tectra
status: paused
source_repo: https://github.com/Capataina/Tectra
lifeos_folder: Projects/Tectra
last_synced: 2026-04-29
sources_read: 9
---

# Tectra

> [!note] Status note — schema enum mapping
> LifeOS records this project as `scaffold`. Mapped to schema-conforming `paused` for the frontmatter (last commit 2025-10-10; ~6 months dormant at LifeOS verification).

## One-line summary

C++20 trading-infrastructure scaffold — CMake build with `-Wall -Wextra -Wpedantic -Werror`, a `Clock` interface with `RealClock` + `VirtualClock` time-virtualisation primitives, a 4-value `LogSeverity` enum, a `main.cpp` exercising the two clocks; README pitches a production-style stack (feed handler, pre-trade risk, kill switch, deterministic replay, strategy framework, backtesting, FIX) but no substantive subsystem is built.

## What it is

Tectra is Caner's self-directed exploration into building the invisible infrastructure of a trading firm in modern C++. The stated ambition: a cohesive stack covering market-data ingest, pre-trade risk, kill switch, deterministic replay, a strategy execution framework, backtesting, and a signal research toolkit. The current reality: foundational plumbing for time virtualisation and build hygiene, nothing else. It is a learning / portfolio project in its earliest phase. The README's 14-milestone roadmap is a design document, not a delivery tracker — none of the milestones are checked. The README-to-source byte ratio is ~13:1 (29,133 byte README vs 2,255 byte source); a project whose documentation is 13× its code is a project that has been thought about far more than built.

## Architecture

```
tectra/
├── CMakeLists.txt          # CMake 3.20, C++20, -Wall -Wextra -Wpedantic -Werror
├── .gitignore
└── src/
    ├── main.cpp            # Prints "Tectra v0.1.0 - Clock Abstraction Test"
    └── common/
        ├── time.hpp        # Clock interface + RealClock + VirtualClock + Timestamp (ns)
        └── logging.hpp     # LogSeverity { DEBUG, INFO, WARN, ERROR }
```

4 source files (.cpp/.hpp). 6 commits across 6 days of activity (2025-10-05 → 2025-10-10).

## Subsystems and components

| Subsystem | README claim | Actual code | State |
|---|---|---|---|
| **Build** | CMake, clang-tidy, ASAN/UBSAN | CMake 3.20, C++20, `-Wall -Wextra -Wpedantic -Werror`; no sanitizers wired | Working build; sanitizer wiring pending |
| **Clock** | Time virtualisation foundational to replay | `RealClock` + `VirtualClock` implementing shared `Clock` interface; nanosecond `Timestamp` | Working — the one substantive subsystem |
| **Logging** | Structured logs, JSON, correlation IDs | `LogSeverity { DEBUG, INFO, WARN, ERROR }` enum; no macros, no sinks, no formatter | Enum only |
| **Entry point** | Multi-service platform | `main.cpp` prints version + exercises the two clocks | Stub |
| **Feed handler / normaliser** | ITCH/OUCH decode, L2 books, derived metrics | Not started | — |
| **Pre-trade risk engine** | Price bands, size limits, credit, per-venue throttles | Not started | — |
| **Kill switch** | Sub-ms fan-out, cancel/isolate/slow-mode | Not started | — |
| **Deterministic replay** | Append-only journals, Merkle roots, golden-run diffing | Not started — `VirtualClock` is the first primitive | Foundation only |
| **Strategy framework** | Plugin API, signal library, position manager | Not started | — |
| **Backtesting engine** | Fill simulator, TCA, P&L | Not started | — |
| **FlatBuffers / Protobuf schemas** | Inter-service messaging schemas | Not present | — |
| **Lock-free shared-memory rings** | SPSC / MPMC | Not present | — |
| **Tests** | Unit + property + fuzz + golden-replay | Zero test files | — |

## Technologies and concepts demonstrated

### Languages
- **C++20** — entire codebase. 4 files. Modern idioms (interface class with virtual destructor, nanosecond `Timestamp` typedef).

### Tools
- **CMake** — build system with strict warning posture from day one.

### Domains and concepts
- **Time virtualisation** — `Clock` interface separating `RealClock` (system time) from `VirtualClock` (test/replay-controlled time). The most load-bearing primitive in the project — every deterministic-replay design downstream depends on it.
- **Trading infrastructure design space** — feed handler, pre-trade risk, kill switch, deterministic replay, strategy framework, backtesting, FIX (design intent visible in README + Roadmap, not implemented).
- **Strict build hygiene from day one** — `-Wall -Wextra -Wpedantic -Werror` guarantees zero warnings on any commit.
- **Nanosecond timestamps** — `Timestamp` typedef in `time.hpp` codifies the precision target.

## Key technical decisions

- **Virtual-clock-first ordering** — time virtualisation is the most load-bearing primitive (every deterministic-replay design depends on it), so `Clock` + `RealClock` + `VirtualClock` ship before any feed handler.
- **C++20** — modern idioms, std::chrono-friendly nanosecond `Timestamp`, RAII-friendly interface design.
- **`-Werror` from day one** — strictness posture decision; zero warnings is a precondition for every commit.
- **CMake without conan/vcpkg** — keep dependency surface minimal at scaffold stage.
- **No sanitizers wired yet** — flagged for later milestones.

## What is currently built

- 6 commits over 6 days (2025-10-05 → 2025-10-10).
- Working CMake C++20 build with strict warnings.
- `Clock` interface + `RealClock` + `VirtualClock` + nanosecond `Timestamp` in `time.hpp`.
- `LogSeverity` enum in `logging.hpp` (no macros, sinks, or formatter yet).
- `main.cpp` exercises the two clocks.
- 2,255 byte source vs 29,133 byte README — ~13:1 README-to-source ratio.

## Current state

Paused (LifeOS: `scaffold`). Last commit 2025-10-10 ("starting next feat."). ~6 months dormant at LifeOS verification.

## Gaps and known limitations

- **Zero tests** — no `tests/` directory, no `#[cfg(test)]` analogue.
- **No sanitizers wired** despite README claim.
- **No structured logging** — only the `LogSeverity` enum exists; no macros, no JSON sinks, no formatter, no correlation IDs.
- **No feed handler, no pre-trade risk, no kill switch, no replay, no strategy framework, no backtesting, no FIX** — every substantive subsystem is unbuilt.
- **No FlatBuffers / Protobuf schemas** — design intent only.
- **No lock-free SPSC / MPMC rings** — design intent only.
- **README-to-source ratio 13:1** — the headline finding for the project's current state.

## Direction (in-flight, not wishlist)

Paused. If revived: complete Milestone 1 (Foundations) — wire the structured logging around `LogSeverity`, add sanitizer flags, write the first tests against `VirtualClock` (deterministic time advance verification). Then Milestone 2 onward toward the feed handler.

## Demonstrated skills

- **Modern C++20 design** — `Clock` interface + concrete implementations + nanosecond `Timestamp` typedef.
- **Time virtualisation as a foundational primitive** — recognising that deterministic replay requires virtual clocks before anything else; correctly placing the most load-bearing primitive first in the implementation order.
- **Strict build hygiene** — `-Wall -Wextra -Wpedantic -Werror` from day one decision.
- **Trading infrastructure design awareness** — design intent across feed handlers, pre-trade risk, kill switches, deterministic replay, strategy frameworks, backtesting, and FIX (visible in README + Roadmap, even though not implemented).
- **Anti-puffing discipline** — Tectra's vault notes explicitly enumerate that the README is intent, not status, and every claim of functionality must come from code.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Tectra/_Overview.md | 89 | "> If a future Caner or future session reads the README and the vault notes and they disagree, the vault notes reflect code. The README has not been updated to mark what is built vs planned." |
| Projects/Tectra/Architecture.md | 164 | "- [[Tectra/Roadmap]] — the README's 14 milestones with velocity overlay" |
| Projects/Tectra/Decisions.md | 154 | "- [[Tectra/Roadmap]] — each open question is pinned to its README milestone" |
| Projects/Tectra/Evolution.md | 98 | "- [[Tectra/Systems/Clock]] — detailed history of the one substantive subsystem" |
| Projects/Tectra/Gaps.md | 149 | "- [[Tectra/Architecture]] — intended vs wired architecture side by side" |
| Projects/Tectra/Roadmap.md | 154 | "- [[Nyquestro/_Overview]] — parallel early-stage trading-infra project; cross-portfolio comparator" |
| Projects/Tectra/Systems/Build.md | 119 | "- [[Tectra/Roadmap]] — Milestone 1 (Foundations) is where the rest of this gets wired" |
| Projects/Tectra/Systems/Clock.md | 146 | "- [[Nyquestro/_Overview]] — Nyquestro uses event frames with embedded timestamps" |
| Projects/Tectra/Systems/Logging.md | 84 | "- [[Tectra/Roadmap]] — the README's Milestone 1 (Foundations) is where logging gets wired" |
