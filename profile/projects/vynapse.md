---
name: Vynapse
status: paused
source_repo: https://github.com/Capataina/Vynapse
lifeos_folder: Projects/Vynapse
last_synced: 2026-05-31
sources_read: 14
---

# Vynapse

## One-line summary

From-scratch Rust evolutionary-algorithm trainer for fixed-topology neural networks, organised as a 4-crate workspace with a 5-parameter generic `Trainer<G, M, C, F, S>` interface and 16 declared-but-empty stub files marking a 10-milestone roadmap toward NEAT, SGD, autodiff and hybrid learning.

## What it is

Vynapse is Caner's from-scratch Rust ML engine. One paradigm — fixed-topology neuroevolution with tournament selection, Gaussian mutation, uniform crossover, and full generational replacement — runs end-to-end against toy tasks (XOR and PowersOfTwo). The codebase is a four-crate Cargo workspace (`vynapse-core`, `vynapse-math`, `vynapse-common`, `vynapse-cli`) with strict dependency direction and a trait layer designed to accept future trainers as drop-ins. Stub files for NEAT, DEAP-style EA, SGD, static-graph execution and hybrid (Lamarckian/Baldwinian) learning sit empty in the module tree as declarative scaffolding for the roadmap. The README pitches a "hybrid runtime unifying PyTorch, TensorFlow, DEAP, and NEAT" — that is Milestone 10, not the current build; Milestone 1 is ~90% complete (CSV logging, deterministic seeding and a real CLI remain).

## Architecture

Four crates with explicit dependency direction (per LifeOS Architecture):

```
                ┌───────────────────┐
                │  vynapse-common   │   error types, Result alias
                │   (4 files, 12KB) │   no deps (only thiserror 2.0.12)
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
          └──────────────┬─────────────┘
                         ▼
                ┌────────────────┐
                │  vynapse-cli   │   clap 4.5.41 declared but
                │  main.rs =     │   main is `println!("Hello World!")`
                │  "Hello World" │   (44 bytes — no real CLI yet)
                └────────────────┘
```

- **`vynapse-common`** — declares `VynapseError` with three variants (`TensorError`, `EvolutionError`, `ConfigError`) and a `Result<T>` alias. Sole dependency: `thiserror = 2.0.12`.
- **`vynapse-math`** — pure tensor primitives with no ML concepts. Depends on `vynapse-common` plus `num-traits = 0.2.19`.
- **`vynapse-core`** — all ML logic. Depends on `vynapse-math`, `vynapse-common`, `rand = 0.9.1`, `rand_distr`, `serde = 1.0.219` (declared but unused — anticipatory pre-import for Milestone 2 JSON config), `thiserror = 2.0.12`.
- **`vynapse-cli`** — depends on `vynapse-core`, `vynapse-math`, `clap = 4.5.41`. Currently a stub.

Inside `vynapse-core`, the layout reflects the trait-first design:

- `traits/` — nine pure trait interfaces (`Activation`, `Crossover`, `Fitness`, `Genome`, `Loss`, `Mutation`, `Selection`, `Task`, `Trainer`). All are `Clone + Debug` except `Trainer`. This uniform bound is what lets `EvolutionaryTrainer<G, M, C, F, S>` parametrise over any combination.
- `components/` — concrete trait implementations. Implemented today: `activation/sigmoid.rs`, `crossover/uniform.rs`, `fitness/task_based.rs` (15.2 KB, hardcoded 1-hidden-layer MLP), `genome/fixed_topology.rs` (flat weight `Vec<f32>`), `loss/mse.rs`, `mutation/gaussian.rs`, `selection/tournament.rs` (11.8 KB, 24 tests). Empty stubs: `crossover/arithmetic.rs`, `genome/neat_genome.rs`, `mutation/uniform.rs`, `selection/fitness_proportionate.rs`, `selection/roulette.rs`.
- `trainers/` — one paradigm per file. Only `evolutionary.rs` (12.4 KB) is implemented; `deap.rs`, `hybrid.rs`, `neat.rs`, `sgd.rs`, `static_graph.rs` are all 0-byte stubs.
- `training_setup/` — trainer-agnostic infrastructure produced by the December 2025 refactor: `evolution_config.rs` (generations + stagnation limit), `population.rs` (`Population<G>` with template-based init), `genetic_operators.rs` (`GeneticOperators<M, C>` bundling mutation + crossover + rates), `fitness_stats.rs` (best/avg/worst + history), `training_stats.rs` (generation counter + `ConvergenceStatus` enum + timer).
- `tasks/` — `powers_of_two.rs`, `xor.rs` implemented; `cartpole.rs` is a 0-byte stub.
- `config/` — all four files (`experiment_config.rs`, `mutation_config.rs`, `selection_config.rs`, `trainer_config.rs`) are 0 bytes.
- `utils/` — `metrics.rs` and `rng.rs` are 0 bytes.

The 16 empty-but-declared files are not accidents. Every one is registered in its `mod.rs` as deliberate **declarative scaffolding**: the module tree encodes the 10-milestone roadmap as named future components with decided locations.

### The generation loop

`EvolutionaryTrainer::step()` (verified at `evolutionary.rs:1555-1596`):

1. Evaluate every genome: `population.get_genomes().iter().map(|g| fitness.evaluate(g))` → `Vec<f32>`.
2. Update stats: `fitness_stats.update_fitness` + `training_stats.update_generation` (bumps generation, stagnation counter, `ConvergenceStatus`).
3. Select parents via `selection.select(fitness, N)` → `Vec<usize>` of length N.
4. Generate N offspring: pair parent `i` with parent `(i+1) mod N` from the selected index list, apply crossover (gated by `crossover_rate`, else clone parent1), then mutate, push to `new_population`.
5. Replace the entire population: `population.set_all_genomes(new_population)` — full generational replacement, no elitism.

Two warnings captured in LifeOS Architecture:

- **No elitism.** The best individual of generation k has no guaranteed presence in generation k+1. Selection pressure is the only mechanism pulling good genes forward.
- **Parent pairing is positional, not random.** Child `i`'s parents are the `i`-th and `(i+1)`-th elements of the selection output, which couples offspring correlation to selection-function output order.

### Ownership shape

Nothing in Vynapse uses `unsafe`, `Arc`, `Rc`, `RefCell`, or interior mutability (LifeOS Architecture: zero matches across the codebase). The entire engine is owned data moving through `&mut self` methods. Cloning is abundant — `get_weights()` returns owned `Vec<f32>` per call; `step()` clones the population-sized `Vec<G>` for selection and clones `training_stats` to return.

## Subsystems and components

### Tensor and math (`vynapse-math`, 48.7 KB, 9 files)

- `Shape { dims: Vec<usize> }` rejects any zero dimension at construction.
- `Tensor<T> { data, shape, strides }` — row-major contiguous storage, strides computed once at construction. `from_vec` validates `data.len() == total_elements`. Access methods (`get`, `get_mut`, `set`) bounds-check rank and each index, returning `TensorError` on violation.
- Arithmetic ops in `ops/arithmetic.rs` (`tensor_add`, `tensor_sub`, `tensor_mul`, `tensor_div`) — elementwise, require exact shape match (no broadcasting). `tensor_div` pre-scans the divisor tensor for any zero and refuses element-wise if found.
- Linalg: only `matrix_vector_mult` exists. No `matrix_matrix_mult` / GEMM — sufficient for current per-input MLP forward pass but blocks batched forward passes (a Milestone 5 / MNIST blocker).
- Transforms: `transpose_2d` (rank-2 only), `reshape_tensor` (always clones data, never a view).
- Generic over element `T` with minimal bounds per op (e.g. `Zero` only where needed, `PartialEq` only for `div`'s zero-check).
- 54 inline `#[test]` functions; `ops/transform.rs` is the most heavily covered (31 tests).

### Evolutionary trainer (`trainers/evolutionary.rs`, 12.4 KB)

The orchestrator composed of the five `training_setup/` components after the December 2025 refactor. Owns: `EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`, plus the `Fitness` and `Selection` components. Public surface is the `Trainer` trait's `train`, `step`, `get_stats`, `reset`, `is_converged`, `validate_config`.

### Genome and components

`FixedTopologyGenome` stores weights as a flat `Vec<f32>`. Weight count = `Σ shape[i] * shape[i+1]` — inter-layer weights only, **no bias terms**. Borderline-solvable for XOR; insufficient for most regression. `GaussianMutation` clamps mutated weights to `[-5.0, 5.0]` (undocumented in code or README, visible only in source).

### Tasks and fitness

- `PowersOfTwo` fits `2^n` for `n ∈ {0..max_input}` (max_input ≤ 12).
- `XOR` — 2 inputs, 1 output, 4 samples.
- `TaskBasedFitness::evaluate` hardcodes `hidden_size = 4` regardless of what the genome's `shape` field reports.
- Sigmoid activation is also applied to the output layer, clamping all outputs to `(0, 1)` — fatal for PowersOfTwo regression (output `256` cannot be expressed).
- Fitness is `1 / (1 + total_MSE)`, bounded in `(0, 1]`.

### Training stats and convergence

`ConvergenceStatus` enum has 5 variants (`WaitingToStart`, `Running`, `TargetReached`, `Stagnated`, `MaxIterationsReached`). `TargetReached` is a dead variant — never set anywhere in current code (no `target_fitness` parameter exists). `is_converged()` returns true for `WaitingToStart` (a latent bug — fresh trainers report converged). Stagnation check uses strict inequality (`last > max_of_rest`), so plateau ties count as non-improvement.

### Error model

`VynapseError` is the single error type, with three variants: `TensorError`, `EvolutionError`, `ConfigError`. Every fallible function returns `Result<T>`. No panics in library code.

### Traits layer

Nine traits, each intentionally thin (≤ 371 B per trait). Only `Trainer` has non-trivial run-lifecycle semantics. The traits constitute the extensibility contract: future trainers (NEAT, SGD, hybrid) implement `Trainer` and can reuse all existing components. Known extension needs captured in LifeOS: `Task::get_dataset` returning eager `Vec<(Vec<f32>, Vec<f32>)>` blocks MNIST-scale data; no `Serializable` trait (no checkpointing); no `Send + Sync` bounds (no parallelism).

## Technologies and concepts demonstrated

### Languages
- **Rust** — entire codebase. 66 source files, ~115.4 KB. Safe Rust only; zero `unsafe` blocks, zero `Arc`/`Rc`/`RefCell` (LifeOS Architecture, verified via search). Heavy use of generics with trait bounds; `EvolutionaryTrainer<G, M, C, F, S>` is the showcase signature.

### Frameworks and libraries
- **`rand` 0.9.1** and **`rand_distr`** — random sampling for mutation, crossover, selection. Currently used via thread-local `rng()` (no seeded RNG yet).
- **`num-traits` 0.2.19** — `Zero`, `One`, `Float` bounds for generic tensor ops.
- **`thiserror` 2.0.12** — `VynapseError` derives.
- **`clap` 4.5.41** — declared in `vynapse-cli/Cargo.toml`; not currently used (the CLI binary is `Hello World!`).
- **`serde` 1.0.219** — declared in `vynapse-core/Cargo.toml`; not currently used (anticipatory pre-import for Milestone 2 JSON config).

### Runtimes / engines / platforms
- **Cargo workspace** — four crates with explicit dependency direction enforced by `Cargo.toml` `[dependencies]` declarations rather than by convention.

### Tools
- **Plans-driven development** — `plans/NNN_<topic>.md` files capture requirements, scope, action items and acceptance criteria before non-trivial work. `plans/evolutionary-trainer-modular-refactor.md` is the worked example. Convention introduced 19 December 2025.
- **Inline tests** — every `#[test]` lives in `#[cfg(test)] mod tests { ... }` at the bottom of the source file it tests. 120 `#[test]` functions across 12 files; no `tests/` integration directory.

### Domains and concepts
- **Evolutionary algorithms (fixed topology)** — Gaussian mutation with weight clamping, uniform crossover with rate gating, tournament selection (`tournament_size=3` is the typical use), generational replacement. No elitism. No (μ+λ) despite the README's checked box.
- **Trait-based generic ML architecture** — nine traits define every extension point; trainers are generic over five type parameters monomorphised at compile time (chosen over `dyn` trait objects for zero-cost abstraction).
- **Tensor primitives from scratch** — Shape with dimension validation, row-major Tensor with computed strides, elementwise arithmetic with shape checking, matrix-vector multiplication, transpose, reshape. No matmul.
- **Multi-crate Rust workspace organisation** — `common` → `math` → `core` → `cli` dependency direction; nothing depends on `core` or `cli`.
- **Result-based error handling** — single `VynapseError` enum, `Result<T>` alias, no panics in library code.
- **Declarative scaffolding** — 16 empty `.rs` files registered in `mod.rs` to encode roadmap intent in the module tree.

## Key technical decisions

(All drawn from LifeOS Decisions.md.)

- **Modular refactor (Dec 2025).** Split the monolithic `EvolutionaryTrainer` into five composed components (`EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`). Rejected alternatives: keeping `population_size` on `EvolutionConfig` ("population_size belongs to Population"), keeping mutation/crossover rates on `EvolutionConfig` ("rates bind with their operators"), returning `&TrainingStats` from `get_stats` instead of owned (accepted clone cost for caller simplicity), internal population initialisation inside the trainer (rejected for flexibility — template-based, file-based, custom init strategies must remain external).
- **Trait-based modularity over `dyn` trait objects.** Generics with monomorphisation chosen over `Box<dyn Mutation>` to avoid heap allocation, dynamic dispatch cost, and loss of inlining. Rejected enums-over-all-components for being closed to external extension. Cost accepted: generic parameters propagate through every trainer-using function.
- **No elitism in MVP.** `step()` does full generational replacement. README claims (μ+λ) and "simple elitism preservation" — neither is implemented. Documented as MVP simplicity; ~10 lines to add. Will need fixing once tasks get harder than XOR.
- **Hardcoded hidden layer.** `TaskBasedFitness::evaluate` hardcodes `hidden_size = 4` in `task_based.rs:212` despite the genome carrying a `shape` field. Known and accepted at introduction (commit `9ca58e4`, 19 Jul 2025); will change at Milestone 2.
- **All-`f32` numerics.** Matches PyTorch/TensorFlow default, native to GPUs, halves memory vs `f64`. Rejected `f64` for accuracy (doubles memory, halves SIMD throughput) and generic-over-`T: Float` (propagates another type parameter through every component).
- **Inline tests, not `tests/` folder.** Colocated tests share scope with the code they test — access to private fields, helper functions, private constructors. 120 tests across 12 files. `repo_stats.py` reports 0 because it only counts `tests/` and `tests.rs`.
- **Tensor crate from scratch (not `ndarray`/`nalgebra`/`candle`).** Vynapse is positioned as a from-scratch learning project — the point is to implement the stack. Re-using `candle` would skip most of Milestones 1, 6, 7, 9. Would change if Vynapse pivoted from learning project to product.
- **Plans-driven development.** Non-trivial refactors get a `plans/<topic>.md` document first, capturing scope, action items, risks and design decisions. Introduced 19 December 2025. Mirrors Caner's broader LifeOS working style.
- **`serde` declared but unused.** Anticipatory pre-import for Milestone 2 (JSON configuration loading). Self-resolves when the first config struct lands.

## What is currently built

- One end-to-end trainer paradigm: fixed-topology neuroevolution with tournament selection, Gaussian mutation, uniform crossover, full generational replacement. Verified by the integration test `test_evolutionary_trainer_learns_powers_of_two` at `evolutionary.rs:1711-1843`.
- Full `vynapse-math` crate (Shape, Tensor, arithmetic, matrix-vector multiplication, transpose, reshape) with 54 inline tests.
- Trait layer: nine traits defining every extension point.
- Concrete components: `Sigmoid` activation, `UniformCrossover`, `TaskBasedFitness` (with hardcoded hidden layer), `FixedTopologyGenome` (no biases), `MSE` loss, `GaussianMutation` (with undocumented `[-5.0, 5.0]` clamp), `TournamentSelection`.
- `training_setup/` infrastructure: `EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats` with `ConvergenceStatus` enum.
- Two tasks: `PowersOfTwo` (effectively saturated against sigmoid output range) and `XOR`.
- `VynapseError` and `Result<T>` plumbed through every fallible function.
- 120 `#[test]` functions across 12 files; `cargo test --release` runs a real training run via the integration test.

Not built (per LifeOS Gaps):

- No CLI — `vynapse-cli/src/main.rs` is `fn main() { println!("Hello World!"); }`.
- No configuration loading — all four files in `config/` are 0 bytes; `serde` declared but unused.
- No CSV logging — Milestone 1 deliverable, unchecked.
- No deterministic seeding — every `rng()` call is thread-local.
- No elitism — README claims it; code does full generational replacement.
- No biases in `FixedTopologyGenome`.
- No `matrix_matrix_mult` / GEMM — only `matrix_vector_mult` exists.
- No autodiff, no SGD, no NEAT, no hybrid learning, no GPU/SIMD, no parallel evaluation.
- 16 declared-but-empty stub files (full inventory in LifeOS Gaps).

## Current state

Paused. Status set to "paused" per LifeOS Overview commit pattern: 36 total commits, with 32 commits in the first 17 days (July 2025), then a five-month silence, then a focused 3-commit burst on 19-21 December 2025 that produced the modular refactor. HEAD is `1c01e02` (21 December 2025). LifeOS Overview describes the project's positioning as "active, Milestone 1 ~90% complete" but the commit log shows the December 2025 refactor was the last burst — Vynapse is "bursty, not continuous" per LifeOS Suggestions, and the 4-month-stale HEAD is flagged as worth checking before investing further vault cycles.

## Gaps and known limitations

- **README pitch over-promises by ~9 milestones.** The opening line describes a "hybrid runtime unifying PyTorch, TensorFlow, DEAP, and NEAT" — that is Milestone 10. A reader at face value will overestimate the current engine by ~9 milestones.
- **README's "(μ+λ) population replacement" claim is incorrect.** The code does full generational replacement; (μ+λ) requires retaining parents and selecting best-N from parents ∪ offspring.
- **No elitism** — fitness can regress across generations; the integration test only asserts `final_best >= initial_best`, a weak signal.
- **Parent pairing is positional, not random** — child `i`'s parents are the `i`-th and `(i+1)`-th elements of the selection output, introducing a locality bias.
- **Hidden size hardcoded at 4** — genome `shape` field is not consulted during fitness evaluation; a genome with shape `[2, 8, 1]` would fail validation.
- **No biases** in `FixedTopologyGenome` — XOR is borderline-solvable, real regression tasks would not work without biases.
- **Activation applied to output layer** — output is always in sigmoid's `(0, 1)` range, fatal for regression tasks like PowersOfTwo where outputs reach 256+.
- **Saturated benchmarks** — PowersOfTwo cannot be learned past tiny `max_input` because of the sigmoid output range; XOR is tight but solvable.
- **`TargetReached` convergence status is dead** — no code path ever sets it.
- **Stagnation strict-inequality bug** — plateau ties count as non-improvement, prematurely terminating runs.
- **`is_converged()` returns true for fresh `WaitingToStart` state** — a caller checking `if !trainer.is_converged() { run() }` sees "done" on a fresh trainer.
- **`FitnessStats::validate()` fails on fresh state** — latent bug; `reset()` on a fresh-and-never-run trainer errors.
- **Weight clamp at `[-5.0, 5.0]` is undocumented** — visible only in `gaussian.rs:855`, not in README or any code comment.
- **No `Send + Sync` bounds** — blocks parallel population evaluation (Milestone 9 prerequisite).
- **No `Serializable` trait** — blocks checkpointing.
- **`Task::get_dataset` returns eager `Vec<(Vec<f32>, Vec<f32>)>`** — blocks MNIST-scale data (Milestone 5 prerequisite).
- **Test gaps:** `GaussianMutation` and `UniformCrossover` have zero `#[test]` functions; `EvolutionConfig::validate`, `Shape::new` edge cases, `Tensor::get/set` bounds-error paths are all untested directly.

## Direction (in-flight, not wishlist)

Per LifeOS Roadmap, the concrete near-term work to close Milestone 1:

- **Deterministic seeding** — add a `SeedableRng` to `EvolutionConfig` (or thread `&mut R: Rng` through `Mutation::mutate`, `Crossover::crossover`, `Selection::select`). Unblocks reproducibility and deterministic test assertions.
- **CLI** — `clap` is already a declared dependency; build a minimal `vynapse train --task <powers_of_two|xor> --generations N --population-size M --seed S`. Closes the Milestone 1 README-stated deliverable.
- **CSV logging** — per-generation `(gen, best, avg, worst, elapsed_ms)` rows to a configurable path.
- **Elitism (optional Milestone 1 polish)** — ~10 lines in `step()` to preserve top-k from generation k into k+1.

After Milestone 1 closes, the LifeOS roadmap's natural sequence is Milestone 2 (configuration) → Milestone 3 (DEAP-style EA) → Milestone 4 (NEAT). Milestones 5-8 (SGD, autodiff, static graph, hybrid) require gradient infrastructure that does not yet exist; Milestone 6 (autodiff) is flagged in LifeOS as the largest single jump.

No active work is in flight today — HEAD is from December 2025 and the project is paused.

## Demonstrated skills

- **Rust generics with trait bounds for zero-cost ML abstraction.** `EvolutionaryTrainer<G, M, C, F, S>` parametrises over genome, mutation, crossover, fitness and selection types simultaneously; monomorphisation chosen explicitly over `dyn` trait objects.
- **From-scratch tensor primitives in safe Rust.** `Shape`, `Tensor<T>`, elementwise arithmetic, matrix-vector multiplication, transpose, reshape — all with shape-validation error paths, no `unsafe`, no external numerics dependency.
- **Multi-crate Cargo workspace design with strict dependency direction.** Four crates (`common`, `math`, `core`, `cli`) with `Cargo.toml`-enforced dependency edges; nothing depends on `core` or `cli`.
- **Trait-first extensibility design.** Nine intentionally thin traits constitute the full extension surface; future trainers (NEAT, SGD, hybrid) implement `Trainer` and reuse every existing component.
- **Result-based error discipline.** Single `VynapseError` enum, `Result<T>` alias, no panics in library code, error propagation across crate boundaries.
- **Evolutionary-algorithm implementation from primitives.** Tournament selection, Gaussian mutation with weight clamping, uniform crossover with rate gating, generational replacement, fitness-history tracking, stagnation-based termination.
- **Declarative scaffolding as architectural communication.** 16 empty `.rs` files registered in their `mod.rs` files encode roadmap intent in the module tree — a reader sees "I know where NEAT goes, I know where SGD goes" without reading the README.
- **Plans-driven development workflow.** `plans/<topic>.md` documents capture scope, acceptance criteria and design decisions before non-trivial refactors; demonstrated by the December 2025 modular-refactor plan being executed against checked-off action items.
- **Inline-test discipline.** 120 `#[test]` functions colocated with source modules, sharing private scope with the code they test; 54 of them in `vynapse-math` alone.
- **Modular refactor against a written plan.** December 2025 refactor split a monolithic trainer into five composed components (`EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`) with explicit rejected-alternatives documentation.

Where the project does **not** yet demonstrate strong evidence (per LifeOS Suggestions): performance discipline (`.clone()` everywhere, thread-local RNG, no `Send + Sync`), numerical discipline (hidden size hardcoded, activation applied to regression output, undocumented weight clamp), and testing rigour on learning behaviour (the one real learning test asserts only `final_best >= initial_best`).

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Vynapse/_Overview.md | 134 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
| Projects/Vynapse/Architecture.md | 242 | "- Error propagation across crate boundaries: [[Vynapse/Systems/Error Model]]" |
| Projects/Vynapse/Decisions.md | 184 | "- The gaps these decisions leave open: [[Vynapse/Gaps]]" |
| Projects/Vynapse/Gaps.md | 268 | "- Analytical suggestions for prioritisation: [[Vynapse/Suggestions]]" |
| Projects/Vynapse/Roadmap.md | 187 | "- RL problem domain overlap with NeuroDrive (Milestone 4 cartpole): [[NeuroDrive/_Overview]]" |
| Projects/Vynapse/Suggestions.md | 197 | "- Profile update implied by the project's portfolio value: [[Profile/Projects]], [[Profile/Skills]]" |
| Projects/Vynapse/Systems/_Overview.md | 42 | "- [[Projects/Vynapse/Roadmap]] — direction-of-travel" |
| Projects/Vynapse/Systems/Error Model.md | 114 | "- A specific latent error-handling bug: [[Vynapse/Gaps#validate() fails on fresh state]]" |
| Projects/Vynapse/Systems/Evolutionary Trainer.md | 244 | "- Why the refactor happened and what it replaced: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Genome and Components.md | 218 | "- Stubbed components as roadmap evidence: [[Vynapse/Roadmap]]" |
| Projects/Vynapse/Systems/Tasks and Fitness.md | 171 | "- Why the current benchmarks under-test learning capability: [[Vynapse/Gaps#Benchmarks are saturated]]" |
| Projects/Vynapse/Systems/Tensor and Math.md | 126 | "- Aurix also has a tensor crate — duplication decision pending: [[Aurix/_Overview]], [[Vynapse/Decisions#Tensor crate vs external]]" |
| Projects/Vynapse/Systems/Training Stats and Convergence.md | 185 | "- Why these were split out from the trainer in Dec 2025: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Traits Layer.md | 185 | "- Stubs that will need new trait work: [[Vynapse/Gaps#Traits that need extension]]" |
