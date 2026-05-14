---
name: AsteroidsAI
status: dormant
source_repo: https://github.com/Capataina/Asteroids-AI
lifeos_folder: Projects/AsteroidsAI
last_synced: 2026-05-13
sources_read: 16
---

# AsteroidsAI

## One-line summary

Python comparative-ML research playground that pits four optimisation paradigms (Genetic Algorithm, diagonal CMA-ES, NEAT, and GNN-backed Soft Actor-Critic) against the same Asteroids-style continuous-control environment through a shared evaluator, encoder family, reward preset, and analytics schema.

## What it is

AsteroidsAI is a "single environment, multiple minds" benchmarking platform — the explicit premise is that holding the environment, state encoders, reward components, and analytics pipeline constant lets differences in learning behaviour be attributed to the paradigm rather than the substrate. The repo stretches across 42 commits from 2025-02-25 to 2026-02-22 with the substantive development concentrated between 2025-12-28 and 2026-01-23 (Phase 1: reward system, Phase 2: GA, Phase 3: analytics, Phase 4: ES, Phase 5: NEAT and SAC landed on the same day, 2026-01-20). The four implemented methods all share a `ComposableRewardCalculator` preset, the `HeadlessAsteroidsGame` for seeded parallel rollouts, the `HybridEncoder` (47-dim fovea+raycast state) or its temporal/graph variants, and a `TrainingAnalytics` pipeline that produces JSON exports plus markdown reports with a versioned schema (`SCHEMA_VERSION = "2.3"`). The README advertises five paradigm families and a parallel training dashboard; LifeOS explicitly flags this as the project's anti-puffing headline — Genetic Programming is zero code and the dashboard has every roadmap checkbox unchecked. As of 2026-04-24 the project has been dormant for roughly two months (last commit `1c55da4`, "Mark roadmap items complete in README" — a tidy-up commit on top of `75a566f` "ran the training again").

## Architecture

AsteroidsAI is layered top-down so the environment, the interface contracts, the agents, and the training algorithms move independently. Every method obeys the same per-step boundary: `game -> interfaces -> agent -> action -> game`.

```
+-----------------------------------------------------------------+
|  Training scripts (training/scripts/)                           |
|    train_ga.py  train_es.py  train_neat.py  train_gnn_sac.py    |
|    view_gnn_sac.py  simulate_gnn_sac.py                         |
|  Orchestrate: evaluator + driver + display + analytics          |
+-----------------------------------------------------------------+
                             |
         +-------------------+---------------------+
         v                   v                     v
+----------------+ +------------------+ +----------------------+
| training/      | | training/methods/| | training/analytics/  |
| core/          | |                  | |  collection/         |
|                | |  genetic_algorithm/|  analysis/          |
| population_    | |  evolution_      | |  reporting/          |
| evaluator      | |  strategies/     | |  TrainingAnalytics   |
| episode_runner | |  neat/           | |  facade              |
| display_       | |  sac/            | |                      |
| manager        | |                  | |                      |
+----------------+ +------------------+ +----------------------+
                             |
                             v
+-----------------------------------------------------------------+
|  Agents (ai_agents/)                                            |
|    BaseAgent contract (get_action + reset)                      |
|    NNAgent (NumPy) -- used by GA + ES                           |
|    NEATAgent -- wraps a compiled NEAT feedforward network       |
|    SACAgent -- PyTorch inference wrapper for GNN-SAC            |
|    NNAgentTF / policies/feedforward_tf -- present, unused       |
+-----------------------------------------------------------------+
                             |
                             v
+-----------------------------------------------------------------+
|  Interfaces (interfaces/)                                       |
|    StateEncoder abstract                                        |
|      HybridEncoder (47-dim) -- GA + ES (base) + NEAT            |
|      TemporalStackEncoder (wraps HybridEncoder, 329-dim) -- ES  |
|      GraphEncoder -- GNN-SAC                                    |
|      VectorEncoder -- present, unused                           |
|    ActionInterface  (boolean | continuous)                      |
|    EnvironmentTracker  (spatial queries, wrapped distances)     |
|    MetricsTracker  (shots/hits/kills/time_alive)                |
|    RewardCalculator -- ComposableRewardCalculator + 22 comps    |
+-----------------------------------------------------------------+
                             |
                             v
+-----------------------------------------------------------------+
|  Game (game/ + Asteroids.py)                                    |
|    AsteroidsGame  (arcade.Window, rendering + manual play)      |
|    HeadlessAsteroidsGame  (seeded parallel rollouts)            |
|    classes: Player / Bullet / Asteroid                          |
|    globals.py (physics + collision constants)                   |
|    debug/visuals.py (overlays + HybridEncoder ray debug)        |
+-----------------------------------------------------------------+
```

Dependency direction is top-down only — the game knows nothing about agents; interfaces know nothing about training methods; methods hand metric dicts up to the analytics reporting layer rather than importing it. Folder weights from the LifeOS scan: `training/` 3.09 MB (includes 2.1 MB of NEAT and SAC artefacts), `interfaces/` 240 KB (23% of source), `plans/` 178 KB (17% — high docs-to-source ratio), `game/` 91 KB (9%), `ai_agents/` 57 KB (6%), `tests/` 60 KB (6%).

**The shared evaluator is the architectural centre of gravity.** `training/core/population_evaluator.py` is 45 KB — the largest Python file in the repo — and is used by GA, ES, and NEAT (SAC has its own step-based collector loop). Its signature accepts `agent_factory`, `state_encoder`, `action_interface`, `reward_factory`, `seeds`, `use_common_seeds`, etc., so each method injects what it needs while inheriting identical seeded parallel rollouts, metric collection, and behaviour-vector / reward-diversity computation. The three evolutionary methods sharing this evaluator is what makes the benchmarking claim *enforceable* — the only legitimate cross-method differences are the driver logic, the config, and the encoder wrapper.

**Toroidal world.** Positions wrap when they leave the screen bounds. `EnvironmentTracker.get_distance(...)` computes shortest wrapped distance. `HybridEncoder` rays use "ghost targets" — asteroids virtually duplicated into 8 surrounding grid cells (N, NE, E, SE, S, SW, W, NW) — so ray intersection tests can see across edges without wrapping the ray itself.

**Operational invariants.** `StateEncoder.clone()` is mandatory (parallel rollouts need their own encoder instance to avoid shared mutable state in temporal buffers and ray history). `update_internal_rewards=False` during training (the windowed game's legacy reward calculator must be suppressed so external `create_reward_calculator()` controls fitness). Collision detection uses explicit radii from `globals.py` because arcade sprite-textures may not load in headless mode. The headless path explicitly filters `bullet_list` and `asteroid_list` by `lifetime > 0` every step because `remove_from_sprite_lists()` is a no-op against plain Python lists — a parity-critical bug fix.

## Subsystems and components

### Game Engine

Two execution paths over the same physics: `Asteroids.py` (`AsteroidsGame(arcade.Window)`, 13 KB, 341 lines) for manual play, best-agent playback, and fresh-game generalisation capture; and `game/headless_game.py` (`HeadlessAsteroidsGame`, 10.7 KB) for thread-pooled parallel seeded rollouts during training. Both read constants from `game/globals.py` (1.6 KB) — `SCREEN_WIDTH=800`, `SCREEN_HEIGHT=600`, `PLAYER_ACCELERATION`, `PLAYER_FRICTION`, `PLAYER_ROTATION_SPEED`, `BULLET_SPEED`, `BULLET_LIFETIME`, `BULLET_COOLDOWN`, `ASTEROID_SPAWN_INTERVAL`, per-tier `ASTEROID_SPEED_*` / `ASTEROID_HP_*` / `ASTEROID_SCALE_*`, and explicit collision radii. Entities: `Player` (3.3 KB) with velocity + friction integration, discrete or continuous rotation, thrust in facing direction, cooldown-based shooting; `Bullet` (969 B) with constant-speed motion and lifetime decrement; `Asteroid` (4.6 KB) with edge-spawn, randomised velocity/rotation, three-tier HP/scale hierarchy (large/medium/small) and fragmentation on kill. Control flags (`update_internal_rewards`, `auto_reset_on_collision`, `manual_spawning`, `external_control`) are exposed on both modes so training can configure them identically. Two control surfaces: boolean (key-press style for GA/ES/NEAT) and continuous (`continuous_control_mode` with `apply_continuous_controls(turn_mag, thrust_mag)` for SAC — shoot stays boolean even in continuous mode because the bullet cooldown is discrete).

### State Encoders

Four encoders, three active:

- **HybridEncoder (47-dim, 11 KB):** 3 proprioception (forward velocity egocentric, lateral velocity egocentric, shoot cooldown fraction) + 3 fovea asteroids × 4 features (wrapped distance, signed bearing, closing speed, asteroid scale) + 16 rays × 2 features (normalised hit distance, normalised closing speed along ray). Fovea + rays redundancy is intentional — pure nearest-N (the now-dead `VectorEncoder`) produced spin-lock turret policies; rays force the policy to care about threats outside the fovea. Used by GA, ES (as the base), and NEAT.
- **TemporalStackEncoder (2.1 KB):** wraps any base encoder and produces `[s(t), s(t-1), s(t-2), s(t-3), delta(t), delta(t-1), delta(t-2)]` = `base * (2N-1)`. For `HybridEncoder(47) + N=4`: 329 features. Used only by ES because ES gradient estimation is sensitive to evaluation noise — temporal stacking smooths the fitness landscape. The 6.5× parameter blow-up vs GA (7,995 vs 1,227) is a real cross-method confound.
- **GraphEncoder (8.9 KB):** emits a `GraphPayload` — `player_features` (5: vel_x, vel_y, heading_sin, heading_cos, shoot_cooldown_frac), `asteroid_features` (N × 3: scale, vel_x, vel_y), `edge_attr` (N × 7 directed asteroid → player: wrapped dx/dy, dist, bearing_sin/cos, rel_vx/vy). Bipartite edges only (no asteroid ↔ asteroid). Wrapped deltas in edge attributes — the player node has no absolute position. Used by GNN-SAC.
- **VectorEncoder (9.2 KB):** dead baseline, 35-dim (3 proprioception + 4 × 8 nearest asteroids with `[1.0, 0.0, 0.0, 0.0]` padding). Not used by any training script. `training/core/population_evaluator.py` still type-hints `state_encoder: VectorEncoder` as latent duck-typing residue.

Encoder schema versioning does not exist — a change to ray count, fovea count, or normalisation bounds silently invalidates every previously trained genome / ES mean / SAC checkpoint. Analytics has a `SCHEMA_VERSION = "2.3"` tag; encoders do not have the equivalent.

### Reward System

`ComposableRewardCalculator` (3.4 KB interface) holds a list of `RewardComponent` instances. 22 reward components ship in `interfaces/rewards/` totalling 47 KB (sorted by size: `DistanceBasedKillReward` 3.9 KB, `ProximityFacingBonus` 3.6 KB, `TargetLockReward` 2.8 KB, `LeadingTargetBonus` 2.7 KB, `ExplorationBonus` 2.4 KB, `MovingTowardDangerBonus` 2.4 KB, `DeathPenalty` 2.2 KB, `VelocityKillBonus` 2.2 KB, `ProximityPenalty` 2.1 KB, `VelocitySurvivalBonus` 2.1 KB, `FacingAsteroidBonus` 2.0 KB, `MaintainingMomentumBonus` 2.0 KB, `ConservingAmmoBonus` 2.0 KB, `SpacingFromWallsBonus` 2.0 KB, `ChunkBonus` 1.8 KB, `KPMBonus` 1.7 KB, `AccuracyBonus` 1.2 KB, `ShootingPenalty` 1.2 KB, `NearMiss` 1.2 KB, `KillAsteroid` 1.1 KB, `SurvivalBonus` 953 B).

The shared preset (`training/config/rewards.py:create_reward_calculator`) — used by all four methods — exercises only five components:

```
VelocitySurvivalBonus(reward_multiplier=1.5, max_velocity_cap=15.0)
DistanceBasedKillReward(max_reward_per_kill=18.0, min_reward_fraction=0.15)
ConservingAmmoBonus(hit_bonus=4.0, shot_penalty=-2.0)
ExplorationBonus(grid_rows=3, grid_cols=4, bonus_per_cell=5.0)
DeathPenalty(penalty=-150.0, early_death_scale=1.0, max_time_alive=max_steps*frame_delay)
```

The preset is "rebalanced so no single component dominates total fitness", and `DeathPenalty` is plumbed with `max_time_alive` so dying early scales the penalty up. Reward anatomy (per-step contributions per component) is tracked into the `reward_breakdown` dict that feeds reward diversity scoring (Shannon entropy over positive components) and analytics reporting (`reward_component_shares`, `reward_entropy`, `reward_dominance_index`, `reward_max_share`).

### Genetic Algorithm

First method implemented; the pilot that proved the shared substrate. `training/methods/genetic_algorithm/` — `driver.py` (8.9 KB), `operators.py` (3.5 KB), `selection.py` (0.9 KB). Genome is a flat `List[float]` of length 1,227 (47×24 + 24 + 24×3 + 3 from `HybridEncoder(47) -> 24 tanh hidden -> 3 sigmoid outputs`). Policy compiled by `FeedforwardPolicy` (2.2 KB) and wrapped in `NNAgent` (1 KB). Per-generation loop: `evaluate_population_parallel(...)` over `ThreadPoolExecutor(max_workers=os.cpu_count())`, `DisplayManager.start_display(...)` windowed best-of-gen playback, `GADriver.evolve(...)` (mutation adaptation under stagnation, combined-score selection scores, tournament-of-3 over `fitness + novelty + diversity`, BLX-alpha crossover at prob 0.7 with alpha=0.5, Gaussian mutation N(0, sigma=0.1) at per-gene prob 0.05, top ~10% elitism, all-time-best injection when stagnation < 5).

Code config: `POPULATION_SIZE=15`, `NUM_GENERATIONS=500`, `SEEDS_PER_AGENT=20`, `MAX_STEPS=1500`, `FRAME_DELAY=1/60`, `USE_COMMON_SEEDS=False`, `HIDDEN_LAYER_SIZE=24`. Plans drifted from code: `plans/GENETIC_ALGORITHM.md` lists `POPULATION_SIZE=10` and `SEEDS_PER_AGENT=5` — a 4× discrepancy in compute per generation.

### Evolution Strategies (CMA-ES)

`training/methods/evolution_strategies/cmaes_driver.py` (10.8 KB, used) plus `driver.py` (17.7 KB classic ES, unused) plus `fitness_shaping.py` (3 KB rank transform + utility, unused by CMA-ES). Diagonal CMA-ES (not full covariance — full LM-CMA was deferred for n^2 memory and n^3 update cost on the 7,995-param vector). Per generation: sample 100 candidates with optional antithetic pairs (`x_i = mean + sigma*y_i`, `x_{i+50} = mean - sigma*y_i`), evaluate Pareto objectives `[hits, time_alive, softmin_ttc]`, rank by Pareto fronts + crowding, select top mu=50 by Pareto order, update mean via weighted direction, update evolution paths `p_sigma` and `p_c`, update diagonal covariance `cov_diag <- (1-c1-cmu)*cov_diag + c1*p_c^2 + cmu*sum w_i*y_i^2`, update sigma. Target-rate adaptation: `CMAES_COV_TARGET_RATE=1e-3` with `CMAES_COV_MAX_SCALE=1e4` rescales `c1/cmu` to hit a target learning rate (additive stability mechanism, not standard CMA-ES). Noise handling: re-evaluate top-5 candidates with extra seeds (`NOISE_HANDLING_TOP_K=5`, `NOISE_HANDLING_EXTRA_SEEDS=1`, `NOISE_HANDLING_SEED_OFFSET=100000`). Restarts: `RESTART_ENABLED=True`, `RESTART_PATIENCE=12`, `RESTART_USE_BEST_CANDIDATE=True`. Tracks the actual best-performing candidate parameter vector (not the smoothed mean) for playback.

CRN is mandatory for ES (`USE_COMMON_SEEDS=True`) because gradient estimation from fitness differences requires paired samples to see identical environments — without CRN, antithetic sampling collapses.

### NEAT

NeuroEvolution of Augmenting Topologies — the only method that evolves connection weights and network *structure* together. `ai_agents/neuroevolution/neat/genome.py` (12.9 KB) holds genes, mutations, crossover, and compatibility distance. `training/methods/neat/driver.py` (16.2 KB) implements speciation, fitness sharing, species stagnation, per-species elitism, adaptive compatibility threshold. `innovation.py` (1.5 KB) assigns globally-unique innovation IDs and tracks add-node splits. `network.py` (2.8 KB) compiles genomes into feedforward DAGs and runs forward passes (feedforward only — no recurrence).

Config: `POPULATION_SIZE=50`, `SEEDS_PER_AGENT=5`, `USE_COMMON_SEEDS=True`, `COMPATIBILITY_THRESHOLD=0.25` with `C1=1.0, C2=1.0, C3=0.4` (excess/disjoint/weight-diff coefficients), `SPECIES_STAGNATION=7` (reduced from 15 to fail fast on dead-end species), `ADAPT_COMPATIBILITY_THRESHOLD=True` with `TARGET_SPECIES=8` and step `+/-0.02`, `FITNESS_STD_PENALTY_RATIO=1.0` (subtracts one std-dev of per-seed fitness from the reported fitness — strong anti-overfitting-to-lucky-seeds signal, applied only by NEAT), `WEIGHT_MUTATION_SIGMA=0.5` (5× GA's 0.1), `ADD_CONNECTION_PROB=0.05`, `ADD_NODE_PROB=0.03`, `MAX_NODES=None` / `MAX_CONNECTIONS=None` (no hard complexity cap), `EARLY_STOPPING_GENERATIONS=25`. Per-generation artefacts: `training/neat_artifacts/gen_XXXX_best.{json,dot}` × 50 gens plus `best_overall.{json,dot}` (~1.5 MB total committed). DOT files open directly in Graphviz for visual topology inspection — the only method in the repo where you can *see* the architecture the optimiser built.

NEAT uses `ActionInterface(turn_deadzone=0.03)` while GA/ES use `0.0` — accommodates NEAT's structural plasticity (newly-added output sigmoids drift around 0.5 but rarely hit it). Pareto ordering on `[hits, time_alive, softmin_ttc]` for *display* best-genome selection only; reproduction is still fitness-driven.

### GNN-SAC

The reinforcement learning method — Soft Actor-Critic with a Graph Neural Network backbone, trained with true continuous control. Only method using PyTorch. Last method added (2026-01-20).

`training/methods/sac/` — `learner.py` (15.4 KB: critic update, actor update, entropy auto-tuning, target Polyak averaging, gradient clipping, AGC), `networks.py` (11.8 KB: `GNNBackbone` with `torch_geometric.nn.GATv2Conv` × 2 layers / 64 hidden / 4 heads / dropout 0.0, `Actor` MLP 256-hidden over graph embedding outputting (turn, thrust, shoot) means + log-stds with tanh squash, `TwinCritics` Q1 + Q2 each MLP 256-hidden over [embedding + action]), `replay_buffer.py` (4.9 KB: graph-native replay storing variable-size graph transitions and collating batches), `normalization.py` (4.6 KB: `GraphNormalizer` running mean/std over graph features). Three entry scripts: `train_gnn_sac.py` (37.3 KB headless training — single biggest orchestration file in the repo), `view_gnn_sac.py` (5.1 KB windowed read-only playback), `simulate_gnn_sac.py` (36.4 KB demo mode that trains headless at `TRAIN_STEPS_PER_FRAME=2` while rendering — viewer seed rotation via `VIEWER_SEED_MODE="increment"`, `VIEWER_SEED_START=200000`, `VIEWER_SEED_RANGE=(200000, 900000)`).

Config: `TOTAL_STEPS=500,000`, `MAX_EPISODE_STEPS=1500`, `GAMMA=0.99`, `TAU=0.005`, `BATCH_SIZE=256`, `REPLAY_SIZE=100,000`, `LEARN_START_STEPS=5,000`, `UPDATES_PER_STEP=1`, `REWARD_SCALE=0.2`, `ACTOR_LR=3e-4`, `CRITIC_LR=1e-4` (3× slower than actor — deliberate stability choice for GNN's noisy early embeddings), `ALPHA_LR=3e-4`, `GRAD_CLIP_NORM=10.0`, `AGC_ENABLED=True`, `AGC_CLIP_FACTOR=0.01`, `AUTO_ENTROPY=True`, `INIT_ALPHA=0.2`, `TARGET_ENTROPY=-3.0`, `CRITIC_LOSS="huber"` with `HUBER_DELTA=1.0` (reduces sensitivity to outlier TD errors from variable-size graph batches), `EVAL_EVERY_EPISODES=5`, `EVAL_SEEDS=[1001..1005]` (intentionally outside training seed range), `NUM_COLLECTORS=1`, `MAX_ASTEROIDS=None`, `OBS_NORM_ENABLED=True`, `ACTION_SMOOTHING_ENABLED=False`. Best checkpoint saved to `training/sac_checkpoints/best_sac.pt` (623 KB: GNN + actor weights + eval metadata). Replay buffer and optimizer state are *not* persisted — resume works for inference but not for continued training. Four named failure modes drive the diagnostic surface: exploration collapse, critic miscalibration, reward-scale instability, GNN oversmoothing.

### Analytics Pipeline

The comparability substrate. Every method feeds the same `TrainingAnalytics` facade. Schema-versioned (`SCHEMA_VERSION = "2.3"`). API: `set_config(dict)` / `record_generation(stats_dict)` / `record_distributions(gen, dists)` / `record_fresh_game(gen, fresh, gen_metrics)` / `save_json(path)` / `generate_markdown_report(path)`. Data model: `generations_data: List[dict]`, `fresh_game_data: Dict[int, dict]`, `distributions_data: Dict[int, dict]`, `config: dict`, `start_time`, `all_time_best_fitness`, `all_time_best_generation`, `generations_since_improvement`.

Per-generation surface: 30+ always-on keys (fitness moments and percentiles `best_fitness/avg_fitness/min_fitness/median_fitness/p25/p75/p90`, `std_dev`, `population_size`, `best_improvement/avg_improvement`, `all_time_best`, `generations_since_improvement`, `evaluation_duration`, `evolution_duration`, `crossover_events/mutation_events/elite_count`, `avg_novelty/avg_diversity/archive_size`) plus up to 100+ method-specific keys. NEAT block (13 keys): `species_count/species_min_size/max/median/pruned`, `avg_nodes/avg_connections`, `best_nodes/best_connections`, `compatibility_threshold/mean/p10/p90`, `add_node_events/add_connection_events/weight_mutation_events`, `innovation_survival_rate`. ES block (10 keys): `sigma`, `mean_param_norm`, `cov_diag_mean/min/max/std`, `cov_diag_mean_abs_dev/max_abs_dev`, `cov_lr_scale/effective_rate`, `pareto_enabled/front0_size/best_crowding`. SAC block (~50 keys — densest): timebase (`sac_env_steps_total`, `sac_updates_total`), eval returns per-seed, learner stability (critic/actor loss, alpha, entropy, Q1/Q2 stats, target Q, TD error magnitudes mean/p90/p99, gradient norms + clip rates), action health (turn/thrust/shoot mean/std + zero rate + saturation rate), replay health, representation health (embedding norm/std/cosine-similarity for collapse detection, policy drift on probe set, critic-target gap, GNN/actor/critic weight mean/std/norm/zero-fraction). Behavioural metrics (~40 keys, always collected for evolutionary methods): combat (kills/accuracy/hits/shots/shots_per_kill/shots_per_hit), survival (steps/time_alive/max_steps), action rates including detailed turn breakdown (left_only/right_only/both_turn + durations + value mean/std + deadzone rate + switch rate + balance + streaks), aim alignment (frontness/frontness_at_shot/frontness_at_hit), engagement (idle_rate/asteroid_dist/screen_wraps/distance_traveled/speed/coverage_ratio), risk (min_dist/danger_exposure_rate/danger_entries/danger_reaction_time/danger_wraps/softmin_ttc), robustness (`fitness_std` across seeds), neural health (output_saturation, action_entropy), reward anatomy (reward_breakdown, quarterly_scores, reward_component_shares, reward_entropy, reward_dominance_index, reward_max_share, reward_positive_component_count), heatmap inputs (best_agent_positions, best_agent_kill_events, population_positions, population_kill_events).

Distributions: 23 sorted per-agent value lists per generation (`fitness_values`, `kills_values`, `steps_values`, `accuracy_values`, ... `fitness_std_values`) plus distribution stats (`fitness_skewness/kurtosis`, `viable_agent_count`, `failed_agent_count`, std-dev snapshots). Fresh-game generalisation: `fresh_game` dict + `generalization_metrics` (`fitness_ratio`, `kills_ratio`, `steps_ratio`, `accuracy_delta`, `generalization_grade` — letter grade thresholds opaque, planned to surface in report). Reporting (`training/analytics/reporting/`): `markdown.py` (11.2 KB MarkdownReporter), `json_export.py` (4.1 KB), `insights.py` (4.7 KB takeaways + warnings), `glossary.py` (10.7 KB metric definitions). Report anatomy includes 20-column ASCII sparklines with phase-based tags, per-section takeaways + warnings, per-section glossary, training-phase splits, milestones (run-relative thresholds), reward evolution analysis, distribution analysis with mean +/- std charts.

NEAT's 50-pop × 500-gen × 40-metric run produces an 8.5 MB `training_data_neat.json` — the largest file in the repo. GA produces 4.9 MB, SAC 992 KB, ES 394 KB (smaller because of population × seeds × episode shape).

### Shared Components

Method-agnostic selection-shaping utilities used by GA, ES, and NEAT (SAC logs compatible metrics only).

- **Behaviour vector** (`training/components/novelty.py`, 5.5 KB): 7D `[0, 1]^7` characterisation — thrust rate, turn rate, shoot rate, accuracy, idle rate, engagement distance (`avg_asteroid_dist/400`), screen coverage (`screen_wraps/20`). Deliberately reward-agnostic.
- **Behaviour archive** (`archive.py`, 3.7 KB): bounded archive, admits behaviours exceeding `novelty_threshold`, random replacement when full.
- **Novelty score**: mean Euclidean distance to k nearest behaviours in population + archive.
- **Reward diversity** (`diversity.py`, 4.3 KB): Shannon entropy over positive reward components only, normalised to `[0, 1]`. Returns 0 if only one positive component or net negative — preventing the bonus from propping up degenerate behaviours.
- **Selection score** (`selection.py`, 3.9 KB): `score = fitness + (novelty_weight * novelty * novelty_fitness_scale if enable_novelty) + (diversity_weight * reward_diversity * max(1, |fitness|) if enable_diversity)`.
- **Pareto** (`training/components/pareto/`): `objectives.py` (2.6 KB build per-candidate objective vector), `ranking.py` (2.9 KB NSGA-II style fronts + crowding distance), `utility.py` (967 B ordering helper). Config: `OBJECTIVES=["hits","time_alive","softmin_ttc"]`, `ACCURACY_MIN_SHOTS=5` with `ACCURACY_ZERO_BELOW_MIN_SHOTS=True` (zeros accuracy below 5 shots to prevent single-shot perfect-accuracy genomes from gaming Pareto), `FITNESS_TIEBREAKER=True`. `softmin_ttc` is an exponentially-weighted soft-min over all asteroid TTCs — smoother than hard-min which is discontinuous under motion.

The three evolutionary methods use novelty/diversity differently: GA tournament-score post-multiplication, ES pre-rank-shaping scaled by fitness spread (stddev with a floor), NEAT selection bonus.

## Technologies and concepts demonstrated

### Languages

- **Python (3.12 / 3.14)** — 141 `.py` files, ~806 KB Python source. The `.pyc` cache shows both 3.12 and 3.14 compiled bytecode, so the supported version range is ambiguous. NumPy-only for evolutionary methods, PyTorch + PyTorch Geometric for GNN-SAC.

### Frameworks and libraries

- **NumPy** — `FeedforwardPolicy` MLP (tanh hidden + sigmoid output), genome unpacking, ES diagonal-CMA covariance update, all evolutionary tensor work.
- **PyTorch** — GNN backbone, actor + twin critics, replay buffer, learner update loop, AGC, gradient clipping. Used exclusively by SAC.
- **PyTorch Geometric (`torch_geometric`)** — `GATv2Conv` message-passing layers (× 2 layers, 64 hidden, 4 attention heads). Backend deps `torch_scatter`/`torch_sparse` are required at install time but not pinned (no `requirements-rl.txt`).
- **Arcade** — windowed game (`AsteroidsGame(arcade.Window)`), sprite-list management, manual play + best-agent playback rendering.

### Runtimes / engines / platforms

- **Two-mode game engine** — windowed Arcade for playback / debugging / fresh-game generalisation, parallel-safe headless mode for thread-pooled rollouts; both read constants from a shared `globals.py`. Headless explicitly filters `bullet_list`/`asteroid_list` by `lifetime > 0` because `remove_from_sprite_lists()` no-ops on plain Python lists.
- **`ThreadPoolExecutor` (Python stdlib)** — `max_workers=os.cpu_count()` over `evaluate_population_parallel`. Per-thread RNGs (`random.Random(random_seed)` per game instance) prevent global-state contamination across concurrent rollouts.

### Tools

- **`scan_repo.py` / `fetch_commits.py` / `repo_stats.py`** — bespoke repo-inspection scripts used to verify scale claims in the LifeOS notes (file counts, commit counts, largest files, folder weights).
- **Graphviz (`dot`)** — NEAT genome topologies exported as `.dot` files per generation, renderable to PNG with `dot -Tpng`. The only method in the repo where the optimised architecture is visually inspectable.

### Domains and concepts

- **Genetic Algorithms** — flat-vector genomes (1,227 floats), tournament-of-3 selection over combined score (fitness + novelty + diversity), BLX-alpha blend crossover with alpha=0.5, Gaussian mutation N(0, sigma=0.1) at per-gene prob 0.05, top ~10% elitism, mutation-adaptation under stagnation, all-time-best injection.
- **Evolution Strategies / diagonal CMA-ES** — sampling distribution (mean, sigma, diagonal covariance), evolution paths `p_sigma` + `p_c`, antithetic sampling for variance reduction, Common Random Numbers (CRN) for paired sample comparability, target-rate adaptation to scale `c1+cmu`, noise handling via top-K re-evaluation, stagnation-triggered restart with `RESTART_USE_BEST_CANDIDATE`, weighted-direction mean update.
- **NEAT (Neuroevolution of Augmenting Topologies)** — innovation tracking with globally unique IDs, add-connection / add-node structural mutations with cycle checks, gene-alignment crossover by innovation id with excess/disjoint handling, disabled-gene inheritance with prob 0.75, compatibility distance `d = C1*E/N + C2*D/N + C3*mean_dw`, fitness sharing (`adjusted_fitness = fitness / species_size`), species stagnation pruning at 7 gens, adaptive compatibility threshold targeting 8 species, fitness-std penalty for noise robustness.
- **Soft Actor-Critic (SAC)** — entropy-regularised stochastic policy, automatic temperature `alpha` learning toward `TARGET_ENTROPY=-3.0`, twin critics + target Q with Polyak averaging at `tau=0.005`, Huber TD loss (`delta=1.0`), asymmetric LR (actor 3e-4 vs critic 1e-4), AGC adaptive gradient clipping (`clip_factor=0.01`) on top of global grad-norm clip, reward scaling 0.2.
- **Graph Neural Networks** — `GATv2Conv` attention-based message passing over a bipartite asteroid → player graph; player has no absolute position; wrapped deltas in edge attributes for the toroidal world; variable-cardinality input (graph grows/shrinks with asteroid count, optional cap via `MAX_ASTEROIDS`).
- **Pareto multi-objective optimisation** — NSGA-II fronts + crowding distance, soft-min time-to-collision aggregator (smoother than hard-min), accuracy guard to prevent single-shot perfect-accuracy gaming, fitness tiebreaker within front.
- **Continuous control** — analog turn magnitude in [-1, 1] and thrust magnitude in [0, 1] (shoot stays boolean since cooldown is discrete), tanh-squashed Gaussian policy outputs, scaled into `PLAYER_ROTATION_SPEED * turn_magnitude` and proportional thrust.
- **Comparative ML / benchmarking design** — single environment, shared reward preset, shared encoder family, shared evaluator, schema-versioned analytics — the substrate that makes "differences are paradigm differences, not infrastructure differences" potentially defensible.
- **Behavioural novelty and diversity selection** — 7D action-and-engagement behaviour vector, kNN novelty scoring against population + archive, Shannon-entropy reward diversity over positive components, three different per-method integration patterns.
- **Toroidal-world geometry** — wrapped distance utility (`EnvironmentTracker.get_distance`), ghost-target ray duplication into 8 surrounding grid cells, wrapped deltas in graph edge attributes.
- **Analytics-pipeline-as-comparability-substrate** — versioned schema, method-agnostic facade, per-method optional keys, ~40 always-on behavioural metrics, 23 distribution lists per gen, fresh-game generalisation capture with `fitness_ratio`/`kills_ratio`/`steps_ratio`/`accuracy_delta`/`generalization_grade`, markdown reports with sparklines + per-section takeaways + glossary.

## Key technical decisions

- **D1 — Four methods, one environment, one reward preset.** Comparability is the project's core value proposition; sharing the substrate is non-negotiable. Diverging on reward preset would let every method "win its own game" and kill the study. PPO was considered and rejected for SAC's slot as "on-policy and less sample-efficient for long-horizon tuning".
- **D2 — Python with dual NumPy + TensorFlow stacks, but only NumPy wired in.** A 37.7 KB `population_evaluator_tf.py`, 5.5 KB `feedforward_tf.py`, and 2.6 KB `nn_agent_tf.py` ship as unused dead code. NumPy is fast enough for `ThreadPoolExecutor`-parallel CPU rollouts; PyTorch is reserved for SAC because gradients + GPU genuinely matter there. The TF stack is kept as optionality for a future scale-up that has not arrived.
- **D3 — HybridEncoder (fovea + rays), not pure nearest-N.** Pure nearest-N (`VectorEncoder`, 35 dims) produced spin-lock turret policies — agents sat still and rotated, shooting passing asteroids. Fovea + rays redundancy gives precise aiming dynamics (fovea) plus coarse global situational awareness (rays) that forces the policy to care about what is approaching from outside the fovea.
- **D4 — ES uses TemporalStackEncoder; GA and NEAT do not.** ES needs temporal awareness because gradient estimation requires smooth fitness differences between samples. GA's tournament selection is noise-tolerant; NEAT's `FITNESS_STD_PENALTY_RATIO=1.0` is its noise-robustness lever. The price: 6.5× parameter-count gap between GA (1,227) and ES (7,995) on the same policy class — the single largest cross-method comparability confound.
- **D5 — Pareto-first selection for ES, scalar fitness for GA, scalar-for-reproduction + Pareto-for-display for NEAT.** ES's rank-based update naturally accommodates Pareto ordering and protects diverse skill profiles during the update. GA's tournament already mixes fitness + novelty + diversity so Pareto would be redundant. NEAT's Pareto-for-display gives humans "best-looking to watch" rather than highest scalar reward — a qualitative choice for interpretability.
- **D6 — CRN defaults: on for ES, off for GA, on for NEAT.** CRN is the necessary enabler for ES's antithetic gradient estimation (paired samples must see identical environments). GA's tournament selection is robust to independent-seed noise. NEAT combines CRN with the std-penalty for robustness.
- **D7 — Diagonal CMA-ES, not full covariance.** Full covariance is O(n^2) memory and O(n^3) update cost for the 7,995-param policy — infeasible on CPU. Diagonal is the sweet spot. LM-CMA (limited-memory full covariance) was deferred as a future option.
- **D8 — Single evaluator, dispatched to method-specific drivers.** `population_evaluator.py` (45 KB) shared by GA, ES, NEAT — each method injects an `agent_factory`. Per-method evaluators were rejected because they would triple code and introduce drift opportunities between methods (different seed derivation, different metric collection, different reward handling).
- **D9 — SAC has its own training loop, not evaluator-based.** Evolutionary methods run on a (generation, candidate, seed, step) timebase; SAC runs on an (env_step) timebase with replay buffer + continuous learning. Shoehorning one into the other would pollute both. The plans codify: "Existing GA/ES/NEAT training scripts must continue to run without modification."
- **D10 — NEAT has `turn_deadzone=0.03`, others 0.0.** NEAT genomes with newly-added output nodes have sigmoid outputs that drift around 0.5 but don't hit it exactly. A 0-deadzone interface maps any `!=0.5` to a turn, producing micro-turning policies. The 0.03 deadzone accommodates NEAT's structural plasticity. A subtle comparability confound — GA/ES micro-turn constantly; NEAT has a dead band.
- **D11 — Genetic Programming scaffolding, no implementation.** README lists GP as a paradigm and marks its roadmap items `[x]`. Zero GP code exists. The commit `1c55da4` "Mark roadmap items complete in README" checked off GP roadmap items without implementing them. This is the anti-puffing headline — the pitch is "5 paradigms benchmarked", reality is 4.
- **D12 — Plans treated as living design docs, not static spec.** The 178 KB `plans/` folder contains 9 files with "Current Implemented System", "In Progress / Partially Implemented", "Planned / Missing / To Be Changed", "Notes / Design Considerations", "Discarded / Obsolete / No Longer Relevant" sections. Plans drift from code in concrete ways (`POPULATION_SIZE` plan 10 vs code 15; `SEEDS_PER_AGENT` plan 5 vs code 20) — the risk of plan-as-living-doc.

## What is currently built

Four methods are end-to-end wired (GA, CMA-ES, NEAT, GNN-SAC) and each has produced a complete training run captured as `training_summary_*.md` reports (45–68 KB each) plus JSON exports (394 KB ES, 992 KB SAC, 4.9 MB GA, 8.5 MB NEAT). The shared evaluator, shared encoder family, shared reward preset (5 of 22 components active), shared analytics pipeline, and three of four methods sharing `population_evaluator.py` are all in place and operating. The development arc was: reward system iteration (Phase 1, Dec 2025), GA infrastructure + agent + analytics (Phase 2-3, early Jan 2026), ES with CMA-ES + Pareto + antithetic + CRN (Phase 4, mid-Jan 2026), NEAT + GNN-SAC landing the same day (Phase 5, 2026-01-20).

What is *not* built:

- Genetic Programming — zero code despite README claims and roadmap `[x]` marks.
- Parallel training dashboard — README describes "All 5 algorithms train simultaneously in separate game instances" with interactive sidebar UI; every dashboard roadmap item is `[ ]` unchecked.
- Genome / state / replay persistence for resuming long runs in GA, ES, and SAC (NEAT writes per-gen artefacts so partial resume is possible there).
- Cross-method comparison document — each method has its own summary but no unified comparison report exists.
- A dependency manifest — no `requirements.txt`, no `pyproject.toml`, no `requirements-rl.txt`, no `Pipfile`. Reproducing a SAC run requires reverse-engineering the right `torch + torch_geometric + torch_scatter/torch_sparse` versions.

Per-method compute reality (the comparability-relevant numbers):
- GA: 15 individuals × 20 seeds × up to 1,500 steps × 500 generations.
- ES: 100 candidates × 3 seeds × up to 1,500 steps × 500 generations, with antithetic pairs and top-5 re-evaluation.
- NEAT: 50 genomes × 5 seeds × up to 1,500 steps × 500 generations.
- SAC: 500,000 environment steps total — a different timebase entirely.

## Current state

Status: dormant. Last commit `1c55da4` 2026-02-22 ("Mark roadmap items complete in README"); the commit before, `75a566f`, was "ran the training again". Real development ceased after GNN-SAC landed on 2026-01-20 (followed by SAC diagnostics on 2026-01-23). 2026-04-24 LifeOS update describes the project as "dormant for ~2 months". Nothing is in flight; no `Work/` folder exists in the LifeOS source for this project.

## Gaps and known limitations

- **README claims five paradigms; code ships four.** Genetic Programming is the phantom — zero code, but README has `[x]` marks on five GP subfeatures including "Tree-based symbolic controllers", "Arithmetic and logic operators", "Parsimony pressure and bloat control", "Subtree crossover strategies", "Decision-logic visualisation".
- **Parallel training dashboard advertised but not built.** Every dashboard item in the roadmap is unchecked. No dashboard code exists.
- **Cross-method comparability confounds.** Parameter count (GA 1,227, ES 7,995, NEAT variable, SAC orders of magnitude larger); compute budget (GA 15×20=300 episodes/gen, ES 100×3=300, NEAT 50×5=250, SAC 500K env steps); CRN default (ES True, GA False, NEAT True); NEAT-only `FITNESS_STD_PENALTY_RATIO=1.0`; NEAT-only `turn_deadzone=0.03`; selection objective (GA scalar+novelty+diversity, ES Pareto, NEAT scalar-for-reproduction + Pareto-for-display); novelty/diversity scaling differs per method (post-score, pre-rank-shaping, selection-bonus). Any "method X beats method Y" claim is compromised until these are controlled.
- **Plan / code drift.** `plans/GENETIC_ALGORITHM.md` says `POPULATION_SIZE=10` and `SEEDS_PER_AGENT=5`; code says 15 and 20 (a 4× discrepancy in compute per generation). `plans/NEAT.md` says no XOR sanity test exists; `tests/test_neat_xor.py` is 16.5 KB. Reading plans without checking code produces a wrong model.
- **Dead TensorFlow stack.** ~47 KB across `population_evaluator_tf.py` (37.7 KB — almost as big as the live evaluator), `feedforward_tf.py` (5.5 KB), `nn_agent_tf.py` (2.6 KB), referenced by zero entry scripts.
- **Dead ES code.** `training/methods/evolution_strategies/driver.py` (17.7 KB classic ES, superseded by CMA-ES) and `fitness_shaping.py` (3 KB unused by CMA-ES driver). Plus AdamW, rank transformation, and elitism settings still in `ESConfig` that the CMA-ES driver does not read.
- **Missing dependency manifests.** No `requirements.txt`, no `pyproject.toml`, no `requirements-rl.txt`, no conda env file. Particularly painful for SAC where PyTorch Geometric backend deps (`torch_scatter`, `torch_sparse`) vary by CUDA vs CPU, Python version, and wheel availability.
- **15 MB of generated training artefacts checked into git.** `training_data_neat.json` (8.5 MB), `training_data.json` (4.9 MB), `training_data_sac.json` (992 KB), `training_data_es.json` (394 KB), four `training_summary_*.md` files (233 KB combined), `best_sac.pt` (623 KB), 50 NEAT generation artefacts (~1.5 MB).
- **Low test coverage.** 4 test files, ~59 KB. `test_kill_asteroid_reward.py` (14.7 KB) and `test_json_export_numpy_types.py` (1.9 KB) work. `test_ga_dimensions.py` (6.7 KB) is broken — references removed legacy modules under `ai_agents/neuroevolution/genetic_algorithm/*`. `test_neat_xor.py` (16.5 KB) status is unclear given the plan/file contradiction. Effective coverage roughly 2% of 141 source files. The 45 KB shared evaluator has no tests despite being the comparability anchor.
- **No checkpoint / resume for 3 of 4 methods.** GA population is memory-only; ES mean/sigma/covariance state is memory-only; SAC saves a best checkpoint but not the replay buffer or optimizer state. Only NEAT writes per-gen artefacts.
- **Broken `EnvironmentTracker.get_tick()`** references `game.time` which does not exist on game objects.
- **No encoder schema versioning.** Analytics has `SCHEMA_VERSION = "2.3"`; encoders do not. A change to ray count, fovea count, or normalisation bounds silently invalidates every previously trained genome / ES mean / SAC checkpoint.
- **No wrap-aware collision detection.** Collisions are resolved after positions wrap, so opposite-edge entities that are toroidally adjacent test as far apart in Euclidean space. Trained agents may learn to exploit the edge as a safe zone.
- **17 of 22 reward components have never been used in a published run.** They represent deliberate optionality but were committed without an attached experiment.
- **Analytics polish gaps.** Novelty/diversity scalars stored but not visualised; `generalization_grade` thresholds not documented in reports; evaluation seed not stored in exports; fresh-game ratio filtering bias (only `fitness_ratio > 0` summarised, hiding frequent failures).

## Direction (in-flight, not wishlist)

Nothing is in flight. The project is dormant. The LifeOS roadmap reflects what *would* make sense if work resumed — explicitly not a commitment to any of it. The plans aggregate to 240+ planned items against 42 actual commits, which the roadmap acknowledges as "research appetite, not commitment".

## Demonstrated skills

- Implements four ML optimisation paradigms from scratch on a shared substrate — Genetic Algorithm with BLX-alpha crossover and tournament selection, diagonal CMA-ES with antithetic + CRN + Pareto-front selection + restart, NEAT with full innovation tracking + speciation + adaptive compatibility threshold + topology growth, and GNN-backed SAC with twin critics + auto-entropy + AGC.
- Designs the comparability substrate that makes "single environment, multiple minds" a defensible claim — a 45 KB shared evaluator that GA, ES, and NEAT all dispatch through, a schema-versioned `TrainingAnalytics` pipeline with method-agnostic facade + per-method optional keys, a shared `ComposableRewardCalculator` preset, a shared encoder family.
- Implements `GATv2Conv`-based graph neural networks for variable-cardinality state inputs over a toroidal world, with bipartite asteroid → player edges, wrapped deltas in edge attributes, and `GraphNormalizer` running-stats normalisation.
- Implements Soft Actor-Critic with twin critics + Polyak target averaging at `tau=0.005`, auto-entropy temperature toward `TARGET_ENTROPY=-3.0`, asymmetric actor/critic LR for stability under GNN-noisy embeddings, Huber TD loss, AGC adaptive gradient clipping on top of global grad-norm clipping, and a graph-native replay buffer.
- Builds dual-mode game engines (windowed Arcade + headless parallel) over shared physics constants, with explicit collision radii to avoid sprite-texture coupling, ghost-target ray duplication for toroidal world geometry, and a parity-critical headless-mode bullet filtering fix that maintains windowed-headless determinism.
- Designs NSGA-II-style Pareto ranking with crowding distance over `[hits, time_alive, softmin_ttc]`, including a soft-min TTC aggregator that smooths the discontinuous hard-min and an accuracy guard that prevents single-shot perfect-accuracy gaming.
- Designs behavioural-novelty selection with 7D action-and-engagement behaviour vectors, kNN novelty scoring against population + archive, and Shannon-entropy reward diversity bonus over positive components only.
- Composes 22 reward components behind a single `ComposableRewardCalculator` interface and integrates per-step reward anatomy into analytics for reward-share, reward-entropy, and reward-dominance reporting.
- Designs a schema-versioned analytics pipeline that supports cross-method comparability — 30+ always-on keys, up to 100+ method-specific keys, 23 distribution lists per generation, fresh-game generalisation with letter-grade scoring, ASCII sparklines, per-section takeaways and inline glossary in markdown reports.
- Honest engineering judgement at the documentation layer — the LifeOS notes explicitly call out the "5 paradigms claimed, 4 shipped" anti-puffing gap, the parameter-count comparability confound, the plan/code drift in `POPULATION_SIZE` and `SEEDS_PER_AGENT`, dead TensorFlow code, and 15 MB of generated artefacts in git. The capacity to look at one's own project and write down what is broken or missing is itself a transferable engineering skill.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/AsteroidsAI/_Overview.md | 110 | "#project/asteroids-ai #lang/python #paradigm/evolutionary #paradigm/neat #paradigm/cma-es #paradigm/sac #ml/rl #ml/gnn #status/dormant" |
| Projects/AsteroidsAI/Architecture.md | 211 | "- [[AsteroidsAI/Roadmap]] — the planned parallel dashboard and its architectural implications" |
| Projects/AsteroidsAI/Decisions.md | 245 | "- [[AsteroidsAI/Roadmap]] — decisions deliberately deferred appear as roadmap items" |
| Projects/AsteroidsAI/Gaps.md | 222 | "- [[AsteroidsAI/Systems/Analytics Pipeline]] — analytics polish gaps" |
| Projects/AsteroidsAI/Roadmap.md | 253 | "- [[Projects/_Overview]] — AsteroidsAI relative to Caner's other active projects (all of which are higher priority as of 2026-04-24)" |
| Projects/AsteroidsAI/Systems/_Overview.md | 38 | "- [[Projects/AsteroidsAI/Roadmap]] — direction-of-travel" |
| Projects/AsteroidsAI/Systems/Analytics Pipeline.md | 235 | "- [[AsteroidsAI/Roadmap]] — analytics polish is much of the remaining in-repo roadmap" |
| Projects/AsteroidsAI/Systems/Evolution Strategies.md | 256 | "- [[AsteroidsAI/Roadmap]] — the Easy/Medium/Hard roadmap from the ES plan is the richest source of next-session ideas" |
| Projects/AsteroidsAI/Systems/GNN-SAC.md | 273 | "- [[NeuroDrive/_Overview]] — NeuroDrive's asymmetric PPO (actor 2x64, critic 2x128) is the sibling gradient-based RL in the vault; comparing SAC continuous control here with PPO continuous control there is a useful cross-project analogy" |
| Projects/AsteroidsAI/Systems/Game Engine.md | 168 | "- [[AsteroidsAI/Gaps]] — broken `get_tick()`, wrap-aware collision, unused arcade APIs" |
| Projects/AsteroidsAI/Systems/Genetic Algorithm.md | 200 | "- [[Vynapse/_Overview]] — Caner's Rust neuroevolution engine; solves similar fixed-topology evolutionary problem in a different language" |
| Projects/AsteroidsAI/Systems/NEAT.md | 207 | "- [[Vynapse/_Overview]] — Vynapse's `trainers/neat.rs` is a 0-byte stub; AsteroidsAI's NEAT is the working reference implementation" |
| Projects/AsteroidsAI/Systems/Reward System.md | 152 | "- [[AsteroidsAI/Gaps]] — 17 components never exercised in a run" |
| Projects/AsteroidsAI/Systems/Shared Components.md | 213 | "- [[AsteroidsAI/Gaps]] — method-parity normalisation not done; cross-method bonus magnitudes not comparable" |
| Projects/AsteroidsAI/Systems/State Encoders.md | 217 | "- [[AsteroidsAI/Gaps]] — encoder drift, schema versioning, VectorEncoder dead code" |
