---
name: AsteroidsAI
status: dormant
source_repo: https://github.com/Capataina/Asteroids-AI
lifeos_folder: Projects/AsteroidsAI
last_synced: 2026-05-31
sources_read: 15
---

# AsteroidsAI

## One-line summary

Python comparative-ML playground that pits four optimisation paradigms (GA, diagonal CMA-ES, NEAT, GNN-SAC) against the same Asteroids-style continuous-control environment through a shared evaluator, state-encoder family, reward preset, and analytics pipeline.

## What it is

AsteroidsAI is a research playground built to compare four optimisation paradigms on the same task: "single environment, multiple minds". The repo spans 2025-02-25 to 2026-02-22 across 42 commits and is implemented in Python (3.12 / 3.14 — both Python versions show up in `.pyc` artefacts) with Arcade for rendering, NumPy for the evolutionary policy stack, and PyTorch + `torch_geometric` for the SAC stack. The substrate (game, encoders, reward calculator, evaluator, analytics) is shared across methods so that observed differences can be attributed to the optimisation paradigm rather than the environment. The README pitches five paradigms and a parallel training dashboard; LifeOS records that four methods are actually implemented (Genetic Programming is a phantom section with zero code) and the dashboard has every roadmap checkbox unchecked. The project is currently dormant — last commit `1c55da4` 2026-02-22 was a README cleanup.

## Architecture

Top-down dependency-only layering: `training scripts → training core/methods/analytics → agents → interfaces → game`. Every method obeys the same per-step loop: `game state → EnvironmentTracker → StateEncoder.encode → BaseAgent.get_action → ActionInterface.to_game_input → Game.on_update → MetricsTracker + RewardCalculator`.

```
┌──────────────────────────────────────────────────────────────────┐
│  Training scripts (training/scripts/)                            │
│    train_ga.py  train_es.py  train_neat.py  train_gnn_sac.py     │
│    view_gnn_sac.py  simulate_gnn_sac.py                          │
└──────────────────────────────────────────────────────────────────┘
                             │
         ┌───────────────────┼─────────────────────┐
         ▼                   ▼                     ▼
┌────────────────┐ ┌──────────────────┐ ┌──────────────────────┐
│ training/core/ │ │ training/methods │ │ training/analytics/  │
│  population_   │ │  genetic_algo/   │ │  collection/         │
│  evaluator     │ │  evolution_      │ │  analysis/           │
│  episode_runner│ │  strategies/     │ │  reporting/          │
│  display_mgr   │ │  neat/  sac/     │ │  TrainingAnalytics   │
└────────────────┘ └──────────────────┘ └──────────────────────┘
                             │
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│  Agents (ai_agents/) — BaseAgent contract                        │
│    NNAgent (NumPy, used by GA + ES)                              │
│    NEATAgent (wraps compiled NEAT feedforward network)           │
│    SACAgent (PyTorch inference wrapper)                          │
│    NNAgentTF / feedforward_tf — present, unused                  │
└──────────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│  Interfaces (interfaces/)                                        │
│    StateEncoder: HybridEncoder(47), TemporalStackEncoder(329),   │
│                  GraphEncoder, VectorEncoder (unused)            │
│    ActionInterface (boolean | continuous)                        │
│    EnvironmentTracker (spatial queries, wrapped distances)       │
│    MetricsTracker, ComposableRewardCalculator + 22 components    │
└──────────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│  Game (game/ + Asteroids.py)                                     │
│    AsteroidsGame (arcade.Window, rendering + manual play)        │
│    HeadlessAsteroidsGame (seeded parallel rollouts)              │
│    Player / Bullet / Asteroid classes, globals.py constants      │
└──────────────────────────────────────────────────────────────────┘
```

The architectural centre of gravity is `training/core/population_evaluator.py` — 45 KB, the largest Python file in the repo, shared by GA, ES and NEAT. Its signature accepts an `agent_factory` so each method injects how a candidate (param vector for GA/ES, genome for NEAT) becomes an agent. SAC has its own step-based collector loop (replay-based, not generation-based) and deliberately does not flow through this evaluator.

Folder weights (from LifeOS `scan_repo.py`): `training/` 3.09 MB (includes ~2.1 MB NEAT+SAC artefacts), `interfaces/` 240 KB (23% of source), `plans/` 178 KB (17%, design docs), `game/` 91 KB, `ai_agents/` 57 KB, `tests/` 60 KB. 141 Python files, ~806 KB Python source. 15 MB of generated training artefacts (JSON exports, NEAT `.dot`/`.json` per-generation snapshots, 623 KB SAC checkpoint) are checked into git.

Design invariants enforced across the layering: `StateEncoder.clone()` returns a fresh instance per parallel rollout thread (temporal buffers must not be shared); `update_internal_rewards=False` during training so the windowed game's legacy reward calculator does not interfere; collision detection uses explicit radii from `globals.py` because sprite-texture collision does not work headless; toroidal screen wrapping via `EnvironmentTracker.get_distance` (HybridEncoder rays use "ghost targets" — virtual duplicates of asteroids into 8 surrounding grid cells); `train_ga/es/neat` must never be broken by SAC changes (codified non-breaking principle).

## Subsystems and components

### Game Engine (`game/` + `Asteroids.py`)
Arcade-based 2D continuous-control game with two modes: `AsteroidsGame` (windowed, manual or playback) and `HeadlessAsteroidsGame` (seeded, deterministic, used by parallel rollouts). Player/Bullet/Asteroid classes; `globals.py` holds physics + collision constants. Toroidal wrapping. `debug/visuals.py` provides overlays including HybridEncoder ray debug rendering.

### State Encoders (`interfaces/`)
Four `StateEncoder` implementations behind a `clone()`-mandatory abstract base:
- **HybridEncoder** (47 dims) — 3 proprioception + 3 fovea asteroids × 4 features + 16 rays × 2 features. Default for GA, ES, NEAT.
- **TemporalStackEncoder** — wraps `HybridEncoder` with `N=4` stack + deltas → 329 dims. Used by ES only.
- **GraphEncoder** — variable-size graph payload (nodes + edges). Used by GNN-SAC.
- **VectorEncoder** (35 dims, nearest-N only) — present, unused; produced spin-lock turret policies in earlier testing.

### Reward System (`interfaces/`)
`ComposableRewardCalculator` aggregates 22 reward components. The shared training preset across all four methods is: `VelocitySurvivalBonus`, `DistanceBasedKillReward`, `ConservingAmmoBonus`, `ExplorationBonus`, `DeathPenalty`. LifeOS records 17 components are never exercised by any current preset.

### Genetic Algorithm (`training/methods/genetic_algorithm/`)
`GAConfig`: population 15, seeds 20, generations 500. `GADriver` implements tournament selection over combined score (fitness + novelty + diversity), BLX-α crossover, Gaussian mutation, elitism of top ~10%. Uses NumPy `NNAgent` policy (1,227 params for `HybridEncoder(47) → 24 → 24 → 3`). `use_common_seeds=False` by default. Best individual is in-memory only — no genome persistence.

### Evolution Strategies (`training/methods/evolution_strategies/`)
`ESConfig`: population 100, generations 500, sigma 0.15. `CMAESDriver` implements diagonal (per-parameter) covariance CMA-ES with target-rate adaptation (`CMAES_COV_TARGET_RATE=1e-3`), Pareto rank-based selection over `[hits, time_alive, softmin_ttc]`, antithetic sampling, and CRN by default. Wraps `HybridEncoder` in `TemporalStackEncoder(N=4, include_deltas=True)` → 7,995 params (6.5× GA). A classic ES `driver.py` (17.7 KB) and `fitness_shaping.py` (3 KB) exist but are dead code; AdamW / rank-transformation / elitism settings linger in `ESConfig` unread by the CMA-ES driver.

### NEAT (`training/methods/neat/`)
`NEATConfig`: population 50, generations 500, target species 8. `NEATDriver` with `genome.py` (12.9 KB), innovation tracker, `species.py` for speciation. Per-generation artefacts written to `training/neat_artifacts/gen_NNNN_best.{json,dot}` plus `best_overall.{json,dot}` (50 generations × 2 files ~ 1.5 MB). Uses `ActionInterface(turn_deadzone=0.03)` (GA/ES use 0.0) to accommodate sigmoid-output drift of newly-added nodes. Scalar fitness for reproduction (with `FITNESS_STD_PENALTY_RATIO=1.0` subtracting std-dev from fitness), Pareto for display only. CRN on by default.

### GNN-SAC (`training/methods/sac/`)
`SACConfig`: 500,000 env steps, batch 256, replay 100K. `SACLearner` implements Soft Actor-Critic with a `GNNBackbone` (GATv2Conv layers from `torch_geometric`), twin critics, graph-native replay buffer that stores `(graph_state, action, reward, next_graph, done)`. Per outer loop: collect transitions from `NUM_COLLECTORS` parallel `HeadlessAsteroidsGame` instances in continuous-control mode → run `UPDATES_PER_STEP` SAC updates from replay → every `EVAL_EVERY_EPISODES=5` eval on fixed seeds `[1001..1005]` → every `SAVE_EVERY_STEPS=50000` save `best_sac.pt` (GNN + actor weights + eval metadata). Only checkpoint with disk persistence; replay buffer and optimizer state are not saved (training is not resumable, only inference).

### Shared Components (`training/`)
Novelty calculation, reward-diversity utilities, behaviour-vector aggregation, Pareto utilities. Plugged into the evaluator's per-agent metric collection. LifeOS notes method-parity normalisation is not done — novelty/diversity bonus magnitudes are not directly comparable across methods.

### Analytics Pipeline (`training/analytics/`)
Three-stage: `collection/` (gathers per-agent and per-generation metrics during evaluator runs), `analysis/` (post-processing including quantile sketches, Pareto fronts where applicable), `reporting/` (writes per-run `training_data_*.json` machine output and `training_summary_*.md` markdown report). `TrainingAnalytics` facade is the single entry point. 200+ analytics fields recorded per generation. Produces 45-68 KB markdown summaries per method.

## Technologies and concepts demonstrated

### Languages
- **Python 3.12 / 3.14** — entire codebase. Both Python versions show up in `__pycache__` bytecode (no explicit version pin).

### Frameworks and libraries
- **Arcade** — 2D game rendering, windowing, manual-play input. Powers both the windowed and headless game variants.
- **NumPy** — feedforward policy implementation (`FeedforwardPolicy`, `NNAgent`), all evolutionary methods' inference path, CMA-ES covariance updates.
- **PyTorch** — SAC actor/critic networks, autograd for SAC policy gradient.
- **torch_geometric** — `GATv2Conv` layers in the GNN-SAC backbone; graph-batching primitives in the SAC replay path.
- **TensorFlow stack present but unused** — `feedforward_tf.py` (5.5 KB), `nn_agent_tf.py` (2.6 KB), `population_evaluator_tf.py` (37.7 KB) all shipped as dead code.

### Runtimes / engines / platforms
- **Arcade game loop** — drives both manual playback and the headless deterministic rollout mode that every training method consumes.
- **`ThreadPoolExecutor(max_workers=os.cpu_count())`** — the evaluator's per-generation parallelism mechanism. Threads work because NumPy releases the GIL during BLAS calls and the per-step Python work is light.

### Tools
- **No build system** — no `pyproject.toml`, no `requirements.txt`, no `requirements-rl.txt`, no conda env. Fresh clones must reverse-engineer the dependency set (particularly painful for `torch` + `torch_geometric` + `torch_scatter` + `torch_sparse` PyG backend compatibility).
- **No CI** captured in LifeOS source evidence.

### Domains and concepts
- **Neuroevolution (fixed-topology)** — Genetic Algorithm with BLX-α crossover, Gaussian mutation, tournament selection, novelty + diversity bonuses combined into a single selection score.
- **Diagonal CMA-ES** — per-parameter covariance (avoids O(n²) memory and O(n³) update of full CMA), target-rate adaptation, antithetic sampling (`+ε`/`−ε` pairs sharing a CRN seed), Pareto rank-based selection on `[hits, time_alive, softmin_ttc]`.
- **NEAT (NeuroEvolution of Augmenting Topologies)** — genome representation, innovation-number tracking, speciation for diversity protection, topology growth, fitness penalty by std-dev for stability preference.
- **Soft Actor-Critic (SAC)** — entropy-regularised off-policy actor-critic, twin Q-networks, continuous control with a GNN backbone consuming variable-size graph observations.
- **Graph Neural Networks** — `GATv2Conv` (graph attention) layers as the SAC policy/value backbone, variable-cardinality observation space.
- **Common Random Numbers (CRN)** — used for variance reduction in ES antithetic sampling and NEAT fitness stability; deliberately off for GA to preserve generalisation pressure.
- **Multi-objective optimisation** — Pareto-front selection in ES; mixed scalar+novelty+diversity scoring in GA; Pareto-for-display-only in NEAT.
- **Novelty search and diversity bonuses** — three different integration patterns across the three evolutionary methods.
- **Shared-evaluator design pattern** — single 45 KB evaluator parameterised by `agent_factory`, `state_encoder`, `action_interface`, `reward_factory`, `seeds`, `use_common_seeds`.
- **Composable reward modelling** — 22-component `ComposableRewardCalculator` with a shared preset across methods for comparability.
- **State-encoder design** — hybrid fovea+raycast encoding to defeat spin-lock turret exploits; toroidal-aware ray casting using "ghost targets" across an 8-cell surrounding grid.

## Key technical decisions

- **Four methods, one environment, one reward preset** — chosen to preserve comparability. Alternatives (per-method reward presets, more methods) rejected because they would let each method "win its own game". PPO was explicitly considered and discarded as too on-policy for long-horizon tuning.
- **Dual NumPy + TensorFlow policy stacks, only NumPy wired in** — NumPy is fast enough for CPU `ThreadPoolExecutor`-parallel headless rollouts; TF was kept as optionality for a future GPU-batched ES path but never wired. 37.7 KB of TF code is dead weight as a result.
- **HybridEncoder (fovea + rays), not pure nearest-N** — redundancy is the feature. Pure nearest-N (`VectorEncoder`) produced spin-lock turret policies; the combined fovea (precise aiming on 3 closest threats) + rays (coarse global awareness) encoder kills the exploit.
- **ES uses TemporalStackEncoder; GA and NEAT do not** — ES gradient estimation needs smooth fitness landscapes (single-frame state is too jagged); GA/NEAT are noise-tolerant and don't benefit. Cost: ES has 7,995 params vs GA's 1,227, a 6.5× confound on any cross-method fitness comparison.
- **Pareto-first selection for ES; scalar fitness for GA; scalar reproduction + Pareto display for NEAT** — ES's rank-based update naturally accommodates Pareto ordering and protects diverse skill profiles during the update; GA tournament already mixes fitness + novelty + diversity so Pareto would be redundant; NEAT uses Pareto for display to give humans "best-looking to watch" rather than "highest scalar reward".
- **CRN defaults differ per method** (ES True, GA False, NEAT True) — ES's antithetic sampling requires paired samples to see identical environments; GA's tournament is robust to per-seed noise; NEAT combines CRN with std-dev fitness penalty.
- **Diagonal CMA-ES, not full covariance** — full covariance would be O(n²) memory + O(n³) update on 7,995-param policies, infeasible on CPU. LM-CMA deferred as more complex.
- **Single shared evaluator, dispatched to method-specific drivers** — guarantees consistent rollout semantics across GA/ES/NEAT, at the cost of one 45 KB file.
- **SAC has its own training loop, not evaluator-based** — SAC's timebase is `(env_step)` not `(generation, candidate, seed, step)`; shoehorning would pollute both designs. Non-Breaking Principle codified in the SAC plan: existing GA/ES/NEAT scripts must continue to run without modification.
- **NEAT has `turn_deadzone=0.03`, others have 0.0** — accommodates sigmoid output drift on newly-added NEAT output nodes. Subtle comparability confound (NEAT has a dead band that GA/ES do not) accepted as the cost of letting NEAT genomes express "don't turn" without perfect 0.5 outputs.
- **Plans treated as living design docs, not static spec** — 178 KB across 9 plan files, each with "Current Implemented / In Progress / Planned / Discarded" sections; explicitly traded the cost of drift (plans now lie about `POPULATION_SIZE` and `SEEDS_PER_AGENT`) against the value of preserved design rationale.
- **Genetic Programming kept in README, never implemented** — a documentation-code discrepancy that survived into the final state. Commit `1c55da4` "Mark roadmap items complete in README" checked GP roadmap items off without implementing them. LifeOS flags this as the anti-puffing headline.

## What is currently built

Four optimisation methods, end-to-end-wired with entry scripts, configs, and analytics output:

- **GA** — `train_ga.py` runs `HybridEncoder(47) → 24 → 24 → 3` MLP through 500 generations of `GADriver`, producing `training_data.json` (4.9 MB) and `training_summary.md` (60 KB).
- **CMA-ES** — `train_es.py` runs `TemporalStackEncoder(329)`-wrapped policy with diagonal CMA-ES and Pareto selection, producing `training_data_es.json` (394 KB) and `training_summary_es.md` (59 KB).
- **NEAT** — `train_neat.py` runs full topology-growing NEAT with speciation, producing `training_data_neat.json` (8.5 MB — largest file in repo), `training_summary_neat.md` (68 KB), and 50 generations × 2-file artefacts in `training/neat_artifacts/`.
- **GNN-SAC** — `train_gnn_sac.py` runs SAC with a `GATv2Conv` GNN backbone over 500K env steps, producing `training_data_sac.json` (992 KB), `training_summary_sac.md` (45 KB), and a 623 KB `best_sac.pt` checkpoint.

Shared substrate built and used by all four: `HeadlessAsteroidsGame`, `HybridEncoder`, `ComposableRewardCalculator` with a 5-component preset, `population_evaluator.py` (45 KB, used by 3 of 4 methods), and the 3-stage `TrainingAnalytics` pipeline producing both machine-readable JSON and human-readable markdown summaries per run.

Test coverage: 4 test files (~59 KB). `test_kill_asteroid_reward.py` and `test_json_export_numpy_types.py` are active; `test_ga_dimensions.py` is broken (references removed modules); `test_neat_xor.py` status is unresolved in LifeOS evidence (plan claims it doesn't exist, file is 16.5 KB on disk).

Not built: Genetic Programming (zero code despite README claims), parallel training dashboard (every roadmap checkbox unchecked), cross-method comparison report (per-method summaries exist; the comparison output that justifies the project's premise does not).

## Current state

**Dormant.** Last commit `1c55da4` (2026-02-22) was titled "Mark roadmap items complete in README" — a tidy-up commit. Real development ceased in January 2026 after GNN-SAC landed; the project has been dormant for ~2 months as of LifeOS's last verification (2026-04-24). The author's active projects are elsewhere.

The arc: 9-month gap after initial scaffolding (2025-03 → 2025-12), then a heavy push from 2025-12-28 through 2026-02-22 that landed GA (with two weeks of iteration), ES, NEAT, and GNN-SAC (the last three across 9 days in January with little per-method iteration). LifeOS's framing: "the signature of a student benchmarking project hitting a submission deadline, not a mature research platform".

No in-flight work captured in LifeOS Work/ — the folder does not exist for this project; direction comes from Roadmap.md as synthesised "what would make sense if work resumed".

## Gaps and known limitations

- **Five paradigms claimed, four shipped.** README lists Genetic Programming with 5 sub-features all marked `[x]` on the roadmap; zero GP code exists. `search_content.py` for `genetic_programming|GeneticProgram|symbolic|deap` returns no hits in any Python file. The "anti-puffing headline".
- **Parallel training dashboard described but not built** — README L107-123 describes simultaneous training of all algorithms in separate game instances with an interactive sidebar; every roadmap item for it is `[ ]` unchecked.
- **Cross-method comparability is structurally compromised.** Eight independent confounds: parameter count (GA 1,227 vs ES 7,995, 6.5×), compute budget per generation (GA 300 vs NEAT 250 vs SAC step-based), CRN default (varies per method), fitness penalty (NEAT subtracts 1.0×std_dev, GA/ES don't), turn deadzone (NEAT 0.03 vs others 0.0), selection objective (scalar vs Pareto vs mixed), novelty/diversity integration scaling, temporal input (ES only). The analytics pipeline is method-agnostic; the experimental design is not.
- **Plan vs code drift.** `plans/GENETIC_ALGORITHM.md` states `POPULATION_SIZE=10, SEEDS_PER_AGENT=5`; code has `15` and `20` (4× discrepancy on compute per generation). `plans/NEAT.md` claims no XOR sanity test exists; `tests/test_neat_xor.py` is 16.5 KB on disk.
- **Dead TensorFlow stack** — 47+ KB of TF code (`feedforward_tf.py`, `nn_agent_tf.py`, `population_evaluator_tf.py`) referenced by zero entry scripts. The TF evaluator is almost as large as the NumPy one (37.7 KB vs 45 KB).
- **Dead ES code** — classic ES `driver.py` (17.7 KB) superseded by CMA-ES driver; `fitness_shaping.py` (3 KB) present but unused by CMA-ES; AdamW / rank-transformation / elitism settings in `ESConfig` are not read.
- **Missing dependency manifests** — no `requirements.txt`, `pyproject.toml`, or `Pipfile`. Particularly painful for GNN-SAC's `torch` + `torch_geometric` + `torch_scatter` + `torch_sparse` stack. Python version drifts between 3.12 and 3.14 in `__pycache__`.
- **15 MB of generated training artefacts checked into git** (`training_data_*.json`, `training_summary_*.md`, `training/neat_artifacts/`, `best_sac.pt`).
- **Low test coverage** — ~2% (2-3 working test files / 141 source files). The shared 45 KB evaluator that underpins the comparability claim has no tests.
- **No checkpoint/resume for 3 of 4 methods** — only NEAT writes per-generation artefacts. GA, ES, SAC lose internal state (and SAC its replay buffer + optimizer) on process exit.
- **Broken / unused helpers** — `EnvironmentTracker.get_tick()` references nonexistent `game.time`; `VectorEncoder` unused; `linear.py` policy unused; `GAConfig.NUM_NEAREST_ASTEROIDS=8` lingers from the `VectorEncoder` era.

## Direction (in-flight, not wishlist)

No in-flight work — project is dormant. LifeOS Roadmap.md is explicitly framed as "what *would* make sense if work resumed, not a commitment". If the project were resumed, the priority-ranked sessions LifeOS proposes are:

1. **Honest README pass** — remove `[x]` marks on GP roadmap items, update dashboard section to "designed not built", reconcile plans' `POPULATION_SIZE`/`SEEDS_PER_AGENT` with code.
2. **Reproducibility basics** — add `requirements-rl.txt` (pinning torch + torch_geometric + PyG backend deps), pin Python version, add a NumPy-only `requirements.txt`.
3. **Artefact hygiene** — gitignore the 15 MB of generated outputs.
4. **Fix or delete broken tests** — `test_ga_dimensions.py` and confirm `test_neat_xor.py` status.

The actual research payoff would be matched-compute reruns + a published cross-method comparison report; without that, the project is "four tutorials in a trench coat" (LifeOS framing).

## Demonstrated skills

- **Comparative ML platform design** — built a shared substrate (game, encoder family, reward calculator, evaluator, analytics) that lets four optimisation paradigms be compared on the same task, with explicit design invariants preserving comparability.
- **Implementation of four optimisation paradigms from scratch** — Genetic Algorithm (BLX-α + Gaussian mutation + tournament), diagonal CMA-ES (with target-rate adaptation, antithetic sampling, Pareto rank selection), NEAT (full topology growth + speciation + innovation tracking), and Soft Actor-Critic with a GNN backbone — all without depending on a high-level ML framework's pre-built optimisers.
- **Graph Neural Network policies for RL** — used `torch_geometric`'s `GATv2Conv` to consume variable-size graph observations in a SAC actor-critic, including a graph-native replay buffer storing `(graph_state, action, reward, next_graph, done)` tuples.
- **PyTorch SAC implementation** — twin critics, entropy regularisation, continuous control, eval-on-fixed-seeds protocol, checkpoint serialisation of GNN + actor weights + eval metadata.
- **State-encoder design** — designed a 47-dim hybrid fovea+raycast encoder that defeats spin-lock turret exploits, including toroidal-aware ray casting via 8-cell ghost-target duplication; understood why a pure nearest-N encoder failed and chose redundancy as the fix.
- **Multi-objective optimisation in practice** — Pareto-front selection in CMA-ES on `[hits, time_alive, softmin_ttc]`; mixed scalar+novelty+diversity scoring in GA; deliberate per-method differentiation of selection criteria.
- **Variance-reduction techniques** — CRN (common random numbers) + antithetic sampling for ES gradient estimation, with deliberate per-method CRN defaults reflecting each algorithm's noise tolerance.
- **Parallel-rollout infrastructure** — `ThreadPoolExecutor` over `HeadlessAsteroidsGame` instances; `StateEncoder.clone()` discipline to avoid shared mutable state across threads (temporal buffer + ray history would corrupt otherwise).
- **Reward shaping at scale** — 22-component `ComposableRewardCalculator` with a shared preset across methods, plus analytics that record per-component contribution per generation.
- **Analytics-pipeline design** — three-stage collection/analysis/reporting producing both machine-readable JSON (200+ fields/generation) and human-readable 45-68 KB markdown summaries per run.
- **Architectural discipline under multi-paradigm pressure** — top-down-only dependency direction across 5 layers; non-breaking invariant codified between SAC's step-based loop and the evolutionary methods' generation-based loop; identified the shared evaluator as the architectural centre of gravity and treated it as such.
- **Honest gap documentation** — extensive LifeOS-side anti-puffing record (claimed-vs-implemented, cross-method comparability confounds, plan-vs-code drift) demonstrates engineering maturity in distinguishing aspiration from reality.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/AsteroidsAI/_Overview.md | 130 | "#project/asteroids-ai #lang/python #paradigm/evolutionary #paradigm/neat #paradigm/cma-es #paradigm/sac #ml/rl #ml/gnn #status/dormant" |
| Projects/AsteroidsAI/Architecture.md | 290 | "- [[AsteroidsAI/Roadmap]] — the planned parallel dashboard and its architectural implications" |
| Projects/AsteroidsAI/Decisions.md | 278 | "- [[AsteroidsAI/Roadmap]] — decisions deliberately deferred appear as roadmap items" |
| Projects/AsteroidsAI/Gaps.md | 257 | "- [[AsteroidsAI/Systems/Analytics Pipeline]] — analytics polish gaps" |
| Projects/AsteroidsAI/Roadmap.md | 282 | "- [[Projects/_Overview]] — AsteroidsAI relative to Caner's other active projects (all of which are higher priority as of 2026-04-24)" |
| Projects/AsteroidsAI/Systems/_Overview.md | 43 | "- [[Projects/AsteroidsAI/Roadmap]] — direction-of-travel" |
| Projects/AsteroidsAI/Systems/Analytics Pipeline.md | 257 | "- [[AsteroidsAI/Roadmap]] — analytics polish is much of the remaining in-repo roadmap" |
| Projects/AsteroidsAI/Systems/Evolution Strategies.md | 291 | "- [[AsteroidsAI/Roadmap]] — the Easy/Medium/Hard roadmap from the ES plan is the richest source of next-session ideas" |
| Projects/AsteroidsAI/Systems/Game Engine.md | 193 | "- [[AsteroidsAI/Gaps]] — broken `get_tick()`, wrap-aware collision, unused arcade APIs" |
| Projects/AsteroidsAI/Systems/Genetic Algorithm.md | 225 | "- [[Vynapse/_Overview]] — Caner's Rust neuroevolution engine; solves similar fixed-topology evolutionary problem in a different language" |
| Projects/AsteroidsAI/Systems/GNN-SAC.md | 355 | "- [[NeuroDrive/_Overview]] — NeuroDrive's asymmetric PPO (actor 2x64, critic 2x128) is the sibling gradient-based RL in the vault; comparing SAC continuous control here with PPO continuous control there is a useful cross-project analogy" |
| Projects/AsteroidsAI/Systems/NEAT.md | 226 | "- [[Vynapse/_Overview]] — Vynapse's `trainers/neat.rs` is a 0-byte stub; AsteroidsAI's NEAT is the working reference implementation" |
| Projects/AsteroidsAI/Systems/Reward System.md | 174 | "- [[AsteroidsAI/Gaps]] — 17 components never exercised in a run" |
| Projects/AsteroidsAI/Systems/Shared Components.md | 225 | "- [[AsteroidsAI/Gaps]] — method-parity normalisation not done; cross-method bonus magnitudes not comparable" |
| Projects/AsteroidsAI/Systems/State Encoders.md | 219 | "- [[AsteroidsAI/Gaps]] — encoder drift, schema versioning, VectorEncoder dead code" |
