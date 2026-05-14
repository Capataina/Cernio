---
name: NeuroDrive
status: active
source_repo: https://github.com/Capataina/NeuroDrive
lifeos_folder: Projects/NeuroDrive
last_synced: 2026-05-13
sources_read: 33
---

# NeuroDrive

## One-line summary

Rust + Bevy 2D autonomous-driving simulator at fixed 60 Hz used as a testbed for a from-scratch biologically-inspired learner — sparse directed graph of rate-coded tanh neurons trained by three-factor plasticity with per-car eligibility traces, raw-reward modulator, Turrigiano-style homeostasis and continual-backprop structural plasticity — running alongside a handwritten PPO baseline for side-by-side diagnostic comparison, with zero external ML framework dependencies.

## What it is

NeuroDrive is a disciplined first-principles investigation into whether biological learning rules can produce coherent real-time control behaviour without backpropagation, without weight resets, and without any external ML framework. The environment is a deterministic 60 Hz Monaco-inspired 14×9 tile grid with kinematic car physics, a 43-dimensional normalised observation vector and a 2-dimensional `(steering, throttle)` action; that environment hosts two learners side by side. The first is a handwritten PPO baseline (asymmetric 2×64 actor / 2×128 critic, GAE, clipped surrogate, AdamW on the critic, PopArt, dual GEMM backend) which exists as the *permanent diagnostic baseline* — not a placeholder. The second is the brain-inspired learner that shipped as Milestone 6: a sparse directed graph of neurons learning continuously across one lifetime via local rules. The research question is whether the *mechanism* (local synaptic plasticity, neuromodulatory broadcast, no global error signal, continuous online learning) is sufficient for a non-trivial continuous control task on a MacBook Air CPU. The biology-first principle is load-bearing — when a pathology appears, the prescribed response is to research biology, never to fall back on an ML-toolkit default such as dropout, batchnorm, experience replay or EWC.

## Architecture

The project is structured as seven Bevy plugins plus a `sim` coordination module, hard-partitioned by a controller-agnostic boundary between `game/` and `brain/`.

```
sim
├── GamePlugin
│   ├── MonacoPlugin   (TrackGrid 14×9 tiles, TrackCenterline with arc-length parameterisation, O(1) collision)
│   └── AgentPlugin    (kinematic car, EnvInstanceId tagging, SensorReadings, ObservationVector, ActionState)
├── BrainPlugin
│   ├── PpoPlugin              (handwritten PPO — diagnostic baseline)
│   └── BrainInspiredPlugin    (M6 substrate)
├── AnalyticsPlugin    (passive — reads from game + brain; exports JSON + 10-section diagnostic markdown)
├── DebugPlugin        (HUD F3, F4 layout cycle, raycast / centreline / crash overlays; passive)
└── ProfilingPlugin    (feature-gated; 17 instrumented systems, ring-buffer FrameRecord, JSON + markdown reports)
```

`GamePlugin` has no compile-time dependency on `BrainPlugin` — `use crate::brain::*` does not appear anywhere in `src/game/`. The boundary is mediated by two shared interface types: `ObservationVector` (`[f32; 43]`, written by `build_observation_vector_system`, read by either learner in the next tick's `SimSet::Input`) and `ActionState` (`{ steering: f32 in [-1,1], throttle: f32 in [0,1] }`, written by the active learner, read by `car_physics_system`).

The simulation tick at 60 Hz uses Bevy's `FixedUpdate` schedule partitioned into four ordered `SimSet` stages:

```
FixedUpdate (60 Hz)
├── SimSet::Input
│   ├── keyboard_action_input_system    (1 keyboard car, optional)
│   ├── ppo_act_all_cars_system         (With<Car>, With<PpoCar>)
│   ├── brain_act_all_cars_system       (With<Car>, With<BrainCar>)
│   └── action_smoothing_system         (EMA blend with previous ActionState)
├── SimSet::Physics
│   └── car_physics_system              (pure kinematic: yaw rate, thrust, drag, integration)
├── SimSet::Collision
│   └── collision_detection_system      (rotated rect's 4 corners vs TrackGrid bool — O(1) per car)
└── SimSet::Measurement
    ├── update_track_progress_system    (arc-length projection, lateral offset, heading error)
    ├── update_sensor_readings_system   (11 fixed-angle raycasts up to 375 units, grid-march + binary refinement)
    ├── build_observation_vector_system (assembles + normalises 43-dim ObservationVector)
    ├── episode_loop_system             (crash event OR 30 s timeout → log + random centreline respawn)
    ├── ppo_collect_rewards_all_cars    (per-tick reward, append to per-car rollout entries)
    ├── ppo_epoch_system                (amortised — 64 samples per tick over 4 epochs when buffer full)
    ├── brain_learn_all_cars_system     (plasticity tick + homeostat per-tick + on-cadence structural plasticity)
    └── analytics + HUD capture systems
```

The causal ordering Input → Physics → Collision → Measurement is strict and enforced by `SimSet` chaining; observation is built *after* physics so it reflects the result of this tick's action. Forward-pass observation lag is one tick (~16 ms), which is structurally unavoidable in a single-pass schedule and biologically reasonable (real neurons have non-zero integration time constants).

The fleet composition is governed by a `TrainerLayout` enum carried on `TrainerConfig`:

```rust
enum TrainerLayout {
    Keyboard,                       // 1 keyboard car
    AllPpo    { count: usize },     // default 8 cars
    AllBrain  { count: usize },     // default 8 cars; this is the default on first boot
    SideBySide { ppo: usize, brain: usize },  // default 8 + 8 = 16 cars
}
```

Each car entity carries exactly one ZST marker component (`PpoCar`, `BrainCar` or `KeyboardCar`) and a matching `Controller` enum variant for analytics tagging. Systems filter by marker via Bevy query filters (`With<PpoCar>`, `With<BrainCar>`), so cross-contamination between learners is a compile-time impossibility — no runtime check needed. F4 cycles `AllBrain → SideBySide → AllPpo → AllBrain`, despawning every `Car` entity and respawning a fresh fleet on each transition (state-leak avoidance — markers are not swapped on existing entities).

## Subsystems and components

### Environment (`MonacoPlugin` + `AgentPlugin`)

Monaco-inspired 14×9 tile grid, 100 px per tile, 1400×900 px world. `TrackGrid` is a flat `bool` array with O(1) `(tile_x, tile_y) → on_road` lookup. `TrackCenterline` is an ordered point sequence with cumulative arc-length per point, supporting: projection (find nearest centreline point, return `(arc_length, lateral_offset, tangent_angle)`), progress tracking (monotonic arc-length integration), and lookahead sampling at 12 distances `[30, 60, 95, 135, 180, 230, 285, 345, 415, 490, 570, 650]` world units returning `(heading_delta, curvature)` at each. Car kinematics are pure: `heading += -steering · rotation_speed · dt; velocity += forward · thrust · throttle · dt; velocity *= drag (0.985); position += velocity · dt`. `rotation_speed = 8.0 rad/s` (raised from 4.0 because 4.0 was insufficient for tight corners at learned speeds). Determinism is verified by a 1200-step replay unit test producing bitwise-identical position / velocity / heading given seed + action stream.

### Observation Vector (43 dims, normalised)

| Range | Feature | Norm |
|---|---|---|
| 0–10 | 11 raycast distances at `[-150°, -90°, -60°, -35°, -15°, 0°, +15°, +35°, +60°, +90°, +150°]` | ÷ 375 → [0, 1] |
| 11 | v_forward (car-local) | normalised → [-1, 1] |
| 12 | v_lateral (car-local) | normalised → [-1, 1] |
| 13 | signed lateral offset from centreline | ÷ 75, clamped → [-1, 1] |
| 14 | signed heading error vs centreline tangent | ÷ π → [-1, 1] |
| 15 | yaw angular velocity | ÷ 8.0 rad/s → [-1, 1] |
| 16 | speed delta (frame-to-frame Δ) | normalised → [-1, 1] |
| 17–28 | 12 lookahead heading deltas | ÷ π → [-1, 1] |
| 29–40 | 12 lookahead curvatures | ÷ 0.05 → [-1, 1] |
| 41 | previous steering | already in [-1, 1] |
| 42 | previous throttle | already in [0, 1] |

This 43-dim contract is consumed by both learners and is RESEARCH-ANCHORED in `BrainInspiredConfig` (each obs dim binds to a reserved `Input(i)` neuron). The vector grew from an original 23 dims; the lookahead expansion (4 → 12 samples, far point 260 → 650 units) was the largest single observability upgrade.

### Reward function (`Episode and Reward`)

Per-tick reward is `r_velocity + r_centerline + r_time` where `r_velocity = dot(velocity, centerline_tangent) / 200 · scale`, `r_centerline = 0.3 · (1 - (|lateral_offset|/50)²)` (quadratic, soft near zero), `r_time = -0.005`. Crash penalty is deliberately **zero** — the agent avoids crashing only because crashing terminates the reward stream, not because crashing is punished. Lap-completion bonus was removed (created reward-cliff pathologies); episodes terminate only on collision or 30-second timeout.

### PPO Implementation (handwritten, no external ML libs)

Asymmetric actor-critic. Actor: `43 → 64 → 64 → 2` with `tanh` hidden activations, learned `log_std` per action dim (initialised `[0.0, 0.0]`, floor-clamped at `-1.0` so σ ≥ ≈0.37 — raised from -2.0 because throttle exploration was collapsing to σ ≈ 0.07). Critic: `43 → 128 → 128 → 1` — wider than the actor because the symmetric 2×64 critic reached 40.6 % tanh saturation with weight norms of 19.3 and couldn't discriminate crash-state vs safe-state values. Weight storage is a flat row-major `Vec<f32>` (replaced `Vec<Vec<f32>>` for cache locality). Orthogonal init scaled √2 on hidden layers, **0.01× scaling on the actor mean output** for near-zero exploratory initial policy. Tanh squashing on actions with explicit Jacobian correction `log π(a|s) = log N(u|μ,σ²) - Σ log(1 - a²)` for numerical correctness of the PPO ratio; throttle remapped `(tanh+1)/2 → [0, 1]`. GAE with `γ = 0.995` (raised in M5 from 0.99 for longer credit horizon) and `λ = 0.95`. Clipped surrogate `ε = 0.2`, 4 PPO epochs, 64 samples/tick amortised (reduced from 128 to smooth stutter). Actor optimiser: Adam LR `3e-4`. Critic optimiser: **AdamW with decoupled weight decay λ = 3e-4** specifically to bound critic weight growth. Adam ε = `1e-5` (raised from 1e-8 for stability). Gradient clip L2-norm 0.5 on each network separately. PopArt adaptive value normalisation with `popart_beta = 3e-2` (raised from 1e-4 to keep tracking real returns). Observation normalisation via running stats. Target-KL early stop. Three forward paths exposed (`forward_actor`, `forward_critic`, `forward`) so action selection skips the critic when not training. Per-system pre-allocated `BatchScratch` (sized for max-512 batch) eliminates training-loop heap allocations. Dual GEMM backend: SIMD scalar fallback + Accelerate AMX for batched mat-mat. Validation `run_1776556719.md` shows 8/8 cars completing the full Monaco loop, fleet max-progress spread 1.1 %, crash rate falling 100 % → 56 % in the best chunk, 96 % of crashes had throttle released > 0.25 s pre-impact.

### Brain-Inspired Learner (M6 substrate)

A sparse directed graph of rate-coded tanh neurons trained by local rules. 43 `Input(i)` neurons (one per observation dim) + 15 `Hidden` neurons by default + 2 `Output(i)` neurons (steering idx 0, throttle idx 1). Seed-graph edge density `initial_edge_density = 0.10`; weights from `Normal(0, σ=0.1)`. Legal edges: input→hidden, input→output, hidden→hidden (cyclic allowed, no self-loops), hidden→output; outputs do not project backwards; inputs have no incoming.

Graph storage is **slot-stable** — `Vec<Neuron>` and `Vec<Synapse>` with `alive: bool` flags and `free_neuron_slots`/`free_synapse_slots` free-lists. Killing a neuron is an O(1) flag flip; `NeuronId`/`SynapseId` (both `u32` for cache packing) are stable for the lifetime of the entity. Per-car eligibility lives inline on each `Synapse` as `eligibility: Vec<f32>` indexed by `EnvInstanceId`; weights are shared across all cars.

The forward pass is one-step propagation with a `prev`/`curr` activation buffer rotation per tick: `curr ← prev; curr[inputs] = obs[i] directly; for every live non-input neuron j: z = bias + Σ prev[source] · weight, curr[j] = tanh(z)`. Order-independence is guaranteed because all reads come from `prev`, all writes go to `curr` — cyclic connections are trivially well-defined with a one-tick propagation delay (biologically realistic given real neurons' integration time constants).

Plasticity is the three-factor rule: `Δw_ij[c] = η · M_c · e_ij[c]; w_ij += Σ_c Δw_ij[c] / divisor` per car c, accumulated then applied at the end of the tick so every car sees the same weights on its forward pass. The eligibility trace evolves as `e_ij[c] ← λ · e_ij[c] + pre_i · post_j` with `pre = prev[source]` (t-1) and `post = curr[target]` (t) — the tick-shift gives STDP-like causal pre-before-post semantics without sub-tick scheduling. Defaults: `λ = 0.992` (RESEARCH-ANCHORED → τ_e ≈ 2 s at 60 Hz, matching γ=0.995's credit horizon), `η = 1e-3` (TUNE), `M_c = EpisodeState.tick.reward` (Option C — raw per-tick reward, no critic, no TD-error δ). On episode terminal for car c, every synapse's `eligibility[c]` is zeroed across the whole graph so stale correlations cannot bleed across resets.

Homeostasis is two biologically-distinct mechanisms on different cadences. **Intrinsic excitability (per tick)** for every non-input live neuron j: `mean_rate_j ← (1-α) · mean_rate_j + α · mean_c(|curr[j][c]|)` with `α = 0.01`; if `mean_rate_j < lo_band (0.10)` then `bias_j += 1e-4`; if `> hi_band (0.60)` then `bias_j -= 1e-4`. `age_ticks` is incremented in this loop too (NOT in the structural pass), which is what couples the CBP maturity gate to homeostasis. **Synaptic scaling (every 128 ticks)** per non-input neuron: `s = Σ_incoming |w_ij|; factor = clamp(1 + 0.05 · (target=2.0 - s)/target, 0.5, 2.0); w_ij *= factor for all incoming`. The `[0.5, 2.0]` clamp prevents shock corrections; multiple cadence passes converge smoothly.

Structural plasticity adapts the topology via four operations. **CBP utility EMA (per tick)** `u_i ← η_u · u_i + (1-η_u) · h_i · outgoing_sum[i]` with `η_u = 0.99` (RESEARCH-ANCHORED, Dohare et al. 2024 §Rank 1), where `h_i = mean_c(|curr[i][c]|)` and `outgoing_sum[i] = Σ_{live syn: source==i} |w|`. **Replacement (on cadence)** picks the `ceil(replace_fraction=5e-4 · hidden_count)` lowest-utility mature (`age_ticks ≥ maturity_ticks = 1000`) hidden neurons; for each: zero outgoing weights (behaviour-preserving), resample incoming from `Normal(0, 0.1)`, zero eligibility across all cars on both incoming and outgoing edges, reset `utility=0, age_ticks=0, mean_rate=0, bias=0`. **Plateau-triggered neurogenesis (on cadence)**: when `detect_plateau` sees `|mean(second_half) - mean(first_half)|/|mean(first_half)| < plateau_threshold (0.02)` over `plateau_episode_window = 50` episodes, `grow_hidden_neuron` allocates a new `Hidden` slot with ~10 random legal incoming + ~10 random legal outgoing edges, then `brain.reward_window.clear()` so a sustained plateau doesn't re-trigger every cadence. **Prune (on cadence)**: synapses with `|weight| < prune_weight_threshold = 0.01` are marked dead, weight + eligibility zeroed, adjacency lists updated, slot returned to free-list. **Sprout (on cadence)** with probability `sprout_probability = 0.10`: sample `sprout_candidates_per_event = 8` random (source, target) pairs and add a new synapse from `Normal(0, 0.1)` weight for each unconnected legal pair. Ablation flags `enable_plasticity`, `enable_homeostasis`, `enable_structural` are all true by default and can be toggled at runtime without recompiling.

### Analytics and Export

Passive plugin. Two-tier JSON export on `AppExit` (in the `Last` schedule): `reports/json/analytics/run_<timestamp>.json` (always — `CompactRunExport` = `RunMetadata` + `EpisodeRecord[]` + `PpoUpdateRecord[]` + `BrainUpdateRecord[]`, no per-tick traces) and optionally `run_<timestamp>_traces.json` (full per-tick traces, gated by `AnalyticsConfig.full_trace_export`). Both directories auto-enforce a retention limit (oldest deleted via `enforce_retention`). 10-section diagnostic markdown report at `reports/analytics/run_<timestamp>_<slug>.md` where slug ∈ `{brain, side, ppo, keyboard}` (layout-aware naming, Decision D23). Sections: Run Summary, Learning Progress (Unicode sparklines + 10-chunk trend), Action Behaviour, Speed & Momentum, Crash Forensics (`Slide/HeadOn/Overshoot/Spin/Stall` classification + heatmap by sector), What Does the Car Think (value-function evolution), Track Coverage (per-sector consistency), Driving Quality (per-car), Training Health (PPO entropy/clip%/KL/EV sparklines), Trajectory Snapshots. M6 added §16/17/18 for brain-inspired (`BrainUpdateRecord` per cadence carries: neuron count, hidden count, synapse count, mean |w|, weight σ, mean |e|, utility p10/p90, replacement/neurogenesis/prune/sprout counts per-window not cumulative, dead-neuron fraction, saturation fraction, mean M) and §19 Fleet Comparison that auto-detects side-by-side from `Controller::Ppo`/`Controller::Brain` tags on episodes. PPO-centric sections (9, 12, 13, 14) skip entirely in brain-only runs; brain sections skip when `brain_records` empty. `EpisodeTracker.brain_records` and `CompactRunExport.brain_records` use `#[serde(default)]` for pre-M6 JSON back-compat. 25 unit tests across `stats`/`chunking`/`timeseries`/`diagnostics`/`consistency`/`phases`/`sparkline`/`turns` modules.

### Profiling System (feature-gated)

`cargo run --features profiling`. Activates per-system timing across all 17 FixedUpdate systems via an `instrument!()` macro that inserts `start_timer("Name")` `.before()` and `stop_timer("Name")` `.after()` each target system. Boundary markers `frame_start_system`, `input_end_system`, `physics_end_system`, `collision_end_system`, `frame_end_system` capture per-SimSet boundaries. `FrameTimings` is a ring buffer of `FrameRecord` (default 1800 frames = 30 s at 60 Hz). Auto-exit after a configurable duration. Two-tier export: JSON (`reports/json/performance/perf_<timestamp>.json`) + 9-section explanatory markdown (`reports/performance/perf_<timestamp>.md`) including frame budget, percentile distribution, pipeline breakdown, per-system detail, stutter analysis with worst-5 ticks, buffer/memory pressure, and auto-generated optimisation recommendations.

### Trainer Layouts + F4 Cycling

ZST markers (`PpoCar`, `BrainCar`, `KeyboardCar`) replace the pre-M6 global `AgentMode` enum (Decision D1). Side-by-side mode `SideBySide { ppo: 8, brain: 8 }` puts 16 cars on the track at ~9 % frame budget (warm palette for PPO, cool palette for brain — distinctness covered by `warm_and_cool_palettes_are_visually_distinct` test). F4 cycle is `AllBrain → SideBySide → AllPpo → AllBrain` (excludes Keyboard, Decision D22). Default first-boot layout is `AllBrain { count: 8 }` (Decision D21 — reflects the project thesis, not the diagnostic baseline). F4 transitions despawn every `Car` entity and reset PPO + brain state from scratch, then respawn the fleet (Decision D3 — avoids state leak from marker swaps).

## Technologies and concepts demonstrated

### Languages

- **Rust** — entire codebase. 80 `.rs` files, 660 KB across `src/` plus 3 integration-test files at 45 KB (`brain_inspired_pipeline.rs` is the largest at 29.8 KB). Brain-inspired module is 7 files under `src/brain/inspired/` (~50 KB source, ~30 KB tests).

### Frameworks and libraries

- **Bevy** — ECS game engine. Uses `FixedUpdate` schedule with custom `SimSet` enum (`Input → Physics → Collision → Measurement`) for the simulation loop; standard `Update` schedule for HUD/overlay rendering (decoupled from simulation). Plugin composition across 7 plugins; component-marker queries (`With<PpoCar>`, `With<BrainCar>`) used as compile-time partitioning; `Resource` for global state (`TrainerConfig`, `BrainBrain`, `PpoBrain`, `EpisodeConfig`); events for cross-plugin signalling (F4 toggle → buffer clear). Bevy gizmos used for raycast / centreline / crash-marker overlays.
- **rand / rand_distr** — `StdRng` seeded per-brain (`config.rng_seed` or `rand::rng()`); `Normal(0, σ)` for synapse-weight init; `Uniform` for spawn point + sprout candidate sampling.
- **serde + serde_json** — custom struct serialisation for `CompactRunExport`, `BrainUpdateRecord`, `PpoUpdateRecord`, `EpisodeRecord`, `RunMetadata`, `RunContext`. `#[serde(default)]` on M6 additions for pre-M6 JSON back-compat (Decision D18).
- **Accelerate (Apple)** — used at the GEMM-backend layer via FFI for AMX (Apple Matrix coprocessor) mat-mat in the PPO update path, behind a dual-backend abstraction with a SIMD fallback that still passes `force-scalar` test builds.

### Runtimes / engines / platforms

- **Bevy ECS** — the only runtime layer. No tokio, no async runtime — the entire simulation is synchronous fixed-step. Bevy's component archetype storage is what makes `With<PpoCar>` query filtering O(1) per archetype rather than per car.
- **macOS / Apple Silicon (M2)** — primary development and target hardware. Apple AMX coprocessor exposed via Accelerate is the GEMM backend for batched mat-mat work in PPO updates. ARM64 NEON SIMD (not SSE/AVX) is the fallback path.

### Tools

- **cargo + standard Rust toolchain** — `cargo build`, `cargo test` (133 green tests across default, `force-scalar`, and release builds — 101 unit + 21 brain-pipeline + 6 GEMM + 5 PPO), `cargo run --features profiling`.
- **Custom profiling system** — feature-gated, ring-buffered, per-system timing via macro-instrumented `start_timer`/`stop_timer` systems. Auto-exit + JSON + 9-section markdown report. Built to diagnose the 426-stutter problem that led to the M4 performance overhaul.
- **In-house analytics + markdown report generation** — `src/analytics/exporters/markdown.rs` (66 KB) generates the 10-section diagnostic report with ASCII sparklines (▁▂▃▄▅▆▇█), horizontal bar charts, sector heatmaps, automated diagnostic flags, learning-phase detection (Exploration → Discovery → Refinement → Plateau → Regression), and per-section auto-generated takeaway sentences.

### Domains and concepts

- **Reinforcement learning from first principles** — handwritten PPO including: actor-critic decomposition; GAE `δ_t = r_t + γ V(s_{t+1}) - V(s_t); A_t = Σ (γλ)^l δ_{t+l}` with γ=0.995, λ=0.95; clipped surrogate `min(r_t · A_t, clip(r_t, 1-ε, 1+ε) · A_t)`; ε=0.2; tanh-squashed actions with Jacobian-corrected log-probability for unbiased ratios; PopArt adaptive critic-target normalisation; target-KL early stop; observation normalisation via running stats; per-car bootstrap at rollout horizon; on-policy with importance-sampled ratio (F4 buffer clear on keyboard transition).
- **Biologically-plausible learning** — three-factor learning rule `Δw = η · M · e` as global broadcast × local trace × global learning rate, contrasted explicitly with backprop's `∂L/∂w_ij`. Eligibility traces as the synaptic-level analog of TD(λ) at the state-action level (`e_ij ← λ e_ij + pre · post` versus `e(s,a) ← γλ e(s,a) + 1`). Hebbian co-activity with rate-coded signed tanh activations giving LTP and LTD from one rule. Neuromodulation theory — dopamine as reward prediction error (Schultz et al. 1997), explicitly *not* implemented in v1; v1 uses raw per-tick reward as the Option-C starting point. STDP theory (LIF neuron model, asymmetric timing window, NMDA coincidence detection, `Δw = A+ · exp(-Δt/τ+)` for pre-before-post LTP and `Δw = -A- · exp(Δt/τ-)` for post-before-pre LTD) and the rate-coding approximation that makes pre-from-`prev` / post-from-`curr` give STDP-like causal semantics without sub-tick spike scheduling.
- **Homeostatic plasticity (Turrigiano synaptic scaling + Marder intrinsic excitability)** — biologically grounded counter to Hebbian runaway, used *instead of* Oja's rule, weight decay, dropout, or batch normalisation. Per-neuron multiplicative scaling of incoming weight sums + per-neuron bias nudges to keep `mean_rate` within `(0.10, 0.60)`. Bounded tanh activation + structural prune as the third leg of the stability stool.
- **Continual learning + structural plasticity** — Continual Backprop (Dohare et al. 2024) adapted from layered MLPs to a sparse graph. Rank-1 utility metric `u_i = |h_i| · Σ|w_out|`. Outgoing-zero replacement protocol (behaviour-preserving at the moment of replacement, then resample incoming). One-brain-one-lifetime constraint as the explicit framing — no weight resets at episode boundaries, no population methods, no generational switching, no external ML frameworks. Catastrophic forgetting positioned as the central future challenge for M10.
- **Vectorised-environment training** — 8 cars sharing one policy via `EnvInstanceId` tagging; shared rollout buffer with per-car bootstrap at horizon; 8× wall-clock data rate without changing per-transition sample efficiency. Brain-inspired variant: one shared `BrainGraph`, per-car eligibility `Vec<f32>` per synapse, per-car activations `NeuronActivations` Component, summed weight updates across cars (Decision D6).
- **ECS architectural discipline** — controller-agnostic boundary (compile-time `use crate::brain::*` not allowed in `src/game/`); shared interface types (`ObservationVector`, `ActionState`) as the only cross-plugin contract; passive plugins (`AnalyticsPlugin`, `DebugPlugin`, `ProfilingPlugin`) that read but never write simulation state.
- **Performance engineering on constrained hardware** — 16.67 ms frame budget at 60 Hz on M2 MacBook Air (8 GB unified memory shared between CPU and GPU). Stutter reduction 426 → 2 and mean frame time 17.3 ms → 9.0 ms via flat row-major weight storage (replaces `Vec<Vec<f32>>` cache-miss pattern), pre-allocated `BatchScratch` buffers (zero training-loop allocations), batched mat-mat operations for forward + backward, iterator-based inner loops for LLVM auto-vectorisation, `std::mem::swap` instead of `clone` for the frozen rollout buffer, and per-step Adam bias-correction precomputation. M4 overhaul delivered 21× frame-time improvement via dual GEMM backend + batched actor.
- **Determinism + reproducibility** — fixed 60 Hz tick, deterministic LCG-based replay unit test (`deterministic_replay_same_seed_same_actions_identical_trajectory`), seeded `StdRng` for policy sampling, layered determinism map (strong physics + ECS schedule; weak controller init seed — flagged as open gap).
- **Diagnostic instrumentation as a first-class engineering output** — the analytics + profiling stack is treated as a deliverable alongside the simulation, with the markdown reports designed to be readable by someone unfamiliar with the codebase. Crash classification with five-category taxonomy (`Slide`, `HeadOn`, `Overshoot`, `Spin`, `Stall`), turn-execution diagnostics with failure-mode classification, 7 automated diagnostic flags (entropy collapse, clip-fraction excursion, KL spike, plateau, action collapse, crash-rate spike, value drift), per-sector behavioural consistency, learning-phase detection.

## Key technical decisions

The LifeOS `Decisions/` folder captures 23 numbered implementation decisions (D1–D23) made during the six-stage M6 rollout, plus seven cross-cutting design decisions. Highlights:

- **Biology First Principle** — load-bearing project thesis. When the brain-inspired learner hits a pathology, the prescribed response is to consult biology, not ML. Dropout, batch norm, experience replay, Elastic Weight Consolidation, NEAT, Oja's rule and weight decay are all explicitly rejected as defaults; their direct biological analogues (synaptic scaling, intrinsic excitability, sleep/replay, structural plasticity, homeostasis, bounded activation) replace them. Hybrid-by-default ("biology where clean, ML where needed") was rejected because "needed" collapses to "convenient" in practice. Pragmatic-default-ML was rejected because each ML shortcut erodes the thesis. The principle is binary: biology-first or not — partial retreat is abandonment.
- **One Brain, One Lifetime** — no weight resets between episodes, no population methods, no generational switching, no external ML frameworks, no backpropagation in the brain-inspired learner (PPO keeps backprop because it is the diagnostic baseline). Catastrophic forgetting is repositioned from "failure mode to avoid" to "central challenge to solve". PPO inside a session also respects within-session continuity. Save/load across sessions is currently an open gap.
- **PPO as Permanent Diagnostic Baseline** (updated 2026-04-19) — pre-research-round plan was that PPO would be retired after Milestone 2 validated the environment; post-research-round plan is that PPO stays permanently live, with the brain-inspired learner *additive* via side-by-side mode. Retiring PPO would remove the known-working reference against which the brain-inspired learner's behaviour can be measured. Side-by-side `SideBySide { ppo: 8, brain: 8 }` is a core feature, not transitional.
- **Raw Reward as Modulator (Option C)** — explicitly chosen for M6 from four options: A (reuse PPO's GAE δ — *rejected* because it would make the brain-inspired learner depend on a backprop-trained component, silently killing the thesis), B (plasticity-trained value predictor — *deferred to M8* as the legitimate biological destination), C (raw per-tick reward — *chosen* as the smallest correct starting point), D (import PPO critic as M as a v1 rescue — *forbidden* by the failure-mode discipline if M6 doesn't learn). Option C is viable specifically because reward is dense per tick, λ=0.992 (τ_e ≈ 2 s) matches the typical action-reward gap, and reward is non-negative so the dynamic is Hebbian-with-amplitude.
- **Graph Topology, Not Layered** — sparse directed graph with cyclic connections rather than a layered feedforward MLP. Three reasons: biological faithfulness (cortical microcircuits are graph-structured, layers are an ML convention), structural plasticity is natural on a graph (slot-flip vs matrix reshape), and the M7 brain-visualisation milestone is natively renderable on a graph but visually sterile on a stack of rectangles. Costs accepted: no automatic forward-pass order (solved by `prev`/`curr` rotation), worse cache locality vs dense matrix, cannot reuse PPO's GEMM kernels.
- **Slot-Stable Graph Storage** — `Vec<Neuron>` + `Vec<Synapse>` with `alive` flag + `free_*_slots` Vec, rather than compacting on death (O(n) shifts + downstream ID invalidation), `HashMap` (poor cache locality), or `Option<Neuron>` slots (`Option` discriminant overhead). `NeuronId`/`SynapseId` as `u32` (not `usize`, not newtype-wrapped) packs `Neuron.incoming/outgoing` and `BrainUpdateRecord` tighter.
- **Reward Design** — velocity-projection (`dot(v, tangent)/200`) over progress-fraction (incentivises speed in the right direction, naturally penalises oversteer/understeer), quadratic centreline-proximity bonus (soft near zero, hard at edges), small negative time penalty (prevents stalling without dominating), zero crash penalty (the agent avoids crashing because crashing terminates positive reward flow, not because crashing is punished — produces aggressive entertaining driving, not cautious driving), lap bonus removed (created reward cliff at finish/start line).
- **Tanh-Squashed Actions with Jacobian Correction** — chosen over clipping a Gaussian (discontinuous distribution, biased gradients), Beta distribution (complex entropy), or truncated Gaussian (expensive normalisation). `a = tanh(u)` where `u ~ N(μ, σ²)`; `log π(a|s) = log N(u|μ,σ²) - Σ log(1 - a²)` with numerical-stability handling near `|a| → 1`. Throttle remap `(tanh+1)/2 → [0, 1]`.
- **D6 — One shared graph + per-car eligibility + per-car activations** — 8 cars learn into one brain at 8× data rate without the per-car credit cross-pollution that summed-across-cars eligibility would cause and without losing "one brain, one lifetime" framing that 8 independent brains would.
- **D14 — Intrinsic homeostat per tick, synaptic scaling on cadence** — `age_ticks` advances in the per-tick intrinsic-homeostat pass, which is what makes CBP's maturity gate (`age_ticks ≥ 1000`) actually fire on a meaningful timescale. If both ran on cadence, the maturity gate would lag 128× and become decorative. This couples enable_homeostasis and enable_structural — disabling homeostasis effectively disables CBP replacement.
- **D15 — Outgoing-zero replacement** — when replacing a low-utility neuron, zero outgoing weights *first* (the neuron immediately stops contributing to downstream — behaviour-preserving) then resample incoming. Reversed order would cause a brief behaviour spike from random incoming weights propagating through stale outgoing weights.
- **D21 — Default layout is `AllBrain`, not `AllPpo`** — first boot reflects the project thesis. PPO-only and side-by-side are one and two F4 presses away.
- **D20 — No explicit cross-contamination tests** — the invariant "PPO doesn't see brain data, brain doesn't see PPO data" is enforced at compile time by ZST marker query filters. Explicit runtime tests would be testing the compiler, not a real failure mode.
- **LinkedIn Article + 3-GIF README Embed (shipped 2026-04-29)** — pre-M11 external-visibility layer. LinkedIn article published with 1280p MP4 cover (the article cover accepts video, contrary to earlier research). 3 GIFs encoded with `gifski 1.34 --fps 15 --width 720 --quality 90` (Mac-native, far better than ffmpeg's GIF encoder) committed to `media/` and embedded in the GitHub README. ~9.5 MB total across 3 GIFs, chosen at 10 s rather than 15 s for the lookahead+raycasts GIF specifically because 15 s would breach GitHub's 10 MB upload cap. Audience split is deliberate: general-audience LinkedIn cover gets the plain driving scene; technical viz overlays (lookahead + raycasts) land on the GitHub README for the engineer audience.

## What is currently built

- **M1–M5: complete.** Environment + 43-dim observation + 2-dim action + 30-second-or-crash episodes + random centreline respawn; PPO baseline (asymmetric 2×64 / 2×128, dual GEMM backend, PopArt, γ=0.995, target-KL stop) validated on Monaco — all 8 cars complete full track loops, fleet max-progress spread 1.1%, 96% of crashes anticipated by throttle release > 0.25 s before impact; 8-car vectorised training with full analytics pipeline (10-section markdown report, two-tier JSON export, five-category crash classification, per-sector consistency, turn-execution diagnostics, 7 automated diagnostic flags, learning-phase detection); M4 performance overhaul (dual GEMM backend, batched actor, centreline-query optimisations) delivering 21× frame-time improvement.
- **M6: code shipped 2026-04-19 across six staged commits.** Full brain-inspired substrate live in master and running alongside PPO. S1 plumbing: ZST markers + `TrainerLayout` enum + `BrainInspiredPlugin` registered + forward pass (no learning). S2 plasticity: three-factor rule with per-car eligibility traces + raw-reward modulator + accumulate-then-apply weight update + terminal eligibility reset. S3 homeostasis: per-tick intrinsic excitability + on-cadence synaptic scaling + age-ticks coupling. S4 structural plasticity: CBP utility EMA + replacement with outgoing-zero protocol + plateau-triggered neurogenesis + prune + sprout. S5 analytics: `BrainUpdateRecord` per cadence + 3 new markdown sections (§16, 17, 18) + `#[serde(default)]` back-compat. S6 side-by-side: `SideBySide { ppo: 8, brain: 8 }` layout + warm/cool palette + Fleet Comparison markdown section (§19) auto-detecting from controller tags + `Controller` enum for analytics tagging. Default first-boot layout set to `AllBrain { count: 8 }` reflecting project thesis.
- **133 green tests** across default, `force-scalar`, and release builds (101 unit + 21 brain-pipeline + 6 GEMM + 5 PPO). The 21 brain-pipeline integration tests cover every mechanic across all six M6 stages: seed graph I/O counts, forward-pass determinism with fixed seed, action-range invariants, eligibility-trace decay to zero with M=0, weight-update magnitude scaling with η, no-NaN/no-Inf over 10k ticks, terminal eligibility zeroing, synaptic scaling reaching target, intrinsic homeostat moving bias into band, homeostasis idempotent at steady state, replacement selecting lowest-utility mature neurons, replacement zeroing outgoing weights, replacement preserving connectivity invariants, plateau detector firing on flat reward windows, neurogenesis growing neuron count, utility-tick arithmetic, BrainUpdateRecord serde round-trip, CompactRunExport skipping brain_records when absent, TrainerLayout total-cars sum, F4 cycle visiting all four variants, warm/cool palette distinctness.
- **External-visibility layer shipped 2026-04-29**: LinkedIn article ("How I made a Rust ML simulator 40x faster on a MacBook Air, with no GPU") + 3-GIF README embed (`01-cars-driving-10s.gif`, `02-lookahead-points-10s.gif`, `03-lookahead-and-raycasts-10s.gif`) on the public GitHub repo.

## Current state

Status: **active**. HEAD `141be5b` at 2026-04-29 (LifeOS verified 2026-05-13). The only commits since the M6-ship period are docs-only — `141be5b` renaming `Neurodrive Media/` → `media/`, `11e0d45` embedding driving + observation overlay GIFs in the README. No source-code changes since 2026-04-19. The 24-day gap between M6 code shipping (2026-04-19) and 2026-05-13 is **not dormancy** — it is the planned pause between mechanical shipping and the first wall-clock SideBySide training run that constitutes the M6 behavioural acceptance bar. Active in-flight work files: a Continual ImageNet adapter (Permuted MNIST 200-task benchmark adapter, paper-grade write-up positioning the M6 substrate as a continual-learning method against MESU / RDBP / EWC / SI baselines) and a CubeCL Kernel Pack (port at-minimum 3 M6 primitives to CubeCL's CUDA + Metal + Vulkan + ROCm + WGPU surface plus an MLX operator pack for Apple-Silicon depth) — both gated on M6 behavioural validation first.

## Gaps and known limitations

- **First real SideBySide training run not yet done — gates M6 behavioural acceptance.** 21 brain-pipeline tests verify mechanics (eligibility decays, no NaN, terminal zeroing); none verify the brain produces meaningful driving behaviour. The M6 success bar — brain-fleet reward trend rising over ~2000 episodes in SideBySide, directional bias observable in analytics, at least one replacement + one neurogenesis event during the run — requires hours of wall-clock training time that has not yet been spent. This is the single most important outstanding piece of work; everything else is minor by comparison. "Honest negative result" (visible plasticity signature, no loop completion) is a valid M6 ship per the plan's stated success bar.
- **No brain persistence across sessions** (high severity, violates one-brain-one-lifetime at session level). Neither `PpoBrain` weights + AdamW state + PopArt tracking, nor `BrainBrain.graph` + `rng` state + `tick_counter` + `BrainTrainingStats.history` is serialised to disk. Every session begins fresh from random init. Especially costly for the brain-inspired learner because plasticity is slower per-episode than PPO (no batched gradient) and structural events fire only on cadence — a multi-hour training run lost to a laptop sleep is a real cost.
- **13 of 22 `BrainInspiredConfig` dials are TUNE** with no empirical backing. `eta` (1e-3), `replace_fraction` (5e-4), `structural_cadence` (128), `plateau_threshold` (0.02), `prune_weight_threshold` (0.01), `sprout_probability` (0.10), `synaptic_scaling_rate` (0.05), `intrinsic_bias_rate` (1e-4), `synaptic_scaling_target` (2.0), `intrinsic_rate_band` (0.10, 0.60), `plateau_episode_window` (50), `sum_per_car_updates` (true). The first SideBySide training run will produce the data for the first informed tuning sweep.
- **Sprout/prune balance is undocumented.** `sprout_probability = 0.10` per cadence and `prune_weight_threshold = 0.01` are individually reasonable, but no analysis cites that expected sprouts per cadence balance expected prunes under steady-state weight distributions. Symptoms of imbalance would be monotonically rising graph density (sprout > prune → wasted computation + growing memory) or collapsing density (prune > sprout → sparse broken network). The first training run should log `sprout_events / prune_events` per cadence.
- **HUD column split for side-by-side is report-level only.** Live HUD still shows single-column PPO stats; brain-fleet metrics have no live HUD presence during side-by-side play. Not gated on M6 acceptance.
- **No PPO behavioural-success integration tests beyond unit scope.** 21 brain-pipeline tests vs 5 PPO tests focused on GAE + update mechanics. No explicit threshold tests (does PPO converge on Monaco within N episodes?). Could become blocking if PPO regresses silently.
- **No multi-track support** (low now, blocks M10). Only Monaco. M10 (continual learning, transfer, curriculum) needs a second track + track-aware spawn logic + `TrackId` enum.
- **User-controllable init seed is partial.** `BrainBrain.rng` uses `config.rng_seed` if present, else `rand::rng()` — configurable but not exposed on the CLI. PPO's model init is seeded from runtime RNG but the init seed itself is not user-controllable. Full reproducibility needs a unified CLI seed surface.
- **Lap detection was deliberately removed.** Episodes end on crash or 30 s timeout only. Implication: the agent cannot distinguish "I completed a full lap" from "I drove for 30 s without crashing". Not blocking M6; may matter for M10 progress metrics. Re-adding it is a lightweight "arc-length passes start/end threshold" detector if needed.
- **8 GB unified-memory constraint is the dominant hardware fact.** M2 MacBook Air, CPU and GPU share one pool. Memory-intensive work (large rollout buffers, full per-tick trace captures) competes with rendering — no unbounded buffers anywhere, ring-buffer profiling, retention-limited analytics directories.

## Direction (in-flight, not wishlist)

- **First SideBySide training run** producing brain-fleet reward trend + directional-bias analytics + at least one replacement + one neurogenesis event — gating M6 behavioural acceptance.
- **M7 brain visualisation** — next milestone after M6 validates. Live graph inspector: neurons as dots (brightness = utility, size = outgoing degree), synapses as lines (thickness = |w|, colour = sign), per-tick activation animation, structural events visually surfaced (replacement, neurogenesis, prune, sprout), input-neuron heat-map showing which obs dimensions drive which parts of the network. The visual feedback loop is what makes empirical tuning of the 13 TUNE dials tractable — numbers alone are too thin for graph dynamics.
- **M8 plastic value predictor (Option B)** — only if v1's raw-reward Option C proves insufficient on the first behavioural run. Plasticity-trained `V(s)` producing a TD-error-analog modulator `M = r + γV(s') - V(s)`. Design questions deliberately deferred from M6: where does V live (subset of output neurons / parallel value sub-graph / separate linear readout); what rule trains it (same three-factor with self-supervision / standard TD); storage sharing with main graph.
- **Continual ImageNet Adapter (proposed)** — additive on top of M6, gated on SideBySide validation. Permuted MNIST 200-task adapter + paper-grade write-up positioning the M6 substrate as a continual-learning method, comparing against MESU + RDBP + standard EWC / Synaptic Intelligence baselines. Honest framing: substrate may underperform standard CL methods on image classification because RL-substrate-design choices may not transfer cleanly; negative result is publishable and durable.
- **CubeCL Kernel Pack (proposed)** — additive infrastructure track. Port at-minimum 3 M6 primitives (sparse graph forward, three-factor plasticity, structural plasticity ops) to Tracel AI's CubeCL surface — CUDA + Metal + Vulkan + ROCm + WGPU. CPU paths stay as correctness oracle. Parallel MLX operator pack for Apple-Silicon depth (M5 Neural Accelerators 19-27% boost). Target 5-20× speedup vs current AMX-GEMM CPU on M2 GPU.

## Demonstrated skills

- **Building a complete reinforcement-learning training environment from scratch in Rust with zero external ML framework dependencies** — handwritten PPO including asymmetric actor-critic, GAE, clipped surrogate objective, tanh-squashed actions with explicit Jacobian-correction, PopArt, AdamW decoupled weight decay applied selectively to the critic, observation normalisation, target-KL early stop, orthogonal init with scale-differentiated heads (0.01× on actor mean output for exploratory initial policy), all from primitive Rust + no PyTorch / TensorFlow / JAX / candle / tch-rs / ndarray-with-autograd. The PPO baseline is validated empirically: 8/8 cars complete the full Monaco track loop, fleet max-progress spread 1.1%, 96% of crashes anticipated by pre-impact throttle release > 0.25 s.
- **Implementing biologically-plausible learning systems** — three-factor rule with per-car eligibility traces; sparse directed graph topology with cyclic connections handled via one-step `prev`/`curr` propagation; CBP utility-based structural plasticity with rank-1 EMA-tracked utility and outgoing-zero behaviour-preserving replacement; Turrigiano synaptic scaling + Marder intrinsic excitability as the biology-first answer to Hebbian runaway (in place of Oja's rule, weight decay, dropout, batchnorm); plateau-triggered neurogenesis with `reward_window` clear; weight-threshold prune + probabilistic sprout. Mapped from primary neuroscience literature (Schultz 1997 on dopamine prediction error, Turrigiano on synaptic scaling, Markram/Bi/Poo on STDP, Dohare et al. 2024 on Continual Backprop) into a Rust implementation.
- **Designing systems for biology-first discipline under engineering pressure** — articulating and defending the principle as a binary constraint, not a preference; mapping standard ML pathology responses to biological analogues (dropout → homeostasis, EWC → complementary learning systems, batchnorm → synaptic scaling, NEAT → CBP, replay buffer → hippocampal consolidation); rejecting an explicitly engineering-pragmatic shortcut (Option A reusing PPO's critic as M) on first-principles grounds; staging M6 in six small commits per mechanism so each ships with its own tests.
- **ECS-based architectural separation of concerns** — controller-agnostic boundary between `game/` and `brain/` enforced at the Rust module level (`use crate::brain::*` does not appear in `src/game/`), with `ObservationVector` and `ActionState` as the only cross-plugin interface types; compile-time partitioning of multi-learner side-by-side simulation via ZST marker components + Bevy query filters making cross-contamination impossible to write; passive plugins (analytics, debug, profiling) that read but never write simulation state; F4 cycle implemented as despawn-then-respawn to avoid state leak from marker swaps on existing entities.
- **Performance engineering on constrained hardware** — diagnosing a 426-stutter problem at 17.3 ms mean frame time on M2 MacBook Air 8 GB unified memory; resolving to 2 stutters at 9.0 ms mean via six concrete optimisations (flat row-major weight storage replacing `Vec<Vec<f32>>` cache-miss pattern, pre-allocated `BatchScratch` for zero training-loop allocations, batched mat-mat forward + backward over sample-by-sample, iterator-based inner loops for LLVM auto-vectorisation, `std::mem::swap` for the frozen rollout buffer, per-step Adam bias-correction precomputation); follow-up M4 dual GEMM backend (SIMD fallback + Accelerate AMX) + batched actor delivering an additional 21× frame-time improvement. Empirical measurements like "wider critic = 7 ms → 13 ms PPO training cost per chunk; 1.7 ms → 3.3 ms action-selection cost for 8 cars" backing capacity-vs-cost trade-offs.
- **Designing diagnostic instrumentation as a first-class deliverable** — feature-gated profiling system with macro-instrumented per-system timing across 17 systems, ring-buffer `FrameRecord` storage, auto-exit, JSON + 9-section explanatory markdown report; ten-section analytics report (run summary, learning progress, action behaviour, speed & momentum, crash forensics, value-function evolution, track coverage, driving quality, training health, trajectory snapshots) with Unicode ASCII sparklines + horizontal bars + sector heatmaps + per-section auto-generated takeaway sentences + 7 automated diagnostic flags + learning-phase classifier + five-category crash taxonomy + turn-execution failure-mode classifier.
- **Determinism + reproducibility engineering** — fixed 60 Hz tick + explicit `SimSet` ordering + deterministic LCG-based 1200-step replay unit test verifying bitwise-identical position/velocity/heading; layered determinism map calling out strong (physics, ECS schedule), improved (seeded `StdRng` for policy sampling), and weak (uncontrolled init RNG) surfaces as honest open gaps; geometry utilities deduplicated into a shared `sim` module to prevent drift.
- **Disciplined research synthesis into engineering decisions** — seven-paper research round before M6 yielding four named options for the modulator (A/B/C/D), explicit failure-mode discipline for each ("if v1 fails, resist the pull to Option D"), and a 23-decision implementation log (D1–D23) capturing fork/chosen/rejected/rationale for every M6-era choice. RESEARCH-ANCHORED vs TUNE labelling on every config dial separating values pinned to literature from values awaiting empirical sweep.
- **Public-facing communication of engineering work** — LinkedIn article shipped 2026-04-29 ("How I made a Rust ML simulator 40x faster on a MacBook Air, with no GPU") + 3-GIF README embed using gifski-encoded autoplay GIFs respecting GitHub's 10 MB per-file constraint, with a deliberate two-audience split (general-purpose driving GIF as the LinkedIn cover, technical viz overlays as the README's lookahead-and-raycasts GIFs).

---

## Evidence Block

| Path | Verbatim last line |
|---|---|
| Projects/NeuroDrive/_Overview.md | "The window 2026-04-19 → 2026-04-29 was a docs / external-visibility burst; no source-code commits landed. The next code-level work begins when the first SideBySide training run is launched." |
| Projects/NeuroDrive/Gaps.md | "#neurodrive #gaps #technical-debt" |
| Projects/NeuroDrive/Architecture/Fixed Tick Pipeline.md | "#neurodrive #rust #architecture #bevy #simulation" |
| Projects/NeuroDrive/Architecture/Module Boundaries.md | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Architecture/Module Map.md | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Decisions/_Overview.md | "- [[Projects/NeuroDrive/Roadmap]] — milestone arc reflecting these decisions" |
| Projects/NeuroDrive/Decisions/Biology First Principle.md | "#neurodrive #decisions #biology-first #discipline" |
| Projects/NeuroDrive/Decisions/Brain v1 Implementation Log.md | "#neurodrive #decisions #implementation-log #milestone-6" |
| Projects/NeuroDrive/Decisions/Graph Not Layered.md | "#neurodrive #decisions #topology #graph #milestone-6" |
| Projects/NeuroDrive/Decisions/LinkedIn Article + Gifs Shipped.md | "- [[Projects/NeuroDrive/Roadmap]] § M11 Writeup/Release" |
| Projects/NeuroDrive/Decisions/One Brain One Lifetime.md | "#neurodrive #decisions #continual-learning #core-constraint" |
| Projects/NeuroDrive/Decisions/PPO as Baseline.md | "#neurodrive #decisions #ppo #milestone-1" |
| Projects/NeuroDrive/Decisions/Raw Reward as Modulator.md | "#neurodrive #decisions #neuromodulation #option-c #milestone-6" |
| Projects/NeuroDrive/Decisions/Reward Design.md | "#neurodrive #decisions #reward-design" |
| Projects/NeuroDrive/Decisions/Slot Stable Graph Storage.md | "#neurodrive #decisions #data-structures #structural-plasticity #milestone-6" |
| Projects/NeuroDrive/Decisions/Tanh Squashed Actions.md | "#neurodrive #decisions #ppo #action-space" |
| Projects/NeuroDrive/Learning/_Overview.md | "- [[Projects/NeuroDrive/Decisions]] — recorded design decisions" |
| Projects/NeuroDrive/Learning/Continual Backprop Utility.md | "#neurodrive #biologically-inspired #continual-backprop #utility-tracking #neuron-replacement #milestone-6" |
| Projects/NeuroDrive/Learning/Eligibility Traces.md | "#neurodrive #biologically-inspired #eligibility-traces #temporal-credit-assignment #milestone-6" |
| Projects/NeuroDrive/Learning/Hebbian Plasticity.md | "#neurodrive #biologically-inspired #hebbian-plasticity #milestone-6" |
| Projects/NeuroDrive/Learning/Homeostasis.md | "#neurodrive #biologically-inspired #homeostasis #synaptic-scaling #intrinsic-excitability #milestone-6" |
| Projects/NeuroDrive/Learning/Neuromodulation.md | "#neurodrive #biologically-inspired #neuromodulation #dopamine #reward-prediction-error #milestone-6" |
| Projects/NeuroDrive/Learning/STDP.md | "#neurodrive #biologically-inspired #stdp #spiking-neural-networks #long-term-plan" |
| Projects/NeuroDrive/Learning/Structural Plasticity.md | "#neurodrive #biologically-inspired #structural-plasticity #continual-backprop #milestone-6" |
| Projects/NeuroDrive/Learning/Three Factor Learning Rule.md | "#neurodrive #biologically-inspired #three-factor-learning #milestone-6 #reinforcement-learning" |
| Projects/NeuroDrive/Roadmap/Milestone Overview.md | "#neurodrive #roadmap #milestones" |
| Projects/NeuroDrive/Roadmap/Milestone 2 Biological Brain.md | "#neurodrive #roadmap #milestone-2 #biologically-inspired #three-factor-learning" |
| Projects/NeuroDrive/Roadmap/Milestone 6 Brain Inspired v1.md | "#neurodrive #roadmap #milestone-6 #brain-inspired #shipped" |
| Projects/NeuroDrive/Roadmap/Milestones 4 to 8.md | "#neurodrive #roadmap #milestones #long-horizon" |
| Projects/NeuroDrive/Systems/_Overview.md | "- [[Projects/NeuroDrive/Roadmap]] — direction-of-travel" |
| Projects/NeuroDrive/Systems/Analytics and Export.md | "#neurodrive #rust #analytics #observability" |
| Projects/NeuroDrive/Systems/Brain-Inspired Learner.md | "#neurodrive #biologically-inspired #brain-inspired-learner #milestone-6 #three-factor-plasticity #continual-backprop #homeostasis" |
| Projects/NeuroDrive/Systems/Debug and HUD.md | "#neurodrive #rust #debug #hud #bevy" |
| Projects/NeuroDrive/Systems/Determinism.md | "#neurodrive #determinism #reproducibility #architecture" |
| Projects/NeuroDrive/Systems/Environment and Track.md | "#neurodrive #rust #environment #simulation" |
| Projects/NeuroDrive/Systems/Episode and Reward.md | "#neurodrive #rust #reward-design #episode" |
| Projects/NeuroDrive/Systems/Multi Car Training.md | "#neurodrive #rust #training #vectorised-environments" |
| Projects/NeuroDrive/Systems/Observation Vector.md | "#neurodrive #rust #observation #sensors" |
| Projects/NeuroDrive/Systems/PPO Implementation.md | "#neurodrive #rust #ppo #reinforcement-learning #milestone-1" |
| Projects/NeuroDrive/Systems/Profiling.md | "#neurodrive #rust #profiling #performance #bevy" |
| Projects/NeuroDrive/Systems/Trainer Layouts and F4 Cycling.md | "#neurodrive #systems #trainer-layout #zst-markers #milestone-6" |
| Projects/NeuroDrive/Work/Continual ImageNet Adapter.md | "#neurodrive #work #continual-learning #publication-arc" |
| Projects/NeuroDrive/Work/CubeCL Kernel Pack.md | "#neurodrive #work #cubecl #mlx #kernels" |
| Projects/NeuroDrive/Work/Performance Lessons.md | "#neurodrive #performance #lessons-learned #hardware #ppo" |
| Projects/NeuroDrive/Work/Performance.md | "#neurodrive #work #performance" |
