---
name: Vynapse
status: paused
source_repo: https://github.com/Capataina/Vynapse
lifeos_folder: Projects/Vynapse
last_synced: 2026-04-26
sources_read: 13
---

# Vynapse

## One-line summary

A from-scratch Rust evolutionary-algorithm trainer for fixed-topology neural networks, built as a 4-crate workspace with a trait-based modular architecture and a 10-milestone roadmap toward hybrid deep learning — Milestone 1 (~90% complete) is the only paradigm currently implemented end-to-end, with NEAT, SGD, autodiff, and hybrid trainers scaffolded as 16 empty stub files registered in the module tree.

## What it is

Vynapse is Caner's from-scratch Rust ML engine. It is **designed** as a "hybrid learning runtime — unifying the optimization paradigms of PyTorch, TensorFlow, DEAP, and NEAT" per the README's opening pitch; what it **currently demonstrates** is one paradigm working: fixed-topology neuroevolution with tournament selection, Gaussian mutation, and uniform crossover, evaluated against toy tasks (PowersOfTwo, XOR). The trait-based component architecture is the substantive design output — `EvolutionaryTrainer<G, M, C, F, S>` is generic over five trait-bounded type parameters so future trainers (NEAT, SGD, hybrid) can reuse the trainer-agnostic `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, and `TrainingStats` infrastructure without rewriting the scaffolding. The 16 empty-but-`mod`-registered stub files (`trainers/neat.rs`, `trainers/sgd.rs`, `trainers/hybrid.rs`, etc.) are deliberate declarative scaffolding — the module tree encodes the 10-milestone plan as visible structure, not as TODO comments.

The repository was started 12 July 2025, hit a five-month silence after late July, then landed a focused December 2025 modular-architecture refactor (HEAD `1c01e02`, 21 December 2025). HEAD is approximately four months old as of LifeOS sync — the project is bursty rather than continuous, and LifeOS classifies its current state as paused-but-revivable. When describing Vynapse externally, LifeOS Overview explicitly directs against quoting the README's opening line and toward: *"a from-scratch Rust evolutionary-algorithm trainer with a roadmap toward hybrid deep learning."*

## Architecture

Four crates with strict dependency direction. `vynapse-common` depends only on `thiserror`. `vynapse-math` depends on `vynapse-common` + `num-traits`. `vynapse-core` depends on `vynapse-math` + `vynapse-common` + `rand` + `rand_distr` + `serde` + `thiserror` (serde is declared but currently unused — pre-imported for the Milestone 2 configuration layer). `vynapse-cli` depends on `vynapse-core` + `vynapse-math` + `clap = 4.5.41`, but its `main.rs` is currently a 44-byte `fn main() { println!("Hello World!"); }` — there is no working CLI.

```
                ┌───────────────────┐
                │  vynapse-common   │   error types, Result alias
                │   (4 files, 12KB) │   thiserror only
                └─────────┬─────────┘
                          │
            ┌─────────────┴─────────────┐
            ▼                           ▼
   ┌──────────────┐            ┌───────────────┐
   │ vynapse-math │            │ vynapse-core  │
   │ (9 files,    │◄───────────┤ (58 files,    │
   │  48.7 KB)    │            │  89.3 KB)     │
   │ Shape/Tensor │            │ traits/       │
   │ ops/arith    │            │ components/   │
   │ ops/linalg   │            │ trainers/     │
   │ ops/transform│            │ training_setup│
   └──────────────┘            │ tasks/ config/│
          ▲                    │ utils/        │
          │                    └───────┬───────┘
          │                            │
          └────────────┬───────────────┘
                       │
              ┌────────▼────────┐
              │  vynapse-cli    │   clap declared, unused
              │  main = "Hello  │
              │  World!"        │
              └─────────────────┘
```

Inside `vynapse-core`, the layout follows a deliberate three-layer split:

- **`traits/`** (9 files) — the extensibility contract: pure trait interfaces (`Genome`, `Mutation`, `Crossover`, `Selection`, `Fitness`, `Loss`, `Activation`, `Task`, `Trainer`). Eight traits require `Clone + Debug`; `Trainer` is the exception (live mutable state should not be cloned).
- **`components/`** — concrete implementations of the traits, one per file. Six implemented (`Sigmoid`, `MSE`, `GaussianMutation`, `UniformCrossover`, `TournamentSelection`, `TaskBasedFitness`, `FixedTopologyGenome`); ten stubbed at 0 bytes (`neat_genome`, `uniform` mutation, `arithmetic` crossover, `fitness_proportionate` and `roulette` selection, four config structs, two utils).
- **`trainers/`** — orchestration, one paradigm per file. `evolutionary.rs` (12.4 KB) is the only implemented trainer; `deap.rs`, `neat.rs`, `sgd.rs`, `hybrid.rs`, `static_graph.rs` are 0-byte stubs.
- **`training_setup/`** — trainer-agnostic infrastructure produced by the December 2025 refactor: `EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`. The refactor's intent is that future trainers (SGD, NEAT) reuse `FitnessStats` and `TrainingStats` unchanged; `Population<G>` will need extension for NEAT's species-aware structure.
- **`tasks/`** — `powers_of_two.rs` and `xor.rs` implemented; `cartpole.rs` is 0 bytes (Milestone 4).

The trainer's generation loop runs: evaluate every genome via the fitness function → update `FitnessStats` and `TrainingStats` (which bumps the stagnation counter and possibly transitions `ConvergenceStatus`) → select N parents via the selection function → generate N offspring by pairing parent `i` with parent `(i+1) mod N`, applying crossover (gated by `crossover_rate`, else cloning parent1) then mutation → fully replace the population (no elitism).

Cross-cutting Rust properties: zero `unsafe`, no `Arc`/`Rc`/`RefCell`/interior mutability anywhere — the entire engine is owned data flowing through `&mut self` methods. `.clone()` is used liberally (every `get_weights()` returns owned `Vec<f32>`, `step()` clones the population-sized vector for selection, `train()` and `step()` return owned `TrainingStats`). At population=500 with ~12 weights per genome this is negligible; at MNIST scale (Milestone 5+) the clone-heavy API becomes a measurable cost.

## Subsystems and components

### `vynapse-math` — tensor primitives

`Shape { dims: Vec<usize> }` rejects any zero dimension at construction. `Tensor<T> { data: Vec<T>, shape: Shape, strides: Vec<usize> }` is row-major contiguous storage with strides computed once at construction. Constructors: `new` (`Default`), `zeros` (`num_traits::Zero`), `ones` (`num_traits::One`), `from_vec` (validates `data.len() == total_elements`).

Arithmetic ops (`tensor_add`, `tensor_sub`, `tensor_mul`, `tensor_div`) are elementwise and require exact shape match — no broadcasting. `tensor_div` pre-scans the divisor tensor for zero elements and refuses element-level if any zero exists. Linear algebra is one primitive only: `matrix_vector_mult(matrix, vector)` — no `matrix_matrix_mult`/GEMM, which forecloses batched forward passes and is a Milestone 5 blocker. Transforms: `transpose_2d` (rank-2 only) and `reshape_tensor` (always clones data; no NumPy-style view). Generics scale to per-op requirements (`Zero` for `add`, `PartialEq` for `div`, etc.).

Test coverage: 19 tests in arithmetic, 4 in linalg, 31 in transform; `shape.rs` and `tensor.rs` themselves have 0 direct tests (bounds-check paths covered indirectly via ops tests).

### `vynapse-common` — error types

`VynapseError` has three variants — `TensorError(String)`, `EvolutionError(String)`, `ConfigError(String)` — derived via `thiserror::Error`. `pub type Result<T> = std::result::Result<T, VynapseError>`. No structured error data, no `#[source]` attribute, no source-chaining: `thiserror` is used only for `Display`/`Error` derivation. Variant choice is currently inconsistent (`TournamentSelection::new(0)` returns `EvolutionError` while `Population::new(0)` returns `ConfigError` — both are construction-time invalid parameters).

### `vynapse-core/traits/` — the extension surface

Nine flat traits. Eight require `Clone + Debug`; `Trainer` does not. `Genome` exposes only `get_weights/set_weights` (returning owned `Vec<f32>`). `Mutation::mutate<G: Genome>(...)`, `Crossover::crossover<G: Genome>(...)`, and `Fitness::evaluate<G: Genome>(...)` are generic over the genome type at the method level. `Selection::select(fitness, count) -> Vec<usize>` returns indices, not genomes — keeping selection pure with no population borrow. `Task::get_dataset` returns `Vec<(Vec<f32>, Vec<f32>)>` materialised eagerly; this is a Milestone 5 blocker for MNIST-scale data. `Trainer` defines `train`, `step`, `get_stats`, `reset`, `is_converged`, `validate_config`. Conspicuously absent: no `Serializable` trait (no checkpointing), no `Send + Sync` bounds (no parallelism), no async, no trait hierarchy.

### `vynapse-core/components/` — concrete implementations

| Component | File | Trait | Tests |
|---|---|---|---|
| `Sigmoid` | `activation/sigmoid.rs` (2.1 KB) | `Activation` | 4 |
| `MeanSquaredError` | `loss/mse.rs` (2.1 KB) | `Loss` | 4 |
| `GaussianMutation` | `mutation/gaussian.rs` (1.6 KB) | `Mutation` | 0 (covered indirectly) |
| `UniformCrossover` | `crossover/uniform.rs` (1.6 KB) | `Crossover` | 0 (covered indirectly) |
| `TournamentSelection` | `selection/tournament.rs` (11.8 KB) | `Selection` | 24 |
| `TaskBasedFitness` | `fitness/task_based.rs` (15.2 KB) | `Fitness` | 13 |
| `FixedTopologyGenome` | `genome/fixed_topology.rs` (3 KB) | `Genome` | 7 |

`FixedTopologyGenome` stores weights as a flat `Vec<f32>` plus a `shape: Vec<usize>`. Weight count = `Σ shape[i] * shape[i+1]` — inter-layer weights only, **no biases**. Weights are sampled uniformly from `[-1.0, 1.0]` via `rng.random_range(-1.0..=1.0)`. `set_weights` is length-validated and is the invariant that prevents mutation/crossover from breaking genome size.

`GaussianMutation` adds `N(0, sigma)` noise to each weight with per-weight probability `rate`, then clamps to `[-5.0, 5.0]` (undocumented except in code). `UniformCrossover` selects each gene from parent1 with probability `inheritance_probability ∈ [0.0, 1.0)` (right-open — `1.0` is rejected), else parent2; the child starts as a `parent1.clone()` and has its weights overwritten. `TournamentSelection` samples `tournament_size` indices with replacement from the fitness array and returns the argmax — the most heavily tested component in the codebase (24 tests, 11.8 KB).

`TaskBasedFitness<T: Task, L: Loss, A: Activation>` is the forward pass: it reconstructs a 2-layer MLP on the fly from the genome's flat weights, runs every `(input, expected)` pair through `matrix_vector_mult → activation → matrix_vector_mult → activation`, sums per-sample loss, and returns `1.0 / (1.0 + total_error)` (bounded in `(0, 1]`). The hidden layer size is **hardcoded to 4** at `task_based.rs:212` regardless of `genome.shape`; the activation is applied to **both** hidden and output layers (so output is always in sigmoid's `(0, 1)` range, fatal for regression tasks like PowersOfTwo with targets `{1, 2, 4, 8, 16}`).

### `vynapse-core/training_setup/` — trainer-agnostic infrastructure

`EvolutionConfig { generations: u32, stagnation_limit: u32 }` — only trainer-level coordination parameters. `Population<G> { genomes: Vec<G>, population_size: usize, expected_size: usize }` — `init_from_template(template, mutation, rate)` clones the template once unchanged then pushes `N-1` mutated clones (so `genomes[0]` is the unmutated template, biasing initial diversity). `GeneticOperators<M, C> { mutation, crossover, mutation_rate, crossover_rate }` — `apply_crossover` rolls a random `f32` against `crossover_rate` and either clones parent1 or runs the actual crossover; rates must be in `[0.0, 1.0]` inclusive.

`FitnessStats` tracks `best`/`average`/`worst`/`fitness_history`/`current_generation_fitness`, initialised to `f32::NEG_INFINITY` as a sentinel. `TrainingStats` embeds `FitnessStats` plus generation counter, `ConvergenceStatus` enum (`WaitingToStart`, `Running`, `TargetReached`, `Stagnated`, `MaxIterationsReached`), `Instant`-based timer, and stagnation counter. `TargetReached` is declared but never assigned anywhere in the code — it is a roadmap artefact awaiting a `target_fitness` config field.

### `vynapse-core/trainers/evolutionary.rs` — the only working trainer

Five generic type parameters: `EvolutionaryTrainer<G: Genome, M: Mutation, C: Crossover, F: Fitness, S: Selection>`. Constructor takes pre-built `EvolutionConfig`, `Population`, `GeneticOperators`, `Fitness`, `Selection`; builds `FitnessStats` and `TrainingStats` inline. Implements the six `Trainer` trait methods. The `step()` loop has **no elitism** — `set_all_genomes(new_population)` fully replaces the previous generation, even though the README's Milestone 1 checklist claims `(μ + λ)` replacement is complete (it is not). Parent pairing is positional (`i` paired with `(i+1) mod N` from the selection output), not random.

## Technologies and concepts demonstrated

### Languages
- **Rust** — the only implementation language; 66 source files, 115.4 KB. Zero `unsafe` blocks. Uses generics + monomorphisation as the abstraction strategy (no `dyn` trait objects, no boxed components, no enum-of-variants). 120 inline `#[test]` functions across 12 files; no `tests/` integration directory.

### Frameworks and libraries
- **`thiserror = "2.0.12"`** — error-type derivation (`#[derive(thiserror::Error, Debug)]`). Used only for `Display`/`Error` derivation; source-chaining features unused.
- **`num-traits = "0.2.19"`** — generic numeric trait bounds (`Zero`, `One`) on tensor primitives.
- **`rand = "0.9.1"`** + **`rand_distr`** — random number generation; every component currently calls `rand::rng()` thread-local, no shared seeded RNG.
- **`serde = "1.0.219"`** — declared in `vynapse-core/Cargo.toml` but completely unused in source. Pre-imported for the Milestone 2 JSON configuration layer.
- **`clap = "4.5.41"`** — declared in `vynapse-cli/Cargo.toml` but completely unused; `main.rs` is `Hello World!`.

### Runtimes / engines / platforms
- **Native Rust workspace** — Cargo multi-crate workspace with strict dependency direction (4 crates: `vynapse-common`, `vynapse-math`, `vynapse-core`, `vynapse-cli`). No GPU backend, no SIMD, no parallelism; everything runs single-threaded on the CPU.

### Tools
- **`cargo` workspace** — 4 crates configured in the root `Cargo.toml`.
- **`gh` CLI / `git`** — used by Caner's repo-inspection scripts (`fetch_commits.py`, `repo_stats.py`, `search_content.py`) referenced in LifeOS evidence chains; not part of the project's own build.
- **Plans-driven workflow** — `plans/` folder convention introduced 19 December 2025 with `plans/evolutionary-trainer-modular-refactor.md` as the single source of truth for that work unit. Mirrors Caner's broader LifeOS working style.

### Domains and concepts
- **Evolutionary algorithms (fixed-topology neuroevolution)** — implements an end-to-end EA: tournament selection, Gaussian mutation, uniform crossover, generational replacement. Calibrated against the textbook against actual code: parent pairing is positional (subtle bias), no elitism (selection pressure is the sole convergence mechanism), tournament samples with replacement.
- **Trait-based modular design in Rust** — `EvolutionaryTrainer<G, M, C, F, S>` is generic over five type parameters with trait bounds, monomorphised at compile time (zero-cost abstraction). Decision explicitly documented: rejected `dyn` trait objects (heap allocation, dynamic dispatch, inlining loss) and rejected enums-over-variants (closed to external extension).
- **Workspace multi-crate organisation with explicit dependency direction** — the 4-crate split with one-way deps is itself a portable pattern.
- **Test-colocation discipline** — 120 `#[test]` functions sit in `#[cfg(test)] mod tests { ... }` blocks at the bottom of the source files they test. Decision: prioritises private-internals access and proximity over public-API-only integration testing.
- **Result-based error handling** — every fallible function returns `Result<T>`; no panics in library code (only in `#[test]` blocks and one safe `.unwrap()` after explicit `is_none()` check).
- **Forward pass of a 2-layer MLP from a flat weight vector** — reshape weights into `[hidden, inputs]` and `[outputs, hidden]` matrices, run `matrix_vector_mult → activation → matrix_vector_mult → activation`.
- **EA hyperparameter routing** — design rule from the December refactor: hyperparameters belong to whichever component their behaviour affects (population size on `Population`, mutation/crossover rates on `GeneticOperators`, generations/stagnation_limit on `EvolutionConfig`).
- **State machine for run lifecycle** — `ConvergenceStatus` enum with explicit transitions (`WaitingToStart → Running → {Stagnated, MaxIterationsReached, TargetReached}`).
- **Numerical safety patterns** — sigmoid implementation has a defensive NaN/Inf guard against `f32::exp()` edge cases; mutation clamps weights to `[-5.0, 5.0]`; `tensor_div` pre-scans for zero divisors.

## Key technical decisions

**Modular refactor (December 2025).** Split the monolithic `EvolutionaryTrainer` into five composed components (`EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`). Pre-refactor, the trainer held inline `TrainingStats`, owned mutation/crossover rates and population size directly, and mixed orchestration with population management. Rejected alternatives (per the plan file): keeping `population_size` on `EvolutionConfig` ("population_size belongs to Population — Population *is* the collection"), keeping `mutation_rate`/`crossover_rate` on `EvolutionConfig` ("rates bind with their operators"), having `Trainer::get_stats` return `&TrainingStats` ("acceptable for MVP" to clone). The refactor's purpose was preparing the codebase for the 10-milestone roadmap — pre-refactor, the trainer could not be reused for SGD or NEAT because loop-logic and component-logic were intertwined.

**Trait-based modularity (July 2025).** Every concrete component implements a thin trait; the trainer is generic over five trait-bounded type parameters. Rejected: `dyn` trait objects (heap allocation per operator, dynamic dispatch cost, loss of inlining), enums-over-all-components (requires editing the enum to add new components — closed to external extension), composition without traits (would lock the trainer to one combination). Rust generics + monomorphisation give extensibility without runtime cost; the cost is longer type signatures in tests.

**No elitism in MVP.** Every `step()` calls `set_all_genomes(new_population)` — full replacement, zero carryover. Best individual of generation *k* has no guaranteed presence in *k+1*. Rejected alternatives: proper `(μ + λ)` (evaluate `parents ∪ offspring`, keep best N), steady-state replacement, k-elitism. Why MVP: elitism requires retaining previous-generation genomes after selection, and the current `set_all_genomes(Vec<G>)` API discards them. **Critical README/code discrepancy**: the README marks `(μ + λ)` replacement as `[x]` complete; this is factually incorrect. A reader taking the README at face value will believe elitism is implemented when it is not.

**Hardcoded hidden layer (July 2025).** `TaskBasedFitness::evaluate` hardcodes `let hidden_size = 4;` at `task_based.rs:212`, ignoring the genome's `shape` field. Introduced in commit `9ca58e4` (19 July 2025) with the truncated commit message confirming hardcoding was known and accepted as MVP simplicity. Rejected alternatives: read from `genome.shape()` (would require extending the `Genome` trait), const generic `<const HIDDEN: usize>` (still rigid), constructor parameter (most flexible — the obvious eventual fix).

**All-`f32` numerics.** Entire ML path uses `f32`. No `f64`, no mixed precision, no `T: Float` generic. Rejected: `f64` for accuracy (doubles memory, halves SIMD throughput, inconsistent with GPU norms), generic over `T: Float` (propagates a parameter through every component, complicates sigmoid). Why: `f32` is the ML standard and matches PyTorch/TensorFlow defaults.

**Inline tests, not `tests/` folder.** Every `#[test]` colocated in `#[cfg(test)] mod tests { ... }` blocks at the bottom of the source file it tests. 120 tests, 0 dedicated test files. Rationale: inline tests share scope with the code they test (private fields, helper functions, private constructors); for a fast-moving MVP, the friction of separate tests directories outweighs the public-API-only benefits.

**Tensor crate from scratch (vs `ndarray` / `nalgebra` / `candle`).** `vynapse-math` is built from scratch rather than using existing Rust ML/numerics crates. Why: this is **a learning project** — the explicit point is to implement the stack, not consume a library. The README states "Built from the ground up in safe, parallel Rust — no Python bindings, no unsafe blocks." Re-using `candle` would skip Milestones 1, 6, 7, 9. Cross-project note: `Aurix` (Caner's other from-scratch Rust numerics project) duplicates tensor primitives — a shared `ata-math` crate is a future organisational decision, not yet made.

**Plans-driven development.** Introduced 19 December 2025 with commit `2419eee`. Any non-trivial refactor or feature is preceded by a plan document (`NNN_<topic>.md`) capturing requirements, scope, files touched, action items, risks, and design decisions. Rejected: working from commit messages only (used for the first 22 commits — impossible to retroactively see the shape of an intended change), GitHub Issues (overhead for a single-contributor project), inline TODO comments (zero TODOs in the codebase). The plan + checklist format mirrors Caner's broader LifeOS working style and the same pattern is used in Cernio and LifeOS itself.

**Serde declared but unused.** `vynapse-core/Cargo.toml` includes `serde = "1.0.219"` with zero usages. Anticipatory pre-import for Milestone 2 (JSON configuration loading); self-resolving when the first config struct lands.

## What is currently built

One paradigm — **fixed-topology evolutionary neuroevolution** — works end-to-end. A future session can run, today, this exact training setup (verified to compile and run via the integration test `test_evolutionary_trainer_learns_powers_of_two` at `evolutionary.rs:1711-1843`):

- Population of 500 `FixedTopologyGenome` instances, shape `[1, 4, 1]` (8 weights each, no bias).
- Tournament selection with `tournament_size = 3`.
- Gaussian mutation (`σ = 0.15`, per-weight `rate = 0.8`, clamped to `[-5.0, 5.0]`).
- Uniform crossover (`inheritance_probability = 0.5`, applied at `rate = 0.7`).
- 100-generation cap with `stagnation_limit = 20`.
- `TaskBasedFitness<PowersOfTwo(5), MSE, Sigmoid>` — reconstructs a 2-layer MLP, runs the dataset, returns `1 / (1 + total_MSE)`.

What is NOT built:

- **No CLI.** `vynapse-cli/src/main.rs` is 44 bytes of `Hello World!`. The Milestone 1 deliverable (`vynapse train --task powers_of_two --generations 100`) does not exist. Invoking training requires writing Rust code or running the colocated integration test via `cargo test --release`.
- **No configuration loading.** All four `config/*.rs` files are 0 bytes; serde is declared but unused; nothing parses or validates JSON.
- **No CSV logging.** Per-generation observability exists only as `println!` calls inside the integration test.
- **No deterministic seeding.** Every component creates its own thread-local `rand::rng()`; runs are not reproducible.
- **No elitism.** Best individual is not preserved across generations.
- **No batched matmul.** Only `matrix_vector_mult` exists; no `matrix_matrix_mult`/GEMM.
- **No biases on `FixedTopologyGenome`.** Inter-layer weights only.
- **Sixteen 0-byte stub files** registered in their `mod.rs`: 5 trainer stubs (`deap`, `hybrid`, `neat`, `sgd`, `static_graph`), 1 genome stub (`neat_genome`), 1 mutation stub (`uniform`), 1 crossover stub (`arithmetic`), 2 selection stubs (`fitness_proportionate`, `roulette`), 4 config stubs, 2 utils stubs (`metrics`, `rng`), 1 task stub (`cartpole`).

Scale markers (from LifeOS Overview): 66 Rust source files, 115.4 KB of source, 4 crates, 0 dedicated test files (tests are inline), 120 `#[test]` functions across 12 files, 16 empty stub files, 36 total commits (HEAD `1c01e02`), first commit 12 July 2025, HEAD 21 December 2025. The largest source file is `vynapse-math/src/ops/transform.rs` at 16 KB; the largest trainer file is `trainers/evolutionary.rs` at 12.4 KB.

## Current state

Status: **paused** (LifeOS Overview classifies it as bursty rather than continuous; HEAD is 21 December 2025, approximately four months stale at LifeOS sync date 24 April 2026). The December 2025 return-to-project was a deliberate restructuring pass — three commits in three days (`2419eee` plan document, `456d9d1` modular refactor execution, `1c01e02` validation-signature cleanup) — followed by another silence. Commit timeline: 32 commits in the first 17 days (12–29 July 2025), then one commit across July → December (the five-month silence is real data, not drift), then three commits 19–21 December 2025. No active in-flight work captured in LifeOS Work/ (the folder does not appear in the synced enumeration). The codebase is fully revivable at any moment — clean modular architecture, plan files in place, all acceptance criteria for the December refactor marked complete.

## Gaps and known limitations

**Documentation vs reality**

- README opening pitch ("hybrid learning runtime unifying PyTorch, TensorFlow, DEAP, and NEAT") describes Milestone 10, not the current build. A reader taking the README at face value overestimates the current engine by ~9 milestones. LifeOS explicitly directs against quoting that line externally.
- README marks `(μ + λ)` population replacement as complete; the code implements full generational replacement with no carryover. The README is factually incorrect on this point.
- README also claims "simple elitism preservation" complete; no elitism exists in code.
- README marks deterministic seeding as unchecked (this one is honest).

**Architectural gaps**

- **Missing elitism** at `evolutionary.rs:1593` — fix is ~10 lines (sort fitness indices, take top-k, prepend to new_population before generating offspring).
- **Parent pairing is positional, not random** at `evolutionary.rs:1577-1581` — child `i`'s parents are the `i`-th and `(i+1)`-th elements of the selection output, introducing a locality bias.
- **Hidden size hardcoded** at `task_based.rs:212` (`let hidden_size = 4;`) — changing network width requires editing the fitness function. A genome with `shape = [2, 8, 1]` will fail validation because the function expects `2*4 + 4*1 = 12` weights regardless.
- **No biases in `FixedTopologyGenome`** — XOR is borderline-solvable without biases; any real regression task needs them.
- **Activation applied to output layer** at `task_based.rs:250-251` — sigmoid clamps every output to `(0, 1)`, fatal for regression. PowersOfTwo with targets `{1, 2, 4, 8, 16}` is mathematically impossible to learn; the integration test measures whether the population can reach `~0.0035` fitness (sigmoid-saturated baseline) vs noise.
- **Saturated benchmarks** — only XOR (borderline-solvable) and PowersOfTwo (impossible to learn given the architecture). No regression task that fits the sigmoid output range.

**State-machine gaps**

- `is_converged()` returns `true` for fresh `WaitingToStart` state — `training_stats.rs:2633` defines it as `status != Running`, so a never-started trainer reports as "done."
- `step()` succeeds post-convergence — `update_generation` only rejects `WaitingToStart`, so for `MaxIterationsReached`/`Stagnated` it bumps the generation past the limit.
- `FitnessStats::validate()` requires non-empty `fitness_history` and `current_generation_fitness` — fails on a fresh struct. The trainer's `reset()` calls `fitness_stats.reset()` which calls `self.validate()` first, so `reset()` on a fresh trainer errors. Latent (not exercised by tests).
- `TargetReached` is a dead enum variant — declared, never assigned anywhere in the code.

**Algorithmic gaps**

- Stagnation check uses strict-inequality (`last > max_of_rest`) — a tie/plateau counts as non-improvement, terminating early.
- Weight clamp at `[-5.0, 5.0]` is undocumented except in code; fixed (not configurable).
- Fitness formula uses `1/(1+total_error)` (sum, not mean) — for larger datasets the denominator grows, making fitness approach zero.

**Trait-surface gaps**

- `Task::get_dataset()` returns owned `Vec<(Vec<f32>, Vec<f32>)>` — eager materialisation per call, called once per genome per generation. Catastrophic at MNIST scale (60k×784 = ~180 MB allocated 10,000 times in a 100-gen, pop=100 run). Hard blocker for Milestone 5.
- `Genome::get_weights()` returns owned `Vec<f32>` — allocates per call.
- No `Serializable` trait — no checkpointing.
- No `Send + Sync` bounds — no parallelism.
- No async — RL rollouts may block.

**Test-coverage gaps**

- `GaussianMutation` and `UniformCrossover` have **zero** `#[test]` functions (covered indirectly via trainer tests).
- `Shape::new` and `Tensor::get/set` bounds-check error paths not directly tested.
- `EvolutionConfig::validate` and `Population::validate` only partially covered.
- The integration test's "learning" assertion is `final_best >= initial_best` — passes trivially for any non-regressing run; does not verify the network actually learns anything meaningful.

**Cross-project duplication**

- `Aurix` (Caner's other from-scratch Rust numerics project) duplicates tensor/shape/ops primitives. No mechanism to share; consolidation question deferred.

## Direction (in-flight, not wishlist)

LifeOS Roadmap captures **no active in-flight work** as of the 24 April 2026 verification — HEAD has been static since 21 December 2025 and the project is classified as bursty/paused. The "real next step" identified by LifeOS Roadmap (if Caner picks the project up) is closing Milestone 1: deterministic seeding (1 hour — introduce a shared `SeedableRng` threaded through operators), build the CLI (2-3 hours — `clap` is already a declared dependency), CSV logging (1 hour — per-generation `(generation, best, avg, worst, elapsed_ms)` to a configurable path), and optionally adding elitism (30 minutes — preserve top-1 across generations and align code with the README's `(μ + λ)` claim).

After Milestone 1 closes, the roadmap's natural sequence is Milestone 2 (configuration layer) → Milestone 3 (DEAP-style population algorithms) → Milestone 4 (NEAT topology evolution). Milestones 5–8 (SGD, autodiff, static graph, hybrid) are larger jumps and depend on gradient infrastructure that does not yet exist; LifeOS Roadmap notes Milestone 6 (autodiff) as the largest single jump — comparable in complexity to all of `vynapse-math + training_setup` combined.

## Demonstrated skills

- **Designing trait-based extensible architectures in Rust.** Nine traits define the extension contract; the trainer is generic over five trait-bounded type parameters monomorphised at compile time. Each design alternative was considered and rejected with reasoning recorded in LifeOS Decisions (`dyn` trait objects → heap allocation cost; enums → closed to external extension; hardcoded composition → locks to one combination).
- **Multi-crate Cargo workspace organisation with strict dependency direction.** 4-crate split (`vynapse-common` ← `vynapse-math` ← `vynapse-core` ← `vynapse-cli`), one-way dependency arrows, each crate's purpose isolated. Strict trait-bound minimalism (each op declares only the traits it strictly needs — `Zero` for `add`, `PartialEq` for `div`).
- **Implementing an end-to-end evolutionary algorithm from scratch.** Tournament selection (with replacement, tested for selection-pressure characteristics across `tournament_size = 1..population_size`), Gaussian mutation, uniform crossover, generational replacement — all written without an EA library dependency.
- **From-scratch tensor implementation.** `Shape`, `Tensor<T>` with row-major contiguous storage and computed strides, generic arithmetic ops (`add`/`sub`/`mul`/`div`) over `T: Add + Zero + Clone`, `matrix_vector_mult`, `transpose_2d`, `reshape_tensor`. No NumPy, no `ndarray`, no `nalgebra`.
- **Result-based error handling discipline.** Every fallible function returns `Result<T>`; the codebase has zero panics in library code (only safe `.unwrap()`s after explicit `is_none()` checks plus test-only unwraps).
- **Test-colocation discipline at scale.** 120 `#[test]` functions across 12 files, all colocated with the code they test. Test design includes invariant-based checks (e.g., the "all-zero weights on XOR gives fitness ≈ 0.5" test, which simultaneously validates Shape, Tensor::from_vec, matrix_vector_mult, Sigmoid, MSE, and the fitness combine formula).
- **Plans-driven engineering workflow.** Every non-trivial refactor/feature is preceded by a plan document (`NNN_<topic>.md`) capturing requirements, scope, files touched, action items, risks, design decisions; plan files become the single source of truth for that work unit. Mirrors the same pattern in Cernio and LifeOS.
- **Architectural restructuring under self-direction.** The December 2025 modular refactor split a monolithic trainer into five composed components, written, planned, and executed in three commits over three days with explicit reasoning for every routing decision (which hyperparameter belongs where).
- **Forward-pass implementation of a 2-layer MLP from a flat weight vector.** Reshape into matrices, run two `matrix_vector_mult → activation` rounds, sum loss across samples, return scalar fitness. Not deep learning — but the primitive is a real implementation, not a library wrapper.
- **Honest scoping of project state.** LifeOS captures the README/code discrepancy explicitly (Overview's "honest one-liner", warnings against quoting the pitch externally). The 16 empty stub files are deliberate declarative scaffolding, not abandoned work — every stub is registered in its `mod.rs` to encode roadmap intent visibly.

What this project does **not** demonstrate (calibrated honesty per LifeOS):

- Performance engineering (`.clone()` everywhere, thread-local RNG, no `Send + Sync`, no SIMD, no GPU).
- Numerical discipline at scale (hidden size hardcoded, output activation applied to regression, undocumented weight clamp).
- Testing rigour at the assertion level (the one real learning test has weak assertions; `final_best >= initial_best` passes trivially).
- Anything beyond the evolutionary paradigm — no autodiff, no SGD, no NEAT, no GPU compute, no distributed training. The scaffolding for those is visible as 0-byte files, but they are not implemented.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Vynapse/Overview.md | 125 | "Vynapse is a **from-scratch Rust evolutionary-algorithm trainer** that runs one end-to-end paradigm (fixed-topology neuroevolution with tournament selection, Gaussian mutation, uniform crossover) on toy tasks, backed by a trait-based component architecture designed to accept NEAT, SGD, autodiff, and hybrid trainers as future milestones. The scaffolding for those is visible as empty files in the module tree." |
| Projects/Vynapse/Architecture.md | 242 | "- Error propagation across crate boundaries: [[Vynapse/Systems/Error Model]]" |
| Projects/Vynapse/Decisions.md | 184 | "- The gaps these decisions leave open: [[Vynapse/Gaps]]" |
| Projects/Vynapse/Gaps.md | 268 | "- Analytical suggestions for prioritisation: [[Vynapse/Suggestions]]" |
| Projects/Vynapse/Roadmap.md | 187 | "- RL problem domain overlap with NeuroDrive (Milestone 4 cartpole): [[NeuroDrive/Overview]]" |
| Projects/Vynapse/Suggestions.md | 197 | "- Profile update implied by the project's portfolio value: [[Profile/Projects]], [[Profile/Skills]]" |
| Projects/Vynapse/Systems/Error Model.md | 114 | "- A specific latent error-handling bug: [[Vynapse/Gaps#validate() fails on fresh state]]" |
| Projects/Vynapse/Systems/Evolutionary Trainer.md | 244 | "- Why the refactor happened and what it replaced: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Genome and Components.md | 218 | "- Stubbed components as roadmap evidence: [[Vynapse/Roadmap]]" |
| Projects/Vynapse/Systems/Tasks and Fitness.md | 171 | "- Why the current benchmarks under-test learning capability: [[Vynapse/Gaps#Benchmarks are saturated]]" |
| Projects/Vynapse/Systems/Tensor and Math.md | 126 | "- Aurix also has a tensor crate — duplication decision pending: [[Aurix/Overview]], [[Vynapse/Decisions#Tensor crate vs external]]" |
| Projects/Vynapse/Systems/Training Stats and Convergence.md | 185 | "- Why these were split out from the trainer in Dec 2025: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Traits Layer.md | 185 | "- Stubs that will need new trait work: [[Vynapse/Gaps#Traits that need extension]]" |
