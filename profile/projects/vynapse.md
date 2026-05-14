---
name: Vynapse
status: paused
source_repo: https://github.com/Capataina/Vynapse
lifeos_folder: Projects/Vynapse
last_synced: 2026-05-13
sources_read: 14
---

# Vynapse

## One-line summary

From-scratch Rust evolutionary-algorithm trainer for fixed-topology neural networks, built as a 4-crate workspace with a trait-based modular architecture and a 10-milestone roadmap whose later paradigms (NEAT, SGD, autodiff, static graph, hybrid) are scaffolded as empty stub files.

## What it is

Vynapse is Caner's from-scratch Rust ML engine. The current build is a **single-paradigm evolutionary trainer** that runs end-to-end on fixed-topology neuroevolution: tournament selection, Gaussian mutation, uniform crossover, generational replacement, with a `[1, 4, 1]` MLP on toy tasks (PowersOfTwo, XOR). The README's opening pitch — *"Rust-native deep learning and neuroevolution engine built as a hybrid learning runtime — unifying the optimization paradigms of PyTorch, TensorFlow, DEAP, and NEAT"* — describes Milestone 10 rather than the current state; the LifeOS Overview explicitly flags this discrepancy and notes that quoting the README pitch externally over-promises the engine by ~9 milestones. The honest framing, from LifeOS: *"a Rust-from-scratch evolutionary-algorithm trainer with a roadmap toward hybrid deep learning."*

The codebase is designed as a learning exercise and a portfolio piece in equal measure. The four-crate workspace, the nine-trait extensibility surface, and the December 2025 modular refactor are the architectural evidence — not the breadth of trained models. Project started 12 July 2025; HEAD is `1c01e02` (21 December 2025); 36 commits across 23 weeks, including a 143-day silence between late July and mid-December that ended with a single focused restructuring burst.

## Architecture

Four-crate Cargo workspace with strict dependency direction:

```
                ┌───────────────────┐
                │  vynapse-common   │   error types, Result<T> alias
                │   (4 files, 12KB) │   depends on: thiserror only
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
              │  vynapse-cli    │   clap 4.5.41 declared
              │  (3 files)      │   main.rs = "Hello World!"
              └─────────────────┘
```

- **`vynapse-common`** — error type (`VynapseError` with `TensorError`, `EvolutionError`, `ConfigError` variants) and `Result<T>` alias. Depends only on `thiserror = "2.0.12"`.
- **`vynapse-math`** — pure tensor primitives, no ML concepts. Depends on `vynapse-common` + `num-traits = "0.2.19"`. 54 `#[test]` functions across arithmetic, linalg, and transform.
- **`vynapse-core`** — all ML logic. Depends on `vynapse-math`, `vynapse-common`, `rand = "0.9.1"`, `rand_distr`, `serde = "1.0.219"`, `thiserror`. (Note: `serde` is declared but currently unused; pre-imported for the Milestone 2 configuration layer.)
- **`vynapse-cli`** — depends on `vynapse-core`, `vynapse-math`, `clap = "4.5.41"`. Currently a 44-byte stub — `fn main() { println!("Hello World!"); }`. The Milestone 1 deliverable CLI (`vynapse train --task powers_of_two --generations 100`) does not exist.

`vynapse-core` decomposes into seven sub-modules, each with a single responsibility:

- **`traits/`** — nine pure trait interfaces (`Genome`, `Mutation`, `Crossover`, `Selection`, `Fitness`, `Loss`, `Activation`, `Task`, `Trainer`). Eight require `Clone + Debug` as a uniform supertrait bound; `Trainer` is the exception because it holds unique mutable run state.
- **`components/`** — concrete trait implementations. Six implemented (`Sigmoid`, `MeanSquaredError`, `GaussianMutation`, `UniformCrossover`, `TournamentSelection`, `TaskBasedFitness`, `FixedTopologyGenome`); ten more declared as 0-byte stub files (`neat_genome`, `arithmetic` crossover, `uniform` mutation, `fitness_proportionate`, `roulette` selection, etc.).
- **`trainers/`** — orchestration, one paradigm per file. `evolutionary.rs` is implemented (12.4 KB); `deap.rs`, `hybrid.rs`, `neat.rs`, `sgd.rs`, `static_graph.rs` are all 0 bytes.
- **`training_setup/`** — trainer-agnostic infrastructure (the December 2025 refactor's product): `EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`.
- **`tasks/`** — `PowersOfTwo` and `XOR` implemented; `cartpole.rs` is 0 bytes (Milestone 4 stub).
- **`config/`** — all four files (`experiment_config`, `mutation_config`, `selection_config`, `trainer_config`) are 0 bytes.
- **`utils/`** — both files (`metrics.rs`, `rng.rs`) are 0 bytes.

**The trait layer as extensibility contract.** `EvolutionaryTrainer<G, M, C, F, S>` parametrises over five generic types with trait bounds — Rust generics + monomorphisation give zero-cost abstraction. `dyn` trait objects were rejected because they incur heap allocation and dynamic dispatch; enum-over-variants was rejected because it closes the extension surface to non-`vynapse-core` users. The cost is longer type signatures propagating through every trainer-using function.

**Stub discipline as architectural map.** The 16 empty-but-`mod.rs`-declared files are deliberate scaffolding, not accidents. Every stub represents a named future component with a decided name and location — the module tree IS the 10-milestone roadmap rendered as Rust files. LifeOS Gaps explicitly notes: *"Do NOT delete the 16 empty stub files — they encode roadmap intent."*

**Generation loop (one `step()`)** (verified against `evolutionary.rs:1555-1596`):

1. Evaluate every genome via `fitness_function.evaluate(genome)` → `Vec<f32>`.
2. Update `fitness_stats` (best/avg/worst) and `training_stats` (generation counter, stagnation, convergence status).
3. Select N parents via `selection_function.select(fitness, N)` → `Vec<usize>` of length N.
4. Generate N offspring: parent `i` paired with parent `(i+1) mod N` from the selected index list; crossover applied with probability `crossover_rate` (else clone parent1); then mutate.
5. Replace the entire population (`set_all_genomes(new)`) — **no elitism**; the previous generation is discarded wholesale.

**Generational replacement, not (μ+λ).** The README's checked "Add (μ+λ) population replacement strategy" claim is incorrect at the code level — true (μ+λ) keeps the best N from `parents ∪ offspring`; this trainer discards parents and keeps only offspring. Selection pressure (tournament size) is the only mechanism preserving good genes across generations.

**Parent pairing is positional, not random.** Child `i`'s parents are the `i`-th and `(i+1)`-th elements of the selection output. Because tournament selection emits a random-looking sequence, the bias is statistical rather than systematic, but it is not the textbook "sample two parents independently."

**Memory and ownership.** Zero `unsafe`, zero `Arc`, zero `Rc`, zero `RefCell`, zero interior mutability across the entire codebase (verified by content search). Owned data moves through `&mut self`. `.clone()` is abundant — `get_weights()` returns `Vec<f32>` not `&[f32]` on every call, `step()` clones `training_stats` to return. At population=500 with ~12 weights/genome this is negligible; at MNIST-scale networks (Milestone 5+) the API becomes a measurable cost.

## Subsystems and components

### Tensor and math (`vynapse-math`, 48.7 KB)

Pure row-major `Tensor<T>` with `data: Vec<T>`, `shape: Shape`, `strides: Vec<usize>`. Strides computed once at construction as `shape.dims[i+1..].iter().product()`. Generic over element type with bounds scaled to each op's requirements (`T: Clone + Default + Add<T> + Zero` for `tensor_add`, plus `PartialEq` for `tensor_div`). Constructors: `new`, `zeros`, `ones`, `from_vec` (validates `data.len() == total_elements`). Access via `get`/`get_mut`/`set` with rank and bounds checks returning `TensorError`.

Element-wise arithmetic: `tensor_add`, `tensor_sub`, `tensor_mul`, `tensor_div`. No broadcasting — shapes must match exactly. `tensor_div` pre-scans for zeros in the divisor; finding any returns `TensorError`.

Linear algebra: **only `matrix_vector_mult`**. No matrix-matrix multiplication, no GEMM. Sufficient for the current MLP forward pass (one input at a time in a loop); blocks batched forward passes (Milestone 5 requirement).

Transforms: `transpose_2d` (rank-2 only, element-by-element copy), `reshape_tensor` (validates total-element match, full data clone — not a view).

All ML code is `f32`. No `f64` path. 54 `#[test]` functions; `Shape` and `Tensor` themselves are untested directly (covered indirectly through ops tests).

### Genome and components (`vynapse-core/components/`)

| Component | Trait | Tests | Key parameter |
|---|---|---|---|
| `FixedTopologyGenome` | `Genome` | 7 | `shape: Vec<usize>` |
| `GaussianMutation` | `Mutation` | 0 (indirect) | `sigma: f32` |
| `UniformCrossover` | `Crossover` | 0 (indirect) | `inheritance_probability: f32 ∈ [0, 1)` |
| `TournamentSelection` | `Selection` | 24 | `tournament_size: usize ≥ 1` |
| `Sigmoid` | `Activation` | 4 | none |
| `MeanSquaredError` | `Loss` | 4 | none |
| `TaskBasedFitness<T, L, A>` | `Fitness` | 13 | task + loss + activation |

`FixedTopologyGenome` stores weights as a flat `Vec<f32>` with a `shape: Vec<usize>` describing intended layer sizes. `new_random(shape)` sizes the weight vector as `Σ shape[i] * shape[i+1]` (inter-layer weights only — **no biases**) and samples each weight uniformly from `[-1.0, 1.0]`. `set_weights` is length-validated; a mutated vector with the wrong length fails at assignment time.

`GaussianMutation` is per-weight, not per-genome: `rate = 0.8` means 80% of weights get noise, not 80% of genomes get one mutation. Weights clamped to `[-5.0, 5.0]` post-mutation — undocumented in README, only visible in code.

`UniformCrossover` does bit-wise gene selection: for each weight position, picks parent 1 with probability `inheritance_probability`, else parent 2. The constructor's range is `(0.0..1.0)` half-open — `p = 1.0` is rejected.

`TournamentSelection` is the most heavily tested component (24 tests, 11.8 KB). Standard tournament: sample `tournament_size` indices with replacement, return argmax. Selection pressure increases monotonically with tournament size — at `size = pop_size`, the best individual is selected >60% of the time; at `size = 1`, it's pure random sampling.

### Tasks and fitness

`Task` trait: `get_dataset() -> Vec<(Vec<f32>, Vec<f32>)>` (eager materialisation — a Milestone 5 blocker, would need to become an iterator for MNIST-scale data), `get_input_size`, `get_output_size`, `get_name`.

Implemented tasks: `PowersOfTwo(max_input ∈ 1..=12)` and `Xor` (4 fixed rows). `cartpole.rs` is 0 bytes — RL doesn't fit the `(input, expected_output)` shape and will force a `Task` trait extension when Milestone 4 lands.

`TaskBasedFitness<T, L, A>::evaluate(genome)` reconstructs a 2-layer MLP on-the-fly from the genome's flat weights:

```
hidden_size = 4   ← HARDCODED at task_based.rs:212, ignoring genome.shape
total_weights = inputs*4 + 4*outputs
[partition flat weights into two matrices via Tensor::from_vec + reshape]
for (input, expected) in task.get_dataset():
    h = activation(matrix_vector_mult(W1, input))
    o = activation(matrix_vector_mult(W2, h))     ← sigmoid on output too
    total_error += loss(o, expected)
fitness = 1.0 / (1.0 + total_error)
```

The hardcoded `hidden_size = 4` is the largest architectural rigidity in the current build — `genome.shape` is consulted only by `new_random()` to size the weight vector; `evaluate` ignores it and assumes `inputs*4 + 4*outputs`. A genome with `shape = [2, 8, 1]` (24 weights) is rejected by the XOR fitness (expects `2*4 + 4*1 = 12`).

Activation applied to **both** hidden and output layers means sigmoid output is always in `(0, 1)` — fatal for regression on `PowersOfTwo`, where targets up to `2^11 = 2048` cannot be represented. Best achievable fitness on `PowersOfTwo(5)` is ~0.0035; the integration test measures whether the population reliably saturates sigmoid against an impossible ceiling, not whether learning works.

### Training stats and convergence

`FitnessStats` tracks per-generation best/avg/worst plus a history of best values per generation. Initial state uses `f32::NEG_INFINITY` as sentinel (zero would be ambiguous with a legitimate fitness of 0).

`TrainingStats` owns the run state machine: `current_generation`, `max_generations`, `convergence_status: ConvergenceStatus`, `starting_time: Option<Instant>`, `elapsed_time: Duration`, `stagnation_counter`, `stagnation_limit`. Embeds a `FitnessStats`.

`ConvergenceStatus` enum has 5 variants — `WaitingToStart`, `Running`, `TargetReached`, `Stagnated`, `MaxIterationsReached`. **`TargetReached` is a dead variant** — declared but never assigned anywhere in code; no `target_fitness` field on `EvolutionConfig`. Anticipatory for Milestone 2.

`is_last_fitness_best` uses **strict inequality** (`last > max_of_rest`), so a plateau (tie with previous best) increments stagnation. With `stagnation_limit = 20`, a run that hits a plateau at generation 5 stagnates at generation 25 — usually earlier than `max_generations = 100`, making stagnation the more likely termination than the generation cap.

Multiple state-machine subtle bugs documented in LifeOS Gaps:
- `is_converged()` returns `true` for fresh `WaitingToStart` state (defined as `status != Running`).
- `step()` after convergence does not error — it bumps the generation counter past the limit.
- `FitnessStats::validate()` fails on fresh state (empty history triggers `EvolutionError`); the trainer's `reset()` calls `fitness_stats.reset()` which calls `validate()` first, so `reset()` errors on a fresh-never-run trainer.

### Error model

Single crate-wide `VynapseError` enum with three string-wrapping variants: `TensorError`, `EvolutionError`, `ConfigError`. `Result<T>` alias in `vynapse-common`. No structured error data, no source-error chaining (`thiserror` used only for `Display`/`Error` derivation, not for `#[source]` traversal). The split between `EvolutionError` and `ConfigError` is inconsistent in current code (`TournamentSelection::new(0)` returns `EvolutionError`; `Population::new(0)` returns `ConfigError` — both are construction-time validation).

Every `get_name()` method returns `Result<&str>` even though implementations never fail — the uniform signature is a hedge against future tasks that might validate runtime parameters (config-loaded names, dataset filenames).

### Traits layer

Nine traits in `vynapse-core/src/traits/`. Eight require `Clone + Debug` as a uniform supertrait bound; `Trainer` is the exception (holds unique mutable run state, no clone semantics).

`Trainer` is the only trait with substantial behaviour — six methods (`train`, `step`, `get_stats`, `reset`, `is_converged`, `validate_config`). Every other trait defines 1–4 thin methods.

Missing trait surface (Milestone-blockers for future work):
- No `Serializable` trait → no checkpointing (cannot dump best genome to disk).
- No `Send + Sync` bounds → no parallelism (Milestone 9 blocker).
- No trait hierarchy → cannot express multi-strategy selection (README's Milestone 3 weighted-selection example).
- No async → cannot batch RL rollouts in Milestone 9.

## Technologies and concepts demonstrated

### Languages

- **Rust** — only language in the project. Entire ML stack (tensor primitives, generic trainer, evolutionary components, error model) in safe Rust. 115.4 KB across 66 source files. Zero `unsafe`, zero `Arc`/`Rc`/`RefCell`/interior mutability (verified by content search). 120 `#[test]` functions across 12 files, all colocated with the code they test rather than in a separate `tests/` folder.

### Frameworks and libraries

- **`thiserror = "2.0.12"`** — error type derivation in `vynapse-common`. Used for `Display`/`Error` impls only; no `#[source]` chaining.
- **`rand = "0.9.1"` + `rand_distr`** — RNG and Gaussian-distribution sampling for mutation, tournament selection, weight initialisation. Currently thread-local-per-component (no shared seed) — deterministic seeding is an unchecked Milestone 1 task.
- **`num-traits = "0.2.19"`** — `Zero`, `One`, and number-type traits for the generic `Tensor<T>` constructors and ops.
- **`clap = "4.5.41"`** — declared as a `vynapse-cli` dependency for the planned CLI; unused (main.rs is a 44-byte "Hello World!").
- **`serde = "1.0.219"`** — declared in `vynapse-core/Cargo.toml`; no `use serde` or `#[derive(Serialize, Deserialize)]` anywhere in the codebase. Pre-imported for Milestone 2's JSON configuration layer.

### Runtimes / engines / platforms

- **Cargo workspace, four crates** — `vynapse-common`, `vynapse-math`, `vynapse-core`, `vynapse-cli`. Strict dependency direction enforced (common is the base; nothing depends on cli; math depends on common; core depends on math + common; cli depends on core + math).

### Tools

- **`plans/` convention** — introduced 19 December 2025 with commit `2419eee`. Any non-trivial refactor is preceded by a plan document (`NNN_<topic>.md`) capturing requirements, scope, files touched, action items, risks, design decisions, with inline checkboxes that become the single source of truth for that work unit. Mirrors Caner's broader LifeOS working style. The December 2025 modular refactor was the first execution of this pattern: `plans/evolutionary-trainer-modular-refactor.md` (5.1 KB) was written, then `456d9d1` and `1c01e02` landed against it.

### Domains and concepts

- **Evolutionary algorithms** — fixed-topology neuroevolution. Implements: tournament selection (with-replacement sampling, configurable size), Gaussian-noise mutation (per-weight probability, clamped to `[-5, 5]`), uniform crossover (per-position parent inheritance with configurable probability), generational replacement. Documented gaps: no elitism, no (μ+λ) despite README's checked claim, positional parent pairing (`i` with `(i+1) mod N`).
- **Trait-based modular design in Rust** — `EvolutionaryTrainer<G, M, C, F, S>` parametrises over Genome, Mutation, Crossover, Fitness, Selection with trait bounds. Zero-cost abstraction via monomorphisation. Nine separate traits compose into one trainer; no `dyn` trait objects.
- **Cargo workspace organisation** — math/core/common/cli split with explicit dependency direction. The `vynapse-common` crate exists specifically to break the dependency cycle between math and core that would otherwise form around the shared error type.
- **MLP forward pass** — 2-layer MLP reconstructed on-the-fly from a flat weight vector per genome evaluation. Weight partitioning, reshape to layer matrices, matrix-vector multiplication, element-wise activation. Currently no batched forward pass (no `matrix_matrix_mult` in the math crate).
- **Sigmoid activation, MSE loss, fitness `1/(1+total_error)`** — bounded `(0, 1]` fitness formula. `total_error` is sum over dataset, not mean — couples difficulty of distinguishing genomes to dataset size.
- **State machine for training run** — `ConvergenceStatus` enum (5 variants), `stagnation_counter` vs `stagnation_limit`, `Instant`-based elapsed-time tracking. Multiple documented edge cases (dead variant, fresh-state `is_converged() == true`, plateau triggers stagnation due to strict inequality).
- **Result-based error handling** — every fallible function returns `Result<T>`; no panics in library code; `unwrap()` calls limited to tests and one safe post-`is_none()`-check unwrap.

## Key technical decisions

### Plans-driven development (introduced 19 Dec 2025)

A `plans/` folder convention. Any non-trivial refactor is preceded by a plan document with requirements, scope, files-touched list, action items, risks, design decisions, and inline checkboxes. The plan file becomes the single source of truth for that work unit. **Alternatives rejected:** working from commit messages (used for the first 22 commits; impossible to retroactively see intended scope), GitHub Issues (workflow overhead disproportionate to one-contributor project), inline TODO comments (zero TODOs in the codebase). **Why:** mirrors LifeOS working style and makes a future-session pickup tractable.

### Modular refactor (Dec 2025)

The pre-Dec 2025 `EvolutionaryTrainer` was monolithic — held inline `TrainingStats` (duplicating the one in `training_setup`), mutation/crossover/population-size all on the trainer struct, generation loop mixing trainer logic with population management and stats. The refactor split it into five composed components: `EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`. The trainer became a thin orchestrator. **Alternatives explicitly rejected in the plan:** keeping `population_size` on `EvolutionConfig` (rejected because "population_size belongs to Population — Population *is* the collection"); keeping `mutation_rate`/`crossover_rate` on `EvolutionConfig` (rejected because rates bind with their operators); having `Trainer::get_stats` return `&TrainingStats` (rejected with explicit "acceptable for MVP" — cost is one clone per call, benefit is no lifetime gymnastics); internal population init inside the trainer (rejected for maximum flexibility — template-based, file-based, custom strategies all want to live outside). **Why:** the pre-refactor trainer could not be reused for SGD or NEAT; the post-refactor `FitnessStats` and `TrainingStats` are trainer-agnostic, ready for future paradigms.

### Trait-based modularity over `dyn` or enums (Jul 2025)

Every concrete component implements a thin trait; the trainer is generic over five type parameters with trait bounds. **Alternatives rejected:** `Box<dyn Mutation>` etc (heap allocation per operator, dynamic dispatch cost, loss of inlining); `enum` over all components (closed to external extension — requires editing the enum to add a new component); concrete-type composition (would lock the trainer to one specific combination). **Why:** Rust generics + trait bounds give extensibility without runtime cost via monomorphisation. The cost is longer type signatures in tests. **Would change if:** a user-facing API needs heterogeneous trainer collections — then `dyn Trainer` becomes necessary; monomorphisation would otherwise bloat the binary if every config combo compiled to its own trainer type.

### Tensor crate from scratch, not `ndarray`/`nalgebra`/`candle` (Jul 2025)

Build `vynapse-math` from scratch rather than depending on an existing crate. **Why:** this is a from-scratch learning project. Re-using `candle` would skip most of Milestones 1, 6, 7, 9 — defeating the purpose. The README's framing ("Rust Performance: Built from the ground up in safe, parallel Rust — no Python bindings, no unsafe blocks") states this intent explicitly. **Would change if:** Vynapse becomes a product rather than a learning project — then replacing `vynapse-math` internals with `ndarray` would give most of the Milestone 9 performance gains. Trait-based architecture makes this swap mostly transparent to callers.

### All-`f32` numerics

Entire ML path uses `f32`. No `f64` option, no mixed precision. **Why:** `f32` is the ML standard, GPUs are native-`f32`, PyTorch/TensorFlow default to `f32`, memory costs half as much. Numerical precision is sufficient for practical neural networks.

### Inline tests, not `tests/` folder

120 tests across 12 files, all in `#[cfg(test)] mod tests { ... }` blocks at the bottom of the source file they test. No `tests/` integration-test directory. **Why:** inline tests share scope with the code they test — access to private fields, helper functions, private constructors. For a fast-moving MVP, the friction of public-only API testing outweighs its benefits. **Would change if:** public-facing integration testing becomes important (Milestone 2+, when the CLI is the user surface).

### Hardcoded hidden-layer size (Jul 2025 — known MVP simplification)

`TaskBasedFitness::evaluate` hardcodes `hidden_size = 4` regardless of what the genome's `shape` field reports. Introduced in commit `9ca58e4` with a commit message acknowledging the hardcoding. **Alternatives:** read from `genome.shape()` (would require adding a `shape()` method to `Genome` trait — feasible, minor API change); `const HIDDEN: usize` generic parameter; constructor argument. **Why this path (MVP):** at commit time, only XOR existed and 4 hidden units was sufficient. **Would change if:** any task requires a different hidden size — Milestone 2 (configurability) will force this.

### No elitism in MVP

The `step()` loop does not preserve the best-N genomes across generations; full generational replacement via `set_all_genomes(new_population)`. **Stated in plan:** "replacement/elitism hooks" mentioned but not implemented. **Stated in README:** "Add (μ + λ) population replacement strategy" checked complete — but the code does generational replacement, not (μ+λ). The README's checkbox is factually incorrect. **Why this path:** MVP simplicity — elitism requires retaining previous-generation genomes after selection; the current `set_all_genomes(Vec<G>)` API discards them. Adding elitism is ~10 lines. **Would change if:** convergence is too erratic on meaningful tasks; the integration test's "fitness doesn't decrease" assertion is too weak to detect this.

## What is currently built

The honest implemented scope, distinct from the design ambition (LifeOS Overview phrases this as "Milestone 1 ~90% complete"):

**Working end-to-end:**

- Cargo workspace with strict 4-crate dependency direction (common ← math ← core ← cli).
- 120 `#[test]` functions across 12 files, all passing.
- `vynapse-math`: `Shape`, `Tensor<T>`, element-wise arithmetic (`tensor_add`/`tensor_sub`/`tensor_mul`/`tensor_div`), `matrix_vector_mult`, `transpose_2d`, `reshape_tensor`. 54 tests.
- `vynapse-core/traits/`: nine trait interfaces.
- `vynapse-core/components/`: `FixedTopologyGenome`, `GaussianMutation`, `UniformCrossover`, `TournamentSelection`, `Sigmoid`, `MeanSquaredError`, `TaskBasedFitness`.
- `vynapse-core/trainers/evolutionary.rs`: 12.4 KB working trainer.
- `vynapse-core/training_setup/`: `EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`.
- `vynapse-core/tasks/`: `PowersOfTwo`, `XOR`.
- Integration test (`test_evolutionary_trainer_learns_powers_of_two`) running 500 individuals × 100 generations against `PowersOfTwo(5)` with sigmoid activation, MSE loss, tournament-3 selection, Gaussian σ=0.15 mutation, uniform p=0.5 crossover.

**Not built (Milestone 1 blockers):**

- No CLI. `vynapse-cli/src/main.rs` is 44 bytes — `fn main() { println!("Hello World!"); }`. The Milestone 1 deliverable `vynapse train --task powers_of_two --generations 100` does not exist; the system is invoked via `cargo test test_evolutionary_trainer_learns_powers_of_two -- --nocapture`.
- No deterministic seeding. Every component creates its own `rand::rng()` thread-local — no shared seed, no reproducibility.
- No CSV logging. Per-generation fitness is `println!`'d inside the integration test; no file output, no plotting.
- No configuration loading. All four `config/*.rs` files are 0 bytes; `serde` is declared but unused.

**Not built (everything beyond Milestone 1):**

- `trainers/deap.rs`, `trainers/neat.rs`, `trainers/sgd.rs`, `trainers/hybrid.rs`, `trainers/static_graph.rs` — all 0 bytes.
- `components/genome/neat_genome.rs`, `components/mutation/uniform.rs`, `components/crossover/arithmetic.rs`, `components/selection/fitness_proportionate.rs`, `components/selection/roulette.rs` — all 0 bytes.
- `tasks/cartpole.rs` — 0 bytes.
- No autodiff infrastructure, no `Node`/`GradientTape`, no batched matrix multiplication, no GPU backend, no SIMD, no parallel evaluation.
- `utils/metrics.rs`, `utils/rng.rs` — both 0 bytes.

## Current state

Status: paused. Last meaningful commit `1c01e02` landed 21 December 2025 — 4 months stale at last LifeOS verification (2026-04-24). LifeOS phrases this as: *"Vynapse is bursty, not continuous. The five-month pause is real data. Confirming with Caner whether this is the next live project or a known-paused one is worth doing before investing heavy vault cycles."* The Suggestions document explicitly flags Cernio and Claude Config as likely higher-priority active projects. No in-flight work captured in a Work/ folder; the December 2025 refactor closed cleanly against its plan file and nothing new has been queued.

## Gaps and known limitations

Architectural and numerical:

- **No biases on `FixedTopologyGenome`.** Weight count is `Σ shape[i] * shape[i+1]` — inter-layer only, no bias terms. XOR is borderline-solvable without biases; any real regression task needs them.
- **Hardcoded hidden-layer size in `TaskBasedFitness`.** `hidden_size = 4` at `task_based.rs:212`. Genomes whose shape disagrees with this are rejected by the fitness function.
- **Activation applied to both layers.** Sigmoid on hidden AND output layers means output is always in `(0, 1)`. Fatal for regression on `PowersOfTwo` (targets > 1 unreachable). Best achievable fitness on `PowersOfTwo(5)` is ~0.0035.
- **Benchmarks saturated.** Only XOR (4 fixed rows) and PowersOfTwo (sigmoid-impossible for `max_input ≥ 2`). Neither leaves room to compare configurations meaningfully.
- **No matrix-matrix multiplication.** `vynapse-math` has only `matrix_vector_mult`. Milestone 5 (SGD + batched MNIST) needs full matmul.
- **No reshape views.** `reshape_tensor` always clones the data, unlike NumPy/PyTorch zero-copy reshape.

Algorithmic:

- **No elitism.** Generation `k+1` has no guaranteed presence of generation `k`'s best individual. README's checked "(μ+λ) population replacement" claim is factually wrong.
- **Positional parent pairing.** Child `i`'s parents are positions `i` and `(i+1) mod N` of the selection output. Statistical bias rather than systematic, but not the textbook "sample two parents independently."
- **`TargetReached` is a dead `ConvergenceStatus` variant.** Declared but never set; no `target_fitness` field anywhere.
- **Stagnation check is strict inequality.** A plateau (tie with previous best) increments the stagnation counter — terminates plateaued runs early.
- **Weight clamp `[-5, 5]` is undocumented.** Visible only in `gaussian.rs:855`, not in README or comments.

State machine:

- **`is_converged()` returns `true` for fresh `WaitingToStart` state** (defined as `status != Running`). A caller checking `if !trainer.is_converged() { run() }` sees "done" on a fresh trainer.
- **`step()` post-convergence does not error.** `update_generation` rejects only `WaitingToStart`; `MaxIterationsReached` and `Stagnated` allow further stepping that bumps the generation counter past the limit.
- **`FitnessStats::validate()` fails on fresh state.** Empty `fitness_history` triggers `EvolutionError`. The trainer's `reset()` calls `fitness_stats.reset()` which calls `validate()` first — so `reset()` on a fresh-never-run trainer errors. Latent (not hit in current tests).

Tests and documentation:

- **`GaussianMutation` has zero `#[test]` functions.** All coverage is indirect through trainer tests.
- **`UniformCrossover` has zero `#[test]` functions.**
- **`Shape::new` and `Tensor::get/set` edge cases untested directly.**
- **Integration test assertions are weak.** `final_best > 0.0` is trivially true (fitness `1/(1+error)` is always positive). `final_best >= initial_best` is the only learning assertion and could fail legitimately with no elitism — the test relies on 100 generations × pop=500 making regression statistically rare.
- **README opening pitch over-promises by ~9 milestones.** Describes Milestone 10 ("hybrid runtime unifying PyTorch, TensorFlow, DEAP, NEAT") rather than the current evolutionary-only build. LifeOS Gaps explicitly says: "When quoting Vynapse externally, do not quote the README's opening line."

Future-blocking trait surface:

- `Task::get_dataset` is eager `Vec<(Vec<f32>, Vec<f32>)>` — at MNIST scale (60k images × 784 features × 4 bytes = ~180 MB per call, called once per genome per generation) this is catastrophic. Needs to become an iterator.
- `Genome::get_weights` returns owned `Vec<f32>`, allocating per call. Negligible at current scale; measurable at MNIST-sized networks.
- No `Serializable` trait — no checkpointing.
- No `Send + Sync` bounds — no parallel evaluation.

## Direction (in-flight, not wishlist)

No active in-flight work as of LifeOS verification (2026-04-24). The December 2025 modular refactor closed cleanly against its plan file; no new plan documents have been added; no Work/ folder exists for Vynapse in LifeOS. The project is in a clean pause state, fully revivable when Caner returns to it.

The closest-to-actionable next steps (from LifeOS Suggestions §Quick wins, ordered by effort/impact) — none currently being executed:

1. Add elitism (~30 minutes; fixes README's incorrect (μ+λ) claim and removes the largest source of fitness regression risk).
2. Deterministic seeding via a shared `SeedableRng` threaded through trainer (~1 hour; unblocks reproducible tests for everything downstream).
3. Build the CLI (~2-3 hours; `clap` is already a dependency — minimal goal `vynapse train --task <powers_of_two|xor> --generations N --population-size M --seed S`).
4. CSV logging per generation (~1 hour; the single most useful observability feature for an EA project).

These four close Milestone 1 as stated in the README. After that, the natural sequence is Milestone 2 (configuration) → Milestone 3 (DEAP) → Milestone 4 (NEAT), with Milestone 6 (autodiff) as the largest single jump (everything from Milestone 5 onwards requires gradient flow, and the autodiff infrastructure is comparable in complexity to all of `vynapse-math + training_setup` combined).

## Demonstrated skills

- **Rust workspace architecture with strict dependency direction.** Four-crate Cargo workspace (`common`, `math`, `core`, `cli`) with one-way dependencies enforced via Cargo.toml. The base crate exists specifically to break a would-be cycle between math and core around the shared error type.
- **Trait-based generic design with zero-cost abstraction.** `EvolutionaryTrainer<G, M, C, F, S>` parametrises over five trait bounds via Rust monomorphisation. Explicit decision to reject `dyn` trait objects and enums in favour of generics; documented in `Decisions.md`.
- **Modular refactor executed against a written plan.** Pre-Dec 2025 monolithic trainer split into five composed components (`EvolutionConfig`, `Population<G>`, `GeneticOperators<M, C>`, `FitnessStats`, `TrainingStats`) — plan written first as `plans/evolutionary-trainer-modular-refactor.md`, executed in commits `456d9d1` and `1c01e02`, every acceptance criterion ticked.
- **Implements evolutionary algorithms from scratch.** Fixed-topology neuroevolution end-to-end: tournament selection (configurable size, with-replacement sampling), Gaussian-noise mutation (per-weight, clamped), uniform crossover (per-position probabilistic inheritance), generational replacement, stagnation-based early termination.
- **Implements tensor primitives from scratch in safe Rust.** Row-major `Tensor<T>` with strides, generic over element type with per-op trait bounds (`T: Clone + Default + Add<T> + Zero` for add; `+ PartialEq` for div). 54 tests covering element-wise ops, matrix-vector multiplication, transpose, reshape, shape mismatches, divide-by-zero, generic integer tensors.
- **Result-based error handling discipline.** Every fallible function returns `Result<T>`; zero panics in library code; `unwrap()` confined to tests and one safe post-`is_none()`-check unwrap. Three-variant `VynapseError` (`TensorError`/`EvolutionError`/`ConfigError`) via `thiserror`.
- **No `unsafe`, no `Arc`, no interior mutability.** 115.4 KB of Rust through `&mut self` and owned data only (verified by content search). Trades some clone overhead for memory-safety simplicity.
- **Test-colocation discipline.** 120 `#[test]` functions across 12 source files, all colocated with the code they test (no separate `tests/` directory). Access to private fields and helper functions; tests live next to the invariant they guard.
- **Architectural state-machine modelling.** `ConvergenceStatus` enum (5 variants) drives the training run lifecycle. Multiple subtle state-machine edge cases identified and documented in `Gaps.md` — evidence of close-reading of one's own code rather than write-and-forget.
- **Honest self-documentation of project state.** LifeOS Overview explicitly flags the 9-milestone gap between README pitch and current build, the incorrect "(μ+λ)" README claim, the dead `TargetReached` enum variant, the hardcoded `hidden_size = 4`, the saturating sigmoid output on `PowersOfTwo`. The discipline of writing down the gaps your own README hides is itself a strong portfolio signal.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Vynapse/_Overview.md | 100 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
| Projects/Vynapse/Architecture.md | 174 | "- Error propagation across crate boundaries: [[Vynapse/Systems/Error Model]]" |
| Projects/Vynapse/Decisions.md | 158 | "- The gaps these decisions leave open: [[Vynapse/Gaps]]" |
| Projects/Vynapse/Gaps.md | 217 | "- Analytical suggestions for prioritisation: [[Vynapse/Suggestions]]" |
| Projects/Vynapse/Roadmap.md | 174 | "- RL problem domain overlap with NeuroDrive (Milestone 4 cartpole): [[NeuroDrive/_Overview]]" |
| Projects/Vynapse/Suggestions.md | 174 | "- Profile update implied by the project's portfolio value: [[Profile/Projects]], [[Profile/Skills]]" |
| Projects/Vynapse/Systems/_Overview.md | 35 | "- [[Projects/Vynapse/Roadmap]] — direction-of-travel" |
| Projects/Vynapse/Systems/Error Model.md | 132 | "- A specific latent error-handling bug: [[Vynapse/Gaps#validate() fails on fresh state]]" |
| Projects/Vynapse/Systems/Evolutionary Trainer.md | 207 | "- Why the refactor happened and what it replaced: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Genome and Components.md | 197 | "- Stubbed components as roadmap evidence: [[Vynapse/Roadmap]]" |
| Projects/Vynapse/Systems/Tasks and Fitness.md | 167 | "- Why the current benchmarks under-test learning capability: [[Vynapse/Gaps#Benchmarks are saturated]]" |
| Projects/Vynapse/Systems/Tensor and Math.md | 137 | "- Aurix also has a tensor crate — duplication decision pending: [[Aurix/_Overview]], [[Vynapse/Decisions#Tensor crate vs external]]" |
| Projects/Vynapse/Systems/Training Stats and Convergence.md | 158 | "- Why these were split out from the trainer in Dec 2025: [[Vynapse/Decisions#Modular refactor Dec 2025]]" |
| Projects/Vynapse/Systems/Traits Layer.md | 165 | "- Stubs that will need new trait work: [[Vynapse/Gaps#Traits that need extension]]" |
