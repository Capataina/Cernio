---
name: Xyntra
status: dormant
source_repo: https://github.com/Capataina/Xyntra
lifeos_folder: Projects/Xyntra
last_synced: 2026-05-31
sources_read: 13
---

# Xyntra

## One-line summary

Early-stage safe-Rust typed IR crate intended as the foundation of an automatic kernel-fusion compiler for deep-learning graphs (ONNX/TorchScript → WGSL/PTX) — currently a skeleton: IR primitives, graph container, error taxonomy, and config validator, with the validator and every downstream pipeline stage unimplemented.

## What it is

Xyntra is a single Rust crate (Rust 2024 edition, zero external dependencies) that aspires to become an automatic kernel-fusion compiler pass: ingesting ML graphs from ONNX / TorchScript, pattern-matching common op chains, and emitting one fused GPU kernel via `wgpu` (WGSL) or CUDA PTX, with the entire pipeline kept `unsafe`-free. The repository as it stands (commit `c639b24`, last activity 2025-07-10) is a typed IR layer with a stub validator: ~11.8KB of source across 9 `.rs` files and ~22.3KB of tests across 4 `.rs` files, 27 `#[test]` functions, no compiler pipeline. LifeOS classifies the design ambition as a roadmap and the implemented surface as a skeleton; the gap between README and code is captured explicitly in the `Reality vs README` note. The project has not been touched in ~9 months at extraction time, with the last commit titled "started on graph validator" adding `todo!()` stubs.

## Architecture

Single binary+library crate `xyntra` (version 0.1.1, edition 2024). The actual layout from LifeOS Architecture.md:

```
xyntra/                    (single crate, binary + library)
├── Cargo.toml             (name = "xyntra", empty [dependencies])
├── src/
│   ├── lib.rs             (2 lines: `pub mod config; pub mod ir;`)
│   ├── main.rs            (3 lines: `pub mod ir;` + `Hello, world!`)
│   ├── config/
│   │   └── mod.rs         (XyntraConfig, BackendType, validate())
│   └── ir/
│       ├── mod.rs         (re-exports errors, graph, ops, types, validation)
│       ├── types.rs       (NodeID, TensorShape, OpKind)
│       ├── ops.rs         (Node struct + accessors)
│       ├── graph.rs       (Graph — HashMap<NodeID, Node>)
│       ├── validation.rs  (GraphValidator — all todo!())
│       └── errors.rs      (XyntraError + 4 sub-enums)
└── tests/                 (common.rs, test_types.rs, test_graph.rs, test_ops.rs)
```

Dependency direction (acyclic, leaves at the bottom):

```
   config::XyntraConfig ─── uses ──> ir::errors
   ir::validation       ─── uses ──> ir::graph, ir::errors, ir::types
   ir::graph            ─── uses ──> ir::ops, ir::types
   ir::ops              ─── uses ──> ir::types
   ir::types            ── leaf ──  (only std)
   ir::errors           ── leaf ──  (only core::fmt)
```

The README describes a workspace split (`xyntra-core`, `xyntra-cli`, `xyntra-ir`); this does not exist. There is one crate. `main.rs` re-declares `pub mod ir;` independently of `lib.rs`, producing two distinct `ir` module trees across the binary and library compile units — LifeOS flags this as a classic Rust newcomer mistake. Future intended pipeline shape, per README (none of these modules exist):

```
cli (clap) → config → ingestion (onnx, tch-rs) → ir::graph → fusion (egg) → codegen (wgpu / ptx) → runtime
```

## Subsystems and components

### IR Types (`src/ir/types.rs`)

The most complete layer in the project. Three primitive value types:

- `NodeID(u32)` — newtype wrapping `u32`, derives `Debug, Clone, Copy, PartialEq, Eq, Hash`. Copy enables free pass-by-value; Hash enables `HashMap<NodeID, _>` keying.
- `TensorShape(Vec<usize>)` — newtype wrapping `Vec<usize>`, derives `Debug, Clone, PartialEq, Eq, Hash`. Methods: `new`, `rank`, `size` (product of dims, returns 1 for scalar via empty-product convention), `is_scalar`. No dtype field.
- `OpKind` — enum with 7 variants: `MatMul`, `Add`, `Gelu`, `Dropout`, `Softmax`, `LayerNorm`, `Custom(String)`. **Does not derive `PartialEq`** — every test that wants to assert op kind has to `match` instead of `assert_eq!`. The vocabulary skews to transformer attention/FFN blocks (no Conv, no Pool, no BatchNorm).

### Graph (`src/ir/graph.rs`, `src/ir/ops.rs`)

`Graph { nodes: HashMap<NodeID, Node>, next_id: u32 }` with three public methods: `new`, `add_node` (assigns `NodeID(next_id)`, increments, inserts), `get_node` (HashMap lookup). `Node { id, op, inputs, outputs }` with all fields `pub` and parallel accessor methods (redundant surface).

Invariants enforced: unique `NodeID` per node (monotonic counter, verified by 100-node and 1000-node stress tests). Invariants **not** enforced: input references existing nodes, no self-loop, acyclicity. The `outputs` field is dead in every test (always `vec![]`); the graph is effectively an inputs-only DAG, standard for SSA-style IRs but unusual for graph-rewriting compilers that frequently ask "what consumes this node?".

### Validation (`src/ir/validation.rs`) — scaffold only

`GraphValidator<'a> { graph: &'a Graph }`, `ValidationContext { current_node: Option<NodeID> }`, `ValidationResult = Result<(), Vec<ValidationError>>`. Three helper functions exist (`ok`, `single_error`, `combine_results`) and `combine_results` is the only piece of real logic (correctly accumulates errors). The four public validation methods (`validate_node_references`, `detect_cycles`, `validate_operation_constraints`, `validate`) are all `todo!()` — calling any of them panics at runtime. This is the project's most shovel-ready unit of work and where active development stopped.

Structural blockers that must be resolved before validators can be implemented: `Graph` exposes no iteration method (no `iter_nodes()`); `Node` has no `TensorShape` field (shape-aware validation impossible without an `ops.rs` schema change).

### Errors (`src/ir/errors.rs`)

Four-category enum hierarchy: `XyntraError { Validation, Parsing, System, Internal }`. The "recoverable vs fatal" split is a code comment, not type-level enforcement.

- `ValidationError` — 9 variants: `InvalidTensorShape`, `IncompatibleShapes`, `InvalidNodeConnection`, `CyclicGraph { cycle_path }`, `MissingNode`, `InvalidOpInputCount`, `InvalidConfigValue`, `InvalidFilePath`, `InvalidGPUParameter`. Only one with a `Display` impl.
- `ParsingError` — 5 variants: `InvalidFormat`, `MalformedOnnx`, `UnsupportedOperation`, `CorruptedFile`, `MissingRequiredField`. No `Display`.
- `SystemError` — 4 variants: `OutOfMemory`, `GpuUnavailable`, `FileNotFound`, `PermissionDenied`. No `Display`.
- `InternalError` — 4 variants: `AssertionFailed`, `UnexpectedNone`, `InvalidState`, `NotImplemented`. No `Display`.

Live use sites: only `ValidationError::{InvalidGPUParameter, InvalidConfigValue, InvalidFilePath}` are constructed (by `XyntraConfig::validate`). Everything else is dead, scaffolded for downstream pipeline stages that do not exist. `XyntraError` itself has no `Display` impl and `std::error::Error` is not implemented anywhere.

### Config (`src/config/mod.rs`)

`XyntraConfig { input_file: Option<PathBuf>, output_dir: PathBuf, backend: BackendType, optimisation_level: u8, tile_size: usize, block_size: usize, enable_debug, export_ir }` with `BackendType { Wgsl (default), CudaPtx }`. Defaults: `tile_size=16`, `block_size=256`, `optimisation_level=2`, `backend=Wgsl`. `validate()` enforces 7 checks: tile_size power-of-2 + range `[4, 64]`, block_size power-of-2 + range `[64, 1024]`, optimisation_level ≤ 3, input_file existence, output_dir metadata-readable. **Bug:** `struct XyntraConfig` lacks `pub`, so external callers cannot construct it; `lib.rs` publishes a module that exposes no usable types. Also: British/American spelling inconsistency (`optimisation_level` field vs `"optimization_level"` error string).

### Testing (`tests/`)

26 `#[test]` functions across 4 files, 730 total lines (test:source ratio ~1.9:1 by bytes). All tests landed in a single commit (`efe2597` "added extensive testing", 2025-07-08, +733 lines) — stylistic uniformity suggests AI-assisted generation. Test infrastructure uses flat `tests/common.rs` + `mod common;` in each test file (triggers per-file `dead_code` warnings). Coverage: `ir::types` 7 tests, `ir::graph` 10 tests including 1000-node linear-chain stress test, `ir::ops` 9 tests; `ir::errors` 0 tests, `ir::validation` 0 tests (everything `todo!()`), `config` 0 tests despite having 7 validation branches. No property-based / fuzz tests (`proptest`, `quickcheck`, `cargo-fuzz` all absent). No CI (`.github/workflows/` absent). No benchmark harness despite README claim.

## Technologies and concepts demonstrated

### Languages
- **Rust** — Rust 2024 edition (`edition = "2024"` in `Cargo.toml`). Entire codebase is `unsafe`-free (trivially today, as there is no GPU/FFI/pointer code). Uses newtype primitives (`NodeID(u32)`, `TensorShape(Vec<usize>)`) for compile-time distinction, derived traits (`Debug, Clone, Copy, PartialEq, Eq, Hash`) for ergonomics, and a four-category error enum with sub-enums.

### Frameworks and libraries
- **None.** `[dependencies]` in `Cargo.toml` is empty. The crate uses only `std` and `core::fmt`. README mentions `egg`, `wgpu`, `clap`, `serde`, `toml`, `tch-rs` as planned dependencies; none are present.

### Runtimes / engines / platforms
- no source evidence in LifeOS for any runtime/engine/platform integration today. README aspires to `wgpu` (WGSL backend) and CUDA Driver API (PTX backend); neither exists in code.

### Tools
- no source evidence in LifeOS for build/profiling/debug tooling beyond `cargo test`. No `clippy` enforcement, no `cargo-deny`, no pre-commit config, no profiler output.

### Domains and concepts
- **Compiler frontend / IR design** — typed intermediate representation with newtype identifiers, operator enum vocabulary, and a HashMap-backed node store with monotonic u32 IDs. Inputs-only DAG representation (SSA-style).
- **Error-taxonomy design** — four-category split (`Validation`/`Parsing`/`System`/`Internal`) with sub-enums per category; recoverable-vs-fatal classification (documented in code comments, not type-enforced); design-ahead-of-use pattern (error variants scaffolded for downstream pipeline stages that do not exist yet).
- **GPU parameter validation** — power-of-2 bit-twiddle checks (`(x != 0) && ((x & (x - 1)) != 0)`) and range bounds for tile/block sizes; defaults (`tile=16`, `block=256`) reflect classic CUDA matmul tiling and warp-aligned block sizing (256 = 8 warps × 32 threads).
- **Test-suite design** — example-based tests with parametrised variant iteration, immutability checks, pointer-equality retrieval consistency, 1000-node linear-chain stress test characterising intended scale (10× larger than realistic ML graphs: BERT ~200 ops, ResNet-50 ~170 layers).
- **Transformer-block op vocabulary** — `MatMul, Add, Gelu, Dropout, Softmax, LayerNorm` is exactly the op set of a transformer FFN/attention layer; implicit signal that the intended target workload is LLMs/attention rather than CNNs.

## Key technical decisions

Drawn from LifeOS Decisions.md (D1–D11):

- **D1. Safe Rust only, no `unsafe` blocks.** README explicitly commits to "100% `unsafe`-free". Trivially satisfied today (no GPU/FFI/pointer code). Sustainability questioned for the future PTX path (NVIDIA Driver API FFI likely needs `unsafe`); `wgpu`'s safe public API makes the WGSL path tractable.
- **D2. Rust 2024 edition.** Cutting-edge positioning; requires rustc 1.85+. None of edition 2024's new features (async fn in traits, improved RPITIT) are actually used.
- **D3. Four-category error split.** `Validation`/`Parsing`/`System`/`Internal` reflects compiler-pass lifecycle (parse → validate → work → system failures). `Internal` gives a slot for `NotImplemented`/`AssertionFailed` during prototyping. Rejected: flat enum, fewer categories (just `UserError`/`InternalError`), per-subsystem split (`IRError`/`ConfigError`/`CodegenError`).
- **D4. HashMap-backed node store with monotonic u32 IDs.** Chosen for zero-effort insert-and-lookup with no removal-thought. Rejected: `Vec<Node>` (faster, denser, invalidates IDs on remove), `slotmap`/`generational_arena` (handles removal safely but adds a dependency), separate adjacency list. Trade-off: slower iteration; `u32` supports 4B nodes (vastly over-provisioned); no remove API today so the choice is not yet exercised.
- **D5. Newtype primitives (NodeID, TensorShape).** Compile-time distinction over type aliases; rejected richer structs (e.g. `TensorShape { dims, dtype }`) because dtype design not done yet.
- **D6. OpKind without PartialEq.** Likely an oversight rather than design (tests have explicit workaround comments). Recommendation in LifeOS: add `#[derive(PartialEq, Eq, Clone)]`.
- **D7. GPU parameter defaults `tile=16, block=256`.** Classic CUDA programming-guide defaults for matmul tiling; ranges `[4, 64]` and `[64, 1024]` mirror `maxThreadsPerBlock` limits.
- **D8. Single crate, not a workspace.** Premature workspace split adds Cargo friction; trigger to split would be real CLI (`clap` dependency isolation) or separate codegen crate.
- **D9. Test infrastructure in flat `tests/common.rs`.** Simplest possible structure; trade-off accepted: per-file `dead_code` warnings for unused helpers.
- **D10. Testing approach — thorough tests upfront.** All 26 tests in one commit (`efe2597`, 730 lines), stylistically uniform — LifeOS infers AI-assisted generation. Subsequent subsystems (config, validator) received no tests because the initial test push was not re-run.
- **D11. README written as a roadmap, marked as progress.** README uses `[x] … *in progress*` markers; 6 items marked `[x]` are not done. LifeOS recommends rewriting with honest `[ ]/[~]/[x]` markers.

## What is currently built

Honest implemented scope from LifeOS Overview.md "What is actually built" table:

| Subsystem | State |
|-----------|-------|
| IR primitives (`NodeID`, `TensorShape`, `OpKind`) | Working, 7 tests |
| Graph / Node structs (HashMap-backed, sequential u32 IDs) | Working, 10 tests including 1000-node stress |
| Error taxonomy (4 top-level categories, 22 total variants) | Defined; only `ValidationError` has `Display` |
| Config struct + validation (7 validation branches) | Present; **bug**: `struct XyntraConfig` lacks `pub` |
| Graph validator scaffold | All 4 public methods `todo!()` |
| `main.rs` | Literally prints `"Hello, world!"` (3 lines) |
| ONNX parser | **Not started** — only `ParsingError::MalformedOnnx` variant |
| TorchScript loader | **Not started** — no `tch-rs` dependency |
| WGSL codegen | **Not started** — only `BackendType::Wgsl` enum variant |
| PTX codegen | **Not started** — only `BackendType::CudaPtx` enum variant |
| e-graph fusion | **Not started** — no `egg` dependency, no rewrite rules |
| Autotuner, occupancy tracing | **Not started** |
| CLI | **Not started** — no `clap`, no argv parsing |

Total: ~39KB / 17 files / ~11.8KB source / ~22.3KB tests / 26 `#[test]` functions / 10 commits / 6-day active dev window (2025-07-05 → 2025-07-10) / 1 GitHub star. LifeOS estimates completeness vs README at ~5% (IR + error types + config scaffold; zero pipeline stages).

## Current state

**Dormant.** Last commit `c639b24` "started on graph validator" on 2025-07-10; no activity in ~9 months as of LifeOS verification (2026-04-24). LifeOS Overview.md frontmatter sets `status: skeleton`. No in-flight work captured in LifeOS — there is no `Work/` folder for this project. The Roadmap.md exists as a phased plan but reflects intent if the project resumes, not active execution.

## Gaps and known limitations

Career-relevant gaps from LifeOS Gaps.md:

**Compile / correctness:**
- `struct XyntraConfig` is module-private (missing `pub`); external callers cannot construct it. One-keyword fix not done.
- `main.rs` re-declares `pub mod ir;` independently of `lib.rs`, producing two independent `ir` module trees across binary and library compile units. Will manifest as cross-unit type mismatches if `main.rs` ever uses library types.

**Structural blockers for the validator:**
- `Graph` exposes no iteration method; `GraphValidator` cannot walk the graph.
- `Node` has no `TensorShape` field; shape-aware validation is structurally impossible without a schema change in `ops.rs`.
- `Node.outputs` field is dead in every test; the graph is effectively inputs-only.

**Error system:**
- `XyntraError` has no `Display` impl; only inner `ValidationError` formats cleanly. `println!("{}", xyntra_error)` will not compile.
- `std::error::Error` not implemented anywhere; `?`-based propagation to `Box<dyn Error>` works only via auto-derived `From` chains.
- `OpKind` does not derive `PartialEq`/`Eq`; pervasive ergonomic papercut (every test asserting op kind uses `match`).

**Testing:**
- `config` (7 validation branches) and `errors` (9 `ValidationError` Display arms) have zero coverage. 26 of 26 tests cover pure data containers; the files with branching logic have no tests.
- No property-based / fuzz tests (no `proptest`, `quickcheck`, `cargo-fuzz`).
- No CI (no `.github/workflows/`); given the `struct XyntraConfig` visibility bug, master may not compile externally and this has not been caught.
- No benchmark harness despite README claim.

**Portfolio-accuracy risk (LifeOS explicit warning):**
- The README and GitHub description describe a compiler pipeline; the code is a typed IR crate with a stub validator. README marks 6 items `[x]` that are not done. External readers will believe the project is further along than it is. LifeOS classifies this as the single most important gap to close when the project resumes.

## Direction (in-flight, not wishlist)

There is no in-flight work — the project has been dormant for ~9 months. LifeOS Roadmap.md captures a phased plan that activates *if* Caner resumes the project; nothing here is current execution. The two concrete phases (later phases are sketched ambition):

- **Phase 0 (sub-day, repo hygiene):** `pub` fix on `XyntraConfig`, remove duplicate `pub mod ir;` from `main.rs`, derive `PartialEq/Eq/Clone` on `OpKind`, add `Display` impls for `XyntraError` and sub-enums, add `std::error::Error`, rewrite README with honest markers, add `cargo check`+`cargo test` CI, add LICENSE, add tests for `XyntraConfig::validate` (7 branches) and `ValidationError::Display` (9 arms).
- **Phase 1 (1–3 days, the validator):** add `Graph::iter_nodes`, implement `validate_node_references` (iterate, confirm input IDs exist), `detect_cycles` (DFS with white/grey/black colouring producing `cycle_path`), `validate_operation_constraints` (static `OpKind → expected_input_count` table: MatMul=2, Add=2, Gelu=1, Softmax=1, LayerNorm=1, Dropout=1, Custom=any), `validate()` (combine via `combine_results`), per-method tests, decide whether `outputs` field stays or goes.

Phases 2–6+ (CLI + config-file loader, ONNX ingestion, one fusion rewrite, minimum WGSL codegen, then rest of README) are roadmap aspiration, not direction.

## Demonstrated skills

What this specific project's LifeOS-captured implementation proves Caner can do today (distinct from README ambition):

- **Designed a typed compiler IR layer in safe Rust 2024** — three primitive value types with appropriate derived traits, a HashMap-backed graph container with monotonic u32 ID allocation, an inputs-only DAG representation, and an explicit dependency-direction architecture (leaves at `ir::types` / `ir::errors`, acyclic flow up to `config` and `validation`).
- **Designed a four-category error taxonomy ahead of its callers** — 22 total variants partitioned into `Validation`/`Parsing`/`System`/`Internal`, with sub-enums scaffolded for downstream compiler-pipeline stages (parse → validate → lower → codegen) that do not yet exist. Design-ahead-of-use pattern applied deliberately.
- **Implemented GPU-parameter validation with bit-twiddle power-of-2 checks** — `tile_size` (range `[4, 64]`, power-of-2) and `block_size` (range `[64, 1024]`, power-of-2) with defaults (`16`, `256`) chosen to match classic CUDA matmul tiling and warp-aligned block sizing (256 = 8 warps × 32 threads).
- **Built a thorough example-based test suite** — 26 tests across 4 files (730 lines), test:source ratio ~1.9:1 by bytes, including a 1000-node linear-chain stress test characterising intended scale at 10× realistic ML graph size; coverage of newtype constructor/equality/hash semantics, parametrised iteration over all `OpKind` variants, immutability and retrieval-consistency checks.
- **Made deliberate, documented design trade-offs** — chose HashMap over slotmap because removal is not yet needed; chose newtype primitives over type aliases for compile-time distinction; chose flat `tests/common.rs` over nested mod for simplicity at this size; documented each trade-off with rejected alternatives and the trigger that would change the decision.
- **Performed honest self-audit against own README** — LifeOS contains a dedicated `Reality vs README.md` note that itemises every README claim, classifies it (Aspirational / Partially true / False / Verified), and re-marks the README's own `[x]` checklist against code reality. Demonstrates the discipline of separating design ambition from implemented scope.

Limits on what this project demonstrates: no compiler pass has been implemented; no parsing, fusion, or codegen exists; no GPU code has been written; the validator is `todo!()` stubs; the project has been dormant for ~9 months. Anyone evaluating fit against a compiler-engineering role should read this project as evidence of IR-design and Rust-modelling instinct, not as evidence of completed compiler work.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Xyntra/_Overview.md | 103 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
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
