---
name: Chrona
status: paused
source_repo: https://github.com/Capataina/Chrona
lifeos_folder: Projects/Chrona
last_synced: 2026-05-13
sources_read: 11
---

# Chrona

## One-line summary

A C++20 personal-learning attempt to reimplement Git's "inner engine" (content-addressed object store, trees, commit DAG, refs, index, diff) from first principles, currently parked at the CMake + CLI + error-model + repo-discovery scaffold stage with the VCS core entirely unbuilt.

## What it is

Chrona is framed in its README as a local, Git-inspired version-control engine designed to make the distinction between *VCS engine* (Git) and *hosting platform* (GitHub/GitLab/Codeberg) concrete by implementing the engine side from scratch. The stated scope is nine milestones (Foundations through Documentation), with the project's stated design principles being a content-addressed immutable core, determinism, derived (not stored) diffs, and milestone-gated correctness-first delivery. LifeOS explicitly flags a gap between this pitch and reality: as of the last verified commit `652fb7f` (2025-12-27), nothing about content-addressing, hashing, trees, commits, refs, indexing, or diffing exists in code — the repository implements only the build system, a one-command CLI parser, an error model, and a parent-walk-up repository-discovery function. The project was committed across a single week (2025-12-23 to 2025-12-27, 7 commits) and has been dormant for roughly four months since. LifeOS characterises Chrona as a "candidate project — scoped, planned, scaffolded, paused", not an active one.

## Architecture

The repository follows a folder-per-subsystem layout under `src/`, with all declarations placed in a single flat `namespace chrona {}` (no nested `chrona::cli::` / `chrona::errors::` namespaces). The current top-level structure as of `652fb7f`:

```
chrona/
├── CMakeLists.txt              C++20, Catch2 v3.5.0 via FetchContent
├── README.md                   Aspirational pitch + 9-milestone roadmap
├── .gitignore
├── plans/                      Planning docs (NNN_topic.md convention)
│   ├── README.md               Plan index
│   ├── ARCHITECTURE.md         Repo-internal architecture reference
│   └── 000_foundations.md      Milestone 0 plan (in_progress)
├── src/
│   ├── main.cpp                Entry point — parse args, switch on action
│   ├── cli/                    Command enum, ParseAction, ParseResult, parse_args, print_usage
│   ├── errors/                 ExitCode, ErrorCode, Error, create/print/exit helpers
│   └── repo/                   find_repo() — walks up parent paths looking for `.chrona/`
└── tests/
    ├── test_cli.cpp            11 Catch2 TEST_CASEs — wired into chrona_tests
    ├── test_errors.cpp         2 TEST_CASEs — commented out of CMake
    └── test_repo.cpp           273-line debug scaffold — commented out of CMake
```

Compile-time dependency direction (from LifeOS Architecture.md):

```
                            ┌──────────────┐
                            │   main.cpp   │
                            └──────┬───────┘
                                   │
                     ┌─────────────┴─────────────┐
                     ▼                           ▼
              ┌──────────────┐           ┌──────────────┐
              │ cli/cli.hpp  │           │ errors/      │
              │ cli/cli.cpp  │           │   error.hpp  │
              └──────────────┘           │   error.cpp  │
                                         └──────────────┘

              ┌──────────────┐
              │ repo/        │   NOT linked from main.cpp today
              │   repo.hpp   │   (find_repo exists, is unused)
              │   repo.cpp   │
              └──────────────┘
```

Two CMake targets share most sources without an intermediate library:

| Target | Sources | Deps |
|--------|---------|------|
| `chrona` | `main.cpp`, `errors/error.cpp`, `repo/repo.cpp`, `cli/cli.cpp` | (none — stdlib only) |
| `chrona_tests` | `repo/repo.cpp`, `errors/error.cpp`, `cli/cli.cpp`, `tests/test_cli.cpp` | `Catch2::Catch2WithMain` |

Build properties verified against `CMakeLists.txt`:

| Facet | Setting |
|-------|---------|
| CMake minimum | 3.20 |
| Project version | 0.1.0 |
| Language standard | C++20 (per-target via `target_compile_features`) |
| External deps | Catch2 v3.5.0 (FetchContent, test-only) |
| Runtime deps | None — stdlib only |
| Warnings (MSVC) | `/W4` |
| Warnings (GCC/Clang) | `-Wall -Wextra -Wpedantic` |
| `-Werror` / `/WX` | not set |
| CTest integration | `enable_testing()` + single `add_test(NAME chrona_tests ...)` registration |

Runtime data flow today consists entirely of CLI parsing followed by a stub dispatch — LifeOS captures the full runtime diagram on one page because almost no runtime behaviour exists:

```
 argv[] → parse_args() → ParseResult{ action, command?, args, error_msg? }
                                  │
                                  ▼
                          switch (action):
                            ShowHelp     → print_usage();              return 0
                            Error        → print_error(InvalidArgument); return 1
                            RunCommand   → switch (command):
                                             Init  → // TODO: Implement; return 0
```

`find_repo()` is compiled into both targets but called from neither — LifeOS Architecture.md flags it explicitly as dead code in practice at `652fb7f`.

## Subsystems and components

### CLI (`src/cli/`)

Responsibility: parse `argc`/`argv` into a `ParseResult` and emit the usage banner. The contract:

```cpp
namespace chrona {
  enum class Command { Init };
  enum class ParseAction { RunCommand, ShowHelp, Error };

  struct ParseResult {
    ParseAction action;
    std::optional<Command> command;
    std::vector<std::string> args;
    std::optional<std::string> error_message;
  };

  ParseResult parse_args(int argc, char* argv[]);
  void print_usage();
}
```

Only one command is registered: `Command::Init`. The full parse-rule table (from LifeOS Systems/CLI.md):

| Input | action | command | error_message |
|-------|--------|---------|---------------|
| `chrona` (argc=1) | `ShowHelp` | none | `"No arguments provided"` |
| `chrona --help` | `ShowHelp` | none | none |
| `chrona init` | `RunCommand` | `Init` | none |
| `chrona <anything else>` | `Error` | none | `"Unknown command: <arg>"` |
| `chrona a b` (argc>2) | `Error` | none | `"Too many arguments provided"` |
| `chrona ""` | `Error` | none | `"Unknown command: "` |
| `chrona INIT` (wrong case) | `Error` | none | `"Unknown command: INIT"` |
| `chrona -h` | `Error` | none | `"Unknown command: -h"` |
| `chrona help` | `Error` | none | `"Unknown command: help"` |

Eleven Catch2 `TEST_CASE`s in `tests/test_cli.cpp` cover the matrix and are the only tests that actually run. Current state: implemented but rigidly constrained — the `argc > 2` guard at the top of `parse_args` blocks any future multi-argument command (e.g. Milestone 1's `chrona hash-object <file>`), and `ParseResult.args` is declared but never populated. The help text advertises a `help` subcommand that returns "Unknown command" and a `https://chrona.com` URL that does not exist.

### Errors (`src/errors/`)

Responsibility: a value-style error model (no exceptions). Two enum classes plus a paired struct:

```cpp
enum class ExitCode { Success = 0, GeneralError = 1, UsageError = 2 };
enum class ErrorCode { NotFound, AlreadyExists, InvalidArgument, IOError, UnknownError };

struct Error {
  ExitCode  exit_code;
  ErrorCode error_code;
  std::string message;
};
```

Creation API has two overloads, both returning `std::optional<Error>`. The shorter overload picks an `ExitCode` automatically (`InvalidArgument` → `UsageError`, everything else → `GeneralError`). Output API is `print_error(const Error&, std::ostream& = std::cerr)` which emits `"Error: <message>\n"`, and `exit_with_error(const Error&, int exit_code = GeneralError)` which prints to stderr and calls `std::exit`.

Three latent defects captured in LifeOS Systems/Errors.md and Gaps.md:

1. `create_error` returns `std::optional<Error>` but never returns `std::nullopt` — every caller has to `.value()` for no safety benefit.
2. `exit_with_error` ignores the struct's `error.exit_code` and uses its separate `int exit_code` parameter (default `GeneralError`), so a future caller writing `exit_with_error(usage_err)` would exit 1 when the struct said 2. Latent because no caller exists today.
3. The parse-error branch in `main.cpp` carefully constructs `ErrorCode::InvalidArgument` (which maps to `UsageError = 2`), then returns hardcoded `1` — contradicting `plans/000_foundations.md` §"Interfaces and contracts" which explicitly specifies `2 = usage error`.

Tests (`tests/test_errors.cpp`) exist (2 `TEST_CASE`s covering both overloads and the print format) but are commented out of `CMakeLists.txt`, so the assertions never run.

### Repo Discovery (`src/repo/`)

Responsibility: walk parent paths looking for a `.chrona/` sentinel. API:

```cpp
namespace chrona {
  std::optional<std::filesystem::path>
  find_repo(const std::filesystem::path& start_path);
}
```

Implementation is 12 lines: loop, `std::filesystem::exists(current / ".chrona")` returns on success, terminate when `parent == current` (filesystem-root idempotence) or `parent.empty()`. Tested against `/` (Unix root) and `C:\\` (Windows root) returning `nullopt`. Has one defect: uses `exists()` not `is_directory()`, so a regular file named `.chrona` somewhere up the tree would be accepted as a repo root. The test file (`tests/test_repo.cpp`) is a 273-line debug scaffold — 9 sections, 7 of which end with `REQUIRE(true)` and exist only to print `std::cout` diagnostic dumps; only 2 sections (root-path cases) assert real behaviour, and the whole file is commented out of `CMakeLists.txt` so even those never run.

Integration gap: `main.cpp` does not include `repo/repo.hpp` and no command calls `find_repo`. `plans/000_foundations.md` §"Step 9: Integration" — wire repo discovery into commands that need it — is unchecked. The function is compiled but exercised by nothing.

### Build and Test (`CMakeLists.txt`)

LifeOS describes the build system as "the most complete system in the project — ironic, given that the project's stated purpose is a VCS engine and the only thing that reliably works end-to-end is its build pipeline". Catch2 v3.5.0 is fetched at configure time via `FetchContent` (offline builds fail unless `build/_deps/` is mirrored). Both executables compile the same `cli.cpp`/`errors/error.cpp`/`repo/repo.cpp` independently — no shared library target. The main target is missing `target_include_directories(chrona PRIVATE src)` that the test target has; `chrona` compiles today only because CMake puts each source file's directory on the include path implicitly. CTest registration is a single `add_test(NAME chrona_tests COMMAND chrona_tests)` line — failures surface as one red line rather than per-test detail (`catch_discover_tests` is not used).

Commit `03c151d` ("changed the entire compiling structure", 2025-12-24) was the pivot from a single-file `main.cpp` experiment to a multi-module project with tests; the error system and test harness were introduced in the same commit that restructured the build, which LifeOS notes as "healthy ordering: infrastructure and content evolved together rather than scaffolding-first / content-later".

### Plans (`plans/`)

Not a code subsystem but a methodological one. The `plans/` folder is the single source of truth for implementation state, decisions, and next steps — a vault-like discipline applied to a single repo. Naming: `NNN_<short_topic>.md` (zero-padded sequence). Status enum: `planned`, `in_progress`, `blocked`, `complete`. Checklists: `[ ]` pending, `[x]` done. Per-plan schema (11 sections): Goal and scope, Context and justification, Rejected alternatives, Assumptions, Interfaces and contracts, Impacted areas, Incremental implementation, Testing and validation, Risks and failure modes, Exit criteria, Future considerations. The single live plan (`000_foundations.md`) establishes this schema; future milestones (`001_object_store.md`, `002_trees.md`, etc.) are expected to follow it. LifeOS Plans Workflow.md characterises this as "a blend of an RFC and a migration plan — heavy on explicit alternatives, rejected options, and measurable exit criteria. Effective for solo work because it forces the decision log to be written before the code, not after."

## Technologies and concepts demonstrated

### Languages

- **C++20** — used across all four subsystems. Standard set per-target via `target_compile_features(<target> PRIVATE cxx_std_20)`. Modern features observed in code: `std::filesystem::path`, `std::optional`, `enum class`, structured initialisation, `namespace chrona {}` flat namespace, `const_cast<char**>` in tests (a real C++ hazard LifeOS flags). Surface area is small (~5KB of `src/` production code, ~15.9KB of tests, ~20.8KB across all C++ files in `src/` per LifeOS Overview) — LifeOS Overview is explicit that one small in-progress project is not yet strong portfolio signal for C++ fluency despite the code being idiomatic.

### Frameworks and libraries

- **Catch2 v3.5.0** — test framework, fetched via `FetchContent` at configure time. `Catch2::Catch2WithMain` target provides `main()` so no test runner needs to be written. Chosen over Google Test for being header-light and pulling cleanly through `FetchContent` (Decision D10).

### Runtimes / engines / platforms

No source evidence in LifeOS — Chrona is a single-binary C++ executable with no runtime engine, framework, or platform layer beyond the C++ standard library.

### Tools

- **CMake 3.20+** — build configuration. `FetchContent` for Catch2, `enable_testing()` + `add_test(...)` for CTest integration, `target_compile_options` for warning discipline.
- **CTest** — wraps `chrona_tests` as a single test entry; per-test discovery via `catch_discover_tests` is not yet wired in.
- **Git** — version control of the project itself.

### Domains and concepts

- **Version-control systems theory** (stated, mostly unimplemented). The decisions are locked in: content-addressed immutable object storage; deterministic byte encoding so identical inputs produce identical object identities; snapshots as primary truth with diffs/status/log derived on demand; sentinel-directory repo identification (`.chrona/`); milestone-gated correctness-first scope. The README's nine milestones (Foundations, Object Store, Trees, Commits, Index, Diff, Branching, Merge, Storage/Performance, Documentation) and `plans/000_foundations.md` §"Rejected alternatives" articulate these. The implementation has not reached the object store yet.
- **Filesystem traversal idioms** — `find_repo`'s walk-up-to-root pattern is the canonical Git approach (the same `parent_path()` idempotence trick at filesystem roots).
- **CLI parser design** — tagged-union via `ParseAction` enum + `std::optional<Command>` + `std::optional<std::string> error_message`, rather than exceptions or a CLI library. Documented alternatives (CLI11, cxxopts) explicitly rejected.
- **Value-style error handling without exceptions** — `Error` struct pairing process-level `ExitCode` with semantic `ErrorCode` plus a message; deliberate choice over a `std::runtime_error` exception hierarchy (Decision D8). LifeOS notes `std::expected` (C++23) would be the natural next iteration but is unavailable in C++20.
- **Externalised planning discipline** — the `plans/NNN_topic.md` schema with eleven required sections per plan is the methodological signal: rejected alternatives and exit criteria written before the code.

## Key technical decisions

The full Decisions.md table from LifeOS is reproduced here; reversibility column quotes LifeOS verbatim.

| # | Decision | What was chosen | What was rejected | Reasoning |
|---|----------|-----------------|-------------------|-----------|
| D1 | Local-first | No networking, no server, no remote protocols, no auth | Git-parity remote push/pull/fetch; web UI; daemon | The README's positioning is engine-not-hosting; networking would make Chrona a half-built Git competitor rather than a learning tool. Reversibility: hard. |
| D2 | Content-addressed immutable core | Write-once content-addressed blobs (planned) | Mutable record storage with separate integrity layer; delta-first encoding (SVN-style) | The README's "integrity by construction" only works if the core is content-addressed. **Status: not implemented.** Reversibility: structural — reversal = new project. |
| D3 | Determinism as first-class | Identical inputs → identical object identities across runs/platforms | Timestamp- or nonce-salted ids; platform-dependent encoding | Without determinism, content-addressing collapses; the Milestone 1 exit criterion ("same content twice → same id") locks this in. Reversibility: structural. |
| D4 | Diffs are derived, not stored | Store snapshots, compute diff/status/log on demand | Store diffs as primary truth (SVN/RCS); hybrid snapshot + cached diffs | Canonical Git model; caching diffs would be an optimisation, not a replacement. Reversibility: easy to extend with a cache. |
| D5 | Milestone-driven, correctness-first | 9 numbered milestones with measurable exit criteria | Build whole system, test at end; no formal milestones | Forces verifiable progress on a learning project; M7 and M8 are flagged optional to bound scope. Reversibility: any time. |
| D6 | Stdlib-first dependency policy | C++ standard library only; one lightweight header-only library allowed if a specific need is unwieldy; no Boost | Heavy frameworks; Boost; Boost.Filesystem (`std::filesystem` is sufficient) | A learning project buried in framework abstractions teaches the framework, not the domain. Reversibility: soft. |
| D7 | Manual `argc/argv` parsing | Raw `argc`/`argv` string compare; `ParseResult` struct with `enum class ParseAction` | CLI11; cxxopts; internal parser generator | At one subcommand, manual is simpler than learning a library API. Known forward-cost: `argc > 2` guard blocks multi-arg commands and must be relaxed in Milestone 1. Reversibility: provisional. |
| D8 | Value errors, no exceptions | `Error` struct + `std::optional<Error>` returns from `create_error` | `throw`/`try`/`catch` hierarchy; `std::expected` (unavailable in C++20); `tl::expected` (would violate D6) | Exceptions add RTTI surface and exception-safety auditing; value errors compose and test more easily. Reversibility: soft — could move to `std::expected` in C++23. |
| D9 | Folder-per-subsystem + flat namespace | `src/cli/`, `src/errors/`, `src/repo/`; all in single `namespace chrona {}` | Everything flat in `src/`; nested namespaces matching folders | Folders for findability, flat namespace to keep public surface small. Reversibility: soft. |
| D10 | Catch2 v3.5.0 as test framework | Catch2 via `FetchContent` | Google Test (heavier, link-time complexity); hand-rolled runner | Header-light, `TEST_CASE`/`SECTION` macros fit small tests, clean `FetchContent` pull. Cost: configure-time network dependency. Reversibility: soft. |
| D11 | `.chrona/` as repo sentinel | Hidden directory at repo root; `find_repo` walks up looking for it | `CHRONA_ROOT` env var (violates D1); central index file (ditto); sentinel file instead of directory (objects/, refs/ need to live inside it) | Same pattern as Git for the same reasons: local-first, self-contained, unambiguous. Reversibility: hard — changing it breaks all existing repos. |
| D12 | Explicit scope exclusions up-front | README §"Explicitly out of scope" names networking, history rewriting, packfiles/delta/GC, signing, hooks, submodules, LFS, sparse checkout | Leaving scope open "to see what develops" | Unbounded scope on a learning project produces a partial imitation rather than the compression. Reversibility: process — can be broadened. |

LifeOS Decisions.md is the strongest single piece of evidence in the project about *design intent* — stronger than commit messages (terse), code comments (sparse), or the README (aspirational).

## What is currently built

LifeOS Overview.md and Gaps.md are explicit that the project sits at "~2% of its stated scope". The honest implemented surface as of `652fb7f`:

| Area | Status | Evidence (LifeOS-verified) |
|------|--------|----------------------------|
| CMake build | Working — C++20, FetchContent Catch2 v3.5.0, two targets (`chrona`, `chrona_tests`) | `CMakeLists.txt` |
| CLI parser | Minimal — `argc > 2` rejected as "Too many arguments", recognises `--help` and `init` literally, unknown commands error | `src/cli/cli.cpp` |
| `chrona init` | **Stub** — CLI dispatches to `Command::Init` but `main.cpp` only has `// TODO: Implement init command` and `return 0` | `src/main.cpp:44` |
| Repo discovery | Implemented — `find_repo()` walks parent paths looking for `.chrona/`, stops when `parent == current` | `src/repo/repo.cpp` |
| Error model | Implemented — `ExitCode` / `ErrorCode` enums, `Error` struct, `create_error` / `print_error` / `exit_with_error` | `src/errors/error.hpp` + `error.cpp` |
| Object store (blobs) | **Not started** — no hashing, no `.chrona/objects/`, no I/O code | absent from `src/` |
| Trees / snapshots | **Not started** | absent |
| Commits / DAG | **Not started** | absent |
| Refs / HEAD / branches | **Not started** | absent |
| Staging / index | **Not started** | absent |
| Diff | **Not started** | absent |
| Tests wired to CTest | **Partial** — only `tests/test_cli.cpp` is in the `chrona_tests` target; `test_errors.cpp` and `test_repo.cpp` are commented out | `CMakeLists.txt:32-34` |

What actually runs when the binary is built:

```
chrona              → prints usage, exits 0
chrona --help       → prints usage, exits 0
chrona init         → // TODO path; exits 0 without creating anything
chrona anythingelse → "Error: Unknown command: anythingelse" → exits 1
chrona a b          → "Error: Too many arguments provided" → exits 1
```

No persistent state. No `.chrona/` directory is ever created. `find_repo()` exists in the library but is not wired into any command.

Scale numbers from LifeOS Overview.md:

- 7 source files, ~20.8KB of C++ in `src/`.
- 3 test files, ~15.9KB of C++.
- 3 markdown planning documents, ~11.5KB.
- 7 commits total.
- Documentation-to-production ratio ~6:1 by bytes (plans/ + README.md + tests vs `src/`).

## Current state

Status: **paused** — LifeOS Overview.md classifies the repo as "dormant for 4 months. Last commit `652fb7f` on 2025-12-27; today is 2026-04-24. No activity for ~16 weeks." Plans Workflow.md flags `plans/000_foundations.md`'s `in_progress` marker as stale: "A `blocked` status would be more honest if the project is paused; `complete` is premature." There is no Work/ folder in the LifeOS project entry (the standard active-project marker). LifeOS Roadmap.md is explicit: "This is a parked project, not an active one... The vault should not treat Chrona as 'in active development' — it is 'scoped and scaffolded, waiting on activation'."

## Gaps and known limitations

LifeOS Gaps.md is structured by severity. Reproduced honestly here without softening — these are the deltas between what the project advertises and what it implements.

**Critical (blocks any forward progress):**

- **G1 — `chrona init` is a TODO stub.** The one command the CLI recognises does nothing; no `.chrona/` directory ever gets created, so every downstream milestone is gated. Fix path: add `init_repo(path)` to `repo.cpp`, create `.chrona/`, `.chrona/objects/`, `.chrona/refs/`, error with `AlreadyExists` if present, wire into `Command::Init` dispatch.
- **G2 — Entire VCS core is unimplemented.** No object store, no hashing, no trees, no commits, no refs, no index, no diff. Anti-puffing note from LifeOS verbatim: "If any external summary describes Chrona as 'a content-addressed VCS in C++', that summary is wrong. A correct summary is 'a CMake/CLI/error-model scaffold in C++, planned as a future VCS'."

**High (correctness or reliability in code that does exist):**

- **G3 — Two of three test files are commented out.** `test_errors.cpp` and `test_repo.cpp` are excluded from the `chrona_tests` target; the error system and repo discovery both have *zero* executing assertions despite tests existing on disk. `test_repo.cpp` is mostly debug scaffolding (8 of 9 sections use `REQUIRE(true)`); only 2 sections (Unix `/` and Windows `C:\\` roots returning `nullopt`) assert real behaviour, and even those never run.
- **G4 — `exit_with_error` drops the struct's `exit_code`.** Takes a separate `int exit_code` parameter defaulting to `GeneralError`, ignoring `error.exit_code`. Latent today — no caller — but any future use writing `exit_with_error(usage_err)` would exit 1 when the struct said 2.
- **G5 — `main.cpp` returns hardcoded `1` instead of propagating `UsageError = 2`.** The parse-error branch carefully constructs `ErrorCode::InvalidArgument` (which maps to `UsageError`), then discards the mapping. Contradicts `plans/000_foundations.md` §"Interfaces and contracts". Shell-level differentiation between usage and general errors is lost.
- **G6 — `find_repo` accepts a regular file named `.chrona` as a repo root.** Uses `std::filesystem::exists` instead of `is_directory`. A leftover dotfile or typo named `.chrona` somewhere up the tree triggers a false-positive repo root.

**Medium (design smells and latent issues):**

- **G7 — `parse_args` rejects any multi-argument command** via `argc > 2`. Milestone 1's `chrona hash-object <file>` cannot be added without rewriting this. The test suite locks in this behaviour with an explicit comment ("This might be a bug").
- **G8 — `ParseResult.args` is declared but never populated.** Every return uses `{}` for it; a field that documents intent without delivering it.
- **G9 — `create_error` returns `std::optional<Error>` that is never `nullopt`.** Every caller has to `.value()` for no safety benefit. Either tighten the signature or give it a real failure mode.
- **G10 — No `target_include_directories` on the main target.** Builds today via implicit per-source include paths; fragile if structure changes.
- **G11 — No shared library target; sources compiled twice** by `chrona` and `chrona_tests`. Negligible at ~2KLOC, measurable at 20KLOC+.
- **G12 — `chrona_tests` CTest registration is single-line** (no per-test breakdown via `catch_discover_tests`).

**Low (cosmetic / hygiene):**

- **G13** — Help text advertises `https://chrona.com` that Caner does not own.
- **G14** — Help text lists a `help` command; `chrona help` errors with "Unknown command: help".
- **G15** — `*.lib` duplicated in `.gitignore`.
- **G16** — No `.clang-format`, `.clang-tidy`, or `.editorconfig`.
- **G17** — No CI configured (no `.github/workflows/`).
- **G18** — No dependency policy document beyond a one-line note (`plans/000_foundations.md` Step 2 unchecked).

**Unverified (LifeOS UNKNOWN):**

- **U1** — "No runtime deps" claim in `plans/ARCHITECTURE.md` cross-checked against `CMakeLists.txt`: no `target_link_libraries` on `chrona`; apparently accurate.
- **U2** — README claims Windows/Linux support but there is no CI, no commit trail of cross-platform fixes; `test_repo.cpp` has Windows-specific `C:\\` handling suggesting intent, but actual cross-platform validation has not happened.

## Direction (in-flight, not wishlist)

There is no in-flight work — the project has been dormant since 2025-12-27. LifeOS Roadmap.md describes what *resumption* would look like, not what is currently being worked on. For completeness, the resumption sequence LifeOS predicts (one weekend of work to take the project from 2% to ~15% of the README roadmap):

1. Close G1 — implement `init` (one afternoon).
2. Close G3 — re-enable `test_errors.cpp` and rewrite `test_repo.cpp` from debug scaffold into real test matrix (one evening).
3. Open `plans/001_object_store.md` using the `000_foundations.md` schema.
4. Pick a hash function — Milestone 1 has open decisions on SHA-1 (Git-parity), SHA-256 (Git's new default), or BLAKE3 (modern, fast); also storage layout (sharded vs flat), compression (zlib vs none while learning), and canonical encoding format.
5. Build `chrona hash-object <file>` to return a hash to stdout.
6. Build `chrona cat-object <hash>` to read it back.

This is forecast, not commitment. There is no source evidence in LifeOS that this work is scheduled or started.

## Demonstrated skills

What the LifeOS source supports as actually demonstrated, separate from what the README aspires to demonstrate:

- **Modern C++20 idioms in a small but disciplined codebase** — `std::filesystem::path` for the repo walk-up, `std::optional` for the result type, `enum class` for both `Command`/`ParseAction` and the error-layer types, flat-namespace single-translation-unit composition. The code is idiomatic; the surface area is small.
- **CMake build authoring with `FetchContent`** — two-target build, per-target language standard via `target_compile_features`, per-platform warning flags (`/W4` vs `-Wall -Wextra -Wpedantic`), `FetchContent_Declare` + `FetchContent_MakeAvailable` for Catch2, `enable_testing()` + `add_test` for CTest integration.
- **Catch2 v3 testing** — 11 working `TEST_CASE`s exhaustively covering the CLI parse-rule matrix; tests use the const-correctness workaround pattern (`const_cast<char**>`) and Catch2's `ostringstream` capture for output assertions. LifeOS notes the parser-test matrix as exhaustive — it covers normal cases, error cases, edge cases (empty string, wrong case, single-flag invocations), and the rejection-of-extra-args contract.
- **Value-style error-model design in C++ without exceptions** — pairing process-level `ExitCode` with semantic `ErrorCode` in a `struct Error`, returning `std::optional<Error>` from `create_error`. Decision D8 traces the alternatives (`throw`/`try`/`catch`, `std::expected` for C++23) and the reasoning for the chosen path. The defects in the model (always-engaged optional, dropped `exit_code` in `exit_with_error`, hardcoded exit `1` in `main.cpp`) are honestly captured in LifeOS — the design intent is sound, the implementation has known bugs.
- **Filesystem-traversal algorithm implementation** — `find_repo`'s walk-up uses `std::filesystem::path::parent_path()` idempotence at filesystem roots as the termination invariant, mirroring Git's `.git/` sentinel approach. The `exists` vs `is_directory` bug is captured but the algorithm itself is correct for the common case.
- **Externalised planning discipline applied to a small repo** — the `plans/NNN_topic.md` schema with eleven required sections per plan (Goal, Context, Rejected alternatives, Assumptions, Interfaces, Impacted areas, Incremental implementation, Testing, Risks, Exit criteria, Future considerations) is unusual for a ~2KLOC personal project and reflects RFC-style methodology. The Decisions.md surface in LifeOS is dense with explicit alternatives and rejection reasoning that the code alone could not reveal.
- **Honest scope-vs-reality reporting at the project level** — LifeOS itself documents the gap between the README's nine-milestone pitch and the actual ~2% implementation without softening, with severity-ranked gap tracking (Critical/High/Medium/Low/Unknown), commit-velocity analysis (7 commits in one week, then 16+ weeks dormant), and explicit anti-puffing language. Whether this counts as a code-level signal or a vault-level signal, the engineer reading their own work clearly distinguishes design intent from shipped code.

Not demonstrated (despite README claims): content-addressed object storage, hashing, tree/snapshot encoding, commit DAG construction, ref management, staging-area model, line-based diff. These are roadmap, not demonstrated skill.

---

## Evidence Block

| Path | Verbatim last line |
|---|---|
| Projects/Chrona/_Overview.md | "Chrona is a correctly-scoped, well-planned, earnestly-scaffolded C++ VCS learning project whose README describes a finished engine and whose code describes a half-finished `argc/argv` parser — the gap between the two is the entire point of this note existing." |
| Projects/Chrona/Architecture.md | "\| No static-analysis config \| No `.clang-tidy`, no IWYU mapping \|" |
| Projects/Chrona/Decisions.md | "- [[Chrona/Roadmap]] — D12's exclusions bound the roadmap" |
| Projects/Chrona/Gaps.md | "- [[Chrona/Roadmap]] — what depends on each of these being closed" |
| Projects/Chrona/Plans Workflow.md | "- [[LifeOS/_Overview]] — LifeOS uses a similar externalised-state discipline at a larger scale; plans/ is the project-local equivalent" |
| Projects/Chrona/Roadmap.md | "- [[Chrona/Plans Workflow]] — the plans/ convention that future milestone plans will follow" |
| Projects/Chrona/Systems/_Overview.md | "- [[Projects/Chrona/Roadmap]] — direction-of-travel" |
| Projects/Chrona/Systems/Build and Test.md | "- [[Chrona/Gaps]] — the commented-out tests, the missing library target, the missing `target_include_directories` on the main target" |
| Projects/Chrona/Systems/CLI.md | "- [[Chrona/Gaps]] — the init-stub gap originates here" |
| Projects/Chrona/Systems/Errors.md | "- [[Chrona/Architecture]] — compile graph showing errors is linked into both targets" |
| Projects/Chrona/Systems/Repo Discovery.md | "- [[Chrona/Gaps]] — the `exists` vs `is_directory` gap and the commented-out test file are recorded there" |
