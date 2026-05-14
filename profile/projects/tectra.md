---
name: Tectra
status: dormant
source_repo: https://github.com/Capataina/Tectra
lifeos_folder: Projects/Tectra
last_synced: 2026-05-13
sources_read: 9
---

# Tectra

## One-line summary

C++20 trading-infrastructure scaffold demonstrating disciplined virtual-clock-first design (time virtualisation primitive built before any service) under a `-Werror`/`-Wpedantic`/`-Wall`/`-Wextra` regime; substantive code is two header files and a smoke test, with an extensive README specifying the unbuilt 14-milestone platform around them.

## What it is

Tectra is Caner's self-directed exploration into building the invisible infrastructure of a trading firm in modern C++. The README states an ambition for a cohesive stack covering market-data ingest, pre-trade risk, kill switch, deterministic replay, a strategy execution framework, backtesting, and a signal research toolkit. The actual implemented scope as of HEAD `2db9f7b` is foundational plumbing: a `Clock` interface with `RealClock` and `VirtualClock` implementations, a four-value `LogSeverity` enum, a strict CMake C++20 build with `-Werror` from day one, and a `main.cpp` that exercises the two clocks. LifeOS frames Tectra as a "learning / portfolio project in its earliest phase" and explicitly designates the README a vision document rather than a delivery tracker. The project is structurally a 14-milestone plan with only the very first foundational primitives of Milestone 1 in code.

## Architecture

LifeOS distinguishes the **intended architecture** (from the README) from the **wired architecture** (from the source). Both are documented here; only the wired form is implemented.

**Wired architecture (actual code, per LifeOS Architecture.md):**

```
┌─────────────────────────────────────────────┐
│             src/main.cpp                    │
│  RealClock + VirtualClock smoke test        │
│  Prints clock outputs; exits.               │
└────────┬───────────────────────┬────────────┘
         │                       │
         ▼                       ▼
┌─────────────────────┐   ┌──────────────────────┐
│ src/common/time.hpp │   │ src/common/logging.hpp│
│  class Clock        │   │  enum LogSeverity     │
│  class RealClock    │   │  { DEBUG, INFO,       │
│  class VirtualClock │   │    WARN, ERROR }      │
│  using Timestamp =  │   │  (no macros / sinks)  │
│    std::int64_t     │   │                       │
└─────────────────────┘   └──────────────────────┘
```

Dependency graph: `main.cpp` depends on `common/time.hpp` plus three standard-library headers (`<chrono>`, `<iostream>`, `<thread>`); `common/time.hpp` depends on `<chrono>` and `<cstdint>`; `common/logging.hpp` is a leaf header with no includes. The CMake build links only the C++ standard library — no third-party dependencies, no `find_package`, no `FetchContent`, no `vcpkg.json` / `conanfile.txt`.

**Intended architecture (README-only, no code):** the README positions Tectra as a dual-plane platform — fast path (binary, zero-copy, lock-free shared-memory rings, cache-aligned bounded queues) carrying messages Feed Handler → Strategy Runtime → Risk Engine → Venue Gateway, with a slow/control plane (HTTP or gRPC, Prometheus metrics, structured logs, operator tooling), append-only journals with Merkle roots feeding a replay/backtest engine, and a kill switch spanning all services with sub-millisecond fan-out. **Every box in that intended diagram is aspirational; none of them have code.**

Namespace and code conventions (verified from the two header files): top-level namespace `tectra::common`; C++ standard C++20 with `CMAKE_CXX_STANDARD_REQUIRED ON`; strict compile options `-Wall -Wextra -Wpedantic -Werror`; `#pragma once` header guards; `final` on concrete `Clock` implementations; digit separators in literals (`1'000'000'000`).

## Subsystems and components

### Clock (`src/common/time.hpp`) — the only substantive subsystem

LifeOS Systems/Clock.md identifies this as "the only substantive subsystem in Tectra as of HEAD `2db9f7b`". Interface:

| Symbol | Type | Purpose |
|---|---|---|
| `tectra::common::Timestamp` | `using = std::int64_t` | Nanosecond-resolution time value |
| `Clock` | abstract class | Virtual interface — `now()` returns `Timestamp`, `is_virtual()` returns `bool` |
| `RealClock` | `final : public Clock` | Wall-time implementation backed by `std::chrono::steady_clock`; returns `int64_t` nanoseconds; no internal state, thread-safe |
| `VirtualClock` | `final : public Clock` | Simulated-time clock with `advance(delta_ns)` and `set_time(absolute_time)`; manually advanced, never advances on its own |

LifeOS notes the deliberate use of `steady_clock` (monotonic, never jumps backwards) rather than `system_clock` (subject to NTP adjustments). `int64_t` nanoseconds overflows after ~292 years, so epoch-drift is not a practical concern. `VirtualClock` is intentionally **not thread-safe** (`current_time_` is a plain `int64_t`, not atomic) and `set_time` can go backwards (required for replay seeking). LifeOS calls out that `main.cpp` itself currently violates the determinism contract by calling `std::this_thread::sleep_for` directly — acceptable in a demo, but must be purged once real services exist.

### Logging (`src/common/logging.hpp`) — placeholder, not a subsystem

Per LifeOS Systems/Logging.md, the entirety of the logging "system" is:

```cpp
namespace tectra::common {
enum class LogSeverity { DEBUG = 0, INFO = 1, WARN = 2, ERROR = 3 };
}
```

There is no logger type, no sink, no macros, no formatter, no correlation IDs, no JSON output, and no integration with any other part of the code. The README's "structured JSON logs with correlation IDs" claim is unbuilt. The explicit integer values suggest intent to use the enum as a severity comparator. The final commit on the repo (`2db9f7b`, "starting next feat.") modified this file with a 3-add/2-delete diff likely adding the explicit integer values, then the project went dormant — LifeOS Evolution.md notes that "the project went dormant at the exact moment the logging subsystem was about to be built."

### Build (`CMakeLists.txt` + `.gitignore`)

Per LifeOS Systems/Build.md: CMake 3.20 minimum, project version 0.1.0, language CXX, C++20 with `CMAKE_CXX_STANDARD_REQUIRED ON`, compile options `-Wall -Wextra -Wpedantic -Werror`, single executable `tectra` from `src/main.cpp`. **Zero external dependencies. Zero package manager. No sanitizers wired. No clang-tidy. No `enable_testing()`. No `add_test`. No `.clang-format`. No `.github/workflows/`.** `.gitignore` covers `build/`, `cmake-build-*/`, `.vscode/`, `.idea/`, `*.swp`, `*.swo`, `.DS_Store`.

The README contains no build instructions despite being 29,133 bytes; LifeOS flags this as a documentation gap.

## Technologies and concepts demonstrated

### Languages

- **C++20** — primary and only language. Verified by `CMAKE_CXX_STANDARD 20` with `CMAKE_CXX_STANDARD_REQUIRED ON`. Used in two header files (`src/common/time.hpp`, `src/common/logging.hpp`) and one source file (`src/main.cpp`). Conventions present: `enum class`, `final` on concrete polymorphic types, `#pragma once`, digit separators in literals, `using` aliases for type clarity.

### Frameworks and libraries

- **C++ standard library only** — `<chrono>` for time types and steady_clock, `<cstdint>` for fixed-width integers, `<iostream>` for the smoke-test output, `<thread>` for `std::this_thread::sleep_for`. No third-party dependencies are wired into CMake.

### Runtimes / engines / platforms

- No source evidence in LifeOS for any runtime or platform beyond the standard library — the binary is a single-threaded executable that prints clock diagnostics and exits.

### Tools

- **CMake 3.20** — sole build tool. Single `CMakeLists.txt`, 13 lines, no subdirectories, no install rules, no test targets.
- **Compiler diagnostic posture** — `-Wall -Wextra -Wpedantic -Werror` enforced globally from the first source commit. LifeOS Decisions.md captures this as a deliberate "be strict from day one" stance, not a default.

### Domains and concepts

- **Time virtualisation** — `Clock` interface plus `RealClock` (wall time) and `VirtualClock` (manually advanced simulated time). LifeOS Systems/Clock.md frames this as the single most load-bearing primitive in the project: deterministic replay and backtesting both require running the same logic twice with the same inputs and getting bit-identical outputs, and wall-clock time is the largest source of non-determinism. The convention is dependency-injection: production code receives `Clock&`, tests/replay receive a `VirtualClock` driver. Concrete-`final` types prevent virtual-method overhead chains; `is_virtual()` is a runtime tag.
- **Trading-system architecture intent** — Caner has *designed* (not built) the dual-plane fast/slow architecture pattern in the README: fast path on lock-free shared-memory rings carrying binary messages, slow path on HTTP/gRPC with Prometheus and structured logs. Demonstrated by the README design only, not by code; do not credit this as a built capability.
- **Determinism as an explicit design constraint** — the build-order choice (clock before services, before everything else) is grounded in the constraint that any code calling `steady_clock::now()` directly is non-deterministic and unreplayable.
- **Disciplined C++ baseline** — `-Werror` plus `-Wpedantic` from line one of CMakeLists.txt, `final` on concrete subclasses, no implicit `int`/narrowing conversions tolerated, no third-party dependencies until forced.

## Key technical decisions

LifeOS Decisions.md captures six load-bearing decisions made so far.

**D1 — Modern C++ (C++20) as the primary language.** Rust, Zig, and modern C++ were the considered alternatives. C++ was chosen for industry realism (production trading codebases are C++), for deliberate contrast with Nyquestro (Rust safety-first trading infrastructure), and as a genuine skill expansion from Caner's existing Rust focus. Would change on a memory-safety bug Rust would have caught, or a decision to make Tectra cooperate with Nyquestro rather than mirror it.

**D2 — `-Werror` from day one.** Enforced globally via `add_compile_options(-Wall -Wextra -Wpedantic -Werror)` with the source comment "Compiler warnings - be strict from day one". Tradeoff accepted: builds will break when GCC/Clang version changes introduce new warnings. Would change if a third-party header forces unsuppressible warnings (then `-Werror` would scope to Tectra's own targets only).

**D3 — Virtual-clock-first ordering.** The first substantive code written in the project (commit `a11fd04`) was the `Clock` / `RealClock` / `VirtualClock` abstraction, before any service, feed handler, or message type. Alternatives rejected: starting with a feed handler (flashy but would need rewriting once replay was added), skipping virtualisation until replay was built (defensible but forfeits the disciplined-build-order story). Rationale: building the constraint into the type system from the start means no service written against `Clock&` will ever accidentally break determinism. Would change if virtual dispatch through `Clock&` proved measurably costly at µs latency targets — mitigation would templatise on `Clock` type in release builds.

**D4 — No dependencies in the scaffold phase.** Zero `find_package`, zero `FetchContent`, zero `vcpkg.json` / `conanfile.txt`. Keeps the project portable and avoids premature coupling to specific Protobuf / FlatBuffers / QuickFIX versions. Would change on the first subsystem that needs a schema library (likely Milestone 1's Foundations work).

**D5 — README-first, then code.** First five commits are README-only (488 lines added in one commit alone) before any source file existed. Treats the README as a design document / spec. LifeOS flags the now-inevitable consequence: the README has become stale relative to code since `2db9f7b` and presents an ambitious unified system while the repo contains a clock demo — a documentation debt item.

**D6 — Single-file `main.cpp` as demo, not service host.** `main.cpp` is a clock smoke test, not a process orchestrator. Consistent with having no services to orchestrate, but means the eventual transition from "demo binary" to "multi-service platform" is an unresolved design question (one binary with threads, multiple binaries on shared memory, library linked elsewhere — README implies multi-process but `main.cpp` is one binary).

**Decisions still owed (LifeOS Decisions.md "Decisions still owed" table):** schema format (FlatBuffers vs Protobuf vs capnp), shared-memory ring library (roll-own vs Aeron vs Disruptor port), HTTP library (Boost.Beast / cpp-httplib / drogon / custom), metrics library (prometheus-cpp vs roll-own), test framework (GoogleTest / Catch2 / doctest), dependency manager (vcpkg / Conan / FetchContent), logger library (spdlog async / fmtlog / custom), FIX library (QuickFIX vs custom). LifeOS Evolution.md hypothesises that decision paralysis across these four-plus simultaneously load-bearing choices is the most likely cause of the project's dormancy.

## What is currently built

LifeOS Overview.md provides a code-evidence table of the README's claims versus actual code. The actually-implemented scope is:

- **Build system**: CMake 3.20 + C++20 + `-Wall -Wextra -Wpedantic -Werror`. No sanitizers wired, no clang-tidy integration, no tests.
- **Clock domain**: `RealClock` plus `VirtualClock` implementing a shared `Clock` interface, nanosecond-resolution `Timestamp = std::int64_t`. Single file `src/common/time.hpp`, 43 lines.
- **Logging**: a four-value enum `LogSeverity { DEBUG = 0, INFO = 1, WARN = 2, ERROR = 3 }`. No macros, no sinks, no formatter. Single file `src/common/logging.hpp`, ~6 lines of content.
- **Entry point**: `main.cpp` that prints "Tectra v0.1.0 - Clock Abstraction Test" and exercises both clocks (measures a 100 ms `sleep_for` interval with `RealClock`, advances `VirtualClock` by 1 s and then sets it to 5 s, prints results). Total runnable behaviour of the binary.

**Not built** (every item is explicitly verified absent in LifeOS Overview.md and Gaps.md): feed handler / ITCH decoder / L2 book builder, normaliser / derived-metric calculators, pre-trade risk engine, per-strategy position tracking, kill switch, deterministic replay, journals with Merkle roots, strategy execution framework, signal library, example strategies, backtesting engine, fill simulator, TCA, research toolkit, operator tooling (`tectractl`), FIX 4.4/5.0 adapter, SBE codec, OUCH adapter, fault injection / chaos, packaging / deployment, schemas (no `.proto`, no `.fbs`), shared-memory rings, config loader, HTTP/gRPC scaffold, `/metrics` endpoint, CI pipeline.

**Scale markers (LifeOS Overview.md):** 6 total files in the repo, 4 source files (`.cpp`/`.hpp`), 2,255 source bytes, 29,133 README bytes, 0 test files, 6 total commits, README-to-source byte ratio ~13:1. First commit 2025-10-05; last commit 2025-10-10 (`2db9f7b`, "starting next feat."); 6 days of active development; ~6 months dormant as of LifeOS extraction date 2026-04-24.

## Current state

**Status: dormant.** LifeOS Overview.md frontmatter sets `status: scaffold`; LifeOS Evolution.md notes ~6 months of zero commits since `2db9f7b` (2025-10-10) with no issues opened, no branches pushed, no external contributions, while Caner shipped significant work on Cernio, Flat Browser, Nyquestro, NeuroDrive, and LifeOS itself. The frontmatter `status: dormant` in this file reflects that observed behaviour. The last commit message is "starting next feat." — LifeOS characterises this as "the last heartbeat", a commit that literally announces the start of the next piece of work that was then never started. No work is currently in flight in LifeOS `Work/` (no such folder exists for Tectra).

## Gaps and known limitations

LifeOS Gaps.md categorises gaps across five tiers; everything below is sourced from there.

**Tier 1 — foundational gaps that block all downstream work**: no schemas (no `.proto`, no `.fbs`, no `schema/`), no shared-memory rings, no logger (only a severity enum), no config loader, no test infrastructure (zero tests, no CI, no sanitizer wiring), no HTTP/gRPC scaffold, no Prometheus `/metrics`, no CI/CD (no `.github/workflows`). Every item gates Milestones 2-14.

**Tier 2 — subsystem gaps**: feed handler / ITCH decoder, normaliser with derived-metric calculators (VWAP, spread, imbalance), pre-trade risk engine, per-strategy position tracking, strategy execution framework with plugin loader, signal library (SMA, EMA, RSI, MACD, Bollinger, Z-score, correlation, cointegration), example strategies (MA crossover, mean reversion, pairs trading, market-making), backtesting engine with fill simulator and TCA, append-only journals, deterministic replay harness with segment manifests and Merkle roots, kill switch with operator TUI, research toolkit (correlation scanner, cointegration tester, regime detector), `tectractl` CLI/TUI, performance/latency engineering harnesses, FIX adapter, SBE / FAST codecs, OUCH adapter, chaos / fault injection, packaging / compose / supervisor. **Count: 19 subsystem-level gaps, each a multi-week piece of work.**

**Tier 3 — unanswered design questions**: schema format (FlatBuffers vs Protobuf vs capnp), shared-memory ring library, HTTP library, metrics library, test framework, dependency manager, logger library, FIX library.

**Tier 4 — documentation and process gaps**: no build instructions in the 29KB README, no "Current Status" section distinguishing built from planned, no LICENSE file (repo is public but unlicensed — default "all rights reserved" technically restricts third-party use), terse commit messages ("`.`", "`starting next feat.`") making history archaeology harder than needed.

**Tier 5 — non-obvious conceptual gaps**: process model is undecided (services as threads in one binary vs separate binaries on shared memory vs a library); target OS is undecided (`io_uring` / `AF_XDP` are Linux-specific, but README claims "portable, cross-platform"); single-host-only vs distributable is ambiguous; licence and commercial position not declared; audience (recruiters, real trading tool, educational artefact) not stated.

**Subsystem-internal gaps within what is built:** the `VirtualClock` is not thread-safe (`current_time_` is plain `int64_t`, not atomic); no separate `WallClock` for calendar time (only `steady_clock`-backed `RealClock`); no tests on the clock subsystem; `is_virtual()` runtime tag instead of compile-time type discrimination; no enforcement (linter / code-review check) that services do not call `steady_clock::now()` directly.

## Direction (in-flight, not wishlist)

**Nothing is in flight.** LifeOS Evolution.md is explicit: "And that's where it stops. No commit since." The last commit `2db9f7b` (2025-10-10) modified `logging.hpp` with a 3-add/2-delete diff and the message "starting next feat." — six months of silence followed. There is no active branch, no open issue, no work-in-progress.

LifeOS Roadmap.md enumerates four mutually-exclusive scope options for a hypothetical resumption (Option A: build all 14 milestones as written — estimated 3-4 years of evenings/weekends; Option B: build Milestone 1 plus one minimal vertical slice — 2-4 months; Option C: re-scope to a learning lab with no production-style framing; Option D: retire), but no decision is captured in LifeOS as having been made. These are options *for* a future decision, not direction currently being executed. The Roadmap document treats the README's 14-milestone plan as "intent, not a delivery schedule".

## Demonstrated skills

What Tectra in its current implemented form demonstrates (only counting code-level evidence per LifeOS, not README aspiration):

- **Modern C++20 fluency at the language-feature level**: `enum class` with explicit integer values, `final` on concrete polymorphic types, `using` aliases, `#pragma once`, digit separators in literals, `override` discipline.
- **CMake-driven build discipline**: setting up a strict toolchain regime (`-Wall -Wextra -Wpedantic -Werror`) from line one of `CMakeLists.txt`; deliberate stance to refuse code the compiler suspects, even in a scaffold.
- **Time virtualisation as a foundational primitive**: design and implementation of a `Clock` abstraction with `RealClock` (`std::chrono::steady_clock` monotonic, nanosecond-resolution `int64_t` epoch) and `VirtualClock` (manually-advanced simulated time supporting both `advance(delta)` and `set_time(absolute)`), with the explicit goal of enabling deterministic replay. Demonstrates awareness that wall-clock time is the largest source of non-determinism in event-driven systems and that dependency-injection of time is the fix.
- **Architectural reasoning about determinism**: the build-order decision to write the clock before any service is itself an engineering choice that demonstrates seniority. LifeOS Decisions.md captures the alternatives (start with a feed handler, defer virtualisation) and the reasoning behind rejecting them.
- **Awareness of monotonic-vs-system clock distinction**: deliberate choice of `steady_clock` over `system_clock` (NTP can make `system_clock` go backwards), with explicit acknowledgement that a future `WallClock` will be needed for calendar-time concerns like venue session boundaries.
- **Trading-infrastructure domain literacy at the design-document level**: the README articulates a coherent multi-component trading platform vision (feed handler, risk engine, kill switch, deterministic replay, strategy execution, backtesting, FIX gateway, observability via Prometheus, structured logging). This is design literacy, not implementation experience — the corresponding code does not exist.
- **Honest self-assessment of scaffold versus product**: the LifeOS notes themselves are a demonstrated skill — Caner has documented his own project's gaps with structural rigour (5-tier gap inventory, code-evidence-versus-README-claim tables, explicit identification of the "starting next feat." commit as a "last heartbeat", explicit naming of the README-to-source 13:1 ratio as a headline finding). This degree of honest project diagnosis is itself a signal worth more than the code it documents.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Tectra/_Overview.md | 88 | "If a future Caner or future session reads the README and the vault notes and they disagree, the vault notes reflect code. The README has not been updated to mark what is built vs planned." |
| Projects/Tectra/Architecture.md | 161 | "- [[Tectra/Roadmap]] — the README's 14 milestones with velocity overlay" |
| Projects/Tectra/Decisions.md | 174 | "- [[Tectra/Roadmap]] — each open question is pinned to its README milestone" |
| Projects/Tectra/Evolution.md | 119 | "- [[Tectra/Systems/Clock]] — detailed history of the one substantive subsystem" |
| Projects/Tectra/Gaps.md | 161 | "- [[Tectra/Architecture]] — intended vs wired architecture side by side" |
| Projects/Tectra/Roadmap.md | 156 | "- [[Nyquestro/_Overview]] — parallel early-stage trading-infra project; cross-pollination possibility" |
| Projects/Tectra/Systems/Build.md | 113 | "- [[Tectra/Roadmap]] — Milestone 1 (Foundations) is where the rest of this gets built" |
| Projects/Tectra/Systems/Clock.md | 156 | "- [[Nyquestro/_Overview]] — Nyquestro uses event frames with embedded timestamps; a fantasy-integration would feed virtual time from Tectra's replay into Nyquestro's engine" |
| Projects/Tectra/Systems/Logging.md | 90 | "- [[Tectra/Roadmap]] — the README's Milestone 1 (Foundations) is where logging gets built" |
