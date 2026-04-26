---
name: NeuroDrive
status: active
source_repo: https://github.com/Capataina/NeuroDrive
lifeos_folder: Projects/NeuroDrive
last_synced: 2026-04-26
sources_read: 41
---

# NeuroDrive

## One-line summary

A Rust/Bevy 2D top-down driving simulator that doubles as a from-scratch testbed for biologically-inspired continual learning — a sparse directed graph of rate-coded tanh neurons trained by local three-factor plasticity with eligibility traces, raw-reward neuromodulation, biological homeostasis, and continual-backprop structural plasticity, with a hand-written PPO baseline running alongside as a permanent diagnostic comparator.

## What it is

NeuroDrive is a disciplined first-principles investigation into whether biological learning rules — local synaptic plasticity, eligibility traces, reward-gated updates, homeostasis, structural adaptation — can produce coherent autonomous driving behaviour in a real-time control task, given a single agent that learns continuously across its entire lifetime, and given no backpropagation. The simulator provides a deterministic, fully-observable Monaco-inspired track; a learning agent inhabits it continuously and must learn to drive without weight resets between episodes, without populations, and without external ML frameworks (no PyTorch, tch-rs, candle, or ndarray-with-autograd — every primitive is hand-written in Rust from numerical scratch).

The project is **active** and milestones M1–M6 are complete as of 2026-04-19; the brain-inspired substrate (M6) shipped in six staged commits across a single day and is live on master, but the first real wall-clock SideBySide training run that would constitute behavioural acceptance has not yet happened. M6 is therefore "code shipped, behavioural validation pending" — the project's stated success bar accepts "visible plasticity signature, no loop completion yet" as a valid M6 ship; the falsification value is treated as equal to the success value.

The architectural split that distinguishes NeuroDrive from a typical RL project is the **biology-first principle**: when the system hits a problem, the answer comes from biology, not from the ML toolkit. Standard ML responses to dead neurons, runaway weights, catastrophic forgetting, exploration collapse — dropout, weight decay, EWC, entropy bonuses — are explicitly forbidden as defaults; their biological analogues (synaptic scaling, intrinsic excitability, complementary learning systems, noradrenergic arousal) are the substitutes. PPO is permanently retained as the diagnostic baseline that proves the environment is learnable and gives the brain-inspired learner a known-working reference to compare against in side-by-side mode (8 PPO + 8 brain cars on the same track simultaneously).

The hardware constraint is deliberate: an M2 MacBook Air, CPU-only, no CUDA. If the architecture cannot run at 60 Hz on modest hardware, it is the wrong architecture. The M4 performance overhaul (dual GEMM backend with SIMD fallback + Accelerate AMX, batched actor across all cars in one GEMM call) delivered ~21× frame-time improvement and made the constraint comfortable even at side-by-side 16 cars (~9% of the 60 Hz frame budget).

## Architecture

The project is a single Bevy application structured as **seven plugins plus a `sim` coordination module**. The most important structural property is that `game/` has no dependency on `brain/` — the environment is controller-agnostic, communicating with whatever learner is loaded through two shared interface types (`ObservationVector`, `ActionState`). This is enforced at the Rust module level: there is no `use crate::brain::*` anywhere under `src/game/`.

```
sim
├── GamePlugin                         (controller-agnostic environment)
│   ├── MonacoPlugin                   src/game/track/      track grid + centerline
│   └── AgentPlugin                    src/game/agent/      car kinematics + obs/action
├── BrainPlugin                        (PpoPlugin AND BrainInspiredPlugin)
│   └── reads:  ObservationVector
│   └── writes: ActionState
├── AnalyticsPlugin                    src/analytics/       passive — reads only
└── DebugPlugin                        src/debug/           passive — reads only

shared interface types: ObservationVector (game → brain), ActionState (brain → game)
```

### The fixed-tick pipeline

The simulation runs at a fixed 60 Hz inside Bevy's `FixedUpdate` schedule, partitioned into four causally-strict `SimSet` stages:

```
FixedUpdate (60 Hz)
├── SimSet::Input        keyboard_action_input → ppo_act_all_cars / brain_act_all_cars → action_smoothing
├── SimSet::Physics      car_physics_system  (kinematic — no rigid-body dynamics, deterministic at 60 Hz)
├── SimSet::Collision    collision_detection_system  (4-corner test against TrackGrid; O(1) per car)
└── SimSet::Measurement  update_track_progress → build_observation_vector → episode_loop
                         → ppo_collect_rewards → ppo_epoch_system        (PPO branch)
                         → brain_learn_all_cars_system                   (brain branch)

Update (frame-rate, uncapped)            HUD + overlays — read-only on simulation state
Last (end of frame)                      ppo_flush_on_exit, JSON + Markdown report export
```

The Input → Physics → Collision → Measurement ordering is causally strict: actions decided before physics, physics before collision check, collision before measurement, observation built after physics so it reflects this tick's outcome, reward computed after observation, PPO update consumes the complete `(s, a, r, s')` tuple. There is one **structural one-tick observation lag** — `*_act_all_cars` reads the `ObservationVector` written in the previous tick's Measurement stage. At 60 Hz this is a 16 ms lag with negligible behavioural effect.

### PPO update amortisation

A full PPO update on the rollout buffer cannot run inside one 16 ms tick without stalling. Instead the buffer collects continuously across many ticks; when it fills, `ppo_epoch_system` processes **64 samples per tick** across 4 epochs, spreading the update over multiple ticks while new transitions accumulate into a fresh buffer. The policy being updated is therefore slightly stale relative to the most recent transitions, but the staleness is bounded and acceptable for on-policy learning at this scale.

### The two-learner partition: ZST markers + TrainerLayout

The pre-M6 design used a global `AgentMode` enum (`Keyboard | PPO | BrainInspired`); a single value controlled every car. This could not express side-by-side comparison ("8 PPO cars AND 8 brain cars on the track simultaneously"). M6 replaced it with three zero-sized type marker components (`PpoCar`, `BrainCar`, `KeyboardCar`) plus a `Controller` enum. Systems filter cars at compile time via Bevy query filters: `Query<..., (With<Car>, With<PpoCar>)>`. Cross-contamination between learners is impossible by construction because the type system excludes mismatched cars from each learner's queries — Decision D20 specifically rejects explicit cross-contamination tests because they would test the compiler, not a real failure mode.

`TrainerLayout` is the fleet-shape configuration:

| Layout | Total cars | Purpose |
|---|---|---|
| `Keyboard` | 1 | Manual-intervention escape hatch |
| `AllPpo { count: 8 }` | 8 | PPO-only diagnostic runs |
| `AllBrain { count: 8 }` | 8 | Brain-only runs (default at first boot, Decision D21) |
| `SideBySide { ppo: 8, brain: 8 }` | 16 | Apples-to-apples comparison; cool/warm palettes distinguish the fleets |

F4 cycles `AllBrain → SideBySide → AllPpo → AllBrain`. The cycle uses despawn-then-respawn (Decision D3) rather than remarking existing entities, because remarking would leak rollout buffers, eligibility traces, activation histories, and partial episode state across the change.

## Subsystems and components

### MonacoPlugin (`src/game/track/`)

Owns the track as a hand-authored 14×9 tile grid, 100 px per tile, total world 1400×900 px. `TrackGrid` is a flat boolean array enabling O(1) per-corner road/off-road queries. `TrackCenterline` stores the ideal racing line as ordered points with cumulative arc-length parameterisation, providing O(log n) projection (returning arc-length, signed lateral offset, tangent angle) and lookahead sampling (12 sample points at fixed arc-length distances from 30 to 650 world units).

### AgentPlugin (`src/game/agent/`)

Owns the cars (kinematic state, `EnvInstanceId` per car, `SensorReadings`, `ObservationVector`, `ActionState`). The car model is purely kinematic — no rigid-body dynamics, no slip, no weight transfer; the difficulty comes from track geometry, not physics complexity. Parameters are fixed constants: `rotation_speed = 8.0 rad/s` (raised from 4.0 because 4.0 was insufficient for tight corners at learned speeds), `thrust = 750.0`, `drag = 0.985` (the sole deceleration mechanism — no braking), throttle range `[0, 1]` (no reverse). The integration step is `step_car_dynamics()`, a pure helper that is also covered by a deterministic-replay unit test (1200 steps with LCG-generated actions, verifying bitwise position/velocity/heading match given identical inputs).

### BrainPlugin — PPO half (`src/brain/ppo/` and `src/brain/common/`)

A from-scratch handwritten PPO implementation: no PyTorch, no tch-rs, no candle, no ndarray-with-autograd — every component (network, optimiser, gradient computation, AMX GEMM) is built from Rust primitives.

- **Asymmetric actor-critic.** Actor `43 → 64 → 64 → 2` with tanh hidden activations; critic `43 → 128 → 128 → 1`. The 2× wider critic was introduced after discovering 40.6% tanh saturation in the symmetric 2×64 critic, with critic weight norms reaching 19.3 and the gap between crash-state values (~26) and safe-state values (~51) too narrow to drive avoidance behaviour.
- **Orthogonal init with scale-differentiated heads.** Hidden layers initialised at √2 (compensates for tanh range); actor mean output `a_mean` initialised at **0.01** so initial outputs are near zero (`tanh(near_zero) ≈ 0`, giving an exploratory uniform initial policy); critic value head at 1.0; biases zeroed; `log_std` initialised to `[0.0, 0.0]` (σ = 1.0).
- **Three forward paths.** `forward_actor`, `forward_critic`, `forward` (both) — saves ~50% per-tick cost when only one is needed.
- **Tanh-squashed Gaussian actions** with explicit Jacobian correction in the log-probability computation: `log π(a|s) = log N(u|μ, σ²) − Σ_i log(1 − tanh²(u_i))`. Skipping the Jacobian biases policy gradients silently. Throttle is remapped to `[0, 1]` via `(tanh(u) + 1) / 2` (no reverse).
- **Hyperparameters.** γ = 0.995 (raised in M5 from 0.99 for longer credit horizon), GAE λ = 0.95, clip ε = 0.2, 4 PPO epochs, rollout horizon = 512, min update = 128, samples per tick = 64 (reduced from 128 to stop stutter), actor LR = 3e-4 plain Adam, critic LR = 5e-4 **AdamW with weight decay λ = 3e-4** (asymmetric optimiser choice — the wider critic needs explicit weight bounding to prevent the saturation that motivated the widening), gradient clip L2 = 0.5, `log_std` floor at **−1.0** (raised from −2.0 because throttle exploration was collapsing to σ ≈ 0.07 and locking the policy at full throttle), Adam ε = 1e-5 (raised from 1e-8 for stability).
- **PopArt adaptive value normalisation** added in M5; `popart_beta = 3e-2` (raised from 1e-4 to keep PopArt tracking real returns).
- **Observation normalisation via running stats** and **target-KL early stop** added in M5.
- **Performance: dual GEMM backend.** SIMD fallback (ARM NEON) plus Accelerate AMX; batched actor for all cars in a single GEMM call. Combined with M4's other changes, this delivered the 21× frame-time improvement.

The PPO module also owns a `TrainerLiveRanking` resource that tracks best/worst car by `0.7 * best_progress_mean + 0.3 * normalised_return_mean` with hysteresis (5% margin to prevent flicker), recomputed once per second.

### BrainPlugin — brain-inspired half (`src/brain/inspired/`, 7 files: config, forward, graph, homeostasis, plasticity, structural, mod)

The substrate that defines the project's thesis. ~50 KB source, ~30 KB tests; M6 v1 shipped on 2026-04-19.

**Topology.** Sparse directed graph of rate-coded tanh neurons, **not** a layered MLP. Hidden→hidden cycles are allowed (with self-loops forbidden); the forward pass uses one-step propagation (every tick rotates `prev ← curr` and reads only from `prev`), which makes iteration order arbitrary and makes cyclic connections trivially well-defined without topological sort. Storage is **slot-stable**: `Vec<Neuron>` and `Vec<Synapse>` with per-element `alive: bool` flags and `free_neuron_slots` / `free_synapse_slots` free-lists. NeuronId and SynapseId are `u32` and stable for the lifetime of the entity — killing a neuron does not invalidate any references. Per-car eligibility lives on the synapse struct itself (`eligibility: Vec<f32>` indexed by `EnvInstanceId`) for cache locality during the plasticity pass.

**Seed graph.** 43 input neurons (one per observation dimension), 15 hidden neurons (default), 2 output neurons (steering and throttle), ~10% density on the legal candidate edges (input→hidden, input→output, hidden→hidden, hidden→output; outputs do not project backwards, inputs receive nothing), weights drawn from `Normal(0, σ=0.1)`.

**Learning rule (three-factor with eligibility traces).**

```
e_ij[c] ← λ · e_ij[c] + pre_i[c] · post_j[c]      (per-car eligibility)
Δw_ij  += η · M_c · e_ij[c]                       (per-car weight update)
w_ij   += Σ_c Δw_ij                               (shared weights, sum-per-car default)
```

with `λ = 0.992` (RESEARCH-ANCHORED — gives τ_e ≈ 2 s at 60 Hz, matching γ = 0.995's ~3.3 s credit horizon), `η = 1e-3` (TUNE), `pre_i = activations.prev[source]`, `post_j = activations.curr[target]`. The tick-shifted pre/post ordering gives **STDP-like causal semantics** ("source fired at t−1, target fired at t") without needing sub-tick spike scheduling. **Critical anti-puffing:** `M_c` is the **raw per-tick reward** from `EpisodeState.tick.reward` — there is no critic, no value module, no TD error δ = r + γV(s') − V(s) anywhere in `src/brain/inspired/`. Plasticity contributions are accumulated into `accumulated_delta: Vec<f32>` first and applied in one batch after every car has been visited so within a tick all cars see the same weights. On episode terminal for car c, `eligibility[c]` is zeroed across every synapse to prevent stale correlations bleeding across resets.

**Homeostasis.** Two biological mechanisms on different cadences. (1) **Intrinsic excitability per tick**: a neuron's `mean_rate` EMA tracks its activation; if it drifts below `lo_band = 0.10` the bias is nudged up by `intrinsic_bias_rate = 1e-4`, if above `hi_band = 0.60` it is nudged down. Inputs are skipped (their bias is part of the I/O contract). `age_ticks` advances here, not in the structural pass. (2) **Synaptic scaling on the structural cadence (every 128 ticks)**: each non-input neuron's incoming weight magnitudes are multiplicatively scaled toward `synaptic_scaling_target = 2.0` with `synaptic_scaling_rate = 0.05`, clamped to a `[0.5, 2.0]` factor to prevent shock corrections. These two replace the role that dropout, batchnorm, and weight decay would play in an ML pipeline.

**Structural plasticity (Continual-Backprop adapted).** Four operations on the structural cadence: (1) **utility EMA** `u_i ← η_u · u_i + (1 − η_u) · h_i · outgoing_sum[i]` per tick with `η_u = 0.99` (CBP §Rank 1); (2) **CBP replacement** picks the lowest-utility mature hidden neurons (`age_ticks ≥ maturity_ticks = 1000`), zeroes outgoing weights (behaviour-preserving at the moment of replacement), resamples incoming weights from `Normal(0, σ=0.1)`, zeroes eligibility across all cars, resets utility/age/mean_rate/bias; (3) **plateau-triggered neurogenesis** detects when the rolling reward window's second-half mean is within 2% relative of the first-half mean and grows a new hidden neuron wired to ~10 random incoming sources and ~10 random outgoing targets, then clears the reward window so the next plateau is measured fresh; (4) **prune** marks any synapse with `|w| < 0.01` dead and returns its slot to the free-list; (5) **sprout** with probability `0.10` per cadence samples 8 random source-target pairs and creates new synapses where the pair is unconnected and legal.

**Plugin scheduling.** `brain_act_all_cars_system` runs in `SimSet::Input` after keyboard input and before action smoothing — the same slot as `ppo_act_all_cars_system`. `brain_learn_all_cars_system` runs in `SimSet::Measurement` after `episode_loop_system` and after `build_observation_vector_system`. The learn function uses two **disjoint Bevy queries on the same entity set** (one read-only of activations, one mutable of `PolicyOutput`) — Bevy permits this because component sets are disjoint, and field-level destructuring inside the function lets `graph`, `rng`, `stats`, and `reward_window` be borrowed simultaneously without aliasing borrow-check errors.

**Side-by-side data isolation.** PPO systems query `(With<Car>, With<PpoCar>)`; brain systems query `(With<Car>, With<BrainCar>)`. PPO's rollout buffer's `env_ids` and the brain's `reward_window` are each populated from their respective queries; cross-contamination is impossible by construction. The only shared state between the two learners is the environment.

### AnalyticsPlugin (`src/analytics/`)

Passive — reads from game and brain, exports on session end. Two-tier export on `AppExit`: a compact JSON (`reports/json/analytics/run_{timestamp}.json`) containing `RunMetadata + EpisodeRecords + PpoUpdateRecords + BrainUpdateRecords`; a full-trace JSON when `AnalyticsConfig.full_trace_export = true` (default false); and a 10-section diagnostic Markdown report (`reports/analytics/run_{timestamp}_<slug>.md` where slug is `brain` / `side` / `ppo` / `keyboard`). Both directories enforce retention limits.

The Markdown report sections are organised around diagnostic questions, not metric modules: Run Summary, Learning Progress, Action Behaviour, Speed & Momentum, Crash Forensics (5-way `CrashKind` classification: Slide / HeadOn / Overshoot / Spin / Stall), What Does the Car Think (value-function evolution), Track Coverage, Driving Quality, Training Health, Trajectory Snapshots. Sections 9, 12, 13, 14 are PPO-centric and **skip entirely with no header** when `tracker.ppo_updates.is_empty()`; sections 16, 17, 18 are brain-centric and skip when `tracker.brain_records.is_empty()`. Section 19 (Fleet Comparison) auto-detects side-by-side from controller tags so it works across multi-cycle F4 runs.

`BrainUpdateRecord` carries per-cadence (per-window, NOT cumulative) snapshots: replacement / neurogenesis / prune / sprout counts, plasticity-health scalars (mean |w|, weight σ, mean |e|), utility percentiles (p10, p90), structural state (neuron / hidden / synapse counts), dead-neuron and saturation fractions. `EpisodeTracker.brain_records` and `CompactRunExport.brain_records` both use `#[serde(default)]` for back-compatibility with pre-M6 JSON files.

ASCII visuals include Unicode sparklines (▁▂▃▄▅▆▇█), horizontal bar charts, single-row heatmaps, and per-section auto-generated takeaway sentences. There are 25 unit tests across the analytics modules (`timeseries`, `diagnostics`, `consistency`, `phases`, `sparkline`, `turns`).

### DebugPlugin (`src/debug/`)

Passive observability. F3 toggles a HUD with per-car episode stats, cross-episode EMAs, PPO health metrics. F4 cycles the `TrainerLayout`. Visual overlays render the 11 raycasts (green for clear, red for hit), the centreline projection with lookahead samples, persistent crash markers (red Xs at recent crash positions), and a per-car leaderboard sorted by progress fraction.

### Profiling system (`src/profiling/`, feature-gated)

Activated by `cargo run --features profiling`; compiles to nothing when the feature is off. Captures per-frame, per-`SimSet`, and **per-system timing** for all 17 instrumented FixedUpdate systems via an `instrument!()` macro that registers `start_timer` / `stop_timer` systems before and after each target. Ring-buffered `FrameTimings` (default 1800 frames = 30 s at 60 Hz) bound memory regardless of run duration. Auto-exits at a configured duration and exports both JSON and a 9-section Markdown report (How to Read, Overall Verdict, Frame Budget, Frame Time Distribution, Pipeline Breakdown, Per-System Detail, Stutter Analysis with PPO-Epoch correlation, Buffer & Memory Pressure, ranked Recommendations). The Markdown report is the largest single source file in the repo at 37.3 KB.

### Determinism

Layered. Strong: fixed timestep, SimSet ordering, pure car dynamics (with deterministic-replay unit test), track construction (no RNG), centreline projection (geometric), observation production (deterministic given car state and track), PPO action sampling (seeded `StdRng` in `PpoBrain`). Weak: PPO model initialisation (uses `rand::rng()` once at startup before seeding `StdRng` — the init seed is not user-controllable). Missing: full ECS replay harness; brain persistence across sessions (Gap #1).

## Technologies and concepts demonstrated

### Languages

- **Rust** — sole implementation language across the whole system (~80 `.rs` files, ~660 KB). Used for: ECS plugin authoring, hand-written numerical primitives (matrix layout, GEMM, Adam/AdamW, gradient computation), graph data structures with slot-stable storage, fixed-tick simulation scheduling, FFI to Apple's Accelerate framework for AMX GEMM, ARM NEON SIMD intrinsics, ring-buffer profiling, Markdown report generation. No `unsafe` is mentioned in LifeOS as a load-bearing pattern; the Accelerate AMX path is the natural exception any Rust-on-macOS numerical stack would have.

### Frameworks and libraries

- **Bevy** — the application's ECS / scheduling / rendering substrate. Plugins, components, systems, queries with `With<...>` filters, the `FixedUpdate` schedule with custom `SimSet` partitioning, the `Update` schedule for frame-rate HUD work, the `Last` schedule for graceful exit flushing, gizmo-based 2D overlays. Used as the architectural backbone, not as a renderer with code bolted on — the entire learner-versus-environment boundary is enforced through Bevy queries.
- **`rand` / `rand::rngs::StdRng`** — seeded RNG for action sampling and brain state.
- **`serde` (with `#[serde(default)]`)** — JSON export for analytics and profiling artefacts; the `default` annotation is load-bearing for back-compat with pre-M6 JSON.
- **Apple Accelerate framework (AMX)** — the second backend in the dual GEMM design, accessed through Rust FFI; the SIMD-fallback path is the first.

### Runtimes / engines / platforms

- **Bevy ECS** — eight plugins composing the application; ZST marker components used as compile-time query gates; `EnvInstanceId` per-car tagging; field-level destructuring of resources to make multiple disjoint mutable borrows valid under the borrow-checker.
- **macOS / Apple Silicon** — primary development target; M2 MacBook Air with 8 GB unified CPU/GPU memory is the reference hardware; ARM64 architecture means NEON SIMD (not SSE/AVX); 60 Hz display drives the fixed-tick choice; AMX is the on-CPU matrix coprocessor used for the optimised GEMM.

### Tools

- **`cargo` with feature flags** — `--features profiling` for the profiling system; `--features force-scalar` for one of the three test build configurations.
- **JSON + Markdown reports** — every meaningful run produces a paired pair under `reports/`; the Markdown is human-grade narrative output, not raw stats.

### Domains and concepts

- **Reinforcement learning (on-policy, continuous control).** PPO with clipped surrogate, GAE (γ = 0.995, λ = 0.95), tanh-squashed Gaussian actions with Jacobian-corrected log-probability for the importance-sampling ratio, adaptive target normalisation via PopArt, target-KL early stop, asymmetric actor-critic sizing with optimiser-level (AdamW vs plain Adam) asymmetry, action smoothing via EMA to suppress steering oscillation, amortised epoch processing for real-time-budget compatibility.
- **Vectorised environments.** Eight cars sharing one policy/brain via per-car `EnvInstanceId` tagging; per-car bootstrap value computation at rollout horizon; ranking with hysteresis to avoid HUD flicker.
- **Hand-written backprop and Adam/AdamW.** No autograd library; gradients computed by hand against the asymmetric architecture; Adam bias correction precomputed once per step (`powi`) rather than per-parameter; per-layer optimiser state.
- **Numerical performance engineering on CPU.** Flat `Vec<f32>` row-major weight storage instead of `Vec<Vec<f32>>` (~43× cache-locality improvement); pre-allocated `BatchScratch` reused across passes for zero-heap-allocation training loops; batched mat-mat operations replacing 64× sample-by-sample mat-vec; iterator-chain inner loops to enable LLVM auto-vectorisation; `std::mem::swap` instead of clones for frozen buffers; result was 426 stutters → 2 and mean frame time 17.3 ms → 9.0 ms with 8 cars.
- **Dual GEMM backend.** SIMD fallback path plus Accelerate AMX, with a batched-actor pass that runs all eight cars' actor forwards as one GEMM call; ~21× frame-time improvement attributable to this and the centreline-query optimisations.
- **Biologically-inspired learning theory and implementation.**
  - *Three-factor plasticity rule*: `Δw = η · M · e` with separation of teaching signal (M) from learning signal (eligibility trace e).
  - *Eligibility traces with exponentially decaying co-activity*: λ = 0.992 chosen against the PPO baseline's γ-derived credit horizon; per-car traces with shared weights as the data-rate-vs-credit-localisation compromise.
  - *Hebbian co-activity at the rate-coded level*: `pre · post` with the `pre = prev[source]` / `post = curr[target]` shift as a deliberate STDP-like causal embedding.
  - *Neuromodulation via raw per-tick reward (Option C)* with explicit recognition that this is a deliberate simplification of the biological prediction-error story, deferred to M8 as an Option B "plastic value predictor" rather than rescued in v1 by borrowing PPO's critic (Option A, explicitly rejected as a thesis violation).
  - *Synaptic scaling (Turrigiano)* and *intrinsic excitability homeostasis (Marder)* as the biological substitutes for ML-toolkit stability mechanisms.
  - *Continual Backprop (Dohare et al. 2024)*, adapted from per-layer ranking on dense MLPs to single-pool ranking on a sparse graph, with the published Rank-1 utility metric and the maturity-gated, behaviour-preserving outgoing-zero replacement protocol.
  - *Plateau-triggered neurogenesis* with the load-bearing detail of clearing the reward window after growth so the trigger does not retrigger every cadence.
  - *Slot-stable graph storage with adjacency-list maintenance* — the data-structure choice that makes structural plasticity correct and cheap, with explicit rejection of compacted Vec (O(n) shifts plus ID invalidation), HashMap (cache-locality penalty), and `Option<Neuron>` slots (`Option` discriminant overhead).
  - *One-step forward propagation on cyclic graphs*: the `prev`/`curr` rotation that makes iteration order arbitrary and makes recurrent connections trivially well-defined; tied back to "real neurons have non-zero integration time constants" as the biological grounding.
  - *Continual learning under "one brain, one lifetime"*: weights never reset between episodes; catastrophic forgetting is the central challenge to be solved, not the failure mode to be avoided; population/evolutionary methods (CMA-ES, NEAT) explicitly excluded; backpropagation excluded from the brain-inspired learner (PPO retains it because PPO is the baseline).
  - *STDP and LIF neurons explicitly understood and explicitly deferred*: the project knows what spike-timing plasticity is and what would be required to retrofit it (LIF neuron model, sub-tick scheduling, spike-trace storage), and chose to ship the rate-coded analog in v1 rather than the spiking version.
- **Type-system-as-design-tool.** ZST marker components as compile-time query gates (cited in the LifeOS notes as preferred over a runtime `Controller::Ppo` enum filter precisely because the type system prunes the query before iteration).
- **ASCII / Unicode data visualisation.** Sparklines, horizontal bar charts, heatmaps inside Markdown reports — chosen as the visual artefact medium when no GUI inspector exists yet.
- **Reward design as a discipline.** Velocity-projection + centreline-proximity + small time-penalty + zero-crash-penalty + no-lap-bonus, with explicit rationale for each component; the zero crash penalty is a deliberate statement that aggressive driving with consequential crashes is the desired behaviour, not cautious crash-avoidance optimised driving.
- **Profiling-driven optimisation.** A feature-gated, per-system-timed profiler purpose-built to diagnose stutter, with auto-exit and rich Markdown reporting; the system was added in the same commit as the optimisation work it was used to validate.

## Key technical decisions

A 23-decision implementation log (D1–D23) is maintained for the M6 build; a smaller set of cross-cutting decisions sits above them.

- **Biology-First Principle (load-bearing project thesis).** When the brain-inspired learner hits a problem, the response is to consult biology first; if biology is unclear, research more biology rather than reaching for the ML toolkit. Concrete consequences: no dropout, no batchnorm, no weight decay (ML toolkit), no EWC, no experience replay as the primary learning mechanism, no genetic algorithms / NEAT / CMA-ES, no PyTorch / TensorFlow / JAX / candle / tch-rs / ndarray-with-autograd. Standard ML techniques are admissible *only when they have a direct biological analogue* (weight clipping → synaptic homeostasis is fine; dropout → no clear analogue, avoid). The principle is binary, not negotiable, because partial retreat empirically collapses to "ML with biological vocabulary".

- **One brain, one lifetime.** The agent's weights are never reset — same parameters adapt across all episodes, all sessions, all tracks. With weight resets, every episode is an independent learning problem (episodic RL); without them, every episode's learning accumulates and catastrophic forgetting becomes the central challenge to be solved rather than avoided. PPO respects this within a session (rollout buffers do not constitute resets); the brain-inspired learner respects it natively.

- **Modulator design — Option C, raw per-tick reward (M6).** Four candidates were enumerated: (A) reuse PPO's GAE δ as the modulator — *rejected* because it would make the brain-inspired learner depend on a backprop-trained component; (B) train a plasticity-trained value predictor and compute a TD-error-analog modulator — *deferred to M8* as the correct biological destination but a real architectural addition deserving its own milestone; (C) use raw per-tick reward — *chosen for v1* as the smallest correct starting point that preserves Option B as a clean future addition; (D) "import PPO's critic if v1 fails" — *forbidden* by the failure-mode discipline (the correct response to v1 failing is biological diagnosis, not crossing the thesis line). Three task properties make (C) more viable than it would be on a generic RL task: dense reward most ticks, λ tuned to the action-reward gap, non-negative reward most ticks.

- **PPO is permanent, not retired.** Pre-restructure, the plan was to replace PPO with the biological brain at M2. Post-restructure (2026-04-19), PPO stays permanently live as the diagnostic baseline; the brain-inspired learner is **additive**, side-by-side is a core feature, and `TrainerLayout::default()` returns `AllBrain { count: 8 }` as the first-boot reflection of the project's thesis (Decision D21). Rationale: the brain-inspired learner's behaviour is only measurable against a known-working reference; retiring PPO would remove that reference permanently.

- **Sparse graph topology, not layered.** Three reasons in priority order: (1) biological faithfulness — real brains are graph-structured; layers are an ML convention; (2) structural plasticity is natural on a graph (allocate-from-free-list vs reshape-the-weight-matrix-and-propagate-dimensions-everywhere); (3) the M7 visualiser ("watch a brain grow") is natively a graph rendering. Costs accepted: no automatic ordering (resolved by `prev`/`curr` rotation), worse cache locality at scale, cannot reuse PPO's GEMM backend (acceptable because brain-inspired at v1 scale is ~5000 MACs per tick per car).

- **Slot-stable graph storage with `alive` flags and free-lists.** Rejected three alternatives explicitly: compact-the-Vec-on-every-death (O(n) shifts plus the bookkeeping nightmare of reindexing every cached NeuronId/SynapseId in every adjacency list and every per-car activation buffer), `HashMap<NeuronId, Neuron>` (cache locality much worse than `Vec`; the forward pass's hot loop dominates), `Vec<Option<Neuron>>` (`Option` discriminant overhead vs an `alive: bool` field already on the cache line being loaded). Per-car eligibility lives on the `Synapse` struct itself for cache locality during the plasticity pass; `num_cars` is fixed at graph construction so the eligibility `Vec<f32>` is sized once and never resized.

- **Forward pass reads `prev`, writes `curr`.** Buffer rotation at tick start. Inputs write directly; non-inputs compute `z = bias + Σ prev[source] · weight`, apply tanh → `curr`. Order-independence invariant: the non-input loop iterates in arbitrary order and never reads `curr` of another non-input written this tick. Cyclic connections are well-defined via the t-1 → t propagation delay, which is *also* biologically reasonable (neurons have non-zero integration time constants).

- **Pre/post ordering for STDP-like causal semantics.** `pre = prev[source]`, `post = curr[target]`. Correlates "source fired at t-1 with target fired at t" — pre-before-post causal pattern — and gives STDP-like semantics from rate-coded tanh neurons without sub-tick spike scheduling. Real STDP requires LIF neurons + sub-tick scheduling (Long-Term Plan, deliberately deferred).

- **Sum per-car weight updates, not average.** Default `sum_per_car_updates = true`. Eight cars' Δw contributions sum into shared weights — "8× data into one brain" rather than the safer-but-slower averaging. Exposed as a config flag for ablation.

- **Three `enable_*` ablation flags** (`enable_plasticity`, `enable_homeostasis`, `enable_structural`) defaulting on, allowing runtime "does this mechanism matter" isolation without recompilation.

- **Intrinsic homeostat per-tick, synaptic scaling on cadence (Decision D14).** `update_intrinsic_homeostat` advances `age_ticks` every tick — if it ran only on the structural cadence (every 128 ticks), CBP's maturity gate (`age_ticks ≥ maturity_ticks = 1000`) would lag by 128× and become meaningless. Synaptic scaling is a whole-graph scan and runs only on the cadence to avoid wasting per-tick compute. The age-tick coupling means disabling homeostasis via `enable_homeostasis = false` *also* disables CBP replacement — intentional but non-obvious.

- **Replacement is outgoing-zero then incoming-resample (Decision D15).** Order matters. Zeroing outgoing first means the replaced neuron immediately stops contributing to downstream computation (small perturbation at worst); then resampling incoming weights gives it fresh input patterns to find signal in. Reverse order would briefly produce activations from random inputs while the old outgoing weights still propagated, causing a behaviour spike.

- **Reward window is cleared after neurogenesis (Decision D16).** Without clearing, a long genuine plateau would re-trigger neurogenesis every 128 ticks until the brain exploded in size.

- **CBP over NEAT for structural plasticity.** Three candidates considered: NEAT (rejected because population evolution is incompatible with one-brain-one-lifetime), Net2Net (rejected because designed for dense feedforward + ReLU-specific), Continual Backprop (chosen — direct task analogy, published on PPO + continuous control + continual training).

- **Tanh hidden activations everywhere, not ReLU.** Pre-tanh ReLU networks produced 34–57% dead neuron rates per layer, halving effective capacity and starving the actor of cornering ability. The switch eliminated dead neurons entirely (0% saturation observed post-switch) and aligned with the Andrychowicz et al. finding that tanh outperforms ReLU in on-policy continuous control. The `dead_relu_fraction` diagnostic was renamed to `tanh_saturation_fraction` (`|output| > 0.99`) and is what motivated widening the critic to 2×128.

- **Asymmetric AdamW for the critic only.** Plain Adam (no weight decay) for the actor; AdamW with decoupled weight decay λ = 3e-4 for the critic. The asymmetry targets the specific problem of unbounded critic weight growth driving tanh saturation; the actor's smaller 2×64 size does not exhibit the same growth pattern.

- **Reward design discipline.** Velocity projection (rewards speed in the right direction, not just movement) + centreline proximity (quadratic falloff, magnitude 0.3 secondary to velocity) + small time penalty (−0.005, prevents stalling, small enough not to incentivise crash-to-stop) + **zero crash penalty** (deliberate statement that aggressive driving is the goal; cautious crash-avoidance produces boring policies; the agent should avoid crashing as a *consequence* of seeking sustained reward, not as an *objective* in itself) + no lap-completion bonus (removed because the +100 reward cliff distorted the landscape, and arc-length progress already provides the equivalent continuous signal).

- **F4 cycle is despawn-then-respawn, not remarking (Decision D3).** Remarking cars would leak rollout buffers, eligibility traces, activation histories, partial episode state. Despawning is the cheap clean slate.

- **Side-by-side defaults to 8+8 = 16 cars (Decision D5), not 4+4 or 12+12.** 4+4 halves each learner's data rate and pollutes the comparison; 12+12 leaves no frame-budget margin. 8+8 means each learner sees the same 8-car data rate it would see in its single-fleet layout, making the comparison literally apples-to-apples.

- **No explicit cross-contamination tests (Decision D20).** The Rust type + component system makes "PPO query never sees a brain car" a compile-time fact, not a runtime assertion. Writing tests for it would be testing the compiler.

- **Layout-aware report naming with PPO-section suppression (Decision D23).** `run_<timestamp>_<slug>.md` where slug is `brain` / `side` / `ppo` / `keyboard`. PPO-centric Markdown sections (9, 12, 13, 14) emit no header at all when `tracker.ppo_updates.is_empty()` — no "no PPO updates recorded" stubs in brain-only reports. Brain sections (16, 17, 18) skip when `tracker.brain_records.is_empty()`. `ls reports/analytics/` immediately communicates what each file is.

- **Hand-written everything, no ML libraries.** Implicit but load-bearing — the project would not exist if this constraint were relaxed. Every primitive (network, optimiser, gradient, AMX GEMM, action sampler) is from Rust + the standard library + (for AMX) Accelerate.

## What is currently built

What runs today on master, separated from the design ambition.

- **Environment.** Monaco track (14×9 hand-authored tile grid, 1400×900 px), kinematic car model at 60 Hz, episode lifecycle (crash or 30 s timeout), random centreline respawn, full 43-dim `ObservationVector`, deterministic-replay unit test verifying bitwise trajectory match. Single track only — multi-track support is not implemented.
- **PPO baseline (M1–M5 complete).** Asymmetric actor 2×64 / critic 2×128 with tanh hidden activations; orthogonal init with scale-differentiated heads; tanh-squashed actions with Jacobian correction; GAE γ = 0.995 / λ = 0.95; PPO clipped surrogate ε = 0.2; 4 epochs; rollout 512 / amortised 64 samples per tick; plain Adam for actor / AdamW for critic; PopArt; observation normalisation; target-KL early stop. M5 validation showed all 8 cars completing the full Monaco loop, fleet max-progress spread 1.1%, crash rate falling 100% → 56% in the best chunk, 96% of crashes had throttle released > 0.25 s before impact (the policy *anticipates* collision).
- **Performance overhaul (M4 complete).** Dual GEMM backend (SIMD fallback + Accelerate AMX), batched actor for all cars in single GEMM, centreline-query optimisations, flat `Vec<f32>` weight storage, pre-allocated `BatchScratch`, batched mat-mat operations, iterator-chain loops, `mem::swap` for frozen buffers, precomputed Adam bias correction. Result: 426 stutters → 2; mean frame time 17.3 ms → 9.0 ms with 8 cars; ~21× overall frame-time improvement.
- **Brain-inspired v1 (M6 — code shipped 2026-04-19; behavioural acceptance pending).** All six mechanisms wired up and live in master: forward pass, three-factor plasticity, eligibility traces, raw-reward modulator, intrinsic excitability + synaptic scaling homeostasis, continual-backprop utility tracking + replacement, plateau-triggered neurogenesis, prune + sprout. ZST marker components and `TrainerLayout` cycling. Side-by-side mode with palette-distinguished fleets. Three `enable_*` ablation flags default on. `BrainBrain` resource carries the graph, RNG, tick counter, running stats, and reward window.
- **Analytics (M5 + M6 extensions complete).** Two-tier JSON export, 10-section Markdown report, 5-way crash classification, layout-aware report naming with PPO/brain section suppression, Fleet Comparison auto-detection from controller tags, `BrainUpdateRecord` per-cadence snapshots through to JSON + Markdown, 25 unit tests in the analytics modules.
- **Profiling system.** Feature-gated, per-system timing for all 17 instrumented FixedUpdate systems, ring-buffered, auto-exit, JSON + 9-section Markdown report.
- **Test suite.** 133 green tests across default, `force-scalar`, and release builds — 101 unit + 21 brain-pipeline integration + 6 GEMM + 5 PPO. The 21 brain-pipeline tests cover forward determinism, action range, eligibility decay, weight-update scaling with η, no NaN/Inf over 10k ticks, terminal eligibility zeroing, synaptic scaling target convergence, intrinsic homeostat band-targeting, homeostasis idempotence at steady state, replacement candidate selection, replacement outgoing-zero, replacement connectivity preservation, plateau detection on flat reward, neurogenesis growing neuron count, utility EMA convergence, `BrainUpdateRecord` serde round-trip, pre-M6 JSON back-compat, trainer-layout total-cars consistency, F4 cycle exhaustiveness, palette visual distinctness.

What is **not** built (anti-puffing — these may appear in older notes or the README's pitch but the code does not support them today): TD-error-as-modulator (no critic, no value module in the brain-inspired learner — Option B is M8); STDP / spiking neurons (rate-coded tanh only, no LIF, no sub-tick scheduling); separate linear critic for the brain (Option B / M8); Oja's rule / weight-normalised Hebbian (plain `pre · post`); save/load of brain or PPO state to disk (no serialisation of weights, eligibility, or RNG state across sessions); multi-track (single Monaco track); Dale's law / synaptic delays / Tsodyks-Markram short-term dynamics / multiple neuron types / sleep-replay consolidation (Long-Term Plan, none scheduled); brain visualiser / live graph inspector (M7, not started).

Code-size summary: 80 `.rs` source files (~660 KB), 3 integration test files (~45 KB, the largest is `brain_inspired_pipeline.rs` at 29.8 KB), 36 Markdown context docs (~939 KB) plus 60 files of learning-archive material (~504 KB), brain-inspired module ~50 KB source + ~30 KB tests across 7 files (config, forward, graph, homeostasis, plasticity, structural, mod).

## Current state

Active. Last meaningful activity 2026-04-24 (post-M6 context upkeep at commit `8fe0787`), with the M6 substrate shipped 2026-04-19 across six staged commits in a single day. The single most important next item is the first real wall-clock SideBySide training run that constitutes M6's behavioural acceptance — until that happens, M6 is "code shipped" rather than "M6 closed". Two open Work files (`Continual ImageNet Adapter`, `CubeCL Kernel Pack`) capture additive proposals that are explicitly gated behind the SideBySide validation run and would not modify the M6 substrate.

## Gaps and known limitations

Honest current state, prioritised by the project's own ranking.

- **First real SideBySide training run (Critical, blocks M6 acceptance).** Wall-clock training time has not yet been spent on a side-by-side run. All 21 brain-pipeline tests verify *mechanics* (eligibility decays, weight updates scale with η, no NaN over 10k ticks, terminal zeroing); none verify the brain produces meaningful driving behaviour. The acceptance bar is brain-fleet reward trend rising over ~2000 episodes, observable directional bias signature in analytics, at least one replacement + one neurogenesis event. Until this happens, every other gap is minor by comparison.

- **Brain persistence across sessions (High).** Neither PPO weights nor brain-inspired state is serialised to disk. Each session begins fresh — `BrainGraph` neurons + synapses + per-car eligibility + `BrainTrainingStats.history` + `BrainBrain.rng` state, plus PPO weights + AdamW optimiser state + PopArt tracking, all lost on exit. Violates one-brain-one-lifetime at the session level. Especially costly for the brain-inspired learner because per-episode learning is slower than PPO's batched gradient and structural events fire only on cadence.

- **Tuning sweep over the 13 `TUNE` dials (Medium).** Of the 22 fields in `BrainInspiredConfig`, 9 are RESEARCH-ANCHORED (λ = 0.992, η_u = 0.99, maturity_ticks = 1000, the I/O-contract fields, and the seed-graph topology fields) and 13 are TUNE — chosen for plausibility, not measurement. The first SideBySide run produces the data for the first informed sweep. Key TUNE dials likely to matter: `eta` (learning rate), `replace_fraction`, `structural_cadence`, `plateau_threshold`, `prune_weight_threshold`, `sprout_probability`, `synaptic_scaling_rate`, `intrinsic_bias_rate`.

- **Sprout/prune balance is undocumented (Low but unusual).** The `sprout_probability = 0.10` per cadence and `prune_weight_threshold = 0.01` are each individually reasonable, but no documented analysis confirms that expected sprout count balances expected prune count under steady-state weight distributions. Symptoms of imbalance would be monotonically rising graph density (sprout > prune) or collapsing density (prune > sprout). The first SideBySide run should log `sprout_events / prune_events` per cadence window.

- **HUD column split for side-by-side (Low).** The analytics report has a Fleet Comparison section; the live HUD still shows single-column PPO stats. In side-by-side mode, brain-fleet metrics have no live visual presence.

- **No multi-track support (Low now, blocks M10).** Only Monaco. Multi-track requires a second tile map, track-selection logic, a `TrackId` enum, and track-aware random spawn. Foundation exists in `src/maps/parts/`.

- **PPO integration tests beyond unit scope (Medium).** 21 brain-pipeline tests cover M6; PPO has 5 tests focused on GAE + update mechanics with no behavioural success-threshold tests (does PPO actually converge on Monaco within N episodes?). A documented testing-strategy plan in the context layer outlines 8 categories but is unimplemented. Could become blocking if PPO regresses silently.

- **User-controllable initialisation seed (Low).** `BrainBrain.rng` honours `config.rng_seed` if set but is not exposed on the CLI; PPO model init uses thread-local RNG once at startup before seeding `StdRng`, making the init seed not user-controllable. A unified seed surface is the remaining prerequisite for fully-reproducible AI runs.

- **Lap detection was removed (Low, deliberate).** The +100 lap-completion bonus created reward-cliff pathologies and was removed along with detection. The agent cannot distinguish "completed a lap" from "drove for 30 s without crashing". Easy to re-add as an arc-length-threshold detector if M10 needs it.

- **Context documentation drift (Low, partially addressed).** The 2026-04-19 upkeep commit (`8fe0787`) addressed most of this; some `context/architecture.md` and `context/systems/*` text still describes the pre-M6 global-`AgentMode` world.

## Direction (in-flight, not wishlist)

Active, near-term work the project is genuinely pointed at.

- **First real SideBySide training run (M6 behavioural acceptance).** The single piece of work that would close M6. Wall-clock training time, brain fleet reward trend over ~2000 episodes, directional bias signature in analytics, at least one observed replacement + neurogenesis event. Everything else is downstream.

- **M7 brain visualisation.** Next milestone after M6 validates. A live inspector for the brain-inspired learner's graph state: neurons as dots (brightness = utility, size = outgoing degree), synapses as lines (thickness = |weight|, colour = sign), activations animated per tick, structural events visually surfaced, heat-map overlay of input neurons. The README calls the visualiser the "emotional core" of the project; M7 earns its position immediately after M6 because the visual feedback loop is what makes tuning the 13 TUNE dials tractable — numbers alone are too thin for a rate-coded graph's dynamics.

- **Two additive proposals captured in `Work/`, both gated on SideBySide validation:**
  - **Continual ImageNet adapter (publication arc).** A benchmark-adapter layer converting Permuted MNIST 200-task input format into the M6-consumable temporal format, treating image classification as an alternate input stream the substrate can be evaluated on; the M6 substrate stays exactly as it is. Targets a paper-grade write-up positioning the substrate as a continual-learning method versus published baselines (MESU, RDBP, EWC, Synaptic Intelligence). Honest framing accepts a negative result as publishable.
  - **CubeCL + MLX kernel pack (infrastructure arc).** Repackaging the existing M6 primitives as accelerated GPU kernels accessible via Tracel AI's CubeCL (CUDA + Metal + Vulkan + ROCm + WGPU) and as MLX-native operators for Apple Silicon, with the existing scalar / AMX-GEMM CPU path as the correctness oracle. Bit-equivalent (or measurably-equivalent within float32 noise) output verified against the CPU reference on fixed-seed input. Converts the project from "research codebase" to "infrastructure other researchers can call".

Beyond these in-flight items the roadmap names M8 (plastic value predictor — Option B, the planned destination if v1 raw-reward modulator proves insufficient), M9 (multi-neuromodulator refinement — dopamine/noradrenaline/acetylcholine/serotonin analogues with distinct roles), the Long-Term Plan (Dale's law, synaptic delays, Tsodyks-Markram short-term dynamics, multiple neuron types, sleep-replay consolidation, spiking + STDP — flexibly ordered, pulled forward when a pathology motivates), M10 (multi-track, transfer, curriculum — the continual-learning evaluation), and M11 (writeup / release). These are roadmap, not in-flight.

## Demonstrated skills

What this specific project proves the author can do, anchored to LifeOS-documented evidence.

- **Build a complete handwritten on-policy RL training pipeline in Rust from numerical primitives.** Asymmetric actor-critic, orthogonal initialisation with scale-differentiated heads, tanh-squashed Gaussian policy with Jacobian-corrected log-probability, GAE with bootstrap-at-horizon, clipped surrogate objective, per-layer Adam / AdamW with decoupled weight decay (asymmetric per-component), gradient clipping, PopArt adaptive value normalisation, observation running-stats normalisation, target-KL early stop, action smoothing — all without any ML library, including the gradient computation.

- **Diagnose and resolve specific RL pathologies with named root-causes.** ReLU dead-neuron rates of 34–57% measured and resolved by switching to tanh; critic tanh saturation of 40.6% with weight norms 19.3 measured and resolved by widening to 2×128 plus AdamW; throttle exploration collapse to σ ≈ 0.07 measured and resolved by raising the `log_std` floor from −2.0 to −1.0; A2C-style policy oscillation resolved by adopting PPO clipping (a2c/ → ppo/ rename across 105 references); braking action collapse to "mostly brake" resolved by reverting throttle to `[0, 1]`. Each fix is documented with the diagnostic observation that motivated it.

- **Engineer 60 Hz CPU-only real-time performance through targeted memory-layout and batching changes.** 21× frame-time improvement via dual GEMM backend (SIMD fallback + Accelerate AMX), batched actor in one GEMM call, flat `Vec<f32>` row-major weight storage replacing `Vec<Vec<f32>>` (~43× cache-locality win), pre-allocated `BatchScratch` for zero-heap-allocation training loops, batched forward/backward replacing 64× sample-by-sample mat-vec, iterator-chain inner loops for LLVM auto-vectorisation, `std::mem::swap` for frozen buffers, precomputed Adam bias correction. Concrete result: 426 stutters → 2 and 17.3 ms → 9.0 ms mean frame time at 8 cars.

- **Build a per-system-timed profiling subsystem from scratch.** Feature-gated `instrument!()` macro that registers `start_timer` / `stop_timer` Bevy systems before and after each target, ring-buffered `FrameTimings`, automatic exit, 9-section Markdown report with stutter-correlation analysis. The profiler's Markdown generator is the largest source file in the repo (37.3 KB).

- **Design an ECS architecture where the learner-environment boundary is a compile-time invariant.** ZST marker components used as Bevy query gates (`With<PpoCar>` / `With<BrainCar>`); enforced one-way dependency (`game/` has no `use crate::brain::*`); cross-contamination between learners impossible by construction; type-system-as-design-tool reasoned about explicitly (the ZST-vs-enum-filter rationale cited as compile-time pruning vs runtime iteration).

- **Translate biological learning theory into shippable code.** Three-factor plasticity with eligibility traces (`Δw = η · M · e`, per-car traces with shared weights), Hebbian rate-coded co-activity with deliberate STDP-like causal embedding via `pre = prev[source]` / `post = curr[target]`, intrinsic excitability homeostasis (Marder), synaptic scaling homeostasis (Turrigiano), continual-backprop utility tracking and replacement (Dohare et al. 2024) adapted from per-layer ranking on dense MLPs to single-pool ranking on a sparse graph, plateau-triggered neurogenesis with reward-window clearing, prune + probabilistic sprout — all running together in one tick pipeline at 60 Hz with 8 cars contributing per-tick.

- **Choose data structures against the actual access pattern, not the conceptual one.** Slot-stable `Vec<Neuron>` and `Vec<Synapse>` with `alive` flags and free-lists, with explicit rejection of three alternatives (compact-on-death, HashMap, `Option<T>` slots) for specific reasons (O(n) shifts plus ID invalidation; cache-locality penalty; discriminant overhead). Per-car eligibility on the `Synapse` struct itself for cache locality during the plasticity pass.

- **Make graph-with-cycles work without a topological sort.** One-step propagation via `prev`/`curr` rotation; iteration order arbitrary because the non-input loop only ever reads `prev`; cyclic connections trivially well-defined because A's effect on B manifests one tick later and vice versa; biologically grounded by appealing to neurons' integration time constants.

- **Reason about credit assignment without backpropagation.** Per-car eligibility traces sized to the action-reward gap (λ = 0.992 chosen against PPO's γ = 0.995 horizon); sum-vs-mean per-car update aggregation as a deliberate data-rate-vs-noise tradeoff; terminal eligibility zeroing as the cross-episode-bleed safeguard; biology-first acceptance that a non-negative raw-reward modulator can only fail-to-strengthen rather than weaken (and explicit deferral of TD-error semantics to M8 rather than rescuing v1 with PPO's critic).

- **Write a research-grade decision log.** D1–D23 implementation decisions captured for the M6 build, each naming the fork, the choice taken, and the alternative rejected; a separate file for each load-bearing decision (Biology First Principle, One Brain One Lifetime, Raw Reward as Modulator, Slot Stable Graph Storage, Graph Not Layered, PPO as Baseline, Reward Design, Tanh Squashed Actions). Decision file template is reusable across projects.

- **Maintain a multi-format diagnostic analytics pipeline.** Two-tier JSON export, 10-section Markdown report with per-section auto-generated takeaway sentences, Unicode sparklines / horizontal bar charts / heatmaps, 5-way crash classification (Slide / HeadOn / Overshoot / Spin / Stall) computed from kinematic state at impact, Fleet Comparison section auto-detected from controller tags, layout-aware report naming with conditional section suppression, full back-compat via `#[serde(default)]` for older JSON. 25 unit tests across analytics modules.

- **Design hardware-aware architecture.** M2 MacBook Air with 8 GB unified memory drives the architectural choices: CPU-only (no CUDA), brain-inspired must run alongside PPO at 16 cars within ~9% of a 60 Hz frame budget, ring-buffered profiling to bound memory regardless of run duration, `BatchScratch` pre-allocated to bound heap activity, retention limits on report directories, ARM NEON SIMD as the relevant intrinsics target (not SSE/AVX). Articulated as a discipline: *"if the architecture cannot run at 60 Hz on a MacBook Air, it is not the right architecture."*

- **Hold a thesis under engineering pressure.** The Biology-First Principle is named explicitly as "load-bearing, binary, non-negotiable", with a documented failure-mode discipline ("if v1 doesn't learn, do not import PPO's critic as a rescue — diagnose in biology terms first; the project's contribution is the falsification, not a success narrative") and a reproducible decision flow for any pathology (observe → consult biology → translate → implement). The discipline is not aesthetic; it is what differentiates the project from "RL with biological vocabulary".

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/NeuroDrive/Overview.md | 204 | "#neurodrive #rust #reinforcement-learning #biologically-inspired #project-active #milestone-6-shipped" |
| Projects/NeuroDrive/Gaps.md | 234 | "#neurodrive #gaps #technical-debt" |
| Projects/NeuroDrive/Architecture/Fixed Tick Pipeline.md | 150 | "#neurodrive #rust #architecture #bevy #simulation" |
| Projects/NeuroDrive/Architecture/Module Boundaries.md | 143 | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Architecture/Module Map.md | 92 | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Decisions/Biology First Principle.md | 145 | "#neurodrive #decisions #biology-first #discipline" |
| Projects/NeuroDrive/Decisions/Brain v1 Implementation Log.md | 141 | "#neurodrive #decisions #implementation-log #milestone-6" |
| Projects/NeuroDrive/Decisions/Graph Not Layered.md | 183 | "#neurodrive #decisions #topology #graph #milestone-6" |
| Projects/NeuroDrive/Decisions/One Brain One Lifetime.md | 127 | "#neurodrive #decisions #continual-learning #core-constraint" |
| Projects/NeuroDrive/Decisions/PPO as Baseline.md | 123 | "#neurodrive #decisions #ppo #milestone-1" |
| Projects/NeuroDrive/Decisions/Raw Reward as Modulator.md | 136 | "#neurodrive #decisions #neuromodulation #option-c #milestone-6" |
| Projects/NeuroDrive/Decisions/Reward Design.md | 142 | "#neurodrive #decisions #reward-design" |
| Projects/NeuroDrive/Decisions/Slot Stable Graph Storage.md | 191 | "#neurodrive #decisions #data-structures #structural-plasticity #milestone-6" |
| Projects/NeuroDrive/Decisions/Tanh Squashed Actions.md | 139 | "#neurodrive #decisions #ppo #action-space" |
| Projects/NeuroDrive/Learning/Continual Backprop Utility.md | 174 | "#neurodrive #biologically-inspired #continual-backprop #utility-tracking #neuron-replacement #milestone-6" |
| Projects/NeuroDrive/Learning/Eligibility Traces.md | 225 | "#neurodrive #biologically-inspired #eligibility-traces #temporal-credit-assignment #milestone-6" |
| Projects/NeuroDrive/Learning/Hebbian Plasticity.md | 168 | "#neurodrive #biologically-inspired #hebbian-plasticity #milestone-6" |
| Projects/NeuroDrive/Learning/Homeostasis.md | 191 | "#neurodrive #biologically-inspired #homeostasis #synaptic-scaling #intrinsic-excitability #milestone-6" |
| Projects/NeuroDrive/Learning/Neuromodulation.md | 177 | "#neurodrive #biologically-inspired #neuromodulation #dopamine #reward-prediction-error #milestone-6" |
| Projects/NeuroDrive/Learning/STDP.md | 213 | "#neurodrive #biologically-inspired #stdp #spiking-neural-networks #long-term-plan" |
| Projects/NeuroDrive/Learning/Structural Plasticity.md | 251 | "#neurodrive #biologically-inspired #structural-plasticity #continual-backprop #milestone-6" |
| Projects/NeuroDrive/Learning/Three Factor Learning Rule.md | 199 | "#neurodrive #biologically-inspired #three-factor-learning #milestone-6 #reinforcement-learning" |
| Projects/NeuroDrive/Roadmap/Milestone 2 Biological Brain.md | 211 | "#neurodrive #roadmap #milestone-2 #biologically-inspired #three-factor-learning" |
| Projects/NeuroDrive/Roadmap/Milestone 6 Brain Inspired v1.md | 155 | "#neurodrive #roadmap #milestone-6 #brain-inspired #shipped" |
| Projects/NeuroDrive/Roadmap/Milestone Overview.md | 312 | "#neurodrive #roadmap #milestones" |
| Projects/NeuroDrive/Roadmap/Milestones 4 to 8.md | 267 | "#neurodrive #roadmap #milestones #long-horizon" |
| Projects/NeuroDrive/Systems/Analytics and Export.md | 228 | "#neurodrive #rust #analytics #observability" |
| Projects/NeuroDrive/Systems/Brain-Inspired Learner.md | 416 | "#neurodrive #biologically-inspired #brain-inspired-learner #milestone-6 #three-factor-plasticity #continual-backprop #homeostasis" |
| Projects/NeuroDrive/Systems/Debug and HUD.md | 150 | "#neurodrive #rust #debug #hud #bevy" |
| Projects/NeuroDrive/Systems/Determinism.md | 69 | "#neurodrive #determinism #reproducibility #architecture" |
| Projects/NeuroDrive/Systems/Environment and Track.md | 162 | "#neurodrive #rust #environment #simulation" |
| Projects/NeuroDrive/Systems/Episode and Reward.md | 162 | "#neurodrive #rust #reward-design #episode" |
| Projects/NeuroDrive/Systems/Multi Car Training.md | 151 | "#neurodrive #rust #training #vectorised-environments" |
| Projects/NeuroDrive/Systems/Observation Vector.md | 170 | "#neurodrive #rust #observation #sensors" |
| Projects/NeuroDrive/Systems/PPO Implementation.md | 306 | "#neurodrive #rust #ppo #reinforcement-learning #milestone-1" |
| Projects/NeuroDrive/Systems/Profiling.md | 184 | "#neurodrive #rust #profiling #performance #bevy" |
| Projects/NeuroDrive/Systems/Trainer Layouts and F4 Cycling.md | 227 | "#neurodrive #systems #trainer-layout #zst-markers #milestone-6" |
| Projects/NeuroDrive/Work/Continual ImageNet Adapter.md | 48 | "#neurodrive #work #continual-learning #publication-arc" |
| Projects/NeuroDrive/Work/CubeCL Kernel Pack.md | 48 | "#neurodrive #work #cubecl #mlx #kernels" |
| Projects/NeuroDrive/Work/Performance Lessons.md | 172 | "#neurodrive #performance #lessons-learned #hardware #ppo" |
| Projects/NeuroDrive/Work/Performance.md | 30 | "#neurodrive #work #performance" |
