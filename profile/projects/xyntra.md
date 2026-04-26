---
name: Xyntra
status: dormant
source_repo: https://github.com/Capataina/Xyntra
lifeos_folder: Projects/Xyntra
last_synced: 2026-04-26
sources_read: 12
---

# Xyntra

## One-line summary

Early-stage Rust crate that defines a typed IR for an aspirational ML kernel-fusion compiler — currently a foundational types layer with a stub graph validator, no parsing, no codegen, no fusion.

## What it is

Xyntra is positioned in its README as an automatic kernel-fusion compiler pass for deep-learning graphs that ingests ONNX/TorchScript and emits one fused GPU kernel through `wgpu` (WGSL) or CUDA PTX, with autotuning and occupancy tracing, in 100% safe Rust. The actual repository at commit `c639b24` (last touched 2025-07-10) is a single Rust crate containing a typed IR (`NodeID`, `TensorShape`, `OpKind`), a `HashMap`-backed `Graph` container, a four-category error hierarchy, a `XyntraConfig` struct with GPU-parameter validation, and a scaffolded `GraphValidator` whose four methods are all `todo!()`. LifeOS classifies this honestly as a "skeleton, not a compiler" and estimates roughly 5% of the README's described pipeline is implemented. The project is dormant — no commits in approximately nine months at extraction time — and the entire active development window was a 6-day sprint between 2025-07-05 and 2025-07-10. The IR layer that does exist is well-tested (26 `#[test]` functions, including a 1000-node stress test) and structurally sound; everything downstream of the IR is unstarted.

## Architecture

Xyntra is a single Rust 2024-edition crate (`name = "xyntra"`, `version = "0.1.1"`) with binary + library targets and an empty `[dependencies]` section — standard library only. The crate is not a workspace despite the README claiming a `xyntra-core` / `xyntra-cli` / `xyntra-ir` split.

```
xyntra/                    (single crate, binary + library)
├── Cargo.toml             (edition = "2024", no external deps)
├── src/
│   ├── lib.rs             (`pub mod config; pub mod ir;`)
│   ├── main.rs            (3 lines: `pub mod ir;` + Hello world)
│   ├── config/
│   │   └── mod.rs         (XyntraConfig, BackendType, validate())
│   └── ir/
│       ├── mod.rs         (re-exports errors, graph, ops, types, validation)
│       ├── types.rs       (NodeID, TensorShape, OpKind)
│       ├── ops.rs         (Node struct + accessors)
│       ├── graph.rs       (Graph — HashMap<NodeID, Node>)
│       ├── validation.rs  (GraphValidator — all todo!())
│       └── errors.rs      (XyntraError + 4 sub-enums)
└── tests/
    ├── common.rs          (test helpers + builders)
    ├── test_types.rs      (7 tests)
    ├── test_graph.rs      (10 tests)
    └── test_ops.rs        (9 tests)
```

Dependency direction is acyclic and flows downward to leaf modules:

```
   config::XyntraConfig ─── uses ──> ir::errors
   ir::validation       ─── uses ──> ir::graph, ir::errors, ir::types
   ir::graph            ─── uses ──> ir::ops, ir::types
   ir::ops              ─── uses ──> ir::types
   ir::types            ─── leaf (std only)
   ir::errors           ─── leaf (core::fmt only)
```

LifeOS Architecture.md flags two structural anti-patterns: (1) `main.rs` re-declares `pub mod ir;` independently of `lib.rs`, producing two distinct `ir` module trees in the binary's compile unit; (2) `struct XyntraConfig` is module-private (missing `pub`) so `lib.rs`'s `pub mod config;` publishes a module exposing no usable types — external callers cannot construct a config today.

## Subsystems and components

### IR Types (`src/ir/types.rs`)

The most complete layer in the project. Contains three primitive types:

- **`NodeID(u32)`** — `Copy` newtype around `u32`, derives `Debug, Clone, Copy, PartialEq, Eq, Hash`. Used as a stable identifier and as a `HashMap` key.
- **`TensorShape(Vec<usize>)`** — newtype around `Vec<usize>`, derives `Debug, Clone, PartialEq, Eq, Hash`. Provides `rank()`, `size()` (product of dimensions, defaults to 1 for scalar), `is_scalar()`. No dtype field.
- **`OpKind`** — enum with 7 variants: `MatMul`, `Add`, `Gelu`, `Dropout`, `Softmax`, `LayerNorm`, `Custom(String)`. Vocabulary skews to transformer FFN/attention layers (no Conv, no Pool, no BatchNorm). Notably does **not** derive `PartialEq`, forcing every test that checks an op kind to use `match` rather than `assert_eq!` — flagged in LifeOS as likely an oversight rather than a deliberate design.

### Graph (`src/ir/graph.rs`, `src/ir/ops.rs`)

`Graph` is a `HashMap<NodeID, Node>` with a monotonic `next_id: u32` counter. Public API: `Graph::new`, `Graph::add_node(OpKind, Vec<NodeID>, Vec<NodeID>) -> NodeID`, `Graph::get_node(NodeID) -> Option<&Node>`. There is no iteration method (`Graph::nodes` is private), which is a blocker for the validator. `Node` carries `id`, `op`, `inputs: Vec<NodeID>`, `outputs: Vec<NodeID>` — `Node` fields are `pub` even though accessor methods exist (redundant surface). The `outputs` field is dead state today: every test passes `vec![]` and no code reads or maintains it. The graph encodes a DAG via `inputs` only — SSA-style, requires a scan to find consumers of a node. None of the structural invariants (input references exist, no self-loops, acyclicity, output consistency) are enforced by `add_node`; they are intended to be caught by the (unimplemented) validator.

### Errors (`src/ir/errors.rs`)

Four-category hierarchy: `XyntraError` enum wraps `Validation(ValidationError)`, `Parsing(ParsingError)`, `System(SystemError)`, `Internal(InternalError)`. The "recoverable vs fatal" split between the first two and the last two exists only as a code comment — there is no type-level enforcement. Variant counts: `ValidationError` has 9 (`InvalidTensorShape`, `IncompatibleShapes`, `InvalidNodeConnection`, `CyclicGraph`, `MissingNode`, `InvalidOpInputCount`, `InvalidConfigValue`, `InvalidFilePath`, `InvalidGPUParameter`); `ParsingError` 5; `SystemError` 4; `InternalError` 4. Only `ValidationError` implements `fmt::Display`; `XyntraError` itself has no `Display` impl, and `std::error::Error` is not implemented anywhere. Only six `ValidationError` variants are actually constructed in live code — all in `config::XyntraConfig::validate`. The remaining variants exist as scaffolding for the future pipeline (parsing, validation, codegen).

### Config (`src/config/mod.rs`)

The only place in the codebase that returns a real `Result<_, XyntraError>` from live logic. `XyntraConfig` carries: `input_file: Option<PathBuf>`, `output_dir: PathBuf`, `backend: BackendType` (`Wgsl` default, or `CudaPtx`), `optimisation_level: u8` (default 2), `tile_size: usize` (default 16, range [4, 64], must be power of 2), `block_size: usize` (default 256, range [64, 1024], must be power of 2), `enable_debug: bool`, `export_ir: bool`. `validate()` enforces seven branches: power-of-2 + bounds for `tile_size` and `block_size`, `optimisation_level ≤ 3`, `input_file` existence, `output_dir` readability via `std::fs::metadata`. The struct itself is not `pub`, which makes it unusable externally. There is also a British/American naming inconsistency: the field is `optimisation_level` but the error string says `"optimization_level"`.

### Validation (`src/ir/validation.rs`)

Pure scaffold. Defines `pub struct GraphValidator<'a> { graph: &'a Graph }`, `struct ValidationContext { current_node: Option<NodeID> }`, and `pub type ValidationResult = Result<(), Vec<ValidationError>>`. The four public methods (`validate_node_references`, `detect_cycles`, `validate_operation_constraints`, `validate`) all panic with `todo!()`. Only `combine_results` (a free helper that flattens `Err` vectors) contains real logic. The most recent commit (`c639b24`, 2025-07-10) is titled "started on graph validator" — this is exactly where active work stopped. Implementing this subsystem would also require: adding `Graph::iter_nodes`, adding a shape field to `Node` (for shape-aware validation), and a static `OpKind → expected_input_count` table.

### Testing (`tests/`)

Total: 26 `#[test]` functions across 4 files, 730 lines, with a test:source ratio of approximately 1.9:1 by bytes. `tests/common.rs` holds shared fixtures (`build_simple_graph`, `build_complex_graph`, `create_all_op_kinds`, assertion helpers) imported via `mod common;` in each test file (flat layout rather than the conventional `tests/common/mod.rs` submodule). Coverage is concentrated on the IR types and graph: `test_types.rs` (7 tests including scalar/4D/zero-dim/ImageNet shapes), `test_graph.rs` (10 tests including a 1000-node linear-chain stress test), `test_ops.rs` (9 tests). `config/mod.rs` has zero tests despite containing seven validation branches; `errors.rs` has zero tests despite the 9-arm `Display` match. No `proptest`, no `quickcheck`, no `cargo-fuzz`, no `criterion` benchmark harness, no CI / GitHub Actions.

## Technologies and concepts demonstrated

### Languages
- **Rust (2024 edition)** — entire codebase. Uses newtype patterns, ownership for graph mutation, lifetime parameters in `GraphValidator<'a>`, derived traits for hash/equality on IR primitives, idiomatic `Result<_, _>` returns. Verified empty `[dependencies]` — standard-library-only Rust.

### Frameworks and libraries
- None. The crate has no external dependencies. The README aspirationally lists `egg`, `wgpu`, `clap`, `tch-rs`, `serde`, `toml`, but `Cargo.toml` confirms none are present.

### Runtimes / engines / platforms
- No source evidence in LifeOS. The `BackendType` enum lists `Wgsl` and `CudaPtx` variants but no WGSL or PTX code exists.

### Tools
- `cargo` build system implied via `Cargo.toml`. No CI, no profiler config, no linter config (no `clippy.toml`, no `cargo-deny`), no pre-commit, no `.github/workflows/`.

### Domains and concepts
- **Compiler frontend / IR design** — typed IR with newtype primitives (`NodeID`, `TensorShape`, `OpKind`), `HashMap`-backed graph store, monotonic ID allocation, dependency direction sketched for a parse → validate → fuse → codegen pipeline.
- **Graph validation algorithms (planned, not implemented)** — the `validation.rs` scaffold targets node-reference checking, cycle detection (white/grey/black DFS as the LifeOS-recommended algorithm), operation-constraint checking via per-op input arity. All `todo!()`.
- **Error taxonomy design** — explicit four-category split with documented (not enforced) recoverable/fatal classification; variant set chosen ahead of callers to anticipate the full compiler pipeline.
- **GPU kernel-launch parameters (validation only)** — `tile_size` and `block_size` validation reflects familiarity with CUDA defaults: 16×16 = 256 threads per tile, 256 = 8 warps of 32, 1024 mirroring `maxThreadsPerBlock`. No GPU code exists.
- **Transformer-block op vocabulary** — the `OpKind` set (MatMul, Add, Gelu, Dropout, Softmax, LayerNorm, Custom) is exactly the FFN/attention vocabulary, signalling LLM/attention as the implicit target workload over CNNs.
- **Test-first or test-heavy practice** — 26 tests including a 1000-node stress test, with fixture builders for canonical graph shapes (input → matmul → output, matmul → gelu → dropout chain). LifeOS infers AI-assisted test generation from the stylistic uniformity and bulk landing in a single commit.

## Key technical decisions

LifeOS Decisions.md captures eleven decisions; the load-bearing ones for portfolio signal:

- **D1. Safe Rust only, no `unsafe`.** README explicitly states "100% `unsafe`-free". Trivially satisfied today (no GPU, no FFI). Sustainable for a `wgpu`-based future backend (whose public API is safe), likely impossible for a PTX-via-NVIDIA-Driver-API backend without an FFI module.
- **D2. Rust 2024 edition.** Cutting-edge positioning; requires rustc 1.85+. None of the new features (`async fn` in traits, improved RPITIT) are actually used.
- **D3. Four-category error split (`Validation`/`Parsing`/`System`/`Internal`)** rather than flat enum or per-subsystem error types. Reflects compiler-pass lifecycle. Recoverable/fatal split is comment-only, not type-enforced.
- **D4. `HashMap<NodeID, Node>` with monotonic `u32` IDs** rather than `Vec<Node>` (faster but invalidates IDs on removal) or `slotmap` (handles removal but adds a dependency). Chosen for zero-effort insert-and-lookup with no current need for removal. Trade-off explicit: slower iteration, no contiguous memory, would push toward `slotmap` once fusion needs node removal/replacement.
- **D5. Newtype primitives (`NodeID(u32)`, `TensorShape(Vec<usize>)`)** rather than type aliases. Compile-time distinction prevents accidentally passing a bare `u32` where a `NodeID` is expected. Standard Rust safety pattern.
- **D6. `OpKind` without `PartialEq`** — likely oversight rather than design (LifeOS flags the test workarounds as evidence the author noticed but did not fix).
- **D7. GPU parameter defaults** — `tile_size=16`, `block_size=256`, ranges `[4, 64]` and `[64, 1024]`. Inferred reasoning: classic CUDA matmul tiling (16×16=256 threads), warp-aligned blocks (256 = 8 warps), mirrors typical `maxThreadsPerBlock=1024`.
- **D8. Single crate, not a workspace** — README claims a three-crate workspace split that does not exist. Single crate is appropriate for current size; workspace split would matter once `clap` is added (isolating CLI's transitive deps from the library).
- **D10. Tests landed in one commit** — all 730 lines, 26 tests in commit `efe2597` "added extensive testing" (2025-07-08). LifeOS infers AI-assisted generation from stylistic uniformity, the 1000-node stress test, and explicit comment workarounds for the missing `PartialEq`.
- **D11. README written as a roadmap, marked as progress** — uses `[x] ... in progress` markers for unbuilt features, which LifeOS classifies as "the worst of both worlds" and recommends rewriting first thing if the project resumes.

## What is currently built

The honest implemented scope, per LifeOS Overview.md's verified evidence:

| Subsystem | State | Evidence |
|---|---|---|
| IR primitives (`NodeID`, `TensorShape`, `OpKind`) | Working, well-tested | `src/ir/types.rs` (47 lines); 7 tests |
| Graph + Node structs | Working, tested; `HashMap` store with monotonic u32 IDs | `src/ir/graph.rs`, `src/ir/ops.rs`; 10 graph tests + 9 ops tests including a 1000-node stress test |
| Error taxonomy | Defined; only `ValidationError` has `Display`; `XyntraError` and the other three sub-enums have no `Display` and no `std::error::Error` impl | `src/ir/errors.rs` (~160 lines) |
| Config struct + GPU parameter validation | Present; 7 validation branches; struct lacks `pub` so unusable externally | `src/config/mod.rs` |
| Graph validator | Scaffold only — types defined, all four public methods `todo!()` | `src/ir/validation.rs` (71 lines) |
| `main.rs` | Prints `"Hello, world!"` literally | `src/main.rs` (3 lines) |
| ONNX parser, TorchScript loader, WGSL codegen, PTX codegen, e-graph fusion, autotuner, occupancy tracing, CLI | **Not started.** | Confirmed by absence in `Cargo.toml` and source tree |

Verified scale: ~39KB total across 17 files, ~11.8KB source LOC across 9 `.rs` files, ~22.3KB test LOC across 4 `.rs` files (test:source ratio ~1.9:1), 10 commits total, 1 GitHub star.

## Current state

**Status: dormant.** Last commit `c639b24` "started on graph validator" on 2025-07-10. The active development window was 2025-07-05 to 2025-07-10 — six days, then nothing. At LifeOS extraction (2026-04-24) the project had been untouched for approximately nine months. Total commit count: 10. There is no active `Work/` folder in the LifeOS source for this project — the in-flight item per LifeOS is finishing the validator (Phase 1 of the roadmap), but no active session is touching it.

## Gaps and known limitations

LifeOS Gaps.md inventories these honestly:

- **`struct XyntraConfig` is not `pub`.** External callers cannot construct or use it. One-keyword fix not yet applied.
- **`main.rs` independently re-declares `pub mod ir;`** — the binary builds an `ir` module tree separate from `lib.rs`'s, producing two compile-unit-scoped views that cannot interoperate.
- **`Graph` has no iteration API.** `GraphValidator` cannot walk the graph; `validate_node_references` cannot be implemented without `Graph::iter_nodes` or equivalent.
- **`Node` has no `TensorShape` field.** Shape-aware validation is structurally impossible without schema change.
- **`OpKind` does not derive `PartialEq`/`Eq`** — pervasive ergonomic papercut, forces `match` for every op-kind assertion.
- **`outputs` field on `Node` is dead state.** Every test passes `vec![]`; no code reads or maintains it.
- **`XyntraError` has no `Display` impl** and `std::error::Error` is unimplemented anywhere — `format!("{}", err)` will not compile for the top-level error type, blocking `?`-based propagation to `Box<dyn Error>` ergonomically.
- **`config` and `errors` have zero test coverage** despite containing the only branching logic in the codebase. The 26 tests cover IR types, graph, ops — not the modules where regressions would actually matter.
- **No CI, no `.github/workflows/`, no `clippy.toml`, no `cargo-deny`.** Given the unfixed `pub` issue on `XyntraConfig`, the current `master` may not externally compile.
- **No property-based tests, no fuzzing, no benchmarks** despite the README claiming a benchmarking suite.
- **No `LICENSE` file** despite the README promising MIT + Apache-2.
- **README overstates the project significantly** — six items marked `[x]` are either not done or not started; only one survives code inspection. LifeOS classifies this as the single most misleading surface in the repo and the most important honesty fix if the project resumes.

## Direction (in-flight, not wishlist)

Per LifeOS Roadmap.md, the only items with concrete near-term plans (Phase 0 + Phase 1) that LifeOS treats as the realistic next step:

- **Phase 0 — Repo hygiene (sub-day).** Add `pub` to `XyntraConfig`; remove duplicate `pub mod ir;` from `main.rs`; add `#[derive(PartialEq, Eq, Clone)]` to `OpKind`; add `Display` impls to all error sub-enums and `XyntraError`; add `std::error::Error for XyntraError`; rewrite README with honest `[ ]` / `[~]` / `[x]` markers; add `cargo check` + `cargo test` GitHub Action; add `LICENSE`; add tests for `XyntraConfig::validate` (7 branches) and `ValidationError::Display` (9 arms).
- **Phase 1 — Implement the validator (1–3 days).** Add `Graph::iter_nodes`; fill in `validate_node_references` (iterate, check inputs exist), `detect_cycles` (white/grey/black DFS), `validate_operation_constraints` (static OpKind→arity table), `validate` (orchestrator using `combine_results`); add tests per method.

Phases 2+ (CLI + config loader, ONNX ingestion, single fusion rewrite, minimum WGSL codegen, then PTX/autotuner/occupancy/plugins) are sequenced in Roadmap.md but are not "in flight" in any meaningful sense — the project is dormant, and LifeOS Roadmap.md notes "Restarting requires a honest decision: is this a project Caner wants to work on, or a portfolio placeholder that should be marked archived and replaced by something that can actually ship?"

## Demonstrated skills

What the existing code, with all its honest limits, actually proves:

- **Designs typed compiler IRs in Rust from first principles** — newtype primitives, derived trait selection chosen for downstream `HashMap` / equality use, lifetime-parameterised validator structs, dependency-direction discipline pointing toward leaf modules.
- **Builds tested foundational data structures in safe Rust with no `unsafe`** — `HashMap`-backed graph container with monotonic ID allocation, accessor methods on `Node`, integration-test fixtures factored into a shared `tests/common.rs`.
- **Designs error taxonomies for a multi-stage pipeline ahead of callers** — four-category enum hierarchy with variants chosen to feed parser / validator / codegen stages that do not yet exist; `Display` formatter for the live-use class.
- **Familiar with CUDA / GPU kernel-launch parameter conventions** — defaults of `tile_size=16` / `block_size=256` and ranges of `[4, 64]` / `[64, 1024]` reflect knowledge of warp size (32), `maxThreadsPerBlock` (1024 on most architectures), and matmul tiling defaults — even though no GPU code is present.
- **Recognises the transformer-block op vocabulary as IR design input** — choice of MatMul/Add/Gelu/Dropout/Softmax/LayerNorm + Custom is targeted at attention/FFN workloads, signalling intent to compile transformer-style models.
- **Practises bottom-up construction with thorough testing of foundational layers** — types → graph → tests → errors → config → validator stubs, with the 1000-node stress test characterising intended scale (well above any realistic ML graph size).
- **Capable of self-honest portfolio assessment** — the LifeOS folder for this project (Reality vs README, Gaps.md) explicitly itemises every gap between the README's claims and the code, and recommends rewriting the README first if the project resumes. This kind of self-audit is itself portfolio signal.

Negative space worth being honest about: this project does **not** currently demonstrate compiler-frontend parsing, graph rewriting, e-graph use, GPU shader codegen, kernel autotuning, or any end-to-end ML compilation. The README claims all of these; the code implements none of them. A grading agent should treat Xyntra as evidence of "Rust compiler-IR foundational layer" only, not as evidence of "ML compiler" capability.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Xyntra/Overview.md | 94 | "`#project/xyntra` `#rust` `#compiler` `#ml-systems` `#skeleton` `#dormant` `#gpu`" |
| Projects/Xyntra/Architecture.md | 153 | "`#project/xyntra` `#rust` `#architecture` `#compiler-frontend` `#ir`" |
| Projects/Xyntra/Decisions.md | 228 | "`#project/xyntra` `#decisions` `#design-rationale`" |
| Projects/Xyntra/Gaps.md | 140 | "`#project/xyntra` `#gaps` `#unimplemented` `#bugs`" |
| Projects/Xyntra/Reality vs README.md | 68 | "`#project/xyntra` `#reality-check` `#anti-puffing` `#portfolio-accuracy`" |
| Projects/Xyntra/Roadmap.md | 147 | "`#project/xyntra` `#roadmap` `#phased-plan`" |
| Projects/Xyntra/Systems/Config.md | 105 | "`#project/xyntra` `#config` `#gpu-parameters` `#validation`" |
| Projects/Xyntra/Systems/Errors.md | 111 | "`#project/xyntra` `#errors` `#error-handling` `#rust`" |
| Projects/Xyntra/Systems/Graph.md | 129 | "`#project/xyntra` `#ir` `#graph` `#dag`" |
| Projects/Xyntra/Systems/IR Types.md | 109 | "`#project/xyntra` `#ir` `#types` `#primitives`" |
| Projects/Xyntra/Systems/Testing.md | 85 | "`#project/xyntra` `#testing` `#test-infrastructure`" |
| Projects/Xyntra/Systems/Validation.md | 112 | "`#project/xyntra` `#validation` `#scaffold` `#todo` `#next-work`" |
