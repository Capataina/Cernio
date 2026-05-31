---
name: Tectra
status: dormant
source_repo: https://github.com/Capataina/Tectra
lifeos_folder: Projects/Tectra
last_synced: 2026-05-31
sources_read: 9
---

# Tectra

## One-line summary

C++20 trading-infrastructure scaffold whose only working subsystem is a virtual-clock abstraction built deliberately before any service, so that future replay and backtest layers can be deterministic by construction.

## What it is

Tectra is Caner's self-directed exploration into building the invisible infrastructure of a trading firm in modern C++. The README positions it as a dual-plane platform covering market-data ingest, pre-trade risk, kill switch, deterministic replay, a strategy execution framework, backtesting, and a signal research toolkit, with a fast (binary, lock-free, shared-memory rings) path and a slow (HTTP/gRPC, Prometheus, structured logs) control path. The implemented reality at HEAD `2db9f7b` is much narrower: a 2 KB C++20 scaffold containing a `Clock` interface with `RealClock` and `VirtualClock`, a four-value `LogSeverity` enum, a `main.cpp` that prints clock diagnostics, and a CMake build wired with `-Wall -Wextra -Wpedantic -Werror` from day one. The LifeOS vault treats the README as intent and the source tree as status; the README-to-source byte ratio is roughly 13:1. As of the last vault verification (2026-04-24), the project has been dormant for about six months since the commit message "starting next feat." on 2025-10-10.

## Architecture

LifeOS records two distinct architectures: the one the README describes and the one the code actually wires.

**Intended (README, aspirational — no code backing):** a multi-service trading platform with feed handler, strategy runtime, risk engine, and venue gateway communicating over lock-free shared-memory rings; a control plane exposing CLI/TUI/HTTP, Prometheus `/metrics`, structured JSON logs, and `tectractl` operator tooling; append-only journals with Merkle roots feeding a replay/backtest engine; a cross-service kill switch with sub-millisecond fan-out; contract-first schemas (FlatBuffers/Protobuf) versioned across services; strategy plugins implementing `IStrategy` as hot-reloadable shared libraries.

**Wired (source, verified at `2db9f7b`):**

```
src/main.cpp
   ├──▶ src/common/time.hpp   (Clock, RealClock, VirtualClock, Timestamp = int64_t)
   └──▶ src/common/logging.hpp (enum class LogSeverity { DEBUG=0, INFO=1, WARN=2, ERROR=3 })
```

`main.cpp` is a single-threaded smoke test that constructs a `RealClock`, measures elapsed time across a 100 ms `sleep_for`, then constructs a `VirtualClock(0)`, calls `advance(1s)` and `set_time(5s)`, and exits. Nothing in the codebase opens ports, spawns threads beyond `main`, reads files, dispatches messages, or applies backpressure.

**Conventions verified in source:** top-level namespace `tectra::common`; C++20 enforced via `CMAKE_CXX_STANDARD 20` + `CMAKE_CXX_STANDARD_REQUIRED ON`; `#pragma once` header guards; `final` on concrete `Clock` implementations; digit separators in numeric literals (`1'000'000'000`).

**Dependency direction:** `main.cpp → common/time.hpp` and `main.cpp → common/logging.hpp`; both internal headers depend only on the C++ standard library (`<chrono>`, `<cstdint>`). Zero third-party dependencies; no package manager (no `vcpkg.json`, no `conanfile.txt`, no `FetchContent` block).

**Module boundaries:** the intended layout (`common/`, `feed/`, `risk/`, `kill/`, `strategy/`, `backtest/`, `replay/`, `control/`, `schema/`) exists only in the README; only `common/` is wired.

## Subsystems and components

### Clock (`src/common/time.hpp`, verified)

The only substantive subsystem in the project. Defines:

| Symbol | Type | Purpose |
|---|---|---|
| `tectra::common::Timestamp` | `using = std::int64_t` | Nanosecond-resolution time value |
| `Clock` | abstract class | `now()` returns `Timestamp`; `is_virtual()` returns `bool` |
| `RealClock` | `final : public Clock` | Wall-time via `std::chrono::steady_clock` |
| `VirtualClock` | `final : public Clock` | Simulated time with `advance(delta_ns)` and `set_time(absolute)` |

`RealClock::now()` casts `steady_clock::now().time_since_epoch()` to nanoseconds and returns `int64_t`. Uses `steady_clock` (monotonic) rather than `system_clock` so NTP adjustments cannot make time jump backwards. `int64_t` nanoseconds overflows after roughly 292 years.

`VirtualClock` holds a plain non-atomic `int64_t current_time_`. `advance(Δ)` adds delta nanoseconds; `set_time(t)` jumps absolutely (including backwards, intentionally — required for replay seeking). LifeOS explicitly flags that `VirtualClock` is **not thread-safe** and that `set_time` accepting earlier values is a documented sharp edge.

The discipline the abstraction enforces (per LifeOS): services should hold a `const Clock&` so they cannot mutate virtual time; only the replay harness or test driver holds a `VirtualClock*`. `main.cpp` itself currently violates the "no direct time access" convention by calling `std::this_thread::sleep_for` directly — acceptable in a demo, must be purged when real services land.

### Logging (`src/common/logging.hpp`, verified — scaffold only)

Six lines of content. A single `enum class LogSeverity { DEBUG = 0, INFO = 1, WARN = 2, ERROR = 3 };` in namespace `tectra::common`. Explicit integer values suggest intent for severity comparison and structured serialisation. There is no `Logger` type, no sink, no macros, no formatter, no JSON output, and no correlation-ID plumbing. The commit `2db9f7b` ("starting next feat.") modified this file by 3 additions and 2 deletions — most likely the addition of the explicit integer values — and was the last commit before dormancy.

### Build (`CMakeLists.txt`, verified)

13-line CMake file. CMake minimum 3.20; project `tectra` version 0.1.0; C++20 with `CMAKE_CXX_STANDARD_REQUIRED ON`; `add_compile_options(-Wall -Wextra -Wpedantic -Werror)` annotated `# Compiler warnings - be strict from day one`; single executable target `tectra` from `src/main.cpp`. No `enable_testing()`, no `add_test`, no sanitiser flags, no clang-tidy integration, no `.clang-format`, no `.github/workflows/`. `.gitignore` covers `build/`, `cmake-build-*/`, `.vscode/`, `.idea/`, editor swap files, and `.DS_Store`.

## Technologies and concepts demonstrated

### Languages

- **C++20** — the entire codebase. Enforced via CMake (`CMAKE_CXX_STANDARD 20`, required). Used in two header files (`time.hpp`, `logging.hpp`) and one source file (`main.cpp`), totalling roughly 80 lines of C++ across `src/common/` and `src/main.cpp`. Uses `#pragma once`, `final`, scoped enums (`enum class`), `using` aliases, and digit-separator literals.

### Frameworks and libraries

- No third-party libraries. The binary links only the C++ standard library. The headers `<chrono>`, `<cstdint>`, `<iostream>`, and `<thread>` are the entire external surface.

### Runtimes / engines / platforms

- No source evidence in LifeOS for a runtime, engine, or platform dependency beyond the C++ standard library and host OS. The README mentions Linux-specific paths (`io_uring`, `AF_XDP`) as future considerations; none are wired.

### Tools

- **CMake 3.20** — sole build tool. Out-of-source build pattern (no in-tree build artefacts; `.gitignore` covers `build/` and `cmake-build-*/`).
- **GCC/Clang compiler diagnostics** — used as a correctness gate via `-Wall -Wextra -Wpedantic -Werror`.

### Domains and concepts

- **Time virtualisation for deterministic replay** — the project's load-bearing concept. LifeOS articulates the reasoning explicitly: deterministic replay and backtesting require bit-identical re-execution under identical input; the largest source of non-determinism is wall-clock time; therefore time must be abstracted behind an interface and injected. Implemented as the `Clock` interface with `RealClock` for production and `VirtualClock` for replay/backtest/tests.
- **Monotonic vs system clock distinction** — `RealClock` uses `std::chrono::steady_clock` rather than `system_clock`, with the rationale that NTP adjustments must never make time go backwards in latency measurement.
- **Const-correctness as a determinism enforcement mechanism** — passing services `const Clock&` removes their ability to call `advance` or `set_time`, so only the replay harness can manipulate virtual time.
- **Build-discipline first** — `-Werror` plus `-Wpedantic` enforced from the first CMake commit; the LifeOS Decisions note frames this as a stance designed to make warning regressions impossible to accrete.
- **Aspirational (README only, no code):** lock-free shared-memory rings (SPSC/MPMC), contract-first schemas (FlatBuffers/Protobuf), append-only journals with Merkle roots, kill-switch fan-out with sub-millisecond propagation, FIX 4.4/5.0 / SBE / OUCH protocol adapters, ITCH decoding, L2 order books, pre-trade risk rules, plugin-based strategy framework, Prometheus metrics, structured JSON logs with correlation IDs. These are listed for completeness; none are implemented.

## Key technical decisions

LifeOS captures six load-bearing decisions in `Decisions.md`.

**D1 — Modern C++ (C++20) as the primary language.** Enforced by `CMAKE_CXX_STANDARD 20` with `STANDARD_REQUIRED ON`. Rationale recorded in LifeOS: industry realism (HFT codebases are C++; QuickFIX, Protobuf, Aeron, Disruptor ports are available); deliberate contrast with Nyquestro, which explores the same domain in safe Rust — two languages, two portfolio stories; genuine skill expansion beyond Caner's existing Rust focus. Rust, Zig, and modern C++ are listed as the alternatives considered.

**D2 — `-Werror` from day one.** `add_compile_options(-Wall -Wextra -Wpedantic -Werror)` set in the first CMake commit, with the source comment `# Compiler warnings - be strict from day one`. Rationale: every warning is cheap to fix at 2 KB and prohibitively boring to fix at 200 KB. Tradeoff accepted: builds will break when a new compiler version introduces new warnings.

**D3 — Virtual-clock-first ordering.** The first substantive code in the project (commit `a11fd04`) was the `Clock` abstraction — before any service, feed handler, or message type. Rationale: deterministic replay is impossible if any code reads wall-clock time directly, so the clock abstraction must predate every component that might want to read time. LifeOS frames this as the most defensible build-order decision in the project: starting with the constraint baked into the type system means no service written against `Clock&` can later accidentally break determinism.

**D4 — No dependencies in the scaffold phase.** Zero `find_package`, zero `FetchContent`, no `vcpkg.json` / `conanfile.txt`. Rationale: dependencies are cheap to add, expensive to remove; postponing the commitment to a specific Protobuf/FlatBuffers/QuickFIX version avoids premature coupling. Pushes the dependency-management decision to whoever builds the first real subsystem.

**D5 — README-first, then code.** The first three commits are README edits totalling ~730 lines before any source existed. LifeOS frames the README as a design document — equivalent to writing a spec before implementation — and explicitly flags the risk this creates: six months of dormancy since `2db9f7b` means the README is the public face of a project that barely exists, and a future session should either update the README to reflect status honestly or add a "Current Status" section marking built vs planned.

**D6 — Single-file `main.cpp` as demo, not service host.** `main.cpp` is a clock smoke test, not a process orchestrator. The eventual transition from "demo binary" to "multi-service platform" is an unresolved question: one binary with threads per subsystem, multiple binaries over shared memory (as the README implies), or a library that other binaries link.

**Decisions still owed** (LifeOS tracks each pinned to the README milestone that will force it): schema format (FlatBuffers vs Protobuf vs capnp), shared-memory ring library (roll-own vs Aeron vs Disruptor port), HTTP library (Boost.Beast vs cpp-httplib vs drogon), metrics library (prometheus-cpp vs custom), test framework (GoogleTest vs Catch2 vs doctest), dependency manager (vcpkg vs Conan vs FetchContent vs manual), logger library (spdlog async vs fmtlog vs custom), FIX library (QuickFIX vs custom).

## What is currently built

Honest implemented scope at HEAD `2db9f7b` (verified against source by LifeOS on 2026-04-24):

- **`src/common/time.hpp`** (43 lines): `Clock` abstract interface, `RealClock` (`final`, uses `std::chrono::steady_clock`), `VirtualClock` (`final`, holds non-atomic `int64_t`, supports `advance(delta_ns)` and `set_time(absolute)`), `Timestamp = std::int64_t` alias.
- **`src/common/logging.hpp`** (4 lines of content): `enum class LogSeverity { DEBUG = 0, INFO = 1, WARN = 2, ERROR = 3 };` in namespace `tectra::common`. No logger, no sink, no macros.
- **`src/main.cpp`** (40 lines): smoke test driving the two clocks; prints elapsed time across a 100 ms `sleep_for`, demonstrates `VirtualClock::advance` and `set_time`, returns 0.
- **`CMakeLists.txt`** (13 lines): CMake 3.20, C++20 required, `-Wall -Wextra -Wpedantic -Werror`, single executable target.
- **`.gitignore`** (9 lines): standard CMake / editor / macOS artefact exclusions.
- **`README.md`** (~29 KB): the project vision document — 14-milestone roadmap with Scope / Interfaces / Data Path / Control Plane / Storage / Observability / Testing / Exit Criteria per milestone.

**Scale markers from LifeOS:**

| Metric | Value | Source |
|---|---|---|
| Total files | 6 | `repo_stats.json` |
| Source files (`.cpp`/`.hpp`) | 4 | `repo_stats.json` |
| Source bytes | 2,255 | `repo_stats.json` |
| README bytes | 29,133 | `repo_stats.json` |
| Test files | 0 | `repo_stats.json` |
| Total commits | 6 | `fetch_commits.py` |
| Active development window | 2025-10-05 → 2025-10-10 (6 days) | commit history |
| README-to-source byte ratio | ~13:1 | derived |

## Current state

Status: **dormant** (LifeOS frontmatter `status: scaffold`; commit history shows no activity since 2025-10-10, six months prior to the 2026-04-24 vault verification). Last commit `2db9f7b` ("starting next feat.") modified `logging.hpp` by 3 additions / 2 deletions — most likely the addition of explicit integer values to `LogSeverity`. There is no `Work/` folder in the LifeOS source and no in-flight scope captured; the project went dormant at the exact moment the logging subsystem was about to be built.

## Gaps and known limitations

LifeOS `Gaps.md` is the most prominent note in the folder; it categorises gaps by tier.

**Tier 0:** the README contains nothing false that is contradicted by the code. The roadmap milestones are all marked with unchecked checkboxes, so the README does implicitly admit everything is unbuilt. No outright lies — just an enormous gap between scope described and scope built.

**Tier 1 (foundational, blocks all downstream work):** no schemas (no `.proto`, no `.fbs`, no `schema/` directory); no shared-memory rings; no logger (only the severity enum); no config loader; no test infrastructure (zero test files, no CI, no sanitisers wired); no HTTP/gRPC scaffold; no Prometheus `/metrics`; no CI/CD (no `.github/workflows`).

**Tier 2 (subsystem gaps, all README-promised, none started):** feed handler / ITCH decoder, normaliser, pre-trade risk engine, per-strategy position tracking, strategy execution framework, signal library (SMA, EMA, RSI, MACD, Bollinger, Z-score, correlation, cointegration), example strategies (MA crossover, mean reversion, pairs trading, market-making), backtesting engine, journals, deterministic replay, kill switch / circuit breaker, research toolkit, operator tooling (`tectractl`), performance / latency engineering, FIX adapter, SBE / FAST codecs, OUCH adapter, fault injection / chaos, packaging / compose / supervisor. LifeOS counts 19 subsystem-level gaps.

**Tier 4 (documentation / process):** no build instructions in the README; no "Current Status" section distinguishing vision from reality; no `LICENSE` file despite the repo being public (default no-licence means all rights reserved); commit messages are terse (`.`, `starting next feat.`).

**Tier 5 (conceptual decisions not made):** process model (threads in one binary vs separate binaries over shared memory vs library); target OS (`io_uring` and `AF_XDP` are Linux-only despite README claiming "portable"); single-host vs distributable; licence and commercial position; audience (portfolio vs real trading tool vs educational artefact).

**Subsystem-level gaps in the Clock subsystem** (LifeOS `Systems/Clock.md`): `VirtualClock` is not thread-safe (plain `int64_t`, not atomic) — multi-threaded backtests would race; no `WallClock` for calendar time (cannot represent venue session boundaries); no tests; `is_virtual()` runtime dispatch rather than compile-time templatisation; the "no direct time access" convention is not enforced anywhere — services could call `steady_clock::now()` directly without anyone noticing.

## Direction (in-flight, not wishlist)

No source evidence in LifeOS for active in-flight work. The LifeOS Roadmap note explicitly distinguishes the README's 14-milestone plan (intent, not delivery schedule) from actual momentum (six months of dormancy). The note frames four scope options for a hypothetical resumption — full fidelity (Option A, 3-4 years of evenings/weekends), foundations plus one vertical slice (Option B, 2-4 months), re-scope to learning lab (Option C, no fixed scope), or retire (Option D) — but explicitly does not commit to any of them. The closest thing to an in-flight item is the implicit "logging is the next subsystem" signal from commit `2db9f7b`'s "starting next feat." message, which was followed by silence.

## Demonstrated skills

What this project, in its current form, proves Caner can do:

- **Modern C++20 with strict discipline from day one.** Writes C++20 with `-Wall -Wextra -Wpedantic -Werror` enforced via CMake before any source file exists; uses `final`, `enum class`, `using` aliases, `#pragma once`, and digit separators consistently across a small but coherent codebase.
- **Designs a deterministic-replay primitive correctly on the first attempt.** Identifies that wall-clock access is the single largest source of non-determinism in a trading system, abstracts time behind a `Clock` interface, ships both `RealClock` (`steady_clock`-backed for monotonicity) and `VirtualClock` (`advance` + `set_time`, with the latter accepting earlier values to support replay seeking), and uses `const Clock&` versus `VirtualClock*` to make the determinism invariant enforceable through the type system rather than convention.
- **Distinguishes `steady_clock` from `system_clock` for the right reason.** The LifeOS Clock note explicitly cites NTP-adjustment-induced backwards jumps as the reason `RealClock` is `steady_clock`-backed — not boilerplate, applied reasoning.
- **Sequences scaffold work by load-bearing-ness, not by visibility.** Started with the clock abstraction (the primitive every downstream feature depends on) rather than the flashy choice (feed handler, fake exchange, demo strategy). LifeOS frames this as a defensible build-order decision.
- **Writes a coherent design-spec README before implementation.** The 29 KB README contains a 14-milestone plan with Scope / Interfaces / Data Path / Control Plane / Storage / Observability / Testing / Exit Criteria per milestone — usable as a design document even though execution has not followed.
- **Reads honestly about own work.** The LifeOS vault notes for this project are unusually candid about the gap between intent (README) and reality (code), explicitly framing the README-to-source 13:1 ratio as the headline finding and listing four resumption options including "retire" as legitimate. This kind of honest project self-assessment is itself a signal.

**Not demonstrated by current code:** any of the README-promised infrastructure (shared-memory rings, schemas, logging beyond an enum, risk engine, kill switch, replay, backtesting, FIX, OUCH, ITCH, strategies, Prometheus, journals). Any role requiring evidence of these specific systems must look elsewhere in Caner's portfolio (notably Nyquestro for safe-Rust matching-engine work).

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Tectra/_Overview.md | 89 | "> If a future Caner or future session reads the README and the vault notes and they disagree, the vault notes reflect code. The README has not been updated to mark what is built vs planned." |
| Projects/Tectra/Architecture.md | 164 | "- [[Tectra/Roadmap]] — the README's 14 milestones with velocity overlay" |
| Projects/Tectra/Decisions.md | 154 | "- [[Tectra/Roadmap]] — each open question is pinned to its README milestone" |
| Projects/Tectra/Evolution.md | 98 | "- [[Tectra/Systems/Clock]] — detailed history of the one substantive subsystem" |
| Projects/Tectra/Gaps.md | 149 | "- [[Tectra/Architecture]] — intended vs wired architecture side by side" |
| Projects/Tectra/Roadmap.md | 154 | "- [[Nyquestro/_Overview]] — parallel early-stage trading-infra project; cross-pollination possibility" |
| Projects/Tectra/Systems/Build.md | 119 | "- [[Tectra/Roadmap]] — Milestone 1 (Foundations) is where the rest of this gets built" |
| Projects/Tectra/Systems/Clock.md | 146 | "- [[Nyquestro/_Overview]] — Nyquestro uses event frames with embedded timestamps; a fantasy-integration would feed virtual time from Tectra's replay into Nyquestro's engine" |
| Projects/Tectra/Systems/Logging.md | 84 | "- [[Tectra/Roadmap]] — the README's Milestone 1 (Foundations) is where logging gets built" |
