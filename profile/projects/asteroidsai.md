---
name: AsteroidsAI
status: dormant
source_repo: https://github.com/Capataina/Asteroids-AI
lifeos_folder: Projects/AsteroidsAI
last_synced: 2026-04-26
sources_read: 14
---

# AsteroidsAI

## One-line summary

Comparative-ML research playground in Python implementing four optimisation paradigms (custom GA, diagonal CMA-ES, NEAT, GNN-based SAC) against a single Asteroids continuous-control environment with a shared evaluator, reward preset, and analytics pipeline.

## What it is

AsteroidsAI is a Python research playground that pits four different optimisation paradigms against the same Asteroids-style continuous-control environment, under the same state encoders, reward presets, and analytics pipeline. The explicit goal in LifeOS Overview.md is comparative — "single environment, multiple minds" — so methods can be compared on emergent behaviour, learning dynamics, representation choices, and qualitative decision-making rather than on disjoint setups. The implemented surface differs from the README pitch: the README claims five paradigm families plus a parallel training dashboard, but LifeOS records that **four** methods are wired (NEAT, GA, CMA-ES, GNN-SAC), Genetic Programming has zero code despite being checked off in the README, and the dashboard has every roadmap checkbox unchecked. The most durable engineering output is not the comparison itself but the shared substrate — the `population_evaluator.py` shared by GA/ES/NEAT plus the `TrainingAnalytics` facade — which any future evolutionary method can plug into via an `agent_factory` and inherit 200+ analytics fields. Real development ran from late December 2025 through January 2026 and ceased in February 2026; the project has been dormant for ~2 months as of the LifeOS verification (2026-04-24).

## Architecture

The repo is organised as a top-down dependency cake — training scripts orchestrate; methods, core, and analytics sit underneath; agents implement a single `BaseAgent` contract; interfaces hold encoders/action/reward contracts; the game sits at the bottom and knows nothing about anything above it.

```
training/scripts/         train_ga.py | train_es.py | train_neat.py | train_gnn_sac.py
                          view_gnn_sac.py | simulate_gnn_sac.py
                          (orchestrate evaluator + driver + display + analytics)
              │
   ┌──────────┼─────────────────┐
   ▼          ▼                 ▼
training/   training/methods/  training/analytics/
core/         genetic_algorithm/   collection/
  population_   evolution_         analysis/
  evaluator     strategies/        reporting/
  episode_      neat/              (TrainingAnalytics facade)
  runner        sac/
  display_
  manager
              │
              ▼
ai_agents/    BaseAgent (get_action + reset)
              NNAgent (NumPy)  — used by GA + ES
              NEATAgent        — wraps a compiled NEAT feedforward network
              SACAgent         — PyTorch inference wrapper
              NNAgentTF / policies/feedforward_tf — present, unused
              │
              ▼
interfaces/   StateEncoder abstract
                HybridEncoder (47-dim)        — GA + ES + NEAT base
                TemporalStackEncoder (329-dim) — wraps HybridEncoder for ES
                GraphEncoder                  — used by GNN-SAC
                VectorEncoder                 — present, unused
              ActionInterface (boolean | continuous)
              EnvironmentTracker (spatial queries, wrapped distances)
              MetricsTracker (shots/hits/kills/time_alive)
              RewardCalculator + ComposableRewardCalculator + 22 reward components
              │
              ▼
game/         AsteroidsGame (arcade.Window — rendering + manual play)
              HeadlessAsteroidsGame (seeded parallel rollouts)
              Player / Bullet / Asteroid
              globals.py (physics + collision constants)
              debug/visuals.py (overlays + HybridEncoder ray debug)
```

`population_evaluator.py` (45 KB) is the architectural centre of gravity — it is shared by GA, ES, and NEAT, and every method injects a method-specific `agent_factory` (param vector → `NNAgent` for GA/ES; genome → `NEATAgent` for NEAT). The evaluator wraps `ThreadPoolExecutor(max_workers=os.cpu_count())` to run `population × seeds_per_agent` rollouts in parallel against `HeadlessAsteroidsGame` instances. SAC has its own step-based collector loop because its timebase (env steps + replay) does not fit the (generation, candidate, seed, step) shape of the evolutionary methods.

The runtime per-step control loop is identical across methods:
`Game state → EnvironmentTracker → StateEncoder.encode → BaseAgent.get_action → ActionInterface.to_game_input → Game.on_update → MetricsTracker + RewardCalculator`. Boolean control thresholds at 0.5 (GA/ES/NEAT); continuous control feeds analog `turn_magnitude` and `thrust_magnitude` plus a still-discrete shoot bit (SAC).

A parallel TensorFlow stack (`feedforward_tf.py`, `nn_agent_tf.py`, `population_evaluator_tf.py` — 37.7 KB on its own) is present but unused by any entry script, kept as optionality for future GPU work that has not arrived.

## Subsystems and components

### Game engine (`game/` + `Asteroids.py`)

Two execution paths sharing the same physics: a windowed `AsteroidsGame(arcade.Window)` for humans and best-agent playback (~341 lines / 13 KB), and a `HeadlessAsteroidsGame` (10.7 KB) for thread-pooled parallel rollouts. Both read constants from `game/globals.py` — this is the parity anchor. Entities are `Player`, `Bullet`, `Asteroid` (three-tier scale hierarchy with HP scaling and fragmentation). The world is **toroidal** — positions wrap, distances use wrapped deltas via `EnvironmentTracker.get_distance`, and `HybridEncoder` rays use "ghost targets" that virtually duplicate asteroids into 8 surrounding grid cells so rays can intersect across screen edges without wrapping the ray itself. Collision detection uses explicit radii from `globals.py` (not arcade sprite-texture geometry) because sprite textures may not load in headless mode. A documented production-learned bug: `Bullet.update()` calls `remove_from_sprite_lists()` which is a no-op against the plain Python lists used in headless mode, so headless requires explicit per-step `lifetime > 0` filtering of `bullet_list` and `asteroid_list` to prevent debris accumulation. Each headless rollout owns its own `random.Random(seed)` so concurrent rollouts in a `ThreadPoolExecutor` cannot contaminate each other's randomness.

### State encoders (`interfaces/encoders/`)

Four encoders implementing a shared contract (`encode`, `get_state_size`, `reset`, `clone`). `clone()` is mandatory because parallel rollouts need per-thread instances to avoid corrupting shared temporal buffers. **`HybridEncoder` (47-dim)** is the workhorse for GA/ES/NEAT: 3 proprioception (forward/lateral velocity, shoot cooldown), 3 fovea asteroids × 4 features (wrapped distance, signed bearing, closing speed, scale), 16 peripheral rays × 2 features (normalised hit distance, normalised closing speed). The fovea+rays redundancy was a deliberate fix for the spin-lock pathology that pure nearest-N (`VectorEncoder`) produced — agents learning to sit still and rotate. **`TemporalStackEncoder`** wraps any base encoder, producing `[s(t), s(t-1), s(t-2), s(t-3), Δ(t), Δ(t-1), Δ(t-2)]` for stack_size=4 with deltas, yielding 329 dims over HybridEncoder; ES uses this because its gradient estimation needs smoother fitness landscape. **`GraphEncoder`** emits a `GraphPayload` (5-dim player features, N×3 asteroid features, N×7 directed asteroid→player edge attributes including wrapped dx/dy, distance, bearing sin/cos, relative velocity); used by GNN-SAC, supports variable cardinality so the network sees exactly the asteroids that exist. **`VectorEncoder`** (35 dims) is the dead baseline — superseded by HybridEncoder; still imported by nothing in `training/scripts/` though the evaluator type-hints its parameter as `VectorEncoder` (harmless duck-typing).

### Reward system (`interfaces/rewards/`)

22 reward components in 47 KB across the largest interface folder. The `ComposableRewardCalculator` holds a list of `RewardComponent` instances; per-step, every component contributes a reward; per-episode, terminal rewards are added; per-component breakdown is tracked for analytics. The components include `DistanceBasedKillReward`, `ProximityFacingBonus`, `TargetLockReward`, `LeadingTargetBonus`, `ExplorationBonus`, `MovingTowardDangerBonus`, `DeathPenalty`, `VelocityKillBonus`, `ProximityPenalty`, `VelocitySurvivalBonus`, `FacingAsteroidBonus`, `MaintainingMomentumBonus`, `ConservingAmmoBonus`, `SpacingFromWallsBonus`, `ChunkBonus`, `KPMBonus`, `AccuracyBonus`, `ShootingPenalty`, `NearMiss`, `KillAsteroid`, `SurvivalBonus`. All four methods use the same fixed preset of 5 components — `VelocitySurvivalBonus(1.5, cap=15)`, `DistanceBasedKillReward(max=18, min_frac=0.15)`, `ConservingAmmoBonus(hit=4, shot=-2)`, `ExplorationBonus(3×4 grid, 5/cell)`, `DeathPenalty(-150, max_time_alive=max_steps × frame_delay)` — with the remaining 17 components scaffolded but never exercised in any stored training run. The shared preset is a deliberate comparability anchor: diverging the rewards per method would let every method "win its own game". The reward breakdown feeds into Shannon-entropy diversity scoring (used by GA and NEAT selection) and into analytics (`reward_component_shares`, `reward_dominance_index`, `reward_max_share`, `reward_entropy`).

### Genetic Algorithm (`training/methods/genetic_algorithm/`)

The first method implemented (Jan 2026) and the pilot that proved the shared evaluator/analytics/display substrate. Genome is a flat `List[float]` of length 1,227 — the parameter vector for a single-hidden-layer NumPy MLP (`FeedforwardPolicy`: `tanh` hidden of 24 units, sigmoid outputs clamped for stability). `GADriver` (8.9 KB) runs per generation: stagnation-adaptive Gaussian mutation (σ=0.1, prob=0.05, σ adapts upward at stagnation ≥10), BLX-α crossover (α=0.5 — children can overshoot the parent range by 50%), tournament selection (size 3) on a combined score of `fitness + novelty_bonus + diversity_bonus`, elitism preserving top ~10% (min 2), best-ever individual injected back when stagnation < 5. Configured `POPULATION_SIZE=15`, `SEEDS_PER_AGENT=20`, `NUM_GENERATIONS=500`, `USE_COMMON_SEEDS=False` (independent seeds — GA's tournament is noise-tolerant). After-action artefacts live in memory only — there is no genome persistence on disk. plans/GENETIC_ALGORITHM.md explicitly flags DEAP migration as wanted but deferred ("Custom implementation is functional and stable. Migration to DEAP is planned but not urgent").

### Evolution Strategies — diagonal CMA-ES (`training/methods/evolution_strategies/`)

Diagonal CMA-ES (mean, sigma, per-parameter diagonal covariance) — full-covariance LM-CMA was considered and rejected for O(n²) memory / O(n³) update cost on 7,995-param policies. Two drivers exist: `cmaes_driver.py` (10.8 KB, used) and `driver.py` (17.7 KB classic ES, present-but-unused), plus `fitness_shaping.py` (3 KB rank transformation, also unused by the CMA-ES driver). Configured `POPULATION_SIZE=100`, `SEEDS_PER_AGENT=3`, `CMAES_SIGMA=0.15` initial, `CMAES_MU=None` (defaults to pop//2 = 50 parents), `CMAES_COV_TARGET_RATE=1e-3` with `CMAES_COV_MAX_SCALE=1e4` for adaptive learning-rate scaling, `SIGMA_MIN=0.02` floor. Antithetic sampling (`+ε`/`−ε` paired) plus CRN (`USE_COMMON_SEEDS=True`) is mandatory — without CRN, paired antithetic samples would see different rollouts and the variance reduction collapses. Restarts on stagnation enabled (`RESTART_PATIENCE=12`, restart from best candidate not random). Selection is **Pareto-first**: candidates are ranked by Pareto fronts + crowding distance over `[hits, time_alive, softmin_ttc]`, not by scalar fitness. Wraps `HybridEncoder` in `TemporalStackEncoder(N=4)` → 329-dim input → 7,995 MLP parameters (6.5× GA's 1,227 — this is a deliberate noise-handling choice but creates a comparability confound). Stores the actual best Pareto-ranked candidate, not the distribution mean (the mean was never directly evaluated). Noise handling re-evaluates the top-5 candidates with extra seeds at offset 100,000 to confirm Pareto rank.

### NEAT (`training/methods/neat/` + `ai_agents/neuroevolution/neat/`)

NeuroEvolution of Augmenting Topologies — the only method that evolves both connection weights **and** network structure, and the only one that writes per-generation genome artefacts to disk. Genome (`genome.py`, 12.9 KB) stores node genes (input/hidden/output) and connection genes (from, to, weight, enabled, innovation_id). `InnovationTracker` (1.5 KB) assigns globally-unique innovation IDs so crossover aligns meaningfully across genomes — the standard NEAT machinery. Genomes compile to feedforward DAGs (`network.py`, 2.8 KB; topologically sorted forward pass — feedforward only, no recurrence). Configured `POPULATION_SIZE=50`, `SEEDS_PER_AGENT=5`, `USE_COMMON_SEEDS=True`, `COMPATIBILITY_THRESHOLD=0.25` initial with `ADAPT_COMPATIBILITY_THRESHOLD=True` and `TARGET_SPECIES=8` (driver bumps the threshold ±0.02 to hit the target species count, clamped [0.05, 3.0]), `SPECIES_STAGNATION=7` (reduced from 15 to fail fast on dead-end species), `ELITISM_PER_SPECIES=1`, `WEIGHT_MUTATION_PROB=0.1` with `σ=0.5` (5× GA's), `ADD_NODE_PROB=0.03`, `ADD_CONNECTION_PROB=0.05`, `MAX_NODES=None` and `MAX_CONNECTIONS=None` (no hard complexity cap — bets on fitness pressure + sharing). `FITNESS_STD_PENALTY_RATIO=1.0` subtracts one standard deviation of per-seed fitness from the reported value — a strong anti-overfitting-to-lucky-seeds signal that no other method applies. `ActionInterface(turn_deadzone=0.03)` (NEAT-specific) gives genomes a dead band so a sigmoid drifting near 0.5 doesn't produce micro-turning policies. Speciation uses compatibility distance `d = C1·E/N + C2·D/N + C3·meanΔw` (excess/disjoint/weight diff). Adjusted fitness = fitness / species_size protects small novel species. Writes `gen_XXXX_best.json` + `.dot` per generation (50 generations × 2 files in the committed artefacts) plus `best_overall.json` + `.dot`; `.dot` files open directly in Graphviz for visual topology inspection. Pareto-by-display-only — reproduction is still scalar-fitness-driven.

### GNN-SAC (`training/methods/sac/`)

Soft Actor-Critic with a Graph Neural Network backbone — the only PyTorch method, the only one with true continuous actuation, and the last method added (Jan 2026). `GNNBackbone` is a 2-layer `torch_geometric.nn.GATv2Conv` stack (64 hidden, 4 attention heads, no dropout). The graph embedding feeds an `Actor` MLP (256 hidden) producing `(turn_mean, turn_logstd) (thrust_mean, thrust_logstd) (shoot_mean, shoot_logstd)` and `TwinCritics` (Q1, Q2, 256 hidden each, target copies via Polyak averaging at `TAU=0.005`). `learner.py` (15.4 KB) handles critic/actor/entropy updates with Huber critic loss (`HUBER_DELTA=1.0`, more outlier-robust than MSE for variable-size graph batches), automatic entropy temperature tuning (`AUTO_ENTROPY=True`, `TARGET_ENTROPY=-3.0` ≈ -dim(action)), gradient clipping (`GRAD_CLIP_NORM=10.0`) plus adaptive per-parameter clipping (`AGC_ENABLED=True`, `AGC_CLIP_FACTOR=0.01`). Asymmetric learning rates: `ACTOR_LR=3e-4`, `CRITIC_LR=1e-4` (3× slower — deliberate stability choice for the GNN's noisy early embeddings). `replay_buffer.py` (4.9 KB) is graph-native — stores variable-size graph transitions and collates batches. `normalization.py` (4.6 KB) maintains running mean/std for graph features. Configured `TOTAL_STEPS=500_000`, `BATCH_SIZE=256`, `REPLAY_SIZE=100_000`, `LEARN_START_STEPS=5_000`, `UPDATES_PER_STEP=1`, `REWARD_SCALE=0.2`, `EVAL_EVERY_EPISODES=5` on fixed seeds `[1001..1005]` (intentionally outside training seed range), `BEST_CHECKPOINT_PATH=training/sac_checkpoints/best_sac.pt` (623 KB containing GNN backbone + actor weights + eval metadata). Three entry scripts: `train_gnn_sac.py` (37.3 KB, pure headless), `view_gnn_sac.py` (5.1 KB, windowed playback of best checkpoint), `simulate_gnn_sac.py` (36.4 KB, headless training + windowed playback in one process at `TRAIN_STEPS_PER_FRAME=2`). Documented failure-mode taxonomy (exploration collapse, critic miscalibration, reward-scale instability, GNN oversmoothing) drives a ~50-key SAC analytics block.

### Shared components — novelty, diversity, Pareto (`training/components/`)

Method-agnostic selection-shaping utilities. **Behaviour vector** (`novelty.py`, 5.5 KB) produces a 7D characterisation per agent (thrust rate, turn rate, shoot rate, accuracy, idle rate, engagement distance, screen coverage) — deliberately reward-agnostic so novelty pushes agents apart in behaviour space even when rewards haven't yet differentiated them. Novelty score is mean Euclidean distance to k nearest neighbours across population + `BehaviorArchive` (3.7 KB; bounded archive with random replacement when full). **Reward diversity** (`diversity.py`, 4.3 KB) computes Shannon entropy over **positive** reward components only (penalties excluded) — a policy that is high on `VelocitySurvivalBonus` and low on `DeathPenalty` should not score "diverse" because it is just dying badly. **Combined selection score** (`selection.py`, 3.9 KB): `score = fitness + behavior_novelty_weight × novelty × novelty_fitness_scale + diversity_weight × reward_diversity × max(1, |fitness|)`. **Pareto** (`pareto/objectives.py`, `ranking.py`, `utility.py`) implements NSGA-II-style Pareto fronts + crowding distance over `[hits, time_alive, softmin_ttc]` (`softmin_ttc` is exponential-weighted soft-min so the closest threat dominates without hard-min discontinuities; `ACCURACY_MIN_SHOTS=5` zeroes accuracy below 5 shots to prevent single-shot-perfect-accuracy gaming). Per-method usage matrix: GA uses novelty + diversity in tournament score; ES uses both pre-rank-shaping and Pareto for selection; NEAT uses both as selection bonus and Pareto for display only; SAC logs metrics only.

### Analytics pipeline (`training/analytics/`)

The comparability substrate. `TrainingAnalytics` facade (9.7 KB) with five entry points (`set_config`, `record_generation`, `record_distributions`, `record_fresh_game`, `save_json`, `generate_markdown_report`) writes a versioned JSON export (`SCHEMA_VERSION="2.3"`) and a markdown report with identical schema across methods. `generations_data[i]` is a dict with **30+ always-on keys plus up to 100+ method-specific keys** — fitness moments and percentiles, timing, NEAT-specific (13 keys: species count + size distribution + pruning, topology size, compatibility telemetry, innovation survival), ES-specific (10 keys: sigma, mean param norm, covariance stats + scaling, Pareto front0 size + best crowding), SAC-specific (~50 keys: timebase + eval returns + held-out eval + learner stability + action health + replay/data health + representation health including embedding cosine similarity as a collapse proxy + per-network weight snapshots), and ~40 evolutionary behavioural keys covering combat, survival, action rates, turn dynamics, aim alignment, shooting cadence, engagement, risk, robustness (`fitness_std`), neural health, reward anatomy, and heatmap inputs. `distributions_data` stores 23 sorted per-agent value lists per generation enabling percentile/spread/mean±std charts. `fresh_game_data` stores playback results and generalisation ratios (`fitness_ratio`, `kills_ratio`, `accuracy_delta`, `generalization_grade`) — the hedge against seed memorisation. Reporting layer (`markdown.py` 11.2 KB, `json_export.py` 4.1 KB, `insights.py` 4.7 KB, `glossary.py` 10.7 KB) produces 20-column ASCII sparklines, per-section takeaways and warnings, training-phase analysis, milestone detection, reward-evolution analysis, distribution analysis. The 4 stored `training_summary_*.md` reports are 45-68 KB each.

## Technologies and concepts demonstrated

### Languages
- **Python (3.12 / 3.14)** — sole language; LifeOS records 141 `.py` files / ~806 KB of source. `__pycache__` shows both `cpython-312` and `cpython-314` compiled bytecode (a manifest would nail this down). Used for everything: physics, numerics, RL, analytics, reporting.

### Frameworks and libraries
- **NumPy** — the substrate for evolutionary policies (`FeedforwardPolicy` MLP, GA genome operations, CMA-ES sampling/update arithmetic, all per-step rollout numerics). The whole evolutionary stack is NumPy-only with `ThreadPoolExecutor` for parallel CPU rollouts.
- **PyTorch** — used exclusively by GNN-SAC for the actor, twin critics, target networks (Polyak averaging), automatic entropy temperature tuning, Huber critic loss, gradient clipping, and adaptive per-parameter clipping (AGC).
- **PyTorch Geometric (`torch_geometric`)** — provides `GATv2Conv` graph attention layers used in the SAC GNN backbone (2 layers, 64 hidden, 4 heads). Backend deps `torch_scatter` and `torch_sparse` are required (and not pinned anywhere, per LifeOS Gaps).
- **Arcade** — windowed game rendering for `AsteroidsGame(arcade.Window)`; manual play, best-agent playback, debug overlays. Headless mode bypasses Arcade and uses plain Python lists.
- **TensorFlow stack — present but unused.** `feedforward_tf.py` (5.5 KB), `nn_agent_tf.py` (2.6 KB), `population_evaluator_tf.py` (37.7 KB) ship as dead code preserved for potential future GPU-batched ES; LifeOS Decisions explicitly notes this was deliberate optionality, not legacy decay.

### Runtimes / engines / platforms
- **`HeadlessAsteroidsGame`** — custom seeded headless physics simulator built to mirror the Arcade windowed game, used as the parallel rollout engine. The "two modes, one physics" parity is the hardest engineering constraint in the codebase.
- **`ThreadPoolExecutor(max_workers=os.cpu_count())`** — concurrency primitive for parallel evaluation; each rollout owns its own `random.Random(seed)` to prevent cross-thread RNG contamination.
- **Graphviz `.dot` files** — NEAT writes per-generation `.dot` files visualising evolved network topologies; `dot -Tpng` produces images of the architectures the optimiser built. Only method in the repo where the architecture is visually inspectable.

### Tools
- **`scan_repo.py` / `fetch_commits.py` / `repo_stats.py` / `search_content.py`** — custom evidence-collection scripts the LifeOS notes use to verify project state against code rather than against the README.
- **`plans/` directory** — 9 living-design-doc files totalling 178 KB (22% docs-to-source ratio), each with explicit "Current Implemented", "In Progress", "Planned", "Notes", "Discarded" sections. Treated as design memory rather than reference docs.

### Domains and concepts
- **Neuroevolution (fixed-topology)** — implements full GA driver from scratch: BLX-α blend crossover (parents-bracket-extended, α=0.5), Gaussian per-gene mutation with stagnation-adaptive σ, tournament selection (size 3) on a combined fitness + novelty + diversity score, elitism, all-time-best injection.
- **Neuroevolution (topology-augmenting)** — full NEAT from scratch: innovation tracking with globally-unique connection IDs, structural mutations (add-node, add-connection with cycle checks), crossover via gene alignment by innovation id with excess/disjoint handling, speciation with compatibility distance and adaptive thresholding, fitness sharing, per-species elitism, species stagnation pruning.
- **Diagonal CMA-ES** — full diagonal Covariance Matrix Adaptation Evolution Strategy from scratch: rank-mu update, diagonal covariance with target-rate adaptation (custom — not standard CMA-ES), sigma adaptation via cumulative path length, antithetic sampling paired with Common Random Numbers (CRN) for variance-reduced gradient estimation, restart on stagnation from best candidate (not random).
- **Multi-objective selection (NSGA-II-style Pareto)** — Pareto fronts + crowding distance over `[hits, time_alive, softmin_ttc]` with soft-min TTC (exponential-weighted aggregation) for continuous spatial-awareness signal.
- **Off-policy reinforcement learning (Soft Actor-Critic)** — full SAC from scratch: twin critics with target Polyak averaging, entropy-regularised stochastic policy with automatic temperature tuning, asymmetric actor/critic learning rates, Huber critic loss, gradient clipping (global + adaptive per-parameter), graph-native replay buffer.
- **Graph Neural Networks** — `GATv2Conv` graph attention message-passing for variable-cardinality state encoding. Bipartite asteroid → player edges only (no asteroid ↔ asteroid) to keep message passing O(N). Player has no position; every asteroid is described relative to the player via wrapped edge attributes — natural fit for a toroidal world.
- **Novelty search and quality-diversity (partial)** — 7D action-space behaviour characterisation with kNN novelty scoring, bounded archive with novelty-threshold admission, Shannon entropy over positive reward components for "uses multiple sources" diversity.
- **Common Random Numbers (CRN) / antithetic sampling** — variance reduction discipline for ES (paired `+ε`/`−ε` samples seeing identical seed sequences); rationale and per-method tuning (`USE_COMMON_SEEDS=True` for ES/NEAT, `False` for GA) explicitly captured.
- **Toroidal world geometry** — wrapped-distance utility, ghost-target ray casting (asteroids virtually duplicated into 8 surrounding grid cells so rays can intersect across edges), wrapped edge attributes in graph encoder.
- **Comparative-ML methodology** — explicit shared substrate (evaluator + reward preset + analytics schema) so that cross-method differences are attributable to the optimisation paradigm; LifeOS Gaps honestly documents seven concrete confounds (parameter count, compute budget, CRN default, fitness penalty, turn deadzone, selection objective, novelty/diversity scaling, temporal input) that compromise the claim despite the substrate.
- **Continuous control (analog actuation)** — `continuous_control_mode` path in the game accepting `turn_magnitude ∈ [-1,1]` and `thrust_magnitude ∈ [0,1]` (shoot remains discrete because bullets have a discrete cooldown).
- **Versioned analytics schema** — `SCHEMA_VERSION="2.3"` on every JSON export so downstream consumers can tell shapes apart.
- **Diagnostic taxonomy for RL failures** — explicit failure-mode catalogue (exploration collapse, critic miscalibration, reward-scale instability, GNN oversmoothing) with primary fix knobs, motivating each diagnostic key in the analytics surface.

## Key technical decisions

**D1 — Four methods, one environment, one reward preset.** All four methods share the same environment, state encoder family, action interface, and reward preset (5 components from the 22-component library). Comparability is the project's core value proposition; diverging the reward preset would let every method "win its own game". Trade-off: a method that consistently plateaus because it cannot exploit the shared reward shape is locked in.

**D2 — Python with dual NumPy + TensorFlow stacks (only NumPy wired in).** Evolutionary policy layer is NumPy (`FeedforwardPolicy`, `NNAgent`); a parallel TensorFlow stack ships as 37+ KB of unused code preserved as optionality for future GPU-batched ES. The right answer is probably to delete it; LifeOS Gaps treats this as a maintenance liability.

**D3 — HybridEncoder (fovea + rays), not pure nearest-N.** The default 47-dim encoder is intentionally redundant — fovea gives precise dynamics for the 3 closest threats; 16 peripheral rays give coarse global situational awareness. This combination kills the "spin-lock turret" pathology that pure nearest-N (`VectorEncoder`) produced, where agents learned to sit still and rotate.

**D4 — ES uses TemporalStackEncoder; GA, NEAT do not.** ES wraps `HybridEncoder(47)` in `TemporalStackEncoder(N=4)` → 329-dim → 7,995 MLP parameters. GA and NEAT use the 47-dim encoder directly (1,227 GA params). ES needs temporal awareness because gradient estimation requires smooth fitness differences between samples; GA's tournament is noise-tolerant and doesn't need it. **Trade-off:** ES optimises a 6.5× larger parameter vector than GA — the single largest comparability confound in the project.

**D5 — Pareto-first selection for ES, scalar fitness for GA.** ES selects by Pareto rank over `[hits, time_alive, softmin_ttc]`; GA selects by `fitness + novelty + diversity`; NEAT uses scalar fitness for reproduction but Pareto for display. ES's rank-based update naturally accommodates Pareto ordering and protects diverse skill profiles during the mean update; GA's tournament already mixes novelty/diversity so Pareto would be redundant.

**D6 — CRN on by default for ES and NEAT, off for GA.** ES needs CRN because antithetic sampling requires paired samples to see identical environments; NEAT uses CRN combined with a fitness-std penalty for noise robustness. GA stays on independent seeds because tournament selection is robust to per-seed noise and CRN would lose generalisation pressure.

**D7 — Diagonal CMA-ES, not full covariance.** Full-covariance CMA-ES has O(n²) memory and O(n³) update cost; for a 7,995-parameter policy on CPU, this is infeasible. LM-CMA was considered and deferred. Diagonal captures per-parameter learning rate without the n² cost; target-rate adaptation (`CMAES_COV_TARGET_RATE=1e-3`) keeps step size consistent.

**D8 — Single evaluator, dispatched to method-specific drivers.** `population_evaluator.py` (45 KB — the largest Python file) is shared by GA/ES/NEAT via injected `agent_factory`. Per-method evaluators were rejected because they would triple the code surface and create drift opportunities (different seed derivation, different metric collection). The shared evaluator is the cheapest way to guarantee cross-method comparability.

**D9 — SAC has its own training loop (not evaluator-based).** SAC's timebase (env_step + replay) does not fit the (generation, candidate, seed, step) shape of the evolutionary methods. Forcing SAC through `evaluate_population_parallel` was rejected as semantic mismatch. plans/GNN_SAC.md codifies the constraint: existing GA/ES/NEAT must continue to run unmodified.

**D10 — NEAT has `turn_deadzone=0.03`, others have 0.0.** NEAT genomes with newly-added output nodes have sigmoid outputs that drift around 0.5 but don't hit it exactly; a 0-deadzone interface maps any `!=0.5` to a turn, producing micro-turning policies. The 0.03 deadzone accommodates NEAT's structural plasticity. **Trade-off:** identical network outputs produce mechanically different policies across methods — a subtle comparability confound.

**D11 — Genetic Programming scaffolding, no implementation.** README lists GP as a planned method and roadmap items show `[x]` for GP features, but no GP code exists (`search_content.py` for `genetic_programming|GeneticProgram|symbolic|deap` returns no matches in any Python file). This is a documentation-code discrepancy that survived into the final commit (`1c55da4` "Mark roadmap items complete in README"). LifeOS treats this as the anti-puffing headline: pitch is "5 paradigms benchmarked", reality is 4.

**D12 — Plans treated as living design docs, not static spec.** 9 plan files / 178 KB, each containing "Current Implemented", "In Progress", "Planned", "Notes", "Discarded" sections. The plans capture *why* — hypotheses, failure modes, discarded alternatives — that a code-generated API ref would miss. **Risk:** plans drift from code (concrete cases: `POPULATION_SIZE` plan says 10 / code says 15; `SEEDS_PER_AGENT` plan says 5 / code says 20).

**Operational invariants** captured in LifeOS: `StateEncoder.clone()` is mandatory (parallel rollouts need per-thread instances), `update_internal_rewards=False` during training (windowed game has a legacy reward calculator that must be suppressed), collision detection uses explicit radii from `globals.py` (sprite-texture geometry doesn't load in headless), toroidal wrapping via `EnvironmentTracker.get_distance` (euclidean distance is wrong at edges), and SAC must never break the evolutionary scripts.

## What is currently built

Distinct from "What it is" (the design ambition) — this is the honest implemented scope.

- **Four methods fully wired end-to-end:** `train_ga.py`, `train_es.py`, `train_neat.py`, `train_gnn_sac.py` all run and produce stored training outputs. The repo ships four `training_summary_*.md` reports (45-68 KB each) and four `training_data_*.json` exports (394 KB to 8.5 MB) confirming that each method has actually been executed end-to-end at least once.
- **141 `.py` files / ~806 KB of Python source** (LifeOS Overview verified via `repo_stats.py`).
- **4 test files / ~59 KB**: `test_kill_asteroid_reward.py` (14.7 KB, active), `test_ga_dimensions.py` (6.7 KB, **broken** — references removed legacy modules), `test_json_export_numpy_types.py` (1.9 KB, active), `test_neat_xor.py` (16.5 KB, status unclear — plans/NEAT.md claims no XOR test exists, contradicting the file's presence). Effective coverage ~2% of source files.
- **9 plan files / 178 KB** of design memory in `plans/` covering each subsystem with planned/in-progress/discarded sections.
- **NEAT artefacts persisted to disk:** 50 generations × 2 files (.json + .dot) plus best_overall (~1.5 MB total). NEAT is the only evolutionary method that persists genomes — GA, ES, and SAC's optimiser/replay state are memory-only and cannot resume.
- **One SAC checkpoint:** `best_sac.pt` (623 KB) containing GNN backbone + actor weights + eval metadata. Sufficient for inference, not for resuming training.
- **15 MB of generated training artefacts checked into git** (`training_data_*.json`, `training_summary_*.md`, NEAT artefacts, SAC checkpoint) — flagged in LifeOS Gaps as a hygiene problem.
- **No build system / dependency manifest:** no `pyproject.toml`, no `requirements.txt`, no `requirements-rl.txt`, no Pipfile, no conda env file. PyTorch + PyTorch Geometric backend deps (which vary by CUDA vs CPU and Python version) are not pinned. Python version drifts (3.12 and 3.14 both present in `.pyc` cache).
- **Genetic Programming: zero code.** Listed in README and checked off in roadmap, not implemented anywhere.
- **Parallel training dashboard: zero code.** Described in README, every roadmap checkbox unchecked.
- **TensorFlow stack: 47+ KB of dead code** (`feedforward_tf.py`, `nn_agent_tf.py`, `population_evaluator_tf.py`) — present but referenced by zero entry scripts.
- **Dead ES code:** `driver.py` (17.7 KB classic ES, superseded), `fitness_shaping.py` (3 KB, unused by CMA-ES driver), plus AdamW/rank-transform/elitism settings still in `ESConfig` that the active driver doesn't read — ~21 KB of dead code in ES alone.

## Current state

**Status: dormant.** Last commit `1c55da4` (2026-02-22) titled "Mark roadmap items complete in README" is a tidy-up commit; the prior commit (`75a566f`, same day) is "ran the training again". Real development ceased in January 2026 after GNN-SAC landed. Project has been dormant for ~2 months as of LifeOS verification (2026-04-24). 42 total commits across 2025-02-25 → 2026-02-22, with concentrated activity Dec 28 → Jan 23 (reward system, GA, analytics, ES, NEAT, GNN-SAC each landing across ~9 days in January with very little per-method iteration afterwards — the cadence signature of a benchmarking project hitting a deadline). LifeOS records no in-flight work; nothing in `Work/` (no `Work/` folder in the LifeOS source folder enumeration).

## Gaps and known limitations

- **Five paradigms claimed in README, four shipped.** Genetic Programming has zero code and zero imports — the README's `[x]` checkboxes for GP features are genuinely false. Any external description of the project must say four methods, not five.
- **Parallel training dashboard does not exist.** README describes "All 5 algorithms train simultaneously in separate game instances" with an interactive sidebar UI; every roadmap checkbox in that section is unchecked.
- **Cross-method comparability is partly compromised.** Seven concrete confounds documented in LifeOS Gaps: parameter count (GA 1,227 vs ES 7,995 — 6.5× difference), compute budget per generation (GA 15×20=300 episodes; ES 100×3=300; NEAT 50×5=250; SAC 500K env steps on a different timebase), CRN default (ES True, GA False, NEAT True), fitness penalty (NEAT subtracts 1.0×std_dev; others don't), turn deadzone (NEAT 0.03, GA/ES 0.0), selection objective (GA scalar+novelty+diversity, ES Pareto, NEAT scalar+Pareto-display), novelty/diversity scaling (each method integrates differently), temporal input (only ES uses TemporalStackEncoder). Naive cross-method fitness comparisons are misleading without controls.
- **Plan ↔ code drift.** plans/GENETIC_ALGORITHM.md says `POPULATION_SIZE=10` and `SEEDS_PER_AGENT=5`; code says 15 and 20. plans/NEAT.md says no XOR sanity test exists; `tests/test_neat_xor.py` is 16.5 KB on disk. Reading plans without spot-checking against code produces a wrong project model.
- **47+ KB of dead TensorFlow code** (`feedforward_tf.py`, `nn_agent_tf.py`, `population_evaluator_tf.py`) plus ~21 KB of dead ES code (classic `driver.py`, `fitness_shaping.py`) plus stale `ESConfig` settings the active CMA-ES driver doesn't read.
- **No dependency manifest.** Reproducing a GNN-SAC run requires reverse-engineering the working torch + torch_geometric + PyG backend (`torch_scatter`, `torch_sparse`) combination — a known multi-hour pain point on a fresh machine. plans/GNN_SAC.md explicitly flags this; no fix shipped.
- **Python version drifts** (3.12 and 3.14 bytecode both present in `__pycache__`).
- **15 MB of generated training artefacts checked into git** instead of `.gitignore` + GitHub Releases. Every clone pulls the entire training history.
- **Test coverage ~2%.** Most drivers (GA, ES, NEAT, SAC), the analytics pipeline, the encoders, and the 45 KB shared evaluator are untested. The shared evaluator at 45 KB with no tests is a substantial risk for the comparability claim — silent behaviour drift propagates across all three evolutionary methods.
- **Three of four methods cannot resume a training run.** Only NEAT writes genome artefacts. GA/ES/SAC lose internal state on process exit (replay buffer, optimiser state, CMA-ES distribution state, all in memory only).
- **Broken / unused helpers.** `EnvironmentTracker.get_tick()` references `game.time` which doesn't exist. `VectorEncoder` is wired to nothing but is type-hinted in the evaluator signature. `GAConfig.NUM_NEAREST_ASTEROIDS=8` lingers from the `VectorEncoder` era. `linear.py` policy is imported by nothing.
- **17 of 22 reward components have never been used in a published run.** They represent deliberate optionality — a toolbox, not a curated recipe — but the work was invested without an experiment attached.
- **Analytics polish gaps.** Novelty/diversity scalars stored but not visualised, fresh-game "training fit" mismatch, generalisation ratio filtering bias hides frequent failures, `generalization_grade` thresholds undocumented in the report, evaluation seed not stored in exports for reproducibility.
- **No encoder schema versioning.** `HybridEncoder` and `GraphEncoder` emit no version tag; a change to ray count, fovea count, or normalisation bounds silently invalidates every previously-trained genome / ES mean / SAC checkpoint. Analytics has `SCHEMA_VERSION="2.3"` but encoders do not.
- **No wrap-aware collision detection.** Player and asteroid on opposite edges that are "adjacent" in toroidal space are tested as far apart in euclidean space; agents may learn to exploit the edge as a safe zone.

## Direction (in-flight, not wishlist)

LifeOS Roadmap explicitly notes the project is dormant and the roadmap is "what would make sense if work resumed — not a commitment to any of it. Caner's active projects are elsewhere." There is no in-flight work. The closest items to "actively wanted" (LifeOS-scoped, low effort, high leverage if work resumed) are:

- **Honest README pass:** delete or clearly mark GP as "not implemented"; remove `[x]` marks on GP roadmap items; update the dashboard section to say "designed, not built".
- **Add a `requirements-rl.txt` or `pyproject.toml`** with pinned PyTorch + torch_geometric + PyG backend deps so SAC is reproducible.
- **Gitignore generated artefacts** and move 15 MB of training history out of git.
- **Fix or delete broken tests** (`test_ga_dimensions.py`; resolve `test_neat_xor.py` status).
- **Update `plans/GENETIC_ALGORITHM.md`** to reflect real `POPULATION_SIZE=15` and `SEEDS_PER_AGENT=20`.

Not in flight, but the highest-value single piece of follow-on work LifeOS calls out: writing a **cross-method comparison report** under matched compute budgets and parameter counts — this is the project's actual value proposition, and without it AsteroidsAI is "four tutorials in a trench coat" (LifeOS Roadmap's exact framing).

## Demonstrated skills

- **Implements four optimisation paradigms from scratch in a single shared substrate** — custom GA driver (BLX-α crossover, stagnation-adaptive Gaussian mutation, tournament selection on combined fitness+novelty+diversity, elitism, all-time-best injection), custom NEAT (innovation tracking, structural mutations, speciation with adaptive compatibility threshold, fitness sharing, per-species elitism), custom diagonal CMA-ES (rank-mu update, target-rate covariance adaptation, sigma adaptation via cumulative path length, antithetic + CRN, restart from best candidate), and full SAC (twin critics with Polyak averaging, automatic entropy temperature tuning, asymmetric LRs, Huber loss, gradient clipping including AGC).
- **Designs and enforces a comparability substrate across four heterogeneous methods** — single 45 KB shared evaluator dispatched to method-specific drivers via `agent_factory` injection, single shared reward preset across all four methods, versioned analytics schema (`SCHEMA_VERSION="2.3"`) with identical structure across method outputs.
- **Builds Graph Neural Networks for variable-cardinality state** — `GATv2Conv` message passing over a bipartite asteroid → player graph with wrapped edge attributes encoding toroidal geometry; integrates the GNN as the perception backbone of an off-policy RL learner.
- **Implements multi-objective selection (NSGA-II-style)** — Pareto fronts + crowding distance over `[hits, time_alive, softmin_ttc]` with smooth soft-min TTC aggregator and accuracy guards against single-shot perfect-accuracy gaming; integrated into both ES selection and NEAT display.
- **Designs novelty + diversity selection mechanisms grounded in behaviour space, not reward space** — 7D action-space behaviour vector, kNN novelty scoring, bounded archive with novelty-threshold admission, Shannon entropy over positive reward components.
- **Engineers two-mode physics parity** — windowed Arcade game and headless rollout simulator sharing a single constants file with documented parity gotchas (e.g., headless `bullet_list` requires explicit `lifetime > 0` filtering because Arcade's `remove_from_sprite_lists()` is a no-op against plain Python lists).
- **Applies variance-reduction discipline to evolution-strategy gradient estimation** — antithetic sampling paired with CRN per-method opt-in, captured as a deliberate cross-method tuning matrix and explained in plan files.
- **Builds a 200+ field analytics pipeline with a method-agnostic facade** — `TrainingAnalytics` + collectors + analysers + reporters producing 45-68 KB markdown reports with sparklines, per-section takeaways, glossaries, training-phase analysis, and reward-evolution analysis; deliberately over-collects (~40 behavioural keys per generation × population × generations) so any cross-section can be surfaced retroactively.
- **Designs explicit failure-mode taxonomies for RL diagnostics** — exploration collapse, critic miscalibration, reward-scale instability, GNN oversmoothing — each tied to a primary fix knob and a set of analytics keys, motivating the densest diagnostic block in the project.
- **Practises anti-puffing engineering hygiene at the documentation layer** — LifeOS notes for this project explicitly enumerate where the README pitch outpaces the code (GP not implemented, dashboard not built, comparability confounds), where plans drift from code, and where dead code accumulates. This level of self-audit is itself a portfolio signal.
- **Composable reward design with 22 components** — `ComposableRewardCalculator` plus a 22-component library covering survival, accuracy, exploration, aim alignment, momentum, ammo conservation, near-misses, proximity penalties, leading targets, and more; with reward-balance principle ("no single component dominates") and `max_time_alive`-scaled DeathPenalty.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/AsteroidsAI/Overview.md | 130 | "#project/asteroids-ai #lang/python #paradigm/evolutionary #paradigm/neat #paradigm/cma-es #paradigm/sac #ml/rl #ml/gnn #status/dormant" |
| Projects/AsteroidsAI/Architecture.md | 290 | "- [[AsteroidsAI/Roadmap]] — the planned parallel dashboard and its architectural implications" |
| Projects/AsteroidsAI/Decisions.md | 278 | "- [[AsteroidsAI/Roadmap]] — decisions deliberately deferred appear as roadmap items" |
| Projects/AsteroidsAI/Gaps.md | 257 | "- [[AsteroidsAI/Systems/Analytics Pipeline]] — analytics polish gaps" |
| Projects/AsteroidsAI/Roadmap.md | 282 | "- [[Projects/Index]] — AsteroidsAI relative to Caner's other active projects (all of which are higher priority as of 2026-04-24)" |
| Projects/AsteroidsAI/Systems/Game Engine.md | 193 | "- [[AsteroidsAI/Gaps]] — broken `get_tick()`, wrap-aware collision, unused arcade APIs" |
| Projects/AsteroidsAI/Systems/State Encoders.md | 219 | "- [[AsteroidsAI/Gaps]] — encoder drift, schema versioning, VectorEncoder dead code" |
| Projects/AsteroidsAI/Systems/Reward System.md | 174 | "- [[AsteroidsAI/Gaps]] — 17 components never exercised in a run" |
| Projects/AsteroidsAI/Systems/Genetic Algorithm.md | 225 | "- [[Vynapse/Overview]] — Caner's Rust neuroevolution engine; solves similar fixed-topology evolutionary problem in a different language" |
| Projects/AsteroidsAI/Systems/Evolution Strategies.md | 291 | "- [[AsteroidsAI/Roadmap]] — the Easy/Medium/Hard roadmap from the ES plan is the richest source of next-session ideas" |
| Projects/AsteroidsAI/Systems/NEAT.md | 226 | "- [[Vynapse/Overview]] — Vynapse's `trainers/neat.rs` is a 0-byte stub; AsteroidsAI's NEAT is the working reference implementation" |
| Projects/AsteroidsAI/Systems/GNN-SAC.md | 355 | "- [[NeuroDrive/Overview]] — NeuroDrive's asymmetric PPO (actor 2x64, critic 2x128) is the sibling gradient-based RL in the vault; comparing SAC continuous control here with PPO continuous control there is a useful cross-project analogy" |
| Projects/AsteroidsAI/Systems/Shared Components.md | 225 | "- [[AsteroidsAI/Gaps]] — method-parity normalisation not done; cross-method bonus magnitudes not comparable" |
| Projects/AsteroidsAI/Systems/Analytics Pipeline.md | 257 | "- [[AsteroidsAI/Roadmap]] — analytics polish is much of the remaining in-repo roadmap" |
