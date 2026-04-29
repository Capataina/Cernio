---
name: Vynapse
status: paused
source_repo: https://github.com/Capataina/Vynapse
lifeos_folder: Projects/Vynapse
last_synced: 2026-04-29
sources_read: 14
---

# Vynapse

## One-line summary

From-scratch Rust evolutionary-algorithm engine — runs one end-to-end paradigm (fixed-topology neuroevolution with tournament selection, Gaussian mutation, uniform crossover, μ+λ replacement) on toy tasks via 120 inline tests, backed by a trait-based 4-crate component architecture designed to accept NEAT, SGD, autodiff, and hybrid trainers as future milestones (currently empty stubs).

## What it is

Vynapse is Caner's from-scratch Rust ML engine. It started 2025-07-12 with a bare tensor implementation, built out evolutionary primitives through summer 2025, went quiet after late July, and landed a modular-architecture refactor on 2025-12-21. The current HEAD (`1c01e02`) is the refactor; the plan file `plans/evolutionary-trainer-modular-refactor.md` marks every acceptance criterion done. The README opening claim — *"a Rust-native deep learning and neuroevolution engine built as a hybrid learning runtime — unifying the optimization paradigms of PyTorch, TensorFlow, DEAP, and NEAT"* — describes Milestone 10. The current build is Milestone 1 (~90% complete): one paradigm compiled cleanly with the rest stubbed as a scaffolding commitment. Empty (0-byte) stub files for NEAT, DEAP, SGD, autodiff, static-graph, hybrid, and config-loading are visible in the module tree as a roadmap-as-code artefact.

## Architecture

4-crate Cargo workspace: `vynapse-core`, `vynapse-math`, `vynapse-cli`, `vynapse-common`. ~115KB of Rust across 66 files. 120 `#[test]` functions across 12 files; tests live inline alongside source modules (no dedicated `tests/` directory).

```
vynapse/
├── vynapse-core/
│   ├── trainers/
│   │   ├── evolutionary.rs    # 12.4KB — the working trainer
│   │   ├── neat.rs            # 0 bytes (Milestone 4)
│   │   ├── deap.rs            # 0 bytes (Milestone 3)
│   │   ├── sgd.rs             # 0 bytes (Milestone 5)
│   │   ├── static_graph.rs    # 0 bytes (Milestone 7)
│   │   └── hybrid.rs          # 0 bytes (Milestone 8)
│   ├── components/
│   │   ├── genome/            # FixedTopologyGenome (working) + neat_genome.rs (stub)
│   │   ├── selection/         # TournamentSelection
│   │   ├── crossover/         # UniformCrossover
│   │   ├── mutation/          # GaussianMutation
│   │   └── activation/        # Sigmoid, Tanh, ReLU
│   ├── tasks/                 # PowersOfTwo (working), XOR, cartpole stub
│   └── config/                # Four empty files (Milestone 2)
├── vynapse-math/
│   └── ops/
│       └── transform.rs       # 16KB largest source file
├── vynapse-cli/
│   └── main.rs                # 44 bytes — println!("Hello World!");
└── vynapse-common/
    └── error/                 # Result type + error enum
```

The "scaffolding for what doesn't exist" pattern is deliberate: empty files declare future milestones in the module tree itself.

## Subsystems and components

| Subsystem | Responsibility | State |
|---|---|---|
| **Tensor and Math** | Generic tensor / shape primitives, math ops | Working |
| **Genome and Components** | `FixedTopologyGenome` (network weights as flat `Vec<f64>`), tournament selection, Gaussian mutation, uniform crossover | Working |
| **Evolutionary Trainer** | μ+λ replacement, fitness eval, tournament parent selection, crossover-rate-gated breeding, generation loop | Working (12.4KB, 1 integration test runs end-to-end) |
| **Training Stats and Convergence** | Population, GeneticOperators, stats infrastructure | Working — split out from trainer in Dec 2025 refactor |
| **Tasks and Fitness** | XOR + PowersOfTwo + `TaskBasedFitness` (MSE-with-Sigmoid scoring) | Working but limited — sigmoid saturates against `2^n` outputs like 256 |
| **Traits Layer** | `Genome`, `Mutation`, `Crossover`, `Selection`, `LossFunction`, `ActivationFunction` traits | Working — designed to accept future paradigms |
| **Error Model** | `VynapseError` + `Result<T, VynapseError>` | Working with one bug: `validate()` fails on fresh `Population` (empty-population corner case) |
| **CLI** | Entry binary | Stub — `fn main() { println!("Hello World!"); }` |
| **Config Layer** | JSON config loader | Empty — Milestone 2 |
| **DEAP / NEAT / SGD / autodiff / static-graph / hybrid / GPU** | Future milestones | All 0-byte stubs |

## Technologies and concepts demonstrated

### Languages
- **Rust 2024** — entire codebase, 115KB across 66 files, zero `unsafe` blocks.

### Libraries
- Standard library only for the working paradigm (no external ML deps in `vynapse-core`).

### Domains and concepts
- **Evolutionary algorithms / neuroevolution** — fixed-topology neural network optimisation via genetic operators (tournament selection, Gaussian mutation, uniform crossover, μ+λ replacement).
- **Trait-based modular architecture** — Genome, Mutation, Crossover, Selection, LossFunction, ActivationFunction traits designed so a new paradigm (NEAT, SGD, autodiff) can be added by implementing the trait set.
- **Plans-driven development** — `plans/evolutionary-trainer-modular-refactor.md` was written before the refactor; each acceptance criterion ticked individually in the file. Same discipline appears in LifeOS, Cernio.
- **Anti-puffing scaffold-as-code** — empty (0-byte) stub files declare future milestones in the module tree, so the README's 10-milestone roadmap is visible at the file system level.

## Key technical decisions

- **Plans-driven development** — the Dec 2025 refactor was preceded by `plans/evolutionary-trainer-modular-refactor.md`; the plan-then-execute cadence is itself a design decision.
- **Trait-based modularity for future paradigms** — Genome/Mutation/Crossover/Selection/LossFunction/ActivationFunction as traits so NEAT, SGD, autodiff can be plugged in.
- **Hardcoded hidden size in PowersOfTwo task** (1→4→1 MLP) — known limitation in the current task; flagged in `Suggestions.md`.
- **No elitism in evolutionary trainer** — entire population replaced each generation; the best genome can be lost generation-to-generation. Flagged in `Decisions.md` as a future fix.
- **Modular refactor (Dec 2025)** — trainer split from population/operators/stats; documented in `Decisions#Modular refactor`.
- **No GPU dependency** — Milestone 9; decided against bringing in `wgpu` until a paradigm exists that needs it.

## What is currently built

- 66 Rust files (~115KB), 4 crates, 120 `#[test]` functions across 12 files, all inline.
- 1 trainer paradigm (evolutionary) working end-to-end, validated by `test_evolutionary_trainer_learns_powers_of_two` integration test.
- 6-component trait layer (Genome, Mutation, Crossover, Selection, LossFunction, ActivationFunction).
- 1 task (PowersOfTwo) running, 1 task (XOR) defined, cartpole stubbed.
- 16 empty (0-byte) stub files declaring future milestones in the module tree.
- 36 commits total (HEAD `1c01e02`, 2025-12-21).

## Current state

Paused. HEAD has not advanced since 2025-12-21. The project's commit cadence shows: 32 commits in first 17 days → 1 commit Jul→Dec (5-month silence) → 3 commits Dec 19–21 (the deliberate restructuring pass). The Dec 2025 return is itself the most recent work; no commits since.

## Gaps and known limitations

- **No CLI** — `vynapse-cli/src/main.rs` is `Hello World!`; the README's promised `vynapse train --task powers_of_two --generations 100` does not exist.
- **No configuration loading** — Milestone 2 stubs.
- **No CSV/JSON logging** — Milestone 1's "Observable Training" not complete.
- **No deterministic seeding** — three of five Milestone 1 bullets unchecked.
- **`validate()` fails on fresh `Population`** — empty-population corner case in Error Model.
- **No elitism in evolutionary trainer** — best genome can be lost between generations.
- **PowersOfTwo task uses hardcoded 1→4→1 MLP and Sigmoid output** — saturates against outputs like 256; closer to a sanity check than a real benchmark.
- **No NEAT, DEAP, SGD, autodiff, static-graph, hybrid, GPU paradigms** — all empty stubs (Milestones 3–9).
- **Tensor crate duplicates Aurix's tensor primitives** — consolidation question pending (`Decisions#Tensor crate vs external`).

## Direction (in-flight, not wishlist)

Paused. If revived, the natural next steps are: finish Milestone 1 observability (deterministic seeding, CSV/JSON logging, basic CLI), then Milestone 2 (config loading), then choose between NEAT (Milestone 4 — has Vynapse's empty `neat.rs` + AsteroidsAI's working Python NEAT as a reference) and SGD (Milestone 5 — the path toward autodiff).

## Demonstrated skills

- **From-scratch evolutionary algorithm trainer in Rust** — μ+λ replacement, tournament selection, Gaussian mutation, uniform crossover, fitness-eval pipeline, no ML framework dependency.
- **Trait-based modular architecture in Rust** — 6-component trait layer designed so future paradigms (NEAT, SGD, autodiff, hybrid) can be added by implementing the trait set.
- **Cargo workspace design** — 4 crates (`vynapse-core`, `vynapse-math`, `vynapse-cli`, `vynapse-common`) with clear dependency direction.
- **Inline-tests-as-design discipline** — 120 `#[test]` functions across 12 files; tests live with source.
- **Plans-driven development methodology** — plan file written before refactor, acceptance criteria ticked individually; pattern echoes across LifeOS and Cernio.
- **Scaffold-as-code anti-puffing** — empty-stub files declare future milestones in the module tree, so the discrepancy between README ambition and code reality is visible at the filesystem level.
- **Zero `unsafe` Rust** — 115KB of safe Rust + zero `unsafe` blocks.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Vynapse/_Overview.md | 135 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
| Projects/Vynapse/Architecture.md | 242 | "- Error propagation across crate boundaries: [[Vynapse/Systems/Error Model]]" |
| Projects/Vynapse/Decisions.md | 184 | "- The gaps these decisions leave open: [[Vynapse/Gaps]]" |
| Projects/Vynapse/Gaps.md | 268 | "- Analytical suggestions for prioritisation: [[Vynapse/Suggestions]]" |
| Projects/Vynapse/Roadmap.md | 187 | "- RL problem domain overlap with NeuroDrive (Milestone 4 cartpole): [[NeuroDrive/_Overview]]" |
| Projects/Vynapse/Suggestions.md | 197 | "- Profile update implied by the project's portfolio value: [[Profile/Projects]]" |
| Projects/Vynapse/Systems/_Overview.md | 42 | "- [[Projects/Vynapse/Roadmap]] — direction-of-travel" |
| Projects/Vynapse/Systems/Error Model.md | 114 | "- A specific latent error-handling bug: [[Vynapse/Gaps#validate() fails on fresh Population]]" |
| Projects/Vynapse/Systems/Evolutionary Trainer.md | 244 | "- Why the refactor happened and what it replaced: [[Vynapse/Decisions#Modular refactor]]" |
| Projects/Vynapse/Systems/Genome and Components.md | 218 | "- Stubbed components as roadmap evidence: [[Vynapse/Roadmap]]" |
| Projects/Vynapse/Systems/Tasks and Fitness.md | 171 | "- Why the current benchmarks under-test learning capability: [[Vynapse/Gaps#Benchmarks under-test]]" |
| Projects/Vynapse/Systems/Tensor and Math.md | 126 | "- Aurix also has a tensor crate — duplication decision pending: [[Aurix/_Overview]]" |
| Projects/Vynapse/Systems/Training Stats and Convergence.md | 185 | "- Why these were split out from the trainer in Dec 2025: [[Vynapse/Decisions#Modular refactor]]" |
| Projects/Vynapse/Systems/Traits Layer.md | 185 | "- Stubs that will need new trait work: [[Vynapse/Gaps#Traits that need extension]]" |
