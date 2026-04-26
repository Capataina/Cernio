---
name: Tectra
status: dormant
source_repo: https://github.com/Capataina/Tectra
lifeos_folder: Projects/Tectra
last_synced: 2026-04-26
sources_read: 9
---

# Tectra

## One-line summary

Early-phase C++20 trading-infrastructure scaffold whose only working primitive is a `Clock` abstraction (`RealClock` + `VirtualClock`) chosen first specifically to make a future deterministic-replay engine possible — everything else in the README's 14-milestone production-style platform vision is unbuilt.

## What it is

Tectra is Caner's self-directed exploration of building the invisible infrastructure of a trading firm in modern C++. The stated ambition (per the README, captured in `Overview.md`) is a cohesive stack covering market-data ingest, pre-trade risk, kill switch, deterministic replay, a strategy execution framework, backtesting, and a signal research toolkit — explicitly framed as a "dual-plane" platform with a binary, lock-free, shared-memory fast path and an HTTP/Prometheus control plane. The current verified reality is foundational plumbing only: a `Clock` interface with two implementations, a `LogSeverity` enum, a CMake project with `-Werror` from line one, and a `main.cpp` that runs as a clock smoke test. LifeOS Overview frames it as a "**learning / portfolio project in its earliest phase**" where the README is intent and the code is status — the README-to-source byte ratio is ~13:1, and none of the 14 README roadmap checkboxes are ticked. The distinction between "what it is designed to do" (a production-style trading platform) and "what it currently demonstrates" (a disciplined two-file C++20 scaffold) is load-bearing for any grading judgement against this project.

## Architecture

LifeOS captures **two architectures** for Tectra explicitly: the README's intended dual-plane platform, and the actual wired graph in source. Both are reproduced because the gap is the headline finding.

### Intended architecture (from README, per `Architecture.md`)

A dual-plane design:

```
                    ┌──────────────────────────────────────────┐
                    │             Control Plane                │
                    │  CLI / TUI / HTTP • Prometheus /metrics  │
                    │  structured JSON logs • RBAC • tectractl │
                    └────────────────┬─────────────────────────┘
                                     │
     ┌───────────────────────────────┼───────────────────────────────┐
     ▼                               ▼                               ▼
┌──────────┐  shm  ┌──────────┐  shm  ┌──────────┐  shm  ┌──────────┐
│  Feed    │  ring │ Strategy │  ring │   Risk   │  ring │  Venue   │
│ Handler  │──────▶│ Runtime  │──────▶│  Engine  │──────▶│ Gateway  │
│(ITCH/etc)│       │ (plugins)│       │ (rules)  │       │ (FIX/OUCH)│
└──────────┘       └──────────┘       └──────────┘       └──────────┘
                                │
                                ▼
                ┌──────────────────────────────────┐
                │ Append-only journals + snapshots │
                │   (Merkle roots, checksums)       │
                └──────────────────────────────────┘
                                │
                                ▼
                    ┌──────────────────────────┐
                    │ Replay / Backtest Engine │
                    │  (time virtualisation)   │
                    └──────────────────────────┘

                  Kill-switch spans all services (sub-ms fan-out)
```

Intended properties (all `readme`, none `verified`): binary zero-copy lock-free shared-memory rings, cache-aligned bounded queues with backpressure on the fast path; HTTP/gRPC + Prometheus + structured JSON on the slow path; contract-first FlatBuffers/Protobuf schemas; deterministic replay via append-only journals with seedable RNG and fixed clock domains; strategy plugins as shared libraries implementing `IStrategy`; cross-service kill switch with sub-millisecond propagation. **Every box above is aspirational.** None of them have code.

### Actual wired architecture (verified from source)

```
┌─────────────────────────────────────────────┐
│             src/main.cpp                    │
│  RealClock real_clock;                      │
│   ...sleep 100ms, measure elapsed...        │
│  VirtualClock virtual_clock(0);             │
│   advance(1s); set_time(5s);                │
└────────┬───────────────────────┬────────────┘
         │                       │
         ▼                       ▼
┌─────────────────────┐   ┌──────────────────────┐
│ src/common/time.hpp │   │ src/common/logging.hpp│
│  class Clock { }    │   │  enum LogSeverity {  │
│  class RealClock    │   │   DEBUG, INFO,       │
│  class VirtualClock │   │   WARN, ERROR }      │
│  using Timestamp =  │   │  (no macros, no sink) │
│    std::int64_t     │   │                       │
└─────────────────────┘   └──────────────────────┘
```

That is the entire dependency graph. `main.cpp` is a smoke test for the clock abstraction, not a platform entry point — nothing in the codebase orchestrates services, opens ports, reads files, spawns threads, or handles messages. Three standard-library headers (`<chrono>`, `<iostream>`, `<thread>`) plus `<cstdint>` are the only external surface; there are zero third-party dependencies and no package manager configured.

### Module boundaries (intended vs wired)

LifeOS captures the asymmetry directly:

| Intended (`readme`) | Wired (`verified`) |
|---|---|
| `common/`, `feed/`, `risk/`, `kill/`, `strategy/`, `backtest/`, `replay/`, `control/`, `schema/` | `common/` only — contains `time.hpp` and `logging.hpp` |

### Code conventions actually in place

- Top-level namespace `tectra::common`.
- C++ standard **C++20**, `CMAKE_CXX_STANDARD_REQUIRED ON` (no fallback).
- Strict compile options: `-Wall -Wextra -Wpedantic -Werror`.
- Header guards via `#pragma once`.
- `final` on concrete `Clock` implementations.
- Digit separators in literals (`1'000'000'000`).

## Subsystems and components

Two subsystems exist in any meaningful sense; a third (Build) is the project shell.

### Clock (`src/common/time.hpp`)

The only substantive subsystem. Defines `tectra::common::Timestamp` (a `using = std::int64_t` representing nanoseconds since some epoch), an abstract `Clock` interface (`now()` returning `Timestamp`, `is_virtual()` returning `bool`), and two concrete `final` implementations:

- **`RealClock`** — wraps `std::chrono::steady_clock`, returns nanoseconds via `duration_cast`. Deliberately uses `steady_clock` (monotonic, never jumps backwards) rather than `system_clock`, which is correct for latency measurement but means it is not a calendar — two `RealClock` instances on different machines cannot be meaningfully compared, and a future `WallClock` over `system_clock` would be needed for venue session boundaries. No internal state; `now()` is pure and thread-safe.
- **`VirtualClock`** — holds a mutable `current_time_` (`int64_t`, **not atomic**, so multi-threaded backtests would race). Constructor takes an optional `start_time = 0`. `advance(delta_ns)` adds nanoseconds; `set_time(absolute)` jumps to an absolute value, including backwards (intentional for replay seeking, but a foot-gun if mis-invoked). `now()` is `const` and never auto-advances.

LifeOS calls this "the single most load-bearing decision in Tectra" because deterministic replay is impossible if any code reads wall-clock time directly — every service, strategy, or risk rule must hold a `const Clock&` rather than calling `steady_clock::now()`. As of `2db9f7b`, `main.cpp` itself violates this convention by calling `std::this_thread::sleep_for` directly, but it is documented as a demo-only allowance to be purged when real services arrive.

### Logging (`src/common/logging.hpp`)

A placeholder, not a subsystem. The entirety is:

```cpp
namespace tectra::common {
enum class LogSeverity { DEBUG = 0, INFO = 1, WARN = 2, ERROR = 3 };
}
```

No logger type, no sink, no macros, no formatter, no correlation IDs, no JSON output, no integration with anything else. The explicit integer values (added in commit `2db9f7b` "starting next feat.") suggest intent to use the enum as a severity comparator and/or to serialise it into structured output, but no such code exists. LifeOS Logging.md notes the design constraint that whenever a real logger is built it must accept a `Clock&` for timestamps — a logger that calls `steady_clock::now()` internally would re-break the determinism that `VirtualClock` was specifically designed to enable.

### Build (`CMakeLists.txt`, `.gitignore`)

A 13-line `CMakeLists.txt`: CMake 3.20 minimum, project version 0.1.0, C++20 with `STANDARD_REQUIRED ON`, the `-Wall -Wextra -Wpedantic -Werror` compile options, and a single `add_executable(tectra src/main.cpp)`. Zero `find_package`, zero `FetchContent`, no `vcpkg.json` / `conanfile.txt`. No `enable_testing()`, no `add_test`, no sanitizer wiring, no `clang-tidy` integration, no `.clang-format`, no `.github/workflows/`. The `.gitignore` covers `build/`, `cmake-build-*/`, `.vscode/`, `.idea/`, `*.swp`, `*.swo`, `.DS_Store` — sensible minimum, but missing entries for `compile_commands.json`, coverage artefacts (`*.gcno`/`*.gcda`), and sanitizer output that will matter once tooling is added. Build flow is the standard `cmake -S . -B build && cmake --build build && ./build/tectra`; the README does not document this.

## Technologies and concepts demonstrated

### Languages

- **C++20** — primary and only language. Used for the entire `src/` tree (`main.cpp`, `common/time.hpp`, `common/logging.hpp`). Depth: scaffold-level — type-system features in use include `enum class` with explicit underlying values, `final` virtual override, `using` aliases, `#pragma once`, digit separators (a C++14 feature), and `std::chrono` interop via `duration_cast`. No template metaprogramming, no concepts, no coroutines, no ranges in current source.

### Frameworks and libraries

No third-party libraries. The only dependencies are C++ standard library headers (`<chrono>`, `<iostream>`, `<thread>`, `<cstdint>`).

### Runtimes / engines / platforms

No runtimes engaged; the binary is a single-process, single-threaded console executable.

### Tools

- **CMake 3.20** — build configuration, single `CMakeLists.txt`, no subdirectories.
- **GCC / Clang (implicit)** — required to honour `-Wall -Wextra -Wpedantic -Werror`. No specific toolchain pinned.

### Domains and concepts

- **Time virtualisation** — abstracting wall-clock access behind a `Clock` interface so that production injects `RealClock` and replay/backtest/test injects `VirtualClock`, which is the foundational primitive for deterministic replay. Implemented end-to-end (interface plus both concrete classes plus a demo exercising both).
- **Monotonic vs system clocks** — deliberate choice of `std::chrono::steady_clock` over `system_clock` to avoid NTP-jump non-monotonicity in latency measurement.
- **`-Werror`-from-day-one strictness posture** — a discipline-lab framing for a portfolio project, not a default setting.
- **Trading-infrastructure architecture vocabulary** (intent only, no code): dual-plane (fast/control), shared-memory rings (SPSC/MPMC), contract-first schemas (FlatBuffers/Protobuf), deterministic replay with append-only journals and Merkle roots, kill-switch with sub-ms fan-out, FIX/OUCH/SBE/FAST adapters, strategy plugin model, pre-trade risk engine, fill simulator + TCA backtesting, signal research toolkit (SMA/EMA/RSI/MACD/Bollinger/Z-score/cointegration). These are README-named only and explicitly captured in LifeOS as `readme` not `verified` — reproduced here as evidence of design vocabulary, not implemented capability.

## Key technical decisions

LifeOS `Decisions.md` captures six load-bearing decisions to date.

**D1 — Modern C++ (C++20) as the primary language.** Enforced by `CMAKE_CXX_STANDARD 20` with `CMAKE_CXX_STANDARD_REQUIRED ON`. Alternatives weighed in LifeOS: Rust (the natural alternative; Caner's other trading-infra project Nyquestro uses Rust for the same domain), Zig (rejected for ecosystem thinness around FIX/FlatBuffers/Prometheus), Modern C++ (chosen — industry-realistic for HFT, libraries like QuickFIX/Aeron available, and a deliberate skill-expansion away from Caner's Rust focus). Trigger that would change it: a serious memory-safety bug Rust would have caught, or a pivot to make Tectra cooperate with Nyquestro rather than contrast with it.

**D2 — `-Werror` from day one.** `add_compile_options(-Wall -Wextra -Wpedantic -Werror)`, annotated in source with `# Compiler warnings - be strict from day one`. Rationale: every warning is a real problem now while the codebase is 2 KB; impossible to clean up later in 200 KB. `-Wpedantic` specifically rejects non-standard extensions, which matters for the README's "portable" claim. Tradeoff accepted: builds will break on toolchain upgrades that introduce new warnings. Trigger for revisiting: a third-party header that triggers warnings impossible to suppress locally — at which point `-Werror` would be scoped to Tectra's own targets only.

**D3 — Virtual-clock-first ordering.** The first substantive code (commit `a11fd04`) was the `Clock`/`RealClock`/`VirtualClock` abstraction, before any service, feed handler, or message type. Rationale: deterministic replay (a stated README differentiator) is impossible if any code reads wall-clock time directly, so the clock abstraction must predate every component that might read time. Alternative considered and rejected: starting with a feed handler (the flashy, README-lead choice) — would produce visible progress faster but would have to be rewritten the moment replay is added because the first draft would call `steady_clock::now()` everywhere. Why this is a defensible call: services receiving `const Clock&` literally cannot call `advance()` or `set_time()`; only the replay harness can. Trigger for revisiting: discovering virtual-time discipline is too costly at the µs latency targets (the `Clock&` virtual call becomes measurable in the hot path), with mitigation being templatising on `Clock` type for release builds.

**D4 — No dependencies in the scaffold phase.** Zero `find_package`, zero `FetchContent`, no manifest. Rationale: keeps the project trivially buildable and avoids premature commitment to a specific Protobuf/FlatBuffers/QuickFIX version. Cost pushed downstream: the first real subsystem will force a dependency-management decision (vcpkg/Conan/FetchContent/manual) that the current scaffold provides no template for. Likely trigger: starting Milestone 1 (Foundations & Contracts).

**D5 — README-first, then code.** The first three commits (`11697b8`, `af1dd47`, `5b56310`) are README-only, with `af1dd47` alone adding +488 lines of README before any source existed. Rationale: README functions as a design document; writing it first is equivalent to writing a spec before implementation; useful for a portfolio project where evaluators read the README first. Risk this creates: README becomes stale relative to code the moment code diverges, and six months of dormancy since `2db9f7b` means the README is the public face of a project that barely exists — a documentation debt item already tracked in LifeOS Gaps.

**D6 — Single-file `main.cpp` as demo, not service host.** `main.cpp` is a clock smoke test, not a process orchestrator. Consistent with having no services to orchestrate, but leaves the binary-vs-multi-service-vs-library question unresolved. README implies multi-process with shared-memory rings, but the transition would require replacing or demoting `main.cpp` and adding CLI argument parsing, config loading, signal handlers, and graceful shutdown — none of which exist.

**Decisions still owed** (captured in LifeOS as load-bearing for any resumption): schema format (FlatBuffers vs Protobuf vs capnp), shared-memory ring library (roll-own vs Aeron vs Disruptor port), HTTP library (Boost.Beast vs cpp-httplib vs drogon vs custom), metrics library, test framework (GoogleTest vs Catch2 vs doctest), dependency manager (vcpkg vs Conan vs FetchContent vs manual), logger library (spdlog async vs fmtlog vs custom), FIX library (QuickFIX vs custom). LifeOS Evolution attributes the project's stall directly to this cluster — four-plus consecutive architecture decisions with real opportunity cost being much harder to start on a random evening than a clear next code task.

## What is currently built

LifeOS Overview enumerates the implemented surface against the README claim explicitly:

| Area | Status | Evidence |
|---|---|---|
| Build system (CMake 3.20, C++20, `-Wall -Wextra -Wpedantic -Werror`) | Built; sanitizers not wired | `CMakeLists.txt` |
| Clock abstraction (`Clock` interface, `RealClock`, `VirtualClock`, nanosecond `Timestamp`) | Built end-to-end | `src/common/time.hpp` |
| Logging severity enum (`LogSeverity { DEBUG, INFO, WARN, ERROR }`) | Enum only — no macros, no sinks, no formatter | `src/common/logging.hpp` |
| Entry point (`main.cpp` exercising both clocks and printing diagnostics) | Built as a smoke test | `src/main.cpp` |
| Feed handler / ITCH-OUCH normaliser, L2 books, derived metrics | Not started | LifeOS scan |
| Pre-trade risk engine (price bands, size limits, credit, throttles) | Not started | LifeOS scan |
| Kill switch (sub-ms fan-out, cancel/isolate/slow-mode) | Not started | LifeOS scan |
| Deterministic replay (append-only journals, Merkle roots, golden-run diffing) | Not started; `VirtualClock` is the first enabling primitive | LifeOS scan |
| Strategy framework (plugin API, signal library, position manager) | Not started | LifeOS scan |
| Backtesting engine (fill simulator, TCA, P&L) | Not started | LifeOS scan |
| Tests (unit / property / fuzz / golden-replay) | **Zero test files** | `repo_stats.json test_ratio 0/4` |
| Schemas (FlatBuffers / Protobuf) | None present (no `.proto`/`.fbs`) | LifeOS scan |
| Shared-memory rings (lock-free SPSC / MPMC) | Not present | LifeOS scan |

**Scale markers** (from LifeOS Overview, sourced from `repo_stats.json` and `fetch_commits.py`):

| Metric | Value |
|---|---|
| Total files | 6 |
| Source files (`.cpp`/`.hpp`) | 4 |
| Source bytes | 2,255 |
| README bytes | 29,133 |
| Test files | 0 |
| Total commits | 6 |
| First commit | 2025-10-05 |
| Last commit | 2025-10-10 (`2db9f7b`, "starting next feat.") |
| Days of active development | 6 |
| Months dormant (as of 2026-04-24) | ~6 |
| README-to-source byte ratio | ~13:1 |

The README-to-source ratio is the headline finding: a project documented at ~13× its code is one that has been thought about far more than it has been built.

## Current state

Status: **dormant**. Last commit was `2db9f7b` on 2025-10-10 ("starting next feat." — a 3-add/2-delete tweak to `logging.hpp` likely adding the explicit `= 0`/`= 1`/... severity values). No commits, no issues, no branch activity in the ~6 months since (as of LifeOS verification on 2026-04-24, two days before this sync). LifeOS Evolution attributes the stall to Milestone 1 decision overhead — the next step requires choosing a schema library, a test framework, a logger, and a shared-memory ring design in succession, which is qualitatively harder to start on than a clear coding task. No work-in-flight items (LifeOS folder has no `Work/` subfolder for Tectra). The repo is public but unlicensed (no `LICENSE` file), which under default copyright law restricts third-party use — fine for a read-only portfolio piece, blocking for any external contribution.

## Gaps and known limitations

Captured comprehensively in LifeOS `Gaps.md` across five tiers; the career-relevant subset:

- **Tier 1 (foundational, blocks all downstream work):** no schemas, no shared-memory rings, no real logger, no config loader, no test infrastructure (zero tests, no CI, no sanitizer wiring), no HTTP/gRPC scaffold, no Prometheus `/metrics`, no `.github/workflows/` CI pipeline.
- **Tier 2 (subsystem-level, all README-promised, all missing):** feed handler / ITCH decoder, normaliser with derived metrics (VWAP/spread/imbalance), pre-trade risk engine, per-strategy position tracking, strategy execution framework + signal library + example strategies, backtesting engine + journals + deterministic replay harness, kill switch / circuit breaker, research toolkit (correlation scanner, cointegration tester, regime detector), operator tooling (`tectractl`), performance/latency engineering (no flamegraphs / sched traces / regression gates), FIX 4.4/5.0 adapter, SBE/FAST codecs, OUCH adapter, fault injection / chaos, packaging / compose / supervisor. **19 subsystem-level gaps**, each multi-week.
- **Tier 4 (documentation/process):** no build instructions in README, no "Current Status" section to distinguish vision from reality, no `LICENSE`, terse commit messages (`.`, "starting next feat.").
- **Tier 5 (conceptual decisions deferred):** process model undecided (threads-in-one-binary vs separate binaries with shared memory vs library); target OS undecided (`io_uring`/`AF_XDP` are Linux-specific yet README claims portability); single-host vs distributable position unclear; licence/commercial position undeclared; audience (portfolio vs real tool vs educational) unspecified.

Internal subsystem-level gaps for the components that do exist:

- `VirtualClock::current_time_` is plain `int64_t`, not atomic — multi-threaded backtests would race.
- No `WallClock` over `system_clock` for calendar time (e.g. venue session boundaries).
- No tests for the clock (or for anything else).
- The `is_virtual()` runtime-dispatch pattern is weak; templatising services on `Clock` type would remove the virtual call if latency measures it.
- The "no direct `steady_clock::now()` outside the clock abstraction" convention is not enforced by any linter or code-review check; `main.cpp` itself currently violates it via `std::this_thread::sleep_for`.

## Direction (in-flight, not wishlist)

There is no in-flight work. The repo has been dormant for ~6 months and LifeOS does not maintain a `Work/` folder for Tectra. The README's 14-milestone roadmap is explicitly captured in LifeOS as **intent, not a delivery schedule** — none of the 14 milestones is checked, and Milestone 1 (Foundations & Contracts) is itself only ~5% complete (the build system, the clock, and the logging enum), making the full plan ~0.3% complete by LifeOS's own accounting.

LifeOS Roadmap enumerates four scope-options for any hypothetical resumption — full-fidelity build of all 14 milestones (3-4 years), foundations + one vertical slice (2-4 months), re-scope as a learning lab without "production-style" framing, or retire — but flags none of them as active. Pragmatically the project is awaiting a decision about whether to resume, re-scope, or shelve, with the LifeOS-noted likely triggers being a finance-sector job application, a Cernio portfolio saturation, or an explicit "force-the-Milestone-1-decisions" session.

## Demonstrated skills

What this specific project, as it exists at `2db9f7b`, evidences:

- **C++20 scaffolding from scratch with disciplined defaults** — `STANDARD_REQUIRED ON`, `-Wall -Wextra -Wpedantic -Werror` from line one of `CMakeLists.txt` (annotated in source with the explicit "be strict from day one" comment), `final` on virtual overrides, `#pragma once` header guards, namespace hygiene under `tectra::common`.
- **Designing a time-virtualisation abstraction correctly** — `Clock` interface with separate `RealClock` (over monotonic `steady_clock`) and `VirtualClock` (mutable, manually advanced) implementations, nanosecond-resolution `int64_t Timestamp`, deliberate `steady_clock`-not-`system_clock` choice for latency measurement, and conscious design of which mutating methods (`advance`, `set_time`) live on the concrete `VirtualClock` type only so that callers holding `const Clock&` cannot break determinism.
- **Building primitives in dependency order** — choosing time virtualisation as the first substantive subsystem because deterministic replay (the README's stated differentiator) is impossible if any service ever reads wall-clock time directly, rather than starting with the flashier feed handler or fake exchange.
- **README-as-design-document** — producing a 488-line README expansion that articulates a 14-milestone plan with per-milestone Scope / Interfaces / Data Path / Control Plane / Storage / Observability / Testing / Exit Criteria sections (per LifeOS Evolution); coherent enough that LifeOS Gaps explicitly notes it would be "a usable spec if Caner ever resumes."
- **Trading-infrastructure architecture vocabulary** — design literacy across dual-plane platforms, lock-free shared-memory rings (SPSC/MPMC), contract-first schemas (FlatBuffers/Protobuf), deterministic replay via append-only journals with Merkle roots, FIX/OUCH/SBE/FAST adapters, kill-switch fan-out, strategy plugin models, fill-simulation + TCA backtesting, and signal-research primitives (SMA/EMA/RSI/MACD/Bollinger/Z-score/cointegration). README-level only — not implemented — but evidence of design familiarity with the domain.
- **Honest project self-assessment** — LifeOS itself, captured by Caner, distinguishes README-claim from code-reality at every step (the per-feature `verified` vs `readme` tagging, the explicit "what is built (almost nothing) vs planned (everything)" framing, the public acknowledgement that the README is intent and the vault notes are status). Skill in honest self-instrumentation is a portfolio signal independent of the code itself.

What this project does **not** evidence (despite the README's framing): any low-latency system actually built and benchmarked, any FIX/OUCH/ITCH/SBE protocol code, any lock-free data structure implementation, any HTTP/gRPC service, any test or sanitizer harness, any CI pipeline, any exchange connectivity. These are README ambitions only.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Tectra/Overview.md | 89 | "> If a future Caner or future session reads the README and the vault notes and they disagree, the vault notes reflect code. The README has not been updated to mark what is built vs planned." |
| Projects/Tectra/Architecture.md | 164 | "- [[Tectra/Roadmap]] — the README's 14 milestones with velocity overlay" |
| Projects/Tectra/Decisions.md | 154 | "- [[Tectra/Roadmap]] — each open question is pinned to its README milestone" |
| Projects/Tectra/Evolution.md | 98 | "- [[Tectra/Systems/Clock]] — detailed history of the one substantive subsystem" |
| Projects/Tectra/Gaps.md | 149 | "- [[Tectra/Architecture]] — intended vs wired architecture side by side" |
| Projects/Tectra/Roadmap.md | 154 | "- [[Nyquestro/Overview]] — parallel early-stage trading-infra project; cross-pollination possibility" |
| Projects/Tectra/Systems/Build.md | 119 | "- [[Tectra/Roadmap]] — Milestone 1 (Foundations) is where the rest of this gets built" |
| Projects/Tectra/Systems/Clock.md | 146 | "- [[Nyquestro/Overview]] — Nyquestro uses event frames with embedded timestamps; a fantasy-integration would feed virtual time from Tectra's replay into Nyquestro's engine" |
| Projects/Tectra/Systems/Logging.md | 84 | "- [[Tectra/Roadmap]] — the README's Milestone 1 (Foundations) is where logging gets built" |
