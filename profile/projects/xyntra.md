---
name: Xyntra
status: dormant
source_repo: https://github.com/Capataina/Xyntra
lifeos_folder: Projects/Xyntra
last_synced: 2026-04-29
sources_read: 13
---

# Xyntra

> [!note] Status note — schema enum mapping
> LifeOS records this project as `skeleton`. Mapped to schema-conforming `dormant` for the frontmatter (last commit 2025-07-10; ~9 months dormant at LifeOS verification).

## One-line summary

Early-stage Rust ML graph-fusion compiler skeleton — typed IR (`NodeID` / `TensorShape` / `OpKind` / `Graph` / `Node`), a four-category error taxonomy, a config struct with GPU-parameter validation, and a graph validator scaffold (every public method `todo!()`); README pitches an automatic kernel-fusion compiler emitting WGSL or PTX from ONNX/TorchScript graphs but no compilation, parsing, fusion, or codegen feature is implemented.

## What it is

Xyntra is an early-stage Rust project that aspires to become an automatic kernel-fusion compiler for deep-learning graphs. In its current form (commit `c639b24`, last touched 2025-07-10) it is a small typed IR crate with tests. The README's GitHub description — *"Automatic kernel-fusion compiler pass…ONNX/TorchScript…wgpu or PTX…autotuning harness, occupancy tracing"* — is roadmap, not status. Treat the README as a roadmap, not a description. Total size ~39KB across 17 files; ~11.8KB of source across 9 `.rs` files; ~22.3KB of tests across 4 `.rs` files (test:source ratio ~1.9:1 by bytes); 27 `#[test]` functions. The repo has not been touched in ~9 months as of LifeOS verification.

## Architecture

Single binary+lib crate named `xyntra` (Rust 2024 edition). Zero external dependencies — `Cargo.toml`'s `[dependencies]` is empty.

```
xyntra/
├── src/
│   ├── main.rs              # 3 lines — println!("Hello, world!");
│   ├── lib.rs               # Public surface
│   ├── ir/
│   │   ├── types.rs         # NodeID, TensorShape, OpKind primitives
│   │   ├── graph.rs         # Graph container + Node struct, HashMap<u32, Node>
│   │   ├── ops.rs           # Op definitions
│   │   ├── errors.rs        # 4-category error taxonomy
│   │   └── validation.rs    # GraphValidator scaffold (every public method todo!())
│   └── config/
│       └── mod.rs           # XyntraConfig (lacks pub — visibility bug)
└── tests/
    ├── test_types.rs        # 7 tests
    ├── test_graph.rs        # 10 tests including 1000-node stress test
    ├── test_ops.rs          # 9 tests
    └── helpers.rs           # 1 shared helper
```

The README claims a workspace split into `xyntra-core`, `xyntra-cli`, `xyntra-ir` — this does not exist; it is a single crate.

## Subsystems and components

| Subsystem | Responsibility | State |
|---|---|---|
| **IR Types** | `NodeID` (newtype), `TensorShape`, `OpKind` enum primitives | Working, 7 tests |
| **Graph** | `Graph` container with `HashMap<NodeID, Node>` storage, sequential `u32` IDs | Working, 10 tests including 1000-node stress test |
| **Ops** | `Op` definitions used by `Node` | Working, 9 tests |
| **Errors** | 4 top-level categories (Validation / Parsing / System / Internal); only `ValidationError` has a `Display` impl | Defined; partial Display |
| **Config** | `XyntraConfig` with `BackendType` enum (`Wgsl`, `CudaPtx`); validates `tile_size` / `block_size` (power-of-2 + bounds), `optimisation_level` ≤ 3, input file existence, output directory readability | Working but `XyntraConfig` lacks `pub` so external consumption blocked |
| **Validation** | `GraphValidator`, `ValidationContext`, `ValidationResult` types defined; every public validation method is `todo!()` | Scaffold only |
| **ONNX parser** | Consume ONNX graph | Not started — only `ParsingError::MalformedOnnx` enum variant exists |
| **TorchScript loader** | Load TorchScript IR | Not started |
| **e-graph fusion** | Pattern-rewriting via egg crate | Not started — no `egg` dependency |
| **WGSL codegen** | Emit WGSL kernels | Not started — only `BackendType::Wgsl` variant exists |
| **PTX codegen** | Emit CUDA PTX | Not started — only `BackendType::CudaPtx` variant exists |
| **Autotuner / occupancy tracing** | GPU performance tuning | Not started |
| **CLI** | Entry binary | `Hello, world!` only — no `clap` dependency, no argv parsing |

## Technologies and concepts demonstrated

### Languages
- **Rust 2024** — entire codebase, ~11.8KB across 9 files; zero external dependencies.

### Domains and concepts
- **Compiler IR design** — typed `NodeID` / `TensorShape` / `OpKind` primitives with bounds-checking; sequential `u32` IDs; HashMap-backed node store with stress-test coverage to 1000 nodes.
- **Error taxonomy design** — 4 top-level categories (Validation / Parsing / System / Internal); intent of separating user-input errors from internal compiler bugs.
- **GPU compilation parameters** — `tile_size`, `block_size`, `optimisation_level`, `BackendType` enum (Wgsl, CudaPtx) — config-level demonstration of GPU-codegen domain knowledge.
- **Test:source ratio discipline** — ~1.9:1 by bytes; tests are heavier than source for the implemented surface.
- **Heavy-test foundational primitives** — even though the surface area is tiny, the IR primitives are well-tested before any pipeline stage is built.

## Key technical decisions

- **Safe Rust, zero `unsafe`.**
- **Four error categories: Validation / Parsing / System / Internal** — separates user-input errors from internal bugs.
- **HashMap-backed node store with sequential `u32` IDs** — chosen for simplicity over arena-based ID allocation.
- **Test the IR primitives heavily before any pipeline stage** — 27 tests for `Graph` + `Node` + `Op` + `OpKind` before any compilation logic.
- **`tile_size` / `block_size` validated as power-of-2** — codifies GPU-tuning constraint at config level.
- **No external dependencies** — even Catch2 / Catch are absent; using `cargo test`'s built-in framework.

## What is currently built

- 10 commits, active dev window 2025-07-05 → 2025-07-10 (6 days).
- 1 binary+lib crate (single crate, not the workspace split the README claims).
- 4 IR primitives (`NodeID`, `TensorShape`, `OpKind`, `Op`) tested.
- `Graph` container with HashMap<u32, Node> storage, 1000-node stress test passing.
- 4-category error taxonomy with partial Display.
- `XyntraConfig` validating power-of-2 tile/block, optimisation level, paths.
- `GraphValidator` scaffold with `todo!()` body.
- 27 tests passing.
- `main.rs` is `Hello, world!`.

## Current state

Dormant. Last commit 2025-07-10; ~9 months without activity at LifeOS verification.

## Gaps and known limitations

- **`XyntraConfig` lacks `pub`** — nothing outside the crate can use it. Compile-blocking for external consumption.
- **`GraphValidator` is `todo!()`** — every public validation method panics on call.
- **No ONNX parser** — only `MalformedOnnx` enum variant.
- **No TorchScript loader.**
- **No e-graph fusion** — no `egg` dependency, no rewrite rules.
- **No WGSL codegen** — only enum variant.
- **No PTX codegen** — only enum variant.
- **No autotuner, no occupancy tracing.**
- **No CLI** — `Hello, world!` only.
- **README workspace split** (`xyntra-core` / `xyntra-cli` / `xyntra-ir`) does not exist.
- **Zero tests for config validation, error Display, validator** — the tested surface is the IR primitives only.

## Direction (in-flight, not wishlist)

Dormant. If revived: fix the `XyntraConfig` visibility bug, implement `GraphValidator` (replace `todo!()`s), add an ONNX parser pass to land the first end-to-end "ONNX → IR" pipeline stage. Real progress requires reaching for the `egg` crate or a similar e-graph library — current empty `[dependencies]` posture is incompatible with the stated ambition.

## Demonstrated skills

- **Compiler IR primitive design in Rust** — typed `NodeID` / `TensorShape` / `OpKind` with bounds-checking, HashMap-backed node store, 1000-node stress test.
- **Error taxonomy design** — 4-category separation of user-input vs internal errors.
- **Test-heavy foundational discipline** — 1.9:1 test-to-source ratio on the implemented surface; primitives well-tested before pipeline stages.
- **GPU compilation domain awareness** — `tile_size` / `block_size` / `optimisation_level` / `BackendType` (Wgsl, CudaPtx) at config level.
- **Anti-puffing discipline** — `Reality vs README.md` LifeOS file explicitly reconciles README claims against code; the project's vault notes refuse to inflate the implementation past the IR layer.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Xyntra/_Overview.md | 104 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
| Projects/Xyntra/Architecture.md | 153 | "`#project/xyntra` `#rust` `#architecture` `#compiler-frontend` `#ir`" |
| Projects/Xyntra/Decisions.md | 228 | "`#project/xyntra` `#decisions` `#design-rationale`" |
| Projects/Xyntra/Gaps.md | 140 | "`#project/xyntra` `#gaps` `#unimplemented` `#bugs`" |
| Projects/Xyntra/Reality vs README.md | 68 | "`#project/xyntra` `#reality-check` `#anti-puffing` `#portfolio-accuracy`" |
| Projects/Xyntra/Roadmap.md | 147 | "`#project/xyntra` `#roadmap` `#phased-plan`" |
| Projects/Xyntra/Systems/_Overview.md | 42 | "- [[Projects/Xyntra/Roadmap]] — direction-of-travel" |
| Projects/Xyntra/Systems/Config.md | 105 | "`#project/xyntra` `#config` `#gpu-parameters` `#validation`" |
| Projects/Xyntra/Systems/Errors.md | 111 | "`#project/xyntra` `#errors` `#error-handling` `#rust`" |
| Projects/Xyntra/Systems/Graph.md | 129 | "`#project/xyntra` `#ir` `#graph` `#dag`" |
| Projects/Xyntra/Systems/IR Types.md | 109 | "`#project/xyntra` `#ir` `#types` `#primitives`" |
| Projects/Xyntra/Systems/Testing.md | 85 | "`#project/xyntra` `#testing` `#test-infrastructure`" |
| Projects/Xyntra/Systems/Validation.md | 112 | "`#project/xyntra` `#validation` `#scaffold` `#todo` `#next-work`" |
