---
name: Nyquestro
status: active
source_repo: https://github.com/Capataina/Nyquestro
lifeos_folder: Projects/Nyquestro
last_synced: 2026-05-31
sources_read: 22
---

# Nyquestro

## One-line summary

From-scratch deterministic multi-instrument limit order matching engine in safe Rust (no `unsafe`), with a live Ratatui observability dashboard, an unauthenticated Coinbase Advanced Trade WebSocket bridge for real BTC-USD/ETH-USD/SOL-USD depth, and a local-only JSONL flight recorder.

## What it is

Nyquestro is a Rust crate building an order matching engine and its surrounding instrumentation from primitives upward. The design ambition is a comprehensive exchange system (lock-free order book, binary wire protocols, real-time risk controls, market-making strategy agent, rigorous benchmarking). The shipped reality, as of the 2026-05-04 step-change session, is a working deterministic single-threaded matching engine with `BTreeMap<Px, PriceLevel>` ladders, multi-instrument routing via a `Symbol(u64)` 8-byte ASCII pack, an Ornstein-Uhlenbeck + Poisson synthetic simulator, a Coinbase `level2` WebSocket live-feed bridge, HDR-histogram per-op latency metrics, JSONL telemetry, and a six-pane Ratatui infographics dashboard. The next-tier scope (wire protocol, risk guard, strategy agent, lock-free book, kernel bypass) is roadmap.

## Architecture

The crate is a single workspace member on Rust edition 2024 with 15 external dependencies. `cargo run` launches the dashboard; `cargo run -- --no-tui [--seed N]` runs a headless 10-second simulation summary; `cargo run -- --live coinbase` swaps the simulator for the Coinbase WebSocket bridge. The release profile uses `lto = "thin"` and `codegen-units = 1`; the lint wiring denies `unused_must_use` and warns on `needless_collect` and `redundant_clone`.

Ten public modules sit in a strict layered graph with no cycles:

```
                ┌──────────────────┐
                │       ui         │  (rendering, input, app loop)
                └─────┬────────────┘
                      │
        ┌─────────────┼─────────────┐──────────┐
        ▼             ▼             ▼          ▼
   ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐
   │ metrics │  │ simulator│  │   book   │  │  feed  │
   └────┬────┘  └─────┬────┘  └─────┬────┘  └────┬───┘
        │             │             │            │
        └─────────────┼─────────────┴────────────┘
                      │
                      ▼
                  ┌────────┐             ┌────────┐
                  │ events │ ──────────► │ order  │
                  └───┬────┘             └───┬────┘
                      │                      │
                      └──────────┬───────────┘
                                 ▼
                          ┌────────────┐
                          │  errors    │
                          └─────┬──────┘
                                ▼
                          ┌────────────┐
                          │   types    │
                          └────────────┘
                                ▲
                                │
                          ┌─────┴─────┐
                          │ telemetry │
                          └───────────┘
```

`types` and `errors` are leaves; every other module imports them. `book`, `simulator`, `metrics`, and `feed` are independent peers above `events`/`order`. `ui` is the only sink. `telemetry` is a side-car — the writer thread sits outside the engine's call graph and consumes `TelemetryEvent`s through a bounded `sync_channel(8192)`.

The dashboard runs a single-threaded loop: every 50ms the `MarketSimulator` (or the Coinbase bridge) emits a `Vec<SimAction>`; `App::handle_submit` calls `Market::submit_limit` (`PER_FRAME_BUDGET=500` actions drained per tick); results feed the tape ring, the `MetricsRegistry`, and `TelemetryHandle::record`. Every 33ms the terminal redraws by reading book state, metrics snapshots, and the mid-price history read-only.

## Subsystems and components

### `types` — Core primitives
`OrderID`, `Symbol(u64)` 8-byte ASCII pack, `Side`, `Px` (u64 cents), `Qty`, `Ts` (u64 nanoseconds since UNIX epoch), and `Status` with a `const fn can_transition_to` matrix. Allocation-free, `Copy`-friendly, uses `checked_*` over `saturating_*`.

### `errors` — Error model
`NyquestroError` flat enum (14 variants per Architecture.md; ~17 per Overview's later count) covering self-match, terminal-status, invariant violations, etc. Single severity classification via `NyquestroError::severity() -> ErrorSeverity` method. `NyquestroResult<T>` alias is re-exported from the crate root.

### `events` — Immutable event frames
`FillEvent`, `QuoteEvent` (live + cleared), `OrderEvent` (Placed/Filled/Cancelled/Rejected). All `Copy`, allocation-free. `FillEvent::new` rejects buyer==seller as defence in depth. Every event carries `Symbol` so multi-instrument streams disambiguate. `OrderRejectionReason` enum drives `OrderEvent::Rejected` (5 variants).

### `order` — Order entity and state machine
`Order` struct with quantity tracking via `checked_sub` (over-fill, zero-amount, and terminal-status fills all return typed errors), `cancel()` for state transition, one-way status enforcement. The order owns its timestamp; matching never calls `Ts::now()`.

### `book` — Matching engine
Three composing types: `OrderBook` (single-symbol, `BTreeMap<Px, PriceLevel>` for both bid and ask), `PriceLevel` (`VecDeque<Order>` FIFO with O(1) `total_quantity` read), and `Market` (`BTreeMap<Symbol, OrderBook>` multi-instrument router that auto-registers symbols on first submit). `submit_limit` runs four phases: snapshot pre-state → aggressive matching loop → self-match handling → rest aggressor + emit top-of-book quote on change. Match price = resting order price (aggressor gets price improvement). Inspection surface: `microprice()`, `ofi(n)`, `spread_cents()`, `depth(n)`, `level_counts()`, `top_n_bids/asks(n)`, `best_bid()`, `best_ask()`. Does not yet implement: market orders, IOC/FOK/AON, modification, atomic cancellation under concurrency, lock-free structures, hidden quantity.

### `metrics` — Per-op observability
`MetricsRegistry` with `hdrhistogram` per `Op::Submit/Match/Cancel` reporting p50/p95/p99/p999/p9999/max. `CounterSet` for orders/fills/cancels/rejects/quotes. `WindowedCounter` maintains rolling 1s/10s/1min/5min windows with lazy pruning. Histograms autoresize rather than panic on outliers.

### `simulator` — Synthetic order flow
`MarketSimulator` per symbol with Ornstein-Uhlenbeck mean-reverting mid-walk, Poisson order arrivals per side, log-normal sizes. `step(dt)` emits `Vec<SimAction>` (Submit / CancelHint / Cancel). `ChaCha8Rng` is the only randomness source — byte-identical action streams given a fixed seed.

### `feed` — Live data bridge
Coinbase Advanced Trade `level2` WebSocket client over `tokio-tungstenite` with the `native-tls` feature (no API key, no signup). A Bridge translates per-level updates into per-order virtual `SimAction` events keyed by `HashMap<(Symbol, Side, Px), OrderID>` so retraction is idempotent. `SNAPSHOT_LEVEL_CAP=50` truncates first-connect snapshots (which can ship 25,000+ levels) at the wire-parse boundary. Reconnect uses exponential backoff (250ms → 30s); status surfaces as a `FeedEvent::Status` banner.

### `telemetry` — Local-only flight recorder
JSONL writer at the platform-canonical data path (`~/Library/Application Support/Nyquestro/last-run.jsonl` on macOS). Truncated on every dashboard launch — exactly one run on disk at any moment. ~17–19 event variants (keys with raw + resulting action, engine events, frame profiles, periodic state snapshots, feed status, shutdown reasons). Bounded `sync_channel(8192)` + `try_send` + atomic drop counter — telemetry can never block the engine. Every line carries schema-version `v: 1`.

### `ui` — Ratatui dashboard
Six panes built on Ratatui 0.29 + Crossterm 0.28: engine pane with gauge stack (spread bar, microprice axis with `╋` marker, OFI, depth ratio, level twin bars); throughput sparklines + ratio gauges; trade tape with size bars; latency distribution-shape bars on log scale with `╫` markers; depth-of-book pressure bar; health-dot system (`●` Green/Yellow/Red from slow-frame counts + p99 thresholds). ANSI-16-only palette plus `Color::Reset` — no hardcoded RGB so user terminal themes (Solarized, Catppuccin) render correctly. `Tab` cycles symbols. `block_bar` is a 1/8-cell precision renderer.

## Technologies and concepts demonstrated

### Languages
- **Rust** (edition 2024, safe-only) — every subsystem; `grep -r "unsafe " src/` returns zero hits. The matching loop, the four-phase submit algorithm, the live-feed bridge, the dashboard renderer, and the telemetry writer are all written without `unsafe`.

### Frameworks and libraries
- **Ratatui 0.29** + **Crossterm 0.28** — six-pane TUI with custom gauge / sparkline / distribution-bar / pressure-bar widgets and a `block_bar` 1/8-cell renderer.
- **hdrhistogram 7.5** — per-op latency percentiles (p50/p95/p99/p999/p9999/max) with autoresize.
- **tokio 1** (`rt`, `rt-multi-thread`, `macros`, `net`, `io-util`, `sync`, `time` features) + **tokio-tungstenite 0.24** (`native-tls`) + **futures-util 0.3** — async runtime for the live-feed WebSocket client; `native-tls` chosen over `rustls-tls` after `rustls` panicked with "Could not automatically determine the process-level CryptoProvider", and as a side-benefit picks up the macOS Security.framework system trust store.
- **rand 0.8** + **rand_chacha 0.3** — `ChaCha8Rng` as the only synthetic-flow randomness source, used to keep the simulator reproducible.
- **serde 1** + **serde_json 1** — JSONL telemetry serialisation.
- **thiserror 1.0** — derive for `NyquestroError`.
- **chrono 0.4** — human-readable timestamp conversion only (not in the engine hot path).
- **url 2**, **dirs 5** — platform data dir resolution for the telemetry file.

### Tools
- `cargo` workspace single-member crate; release profile `lto = "thin"`, `codegen-units = 1`; lint wiring (`unused_must_use = "deny"`, `needless_collect = "warn"`, `redundant_clone = "warn"`).
- `examples/live_smoke.rs` (~120 lines) and `examples/telemetry_smoke.rs` (~100 lines) as manual integration verifiers run outside `cargo test`.

### Domains and concepts
- **Limit order book matching** — strict price-time priority, BTreeMap-keyed sorted ladders, VecDeque FIFO per price level, four-phase deterministic `submit_limit` algorithm with snapshot-and-compare top-of-book quote semantics.
- **Multi-instrument routing** — `Symbol(u64)` 8-byte ASCII pack hashable + orderable in O(1), `Market` wrapper holding one `OrderBook` per `Symbol`, auto-registration on first submit, allocation-free hot path.
- **Deterministic engine** — no `Ts::now()` in the matching loop, single ChaCha8Rng-seeded synthetic flow, pinned by `tests/matching_test.rs::run_twice_identical_sequence_identical_output`.
- **Live-market data ingestion** — Coinbase Advanced Trade `level2` WebSocket without authentication; L2-per-level updates translated to per-order virtual `SimAction` stream via an idempotent retraction map keyed by `(Symbol, Side, Px)`.
- **Backpressure as a structural property** — `SNAPSHOT_LEVEL_CAP=50` at the wire boundary AND `PER_FRAME_BUDGET=500` at the dispatch boundary together make the dashboard non-freezable under any feed load; `sync_channel(8192)` + `try_send` + atomic drop counter make telemetry non-blocking by construction.
- **Microstructure inspection** — `microprice()` (volume-weighted between best bid/ask), order-flow imbalance `ofi(n)`, spread, depth, level counts, top-N book slice; all read-only.
- **Per-op latency observability with HDR percentiles** — `Op::Submit`/`Op::Match`/`Op::Cancel` histograms with full tail (p999/p9999/max), windowed counters with lazy pruning across 1s/10s/1min/5min.
- **Safe-Rust systems engineering** — every place where `unsafe` would be the conventional choice (intrusive structures, manual lifetimes) is implemented with safe abstractions or deferred to roadmap.
- **TUI infographics design** — gauges, sparklines, distribution-shape bars, pressure bars, health-dot system, log-scale latency rendering; ANSI-16-only colour discipline to respect arbitrary terminal themes.
- **Local-only structured telemetry** — JSONL flight recorder with truncate-on-startup, schema-version per line, never uploaded.
- **Stochastic synthetic order flow** — Ornstein-Uhlenbeck mean-reverting mid-walk, Poisson arrivals per side, log-normal sizes.

## Key technical decisions

D1 — **Safe Rust only.** No `unsafe` anywhere. The compiler guarantees absence of data races; equivalent machine code from safe abstractions is the required workaround for cases where `unsafe` would be conventional.

D2 — **Correctness before performance.** Build a deterministic engine first on `BTreeMap`/`VecDeque`; lock-free structures and slab allocation are deferred while the API surface stabilises.

D3 — **Events as immutable `Copy` frames.** All event types are `Copy` and allocation-free so fan-out and replay testing cost nothing. `OrderRejectionReason` exists as an enum precisely because a `String` reason field would break `Copy`.

D4 — **Cents-based prices.** `Px` is `u64` cents, not float; integer comparison is exact, deterministic, fast. Dollar-to-cents conversion happens at the boundary with documented float truncation.

D5 — **Nanosecond timestamps.** `Ts` is `u64` nanoseconds since UNIX epoch — `Copy`, comparison-friendly, wraps in ~584 years.

D6 — **Severity as method classification, not type hierarchy.** `NyquestroError::severity()` is the canonical entry point rather than separate `RecoverableError`/`FatalError` enums.

D7 — **Order owns its timestamp.** `Order::new()` reads the clock via `Ts::now()`; matching never calls `Ts::now()`. Determinism falls out as a consequence.

D8 — **`FillEvent` decoupled from `Order`.** A one-day December 2025 experiment had `Order::fill()` return `FillEvent`; it was reverted within 24 hours because it coupled `order` to `events` (violating dependency direction) and conflated buyer/seller IDs.

D9 — **Multi-instrument via `Symbol(u64)` 8-byte ASCII pack.** Big-endian pack so `"AAPL"` < `"GOOG"` lexicographically when compared as integers. Allocation-free, `Copy`, hashable, orderable in O(1). `Market` wrapper auto-registers symbols on first submit.

D10 — **Coinbase Advanced Trade `level2` WebSocket for live data (no auth).** Highest-quality free continuous-depth source for the "zero-cost-forever" data constraint. `native-tls` chosen over `rustls-tls` after a CryptoProvider startup panic; gets macOS Security.framework system trust store for free as a side-benefit.

D11 — **`SNAPSHOT_LEVEL_CAP=50` + `PER_FRAME_BUDGET=500` as paired structural caps.** Naive integration of Coinbase's 25,000+-levels-per-side first-connect snapshot froze the dashboard so badly that `q` could not quit. The two-pronged defence makes worst-case input bounded at both the wire-parse boundary and the dispatch boundary; either alone is insufficient.

D12 — **JSONL telemetry: local-only, truncate-on-startup, drop-on-full.** Append-only JSONL keeps the file greppable with any tool. Truncate keeps disk bounded. Drop-on-full via `sync_channel(8192)` + `try_send` + atomic counter is the structural guarantee that telemetry can never freeze the dashboard. Never uploaded, never aggregated, never analytics. Schema version `v: 1` on every line.

D13 — **Dashboard infographics over numeric tables.** Gauges, sparklines, distribution-shape bars, pressure bars, and a health-dot system replace numeric panels because numeric tables don't help the eye scan a real-time stream. ANSI-16-only palette keeps it readable on any user terminal theme.

## What is currently built

Multi-instrument deterministic limit-order matching engine across ~6.5k lines of Rust in 30+ source files spanning ten modules. `book/` implements the four-phase `submit_limit` algorithm with `BTreeMap<Px, PriceLevel>` ladders, `VecDeque<Order>` FIFO per level, snapshot-and-compare top-of-book quote emission, self-match rejection at both event and engine layers, and a `Market` multi-instrument wrapper. `metrics/` provides HDR-histogram per-op latency and rolling 1s/10s/1min/5min counters. `simulator/` drives synthetic AAPL/MSFT/NVDA flow via Ornstein-Uhlenbeck + Poisson + log-normal. `feed/` ingests live BTC-USD/ETH-USD/SOL-USD depth from Coinbase Advanced Trade WebSocket. `telemetry/` writes a JSONL flight recorder. `ui/` renders a six-pane Ratatui dashboard with infographics.

Test coverage at last full run: 88 tests across 47 inline unit tests in `src/` and 41 integration tests across 5 files in `tests/` (`matching_test.rs` 12 tests, `events_test.rs` 9, `order_test.rs` 8, `price_level_test.rs` 6, `types_test.rs` 6), plus two manual smoke-binary integration verifiers under `examples/`. The determinism test `run_twice_identical_sequence_identical_output` pins byte-identical engine output across runs.

Not yet built: wire protocol (binary UDP / FIX TCP), risk guard layer, strategy / market-making agent, lock-free order book, slab allocation, kernel bypass, criterion benchmark harness, property-based tests, CI pipeline. The `risk-layer`, `extensive-testing-framework`, `itch-replay-harness`, `cpp-reference-impl`, and `extended-order-types` specs are filed in the repository's `context/plans/` but unimplemented.

## Current state

Status: **active**. Last meaningful code commit `6516eb6` on 2026-05-04 (the multi-phase shipping session); HEAD `cc1deb0` is a 2026-05-05 docs-only repoint of Learning references. 35 total commits since June 2025. The project's pattern is intense bursts followed by long pauses — six 30+-day gaps punctuate the history. In flight: a README demo recording and a proposed V2 distributed extension (VSR consensus + `madsim` deterministic-simulation-testing harness + Kani formal proofs of matching invariants); both are gated on additional V1 work (STP, journal) shipping first.

## Gaps and known limitations

- **No property-based tests of matching invariants.** Plan filed at `context/plans/extensive-testing-framework.md` (27.6KB, 5-day buildout covering proptest, proptest-state-machine, criterion, insta snapshots, stress runs, llvm-cov, cargo-mutants). HFT firms expect this; ranked the highest-leverage outstanding gap by the repo's `context/notes/hft-firm-priorities.md` §8.
- **No stress harness** — capacity claims for the matching engine are unsubstantiated.
- **No CI pipeline configured** — tests run locally only via `cargo test`.
- **No mutation testing or coverage measurement** — blind spots in the test suite are unmeasured.
- **No wire protocol** (binary UDP gateway, FIX TCP acceptor, market-data multicast) — roadmap.
- **No risk guard layer** (fat-finger, position/PnL, rolling VaR circuit breaker, throttles) — roadmap; `context/plans/risk-layer.md` exists.
- **No strategy agent** (book reconstructor, OFI signal, two-sided quoting, inventory tracking) — roadmap.
- **No lock-free order book** — current `BTreeMap`/`VecDeque` MVP is explicit; lock-free deferred per D2.
- **No criterion benchmarks** — per-op latency budget and regression detection unimplemented; HDR histograms cover the runtime-latency angle.
- **`book/market.rs` has light direct test coverage** — multi-instrument routing exercised only indirectly through `matching_test.rs`.
- **`simulator/`, `feed/`, `ui/` have minimal isolation tests** — `feed/` bridge state machine could be unit-tested without network; UI relies on `cargo build --release` + headless smoke + manual inspection.
- **README describes ~50+ features across 6 categories; code covers the foundational tier only.** The README is aspirational direction, not current state.

## Direction (in-flight, not wishlist)

- **Property-based + state-machine testing framework** — the unambiguous top pick per `notes/hft-firm-priorities.md` §8; 5-day buildout already specified in `context/plans/extensive-testing-framework.md`.
- **README demo recording** — work file open under `Work/README Demo.md`.
- **V2 distributed extension** (proposed, gated on V1 STP + journal): a `consensus/` module implementing Viewstamped Replication in safe Rust alongside the existing typed primitives, a `madsim`-driven deterministic-simulation-testing harness for the consensus layer, Kani formal-verification harnesses in a separate `proofs/` module covering 3–5 critical matching invariants (no negative inventory, debit total = credit total across any match, saturating fill bounds, FIFO within price level under reordering, no use-after-free on the lock-free path), and reproducible p50/p99/p99.9 benchmarks vs Liquibook on identical input. Additive on top of V1 — every existing primitive stays.

## Demonstrated skills

- **Implements a multi-instrument deterministic limit-order matching engine from scratch in safe Rust** — strict price-time priority, snapshot-and-compare top-of-book quote semantics, self-match rejection at two layers, multi-instrument routing via an 8-byte ASCII-packed `Symbol(u64)`.
- **Designs a strictly layered crate with zero import cycles.** Layering is enforced by the compiler; the December 2025 `Order → events` coupling experiment was caught and reverted within 24 hours, and the regression is now mechanically prevented by the import graph.
- **Engineers backpressure as a structural property.** `SNAPSHOT_LEVEL_CAP=50` and `PER_FRAME_BUDGET=500` together make the dashboard non-freezable under any feed load; bounded `sync_channel` + `try_send` + atomic drop counter makes telemetry non-blocking by construction.
- **Integrates a real exchange WebSocket feed end-to-end** — Coinbase Advanced Trade `level2` without auth, L2-to-virtual-order translation with idempotent retraction via `HashMap<(Symbol, Side, Px), OrderID>`, exponential reconnect backoff, status surfaced as a UI banner.
- **Builds a six-pane Ratatui dashboard with infographics** — gauges, sparklines, distribution-shape bars on log scale, pressure bars, health-dot system, ANSI-16-only colour discipline, custom `block_bar` 1/8-cell precision renderer.
- **Builds custom observability instead of pulling a framework.** `MetricsRegistry` over `hdrhistogram` with autoresize, `WindowedCounter` with lazy pruning over rolling 1s/10s/1min/5min, schema-versioned JSONL flight recorder with truncate-on-startup.
- **Maintains a tight test posture for a state-machine-heavy crate** — 88 tests across 5 integration files + inline unit tests in every `src/` module, including a determinism guard (`run_twice_identical_sequence_identical_output`) that pins byte-identical engine output across runs.
- **Reasons about choice points in the open** — 13 decisions recorded with rationale, alternatives, and implications; 8 pending decisions tracked through a 40-day planning gap and all resolved in a single shipping session.
- **Operates on free, public data sources by design** — "zero-cost-forever" data constraint shapes the Coinbase choice and the `native-tls` pragmatism after the `rustls` CryptoProvider panic.
- **Synthesises stochastic order flow from first principles** — Ornstein-Uhlenbeck mid-walk + Poisson per-side arrivals + log-normal sizes, ChaCha8-deterministic.
- **Composes safe-Rust async I/O without `unsafe`** — `tokio` + `tokio-tungstenite` + `futures-util` for the live feed; the writer thread for telemetry runs outside the engine's call graph through a bounded channel.
- **Treats documentation as infrastructure.** A 26.3KB README + an 18.1KB `context/ARCHITECTURE.md` + per-system context notes + a project-local principal-engineer `CLAUDE.md` let a 40-day idle period resolve into a single-day multi-phase ship.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Nyquestro/_Overview.md | 118 | "The README is written as a portfolio piece describing a comprehensive exchange system. The code is a careful, incremental build starting from primitives up. This is not a disconnect — it is a roadmap expressed as prose. The implementation approach (primitives → events → errors → hardening → matching → protocol → risk → strategy) is methodical and sound. The risk is that the aspirational README creates expectations the code cannot yet satisfy." |
| Projects/Nyquestro/Architecture.md | 299 | "- The repo's own `context/ARCHITECTURE.md` is the canonical implementation memory and is regenerated as the code changes." |
| Projects/Nyquestro/Decisions.md | 164 | "**Status:** Shipped `[verified]`. Health-dot system, sparklines, and per-pane gauges all live." |
| Projects/Nyquestro/Evolution.md | 188 | "- The repo's commit history is the canonical source for evolution timestamps" |
| Projects/Nyquestro/Gaps.md | 158 | "> Most divergences are expected — the README describes the end state, not the current state. The concerning gaps are property-based tests, CI, and the core correctness issues (G1-G3) that exist in already-implemented code." |
| Projects/Nyquestro/Roadmap.md | 158 | "> The README describes ~50+ features across 6 major categories. Current implementation covers ~3 of them (typed primitives, event frames, error handling — all marked [x] in the README). The gap between stated scope and implemented scope is enormous. This is fine for a portfolio project if the next steps are taken; the risk is that the README promises remain aspirational indefinitely." |
| Projects/Nyquestro/Testing.md | 173 | "- The repo's `context/notes/hft-firm-priorities.md` §8 ranks this plan as the highest hiring-signal-per-hour work available" |
| Projects/Nyquestro/Systems/_Overview.md | 54 | "- The repo's own `context/systems/` is the canonical implementation memory for each subsystem; these vault notes interpret and contextualise that material." |
| Projects/Nyquestro/Systems/Book.md | 273 | "- The repo's `context/systems/book.md` is the canonical implementation memory and is regenerated as the code changes" |
| Projects/Nyquestro/Systems/Core Types.md | 194 | "- The repo's `context/systems/types.md` is the canonical implementation memory" |
| Projects/Nyquestro/Systems/Dashboard UI.md | 191 | "- The repo's `context/plans/dashboard-infographics.md` is the shipped plan" |
| Projects/Nyquestro/Systems/Error Model.md | 175 | "- The repo's `context/systems/errors.md` is the canonical implementation memory" |
| Projects/Nyquestro/Systems/Event System.md | 218 | "- The repo's `context/systems/events.md` is the canonical implementation memory" |
| Projects/Nyquestro/Systems/Feed.md | 158 | "- The repo's `context/plans/live-crypto-feed.md` is the shipped plan that drove this work" |
| Projects/Nyquestro/Systems/Matching Engine.md | 88 | "The \"additive principle\" from the vault's update-mode guidance applies here: this note was reframed as history rather than overwritten. The new reality lives in [[Nyquestro/Systems/Book]]; this file remembers what it was." |
| Projects/Nyquestro/Systems/Metrics.md | 171 | "- The repo's `context/notes/hft-firm-priorities.md` §3 — the rationale for tail-latency emphasis" |
| Projects/Nyquestro/Systems/Order Model.md | 242 | "- The repo's `context/systems/order.md` is the canonical implementation memory" |
| Projects/Nyquestro/Systems/Simulator.md | 152 | "- The repo's `context/systems/simulator.md` is the canonical implementation memory" |
| Projects/Nyquestro/Systems/Telemetry.md | 163 | "- The repo's `context/notes/telemetry-policy.md` is the policy document covering local-only / truncate / drop-on-full" |
| Projects/Nyquestro/Work/HFT Observability Dashboard.md | 98 | "#nyquestro #work #observability #hft #additive" |
| Projects/Nyquestro/Work/README Demo.md | 45 | "- Cernio demo commit: `Capataina/Cernio` `4a93239`" |
| Projects/Nyquestro/Work/V2 Distributed Extension.md | 70 | "#nyquestro #work #distributed-systems #consensus" |

## Anomalies

One numeric discrepancy in the LifeOS source itself: `_Overview.md` describes the error enum as "14 variants" in one place and "~17 typed variants" in another; the per-project file states "14 (per Architecture.md; ~17 per Overview's later count)" rather than picking arbitrarily. No files were unreadable; every `.md` file enumerated in the folder listing (including the historical `Systems/Matching Engine.md`) was fetched and incorporated. Status `active` is explicit in the Overview frontmatter, not inferred.
