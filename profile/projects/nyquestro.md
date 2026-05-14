---
name: Nyquestro
status: active
source_repo: https://github.com/Capataina/Nyquestro
lifeos_folder: Projects/Nyquestro
last_synced: 2026-05-13
sources_read: 22
---

# Nyquestro

## One-line summary

From-scratch deterministic limit-order matching engine in safe Rust with HDR-histogram latency telemetry, an Ornstein-Uhlenbeck synthetic simulator plus Coinbase L2 WebSocket bridge, and a six-pane Ratatui observability dashboard — single-threaded MVP, lock-free and binary-protocol work explicitly deferred.

## What it is

Nyquestro is a single-binary Rust crate (edition 2024, ~30 source files, ~6.5k LOC) that implements the core of an exchange matching engine alongside the observability layer needed to watch it run live. The project's design intent is a comprehensive exchange system (lock-free order books, binary UDP gateway, FIX TCP acceptor, risk guard layer, market-making strategy agent, kernel-bypass networking) per the README's long-term scope. What is currently demonstrated is the foundational MVP shipped in a single multi-phase session on 2026-05-04: a deterministic price-time-priority `BTreeMap<Px, PriceLevel>` order book, a `Market` multi-instrument wrapper, three-symbol synthetic flow driven by a classical OU + Poisson + log-normal microstructure model, a live Coinbase Advanced Trade `level2` WebSocket bridge feeding BTC-USD / ETH-USD / SOL-USD into the same `SimAction` abstraction the synthetic simulator uses, HDR-histogram latency telemetry per `Op::{Submit, Match, Cancel}`, rolling 1s/10s/1min/5min counters, a JSONL local-only flight recorder, and a six-pane Ratatui dashboard with ANSI-16-only colour discipline. The matching engine, the telemetry, and the dashboard are all deliberately single-threaded; performance-tier work (lock-free internals, slab allocation, SIMD price comparison, kernel bypass) is roadmap, not implemented.

## Architecture

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
                          │ telemetry │  (side-car; reads no engine state)
                          └───────────┘
```

The crate is strictly layered with no import cycles. `types` and `errors` sit at the bottom; every other module imports them. `order` and `events` form the middle layer. `book`, `simulator`, `metrics`, and `feed` are independent peers above. `ui` is the only sink — nothing imports `ui`. `telemetry` is a side-car: the writer thread sits outside the engine's call graph and consumes `TelemetryEvent`s through a bounded `sync_channel(8192)` with drop-on-full backpressure. The compiler enforces the layering — re-introducing the December 2025 coupling where `Order::fill` returned a `FillEvent` (commit `3556a70`, reverted in `864b488`) would re-trigger an architectural regression that the import graph mechanically prevents today.

Core data flow inside the dashboard's single-threaded run loop:

```
[every 50ms]                                           [every 33ms]
MarketSimulator::step(dt) (per symbol)                 Terminal::draw(panes::render)
        │                                                       │
        ▼                                                       │
Vec<SimAction> { Submit(Order), CancelHint, Cancel{…} }         │
        │                                                       │
        ▼ (drain ≤ PER_FRAME_BUDGET=500 / tick)                 │
Market::submit_limit ──► OrderBook::submit_limit ──► PriceLevel │
        │                                  │                   │
        ▼                                  ▼                   │
SubmitResult { fills, quotes, lifecycle }  (book mutated)       │
        │                                                       │
        ├─► tape ring (newest fills, ≤ 200)                     │
        ├─► MetricsRegistry::record_latency(Op::Submit/Match)   │
        ├─► MetricsRegistry::record_orders/fills/rejects(n)     │
        └─► TelemetryHandle::record(TelemetryEvent::…)          │
                                                                ▼
                                                     panes::render reads (read-only)
```

The render path is read-only on engine state — nothing in `ui::panes` mutates the book, metrics, or simulator. The 33ms render / 50ms sim / 10ms input-poll cadence makes the dashboard responsive to keypresses within ~10ms regardless of frame state. Two structural invariants protect dashboard responsiveness when the Coinbase firehose lands: `SNAPSHOT_LEVEL_CAP=50` at the wire-parse boundary (Coinbase L2 snapshots can ship 25,000+ levels per side on first connect; the parser truncates to 50 closest-to-touch levels per side) and `PER_FRAME_BUDGET=500` at the dispatch boundary (the render loop drains at most 500 actions from the queue per frame, surplus carries to the next tick). Either alone is insufficient; both together make the dashboard non-freezable under any feed load.

## Subsystems and components

### `types` — Domain primitives (`src/types.rs`, ~15.9KB)

Owns `OrderID(u64)`, `Symbol(u64)`, `Side`, `Px`, `Qty`, `Ts`, and `Status`. Every primitive is `Copy + Clone + Debug + PartialEq` and designed for zero-allocation use on the hot path. `Symbol` is the load-bearing primitive for multi-instrument routing — it packs up to 8 ASCII bytes big-endian into a `u64` (so `Symbol::from_const("AAPL") < Symbol::from_const("GOOG")` by integer compare; works as both `HashMap` and `BTreeMap` key with no boxing; fits in a single CPU register). `Px` stores price as `u64` cents — integer-only matching, no floating-point precision issues; `from_dollars` rounds rather than truncates (a 2026-05-04 correctness fix from the prior `as u64` truncation). `Qty::checked_sub` returning `Option<Qty>` is what enables `Order::fill` to reject over-fills cleanly; the prior `saturating_sub` silently clamped at zero, hiding over-fill bugs. `Ts::from_nanos(u64)` is the deterministic constructor and `Ts::now()` exists but is reserved for ad-hoc callers — `OrderBook::submit_limit` never calls `Ts::now()`. `Status::can_transition_to(next)` is a `const fn` enforcing one-way transitions: `Open → PartiallyFilled → FullyFilled` (terminal), with `Cancelled` reachable from `Open` or `PartiallyFilled`.

### `errors` — Error taxonomy (`src/errors.rs`, ~6.3KB)

Defines `NyquestroError` (14 variants partitioned into primitive validation, order lifecycle, matching engine, and a single Fatal `InvariantViolation(&'static str)`), `ErrorSeverity::{Recoverable, Fatal}`, and the `NyquestroResult<T>` alias used by every fallible operation. Severity is derived from the variant via an exhaustive `match` (single source of truth — adding a 15th variant forces a compile error in `severity()`). The 2026-05-04 rewrite deleted generic catch-alls (`RecoverableError`, `FatalError`, `ErrorSeverity { severity: &'static str }`, `ErrorSeverityCannotBeDetermined`, `MatchingEngineError`) and added specific variants for each engine failure mode (`OverFill { order_id, fill, remaining }`, `InvalidStatusTransition { order_id, from, to }`, `SelfMatch(u64)`, `PriceLevelMismatch { expected_cents, actual_cents }`, etc.). Built on `thiserror::Error` for derived `Display` + `Error::source`.

### `order` — Order entity (`src/order.rs`, ~8.5KB)

`Order` is `Copy` (every field is `Copy`) with private fields and `&self` accessors throughout. The constructor takes a caller-supplied `Ts` (with `Order::new_now` as a convenience for non-deterministic callers); this is the change that made the matching engine deterministic. `fill(amount: Qty) -> NyquestroResult<()>` has three guards: terminal-status rejection, zero-amount rejection, and over-fill rejection via `checked_sub` returning `OverFill { order_id, fill, remaining }`. State is preserved on every error path; the integration test `fill_rejects_overfill_without_mutation` pins this contract. `transition_to` consults `Status::can_transition_to`; backward transitions and re-activation of terminal orders are rejected.

### `events` — Engine event frames (`src/events/`)

Three immutable `Copy + Eq + Hash` event frames the engine emits: `FillEvent`, `QuoteEvent`, `OrderEvent`. Files restructured 2026-05-04: `fill_event.rs → fill.rs`, `quote_event.rs → quote.rs`, `order_event.rs → lifecycle.rs`. `FillEvent::new` rejects zero quantity and self-match (`buyer == seller`) — the self-match check is active, not commented out as in the pre-2026-05-04 state. `QuoteEvent` has two named constructors with different semantics: `live(side, symbol, price, qty, ts)` rejects zero quantity (a non-zero best with zero displayed quantity is meaningless), `cleared(side, symbol, price, ts)` is infallible (zero quantity is meaningful when the side has emptied). `OrderEvent` is a four-variant enum (`Placed`, `Filled`, `Cancelled`, `Rejected`); `OrderRejectionReason` covers five rejection modes. Every event carries a `Symbol` so multi-instrument streams disambiguate.

### `book` — Matching engine (`src/book/`)

The system the rest of the project exists to observe. Three types compose it. `OrderBook` (`order_book.rs`, ~12.3KB) owns the bid/ask `BTreeMap<Px, PriceLevel>` ladders for a single symbol — best bid is `iter().next_back()`, best ask is `iter().next()`. `PriceLevel` (`price_level.rs`, ~6.7KB) is one FIFO `VecDeque<Order>` queue at one price with a running `total_quantity: Qty` for O(1) read; supports `push_back`, `front_mut`, `record_execution`, `pop_front`, `remove_by_id`. `Market` (`market.rs`, ~4.2KB) is the multi-instrument wrapper — `BTreeMap<Symbol, OrderBook>` that auto-registers symbols on first submit.

The submission algorithm runs four phases: (1) snapshot pre-state of best bid + ask for change-detection, (2) aggressive matching loop — probe opposite top, break if not crossing or self-match, compute `trade_qty = min(aggressor.remaining, resting.remaining)`, fill both sides via `checked_sub`, emit `FillEvent::new(buyer, seller, sym, resting.price, trade_qty, resting.ts)` plus `OrderEvent::Filled` for the aggressor and (if resting becomes terminal) for the resting order, pop the level and remove the empty `BTreeMap` entry, (3) if self-match was detected, push `OrderEvent::Rejected { reason: SelfMatch }` and skip phase 4, (4) if the order has remaining quantity and is active, push to the same-side ladder and emit `OrderEvent::Placed`, then emit `QuoteEvent::live` or `QuoteEvent::cleared` for any side whose top changed.

Match price equals resting price (aggressor gets price improvement when crossing); self-match is rejected at both layers (engine first as load-bearing, `FillEvent::new` as defence-in-depth); quotes are emitted only when best price or displayed quantity changes (snapshot+compare pattern, not delta-tracking). Microstructure inspection surface: `microprice()`, `ofi(n)`, `spread_cents()`, `depth(n)`, `level_counts()`, `top_n_bids(n)`, `top_n_asks(n)`. Cancellation is O(N_levels × N_orders_per_level) — acceptable for MVP but deferred indexed-cancel work for high-cancel-rate workloads.

### `metrics` — Observability hub (`src/metrics/`)

`MetricsRegistry` holds three `hdrhistogram::Histogram<u64>` instances (`submit_lat`, `match_lat`, `cancel_lat`) each configured 1ns lower bound, 1 hour upper bound, 3 significant figures (~0.1% precision), `auto(true)` autoresize. `Op::Match` is the strict subset of submits that produced a fill. `LatencySnapshot` exposes p50/p95/p99/p999/p9999/max/mean — the dashboard surfaces all of these because "a candidate showing only p50 gets dismissed politely" per the repo's HFT-firm-priorities note §3. `CounterSet` holds five `WindowedCounter`s (orders/fills/cancels/rejects/quotes); each `WindowedCounter` stores `(Instant, count)` pairs in a `VecDeque` and lazily prunes entries older than 5 minutes on every record. `snapshot()` returns a `Copy`-able `RegistrySnapshot` so the dashboard reads a value once per render frame without holding any reference to the live registry. Single-threaded — no atomics, no `Arc`/`Mutex`; multi-threaded engine work is roadmap.

### `simulator` — Synthetic order flow (`src/simulator/`)

`MarketSimulator` (`market.rs`, ~10.7KB) produces a configurable, byte-identical-per-seed stream of `SimAction`s combining three classical microstructure models: an Ornstein-Uhlenbeck mean-reverting walk for the mid (`mid_next = mid + theta*(mid_mean - mid)*dt + sigma*sqrt(dt)*randn()`), independent Poisson arrivals per side with configurable intensities, log-normal order sizes, and a truncated geometric price selection centred at the current mid. OU was chosen over geometric Brownian motion because GBM diverges in expectation and produces visually unstable charts on long runs — OU stays anchored to `mid_mean`. `ChaCha8Rng` is the only RNG; two `MarketSimulator::new(_, 42)` instances produce byte-identical action streams. `reseed(0xC0FFEE)` is the user's reset key on `r`. Order timestamps come from the simulator's own monotonic clock (`self.now_ns`) via `Ts::from_nanos`, never `Ts::now()`. Each simulator instance is symbol-scoped; the multi-symbol dashboard runs three instances (AAPL @ $150, MSFT @ $300, NVDA @ $500).

`SimAction` is the abstraction that unifies synthetic and live producers: `Submit(Order)`, `CancelHint` (synthetic-only — intent without referent, the `App` picks a random resting id from its cache), and `Cancel { symbol, order_id }` (feed-only — exact id retraction).

### `feed` — Live data path (`src/feed/`)

`coinbase.rs` (~10.9KB) opens a TLS WebSocket to `wss://advanced-trade-ws.coinbase.com` via `tokio-tungstenite` with the `native-tls` feature (chosen over `rustls-tls` after `rustls` panicked at startup with "Could not automatically determine the process-level CryptoProvider"; `native-tls` yields the macOS system trust store for free). Subscribes to the `level2` channel for `product_ids: ["BTC-USD", "ETH-USD", "SOL-USD"]` (public, no auth, free forever — satisfying the project's "zero-cost-forever" data constraint). Parses `events[].updates[]` into `FeedEvent::Snapshot { symbol, bids, asks }`, `FeedEvent::Update { symbol, side, price, new_quantity }`, or `FeedEvent::Status(String)`. On any error, sends a status event, sleeps `current_delay` (starting at 250ms), doubles up to a 30s cap, and retries; a successful subscription resets the delay.

`bridge.rs` (~9.0KB) is the L2-to-virtual-order translator. Coinbase says "best bid is now 65,000.50 with 0.5 BTC"; the engine wants "submit order X for 0.5 BTC at 65,000.50". The bridge maintains `HashMap<(Symbol, Side, Px), OrderID>` so cancellations are idempotent against the engine. For each per-level update it emits the appropriate `Submit`/`Cancel`/`Cancel-then-Submit` sequence. For snapshots it clears all existing keys for that symbol/side (after `SNAPSHOT_LEVEL_CAP` truncation), emits cancels for retracted levels, then submits for new levels.

### `telemetry` — Local-only flight recorder (`src/telemetry/`)

JSONL writer at the platform-canonical data path (`~/Library/Application Support/Nyquestro/last-run.jsonl` on macOS, `~/.local/share/nyquestro/` on Linux, `%LOCALAPPDATA%\Nyquestro\` on Windows). Truncated on every launch — exactly one run on disk at any moment. `TelemetryHandle::record(event)` uses `try_send` into a bounded `sync_channel(8192)`; on `Full` the event is dropped and an `AtomicU64` counter is incremented, with the writer periodically flushing a `DroppedEvents { count: N }` summary. The writer thread owns a `BufWriter<File>` (64KB buffer) and is the only thing that touches disk. `TelemetryEvent` is a tagged enum (`#[serde(tag = "kind", rename_all = "snake_case")]`) with ~19 variants spanning lifecycle, input, engine events, profiling, periodic state snapshots, feed status, and the `DroppedEvents` health summary. `FrameSlow` carries a `reason` field (`per_frame_budget_exhausted` / `render_blocked` / `step_dominated`) classifying where the time went on slow frames. `Quote` is sampled 1-in-10 because busy live mode produces 1000+ quotes/sec and the periodic `BookState` snapshot covers the load-bearing information. Never uploaded, never aggregated, never analytics — the user's audit trail and the agent's debugging surface.

### `ui` — Ratatui dashboard (`src/ui/`)

Three modules: `theme.rs` (~7.6KB) — ANSI-16-only palette (BID=Green, ASK=Red, ACCENT=Yellow, CHROME=DarkGray, GOOD/WARN/ALERT=LightGreen/Yellow/Red), `Style` helpers, and the load-bearing `block_bar(ratio, width)` sub-cell-precision bar renderer using `▏▎▍▌▋▊▉█`; `app.rs` (~32.5KB) — `App` state, `Action` enum, `EngineState`, key-mapping, terminal setup/restore, the 33ms-render / 50ms-sim / 10ms-input run loop; `panes.rs` (~33.0KB) — top-level `render`, per-pane render functions, formatters.

Six panes laid out as 1-row status / body (60/40 vertical split with three panes in each row) / 1-row keybind footer:

| Pane | Position | Treatment |
|---|---|---|
| Depth of Book | top-left, tallest | Asks worst-first toward spread row; spread row; bids best-first below; per row price+qty+horizontal block bar proportional to qty/max; bid bars Green, ask bars Red; spread row in DarkGray; pressure bar at bottom |
| Trade Tape | top-centre | Newest-first fills capped to pane height (200 ring); per row `HH:MM:SS.MMM`, price, aggressor glyph (`▲` Buy Green, `▼` Sell Red), quantity |
| Latency | top-right | Per-`Op` percentiles (p50/p95/p99/p999/p9999/max/mean) as distribution-shape bars on log scale with `╫` markers; p99 and max highlighted Yellow; mid-price-delta sparkline at bottom |
| Mid Price | bottom-left, half-width | Ratatui `Chart` with Braille markers over the last ~600 ticks (~30s); ACCENT-coloured; auto-bounded |
| Throughput | bottom-centre | Four rows of `last_1s/s · last_10s/10s` from `WindowSnapshot`; sparkline trend per row |
| Engine | bottom-right | Two-section card: "book" (best bid Green, best ask Red, resting count, microprice, OFI gauge); "lifetime" (submitted/filled/cancelled/rejected totals); health-dot system (`●` Green/Yellow/Red) from slow-frame counts + p99 thresholds |

Keybinds: `q/Q/Esc` quit, `p/P/Space` pause, `r/R` reset+reseed `0xC0FFEE`, `+/=` speed ×1.5 (cap 50.0), `-/_` speed ÷1.5 (cap 0.1), `Tab/s/S` cycle focused symbol. Three execution modes: synthetic (default — three `MarketSimulator` instances), live (`--live coinbase` — Coinbase WebSocket bridge), headless (`--no-tui [--seed N]` — 10-second silent simulation, 6-line summary, the smoke-test path).

## Technologies and concepts demonstrated

### Languages

- **Rust (edition 2024)** — every line of the project; ~6.5k LOC across ~30 source files. Heavy use of `Copy` types in the hot path, `const fn` for compile-time state-machine checks, exhaustive enum matches forcing compile-time discipline, `&self` accessors throughout. Zero `unsafe` blocks anywhere in the crate (`grep -r "unsafe " src/` returns zero hits — invariant enforced by convention plus repo-level `notes/safe-rust-philosophy.md`).

### Frameworks and libraries

- **`ratatui` 0.29** — core TUI framework; six-pane dashboard rendering, `Chart` with Braille markers for the mid-price pane, `Paragraph` widgets composed with pre-rendered block-element bar strings (Ratatui's `BarChart` couldn't do mirrored bid/ask cleanly).
- **`crossterm` 0.28** — terminal backend; raw-mode setup, input event polling (10ms cadence), terminal restore.
- **`hdrhistogram` 7.5** — latency percentile recording; per-`Op` histograms with 1ns–1h bounds, 3 significant figures, autoresize on overflow.
- **`tokio` 1** with `rt`, `rt-multi-thread`, `macros`, `net`, `io-util`, `sync`, `time` features — the live-feed runtime; the WebSocket task lives in a `tokio::task` spawned at startup, the main thread reads `FeedEvent`s via an `mpsc::Receiver`.
- **`tokio-tungstenite` 0.24 (native-tls feature)** — WebSocket client to `wss://advanced-trade-ws.coinbase.com`; `native-tls` chosen over `rustls-tls` for macOS Security.framework integration after a startup-time `CryptoProvider` panic with rustls.
- **`futures-util` 0.3** — `SinkExt`/`StreamExt` for the WebSocket split.
- **`rand` 0.8 + `rand_chacha` 0.3** — `ChaCha8Rng` as the only randomness source in the synthetic simulator; deterministic action streams per seed.
- **`serde` 1 + `serde_json` 1** — JSONL telemetry event serialisation; tagged enums with `#[serde(tag = "kind", rename_all = "snake_case")]`.
- **`thiserror` 1.0** — derive `Error` + `Display` for the 14-variant `NyquestroError`; `#[error("…")]` format strings embed field values for readable test assertions and UI-layer messages.
- **`chrono` 0.4** — human-readable time conversion only; not used in the matching loop.
- **`url` 2** — WebSocket URL parsing.
- **`dirs` 5** — platform-canonical data directory lookup for the JSONL flight recorder.

### Runtimes / engines / platforms

- **Tokio async runtime** — only inside the live-feed task; the dashboard's main loop is synchronous and the bridge consumes `FeedEvent`s through the main-thread channel. The engine itself is not async.

### Tools

- **`cargo` (edition 2024)** with `lto = "thin"`, `codegen-units = 1` release profile; `unused_must_use = "deny"`, `needless_collect = "warn"`, `redundant_clone = "warn"` lints.
- **`examples/` smoke binaries** — `live_smoke.rs` (Coinbase end-to-end verifier, ~4.0KB) and `telemetry_smoke.rs` (JSONL writer verifier flushing 9 representative events, ~2.8KB). Run manually because they have side effects (network / disk).
- **Headless mode** (`cargo run -- --no-tui --seed 42`) — the engine + simulator + metrics + telemetry stack runs without a terminal and prints a deterministic 6-line summary; the compile-and-correctness gate that doesn't require a TTY.

### Domains and concepts

- **Limit order book mechanics** — strict price-time priority (best price first, FIFO within price), self-match rejection at match time (aggressor wholly rejected, resting untouched), match price equals resting price (aggressor gets price improvement on cross), top-of-book quote semantics (snapshot+compare, emits only on best-of-side change including the side-cleared case).
- **Multi-instrument routing** — `Market::submit_limit(order)` reads `order.symbol()` and routes to the per-symbol `OrderBook`; symbols auto-register on first submit; `Symbol(u64)` 8-byte ASCII pack gives `Copy + Hash + Ord + Eq` with allocation-free hot path.
- **Microstructure inspection** — `microprice` (size-weighted mid), `ofi(n)` (order flow imbalance over top n levels), `spread_cents`, `depth(n)`, `level_counts`, `top_n_bids/asks(n)`. The dashboard's engine pane consumes these directly.
- **Classical microstructure simulation** — Ornstein-Uhlenbeck mean-reverting mid (`dX_t = θ(μ - X_t)dt + σ dW_t`), independent Poisson arrival processes per side with configurable intensities, log-normal order sizes (right-skewed: most small, occasional large), truncated geometric price selection centred at mid.
- **Deterministic engine execution** — `OrderBook::submit_limit` never calls `Ts::now()`; resting-order timestamps are reused for fills; the aggressor's timestamp is reused for the placed/rejected lifecycle. Two `MarketSimulator::new(_, 42)` runs produce byte-identical `Vec<SimAction>` outputs; the integration test `run_twice_identical_sequence_identical_output` pins this contract across the full simulator→engine→event-stream pipeline.
- **HDR-histogram tail-latency tracking** — p50 through p9999 per `Op`; the dashboard surfaces all percentiles because the working trader looks at the right edge of the distribution. Histogram autoresizes rather than panicking on outliers.
- **Sliding-window counter design** — `WindowedCounter` with lazy pruning on record (entries older than 5 min dropped); `sum_within(now, window)` walks the deque newest-first with `take_while` until the cutoff; memory bounded by retention × event rate.
- **Bounded-channel backpressure as a freeze defence** — `try_send` on a `sync_channel(8192)` with drop-on-full and an atomic drop counter; the same lesson the Coinbase-snapshot incident taught at the input boundary (`SNAPSHOT_LEVEL_CAP=50`) applied at the telemetry output boundary.
- **Live market data integration via free public WebSocket** — Coinbase Advanced Trade `level2` channel with no API key, no signup, no auth; L2-to-virtual-order translation with `(Symbol, Side, Px) → OrderID` map for idempotent cancellation against per-level diffs.
- **One-way state machine via `const fn`** — `Status::can_transition_to(self, next)` compile-time-evaluable; backward transitions and terminal-state mutations rejected with `NyquestroError::InvalidStatusTransition { order_id, from, to }`.
- **`Copy`-friendly allocation-free events** — `FillEvent`, `QuoteEvent`, `OrderEvent` all `Copy + Clone + Eq + Hash`; designed for fan-out and replay without cloning costs; `tests/events_test.rs::events_are_copy` static-asserts via `fn assert_copy<T: Copy>(_: T)`.
- **Block-element sub-cell-precision rendering** — `theme::block_bar(ratio, width)` with `▏▎▍▌▋▊▉█` for 1/8-cell precision; the visual primitive behind smooth DOB bars as quantities change.
- **ANSI-16 colour discipline** — every colour is `Color::Reset` or one of the ANSI 16 (Black/Red/Green/Yellow/Blue/Magenta/Cyan/White plus Light variants); hardcoded RGB would break user terminal themes (Solarized, Catppuccin, Tokyo Night, accessibility palettes). Tested across Catppuccin, Solarized-light, Tokyo Night.

## Key technical decisions

| D# | Decision | Rejected alternative | Why |
|---|---|---|---|
| D1 | Safe Rust only — no `unsafe` anywhere | Standard `unsafe`-permitting design for performance | Compiler guarantees data-race freedom; constrains future perf optimisation but eliminates an entire class of bugs |
| D2 | Correctness before performance | Lock-free structures from day one | Build a correct deterministic engine first with standard-library containers; lock-free can be introduced later behind a stable API |
| D3 | Events as immutable `Copy` frames | Heap-allocating events with `String` reasons | Compatible with fan-out and replay without cloning costs; the `OrderRejectionReason` enum exists because a `String` reason field would break `Copy` |
| D4 | Cents-based `Px` (u64), not floating-point | `f64` prices | Integer comparison is exact, deterministic, fast; no floating-point precision surprises in price comparison |
| D5 | Nanosecond `Ts` (u64) | `Instant` or `chrono::DateTime` | Matches HFT granularity; single `u64` is `Copy` and comparison-friendly; wraps at ~584 years from epoch (not a practical concern) |
| D6 | Severity as classification (method), not hierarchy | Separate `RecoverableError` / `FatalError` enums | Flat enum + exhaustive `severity()` match is a single source of truth; adding a variant forces a compile error in one place |
| D7 | Order takes a caller-supplied `Ts` | `Order::new` reads `Ts::now()` internally | Internal `Ts::now()` made matching engine determinism impossible; explicit timestamp parameter plus `Order::new_now` convenience for non-deterministic callers |
| D8 | `Order::fill` returns `NyquestroResult<()>`, not `FillEvent` | `Order::fill` returns the event (tried Dec 22, reverted Dec 23) | Coupling `order` to `events` violates dependency direction; the engine, not the order, has the context to construct a `FillEvent` (buyer ID, seller ID, matched price are not properties of one order) |
| D9 | Multi-instrument via `Symbol(u64)` 8-byte ASCII pack | `String`-keyed `HashMap<String, OrderBook>` | Allocation-free, `Copy`, fits in a register; lex-ordering by integer compare is a free side-benefit; 8-byte limit fits stock tickers and crypto pairs (futures with longer tickers would need `[u8; 16]` extension) |
| D10 | Coinbase Advanced Trade `level2` WebSocket, no auth | Coinbase Pro `level2` (API key required); paid feeds | "Zero-cost-forever" data constraint; Coinbase's public stream is highest-quality free source (continuous depth, sub-second cadence, multi-symbol). `native-tls` over `rustls-tls` was pragmatic — rustls panicked at startup with `CryptoProvider` ambiguity; native-tls yields macOS system trust store for free |
| D11 | `SNAPSHOT_LEVEL_CAP=50` + `PER_FRAME_BUDGET=500` (both) | Either one alone | Naive integration of 25,000+ levels per side on first connect froze the dashboard so badly that `q` could not quit; two-pronged defence at input boundary AND dispatch boundary; either alone insufficient |
| D12 | JSONL telemetry — local-only, truncate-on-startup, drop-on-full | Rotated logs; remote shipping; in-memory ring | JSONL keeps the file `cat | jq`-able; truncate bounds disk usage by definition; drop-on-full is the structural guarantee that telemetry can never freeze the dashboard (same lesson as the Coinbase snapshot at the input side) |
| D13 | Dashboard infographics (gauges, sparklines, distribution bars, health dots, pressure bars) | Numeric tables | Numeric tables don't help the eye scan a real-time stream; HFT trading-floor culture is terminal-first AND visual-first; the infographics layer is what turns "a matching engine that runs in a terminal" into "a piece of internal HFT tooling that's actually pleasant to watch live" |

## What is currently built

The 2026-05-04 step-change session shipped the foundational MVP across 8 commits in one day, taking the project from "core types + event frames + empty matching_engine.rs" to a working multi-instrument matching engine with a live Ratatui observability dashboard, a Coinbase Advanced Trade WebSocket bridge, and a local-only JSONL flight recorder. Concretely:

- **`src/types.rs` (~15.9KB)** — `OrderID`, `Symbol`, `Side`, `Px`, `Qty`, `Ts`, `Status` primitives; `Symbol::from_const` compile-time literal + `Symbol::from_str` runtime constructor; `Px::from_cents` / `Px::from_dollars` (rounding, not truncating); `Qty::checked_sub` / `Qty::ZERO` / `Qty::is_zero`; `Ts::from_nanos` deterministic constructor + `Ts::now()` reserved for ad-hoc callers; `Status::can_transition_to` and `Status::is_terminal` as `const fn`s.
- **`src/errors.rs` (~6.3KB)** — `NyquestroError` 14 variants; `severity()` method; `NyquestroResult<T>`; `is_recoverable` / `is_fatal` helpers; 3 inline unit tests.
- **`src/order.rs` (~8.5KB)** — `Order` with caller-supplied timestamp, `Order::new_now` convenience, all `&self` accessors, `fill` using `checked_sub` with three guards (terminal / zero / over-fill), `cancel`, `transition_to` via `Status::can_transition_to`, `Display` impl; 8 inline unit tests.
- **`src/events/` (3 files)** — `FillEvent` with active self-match check, `QuoteEvent::live` + `QuoteEvent::cleared`, `OrderEvent::{Placed, Filled, Cancelled, Rejected}`, `OrderRejectionReason` (5 variants), `QuoteSide`; ~11 inline unit tests.
- **`src/book/` (3 files, ~23KB total)** — `OrderBook` with four-phase `submit_limit`, `cancel`, microstructure inspection (`microprice`, `ofi`, `spread_cents`, `depth`, `level_counts`, `top_n_bids/asks`); `PriceLevel` (`VecDeque<Order>` with `push_back`, `front_mut`, `record_execution`, `pop_front`, `remove_by_id`); `Market` multi-instrument wrapper; 8 inline + 6+12 integration tests in `tests/price_level_test.rs` + `tests/matching_test.rs`.
- **`src/metrics/` (3 files)** — `MetricsRegistry` with HDR histograms per `Op` (Submit/Match/Cancel); `CounterSet` of five `WindowedCounter`s (orders/fills/cancels/rejects/quotes); `WindowedCounter` with rolling 1s/10s/1min/5min snapshots; `RegistrySnapshot` value-type for the dashboard.
- **`src/simulator/` (1 main file)** — `MarketSimulator` (Ornstein-Uhlenbeck + Poisson + log-normal + truncated geometric, `ChaCha8Rng`-deterministic, symbol-scoped); `SimAction` enum.
- **`src/feed/` (2 files)** — Coinbase Advanced Trade `level2` WebSocket client with reconnect+exponential-backoff (250ms → 30s cap); L2-to-virtual-order Bridge maintaining `HashMap<(Symbol, Side, Px), OrderID>`; `SNAPSHOT_LEVEL_CAP=50`.
- **`src/telemetry/` (3 files)** — JSONL flight recorder; bounded `sync_channel(8192)` with `try_send` and `AtomicU64` drop counter; ~19 `TelemetryEvent` variants; truncate-on-startup; platform-canonical data path via `dirs::data_local_dir()`.
- **`src/ui/` (3 files, ~73KB total)** — `theme.rs` (palette + `block_bar` 1/8-cell renderer), `app.rs` (`App` state, run loop, key mapping, terminal setup/restore), `panes.rs` (six-pane render); `PER_FRAME_BUDGET=500`; ANSI-16-only colour discipline; `Tab` cycle between symbols.
- **`examples/` (2 binaries)** — `live_smoke.rs` Coinbase end-to-end verifier, `telemetry_smoke.rs` JSONL writer verifier.
- **Tests** — 88 tests at last full run (47 inline unit tests in `src/` + 41 integration tests across `tests/types_test.rs`, `tests/order_test.rs`, `tests/events_test.rs`, `tests/price_level_test.rs`, `tests/matching_test.rs`). Determinism is positively tested: `run_twice_identical_sequence_identical_output` pins byte-identical engine output across runs. Test-to-source ratio ~0.23; for the state-machine code specifically the ratio is closer to 1:1 lines and ~3:1 in test count to mutation paths.

Not yet implemented (next-tier scope): binary UDP gateway / FIX TCP acceptor, market data multicast, risk guard layer with fat-finger protection / position+PnL / VaR circuit breaker, strategy agent (book reconstructor, two-sided quoter, inventory tracker, adverse-selection detection), lock-free internals, slab allocator, thread-to-core affinity, kernel bypass, SIMD price comparison, market/IOC/FOK/AON orders, order modification, hidden quantity, indexed cancellation, property-based test harness, `criterion` benchmarks, ITCH replay harness, C++ reference implementation, formal verification with Kani, VSR consensus for v2 distributed extension.

## Current state

Status: `active`. HEAD commit is `cc1deb0` (2026-05-05, docs-only repoint of learning references). Last meaningful commit is `6516eb6` (2026-05-04, `docs(context): regenerate after Phase A+B+C+D ship`); the 2026-05-05 commits are documentation-only learning-archive churn. 35 total commits Jun 2025 – May 2026. The work pattern is bursty: six clear gaps of 30+ days punctuate the project's history, with intense focused sessions ending each idle period. The 2026-05-04 step-change session accounts for ~50% of the project's lifetime commits. As of 2026-05-13, 9 days have elapsed since the last meaningful commit; nothing is currently in flight in `Work/` beyond the README Demo recording (pending) and the proposed V2 distributed extension (gated on additional v1 prerequisites).

## Gaps and known limitations

- **No property-based tests of matching invariants.** `proptest` for price-time priority, no-self-match, partial-fill arithmetic sums, post-state invariants is filed as the highest-leverage outstanding work in the repo's `context/plans/extensive-testing-framework.md`. HFT firms expect this; today's coverage is example-based.
- **No stress harness.** "Millions of orders/sec sustained, p99.99 stable" is the HFT-firm capacity test; capacity claims are currently unsubstantiated.
- **No criterion benchmarks.** Per-op latency budget and regression detection are tracked in the testing-framework plan but not yet implemented.
- **No mutation testing or coverage measurement.** `cargo-mutants` + `llvm-cov` would reveal blind spots in the example-based suite.
- **No CI pipeline.** Tests run locally only; `cargo test` works but isn't automated.
- **`OrderBook::cancel` is O(N_levels × N_orders_per_level).** Acceptable for the dashboard's use case (cancellations are rare relative to fills, resting-id cache refreshes every 250ms) but a high-cancel-rate HFT workload would need an `OrderID → (Side, Px, queue_pos)` index for O(log n) or O(1) cancellation.
- **Engine is single-threaded.** No atomics, no mutexes, no lock-free structures. Deliberate per D2 (correctness before performance) — the `BTreeMap`+`VecDeque` internals can be swapped for atomic structures without changing `submit_limit`'s signature, but the swap hasn't happened.
- **No wire protocol.** No binary UDP gateway, no FIX TCP acceptor, no market data multicast. README describes these; code does not.
- **No risk layer.** No fat-finger protection, no position tracking, no VaR circuit breaker, no per-session throttles. `risk-layer.md` plan filed in `context/plans/`.
- **No strategy agent.** No book reconstructor, no two-sided quoter, no inventory tracker, no adverse-selection detection.
- **No sequence-gap detection on Coinbase L2.** The bridge applies updates without tracking Coinbase's per-update sequence number; recovery from gaps would require requesting a fresh snapshot.
- **`Symbol` 8-byte ASCII limit.** Fits equities and most crypto pairs; some futures contracts with longer tickers don't fit (e.g. Eurex `FGBLM26`). Future extension to `[u8; 16]` would lose register-size.
- **`Px::from_dollars` still loses sub-cent precision** (cents are the smallest unit). `$10.005` rounds to 1001 cents, not stored separately. By design — matching is integer-only.
- **No automated UI tests.** Ratatui's `TestBackend` is available but unwired; visual correctness is verified manually plus the headless smoke binary.
- **Panic during render does not restore the terminal.** A `std::panic::set_hook` calling `restore_terminal` would close it; not yet implemented.
- **`TelemetryEvent::PaneRender` is declared but not emitted.** Trivial next iteration.
- **No `Display` impl for `Symbol`** — `as_str()` is the only render path.
- **`Order::new_now()` and `Ts::now()` `.unwrap()` on `SystemTime::now().duration_since(UNIX_EPOCH)`.** Would panic if the system clock is before 1970 (extremely unlikely; would be the only project-wide path that violates the error-handling philosophy).
- **Long simulator step at high speed multiplier stalls render.** `speed = 50.0` with high arrival rates on three symbols can drop frames because engine + UI share a thread. Deliberate per D2; default settings don't exercise it.

## Direction (in-flight, not wishlist)

- **Extensive testing framework** (the unambiguous Tier-1 next pick per the repo's `context/notes/hft-firm-priorities.md` §8). Plan filed at `context/plans/extensive-testing-framework.md` (27.6KB, 5-day buildout): Day 1 `proptest` property tests for matching engine invariants; Day 2 `proptest-state-machine` for stateful properties across submit/cancel sequences; Day 3 `criterion` benchmarks with regression budget; Day 4 `insta` snapshot tests + stress harness exercising millions of orders/sec; Day 5 `llvm-cov` coverage + `cargo-mutants` mutation testing + CI integration. This is the highest hiring-signal-per-hour iteration available — HFT firms ask "how do you test a matching engine?" and this plan is the answer.
- **README demo recording** (`Work/README Demo.md`, pending). NeuroDrive-style hero gif at the top of the README capturing the live dashboard's signal flow; gif+mp4 in `media/` folder; 10s loop minimum for the hero, optional 15-20s scenes for secondary views.

## Demonstrated skills

- Designing and implementing a correct, deterministic, multi-instrument limit-order matching engine from scratch in safe Rust — strict price-time priority via `BTreeMap<Px, PriceLevel>` + `VecDeque<Order>` FIFO, four-phase `submit_limit`, self-match rejection at both engine and event layers, top-of-book snapshot+compare quote emission, byte-deterministic output across runs (positively tested via `run_twice_identical_sequence_identical_output`).
- Engineering a strictly layered crate (10 modules, ~30 files, ~6.5k LOC) with no import cycles enforced mechanically by the import graph — every component is `pub` for direct integration-test composition, layering documented in `lib.rs`, and a previously-tried coupling violation (Order → Events, December 2025) was reverted within 24 hours when the architecture problem was recognised.
- Designing strongly-typed, allocation-free, `Copy`-friendly domain primitives (`OrderID`, `Symbol(u64)` 8-byte ASCII pack, `Px` in cents, `Qty` with `checked_sub`, `Ts` in nanoseconds, `Status` with `const fn can_transition_to`) that fit register-sized in the hot path while preserving compile-time state-machine guarantees.
- Building real-time observability tooling for trading systems: HDR-histogram tail-latency tracking (p50 through p9999 per operation, autoresize on overflow), sliding-window event counters with lazy pruning, a `Copy`-able snapshot value-type that lets render code work without holding live-registry references, and a six-pane Ratatui dashboard with sub-cell-precision block-element bars, distribution-shape latency bars on log scale, mid-price Braille charts, pressure bars, sparklines, and health-dot indicators — all in ANSI-16-only colour palette compatible with user-curated terminal themes.
- Integrating live external market data via free public WebSocket: `tokio-tungstenite` with `native-tls`, exponential reconnect backoff, L2-to-virtual-order translation with idempotent cancellation via `HashMap<(Symbol, Side, Px), OrderID>`, and structural freeze-defence (`SNAPSHOT_LEVEL_CAP=50` at the wire boundary plus `PER_FRAME_BUDGET=500` at the dispatch boundary) tested against a real 25,000+-level Coinbase snapshot that previously froze the dashboard.
- Building a from-scratch synthetic market simulator combining three classical microstructure models (Ornstein-Uhlenbeck mean-reverting mid with `dX_t = θ(μ - X_t)dt + σ dW_t`, independent Poisson arrival processes per side, log-normal sizes, truncated geometric price selection) with `ChaCha8Rng` as the sole randomness source for replayability.
- Designing a local-only structured-telemetry flight recorder (~19 `TelemetryEvent` variants, JSONL output, `try_send` on a bounded `sync_channel(8192)` with `AtomicU64` drop-counter, truncate-on-startup, `BufWriter<File>` writer thread, schema designed for `cat | jq` analysis with `kind`-tagged variants and `FrameSlow` reason classification).
- Reasoning explicitly about thirteen named architectural decisions (D1–D13) with rejected alternatives and trade-offs documented — including the `rustls` → `native-tls` switch rationale, the `Symbol(u64)` design choice, the `SNAPSHOT_LEVEL_CAP`+`PER_FRAME_BUDGET` two-pronged defence, severity-as-method-not-hierarchy, and dashboard infographics over numeric tables.
- Maintaining engineering discipline across long idle periods: 132 days between scaffolding the empty `src/matching_engine/order_book.rs` and implementing it; 40 days of pure silence preceded the 2026-05-04 step-change. Documentation-heaviness (README, repo `context/`, vault Notes) is the structural defence — Caner returned after 40 days of silence and shipped a multi-phase implementation in one day, with the prior `IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md` plan landing largely as written.
- Hardening a foundation against silent state corruption: replacing `saturating_sub` with `checked_sub` for over-fill rejection; replacing `get_status(self)` with `status(&self)` to eliminate consume-on-read; adding `Status::can_transition_to` as a `const fn` enforcing one-way transitions; enabling a previously-commented self-match check in `FillEvent::new` as defence-in-depth behind the engine's own check.
- Designing a unified `SimAction` abstraction that lets the matching engine consume synthetic and live data identically — same `Submit(Order)` shape, same `Cancel` semantics, with `CancelHint` (synthetic-only intent without referent) and explicit `Cancel { symbol, id }` (feed-only retraction) covering the two producer asymmetries.
- Writing comprehensive integration tests for engine semantics: simple cross, three-level sweep, partial-then-rest, FIFO within a level, self-match rejection (asserting the resting order is untouched), cancellation success and unknown-id paths, determinism across runs on a 6-order sequence, top-of-book quote semantics (asserting no quote emitted on worse-than-best resting), aggressor-full-fill-does-not-rest, aggressor terminal state — 12 integration tests in `tests/matching_test.rs` plus inline coverage per module.

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
