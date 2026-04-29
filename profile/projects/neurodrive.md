---
name: NeuroDrive
status: active
source_repo: https://github.com/Capataina/NeuroDrive
lifeos_folder: Projects/NeuroDrive
last_synced: 2026-04-29
sources_read: 44
---

# NeuroDrive

## One-line summary

Rust + Bevy 2D top-down autonomous-driving simulator used as a research testbed for biologically-inspired learning — sparse-graph topology with three-factor plasticity, eligibility traces, raw-reward neuromodulation, homeostasis, and continual-backprop structural plasticity, running alongside a hand-rolled PPO baseline as the diagnostic comparator.

## What it is

NeuroDrive is a Rust/Bevy 2D top-down autonomous driving simulator used as a testbed for biologically-inspired learning research. It is not a game and not a benchmark — it is a disciplined first-principles investigation into whether biological learning rules can produce coherent real-time control behaviour without backpropagation, without weight resets, and without external ML frameworks. The simulator provides a deterministic, fully-observable environment. A learning agent inhabits it continuously. The goal is to drive — to navigate a Monaco-inspired track, maintain speed, stay on the racing line. The means of achieving that goal is the entire research programme. The core research question is whether **biologically-inspired learning rules — local synaptic plasticity, eligibility traces, reward-gated updates, homeostasis, structural adaptation — can produce coherent autonomous-driving behaviour in a real-time control task, given a single agent that learns continuously across its entire lifetime, and given no backpropagation**. As of 2026-04-24 the project has completed M1–M6 and shipped the brain-inspired substrate (M6, 2026-04-19); the first real SideBySide training run (the behavioural acceptance bar) has not yet been done.

## Architecture

Bevy ECS with a strict 4-stage `SimSet` fixed-tick pipeline running at 60Hz. Module map:

```
src/
├── brain/inspired/        # M6 v1 — sparse-graph plasticity (7 files, ~50KB)
│   ├── config.rs          # Three-factor rule parameters
│   ├── forward.rs         # Sparse forward pass over slot-stable graph
│   ├── graph.rs           # Slot-stable storage + free-list (alive flag)
│   ├── plasticity.rs      # Hebbian + eligibility-trace + neuromodulator
│   ├── homeostasis.rs     # Synaptic scaling + intrinsic excitability
│   ├── structural.rs      # Continual-backprop utility / replace / prune / sprout
│   └── mod.rs
├── ppo/                   # Diagnostic baseline: asymmetric actor-critic
├── env/                   # Monaco track, kinematics, episode lifecycle
├── obs/                   # 43-dim observation vector (stable contract)
├── reward/                # Velocity-projection + centreline reward
├── analytics/             # Two-tier export (per-tick / per-episode), markdown reports
├── trainer/               # SideBySide layouts + F4 cycling
├── debug/                 # HUD panels, leaderboards
└── main.rs                # Bevy app, plugin composition, system ordering
```

Test count: **133 green** across default, `force-scalar`, and release builds (101 unit + 21 brain-pipeline + 6 gemm + 5 ppo).

## Subsystems and components

| Subsystem | Responsibility | Key files |
|---|---|---|
| **Brain-Inspired Learner** | M6 v1 — sparse-graph plasticity with three-factor learning rule, eligibility traces, raw-reward modulator, homeostasis, structural plasticity (CBP) | `src/brain/inspired/` |
| **PPO Implementation** | Diagnostic baseline: asymmetric actor-critic (2×64 actor, 2×128 critic), AdamW, PopArt critic target scaling, dual GEMM backend (Accelerate AMX on macOS, fallback elsewhere), γ=0.995, target-KL stop | `src/ppo/` |
| **Environment and Track** | Monaco-inspired track, kinematics, episode lifecycle | `src/env/` |
| **Observation Vector** | 43-dim observation (stable contract across all milestones) | `src/obs/` |
| **Episode and Reward** | Velocity-projection + centreline reward; no crash penalty | `src/reward/` |
| **Multi Car Training** | 8-car vectorised training with ranking | `src/trainer/multi_car.rs` |
| **Analytics and Export** | Two-tier export, layout-aware reports, 16-field tick capture, 25-field episode aggregates, 10-section diagnostic Markdown reports | `src/analytics/` |
| **Debug and HUD** | HUD panels, leaderboard | `src/debug/` |
| **Profiling** | Feature-gated per-system timing | `src/profiling/` |
| **Determinism** | Multi-layer determinism; reproducibility status documented | `src/determinism/` |
| **Trainer Layouts and F4 Cycling** | ZST markers, side-by-side, F4 cycle for switching layouts | `src/trainer/layouts.rs` |

## Technologies and concepts demonstrated

### Languages
- **Rust 2021** — entire codebase. 80 `.rs` files, 660KB. Three integration test files (45KB). 36 markdown context docs (939KB). 60 learning files (504KB).

### Frameworks
- **Bevy** — ECS architecture, fixed-timestep simulation, plugin composition, system ordering sets, world-space debug overlays, UI panels.

### Libraries
- **Hand-rolled PPO** — no `tch`, no `candle`, no PyTorch. Asymmetric actor-critic, GAE, clipped surrogate objective, PopArt critic target scaling, dual GEMM (Accelerate AMX + fallback) for ~21× frame-time improvement at M4.

### Engines and runtimes
- **Bevy ECS** — simulation runtime; Bevy plugin composition across 8 subsystems.

### Domains and concepts
- **Reinforcement learning, framework-free** — PPO with clipped surrogate, GAE, asymmetric actor-critic, PopArt target scaling, target-KL stop, observation normalisation. Hand-rolled, no ML framework dependency.
- **Biologically-inspired learning** — three-factor plasticity rule (`Δw = η · pre · post · M`), eligibility traces (λ=0.992 default, terminal zeroing), raw-reward modulator (Option C, v1), Hebbian co-activity (rate-coded), homeostasis (synaptic scaling + intrinsic excitability), continual-backprop structural plasticity (utility tracking + replacement / prune / sprout).
- **Graph-not-layered topology** — sparse graph with slot-stable storage (`alive` flag + free-lists) instead of fixed layered topology.
- **One Brain, One Lifetime constraint** — same parameters adapt continuously across all episodes; no weight resets, no population methods, no backpropagation in the brain-inspired learner.
- **Performance engineering on commodity hardware** — M2 MacBook Air, all CPU, no CUDA. M4 milestone delivered ~21× frame-time improvement via dual-GEMM + batched actor.
- **Custom analytics pipeline** — 16-field tick capture, 25-field episode aggregates, 10-section diagnostic Markdown reports, layout-aware exports.

## Key technical decisions

- **Biology First Principle.** When NeuroDrive hits a problem, the answer comes from biology, not from the ML toolkit. Rules out dropout, batch norm, experience replay, EWC unless they have a direct biological analogue.
- **One Brain, One Lifetime.** No weight resets, no populations, no backpropagation in the brain-inspired learner, no external ML frameworks. Catastrophic forgetting is not a failure mode to be avoided — it is the central challenge to be solved.
- **Graph-not-layered topology.** Slot-stable storage with `alive` flag + free-lists; structural plasticity grows / prunes / replaces neurons.
- **Raw reward as modulator (Option C).** Rejected Option A (TD-error direction) and Option B (predicted-vs-actual). Option C is the v1 simplification — raw reward modulates plasticity directly.
- **PPO as diagnostic baseline, not retired.** PPO stays alongside the brain-inspired learner; it is the comparator that lets the brain-inspired arc be falsified.
- **Tanh-squashed actions with Jacobian correction.** Action bounding for the PPO side; mathematically correct change-of-variables.
- **Reward design: velocity projection + centreline, no crash penalty.** Rewards along-track velocity; centreline distance penalty; no crash penalty (deliberate — let the agent discover crash boundaries).
- **STDP deferred.** Despite being prominently named in early notes, STDP requires LIF neurons + sub-tick scheduling. Rate-coded `pre · post` gives STDP-like causal semantics without sub-tick scheduling.
- **PopArt critic target scaling, γ=0.995, target-KL stop.** M5 critic-stability work.
- **Dual GEMM backend (M4 perf overhaul).** Accelerate AMX on macOS, fallback elsewhere. ~21× frame-time improvement.

## What is currently built

- M1: Environment + keyboard controller — complete.
- M2: PPO baseline from scratch — complete.
- M3: Multi-car + analytics pipeline — complete.
- M4: Performance overhaul — complete (~21× frame-time improvement via dual GEMM + batched actor).
- M5: Critic target-scaling (PopArt, γ, obs norm, target-KL) — complete.
- M6: Brain-inspired v1 — code complete (shipped 2026-04-19); behavioural acceptance pending first SideBySide training run.
- 80 Rust files, 660KB source. 133 tests green. Brain-inspired module: 7 files, ~50KB source + ~30KB tests.
- 36 context documents (939KB) — design history, decisions, learning theory.
- 60 learning archive files (504KB) — educational corpus from PPO theory to biological learning rules.

## Current state

Active. M6 shipped 2026-04-19; awaiting first behavioural validation run. M7 (brain visualisation) is next after M6 validates.

## Gaps and known limitations

- **M6 behavioural validation pending** — code is complete but no SideBySide training run has produced a quality bar yet.
- **STDP still in Long-Term Plan, not in any milestone** — requires LIF neurons + sub-tick scheduling.
- **No CUDA path** — M2 MacBook Air constraint; if architecture cannot run at 60Hz on commodity hardware, it is the wrong architecture.
- **Open performance items file currently empty** — closed 2026-04-19; performance work paused for M7+.

## Direction (in-flight, not wishlist)

- **M7 — Brain visualisation.** Next after M6 behavioural validation.
- **M8 — Plastic value predictor (Option B).** Future biology-mechanism milestone.
- **M9 — Multi-neuromodulator refinement.** Future.
- **M10 — Evaluation (multi-track, transfer, curriculum).**
- **M11 — Writeup / release preparation.**
- **Long-Term Plan (ordering flexible)**: Dale's law, synaptic delays, short-term synaptic dynamics (Tsodyks-Markram), multiple neuron types (pyramidal + interneurons), sleep/replay consolidation, spiking neurons + STDP.

## Demonstrated skills

- **Reinforcement learning from first principles** — handwritten PPO with clipped surrogate objective, GAE, asymmetric actor-critic (2×64 actor, 2×128 critic), AdamW, PopArt critic target scaling, dual GEMM backend, target-KL stop. No `tch`, no `candle`, no PyTorch — every learning rule from first principles.
- **Biologically-inspired learning research** — three-factor plasticity rule, eligibility traces, raw-reward modulator, homeostasis, continual-backprop structural plasticity. Sparse graph topology with slot-stable storage. Falsifiable arc against PPO baseline.
- **Bevy ECS architecture** — 8 subsystems with plugin composition, system ordering sets, fixed-timestep simulation, world-space debug overlays, UI panels, F4-cycled trainer layouts.
- **Performance engineering on commodity hardware** — M4 ~21× frame-time improvement via dual-GEMM (Accelerate AMX + fallback) + batched actor; M2 MacBook Air target as a deliberate constraint.
- **Custom analytics pipeline** — 16-field tick capture, 25-field episode aggregates, 10-section diagnostic Markdown reports.
- **Determinism + reproducibility** — multi-layer determinism design with documented reproducibility status.
- **Project discipline / first-principles framing** — Biology First Principle as a load-bearing decision; One Brain, One Lifetime as a core constraint that defines the research programme; PPO retained as a diagnostic baseline rather than discarded after M2.
- **Cross-domain Rust** — adds to Caner's Rust portfolio: TUI (Cernio), DeFi RPC (Aurix), desktop ML (Image Browser), HFT-style systems (Nyquestro), now ECS + RL + biological learning (NeuroDrive).

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/NeuroDrive/_Overview.md | 215 | "- `9882a63` (2026-04-24) — Session 2026-04-24: 14-project extraction + Strategy Research plan + Profile-reorg cleanup" |
| Projects/NeuroDrive/Gaps.md | 234 | "#neurodrive #gaps #technical-debt" |
| Projects/NeuroDrive/Architecture/Fixed Tick Pipeline.md | 150 | "#neurodrive #rust #architecture #bevy #simulation" |
| Projects/NeuroDrive/Architecture/Module Boundaries.md | 143 | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Architecture/Module Map.md | 92 | "#neurodrive #rust #architecture #bevy" |
| Projects/NeuroDrive/Decisions/_Overview.md | 39 | "- [[Projects/NeuroDrive/Roadmap]] — milestone arc reflecting these decisions" |
| Projects/NeuroDrive/Decisions/Biology First Principle.md | 145 | "#neurodrive #decisions #biology-first #discipline" |
| Projects/NeuroDrive/Decisions/Brain v1 Implementation Log.md | 141 | "#neurodrive #decisions #implementation-log #milestone-6" |
| Projects/NeuroDrive/Decisions/Graph Not Layered.md | 183 | "#neurodrive #decisions #topology #graph #milestone-6" |
| Projects/NeuroDrive/Decisions/One Brain One Lifetime.md | 127 | "#neurodrive #decisions #continual-learning #core-constraint" |
| Projects/NeuroDrive/Decisions/PPO as Baseline.md | 123 | "#neurodrive #decisions #ppo #milestone-1" |
| Projects/NeuroDrive/Decisions/Raw Reward as Modulator.md | 136 | "#neurodrive #decisions #neuromodulation #option-c #milestone-6" |
| Projects/NeuroDrive/Decisions/Reward Design.md | 142 | "#neurodrive #decisions #reward-design" |
| Projects/NeuroDrive/Decisions/Slot Stable Graph Storage.md | 191 | "#neurodrive #decisions #data-structures #structural-plasticity #milestone-6" |
| Projects/NeuroDrive/Decisions/Tanh Squashed Actions.md | 139 | "#neurodrive #decisions #ppo #action-space" |
| Projects/NeuroDrive/Learning/_Overview.md | 38 | "- [[Projects/NeuroDrive/Decisions]] — recorded design decisions" |
| Projects/NeuroDrive/Learning/Continual Backprop Utility.md | 174 | "#neurodrive #biologically-inspired #continual-backprop #utility-tracking #neurons" |
| Projects/NeuroDrive/Learning/Eligibility Traces.md | 225 | "#neurodrive #biologically-inspired #eligibility-traces #temporal-credit-assignment" |
| Projects/NeuroDrive/Learning/Hebbian Plasticity.md | 168 | "#neurodrive #biologically-inspired #hebbian-plasticity #milestone-6" |
| Projects/NeuroDrive/Learning/Homeostasis.md | 191 | "#neurodrive #biologically-inspired #homeostasis #synaptic-scaling #intrinsic-excitability" |
| Projects/NeuroDrive/Learning/Neuromodulation.md | 177 | "#neurodrive #biologically-inspired #neuromodulation #dopamine #reward-prediction" |
| Projects/NeuroDrive/Learning/STDP.md | 213 | "#neurodrive #biologically-inspired #stdp #spiking-neural-networks #long-term-plan" |
| Projects/NeuroDrive/Learning/Structural Plasticity.md | 251 | "#neurodrive #biologically-inspired #structural-plasticity #continual-backprop #milestone-6" |
| Projects/NeuroDrive/Learning/Three Factor Learning Rule.md | 199 | "#neurodrive #biologically-inspired #three-factor-learning #milestone-6 #reinforcement" |
| Projects/NeuroDrive/Roadmap/Milestone 2 Biological Brain.md | 211 | "#neurodrive #roadmap #milestone-2 #biologically-inspired #three-factor-learning" |
| Projects/NeuroDrive/Roadmap/Milestone 6 Brain Inspired v1.md | 155 | "#neurodrive #roadmap #milestone-6 #brain-inspired #shipped" |
| Projects/NeuroDrive/Roadmap/Milestone Overview.md | 312 | "#neurodrive #roadmap #milestones" |
| Projects/NeuroDrive/Roadmap/Milestones 4 to 8.md | 267 | "#neurodrive #roadmap #milestones #long-horizon" |
| Projects/NeuroDrive/Systems/_Overview.md | 48 | "- [[Projects/NeuroDrive/Roadmap]] — direction-of-travel" |
| Projects/NeuroDrive/Systems/Analytics and Export.md | 228 | "#neurodrive #rust #analytics #observability" |
| Projects/NeuroDrive/Systems/Brain-Inspired Learner.md | 416 | "#neurodrive #biologically-inspired #brain-inspired-learner #milestone-6 #three-factor" |
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
