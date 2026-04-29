---
name: AsteroidsAI
status: dormant
source_repo: https://github.com/Capataina/Asteroids-AI
lifeos_folder: Projects/AsteroidsAI
last_synced: 2026-04-29
sources_read: 15
---

# AsteroidsAI

## One-line summary

Python research playground pitting four optimisation paradigms (Genetic Algorithm, CMA-ES, NEAT, GNN-SAC) against the same Asteroids-style continuous-control environment under a shared HybridEncoder + ComposableRewardCalculator + analytics pipeline — single environment, multiple minds — backed by ~141 Python files across NumPy-only GA/ES, hand-rolled NEAT speciation + innovation tracking, diagonal CMA-ES with Pareto ranking, and PyTorch + torch_geometric GNN-SAC with GATv2Conv backbone.

## What it is

AsteroidsAI is a Python 3.12/3.14 research playground that pits four different optimisation paradigms against the **same** Asteroids-style continuous-control environment, under the same state encoders, reward presets, and analytics pipeline. The point is comparative: "single environment, multiple minds". The repo stretches from 2025-02-25 to 2026-02-22 across 42 commits. The README lists five AI paradigm families and a parallel training dashboard; code reality is **four** methods implemented (NEAT, GA, CMA-ES, GNN-SAC), with Genetic Programming a ghost section having zero code and the parallel-training dashboard's roadmap entirely unchecked. The most durable thing this repo built is not the comparison study — it is the shared evaluator + analytics stack, which any future evolutionary method can plug into via `agent_factory` and get 200+ analytics fields for free.

## Architecture

```
asteroids-ai/
├── ai_agents/             # Base agent + per-method agents
├── interfaces/            # ActionInterface, observation interface
├── training/
│   ├── config/            # GAConfig, ESConfig, NEATConfig, SACConfig
│   ├── scripts/           # train_ga.py / train_es.py / train_neat.py / train_gnn_sac.py
│   ├── drivers/           # GADriver, CMAESDriver, NEATDriver, SACLearner
│   ├── operators/         # GAGeneticOperators, fitness_shaping (unused)
│   └── components/        # Shared: novelty, reward diversity, Pareto utilities
├── encoders/              # HybridEncoder, TemporalStackEncoder, GraphEncoder, VectorEncoder (unused)
├── reward/                # 22 reward components + ComposableRewardCalculator
├── game/                  # HeadlessAsteroidsGame, windowed Asteroids.py (Arcade)
├── analytics/             # TrainingAnalytics, training_data_*.json + summary_*.md outputs
└── tests/                 # 4 test files (1 broken — references deleted modules)
```

All four methods share:
- `HybridEncoder` (47-dim fovea+raycast state) via `encoders/`
- `ComposableRewardCalculator` with shared preset (VelocitySurvivalBonus, DistanceBasedKillReward, ConservingAmmoBonus, ExplorationBonus, DeathPenalty)
- `HeadlessAsteroidsGame` for seeded parallel rollouts
- `TrainingAnalytics` pipeline producing `training_data_*.json` + `training_summary_*.md`

This shared substrate is the comparability story: when results from one method differ, the difference can be attributed to the method rather than the environment.

## Subsystems and components

| Subsystem | Responsibility |
|---|---|
| **Game Engine** | Windowed (Arcade) + headless game, physics, continuous control |
| **State Encoders** | HybridEncoder (active), TemporalStackEncoder, GraphEncoder (for GNN-SAC), VectorEncoder (unused) |
| **Reward System** | 22 reward components + ComposableRewardCalculator preset |
| **Genetic Algorithm** | GADriver, GAGeneticOperators, BLX-alpha crossover, Gaussian mutation, tournament selection with novelty/diversity |
| **NEAT** | NEATDriver, genome.py (12.9KB), innovation tracker, species.py — produces `neat_artifacts/*.dot/.json` per generation |
| **Evolution Strategies (CMA-ES)** | CMAESDriver (diagonal CMA-ES), `fitness_shaping.py` (unused dead code), Pareto ranking, antithetic + CRN |
| **GNN-SAC** | SACLearner, GNNBackbone (GATv2Conv), twin critics, GraphEncoder, graph-native replay |
| **Shared Components** | Novelty archive, reward diversity, Pareto utilities |
| **Analytics Pipeline** | TrainingAnalytics, collectors, markdown + JSON reporting (200+ fields) |

## Technologies and concepts demonstrated

### Languages
- **Python 3.12 / 3.14** — entire codebase, ~141 files, ~806KB Python source.

### Frameworks and libraries
- **Arcade** — windowed game rendering.
- **NumPy** — GA, CMA-ES, NEAT (all NumPy-only).
- **PyTorch** — SAC twin critics, actor network.
- **torch_geometric** — GNN backbone (GATv2Conv) for GNN-SAC.

### Domains and concepts
- **Genetic Algorithms** — BLX-alpha crossover, Gaussian mutation, tournament selection with novelty/diversity scoring.
- **NEAT (Neuroevolution of Augmenting Topologies)** — speciation, innovation tracking, topology growth, hand-rolled.
- **CMA-ES (Covariance Matrix Adaptation)** — diagonal variant with Pareto ranking, antithetic sampling, common random numbers (CRN) for variance reduction.
- **Soft Actor-Critic (SAC)** — twin critics, entropy-regularised policy, graph-native replay buffer.
- **Graph Neural Networks** — GATv2Conv (Graph Attention v2) backbone for SAC; the state is encoded as a graph rather than a flat vector.
- **Comparative ML benchmarking** — single environment, single state encoder, single reward preset, single analytics pipeline; method swapped at the driver layer.
- **Reward shaping / composable reward systems** — 22 reward components plugged together via ComposableRewardCalculator preset.
- **Novelty / diversity / Pareto-multi-objective optimisation** — shared utilities used by GA + ES.

## Key technical decisions

- **Same environment, same encoders, same reward preset for all methods** — comparability story requires shared substrate.
- **`HybridEncoder` (47-dim fovea+raycast)** as the canonical state — TemporalStackEncoder and VectorEncoder retained but unused; GraphEncoder used only by GNN-SAC.
- **`ComposableRewardCalculator` preset shared across methods** — keeps reward signal identical so methods are comparable.
- **NumPy for the evolutionary methods, PyTorch only for SAC** — keeps the dependency surface small for the methods that don't need autodiff.
- **`fitness_shaping.py` retained as dead code** — earlier ES driver design exists in repo but is not wired into the current CMAESDriver.
- **No build manifest** — no `pyproject.toml`, no `requirements.txt` — flagged in `plans/GNN_SAC.md`.
- **GP (Genetic Programming) declared but not built** — README mentions five paradigms; only four implemented.

## What is currently built

- 4 working training paradigms: GA (`train_ga.py`), CMA-ES (`train_es.py`), NEAT (`train_neat.py`), GNN-SAC (`train_gnn_sac.py`).
- 4 training summary outputs (45–68KB each) — runs completed for all four methods.
- 16MB of training JSON, 50 NEAT generation snapshots (.json + .dot), one 623KB SAC checkpoint.
- 1 broken test file (`test_ga_dimensions.py` references deleted modules).
- Zero CI / zero build manifest.

## Current state

Dormant. Last commit `1c55da4` (2026-02-22) — literally titled "Mark roadmap items complete in README" — a tidy-up commit. Real development ceased in January 2026 after GNN-SAC landed. Project has been dormant for ~2 months at LifeOS verification.

## Gaps and known limitations

- **Genetic Programming is a ghost** — README mentions it, no code exists.
- **Parallel training dashboard not built** — every roadmap checkbox unchecked.
- **Cross-method comparability is weak** — each method tuned individually with different `POPULATION_SIZE`, `SEEDS_PER_AGENT`, `CRN` mode defaults. The shared environment + encoder + reward preset gives infrastructure-level comparability but not hyperparameter-level fairness.
- **`test_ga_dimensions.py` broken** — references deleted modules.
- **No build manifest** — `pyproject.toml`, `requirements.txt`, `requirements-rl.txt` all absent.
- **17 reward components never exercised in a run** — flagged in Reward System gap.
- **`get_tick()` broken** — flagged in Game Engine gap.
- **Unused arcade rendering code** — flagged in Game Engine gap.
- **Encoder drift** — schema versioning not present; flagged in State Encoders.
- **VectorEncoder is dead code.**

## Direction (in-flight, not wishlist)

Dormant. If revived: address cross-method comparability (standardise hyperparameters), wire the parallel-training dashboard, fix `test_ga_dimensions.py`, add a build manifest. The most durable extension would be opening up `agent_factory` so external evolutionary methods plug in trivially.

## Demonstrated skills

- **Comparative ML research platform** — single environment, four optimisation paradigms, shared analytics; the comparability infrastructure is the durable contribution.
- **NumPy-only evolutionary methods** — GA + CMA-ES + NEAT all hand-rolled in NumPy without ML framework dependency.
- **NEAT from scratch** — speciation, innovation tracking, topology growth via genome.py + species.py + innovation tracker.
- **CMA-ES from scratch** — diagonal variant with Pareto ranking, antithetic sampling, common random numbers.
- **Soft Actor-Critic in PyTorch + torch_geometric** — twin critics, entropy regularisation, GNN-encoded state via GATv2Conv backbone, graph-native replay.
- **Composable reward design** — 22 components combined via preset; reward diversity utilities; Pareto multi-objective scoring.
- **Analytics pipeline architecture** — TrainingAnalytics + collectors + markdown/JSON reports with 200+ analytics fields, plug-in-via-`agent_factory`.
- **Cross-paradigm Python ML breadth** — NumPy + PyTorch + torch_geometric in one project; range matters more than depth here for portfolio presentation.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/AsteroidsAI/_Overview.md | 130 | "#project/asteroids-ai #lang/python #paradigm/evolutionary #paradigm/neat #paradigm/cma-es #paradigm/sac #ml/rl #ml/gnn #status/dormant" |
| Projects/AsteroidsAI/Architecture.md | 290 | "- [[AsteroidsAI/Roadmap]] — the planned parallel dashboard and its architectural shape" |
| Projects/AsteroidsAI/Decisions.md | 278 | "- [[AsteroidsAI/Roadmap]] — decisions deliberately deferred appear as roadmap items" |
| Projects/AsteroidsAI/Gaps.md | 257 | "- [[AsteroidsAI/Systems/Analytics Pipeline]] — analytics polish gaps" |
| Projects/AsteroidsAI/Roadmap.md | 282 | "- [[Projects/_Overview]] — AsteroidsAI relative to Caner's other active projects" |
| Projects/AsteroidsAI/Systems/_Overview.md | 43 | "- [[Projects/AsteroidsAI/Roadmap]] — direction-of-travel" |
| Projects/AsteroidsAI/Systems/Analytics Pipeline.md | 257 | "- [[AsteroidsAI/Roadmap]] — analytics polish is much of the remaining in-repo roadmap" |
| Projects/AsteroidsAI/Systems/Evolution Strategies.md | 291 | "- [[AsteroidsAI/Roadmap]] — the Easy/Medium/Hard roadmap from the ES plan is the work" |
| Projects/AsteroidsAI/Systems/GNN-SAC.md | 355 | "- [[NeuroDrive/_Overview]] — NeuroDrive's asymmetric PPO (actor 2x64, critic 2x128) is the conventional-RL counterpart" |
| Projects/AsteroidsAI/Systems/Game Engine.md | 193 | "- [[AsteroidsAI/Gaps]] — broken `get_tick()`, wrap-aware collision, unused arcade code" |
| Projects/AsteroidsAI/Systems/Genetic Algorithm.md | 225 | "- [[Vynapse/_Overview]] — Caner's Rust neuroevolution engine; solves similar fixed-topology problem" |
| Projects/AsteroidsAI/Systems/NEAT.md | 226 | "- [[Vynapse/_Overview]] — Vynapse's `trainers/neat.rs` is a 0-byte stub; AsteroidsAI's NEAT is the reference implementation" |
| Projects/AsteroidsAI/Systems/Reward System.md | 174 | "- [[AsteroidsAI/Gaps]] — 17 components never exercised in a run" |
| Projects/AsteroidsAI/Systems/Shared Components.md | 225 | "- [[AsteroidsAI/Gaps]] — method-parity normalisation not done; cross-method bonus" |
| Projects/AsteroidsAI/Systems/State Encoders.md | 219 | "- [[AsteroidsAI/Gaps]] — encoder drift, schema versioning, VectorEncoder dead code" |
