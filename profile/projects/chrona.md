---
name: Chrona
status: paused
source_repo: https://github.com/Capataina/Chrona
lifeos_folder: Projects/Chrona
last_synced: 2026-04-26
sources_read: 9
---

# Chrona

## One-line summary

A modern-C++20 personal learning project rebuilding the inner engine of Git from first principles — currently a CMake + CLI + error-model + repo-discovery scaffold, with the content-addressed VCS core scoped, planned, and unbuilt.

## What it is

Chrona is a deliberately-scoped C++20 systems learning project aimed at implementing the engine half of Git (object storage, trees, commits, refs, index, diff) from first principles, drawing a sharp line between *the VCS engine* and *the hosting platforms* (GitHub, GitLab, Codeberg) layered on top. The project is local-first by construction — no networking, no server component, no auth — and treats history objects as write-once content-addressed blobs whose integrity is detectable by design. It is structured as a 9-milestone roadmap (Milestone 0 Foundations through Milestone 8 Storage/Performance, plus a Milestone 9 documentation track), each gated by demo-able output and tests. What the project *currently demonstrates* is a working CMake build, a Catch2 test harness, a manual-`argc/argv` CLI parser, a value-style error model, and a parent-walk-up repo discovery function — the VCS engine itself is not started. The distinction between design ambition (a complete Git-style engine) and implemented scope (a 5KB scaffold) is explicit in LifeOS and load-bearing for any honest reading of the project.

## Architecture

Chrona is a single-binary C++20 project with a folder-per-subsystem layout under `src/`, all declarations sitting in one flat `namespace chrona {}`. Two CMake executables share most source files:

- **`chrona`** — main executable, built from `src/main.cpp` plus `errors/error.cpp`, `repo/repo.cpp`, `cli/cli.cpp`. Zero runtime dependencies (stdlib only).
- **`chrona_tests`** — Catch2 test executable built from the same three source files plus `tests/test_cli.cpp`. Catch2 v3.5.0 is fetched at configure time via `FetchContent`; only `test_cli.cpp` is in the target — `test_errors.cpp` and `test_repo.cpp` are commented out.

```
chrona/
├── CMakeLists.txt              C++20, FetchContent Catch2 v3.5.0, /W4 or -Wall -Wextra -Wpedantic
├── README.md                   Aspirational pitch + 9-milestone roadmap
├── plans/
│   ├── README.md               Plan index (NNN_topic.md convention)
│   ├── ARCHITECTURE.md         Repo-internal architecture reference
│   └── 000_foundations.md      Milestone 0 plan (in_progress, 6/9 steps checked)
├── src/
│   ├── main.cpp                Entry point — parse args, switch on action
│   ├── cli/                    parse_args(), print_usage(), Command/ParseAction enums
│   ├── errors/                 ExitCode/ErrorCode enums, Error struct, create/print helpers
│   └── repo/                   find_repo() walk-up parent_path search
└── tests/                      11 Catch2 TEST_CASEs running; 11 more exist but commented out
```

Compile-time dependency direction: `main.cpp` depends on `cli/` and `errors/`. `repo/` is compiled into both targets but is **dead code in practice** — `main.cpp` never calls `find_repo()`, the only caller is `tests/test_repo.cpp` which is commented out of the build. The `chrona_tests` target uses `target_include_directories(... PRIVATE src)`; the equivalent is missing on the `chrona` target (compiles today by implicit source-directory include resolution, fragile to layout changes). Both executables compile each shared source file independently — there is no `add_library(chrona_core ...)` target yet (intended refactor when the codebase grows past ~10 source files).

Runtime data flow today is one screen of code: `argv[]` → `parse_args()` → `ParseResult { action, command?, args, error_message? }` → switch on `action` (`ShowHelp` → `print_usage`; `Error` → `print_error` + `return 1`; `RunCommand` → switch on `command` → `Command::Init` → `// TODO: Implement init command` → `return 0`).

## Subsystems and components

### CLI (`src/cli/`)

The entire user-facing surface. `parse_args(int argc, char* argv[])` returns a `ParseResult` tagged-union of `ParseAction` (`RunCommand` / `ShowHelp` / `Error`) plus an optional `Command` enum (only `Command::Init` registered today), an args vector (declared but never populated), and an optional error message. Manual `argc/argv` string-compare parsing — CLI11 and cxxopts were considered and explicitly rejected in the Foundations plan. Parse rules are exhaustively tested by 11 Catch2 TEST_CASEs in `tests/test_cli.cpp`. A hard `argc > 2` guard at the top of `parse_args` rejects any future multi-arg command (e.g. Milestone 1's planned `chrona hash-object <file>`); a test comment explicitly acknowledges this as future-work debt. Help text advertises a non-existent `help` command and a non-owned `chrona.com` URL.

### Errors (`src/errors/`)

Centralised error model. `ExitCode` (`Success`=0, `GeneralError`=1, `UsageError`=2) is the process-level signal; `ErrorCode` (`NotFound`, `AlreadyExists`, `InvalidArgument`, `IOError`, `UnknownError`) is the semantic category. `Error` struct pairs them with a message. Two `create_error` overloads return `std::optional<Error>` (the optional is always engaged — a known over-engineered signature); `InvalidArgument` auto-maps to `UsageError`, everything else to `GeneralError`. Output via `print_error(const Error&, std::ostream& = std::cerr)` formats as `"Error: <message>\n"` (no code, no source location); `exit_with_error` exists but takes a separate `int exit_code` parameter that ignores the struct's own `exit_code` (latent defect — never called yet). The two TEST_CASEs in `tests/test_errors.cpp` are not compiled into the test target.

### Repo Discovery (`src/repo/`)

`find_repo(const std::filesystem::path& start_path) -> std::optional<std::filesystem::path>`. Walks up `parent_path()` from a start path looking for a `.chrona/` sentinel directory; terminates when `parent == current` or `parent.empty()`. Same algorithm Git uses with `.git/`. Two latent bugs: (1) uses `std::filesystem::exists` rather than `is_directory`, so a regular file named `.chrona` would be accepted as a repo root; (2) the function is not wired to any command and the `init` command that would create the first `.chrona/` is a TODO stub — `find_repo` is currently looking for a directory no command in the project can create. The 273-line `tests/test_repo.cpp` is mostly debug scaffolding (8 of 9 SECTIONs end in `REQUIRE(true)`) plus 2 real assertions for `/` and `C:\\` root cases; the file is commented out of the build so even those do not run.

### Build and Test (`CMakeLists.txt`, `tests/`)

CMake 3.20 minimum, C++20 via per-target `target_compile_features`, Catch2 v3.5.0 fetched at configure time. `enable_testing()` + a single-line `add_test(NAME chrona_tests COMMAND chrona_tests)` — no `catch_discover_tests` per-test breakdown. `/W4` on MSVC, `-Wall -Wextra -Wpedantic` on GCC/Clang, no `-Werror`, no suppressions. Of 22 written tests across three files, 11 actually run (only `test_cli.cpp` is in the target). No CI configured. No `.clang-format`, `.clang-tidy`, or `.editorconfig`. `.gitignore` covers Windows/Linux/macOS object and binary patterns plus IDE caches; `claude.md` is ignored, suggesting Caner has had Claude-specific in-repo notes at some point.

### Plans Workflow (`plans/`)

Unusual for a ~2KLOC C++ project: a formal `plans/` directory with an `NNN_<short_topic>.md` filename convention, a status enum (`planned`/`in_progress`/`blocked`/`complete`), and a per-plan checklist schema. The single live plan, `plans/000_foundations.md`, establishes an 11-section template every future plan is expected to follow (Goal/Scope, Context/Justification, Rejected alternatives, Assumptions, Interfaces and contracts, Impacted areas, Incremental implementation, Testing/validation, Risks/failure modes, Exit criteria, Future considerations). The `plans/` discipline mirrors Caner's broader preference for externalised working memory at the LifeOS scale, applied to a single repo. Plan-as-SSOT philosophy: the plan is the falsifiable statement, the README is the elevator pitch.

## Technologies and concepts demonstrated

### Languages

- **C++20** — primary language. Uses `std::filesystem` (path traversal, parent_path, exists), `std::optional` (return values, optional command, optional error message), `enum class` with explicit underlying values, `std::string`, `std::vector`, `std::ostream` defaulting. Sources tagged `cxx_std_20` per-target. ~5KB of source across 7 files in `src/`; ~16KB of tests across 3 files in `tests/`. First C++ project in Caner's vault — small surface area, idiomatic but in-progress.

### Frameworks and libraries

- **Catch2 v3.5.0** — test framework, fetched at configure time via CMake `FetchContent`. Test files use `TEST_CASE` and `SECTION` macros, captured to `std::ostringstream` for output assertions. The `Catch2::Catch2WithMain` target provides the test runner's `main()`. Configure-time network dependency.

### Runtimes / engines / platforms

- **CMake 3.20+** — build configuration. Two-executable layout with shared sources compiled per-target. `target_compile_features`, `target_compile_options`, `target_include_directories`, `enable_testing` + `add_test`. Cross-platform warning flags (`/W4` on MSVC, `-Wall -Wextra -Wpedantic` on GCC/Clang).
- **CTest** — registered but coarse (single-line `add_test` with no per-test discovery).

### Tools

- **Catch2 FetchContent** — the only build-time dependency download.
- No formatter (`.clang-format`), no linter (`.clang-tidy`), no editor config (`.editorconfig`), no CI workflow — explicit gaps captured in LifeOS.

### Domains and concepts

- **Version control internals (planned, not built)** — the project's stated subject. Content-addressed object storage, commit DAGs, snapshot trees, derived diffs, refs/HEAD, staging index. None implemented; design principles locked into LifeOS Decisions (immutable-by-construction core, derived diffs, deterministic byte encoding for object identity).
- **Repo-discovery walk-up algorithm** — implemented. Sentinel-directory walk (`.chrona/`) up `parent_path()` to filesystem root, terminating on `parent == current` idempotence — the canonical Git approach.
- **Value-style error handling** — `Error` struct + `std::optional<Error>` returns instead of exceptions. Two-layer split between process exit codes and semantic error categories. Exceptions explicitly rejected (RTTI surface, exception-safety auditing, harder pipeline reasoning).
- **Manual CLI parsing** — tagged-union result via `ParseAction` enum + optional command + optional error message. CLI11 and cxxopts considered and rejected in favour of stdlib-first discipline.
- **Externalised plan-as-SSOT methodology** — `plans/NNN_topic.md` schema with rejected alternatives, exit criteria, and per-step checklists. Mirrors LifeOS-style externalised working memory at single-repo scale.
- **Milestone-driven scope control** — 9 explicitly-numbered milestones with named exit criteria, plus an explicit "out of scope" list (networking, history rewriting, packfiles/delta/GC, signing, hooks, submodules, LFS, sparse checkout) to bound the project against "endless Git parity".

## Key technical decisions

Drawn from `Decisions.md`. Twelve decisions captured; the headline ones:

| # | Decision | Status | Why |
|---|---|---|---|
| D1 | Local-first; no networking, no server | Locked | Project's value is reimplementing the *engine*, not the hosting platform. Networking would turn it into a half-built Git competitor instead of a learning tool. |
| D2 | Content-addressed, immutable-by-construction core | Locked (not built) | Content addressing is what makes Git's guarantees strong; replacing it would defeat the learning purpose. |
| D3 | Determinism as a first-class property | Locked (pre-impl) | Identical input → identical object id across runs and platforms; without it, content addressing is meaningless. Implies canonical byte encoding, UTC timestamps, no reliance on filesystem traversal order. |
| D4 | Diffs are derived, not stored | Locked | Snapshots are primary truth; diffs/status/log computed on demand. Cached diffs would be optional perf, never replacement. |
| D5 | Milestone-driven, correctness-first | Process | 9 milestones with demo-able output and tests. Exit criteria gate completion. Two milestones (7 merge, 8 storage/perf) flagged as optional "deep dive" / "systems flex" to bound scope. |
| D6 | Stdlib-first dependency policy | Soft | Prefer C++ standard library; one lightweight header-only library allowed if a specific need becomes unwieldy. No Boost, no heavy framework. CLI11/cxxopts explicitly rejected. |
| D7 | Manual `argc/argv` argument parsing | Provisional | Subcase of D6. At one subcommand, manual is simpler than learning a library API. Will need revisiting when commands take args. |
| D8 | Value errors via struct + optional, not exceptions | Soft | Avoids RTTI surface and exception-safety auditing; values are pattern-matchable. Could move to `std::expected` in C++23 later. |
| D9 | Folder-per-subsystem with flat namespace | Soft | `src/cli/`, `src/errors/`, `src/repo/` — folder separation for findability; one `namespace chrona {}` for surface simplicity. Nestable if collisions arise. |
| D10 | Catch2 v3.5.0 over GoogleTest | Soft | Header-light, fits small tests, pulls cleanly through `FetchContent`. Cost is configure-time network dependency. |
| D11 | `.chrona/` directory as repository sentinel | Locked | Same Git pattern (`.git/`) for the same reason: local-first, self-contained, unambiguous. Env var and central index alternatives violate D1. |
| D12 | Explicit scope exclusions up-front | Process | README §"Explicitly out of scope" names what Chrona will *not* build (networking, packfiles, signing, hooks, submodules, LFS, sparse checkout) to prevent unbounded scope creep on a learning project. Escape clause: optional advanced modules later if Chrona earns it. |

## What is currently built

Counted from LifeOS Overview's verified evidence:

- **CMake build** — working. C++20, FetchContent Catch2 v3.5.0, two targets.
- **CLI parser** — working. `argc > 2` rejects, `--help` and `init` recognised literally, unknown commands error to stderr with exit 1.
- **`chrona init`** — **stub**. CLI dispatches to `Command::Init` but `main.cpp` only contains `// TODO: Implement init command` and `return 0`.
- **Repo discovery** — implemented but unused. `find_repo()` walks parent paths looking for `.chrona/`, stops at filesystem root. Not called from any command.
- **Error model** — implemented. `ExitCode` / `ErrorCode` enums, `Error` struct, `create_error` / `print_error` / `exit_with_error` helpers (with the latent defects called out in the Errors system note).
- **Object store, trees, commits, refs, index, diff** — **none started**. No hashing, no `.chrona/objects/`, no I/O code for VCS objects.
- **Tests wired to CTest** — partial. 11 of 22 tests run; `test_errors.cpp` and `test_repo.cpp` are commented out of the `chrona_tests` target.

LifeOS Overview frames the project as "at ~2% of its stated scope" — Milestone 0 (Foundations) is in progress with 6/9 steps complete in the live plan checklist; Milestones 1-8 are zero source code. Total: 7 source files / ~5KB in `src/`, 3 test files / ~16KB in `tests/`, 3 plan files / ~12KB in `plans/`, README ~7.5KB. 7 commits total, all in the week of 2025-12-22 to 2025-12-27.

## Current state

**Status: paused.** LifeOS Overview classifies the project as "foundational — Milestone 0 partially complete, VCS core unbuilt". Repository created 2025-12-23, last commit `652fb7f` on 2025-12-27, dormant for ~16 weeks as of 2026-04-24. The `plans/000_foundations.md` plan is marked `in_progress` with timestamp 2025-12-26 and has not advanced — LifeOS notes that `blocked` would be a more honest status if the project is parked. Per LifeOS Roadmap, Chrona is **not listed as an active project** in the vault's Projects index; it is a candidate project — scoped, planned, scaffolded, paused — sitting alongside active threads (Cernio, Flat Browser, Aurix, NeuroDrive, Nyquestro, LifeOS, Claude Config) that have shipped artefacts and recent commits.

## Gaps and known limitations

LifeOS Gaps catalogues 18 items across CRITICAL/HIGH/MEDIUM/LOW. The career-relevant headlines:

**Structural gaps (CRITICAL):**

- `chrona init` is a TODO stub. The one command the CLI recognises does nothing; no `.chrona/` directory ever gets created, blocking every downstream milestone.
- The entire VCS core is unimplemented — no object store, no hashing, no trees, no commits, no refs, no index, no diff. Any external summary calling Chrona "a content-addressed VCS in C++" would be wrong; the honest summary is "a CMake/CLI/error-model scaffold in C++, planned as a future VCS".

**Correctness gaps in code that does exist (HIGH):**

- 2 of 3 test files are commented out of the build (11 of 22 tests run). Error system and repo discovery have zero executing assertions.
- `exit_with_error` ignores the struct's own `exit_code` field, taking a separate `int` parameter defaulting to `GeneralError`. Latent — never called yet.
- `main.cpp` returns hardcoded `1` for parse errors instead of propagating `UsageError = 2` through the `Error` struct's exit code, contradicting the plan's interface contract.
- `find_repo` uses `std::filesystem::exists` rather than `is_directory`, accepting a regular file named `.chrona` as a false-positive repo root.

**Design smells (MEDIUM):**

- `parse_args` rejects any multi-argument command via `argc > 2` — Milestone 1's `chrona hash-object <file>` will require this guard to be relaxed.
- `ParseResult.args` is declared but never populated.
- `create_error` returns a `std::optional<Error>` that is never `nullopt` — over-engineered signature.
- No `target_include_directories` on the main executable target (compiles by implicit resolution, fragile).
- No shared library target — sources compiled twice across the two executables.
- `add_test` is single-line; no per-test CTest breakdown via `catch_discover_tests`.

**Hygiene (LOW):** Help text advertises `chrona.com` (not owned) and a `help` command (does not exist); duplicate `*.lib` line in `.gitignore`; no `.clang-format`, `.clang-tidy`, `.editorconfig`; no CI configured; dependency policy not documented beyond a one-line note.

## Direction (in-flight, not wishlist)

Per LifeOS, Chrona is dormant rather than in flight — the only live plan (`plans/000_foundations.md`) has been at 6/9 checked steps since 2025-12-26 with no commits since 2025-12-27. There is no actively-progressing work to report. LifeOS Roadmap describes the *resumption sequence* if Caner restarts the project (close `init` stub, re-enable test files, open `001_object_store.md`, pick a hash function, build `chrona hash-object` / `chrona cat-object`), but that is a hypothetical opening sequence, not in-flight execution.

Aspirational (in LifeOS Roadmap, deliberately excluded from this section): Milestones 1-8 (object store, trees, commits, index, diff, branching, optional merge, optional storage/performance) and Milestone 9 (documentation as a feature — short "How Git works" written from Chrona's implementation, design notes per milestone, minimal reproducible demos). LifeOS flags Milestone 9 as the project's "real payoff" since the stated purpose is learning and explanation.

## Demonstrated skills

What Chrona, in its current scaffold state, actually proves Caner can do:

- **Modern C++20 idioms.** Uses `std::filesystem` for path traversal, `std::optional` for result types, `enum class` with explicit underlying values, namespaces, default-argument streams. Idiomatic for the surface area built; small in absolute size.
- **CMake project structure for a multi-target C++ project.** Two executables sharing source files, FetchContent for a test dependency, per-target compile features and warning flags, cross-platform warning discipline (`/W4` MSVC vs `-Wall -Wextra -Wpedantic` GCC/Clang), CTest integration.
- **Catch2 testing.** Test cases with `TEST_CASE` and `SECTION`, output capture to `std::ostringstream`, exhaustive parser-rule coverage in `test_cli.cpp`.
- **Value-style error handling design.** Two-layer enum split between process exit codes and semantic categories; rationale for choosing values over exceptions captured (avoids RTTI and exception-safety auditing).
- **Repo-discovery algorithm implementation.** Walk-up parent-path search with idempotence-based termination at filesystem root — the canonical Git approach for sentinel-directory discovery.
- **Externalised planning discipline at single-repo scale.** Formal `plans/NNN_topic.md` convention with status enum, rejected-alternatives sections, per-step checklists, and exit criteria — a methodology that mirrors LifeOS at vault scale, applied to a ~2KLOC C++ project.
- **Honest scope discrimination between design ambition and implemented reality.** LifeOS Overview itself draws the "Reality vs README" line explicitly; the project's documentation refuses to conflate "what the README pitches" with "what the code does". Demonstrates engineering maturity around scope honesty.
- **First-principles reasoning about VCS internals (planned, not yet shipped).** Decisions document captures the *why* behind content-addressed storage, deterministic encoding, derived diffs, and `.chrona/` sentinel discovery — engagement with the Git data model at the design level even though the implementation is not yet started.

What Chrona does *not* yet demonstrate: any working VCS functionality, hashing, object storage, commit DAG construction, or anything from Milestones 1-8. Honest current scope is "scaffold + planning artefact for a future VCS engine", not "a VCS engine".

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Chrona/Overview.md | 71 | "Chrona is a correctly-scoped, well-planned, earnestly-scaffolded C++ VCS learning project whose README describes a finished engine and whose code describes a half-finished `argc/argv` parser — the gap between the two is the entire point of this note existing." |
| Projects/Chrona/Architecture.md | 138 | "| No static-analysis config | No `.clang-tidy`, no IWYU mapping |" |
| Projects/Chrona/Decisions.md | 226 | "- [[Chrona/Roadmap]] — D12's exclusions bound the roadmap" |
| Projects/Chrona/Gaps.md | 270 | "- [[Chrona/Roadmap]] — what depends on each of these being closed" |
| Projects/Chrona/Plans Workflow.md | 130 | "- [[LifeOS/Overview]] — LifeOS uses a similar externalised-state discipline at a larger scale; plans/ is the project-local equivalent" |
| Projects/Chrona/Roadmap.md | 195 | "- [[Chrona/Plans Workflow]] — the plans/ convention that future milestone plans will follow" |
| Projects/Chrona/Systems/Build and Test.md | 176 | "- [[Chrona/Gaps]] — the commented-out tests, the missing library target, the missing `target_include_directories` on the main target" |
| Projects/Chrona/Systems/CLI.md | 117 | "- [[Chrona/Gaps]] — the init-stub gap originates here" |
| Projects/Chrona/Systems/Errors.md | 124 | "- [[Chrona/Architecture]] — compile graph showing errors is linked into both targets" |
| Projects/Chrona/Systems/Repo Discovery.md | 138 | "- [[Chrona/Gaps]] — the `exists` vs `is_directory` gap and the commented-out test file are recorded there" |
