---
name: Xyntra
status: dormant
source_repo: https://github.com/Capataina/xyntra
lifeos_folder: Projects/Xyntra
last_synced: 2026-05-13
sources_read: 13
---

# Xyntra

## One-line summary

An early-stage typed IR crate in safe Rust 2024 — the foundational layer of an aspired ML kernel-fusion compiler, currently consisting of a well-tested graph / node / types layer and a stub `GraphValidator` whose four methods are all `todo!()`.

## What it is

Xyntra is a personal Rust project that aspires to become an automatic kernel-fusion compiler for deep-learning graphs — ingesting ONNX / TorchScript, applying rewrite passes, and emitting one fused GPU kernel through WGSL or CUDA PTX. The README describes that pipeline as the design target. The repository as it stands implements **only** the foundational IR layer: `NodeID`, `TensorShape`, `OpKind`, `Node`, `Graph`, a four-category error taxonomy, a partially-wired `XyntraConfig`, and a validator scaffold whose every public method panics via `todo!()`. The distinction between *design ambition* and *current implementation* is large — roughly 3–5% of the README's scope is built — and the project has been dormant since 2025-07-10 (~9 months at the LifeOS extraction point of 2026-04-24). Treating the README as a roadmap rather than a description is the honest framing the LifeOS source explicitly insists on.

## Architecture

Single crate (`xyntra`, version 0.1.1, Rust 2024 edition, binary + library, zero external dependencies — `[dependencies]` in `Cargo.toml` is empty). The README claims a workspace split into `xyntra-core`, `xyntra-cli`, `xyntra-ir`; this does not exist.

```
xyntra/
├── Cargo.toml             (name = "xyntra", edition = "2024", no deps)
├── src/
│   ├── lib.rs             (2 lines: `pub mod config; pub mod ir;`)
│   ├── main.rs            (3 lines: `pub mod ir;` + Hello world print)
│   ├── config/
│   │   └── mod.rs         (XyntraConfig, BackendType, validate())
│   └── ir/
│       ├── mod.rs         (re-exports errors, graph, ops, types, validation)
│       ├── types.rs       (NodeID, TensorShape, OpKind — ~47 lines)
│       ├── ops.rs         (Node struct + accessors)
│       ├── graph.rs       (Graph — HashMap<NodeID, Node>)
│       ├── validation.rs  (GraphValidator — all four methods todo!() — 71 lines)
│       └── errors.rs      (XyntraError + four sub-enums — ~160 lines)
└── tests/
    ├── common.rs          (test helpers + builders)
    ├── test_types.rs      (7 tests)
    ├── test_graph.rs      (10 tests)
    └── test_ops.rs        (9 tests)
```

**Dependency direction (actual, acyclic):** `config` and `validation` both depend on `ir::errors`; `validation` additionally depends on `ir::graph` and `ir::types`; `graph` depends on `ops`, which depends on `types`; `types` and `errors` are leaves (only `std` / `core::fmt`). Good shape for a foundational layer — flow is uniformly downward.

**Anti-architecture flags surfaced in LifeOS:**

- `main.rs` declares `pub mod ir;` independently of `lib.rs`'s `pub mod ir;`, producing two distinct `ir` module trees in the binary's compile unit. Compiles today only because `main.rs` does not use anything library-side; type mismatches will appear if it ever does.
- `struct XyntraConfig` lacks `pub` at `src/config/mod.rs:10`, so although `lib.rs` re-exports `pub mod config;`, the struct itself is module-private. External consumers (future CLI, examples) cannot construct one. One missing keyword.
- `GraphValidator::validate_node_references`, `detect_cycles`, `validate_operation_constraints`, and `validate()` are all `todo!()` — calling them panics.

## Subsystems and components

### IR Types (`src/ir/types.rs`)

The most complete layer. Three primitives:

- **`NodeID(u32)`** — `Copy` newtype, derives `Debug, Clone, Copy, PartialEq, Eq, Hash`. Usable as a `HashMap` key directly. Two NodeIDs with the same `u32` are interchangeable across `Graph` instances (no opaque identity / `PhantomData` scoping) — a footgun if multi-graph support arrives, but not relevant today.
- **`TensorShape(Vec<usize>)`** — newtype, derives `Debug, Clone, PartialEq, Eq, Hash`. Methods: `new`, `rank`, `size` (product of dims, returns 1 for empty / scalar matching the empty-product convention), `is_scalar`. No dtype field. `size()` uses unchecked `*=` — silent overflow on pathological shapes like `[usize::MAX, 2]`.
- **`OpKind`** — enum of 7 variants: `MatMul`, `Add`, `Gelu`, `Dropout`, `Softmax`, `LayerNorm`, `Custom(String)`. **Does not derive `PartialEq` / `Eq`** — tests explicitly work around this with `match` statements (the LifeOS file flags this as likely an oversight rather than a deliberate decision). The vocabulary skews to transformer FFN / attention blocks — no `Conv`, no `Pool`, no element-wise `Mul`/`Div`, no `BatchNorm`.

### Graph (`src/ir/graph.rs`, `src/ir/ops.rs`)

- **`Graph { nodes: HashMap<NodeID, Node>, next_id: u32 }`** with three methods: `new`, `add_node(OpKind, Vec<NodeID>, Vec<NodeID>) -> NodeID` (assigns monotonic `NodeID(next_id)`, increments, inserts), `get_node(NodeID) -> Option<&Node>`.
- **`Node { id: NodeID, op: OpKind, inputs: Vec<NodeID>, outputs: Vec<NodeID> }`** with both `pub` fields and accessor methods (redundant surface — tests use field access directly).
- **Encoding:** inputs-only DAG. The `outputs` field is dead state — every test passes `vec![]` for it, no code reads or maintains it. To answer "what consumes node X?" you must scan all nodes.
- **Invariants enforced:** unique `NodeID` per node (monotonic counter, verified by a 100-node and a 1000-node stress test). **Not enforced:** inputs reference existing nodes; no self-loop; acyclicity; `outputs` populated. These are what `GraphValidator` is meant to catch.

### Errors (`src/ir/errors.rs`)

Four-category hierarchy: `XyntraError { Validation(ValidationError), Parsing(ParsingError), System(SystemError), Internal(InternalError) }`. The recoverable-vs-fatal split is a code comment, not a structural property.

- **`ValidationError`** — 9 variants: `InvalidTensorShape`, `IncompatibleShapes`, `InvalidNodeConnection`, `CyclicGraph`, `MissingNode`, `InvalidOpInputCount`, `InvalidConfigValue`, `InvalidFilePath`, `InvalidGPUParameter`. **The only sub-enum with a `Display` impl.**
- **`ParsingError`** (5 variants: `InvalidFormat`, `MalformedOnnx`, `UnsupportedOperation`, `CorruptedFile`, `MissingRequiredField`), **`SystemError`** (4: `OutOfMemory`, `GpuUnavailable`, `FileNotFound`, `PermissionDenied`), **`InternalError`** (4: `AssertionFailed`, `UnexpectedNone`, `InvalidState`, `NotImplemented`) — no `Display`, no `std::error::Error` impl anywhere.
- **Actually used today:** only `ValidationError::{InvalidGPUParameter, InvalidConfigValue, InvalidFilePath}` from `XyntraConfig::validate`. Every other variant is uninstantiated — scaffolded ahead of need.

### Config (`src/config/mod.rs`)

The only module returning a live `Result<_, XyntraError>` from real logic. Fields: `input_file: Option<PathBuf>`, `output_dir: PathBuf` (default `"."`), `backend: BackendType` (default `Wgsl`, other variant `CudaPtx`), `optimisation_level: u8` (default 2, range 0–3), `tile_size: usize` (default 16, range 4–64, must be power of 2), `block_size: usize` (default 256, range 64–1024, must be power of 2), `enable_debug: bool`, `export_ir: bool`. `validate()` enforces all seven rules. The defaults are CUDA-conventional: `tile_size=16` → 256-thread tile (8 warps), `block_size=256` matching `maxThreadsPerBlock` lineage.

Issues: struct lacks `pub`; British / American naming inconsistency (`optimisation_level` field, `"optimization_level"` error string); cosmetic error-message punctuation inconsistencies; `tile_size == 0` slips the power-of-2 branch (caught by the bound check anyway). `output_dir` "exists" check is metadata-based, conflating not-found with permission errors.

### Validation (`src/ir/validation.rs`)

Pure scaffold. `GraphValidator<'a> { graph: &'a Graph }`, `ValidationContext { current_node: Option<NodeID> }`, `pub type ValidationResult = Result<(), Vec<ValidationError>>`. Three free helpers — `ok()`, `single_error(err)`, `combine_results(Vec<ValidationResult>)` — the last being the only piece of real logic in the file (correctly flattens error vectors into a combined `Err`). The four public methods (`validate_node_references`, `detect_cycles`, `validate_operation_constraints`, `validate`) are all `todo!()`. Last commit `c639b24` is titled *"started on graph validator"* — the project paused at this exact unit.

Structural blockers to filling them in: `Graph` exposes no iteration API (no `iter_nodes`); `Node` has no `TensorShape` field, so shape-aware validation requires a schema change in `ops.rs`, not just filling in `validation.rs`.

### Testing (`tests/common.rs`, `tests/test_*.rs`)

26 `#[test]` functions across 4 files (730 lines total, ~1.9:1 test-to-source ratio by bytes), all landed in a single commit `efe2597` *"added extensive testing"* on 2025-07-08. Coverage is concentrated on the easy targets:

| Subsystem | Tests |
|---|---|
| `ir::types` | 7 (rank, size, scalar detection, equality, hashing, edge cases) |
| `ir::graph` | 10 (empty, single / multi add, diamond, ID uniqueness, 1000-node stress chain) |
| `ir::ops` | 9 (creation, accessors, all OpKind variants, immutability) |
| `ir::errors` | **0** — Display formatting never asserted |
| `ir::validation` | **0** — every method `todo!()` |
| `config` | **0** — 7 validation branches unexercised |

`tests/common.rs` provides `create_test_node_id`, `build_simple_graph` (input → matmul → output), `build_complex_graph` (matmul → gelu → dropout chain), `create_all_op_kinds`. Test infrastructure is in a flat `tests/common.rs` with `mod common;` per test file rather than the conventional `tests/common/mod.rs` — works but emits `dead_code` warnings for unused helpers.

No `proptest`, no `quickcheck`, no `cargo-fuzz`, no benchmarks, no CI config (`.github/workflows/` does not exist). The LifeOS source explicitly flags AI-assisted test generation as the likely origin of the single 733-line testing commit — stylistic uniformity, retroactive coverage of every accessor method, the 1000-node stress test at prototyping stage.

## Technologies and concepts demonstrated

### Languages

- **Rust 2024 edition** — entire codebase, no `unsafe` blocks anywhere. The 2024-edition choice is cutting-edge (requires rustc 1.85+); the edition's new features (async-fn-in-traits, improved RPITIT) are not actually exercised by current code.

### Frameworks and libraries

- **`std` only** — `Cargo.toml`'s `[dependencies]` is empty. Pure standard-library implementation. The README plans `egg`, `wgpu`, `clap`, `serde`, `toml`, `tch-rs`, none of which are present.

### Runtimes / engines / platforms

- No source evidence in LifeOS — no runtime / engine integration is present in the current code. The README's targets (`wgpu`, CUDA PTX, ONNX Runtime EP, PyTorch 2 plugin) are aspirational.

### Tools

- **`cargo`** — single-binary-plus-library crate. No CI, no clippy enforcement, no `cargo-deny`, no benchmark harness, no `.github/workflows/`.

### Domains and concepts

- **Typed compiler IR design** — newtype primitives (`NodeID`, `TensorShape`), enum-based op vocabulary, `HashMap`-backed node store with monotonic `u32` IDs, inputs-only DAG encoding. Foundational layer of a compiler frontend, designed before any downstream pass exists.
- **GPU-parameter validation** — power-of-2 + bound checking for tile and block sizes, with defaults (`tile_size=16`, `block_size=256`) drawn from CUDA programming-guide conventions for matmul tiling and warp-aligned occupancy.
- **Error-taxonomy design** — four-category split (Validation / Parsing / System / Internal) anticipating a full compiler pipeline (parse → validate → lower → codegen), scaffolded ahead of callers. Only one sub-enum (`ValidationError`) implements `Display`; `std::error::Error` is not implemented.
- **Test-infrastructure patterns** — fixture-builder helpers, parametrised iteration via a `Vec` of all op variants, immutability assertions via repeated accessor calls, pointer-equality checks for HashMap-retrieval idempotency, a 1000-node linear-chain stress test that exceeds realistic ML-graph sizes (BERT-base ~200 ops, ResNet-50 ~170 layers) by an order of magnitude.
- **Compiler-design vocabulary (named in LifeOS, not implemented):** graph rewriting, e-graph fusion (`egg`), pattern-matched op-chain fusion (`MatMul → Gelu`, bias fusion), WGSL / PTX codegen, autotuning, occupancy modelling, mixed-precision (FP16/BF16), shared-memory tiling, vectorisation, gradient checks, golden correctness tests. The README lists these; the code contains zero implementation.

## Key technical decisions

| # | Decision | Rejected alternatives | Reason |
|---|---|---|---|
| D1 | **Safe Rust only, zero `unsafe` blocks** | Allow `unsafe` for GPU FFI; encapsulate `unsafe` behind an FFI abstraction crate | README's stated portfolio framing; educational positioning. Trivially satisfied today (no GPU / FFI code yet). Sustainable for the WGSL path via `wgpu`'s safe public API; the PTX path via NVIDIA Driver API would likely require an `unsafe` FFI module. |
| D2 | **Rust 2024 edition** | 2021, 2018 | Cutting-edge positioning. Narrows install base to rustc 1.85+. No 2024-specific features actually used. |
| D3 | **Four-category error split** (`Validation` / `Parsing` / `System` / `Internal`) | Flat enum; coarser two-category split (`UserError` / `InternalError`); per-subsystem split (`IRError`, `ConfigError`, `CodegenError`) | Mirrors compiler-pass lifecycle. `Internal` slot for `NotImplemented` / `AssertionFailed` is useful during prototyping. Recoverable-vs-fatal is conceptual, not type-enforced. |
| D4 | **`HashMap<NodeID, Node>` with monotonic `u32` IDs** | `Vec<Node>` indexed by `usize` (faster, denser, invalidates on removal); `slotmap` / `generational_arena` (safe removal + dependency cost); separate adjacency list | Zero-effort insert-and-lookup; `Copy` IDs; no ID reuse keeps debug traces coherent. Trade-off: slower iteration; `slotmap` would be more attractive once fusion-time node removal arrives. |
| D5 | **Newtype primitives (`NodeID`, `TensorShape`)** | Type aliases (`type NodeID = u32;` — cheaper, no compile-time distinction); richer structs (e.g. `TensorShape { dims, dtype }` — requires dtype design first) | Compile-time type safety. Custom methods attach cleanly. Standard Rust practice. |
| D6 | **`OpKind` deliberately without `PartialEq`** (per code-comment evidence in tests) | Derive `PartialEq, Eq` | The LifeOS source classifies this as likely an oversight rather than a deliberate decision — `Custom(String)` would derive cleanly via `String` equality. The papercut is visible in every test's `match` workaround. |
| D7 | **GPU parameter defaults: tile=16, block=256** | Any other power-of-two combination | CUDA-conventional matmul tiling (16×16 = 256 threads per tile = 8 warps); `block_size=256` matches typical `maxThreadsPerBlock` lineage; ranges `[4,64]` and `[64,1024]` keep shared-memory footprint manageable. |
| D8 | **Single crate, not a workspace** | The README's claimed `xyntra-core` / `xyntra-cli` / `xyntra-ir` workspace split | Small enough that a workspace adds Cargo friction with no benefit. Trigger to split: a real CLI pulling in `clap`, or a heavy codegen crate that only the backend needs. |
| D9 | **Flat `tests/common.rs` + `mod common;` per file** | `tests/common/mod.rs` (the Rust-idiomatic non-test-target form); a separate `xyntra-testkit` crate | Simplest possible structure. Acknowledged cost: `dead_code` warnings for helpers not used in every test file. |
| D10 | **All 26 tests landed in one 733-line commit** | Incremental test-as-you-go | LifeOS infers AI-assisted generation from stylistic uniformity, retroactive accessor-level coverage, and the 1000-node stress test at prototyping stage. Consequence: subsequent subsystems (config, validation) received zero tests because the burst was not repeated. |
| D11 | **README written aspirationally with `[x] … in progress` markers** | Honest `[ ]` / `[~]` / `[x]` taxonomy | LifeOS flags this as the single biggest misleading surface in the repo — `[x]` in GFM means done, contradicts `*in progress*`, and overstates the project by ~20× relative to actual code. Recommended fix is a 20-minute README rewrite. |

## What is currently built

Concretely, at commit `c639b24` (2025-07-10):

- **IR primitives** — `NodeID`, `TensorShape`, `OpKind` (7 variants), `Node`, `Graph`. Working, tested.
- **Error taxonomy** — `XyntraError` + four sub-enums totalling 22 variants. Only `ValidationError` has `Display`. Only 3 variants are constructed by live code (all from `XyntraConfig::validate`).
- **Config scaffold** — `XyntraConfig` (module-private), `BackendType { Wgsl, CudaPtx }`, `validate()` with 7 rules. Not wired to any CLI or loader.
- **Validator scaffold** — types and signatures exist; all four `validate_*` methods are `todo!()`; only `combine_results` is real code.
- **Entry point** — `main.rs` prints `"Hello, world!"`. Three lines including braces.
- **Tests** — 26 functions, 730 lines, all in one commit, concentrated on types / graph / ops; zero on config, errors, or validator.

**What is not built** (every item from the README, mapped to current code):

| README claim | Current state |
|---|---|
| Automatic kernel-fusion compiler pass | Not started |
| ONNX / TorchScript ingestion | Not started — only the `ParsingError::MalformedOnnx` enum variant exists |
| Pattern-matched op-chain fusion | Not started |
| WGSL or CUDA PTX codegen | Not started — only `BackendType::{Wgsl, CudaPtx}` enum variants |
| `egg` e-graph rewriting | Not started — no `egg` dependency |
| GPU occupancy modelling | Not started |
| Autotuned codegen | Not started |
| Multi-crate workspace (`xyntra-core` / `xyntra-cli` / `xyntra-ir`) | Does not exist — single crate |
| Golden fused-vs-unfused tests | Not started — fusion does not exist |
| CLI / `fusion.toml` loader | Not started — no `clap`, no `toml` |
| CI pipeline | Not started — no `.github/workflows` |
| PyTorch 2 plugin / ONNX Runtime EP | Not started |
| Benchmarks (criterion / flamegraphs) | Not started — no benchmark harness |

LifeOS's own estimate: **~3–5% of the README's scope is built**. Test:source byte ratio ~1.9:1 indicates more test material than implementation.

## Current state

**Dormant.** Last commit `c639b24` *"started on graph validator"* lands 2025-07-10. The 6-day active development window 2025-07-05 → 2025-07-10 produced the entire current codebase across 10 commits. No activity in ~9 months as of the LifeOS extraction date 2026-04-24. The project paused at the first hard problem — cycle detection and shape validation — after the easier foundation (types, graph, tests, errors, config) was complete. Nothing in flight in LifeOS Work / Roadmap is being actively worked.

## Gaps and known limitations

**Compile / correctness:**

- `struct XyntraConfig` is not `pub` (missing keyword at `src/config/mod.rs:10`) — external consumers cannot construct one despite `pub mod config;` in `lib.rs`.
- `main.rs` declares its own `pub mod ir;` parallel to `lib.rs`'s `pub mod ir;`, producing two independent `ir` module trees in the binary's compile unit. Compiles only because `main.rs` does not yet reach for anything library-side.
- `GraphValidator` methods are `todo!()` — any call panics in release.

**Structural / API:**

- `Graph` exposes no iteration method — `GraphValidator` cannot walk the node map even if its methods were filled in.
- `Node` has no `TensorShape` field — shape-aware validation is structurally impossible without a schema change in `ops.rs`.
- `OpKind` lacks `PartialEq` / `Eq` derives, forcing `match` workarounds in every test.
- `Node.outputs` is dead state — every test passes `vec![]`, no code reads or maintains it.
- `Node` fields are `pub` and accessor methods exist — redundant API surface.
- `XyntraError` has no `Display` and no `std::error::Error` impl — `format!("{}", err)` does not compile at the top level; `?`-propagation to `Box<dyn Error>` works only via Rust's automatic `From` plumbing.
- `lib.rs` is 2 lines with no convenience re-exports — every consumer writes the full `xyntra::ir::types::NodeID` path.

**Testing:**

- Zero tests for `config` (7 validation branches), `errors` (9 Display arms), or `validation` (every method `todo!()`).
- No property-based or fuzz tests despite invariants (acyclicity, unique IDs, shape compatibility) that would benefit from `proptest` / `quickcheck` / `cargo-fuzz`.
- No CI — the `XyntraConfig` visibility bug has not been caught by automation because none exists.
- `TensorShape::size()` overflow with `usize::MAX` not tested.

**Documentation:**

- No `LICENSE` file despite README promising MIT + Apache-2.
- No `CHANGELOG.md`, no `CONTRIBUTING.md`, no `examples/`, no architecture diagram, no public rustdoc on production code.
- README marks 6 items `[x]` that are not done or not started — the single most misleading surface in the repo per the LifeOS "Reality vs README" audit.

**Portfolio-accuracy risk** (explicit warning in LifeOS Gaps):

> An external reader browsing the repo will see "automatic kernel-fusion compiler pass" in the description and `[x]` markers across IR, errors, config, modular layout, and golden-output tests. Actual compiler behaviour: zero. Misrepresentation risk is the single most important gap to close before resuming.

## Direction (in-flight, not wishlist)

Nothing is actively in flight — the project is dormant. The LifeOS Roadmap lays out a phased restart plan; only Phase 0 and Phase 1 represent concrete near-term work, the rest is aspirational backlog:

- **Phase 0 — Repo hygiene (one afternoon)** — add `pub` to `XyntraConfig`; remove `main.rs`'s duplicate `pub mod ir;`; add `#[derive(PartialEq, Eq, Clone)]` to `OpKind`; add `Display` and `std::error::Error` impls for the four error enums; rewrite the README with honest markers; add `cargo check` + `cargo test` GitHub Action; add a `LICENSE`; add tests for `XyntraConfig::validate` (7 branches) and `ValidationError::Display` (9 arms).
- **Phase 1 — Implement the validator (1–3 days)** — add `Graph::iter_nodes`; implement `validate_node_references` (linear scan), `detect_cycles` (DFS with white / grey / black colouring), `validate_operation_constraints` (static `OpKind → expected_input_count` table: MatMul=2, Add=2, Gelu=1, Softmax=1, LayerNorm=1, Dropout=1, Custom=any); orchestrate via `validate()` and `combine_results`; decide the fate of the dead `outputs` field.

Phases 2–6 (CLI + `clap` + `toml`; ONNX ingestion via JSON-first then `tract-onnx`; one hand-rolled `MatMul → Gelu` fusion; minimum WGSL codegen verified against a NumPy-equivalent reference; PTX / autotuner / occupancy / plugins) are roadmap items, not in-flight work. The LifeOS Roadmap explicitly flags the ambition-vs-momentum mismatch: the README describes 6–12 months of team effort; peak velocity on this project was 6 days producing the IR.

## Demonstrated skills

What this specific project, in its current state, proves the user can do:

- **Designs a typed Rust IR from scratch.** Newtype primitives with derived `Hash`+`Eq` for HashMap-keying, enum-based op vocabularies, monotonic-counter ID allocation, inputs-only DAG encoding — all idiomatic, all flowing acyclically through the module tree.
- **Reasons about compiler-frontend architecture.** Identifies the right layers (types → ops → graph → validation → errors → config) and the right dependency direction. Anticipates downstream needs (error variants for parsing / codegen / system stages that have no caller today) by scaffolding the taxonomy before the consumers exist.
- **Writes systems Rust under a strict `unsafe`-free constraint.** Zero `unsafe` blocks in any current file; the constraint is honoured trivially today but the design positions the project for safe-by-default GPU work via `wgpu`'s safe public API.
- **Implements GPU-parameter validation grounded in CUDA conventions.** Power-of-2 + bound checking with CUDA-programming-guide-aligned defaults (16×16 tiles, 256-thread blocks, ranges scaled to typical `maxThreadsPerBlock` limits). Shows familiarity with the warp-size / shared-memory mental model even before any GPU code is written.
- **Builds substantial test infrastructure with reusable fixtures.** 730 lines of test code, fixture builders (`build_simple_graph`, `build_complex_graph`), parametrised iteration over all op variants, a 1000-node stress test characterising the intended scale envelope. Test:source byte ratio ~1.9:1.
- **Practises evidence-anchored honesty about scope.** The LifeOS "Reality vs README" audit is itself evidence — Caner systematically reconciles every README claim against the code, classifies the gap (Aspirational / Partially-true / False / Verified), and surfaces it rather than letting the README's pitch language stand. This is a stronger portfolio signal than the README itself.
- **Recognises the dormant-vs-active distinction.** The LifeOS source explicitly flags the 9-month gap and frames a restart as a deliberate decision rather than drift, with Phase 0 (repo hygiene) sized as one afternoon to buy a credible floor regardless of whether the rest of the roadmap is pursued.

What this project does **not** yet prove:

- Compiler-pass implementation (the validator is scaffolded, not built).
- ONNX or TorchScript ingestion (no parsing code exists).
- Pattern-matched graph rewriting or e-graph use (no rewrite engine, no `egg`).
- GPU kernel codegen — WGSL or PTX (no shader emission code, no `wgpu` integration).
- Autotuning, occupancy analysis, or any runtime measurement (no benchmarks).
- CI / release engineering (no workflows, no LICENSE file).
- Cross-language plugin integration (no PyTorch / ONNX Runtime hooks).
