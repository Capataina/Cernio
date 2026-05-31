---
name: NeuroDrive
status: active
source_repo: https://github.com/Capataina/NeuroDrive
lifeos_folder: Projects/NeuroDrive
last_synced: 2026-05-31
sources_read: 45
---

# NeuroDrive

## One-line summary

Rust/Bevy 2D driving simulator used as a testbed for a biologically-inspired learning substrate — sparse-graph rate-coded neurons trained by local three-factor plasticity with eligibility traces, raw-reward modulation, biological homeostasis, and continual-backprop structural plasticity — running side-by-side against a handwritten-from-scratch PPO baseline, with no backpropagation in the brain path and no external ML frameworks anywhere.

## What it is

NeuroDrive is a disciplined first-principles investigation into whether biological learning rules can produce coherent real-time continuous control without backpropagation, without weight resets, and without external ML frameworks. The simulator (Monaco-inspired hand-authored 14×9 tile track, 43-dimensional observation, 2-dimensional action) is deterministic and fully-observable; the same agent inhabits it continuously across all episodes and sessions ("one brain, one lifetime"). The project's load-bearing thesis — the **Biology-First Principle** — is that when a pathology appears, the response comes from biology, not from the ML toolkit: no dropout, no batch norm, no experience replay, no Elastic Weight Consolidation unless they have a direct biological analogue.

The project is structured as 11 milestones (plus a flexible biological-realism Long-Term Plan and an out-of-scope Research Frontier). As of 2026-05-13, M1–M6 have shipped. M6 ("Brain-Inspired v1 — the substrate") was completed on 2026-04-19 across six staged commits in one day and is live in master alongside the PPO diagnostic baseline. Behavioural acceptance — the first multi-hour SideBySide wall-clock training run — is the gating outstanding item.

## Architecture

NeuroDrive is structured as seven Bevy plugins plus a `sim` coordination module, with a hard structural boundary between `game/` (environment) and `brain/` (learner) enforced at the Rust module level — `src/game/` contains no `use crate::brain::*` import. Two shared interface types cross the boundary: `ObservationVector` (43-dim `[f32; 43]`, `game → brain`) and `ActionState` (`{ steering: f32, throttle: f32 }`, `brain → game`). Steering is `[-1, 1]`, throttle `[0, 1]` (no reverse; braking was tried and reverted as a safe local optimum).

```
sim
├── GamePlugin
│   ├── MonacoPlugin   (14x9 TrackGrid, TrackCenterline with arc-length parameterisation, O(1) tile collision)
│   └── AgentPlugin    (kinematic car, per-car EnvInstanceId, ObservationVector, ActionState)
├── BrainPlugin        (PpoPlugin + BrainInspiredPlugin — both run, partitioned by ZST markers)
├── AnalyticsPlugin    (per-tick capture, episode summaries, BrainUpdateRecord, two-tier export)
└── DebugPlugin        (F3 HUD, raycast/centreline overlays, F4 layout cycling)
```

The simulation runs at a fixed 60 Hz `FixedUpdate`, partitioned into four causally-ordered `SimSet` stages: `Input` (keyboard/PPO/brain action systems + temporal smoothing), `Physics` (kinematic integration), `Collision` (4-corner rotated-rectangle vs `TrackGrid`), `Measurement` (centreline projection, observation assembly, episode loop, reward, learner update). Frame-rate systems (HUD, overlays) run in the standard `Update` schedule decoupled from physics so the simulation stays deterministic regardless of rendering speed. There is a documented one-tick observation lag — `*_act_all_cars` systems read the `ObservationVector` written by `build_observation_vector` in the previous tick's `Measurement` stage.

PPO updates are amortised across ticks (64 samples per tick from a rolling 512-sample buffer; 4 epochs) so that no single tick stalls the 60 Hz pipeline.

## Subsystems and components

### Brain-Inspired Learner (`src/brain/inspired/`, 7 files, ~50 KB source + ~30 KB tests)

The thesis substrate. A sparse directed graph of rate-coded tanh neurons trained by **three-factor plasticity** `Δw_ij = η · M_c · e_ij[c]` where `M_c` is the **raw per-tick reward** from `EpisodeState.tick.reward` — there is no critic, no TD-error δ, no value module in v1. Per-car eligibility traces decay with `λ = 0.992` (τ_e ≈ 2 s at 60 Hz, matched to γ = 0.995). Eligibility update uses `pre = prev[source]` and `post = curr[target]`, giving STDP-*like* causal semantics from rate-coded neurons without sub-tick spike scheduling.

Graph storage is **slot-stable** (`Vec<Neuron>` and `Vec<Synapse>` with `alive: bool` flags and `free_*_slots: Vec<...Id>` free lists) so structural plasticity events are O(1) and `NeuronId`/`SynapseId` are eternal. Seed graph: 43 `Input(i)` neurons (one per observation dim, bound to `BrainInspiredConfig.obs_dim = 43`), 15 `Hidden`, 2 `Output(i)`, ~10% directed-edge density, weights `Normal(0, σ=0.1)`. Cyclic `hidden→hidden` edges are allowed because the forward pass always reads `prev` and writes `curr`, making it order-independent.

Homeostasis runs on two cadences: per-tick **intrinsic excitability** (EMA `mean_rate` per non-input neuron with `α=0.01`, biases nudge toward `(0.10, 0.60)` band at rate `1e-4`) and on-cadence (`structural_cadence = 128` ticks) **synaptic scaling** (`Σ|w_in|` toward target 2.0 at rate 0.05, clamped `[0.5, 2.0]`).

Structural plasticity follows Continual-Backprop (Dohare et al. 2024) with per-tick utility EMA (`η_u = 0.99`) plus four cadence operations: (1) `replace_low_utility` resamples incoming weights and zeros outgoing weights for the lowest-utility mature hidden neurons; (2) `detect_plateau + grow_hidden_neuron` allocates a new hidden neuron wired to ~10 random sources and targets when the rolling `reward_window` plateaus; (3) `prune_synapses` kills weights below `0.01`; (4) `sprout_synapses` adds new synapses with probability `0.10`. Slot-stable storage makes all of this cheap.

The config surface is 22 fields, every one tagged either **RESEARCH-ANCHORED** (value fixed by the seven-paper research round, e.g. `lambda = 0.992`, `eta_utility = 0.99`, `maturity_ticks = 1000`) or **TUNE** (13 dials awaiting empirical sweep, e.g. `eta = 1e-3`, `replace_fraction = 5e-4`).

### PPO Implementation (`src/brain/ppo/`)

The **permanent diagnostic baseline** — explicitly not retired after M6. Asymmetric actor-critic: actor `43 → 64 → 64 → 2`, critic `43 → 128 → 128 → 1` (critic doubled after a symmetric 2×64 critic reached 40.6% tanh saturation with weight norms of 19.3). Tanh hidden activations everywhere after a ReLU build produced 34–57% dead neurons. Orthogonal init with `√2` scale on hidden layers and **0.01 scale on the actor mean output** to keep the initial policy near zero and exploratory. AdamW with decoupled weight decay `λ = 3e-4` on the critic (actor uses plain Adam, LR `3e-4`); Adam `ε = 1e-5` for stability. `log_std` is a separate learned parameter floor-clamped at `-1.0` (σ ≈ 0.37) after a `-2.0` floor let throttle exploration collapse to 0.07. GAE with γ=0.99 (M5 raised to 0.995 via `gamma` config), λ=0.95, PPO clip ε=0.2, 4 epochs, 512-sample rollout horizon, 64 samples per tick amortised. M5 added PopArt adaptive value normalisation, observation normalisation via running stats, and target-KL early stop.

### Multi-Car Training

8 cars per fleet, each a separate Bevy entity tagged with a unique `EnvInstanceId`. PPO collects all 8 cars' transitions into a single rollout buffer, attributing bootstrap values per car at horizon. The brain-inspired learner accumulates per-car Δw contributions into `accumulated_delta: Vec<f32>` and applies them after every car is visited so that all cars within a tick see the same weights.

### Trainer Layouts and F4 Cycling (post-M6 ZST-marker architecture)

Pre-M6 had a global `AgentMode` enum that could not express "8 PPO + 8 brain on the same track". M6 replaced it with per-car **zero-sized type marker components** — `PpoCar`, `BrainCar`, `KeyboardCar` (each `#[derive(Component)]` ZST) — plus a `Controller` enum on every car for analytics tagging. Bevy queries filter via `(With<Car>, With<PpoCar>)` at the archetype level (O(1)), so PPO systems literally cannot see brain cars and vice versa. `TrainerLayout` enumerates the fleet shapes: `Keyboard`, `AllPpo { count }`, `AllBrain { count }`, `SideBySide { ppo: 8, brain: 8 }` (16 cars: warm palette for PPO, cool palette for brain). F4 cycles between layouts at runtime; `reset_to_seed` rebuilds the brain graph cleanly.

### Observation Vector (`src/agent/observation.rs`)

43-dim normalised `[f32; 43]` (`OBSERVATION_DIM = 43`), layout: `[rays(11), v_forward, v_lateral, lateral_offset, heading_error, angular_velocity, speed_delta, lookahead_headings(12), lookahead_curvatures(12), prev_steering, prev_throttle]`. 11 raycasts at fixed angular offsets `[-150°, -90°, -60°, -35°, -15°, 0°, +15°, +35°, +60°, +90°, +150°]`, up to 375 world units, grid-march with 8-iteration binary-search refinement at boundary. Velocity is **car-local** (`v_forward`, `v_lateral`) not world-space. Lookahead samples 12 points at distances `[30, 650]` along the centreline returning `(heading_delta, curvature)` per sample. Contract is stable and shared: the brain-inspired learner binds each of the 43 dims to a reserved `Input(i)` neuron, so any change desynchronises both learners simultaneously.

### Environment and Track (`src/maps/`, `src/game/`)

Hand-authored Monaco-inspired 14×9 tile grid (100px per tile, 1400×900 world). `TrackGrid` is a flat `bool` array indexed `(tile_x, tile_y)` — `true` = road, `false` = off-track — supporting O(1) collision via 4-corner rotated-rectangle test. `TrackCenterline` stores ordered world-space points with cumulative arc-length parameterisation, enabling O(log n) projection, monotonic progress tracking, and lookahead sampling. Kinematic car model only — no rigid body, no slip, no weight transfer; deterministic at 60 Hz. `rotation_speed = 8.0 rad/s` (raised from 4.0 — insufficient for tight corners at learned speeds), `thrust = 750.0`, `drag = 0.985` per tick.

### Analytics and Export

Per-tick trace capture, per-episode summaries, crash classification, reward decomposition, pre-crash analytics, ranking. Generates 10-section diagnostic markdown reports plus a Fleet Comparison section (§19) that auto-detects side-by-side from `Controller` tags. `BrainUpdateRecord` (serde) carries per-window counts (replacements, neurogenesis, prune, sprout), plasticity-health scalars, utility percentiles, and structural state. Pre-M6 JSON files deserialise cleanly via `#[serde(default)]` on the new fields. Report file slug encodes the layout: `run_<timestamp>_<slug>.md` where slug is `brain`/`side`/`ppo`/`keyboard`; PPO-centric sections (9, 12, 13, 14) are entirely skipped in brain-only reports rather than stubbed.

### Debug and HUD, Profiling, Determinism

F3 HUD with PPO training stats and per-layer health (weight L2 norm, gradient L2 norm, tanh saturation fraction). Feature-gated per-system timing. Determinism layers documented: kinematic physics is bit-deterministic at fixed 60 Hz; RNG seeded via `config.rng_seed` when present; brain RNG state is not yet serialised across sessions (Gap #1).

### Performance (M4 overhaul, commit `3c512f9` and the surrounding chain)

Driven by an 8 GB unified-memory M2 MacBook Air at 60 Hz. Pre-overhaul: 426 stutters per run, mean frame time 17.3 ms with 8 cars. Post-overhaul: 2 stutters per run, mean 9.0 ms — a 48% improvement and 426→2 stutter reduction. The improvements were memory-layout and batching changes, not algorithmic: flat row-major `Vec<f32>` weights replacing `Vec<Vec<f32>>`, `BatchScratch` pre-allocated once (max 512), batched mat-mat instead of 128× mat-vec, iterator chains enabling LLVM auto-vectorisation, `std::mem::swap` for frozen buffers, precomputed Adam bias correction via `powi` once per step. Dual GEMM backend with SIMD fallback plus Apple Accelerate AMX path.

## Technologies and concepts demonstrated

### Languages

- **Rust** — entire codebase. 80 `.rs` files, ~660 KB source plus ~45 KB integration tests. Heavy use of Bevy ECS idioms (Components, Resources, Systems, Plugins, Queries with marker filters), disjoint-query patterns on the same entity set (`brain_learn_all_cars_system` uses two queries on disjoint component sets), field-level destructuring of `&mut self` to satisfy the borrow checker when multiple sub-fields are needed simultaneously.

### Frameworks and libraries

- **Bevy** — ECS engine. Plugins-and-Systems architecture, `FixedUpdate` for the 60 Hz simulation, `Update` for frame-rate-decoupled rendering, `Last` schedule for shutdown flush. Per-archetype filtering via ZST marker components is leveraged as the core of the post-M6 trainer-layout design.
- **serde** — serialisation of `BrainUpdateRecord`, `CompactRunExport`, `EpisodeTracker`. `#[serde(default)]` used for forward/backward-compatible JSON across the pre/post-M6 boundary.
- **rand** / `StdRng` — seedable RNG on `BrainBrain`, plus `rand::rng()` fallback when no seed is configured.
- **No external ML frameworks.** Explicitly excluded: PyTorch, tch-rs, candle. Every component (network, optimiser, GEMM, gradient computation) is handwritten Rust.

### Runtimes / engines / platforms

- **Bevy ECS** — coordinates seven plugins + the `sim` module across four `SimSet` stages.
- **Apple Accelerate AMX** — used in the dual GEMM backend as the fast path on Apple Silicon (vs SIMD fallback for portability).
- **CPU-only execution on Apple Silicon (M2 MacBook Air, 8 GB unified memory).** Deliberate hardware constraint — "if the architecture cannot run at 60 Hz on a MacBook Air, it is not the right architecture."

### Tools

- Per-system feature-gated profiling infrastructure.
- F3 HUD for live diagnostics; F4 keyboard cycle for layout switching.
- Tracked `tanh_saturation_fraction` and `dead_neuron_fraction` as architectural-health signals — these triggered the ReLU→tanh switch and the actor/critic asymmetric resizing decisions.

### Domains and concepts

- **Biologically-plausible learning.** Three-factor plasticity (`Δw = η · M · e`) with per-car eligibility traces (`e ← λ·e + pre·post`, `λ=0.992`, τ_e ≈ 2 s at 60 Hz). STDP-like causal pre/post ordering from rate-coded neurons without sub-tick spike scheduling. Raw-reward modulator (v1, Option C — Options A "delta-as-dopamine"/B "plasticity-trained value predictor" explicitly deferred to M8).
- **Homeostasis.** Intrinsic excitability (mean-rate EMA + bias nudge into a target rate band) and synaptic scaling (incoming-weight-sum drift toward a target).
- **Continual-backprop structural plasticity** (Dohare et al. 2024). Utility EMA per neuron, periodic replacement of low-utility mature neurons, plateau-triggered neurogenesis, prune/sprout balance.
- **Sparse directed graph topology, not layered.** Cyclic `hidden→hidden` connections permitted; correctness preserved by the `prev`/`curr` read-write split.
- **PPO from scratch.** Asymmetric actor-critic, GAE, clipped surrogate, AdamW with decoupled weight decay on the critic, PopArt adaptive value normalisation, observation running-stats normalisation, target-KL early stop, tanh-squashed actions with Jacobian correction.
- **Multi-agent vectorised RL** with per-`EnvInstanceId` rollout attribution and per-car bootstrap values at horizon.
- **Continual / one-lifetime learning.** No weight resets between episodes; the same parameters adapt continuously. Catastrophic forgetting is framed as the central challenge, not as a failure mode to avoid.
- **Bevy ECS architecture patterns.** ZST marker components as compile-time partitioning; controller-agnostic environment boundary enforced at the module level.
- **Deterministic real-time simulation** at fixed 60 Hz with causally-ordered staged pipeline.
- **Performance engineering on a constrained budget** (16.67 ms per tick, 8 GB unified memory): SoA-style flat buffers, batching, amortisation, Apple AMX GEMM.

## Key technical decisions

Drawn from the 10 dedicated Decisions notes plus the Brain-v1 Implementation Log (D1–D23). The most load-bearing:

- **Biology-First Principle.** When pathologies appear, the response comes from biology, not from the ML toolkit. Rules out reaching for dropout, batch norm, experience replay, EWC, etc. as defaults; demands biological motivation for any rescue mechanism.
- **One Brain, One Lifetime.** No weight resets across episodes or sessions; no population/evolutionary methods; no generational switching; no external ML frameworks; no backpropagation in the brain-inspired learner (PPO uses it because PPO is the diagnostic baseline). The same parameters adapt continuously.
- **Raw Reward as Modulator (Option C, v1).** Chosen over Option A (delta-as-dopamine TD-error, no value module to source δ from) and Option B (plasticity-trained value predictor — deferred to M8 as the natural next step if v1 is insufficient). Explicit anti-puffing rule: v1 has no critic, no TD error, no dopamine-encoded prediction error.
- **Graph, not layered.** Sparse directed graph with cyclic connections allowed; topology is the substrate, not the design surface. Forward pass reads `prev`, writes `curr`, eliminating ordering concerns.
- **Slot-stable graph storage.** `Vec<Neuron>`/`Vec<Synapse>` with `alive: bool` and free-lists; rejected alternatives were `HashMap` (allocator churn and pointer chasing) and compacted `Vec` (every death is O(n) shift and invalidates downstream IDs).
- **PPO stays as permanent diagnostic baseline** (Decision D21). Pre-restructure the plan was to replace PPO with the biological brain; post-restructure PPO is additive and lives forever in side-by-side mode. The brain-inspired learner's behaviour is only measurable against a known-working reference.
- **ZST marker components over a global `AgentMode` enum.** Compile-time partitioning at the archetype level enables `SideBySide { ppo: 8, brain: 8 }` without any runtime branching or cross-fleet aliasing risk.
- **Reward design: velocity projection + centreline proximity, no lap-completion bonus.** Lap detection was deliberately removed — the +100 lap bonus created reward-cliff pathologies. Episodes end only on crash or 30-second timeout. Braking (`throttle ∈ [-1,1]`) was tried and reverted: policy converged to "mostly brake" (mean -0.60) as a safe local optimum.
- **Tanh activations throughout** after a ReLU build produced 34–57% dead neurons. Tanh-squashed actions with explicit Jacobian correction for log-prob computation; `log_std` floor raised `-2.0 → -1.0` after throttle exploration collapsed to σ = 0.07.
- **Critic widened to 2×128 + AdamW with weight decay `λ=3e-4`** specifically to fix 40.6% tanh saturation and weight norms of 19.3 in the symmetric 2×64 critic — asymmetric sizing motivated by measured pathology, not by guess.
- **STDP deferred to the Long-Term Plan.** Despite prominent mention in older notes, full STDP requires LIF neurons and sub-tick scheduling. The rate-coded `pre = prev[source]` / `post = curr[target]` ordering gives STDP-like semantics today without that architectural cost.

## What is currently built

- **Environment, kinematics, and observation contract** (M1) — Monaco track, 60 Hz fixed-tick pipeline, 4-corner collision, arc-length centreline, full 43-dim observation, keyboard car for sanity.
- **From-scratch PPO baseline** (M2) — actor `43→64→64→2`, critic `43→128→128→1`, orthogonal init, AdamW on critic, GAE, clipped surrogate, tanh-squashed actions, Jacobian correction.
- **8-car vectorised training + analytics pipeline** (M3) — per-`EnvInstanceId` rollout, 10-section markdown reports, crash classification, reward decomposition, pre-crash analytics, ranking.
- **Performance overhaul** (M4) — dual GEMM backend (SIMD fallback + Apple Accelerate AMX), batched actor, centreline-query optimisations. 21× frame-time improvement; stutters 426 → 2, mean frame time 17.3 ms → 9.0 ms.
- **Critic target-scaling** (M5) — PopArt, γ = 0.995, observation normalisation, target-KL early stop. Validation run `run_1776556719.md` confirmed all 8 cars completing the full Monaco loop, fleet max-progress spread 1.1%, crash rate falling 100% → 56% in the best chunk, 96% of crashes with throttle released > 0.25 s before impact (the policy anticipates collision). Environment confirmed learnable.
- **Brain-inspired v1 substrate** (M6, shipped 2026-04-19 across six staged commits) — full sparse-graph topology, slot-stable storage, forward pass, three-factor plasticity with eligibility traces, raw-reward modulator, intrinsic + synaptic homeostasis, continual-backprop utility tracking, replacement/neurogenesis/prune/sprout structural plasticity, analytics integration (`BrainUpdateRecord` + 3 markdown sections + Fleet Comparison auto-detection), side-by-side trainer layout with palette differentiation. Default layout is `AllBrain`.
- **External-visibility shipping** — LinkedIn article + 3-GIF README embed (2026-04-29).

Test count: **133 green** across default, `force-scalar`, and release builds (101 unit + 21 brain-pipeline + 6 GEMM + 5 PPO). Every M6 test verifies mechanics (eligibility decays, no NaN over 10k ticks, terminal zeroing, synaptic-scaling reaches target, replacement picks lowest-utility mature neurons, plateau-detector fires on flat reward, neurogenesis grows neuron count, etc.) — **none verify behavioural learning**, by design.

## Current state

**Status:** active. **Last meaningful repo activity:** 2026-04-29 (docs + media rename), HEAD `141be5b`. **Last code/feature commit:** 2026-04-19. The 24-day gap between M6 ship and the next code commit is documented as planned pause time pending the first wall-clock SideBySide training run — not dormancy. The post-M6 window has been external-visibility shipping (LinkedIn article + 3-GIF README embed) rather than additional substrate work. M7 (live brain visualisation) is queued to begin after M6 behavioural acceptance.

## Gaps and known limitations

1. **First real SideBySide training run not yet done.** M6 code shipped 2026-04-19 with 21 brain-pipeline tests green, all testing mechanics — none testing whether the brain produces meaningful driving behaviour. The success bar (rising brain-fleet reward trend over ~2000 episodes, directional bias signature observable in analytics, at least one replacement + one neurogenesis event during the run) requires hours of wall-clock training that has not yet been spent. This is the single most important outstanding item.
2. **Brain persistence across sessions is unimplemented.** Neither `BrainBrain.graph` weights, per-car eligibility, `BrainTrainingStats.history`, `tick_counter`, nor RNG state is serialised to disk. Every session begins fresh. PPO weights + AdamW state + PopArt tracking are similarly lost. More costly for the brain-inspired learner than for PPO because per-episode learning is slower and structural events only fire on cadence. Strictly violates "one brain, one lifetime" at the session boundary.
3. **Tuning sweep not run.** 13 of 22 `BrainInspiredConfig` fields are `TUNE` (plausible starting points, no empirical backing) — `eta`, `replace_fraction`, `structural_cadence`, `plateau_threshold`, `prune_weight_threshold`, `sprout_probability`, `synaptic_scaling_rate`, `intrinsic_bias_rate`, etc. The first SideBySide run is the prerequisite for the first informed tuning round.
4. **Post-M6 `context/` upkeep is partial.** `context/architecture.md` and `context/systems/*` still describe the pre-M6 global-`AgentMode` world in several places.
5. **Live HUD column split for side-by-side is pending** (report-level done in M6 S6).
6. **No multi-track support.** Only Monaco exists; multi-track is the prerequisite for M10 continual-learning experiments.
7. **Unified user-controllable initialisation seed.** `BrainBrain.rng` honours `config.rng_seed` but it is not exposed on the CLI; PPO's model init seed is not similarly controllable.
8. **Lap detection was removed deliberately.** Reward-cliff pathology from the +100 lap bonus motivated the removal. Implication: agent cannot distinguish "completed a full lap" from "drove 30 s without crashing".
9. **No PPO integration tests beyond unit scope.** 5 PPO tests cover GAE + update mechanics; no explicit convergence test.
10. **Sprout/prune balance is undocumented.** Config comment claims "roughly balances" but no empirical data is cited. The first run should log `prune_events / sprout_events` per cadence and analyse the ratio.

## Direction (in-flight, not wishlist)

- **First SideBySide wall-clock training run** — the gating M6-acceptance item.
- **M7 brain visualisation** — live graph inspector (neurons rendered as dots with utility/degree encoding, synapses as weighted lines, animated activations, surfaced structural events, observation→input heat-map). Begins after M6 validates. Framed in the README as the project's "emotional core" — "watch a brain grow" is what differentiates it from reading a training log.
- **M8 plastic value predictor (Option B)** — natural next step if v1's raw-reward modulator proves insufficient; a plasticity-trained `V(s)` producing a TD-error-analog `M_new = r + γV(s') − V(s)` without backpropagation. Several open design questions tracked (where V lives, what rule trains V, how V's output is normalised, whether V shares the main graph's plasticity dials).
- **Proposed publication-arc work file (`Work/Continual ImageNet Adapter.md`, status `proposed`)** — Permuted MNIST 200-task adapter + paper-grade write-up positioning the M6 substrate as a continual-learning method, comparing against MESU / RDBP / EWC / Synaptic Intelligence. Sequencing gate: requires M6 SideBySide validation first. Additive — substrate untouched.
- **Proposed infrastructure-arc work file (`Work/CubeCL Kernel Pack.md`, status `proposed`)** — CubeCL backend kernels (CUDA + Metal + Vulkan + ROCm + WGPU) for the M6 primitives, plus an MLX operator pack for Apple Silicon depth. CPU paths stay as correctness oracles. Same sequencing gate.

## Demonstrated skills

- Designs and implements a complete reinforcement-learning training stack in safe Rust with zero external ML framework dependency — handwritten Adam/AdamW with per-layer state and AdamW-style decoupled weight decay, PPO with GAE and clipped surrogate, orthogonal initialisation with scale-differentiated heads, tanh-squashed actions with Jacobian correction, PopArt adaptive value normalisation, observation running-stats normalisation, target-KL early stop.
- Builds a non-trivial biologically-inspired learner from scratch: sparse directed graph of rate-coded tanh neurons, three-factor plasticity with per-car eligibility traces, biological homeostasis on two cadences (intrinsic excitability + synaptic scaling), continual-backprop utility tracking with mature-neuron replacement, plateau-triggered neurogenesis, prune/sprout structural plasticity, all running live alongside PPO in the same Bevy simulation.
- Architects a controller-agnostic environment boundary enforced at the Rust module level (no `use crate::brain::*` in `src/game/`), with two narrow shared interface types (`ObservationVector`, `ActionState`) that survived the introduction of a totally new learner without environment changes.
- Designs ECS-native compile-time partitioning via zero-sized type marker components so that two different learners can train simultaneously in the same simulation with zero cross-fleet aliasing risk by construction.
- Performance-engineers a 60 Hz real-time simulation on an 8 GB-unified-memory M2 MacBook Air: 426 → 2 stutters, 17.3 ms → 9.0 ms mean frame time. Specific techniques: flat row-major weight buffers, pre-allocated `BatchScratch`, batched mat-mat replacing mat-vec, iterator chains for LLVM auto-vectorisation, `mem::swap` for frozen buffers, precomputed Adam bias correction, dual GEMM backend with Apple Accelerate AMX path plus SIMD fallback.
- Diagnoses architectural pathology from instrumentation: ReLU→tanh switch driven by measured 34–57% dead-neuron fraction; critic widened 2×64→2×128 driven by measured 40.6% tanh saturation and weight-norm 19.3; `log_std` floor raised after measured throttle σ collapse to 0.07; `popart_beta` bumped 1e-4→3e-2 after PopArt drifted off real returns.
- Practices research-discipline restraint: the Biology-First Principle explicitly rules out reaching for ML defaults (dropout, batch norm, experience replay, EWC) and the project documents "biology-first under pressure" — if the brain doesn't learn on the first run, the prescribed move is to diagnose in biology terms (plasticity too fast/slow, dead neurons the homeostat should rescue, structural churn not settling) rather than import PPO's critic as the modulator.
- Maintains documentation discipline that supports the research: 23-decision implementation log for M6 alone, dedicated learning notes for every biological mechanism cited, explicit anti-puffing tables in implementation reality docs ("what the README implies vs what the code does") so future readers don't misread.
- Carries an honest-negative-result framing: shipping M6 with "visible plasticity signature, no loop completion yet" is the planned success bar; the project's contribution is the falsification, not a forced success narrative.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/NeuroDrive/_Overview.md | 237 | "The window 2026-04-19 → 2026-04-29 was a docs / external-visibility burst; no source-code commits landed. The next code-level work begins when the first SideBySide training run is launched." |
| Projects/NeuroDrive/Gaps.md | 237 | "#neurodrive #gaps #technical-debt" |
| Projects/NeuroDrive/Architecture/Fixed Tick Pipeline.md | 150 | "#neurodrive #rust #architecture #bevy #simulation" |
| Projects/NeuroDrive/Architecture/Module Boundaries.md | 143 | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Architecture/Module Map.md | 92 | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Decisions/_Overview.md | 40 | "- [[Projects/NeuroDrive/Roadmap]] — milestone arc reflecting these decisions" |
| Projects/NeuroDrive/Decisions/Biology First Principle.md | 145 | "#neurodrive #decisions #biology-first #discipline" |
| Projects/NeuroDrive/Decisions/Brain v1 Implementation Log.md | 141 | "#neurodrive #decisions #implementation-log #milestone-6" |
| Projects/NeuroDrive/Decisions/Graph Not Layered.md | 183 | "#neurodrive #decisions #topology #graph #milestone-6" |
| Projects/NeuroDrive/Decisions/LinkedIn Article + Gifs Shipped.md | 42 | "- [[Projects/NeuroDrive/Roadmap]] § M11 Writeup/Release" |
| Projects/NeuroDrive/Decisions/One Brain One Lifetime.md | 127 | "#neurodrive #decisions #continual-learning #core-constraint" |
| Projects/NeuroDrive/Decisions/PPO as Baseline.md | 123 | "#neurodrive #decisions #ppo #milestone-1" |
| Projects/NeuroDrive/Decisions/Raw Reward as Modulator.md | 136 | "#neurodrive #decisions #neuromodulation #option-c #milestone-6" |
| Projects/NeuroDrive/Decisions/Reward Design.md | 142 | "#neurodrive #decisions #reward-design" |
| Projects/NeuroDrive/Decisions/Slot Stable Graph Storage.md | 191 | "#neurodrive #decisions #data-structures #structural-plasticity #milestone-6" |
| Projects/NeuroDrive/Decisions/Tanh Squashed Actions.md | 139 | "#neurodrive #decisions #ppo #action-space" |
| Projects/NeuroDrive/Learning/_Overview.md | 38 | "- [[Projects/NeuroDrive/Decisions]] — recorded design decisions" |
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
| Projects/NeuroDrive/Systems/_Overview.md | 48 | "- [[Projects/NeuroDrive/Roadmap]] — direction-of-travel" |
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

**Anomalies:** None. All 45 markdown files in `Projects/NeuroDrive/` (recursive) fetched cleanly and synthesised into the per-project file. No section of the schema was without LifeOS source evidence. Status set to `active` per Overview frontmatter (`status: active`, tag `#project-active`) and the documented framing that the 24-day post-M6 gap is a planned pause for behavioural validation rather than dormancy. The "Subsystems and components" section is comprehensive because LifeOS Systems/ contains 11 detailed subsystem files plus an Overview. Two `Work/` files (`Continual ImageNet Adapter`, `CubeCL Kernel Pack`) are surfaced under "Direction" with explicit `status: proposed` framing rather than as built capabilities.
