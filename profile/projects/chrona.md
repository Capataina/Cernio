---
name: Chrona
status: paused
source_repo: https://github.com/Capataina/Chrona
lifeos_folder: Projects/Chrona
last_synced: 2026-04-29
sources_read: 11
---

# Chrona

> [!note] Status note — schema enum mapping
> LifeOS records this project as `foundational — Milestone 0 partially complete, VCS core unbuilt`. Mapped to schema-conforming `paused` for the frontmatter (the repo has been dormant for ~16 weeks at LifeOS verification — last commit 2025-12-27).

## One-line summary

C++20 from-scratch Git-inspired version control system at the scaffold stage — CMake build with FetchContent Catch2, minimal CLI parser (recognises `init` literal), implemented repo-discovery walk-up via `std::filesystem`, an `ExitCode`/`ErrorCode`/`Error` model with `std::optional`-based propagation — but no object store, no commit DAG, no staging, no diff, no refs.

## What it is

Chrona is a deliberately-scoped personal learning project in modern C++ (C++20) aimed at rebuilding the "inner engine" of Git from first principles. Caner's framing in the README draws a sharp line between Git (the VCS engine + data model) and GitHub/GitLab/Codeberg (hosting platforms layered on top) — the project exists to make that distinction concrete by implementing the engine rather than the hosting side. The repo is public on GitHub at `Capataina/Chrona`, created 2025-12-23, last touched 2025-12-27. 7 commits total. ~20.8KB of C++ across 7 source files; ~15.9KB of C++ tests; ~11.5KB of markdown planning documents. The README pitches a full Git-inspired local VCS with content-addressed object storage, commit DAGs, staging, and diffs. **None of that is implemented**. The repo today is a CMake + CLI + error-model + repo-discovery scaffold that does not yet create, store, hash, or version anything.

## Architecture

```
chrona/
├── CMakeLists.txt          # CMake 3.x, C++20, FetchContent Catch2 v3.5.0
├── src/
│   ├── main.cpp            # CLI entry — dispatches to Command::Init (TODO body)
│   ├── cli/
│   │   └── cli.cpp         # argc/argv parser, --help, init recognition
│   ├── repo/
│   │   └── repo.cpp        # find_repo() walks parent paths looking for .chrona/
│   └── errors/
│       ├── error.hpp       # ExitCode / ErrorCode / Error
│       └── error.cpp       # create_error / print_error / exit_with_error
├── tests/
│   ├── test_cli.cpp        # Wired into chrona_tests target
│   ├── test_errors.cpp     # Commented out of build
│   └── test_repo.cpp       # Commented out of build
└── plans/                  # NNN_topic.md plan files (workflow convention)
```

Two CMake targets: `chrona` (main executable) and `chrona_tests` (Catch2 v3.5.0). The repo-discovery library walks parent paths via `std::filesystem`; the CLI dispatches `init` to a Command enum but the body is a `// TODO`.

## Subsystems and components

| Subsystem | Responsibility | State |
|---|---|---|
| **Build and Test** | CMake C++20 build with FetchContent Catch2 v3.5.0, `chrona` + `chrona_tests` targets, `-Wall -Wextra -Wpedantic` | Working — but two of three test files commented out of build target |
| **CLI** | Minimal argc/argv parser; `--help` and `init` literally; `argc > 2` rejected as "Too many arguments"; unknown command errors | Working but `init` body is `// TODO` |
| **Repo Discovery** | `find_repo()` walks parent paths, stops when `parent == current` | Working — but not wired to any command |
| **Errors** | `ExitCode` / `ErrorCode` enums, `Error` struct, `create_error` / `print_error` / `exit_with_error`, `std::optional<T>` convention | Working |
| **Object store (blobs)** | content-addressed storage | Not started |
| **Trees / snapshots** | recursive trees | Not started |
| **Commits / DAG** | commit objects + parent links | Not started |
| **Refs / HEAD / branches** | symbolic refs | Not started |
| **Staging / index** | working area | Not started |
| **Diff** | tree comparison | Not started |

## Technologies and concepts demonstrated

### Languages
- **C++20** — entire codebase, idiomatic modern C++ (`std::filesystem`, `std::optional`, enum classes, namespaces).

### Tools
- **CMake** — build system with FetchContent for Catch2.
- **Catch2 v3.5.0** — testing framework.

### Domains and concepts
- **Version control system internals** — content-addressed object storage design intent (from README); commit DAG; immutable snapshots; staging semantics.
- **Modern C++ idioms** — `std::filesystem` for path walking, `std::optional<T>` for fallible operations instead of error codes mixed with valid returns, enum classes for type-safe error categories, namespaces for module boundaries.
- **Plans-driven development** — `plans/NNN_topic.md` schema with explicit status enum + per-plan checklists; same vault-like discipline as Cernio + Vynapse + LifeOS.
- **Build hygiene** — `-Wall -Wextra -Wpedantic` from day one decision; FetchContent-based dependency policy.

## Key technical decisions

- **Local-first, stdlib-first** — Git is the comparator; no external libraries beyond Catch2 for tests.
- **Milestone-gating** — 9-milestone roadmap with explicit scope exclusions; clear README boundaries between VCS engine and hosting platforms.
- **Manual argc/argv** instead of a CLI library — deliberately implements the parser rather than reaching for `argparse`/`CLI11`.
- **`std::optional<T>` convention for fallible operations** — instead of error code + out-parameter or exception throwing.
- **Plans directory convention** — `plans/NNN_topic.md` schema as the externalised state, mirroring LifeOS's plan-as-SSOT philosophy.
- **`-Wall -Wextra -Wpedantic` from day one** — zero-warning posture for any commit.

## What is currently built

- 7 commits total; 7 C++ source files, 3 test files (1 wired into the build).
- Working CMake build with two targets.
- Working CLI parser for `--help` and `init` literal (init body is `// TODO`).
- Working `find_repo()` walk-up.
- Working error model with `ExitCode` / `ErrorCode` / `Error` types.
- 5 documented executable behaviours (no-args usage, --help, init no-op, unknown command error, too-many-args error).

## Current state

Paused. Repo dormant for ~16 weeks at LifeOS verification (last commit `652fb7f` on 2025-12-27). No `.chrona/` is ever created at runtime; `find_repo()` exists in the library but is not wired to any command. The project is at ~2% of its stated README scope.

## Gaps and known limitations

- **`chrona init` is a stub** — `// TODO: Implement init command` and `return 0`.
- **No object store** — no hashing, no `.chrona/objects/`, no I/O code.
- **No trees, no commits, no DAG, no refs, no HEAD, no branches, no staging, no diff** — every substantive VCS feature is unbuilt.
- **Two of three test files commented out of build target** — `test_errors.cpp` and `test_repo.cpp`.
- **`find_repo()` not wired into any command path.**
- **`exists` vs `is_directory` gap in Repo Discovery** — flagged in the systems gap.
- **No `.clang-tidy`, no IWYU mapping** — flagged in Architecture's static-analysis-config gap.

## Direction (in-flight, not wishlist)

Paused. If revived: complete Milestone 0 (wire all three test files, implement `init`, define dependency policy beyond the one-line note), then attempt Milestone 1 (object store: hash + write a blob to `.chrona/objects/`).

## Demonstrated skills

- **Modern C++20 idioms** — `std::filesystem`, `std::optional<T>`, enum classes, namespace-based module boundaries.
- **CMake + Catch2 build wiring** — FetchContent-based dependency policy, two-target structure, explicit `-Wall -Wextra -Wpedantic`.
- **Hand-rolled argc/argv parsing** — without reaching for a CLI library.
- **Error-handling discipline** — `std::optional<T>` convention; `ExitCode`/`ErrorCode`/`Error` taxonomy.
- **Plans-driven workflow discipline** — `plans/NNN_topic.md` schema mirroring LifeOS / Cernio / Vynapse.
- **VCS engine design awareness** — content-addressed storage, commit DAG, staging, refs (design intent visible in README + Roadmap, even though not implemented).

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Chrona/_Overview.md | 82 | "Chrona is a correctly-scoped, well-planned, earnestly-scaffolded C++ VCS learning project whose README describes a finished engine and whose code describes a half-finished `argc/argv` parser — the gap between the two is the entire point of this note existing." |
| Projects/Chrona/Architecture.md | 159 | "\| No static-analysis config \| No `.clang-tidy`, no IWYU mapping \|" |
| Projects/Chrona/Decisions.md | 226 | "- [[Chrona/Roadmap]] — D12's exclusions bound the roadmap" |
| Projects/Chrona/Gaps.md | 276 | "- [[Chrona/Roadmap]] — what depends on each of these being closed" |
| Projects/Chrona/Plans Workflow.md | 117 | "- [[LifeOS/_Overview]] — LifeOS uses a similar externalised-state discipline at vault scale" |
| Projects/Chrona/Roadmap.md | 225 | "- [[Chrona/Plans Workflow]] — the plans/ convention that future milestone plans will use" |
| Projects/Chrona/Systems/_Overview.md | 39 | "- [[Projects/Chrona/Roadmap]] — direction-of-travel" |
| Projects/Chrona/Systems/Build and Test.md | 203 | "- [[Chrona/Gaps]] — the commented-out tests, the missing library target, the missing dependency policy" |
| Projects/Chrona/Systems/CLI.md | 118 | "- [[Chrona/Gaps]] — the init-stub gap originates here" |
| Projects/Chrona/Systems/Errors.md | 123 | "- [[Chrona/Architecture]] — compile graph showing errors is linked into both targets" |
| Projects/Chrona/Systems/Repo Discovery.md | 129 | "- [[Chrona/Gaps]] — the `exists` vs `is_directory` gap and the commented-out tests" |
