---
name: Chrona
status: paused
source_repo: https://github.com/Capataina/Chrona
lifeos_folder: Projects/Chrona
last_synced: 2026-05-31
sources_read: 11
---

# Chrona

## One-line summary

A C++20 learning project rebuilding the "inner engine" of Git from first principles — currently a CMake + CLI + error-model + repo-discovery scaffold rather than a working VCS, deliberately scoped against "endless Git parity" via 9 milestone gates and a `plans/` SSOT directory.

## What it is

A deliberately-scoped personal learning project in modern C++ (C++20) aimed at rebuilding Git's content-addressed engine from first principles — drawing a sharp line between the VCS engine (Git) and the hosting platforms layered on top (GitHub/GitLab/Codeberg) and choosing to implement the engine side. The project is public at `Capataina/Chrona`, created 2025-12-23 and last touched 2025-12-27 (7 commits total, all within one week, then dormant for ~16 weeks per the 2026-04-24 vault verification). The current scope is ~20.8KB of C++ across 7 source files plus ~15.9KB of C++ tests and ~11.5KB of markdown planning documents. The design ambition is a full 9-milestone roadmap (foundations, object store, trees, commits, index, diff, branches, merge, storage/perf, documentation); the implemented scope is Milestone 0 only, and even that is partial (init is a TODO stub, two of three test files commented out of CMake). The README/LifeOS distinguish "what is designed" from "what is built" explicitly — the project's value-as-portfolio sits in the design discipline and planning artefacts, not in shipped VCS behaviour.

## Architecture

Single-binary C++20 project built with CMake 3.20+, organised folder-per-subsystem under `src/` with a flat `namespace chrona {}` (no nested `chrona::cli::` etc.). The build defines two executables that share most source files rather than going through a shared library target:

```
chrona/
├── CMakeLists.txt              C++20, Catch2 v3.5.0 via FetchContent
├── README.md                   Aspirational 9-milestone pitch
├── plans/                      Planning SSOT (NNN_topic.md schema)
│   ├── README.md               Plan index
│   ├── ARCHITECTURE.md         Repo-internal architecture reference
│   └── 000_foundations.md      Milestone 0 plan (in progress)
├── src/
│   ├── main.cpp                Entry — parse args, switch on action
│   ├── cli/                    Command enum, ParseAction, ParseResult, parse_args, print_usage
│   ├── errors/                 ExitCode, ErrorCode, Error, create/print/exit helpers
│   └── repo/                   find_repo() walk-up implementation
└── tests/
    ├── test_cli.cpp            11 TEST_CASEs — compiled into chrona_tests
    ├── test_errors.cpp         2 TEST_CASEs — commented out of CMake
    └── test_repo.cpp           273-line debug scaffold — commented out
```

**Compile-time dependency direction:**

```
              ┌──────────────┐
              │   main.cpp   │
              └──────┬───────┘
                     │
        ┌────────────┴────────────┐
        ▼                         ▼
   cli/cli.{hpp,cpp}         errors/error.{hpp,cpp}

   repo/repo.{hpp,cpp}   ← compiled into both targets, NOT called from main.cpp
                           (only caller was test_repo.cpp which is commented out)
```

`src/repo/` is therefore dead code in practice at the verified commit: `find_repo` is declared, defined, and compiled into both `chrona` and `chrona_tests` targets, but `main.cpp` never calls it and the only test that would call it is excluded from the build.

**Target composition:**

| Target | Sources | External deps |
|---|---|---|
| `chrona` | `main.cpp`, `errors/error.cpp`, `repo/repo.cpp`, `cli/cli.cpp` | none (stdlib only) |
| `chrona_tests` | `repo/repo.cpp`, `errors/error.cpp`, `cli/cli.cpp`, `tests/test_cli.cpp` | `Catch2::Catch2WithMain` |

Both executables compile the shared `.cpp` files independently (no `add_library(chrona_core ...)` yet). At ~2KLOC this is negligible; the LifeOS Build-and-Test note flags an eventual `chrona_core` static library refactor.

**Runtime data flow (current — almost everything is "not yet"):**

```
argv[] → parse_args() → ParseResult{action, command?, args[], error_msg?}
                              │
                              ▼
                       switch (action)
              ┌───────────────┼────────────────┐
              ▼               ▼                ▼
          ShowHelp          Error          RunCommand
              │               │                │
       print_usage()   print_error     switch (command)
              │               │                │
              ▼               ▼                ▼
           exit 0         exit 1       Command::Init
                                              │
                                              ▼
                                       // TODO: Implement
                                          return 0;
```

`find_repo()` is conspicuously absent from this diagram — the integration step (Step 9 of the foundations plan) is unchecked.

**Build settings:** CMake minimum 3.20; C++20 via `target_compile_features`; warnings `/W4` (MSVC) or `-Wall -Wextra -Wpedantic` (GCC/Clang); no `-Werror`; no suppressions. CTest registers the test binary as a single test (`add_test(NAME chrona_tests COMMAND chrona_tests)`) rather than per-Catch2-test (`catch_discover_tests` is documented as a future improvement). Include-directory discipline is asymmetric: `target_include_directories(chrona_tests PRIVATE src)` is set, the equivalent on the `chrona` main target is missing (relies on implicit resolution).

## Subsystems and components

### CLI (`src/cli/`)

Manual `argc/argv` parser, no third-party CLI library. Public surface is `enum class Command { Init }` (one variant only), `enum class ParseAction { RunCommand, ShowHelp, Error }`, a `ParseResult` struct carrying the action plus `std::optional<Command>` / `std::vector<std::string> args` / `std::optional<std::string> error_message`, and the entry points `parse_args(int, char*[])` and `print_usage()`. The parser exhaustively handles: no-args (ShowHelp with "No arguments provided" message), `--help` (ShowHelp), `init` (RunCommand→Init), unknown commands (Error), and `argc > 2` rejected as "Too many arguments provided". Case-sensitive (`INIT` ≠ `init`), and the `-h` short flag is rejected as an unknown command. 11 Catch2 TEST_CASEs in `tests/test_cli.cpp` cover every branch. `ParseResult.args` is declared but never populated — every return site uses `{}` for it. The `argc > 2` guard is a forward-cost: it explicitly blocks every milestone beyond M0 (M1's `chrona hash-object <file>` requires relaxing it), and the existing test even contains a comment admitting *"This might be a bug - you may want to handle extra args for init differently"* while still asserting the current behaviour.

### Errors (`src/errors/`)

Value-based error model (no exceptions). Two enum-class layers — `ExitCode { Success=0, GeneralError=1, UsageError=2 }` is the process-level signal, `ErrorCode { NotFound, AlreadyExists, InvalidArgument, IOError, UnknownError }` is the semantic signal — paired in a `struct Error { exit_code, error_code, message }`. Two overloads of `create_error` both return `std::optional<Error>` (always-engaged in practice — the function never returns `nullopt`, so callers redundantly `.value()` the result). `print_error` formats as `"Error: <message>\n"` to a configurable stream (default `std::cerr`). `exit_with_error` exists but is unused; it ignores the struct's own `exit_code` field and uses a separate `int exit_code` parameter (latent defect). Tests (`tests/test_errors.cpp`, 2 TEST_CASEs) exist but are commented out of the CMake target.

### Repo Discovery (`src/repo/`)

`std::optional<std::filesystem::path> find_repo(const std::filesystem::path& start_path)` — Git's classic walk-up algorithm. Loops from `start_path` upward, returning the first ancestor that contains a `.chrona/` entry; terminates when `parent_path() == current` (filesystem root, both Unix `/` and Windows `C:\\`) or `parent.empty()`. Uses `std::filesystem::exists`, which the LifeOS Gaps and Repo-Discovery notes both flag as a defect — it accepts a regular file named `.chrona` as a repo root because `exists()` does not distinguish files from directories (`is_directory` is the correct check). Tests live in a 273-line debug-scaffold `tests/test_repo.cpp` where 8 of 9 SECTIONs end in `REQUIRE(true)` and only 2 sections (root-returns-nullopt on `/` and `C:\\`) make real assertions — and the entire file is commented out of CMake, so zero of those assertions run. `find_repo` is currently dead code: `main.cpp` does not include the header, and the only command that exists (`init`) is a stub that would not need it anyway.

### Build and Test (CMake + Catch2)

Most-complete subsystem in the project. CMake 3.20+, project version `0.1.0`, language `CXX`. Catch2 v3.5.0 pulled via `FetchContent` from `github.com/catchorg/Catch2.git` at configure time (no vendoring, no system-install path; offline first-configures fail unless Catch2 is mirrored). Warning level: `/W4` on MSVC, `-Wall -Wextra -Wpedantic` on GCC/Clang. CTest enabled with a single `add_test(NAME chrona_tests COMMAND chrona_tests)`. `.gitignore` covers cross-platform binary patterns plus `build/`, `.cursor/`, `.cache/`, and `claude.md` (with `*.lib` duplicated harmlessly). 11 of 22 written tests run (test_errors.cpp and test_repo.cpp commented out — no commit message explains the exclusion; the LifeOS note speculates noise from test_repo's `std::cout` debug output as the likely reason).

## Technologies and concepts demonstrated

### Languages
- **C++20** — entire codebase. Uses modern features: `enum class`, `std::optional<T>`, `std::filesystem`, `std::vector<std::string>`, namespaced declarations, `target_compile_features(... cxx_std_20)`. Code surface is small (~5KB of `src/`), idiomatic, and free of legacy C-style constructs in the live source.

### Frameworks and libraries
- **Catch2 v3.5.0** — test framework, fetched via CMake `FetchContent`. Uses `Catch2::Catch2WithMain` target (Catch2 provides `main()`), and `TEST_CASE` + `SECTION` style assertions. Test-only dependency; runtime is stdlib-only.

### Runtimes / engines / platforms
- **No runtime engine.** Single CLI binary, no daemon, no GUI, no library consumers. Cross-platform intent (Windows MSVC + Linux/macOS GCC/Clang code paths in CMake), though the LifeOS note marks actual cross-platform validation as unverified (no CI exists).

### Tools
- **CMake 3.20+** — primary build system. `FetchContent` for the test dependency, `target_compile_features` for per-target language standard, conditional `MSVC` block for compiler-specific warning flags, `enable_testing()` + `add_test` for CTest integration.
- **Catch2 CTest registration** — single-test registration today; `include(Catch) + catch_discover_tests` documented as the eventual evolution for per-test reporting.

### Domains and concepts
- **Version control internals (planned, not built).** The README and Roadmap commit to content-addressed object stores, deterministic encoding, snapshot-as-truth with derived diffs, commit DAG, ref system, index/staging layer, three-way merge, and packfile-style storage as future milestones. None of these are implemented as of `652fb7f`; what is demonstrated is the *design discipline* around them, not the implementations themselves.
- **CLI argument parsing without a library** — explicit `argc/argv` string-compare dispatch with a tagged-union `ParseResult`. Conscious rejection of CLI11/cxxopts (documented in `plans/000_foundations.md` §"Rejected alternatives").
- **Value-based error handling** — `Error` struct + `std::optional` return, with a deliberate split between process-level `ExitCode` (shell exit codes) and semantic `ErrorCode` (failure category) so future translation/recovery can branch on the semantic layer without exit-code collisions.
- **Filesystem walk-up repo discovery** — Git's `.git/`-sentinel algorithm reimplemented for `.chrona/`. Termination via `parent_path()` idempotence at filesystem root with a defensive `empty()` check.
- **Externalised planning state (plan-as-SSOT)** — the `plans/NNN_<topic>.md` convention is the project's most distinctive engineering artefact. Each plan follows an 11-section schema (goal/scope, context, rejected alternatives, assumptions, interfaces/contracts, impacted areas, incremental implementation with per-step verification, testing/validation, risks, exit criteria, future considerations). The plan is the falsifiable design document; the README is the elevator pitch. This pattern mirrors LifeOS's broader externalised-working-memory discipline applied to a single repo.
- **Milestone-gated correctness-first delivery** — 9 README milestones each with demo-able output and tests, no milestone closed without exit-criteria validation. M7 (merge) and M8 (storage/perf) marked explicitly optional ("deep dive"/"systems flex") to bound scope against open-ended Git parity.

## Key technical decisions

Twelve decisions are formally captured in `Projects/Chrona/Decisions.md`, each with alternatives and reasoning:

- **D1 Local-first (no networking).** Explicit out-of-scope statement in README §"Explicitly out of scope". Networking would turn the project into a half-built Git competitor rather than a learning tool about the engine.
- **D2 Content-addressed, immutable-by-construction core.** Write-once content-addressed blobs with corruption/tampering detectable by design. Rejected: mutable record storage with separate integrity layer; delta-first encoding (SVN-style). **Status: not implemented** — locked in by README but unbacked by code.
- **D3 Determinism as a first-class property.** Identical inputs → identical object identities across runs and platforms. Implies canonical byte encoding, UTC timestamps, stable integer widths, no reliance on filesystem traversal order. Pre-implementation but enforced by M1 exit criteria.
- **D4 Diffs are derived, not stored.** Snapshots are primary truth; diff/status/log computed on demand. Rejected: SVN/RCS-style diff-as-truth; hybrid snapshot+cached-diff.
- **D5 Milestone-driven, correctness-first, scope-controlled.** 9 milestones (0–8) each with demo-able output, tests, and documented exit criteria. M7 and M8 flagged optional to bound scope.
- **D6 Stdlib-first dependency policy.** Prefer C++ stdlib; one lightweight header-only library allowed if a specific pain becomes unwieldy. Rejected CLI11, cxxopts, Boost. Runtime: zero dependencies. Test: Catch2 only.
- **D7 Manual `argc/argv` argument parsing.** Subcase of D6. Implemented. Known forward-cost: the `argc > 2` guard blocks M1's first multi-arg command.
- **D8 Error model: struct + optional, not exceptions.** Values can be pattern-matched and composed; exceptions add RTTI surface and exception-safety auditing burden. Known defect: the `std::optional<Error>` return is always engaged (over-engineered relative to current behaviour). `std::expected<T,Error>` rejected as C++23-only; `tl::expected` would violate D6.
- **D9 Folder-per-subsystem with flat namespace.** `src/cli/`, `src/errors/`, `src/repo/` folders but one `namespace chrona {}`. Avoids `using namespace` verbosity at small size; nested namespaces revisitable if name collisions emerge.
- **D10 Catch2 (over GoogleTest).** Header-light, `TEST_CASE`+`SECTION` fits small tests well, `Catch2::Catch2WithMain` removes boilerplate. Cost: configure-time network dependency.
- **D11 `.chrona/` as the repository sentinel.** Directory (not file, not env var, not central index) at repo root. Mirrors Git's `.git/` pattern for the same reasons: local-first, self-contained, unambiguous.
- **D12 Explicit scope exclusions up-front.** README §"Explicitly out of scope" names networking, history rewriting, packfiles/delta/GC, signing, hooks, submodules, LFS, sparse checkout as out-of-scope to prevent "endless Git parity" failure mode.

## What is currently built

What actually exists and runs at the verified commit `652fb7f`:

| Component | State | Evidence |
|---|---|---|
| CMake build (C++20, Catch2 v3.5.0 FetchContent, two targets) | Working | CMakeLists.txt |
| CLI parser (`parse_args`, `print_usage`, manual `argc/argv`) | Implemented | src/cli/cli.cpp |
| `chrona init` | **Stub** — `// TODO: Implement init command` then `return 0` | src/main.cpp:43-45 |
| Repo discovery (`find_repo` walk-up) | Implemented but unused | src/repo/repo.cpp |
| Error model (`ExitCode`, `ErrorCode`, `Error`, `create_error`, `print_error`, `exit_with_error`) | Implemented (with two latent defects) | src/errors/ |
| Object store (blobs, hashing, `.chrona/objects/`) | **Not started** | absent from src/ |
| Trees / snapshots | **Not started** | absent |
| Commits / DAG | **Not started** | absent |
| Refs / HEAD / branches | **Not started** | absent |
| Staging / index | **Not started** | absent |
| Diff | **Not started** | absent |
| Tests wired to CTest | **Partial** — `test_cli.cpp` runs (11 tests); `test_errors.cpp` and `test_repo.cpp` commented out of build | CMakeLists.txt:32-34 |

Runnable behaviour after `cmake -B build && cmake --build build`:
1. `chrona` (no args) → prints usage to stdout, exits 0
2. `chrona --help` → prints usage, exits 0
3. `chrona init` → does nothing, exits 0 (the TODO path)
4. `chrona <unknown>` → `Error: Unknown command: <unknown>` to stderr, exits 1
5. `chrona a b` → `Error: Too many arguments provided` to stderr, exits 1

No persistent state is ever created. No `.chrona/` directory ever appears. The LifeOS Overview explicitly characterises the project as *"at ~2% of its stated scope"* — only Milestone 0 (Foundations) is in progress, and even Foundations is incomplete (init stub, two test files excluded, dependency policy unresolved beyond a one-line note).

## Current state

**Status: paused.** Marked `foundational — Milestone 0 partially complete, VCS core unbuilt` in the LifeOS Overview frontmatter. All 7 commits landed between 2025-12-23 and 2025-12-27; no commits since (~16 weeks dormant at the 2026-04-24 verification, ~22 weeks dormant at this sync). The `plans/000_foundations.md` plan has remained at `in_progress` since 2025-12-26 — the LifeOS note suggests `blocked` would be more honest than `in_progress` for a project that has not moved. Effort during the active week skewed heavily toward planning and tests: ~30KB across `plans/`, `README.md`, and tests vs ~5KB of production `src/`, a ~6:1 documentation-to-production ratio. Per LifeOS cross-vault notes, Chrona is not on the active-projects list (which contains Cernio, Flat Browser, Aurix, NeuroDrive, Nyquestro, LifeOS, Claude Config); it is a candidate project — scoped, planned, scaffolded, paused.

## Gaps and known limitations

Eighteen gaps formally catalogued in `Projects/Chrona/Gaps.md`, ranked by severity. Highlights with portfolio-relevance:

**Critical:**
- **G1 `chrona init` is a TODO stub.** The one command the CLI recognises does nothing. Plan reference `plans/000_foundations.md §"Step 7"` is unchecked. Every downstream milestone depends on init working.
- **G2 Entire VCS core is unimplemented.** No object store, hashing, trees, commits, refs, index, or diff. The LifeOS overview explicitly flags this as the dominant honest summary: *"a CMake/CLI/error-model scaffold in C++, planned as a future VCS"*, not *"a content-addressed VCS in C++"*.

**High (real defects in code that exists):**
- **G3 Two of three test files are commented out of the build.** 11 of 22 written tests run. No commit message explains the exclusion.
- **G4 `exit_with_error` ignores the struct's own `exit_code`.** Latent (no caller today); future caller `exit_with_error(usage_err)` would exit 1 instead of 2.
- **G5 `main.cpp` returns hardcoded `1` instead of propagating `err.exit_code`.** Contradicts plan-stated `2 = usage error` convention; shell-level differentiation between usage and general errors is lost.
- **G6 `find_repo` accepts a file named `.chrona` as a repo root.** Uses `exists`, not `is_directory`.

**Medium (design smells):**
- **G7 `parse_args` rejects any multi-argument command.** `argc > 2` guard blocks every milestone beyond M0.
- **G8 `ParseResult.args` declared but never populated.** Zero impact today; field is documentation of intent without delivery.
- **G9 `create_error` returns `std::optional<Error>` that is never `nullopt`.** Dead optionality.
- **G10 No `target_include_directories` on the main `chrona` target** (asymmetric with `chrona_tests`).
- **G11 No shared library target** — `errors/`, `repo/`, `cli/` source files compiled twice (negligible at 2KLOC; eventual refactor at 20KLOC).
- **G12 CTest registration is single-line.** No per-test breakdown; `catch_discover_tests` is the documented fix.

**Low (hygiene):**
- **G13 Help text advertises `chrona.com` that Caner does not own.**
- **G14 Help text lists a `help` command that returns an error.**
- **G15 `*.lib` duplicated in `.gitignore`** (harmless).
- **G16 No `.clang-format`, `.clang-tidy`, `.editorconfig`.**
- **G17 No CI configured.** No GitHub Actions, no Travis. Defensible on a single-author paused learning project.
- **G18 No dependency policy document beyond a one-line note** (plan Step 2 unchecked).

**Unverified:** Cross-platform Windows/Linux claim in the README is unverified — no CI, no commit trail of cross-platform fixes. `test_repo.cpp` has Windows-specific `C:\\` handling suggesting the intent.

## Direction (in-flight, not wishlist)

Nothing is actively in flight. The project has been dormant since 2025-12-27 and the LifeOS Roadmap explicitly recommends treating it as "scoped and scaffolded, waiting on activation" rather than "in active development".

If/when resumption happens, the LifeOS Roadmap sketches the likely opening sequence: close G1 (implement `init`, one afternoon), close G3 (re-enable `test_errors.cpp` and rewrite `test_repo.cpp` as real tests, one evening), open `plans/001_object_store.md` using the foundations schema, pick a hash function (SHA-256 or BLAKE3 are the candidates discussed), build `chrona hash-object <file>` and `chrona cat-object <hash>`. That sequence would move the project from ~2% to ~15% of the README roadmap (M0 closed, M1 MVP). This is a weekend's work, not a sprint, but it is not currently scheduled.

## Demonstrated skills

What this project actually proves at its current state — distinct from what its README aspires to demonstrate:

- **Modern C++ (C++20) competence on a small surface.** Idiomatic use of `enum class`, `std::optional`, `std::filesystem`, namespaced declarations, `target_compile_features` per target. The surface is tiny (~5KB of `src/`), so this is signal of fluency rather than depth — single small in-progress project, not a portfolio centrepiece for C++ specifically.
- **CMake build engineering.** Multi-target build with `FetchContent` for a test-only dependency, conditional MSVC/GCC/Clang warning flags, per-target language standard via `target_compile_features`, CTest integration. Includes documented awareness of asymmetries (`target_include_directories` missing on main target) and growth-path refactors (`add_library(chrona_core ...)`).
- **Manual CLI parsing without a library.** Tagged-union `ParseResult` with `enum class ParseAction`, exhaustive branch coverage tested by 11 Catch2 TEST_CASEs, explicit rejection of CLI11/cxxopts with reasoning recorded.
- **Value-based error modelling.** Two-layer enum split (`ExitCode` for process, `ErrorCode` for semantics) joined in a struct; conscious rejection of exceptions with rationale (RTTI surface, exception-safety auditing burden); awareness that the `std::optional<Error>` signature is over-engineered relative to its current behaviour.
- **Filesystem walk-up algorithm** for repo discovery (Git's `.git/`-sentinel pattern reimplemented for `.chrona/`), with explicit termination invariants documented for both Unix and Windows filesystem roots.
- **Plan-as-SSOT engineering discipline.** Eleven-section plan schema (goal/scope, rejected alternatives, assumptions, interfaces/contracts, impacted areas, per-step incremental implementation with verification, testing strategy, risks, exit criteria, future considerations) externalises design state before implementation, makes the decision log falsifiable, and provides per-milestone exit criteria. This is unusual for a ~2KLOC personal project and is one of the strongest portfolio signals the project produces.
- **Honest scope discipline.** Explicit out-of-scope statement in the README naming what will *not* be built (networking, history rewriting, packfiles/delta/GC, signing, hooks, submodules, LFS, sparse checkout) to prevent "endless Git parity" sprawl. Pairs with milestone-gated correctness-first methodology that refuses to mark a milestone complete without passing tests and exit-criteria validation.
- **Catch2 testing discipline (selective).** 11 working TEST_CASEs in `test_cli.cpp` cover every parser branch exhaustively. Caveated by the visible tech debt of two test files commented out of the build without an explanatory commit message.
- **Self-aware engineering documentation.** The LifeOS notes themselves demonstrate willingness to record gaps, latent defects, dead code in practice, and the gap between the README's pitch and the code's reality. The "Reality vs README" call-out at the top of the Overview ("Every VCS claim in the README is aspirational roadmap, not current code") is the strongest signal: the engineer maintains an honest internal accounting of state separately from the external pitch.

What this project does *not* demonstrate, contrary to README pitch language: working content-addressed object storage, hashing, commit DAGs, diffs, branching, merge, or anything else from M1-M8. Any role requiring shipped VCS-internals experience should not weight Chrona as evidence of that experience — it is evidence of having scoped, planned, and started such a project, then paused.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Chrona/_Overview.md | 82 | "Chrona is a correctly-scoped, well-planned, earnestly-scaffolded C++ VCS learning project whose README describes a finished engine and whose code describes a half-finished `argc/argv` parser — the gap between the two is the entire point of this note existing." |
| Projects/Chrona/Architecture.md | 159 | "\| No static-analysis config \| No `.clang-tidy`, no IWYU mapping \|" |
| Projects/Chrona/Decisions.md | 226 | "- [[Chrona/Roadmap]] — D12's exclusions bound the roadmap" |
| Projects/Chrona/Gaps.md | 276 | "- [[Chrona/Roadmap]] — what depends on each of these being closed" |
| Projects/Chrona/Plans Workflow.md | 117 | "- [[LifeOS/_Overview]] — LifeOS uses a similar externalised-state discipline at a larger scale; plans/ is the project-local equivalent" |
| Projects/Chrona/Roadmap.md | 225 | "- [[Chrona/Plans Workflow]] — the plans/ convention that future milestone plans will follow" |
| Projects/Chrona/Systems/_Overview.md | 39 | "- [[Projects/Chrona/Roadmap]] — direction-of-travel" |
| Projects/Chrona/Systems/Build and Test.md | 203 | "- [[Chrona/Gaps]] — the commented-out tests, the missing library target, the missing `target_include_directories` on the main target" |
| Projects/Chrona/Systems/CLI.md | 118 | "- [[Chrona/Gaps]] — the init-stub gap originates here" |
| Projects/Chrona/Systems/Errors.md | 123 | "- [[Chrona/Architecture]] — compile graph showing errors is linked into both targets" |
| Projects/Chrona/Systems/Repo Discovery.md | 129 | "- [[Chrona/Gaps]] — the `exists` vs `is_directory` gap and the commented-out test file are recorded there" |
