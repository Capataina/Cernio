---
name: Aurix
status: active
source_repo: https://github.com/Capataina/Aurix
lifeos_folder: Projects/Aurix
last_synced: 2026-05-13
sources_read: 29
---

# Aurix

## One-line summary

Local-first, zero-cost, read-only DeFi analytics desktop application (Tauri 2 + React 19 + Rust) built around a clean-room Uniswap V3 LP backtester — clean-room Q64.96 tick mathematics, SQLite persistence with WAL + writer-thread + r2d2 reader pool, 3-tier free-data ingest fallback (subgraph → Alchemy → public RPC → empty state), full cartesian strategy grid, multi-asset benchmark comparison (DeFi yields + tradfi + staking + HODL), and an adaptive-tercile vol-regime classifier feeding a verdict-prose synthesis of "should I have just held/staked/yielded instead?".

## What it is

Aurix is a Tauri 2 desktop application pairing a React 19 + TypeScript 5.8 frontend with a Rust backend (Tokio runtime, edition 2021) to monitor and analyse decentralised exchange markets entirely on-device — no cloud backend, no wallet connection, no transaction capability. The core premise is that serious DeFi analytics tooling either costs money (premium APIs, subscriptions) or requires trusting a third party with the user's monitoring activity; Aurix makes the computation local, the data source public (free on-chain RPC + free hosted subgraphs + DefiLlama + FRED), and the scope deliberately read-only.

The project hosts two shipped tabs and three roadmap tabs: **Tab 1** is a cross-DEX arbitrage detection dashboard scanning four venues (Uniswap V3 5bps, Uniswap V3 30bps, Uniswap V2, SushiSwap) for WETH/USDC at a default 12-second tick (one Ethereum block post-Merge), with per-leg-fee round-trip P/L math, severity-graded insight cards, and a four-mode SVG chart; **Tab 2** is the Vector A Uniswap V3 LP backtester — a research-grounded, code-complete end-to-end LP backtesting stack (M2.0 SQLite persistence → M2.1 archive ingest → M2.2 Q64.96 maths → M2.3 per-swap simulation engine → M2.4 validation harness → M2.5 strategies grid → M2.7 multi-asset benchmarks → M2.8 capital-allocation headline + verdict prose), extended in one session with a four-tier cross-chain / V3-forks / non-USD-quote-pools generalisation.

LifeOS frames Aurix explicitly as **the crypto-domain hiring project in Caner's portfolio**, sitting alongside Cernio, NeuroDrive, Image Browser, and Nyquestro. The portfolio strategy LifeOS articulates is *depth-on-one-vector over breadth-across-shallow-tabs*: a genuinely deep Tab 2 (verified Q64.96 math + multi-asset benchmarks + capital-allocation verdict) signals more to a quant-LP desk than five shallow tabs would.

The current build state per LifeOS Overview frontmatter is `active`; Vector A is code-complete and audit-improved but live-data verification is blocked by a truncated `.env` Alchemy key (~21 chars vs typical 32+) combined with a broken Alchemy account login the user could not currently restore at last verification.

## Architecture

Aurix runs two separate runtimes communicating exclusively via Tauri's IPC mechanism. The boundary is strict — no shared memory, no shared filesystem, no WebSocket. The 2026-05-03 sprint generalised the original "single-feature single-page" runtime into a "multi-feature multi-tab + shared storage + cross-cutting telemetry" runtime.

### Repository shape (from LifeOS Architecture)

```
Aurix/
├── README.md
├── package.json + pnpm-lock.yaml
├── public/
├── src/                              # React 19 + TS 5.8 frontend (~75 files)
│   ├── App.tsx                       # Multi-tab routing root (186 lines)
│   ├── main.tsx
│   ├── features/
│   │   ├── arbitrage/                # Tab 1
│   │   └── lp-backtest/              # Tab 2 (8 files)
│   ├── components/
│   │   ├── shell/                    # TopBar, SettingsMenu (route-aware)
│   │   ├── primitives/               # Heatmap, Icon, Dial, RangeIndicator
│   │   └── blocks/{arbitrage,lp,shared}/
│   ├── hooks/                        # useMarketData, usePersistedState
│   ├── lib/                          # telemetry.ts (397 lines), arbitrage.ts, format.ts, venues.ts
│   └── styles/{tokens.css, dashboard.css, components/* (12 files)}
├── src-tauri/                        # Rust backend (~98 files, 821KB)
│   ├── Cargo.toml + tauri.conf.json + capabilities/default.json
│   └── src/
│       ├── main.rs + lib.rs (Tauri builder + 19 IPC handlers)
│       ├── config/                   # mod.rs (env + AppConfig) + chains.rs (5 chains × 3 protocols)
│       ├── ethereum/client.rs        # Read-only JSON-RPC transport
│       ├── market/types.rs           # Tab 1 cross-runtime DTOs
│       ├── dex/                      # Tab 1 — uniswap_v2.rs + uniswap_v3.rs
│       ├── commands/                 # market.rs (Tab 1: 3) + lp.rs (Tab 2: 14, 628 LOC, fan-out 16) + telemetry.rs (2)
│       ├── storage/                  # M2.0 — 12 submodules + V001 + V002 migrations
│       ├── math/                     # M2.2 — q96 + tick + liquidity + fees + il (clean-room V3 port)
│       ├── ingest/                   # M2.1 — ArchiveSource trait + Alchemy + Subgraph + Mock + decoder
│       ├── backtest/                 # M2.3 — engine + price + metrics + gas + position + rebalance
│       ├── validation/               # M2.4 — synthetic fixtures (real pending)
│       ├── strategies/               # M2.5 — GridRunner cartesian search
│       ├── benchmarks/               # M2.7 — DefiLlama + TradFi + beaconchain + V2-LP + HODL + alpha
│       └── headline/                 # M2.8 — adaptive-tercile classifier + verdict prose
├── context/                          # 1.16MB project memory (54 files: architecture, systems, notes, plans, references)
└── learning/                         # 56-file curriculum
```

### Two-runtime model

```
┌──────────────────────────────────────────────────────────────────┐
│  FRONTEND  (Chromium WebView, src/)                              │
│  App.tsx                                                         │
│    ├── activeTabId state ───────► tab routing                    │
│    ├── usePersistedState × 8 ───► localStorage UI state          │
│    ├── TopBar + SettingsMenu (route-aware)                       │
│    └── <main>                                                    │
│         ├── ArbitragePage (Tab 1) — 12s tick                     │
│         │     └── invoke("fetch_market_overview")                │
│         └── LpBacktestPage (Tab 2) — auto-run on settingsKey     │
│              └── 9 IPC calls per pipeline run                    │
│  telemetry.ts ◄── every IPC + click + change + lifecycle         │
└─────────────────────────┬────────────────────────────────────────┘
                          │  Tauri IPC (JSON, camelCase)
                          ▼
┌──────────────────────────────────────────────────────────────────┐
│  BACKEND  (Native Rust, src-tauri/)                              │
│  lib.rs::run                                                     │
│    ├── tokio::Builder::new_multi_thread.build()                  │
│    ├── runtime.block_on(open_storage())                          │
│    │       └── Storage::open(~/.aurix/aurix.sqlite)              │
│    │           ├── refinery migrations V001 + V002               │
│    │           ├── tokio-rusqlite writer (one thread)            │
│    │           └── r2d2 reader pool                              │
│    ├── spawn_wal_checkpoint_task (60s tokio interval)            │
│    └── tauri::Builder::default()                                 │
│          ├── .manage(Arc<Storage>)                               │
│          └── .invoke_handler![19 commands]                       │
│  Tab 1 path: commands::market::fetch_market_overview             │
│    └── tokio::join!(V3 5bps, V3 30bps, V2, Sushi, gas)           │
│  Tab 2 path: commands::lp::run_lp_ingestion (3-tier fallback)    │
│    ↓                                                             │
│    Engine::simulate ↔ Storage ↔ math ↔ benchmarks ↔ headline     │
└──────────────────────────────────────────────────────────────────┘
```

### Critical path (Tab 2 / Vector A — 8-10 systems per pipeline run)

The dominant Vector A flow when the user clicks Re-run on the LP backtester:

```
LpBacktestPage useEffect fires (settings JSON-key changed OR rerunNonce bumped)
  ├─ telemetry.record("lp.pipeline.start", {...})
  ├─ lpPoolMetadata(addr)              ─IPC─► Subgraph: token0/token1 decimals + symbols + fee tier
  ├─ lpGetChainHead(chain)             ─IPC─► AlchemyArchiveSource.latest_finalized_block() OR public RPC
  ├─ runLpIngestion(pool, head-N, head, chain, proto)
  │     │ Tier 1: UniswapV3SubgraphSource::for_protocol → Ingester::backfill → Storage::insert_swap_events_batch
  │     │ Tier 2: AlchemyArchiveSource::from_environment (mainnet only) → Ingester::backfill
  │     │ Tier 3: AlchemyArchiveSource::with_rpc_url(public_rpc) → Ingester::backfill
  │     │ Terminal: CommandError (never synthetic fallback in user-facing pipeline)
  ├─ lpQueryFirstSwapPrice              ─IPC─► Storage range query
  ├─ lpTokenUsdPrices                   ─IPC─► DefiLlama coins API (no key)
  ├─ runLpBacktest(config, rule)        ─IPC─► Engine::simulate
  │     │ ├─ Storage::query_swaps_for_pool_range → Vec<SwapEventRow>
  │     │ ├─ parse_swaps() → Vec<ParsedSwap>     [audit hoist]
  │     │ ├─ HoldOnlyEvaluator pre-compute       [audit hoist]
  │     │ ├─ per-swap loop:
  │     │ │     ├─ math::fees::fee_share_token0/1 (in-range + share)
  │     │ │     ├─ LVR discrete approximation
  │     │ │     ├─ rebalance trigger via RebalanceContext + RebalanceRule
  │     │ │     ├─ value_usd via position_usd_value(_explicit)
  │     │ │     └─ fees_usd_acc += delta_fees_usd  [audit incremental]
  │     │ └─ Storage::persist_position_run (idempotent on config_hash)
  ├─ runLpGrid(grid_config)             ─IPC─► GridRunner per cell × 81 cells (3×3×3×3 default)
  ├─ runLpHeadline                      ─IPC─► HeadlineRunner per-month × 3 LP variants × N months
  │     │  + adaptive-tercile vol regime classifier + verdict prose synthesis
  └─ lpFetchBenchmarkSeries(key) × N   ─IPC─► DefiLlama / FRED / beaconchain providers (free)
```

Idempotency holds at every storage write — re-running the pipeline is a cache-hit at every step, so React StrictMode's intentional double-mount is structurally safe.

### Critical path (Tab 1 — Arbitrage tick)

```
ArbitragePage setInterval(loadSnapshot, 12_000ms)
  └─ invoke("fetch_market_overview")
        ├─ AppConfig::from_environment (Once-guarded dotenv)
        ├─ EthereumRpcClient::new
        ├─ tokio::join!(V3 5bps, V3 30bps, V2, Sushi, gas_price)  // FAIL-FAST
        └─ MarketOverview { venues: [..4..], gas_price_gwei, fetched_at_unix_ms }
  ← JSON camelCase ←
React state → 100-sample rolling history → derive insights → render
```

Tab 1 is fail-fast — any one venue's `.map_err` rejects the whole tick. Tab 2's 3-tier ingest fallback closes this gap structurally for the LP path.

### Inter-system relationships (Vector A — most consequential edges)

| Upstream | Downstream | Mechanism | What breaks |
|---|---|---|---|
| ingest | storage | `Storage::insert_swap_events_batch(Vec<SwapEventRow>)` via writer thread; `INSERT OR IGNORE` on `(pool, block, log_index)` | `Ingester::backfill` returns `IngestError`; LP page banners. Idempotency means partial failures recover on next run |
| storage | backtest | `Storage::query_swaps_for_pool_range` returns rows ordered `(block_number, log_index)` | `Engine::simulate` returns `BacktestError::EmptyData` when zero rows |
| math | backtest | Pure-function imports of tick/liquidity/fees primitives | `V3MathError` propagates as `BacktestError::MathError` |
| backtest | strategies | `GridRunner::run_grid` invokes `Engine::simulate` per cell sequentially | Per-cell failure halts that cell; grid continues |
| backtest | headline | `HeadlineRunner::run` invokes `Engine::simulate` per month × 3 LP variants | Per-month failure halts that month; verdict synthesises from remaining |
| commands/lp.rs | every Vector A backend | Tauri IPC dispatch (fan-out 16) | Backend error → CommandError to frontend; `key_required` lets frontend prompt structurally |
| telemetry | every page + IPC | `telemetry.record(eventName, payload)` + `telemetry_persist` IPC | Missing flush = recent events lost from `last-session.json` |

### State ownership

- **`AppConfig`** owns the resolved RPC URL (dotenv parsed once via `ENVIRONMENT_BOOTSTRAP: Once`).
- **`ChainConfig` constants** in `config/chains.rs` source-level: 5 chains × 3 protocols, per-chain block-time conventions, subgraph URLs, public RPCs.
- **`Storage` handle** is process-lifetime `Arc<Storage>` registered as Tauri-managed state (`~/.aurix/aurix.sqlite`, WAL, checkpoint every 60s).
- **Tab 1 market history** is `ArbitragePage` React state — session-only, wiped on reload, configurable rolling window (default 100).
- **Persisted UI state via `usePersistedState`** in localStorage: `aurix:refresh-interval` (default `12_000`), `aurix:pair-id`, `aurix:pnl-mode`, `aurix:hero-venue`, `aurix:history-limit`, `aurix:stale-threshold-ms`, `aurix:lp-settings` (object, gets shape-merge defence).
- **Venue metadata `VENUES`** and **chart series metadata `SERIES_META`** are module-level source constants in `ArbitragePage.tsx` and `MarketChart.tsx` — joined to backend payload by `dexName` string equality (two-copy keyspace; renaming a backend `dex_name` without updating both crashes the chart render and falls the price card to `$0.00`).

## Subsystems and components

### Storage (M2.0) — `src-tauri/src/storage/`

SQLite + WAL + writer-thread + `r2d2_sqlite::Pool` reader pool + refinery embedded migrations. Twelve domain CRUD submodules (`connection`, `migrations`, `error`, `snapshots`, `swaps`, `pool_events`, `gas`, `runs`, `strategy`, `benchmarks`, `headline`, `state`). Fan-in 26 (highest in the repo). Two migrations shipped: `V001__initial.sql` (M2.0 — all Vector A tables) and `V002__multi_asset_headline.sql` (M2.8 — multi-asset return columns). 18 unit/integration tests covering per-table round-trip and idempotency. Key invariants enforced at this layer:

- **Idempotency via domain-natural composite keys** — `INSERT OR IGNORE` on `(pool_address, block_number, log_index)` for events; `config_hash` (deterministic SHA over input config) for `position_runs` and `headline_runs`; `(grid_id, cell_index)` for `strategy_results`.
- **Address-case normalisation** — pool addresses lowercased on both insert and query paths (closes a real silent-empty-dashboard bug fixed in commit `c1e5594`).
- **Synthetic-vs-live separation via `SYNTHETIC_TX_HASH = "0x...deadbeef"` sentinel** — `delete_synthetic_swaps_in_range` purges only synthetic rows (real Ethereum collision probability ~2⁻²²⁴).
- **TEXT decimal big-integer encoding** — `amount0`, `amount1`, `sqrt_price_x96`, `liquidity` stored as TEXT decimal strings (SQLite INTEGER would lose precision above 2⁶³); `tick` stored as INTEGER `i32`.
- **`benchmark_series` is the lone `INSERT OR REPLACE` table** — FRED and similar providers revise older series prints, so replace-on-collision keeps the cache aligned with provider truth.
- **WAL checkpoint cadence** — `Storage::checkpoint()` issues `PRAGMA wal_checkpoint(TRUNCATE)` every 60 seconds via a Tokio interval task spawned in `lib.rs::run` (added 2026-05-04 to close unbounded WAL growth under sustained ingest).

### Math (M2.2) — `src-tauri/src/math/`

Clean-room port of Uniswap V3's mathematics — no `ethers-rs`, no third-party V3 SDK, no transitive crypto dependency. Built on `num-bigint::BigUint` rather than fixed-width `U256`. Layout: `mod.rs` + `error.rs` (V3MathError) + `q96.rs` (Lazy<BigUint> constants + `mul_div` / `mul_div_round_up`) + `tick.rs` (TickMath.sol port, 290 lines, with `Lazy<[BigUint; 20]>` magic-constant precompute) + `liquidity.rs` (LiquidityAmounts + SqrtPriceMath port, 322 lines, per-direction rounding) + `fees.rs` (`fee_share_token0/1`, `bps_to_protocol_units`) + `il.rs` (V2 + V3-concentrated closed forms). Fan-in 12 (second-most-imported). `MIN_TICK / MAX_TICK = ±887_272`. `sqrt_price_x96_to_tick` uses f64 log-estimate + ±2-tick refinement (faster than Solidity's bit-walking inverse, correctness pinned by round-trip tests). 30+ unit tests — most heavily tested subsystem in the repo.

### Ingest (M2.1) — `src-tauri/src/ingest/`

Three-tiered fallback chain letting Aurix run without paid API keys. Files: `mod.rs` (Ingester + IngestionReport + ArchiveSource trait + tests, 392 lines, 6 pipeline tests) + `error.rs` (with `KeyRequired` variant) + `source.rs` (EthLog + trait) + `decoder.rs` (ABI decoders for Swap/Mint/Burn/Collect, 515 lines, 10+ tests) + `alchemy.rs` (chunked `eth_getLogs` + finalized-block helpers) + `subgraph.rs` (UniswapV3SubgraphSource — hosted GraphQL backfill, per-chain × per-protocol URLs, 549 lines) + `pipeline.rs` + `mock.rs`. Composite hotspot 0.95 — central to every Vector A flow. The `ArchiveSource` trait provides `fetch_logs`, `latest_finalized_block`, `block_gas_price_gwei`, `pool_metadata`. The fallback chain is orchestrated server-side in `commands::lp::run_lp_ingestion` and ends in `CommandError`, never in synthetic fallback. `IngestionReport` carries `source_label` (which tier succeeded) and `attempted_sources` (any tier errors along the way).

### Backtest Engine (M2.3) — `src-tauri/src/backtest/`

Per-swap simulation engine — load-bearing computational core. Composite hotspot 0.93. Layout: `mod.rs` (Engine + tests fixture + 6 integration tests, 306 lines) + `engine.rs` (the per-swap loop, 488 lines post-audit) + `error.rs` + `gas.rs` (MgmtGasOp + cost_usd + mev_haircut_usd) + `metrics.rs` (Sharpe / Sortino / Calmar / max-drawdown / annualise) + `position.rs` (PositionConfig + tick-spacing alignment validation) + `price.rs` (`sqrt_price_x96_to_human_price` + `position_usd_value(_explicit)`) + `rebalance.rs` (RebalanceRule enum: Static / Schedule / OutOfRange + RebalanceContext). `Engine::simulate(config, rule)` validates, loads swaps via storage, pre-parses to `Vec<ParsedSwap>` once, initialises position from first swap's `sqrt_price_x96`, runs the per-swap loop (in-range check → `math::fees::fee_share_token0/1` → LVR discrete approximation `0.5 × Δsqrt² × L / (sqrt × Q96)` → rebalance trigger → position USD value → hold-only baseline → `il_usd = raw_position_value - hold_only_usd` → incremental `fees_usd_acc`), aggregates run summary (max DD, Sharpe, Sortino, Calmar, time-in-range, rebalance count), persists to `position_runs` + `equity_curve_points` keyed on `config_hash`. Audit (2026-05-04) applied three perf wins: pre-parse swap rows once (Data Layout), `HoldOnlyEvaluator` hoist (loop-invariant hoist), incremental `fees_usd_acc` running sum (quadratic → linear in swap count, ULP drift bounded at ~`2e-11` USD for typical 10k-swap scale).

### Strategies (M2.5) — `src-tauri/src/strategies/`

Cartesian grid over `range × rule × deposit × period`. `GridConfig` axes: `range_widths_ticks: Vec<i32>`, `rebalance_rules: Vec<RebalanceRule>`, `deposit_splits: Vec<(f64, f64)>`, `period_days: Vec<u32>`. Default grid is 3 × 3 × 3 × 3 = 81 cells. Per cell ~1000 swap rows × per-swap cost ≈ a few seconds, so default grid ~1-2 minutes sequential. `GridRunner::run_grid` constructs a `PositionConfig` per cell, calls `Engine::simulate` (cached via `config_hash`), aggregates into a `StrategyResultRow`, persists. Per-chain block-time conventions (Ethereum 12s, Arbitrum 0.25s, Optimism 2.0s, Base 2.0s, Polygon 2.2s) convert period-days to block ranges via `ChainConfig::block_time_seconds`. 3 tests covering cell-count multiplication, human→raw rounding, empty-axis rejection.

### Benchmarks (M2.7) — `src-tauri/src/benchmarks/`

9-series multi-asset reference set. Files: `mod.rs` + `error.rs` + `http.rs` (`ReqwestFetcher` + `MockHttpFetcher` trait) + `defi.rs` (DefiLlamaProvider — Aave V3 USDC supply, Compound V3 USDC supply, Lido stETH APYs, all free no-key) + `tradfi.rs` (TradFiProvider — FRED CSV parsing for `DGS3MO`, `SP500`, `LBMA gold`, all free no-key) + `beaconchain.rs` (ETH.STORE staking yield — KEY_REQUIRED, surfaces `IngestError::KeyRequired("beaconchain")`) + `v2lp.rs` (synthetic V2 constant-product LP baseline) + `hodl.rs` (price-only baseline) + `alpha.rs` (period + rolling 30/60/90-day alpha decomposition). HTTP layer gained a 15s timeout in commit `391eadd` after DefiLlama Aave/Lido fetches hung indefinitely. Stooq → FRED swap in the same commit closed silent VOO benchmark breakage after Stooq added an API-key requirement. 4+ persistence tests including replace-on-duplicate-key.

### Headline (M2.8) — `src-tauri/src/headline/`

Adaptive-tercile vol regime classifier + verdict prose synthesis. Two files: `mod.rs` (HeadlineConfig + HeadlineRunner + HeadlineOutput) + `verdict.rs` (classifier + prose synthesis, 414 lines). For each backtest window: computes realised volatility per month, partitions into low/mid/high terciles by observed distribution (not fixed thresholds — earlier 0.5%/2.0% daily-vol fixed thresholds broke on DAI/USDC and WBTC/ETH alike), classifies each month. Per month runs three LP variants — **best** (best in-month Sharpe across strategies grid, selection-bias-inflated), **naive** (fixed ±100 ticks, Static rule, 50/50 split, no tuning), **median** (median-Sharpe cell). Each variant's monthly return compared against 6 benchmarks: Aave / Lido / S&P 500 / Gold / 3-month T-bill / HODL. The `V002__multi_asset_headline.sql` migration adds `months_lp_beat_*` aggregate columns. Verdict synthesises win-counts into English prose ("In low-vol months, naive LP beat Aave 7/12 times with a median spread of +1.4% APY...").

### Validation (M2.4) — `src-tauri/src/validation/`

The "are we computing the right numbers?" harness — independent of in-crate unit tests, which only verify primitive correctness. Layout: `mod.rs` (ValidationReport + driver, 170 lines) + `error.rs` + `synthetic.rs` (synthetic fixture generator). Two fixture modes: synthetic round-trip (working today, 1 test pinning `synthetic_fixtures_round_trip` within tolerance) and real-on-chain fixtures (planned, not started — requires curated mainnet LP positions + burn-tx receipt parsing for ground-truth fees). The synthetic round-trip is the only end-to-end engine validation surface today; the real-fixture diff is the highest-impact validation gap.

### LP Backtest GUI (Vector A frontend, Tab 2) — `src/features/lp-backtest/`

Auto-run pipeline orchestrator + 10 dashboard blocks. `LpBacktestPage.tsx` (668 lines, useEffect pipeline + 8 useState) + `LpSettingsForm.tsx` (409 lines, 10+ controls, route-aware via `SettingsMenu`) + `api.ts` (typed Tauri IPC wrappers, mirrors `commands/lp.rs`) + `chains.ts` (CHAIN_CONFIGS) + `pools.ts` (curated pool list per-chain × per-protocol) + `defaults.ts` (DEFAULT_GRID_RULES + DEFAULT_GRID_RANGE_WIDTHS + DEFAULT_GRID_PERIOD_DAYS) + `types.ts` (217 lines of Rust↔TS type mirrors). Blocks in `src/components/blocks/lp/`: BenchmarkCacheBlock, EquityCurveBlock, HeadlineVerdictBlock, KeyMetricsBlock, MultiAssetCompareBlock (319 lines — hero league table sorting 9 assets by cumulative return, LP highlighted), PositionPnlBlock, PositionRangeBlock, RegimePanelBlock, StrategyControlsBlock (360 lines, folded into SettingsMenu post-2026-05-03), StrategyHeatmapBlock. The page auto-runs whenever `settingsKey = JSON.stringify(settings)` changes OR `rerunNonce` bumps — typing into a stepper bumps object identity but not JSON, so the pipeline doesn't fire mid-keystroke. StrictMode discipline: the `mounted` flag gates **state setters only**, never the pipeline body — both StrictMode mounts run the pipeline; idempotency at storage makes the second mount a cache hit. `usePersistedState` shape-merge defends against future field additions to persisted objects (closes a `CHAIN_CONFIGS[undefined].label` crash fixed in commit `b01b4f4`). Zero frontend tests — Vitest setup deferred.

### Runtime Foundation — `src-tauri/src/lib.rs`, `src/App.tsx`, `src-tauri/src/config/`

Multi-tab desktop shell + IPC handler list + env-backed RPC config + WAL checkpoint task + chain configuration. `lib.rs` is 105 lines, registers 19 IPC handlers across 3 modules (3 `commands::market::*`, 14 `commands::lp::*`, 2 `commands::telemetry::*`). `config/mod.rs` resolves RPC URL via `MAINNET_RPC_URL` (preferred) or `ALCHEMY_API_KEY` (fallback) with `dotenvy` and `Once`-guarded bootstrap. `config/chains.rs` (206 lines) covers `ChainId` enum (Ethereum / Arbitrum / Optimism / Base / Polygon) × `Protocol` enum (UniswapV3 / SushiswapV3 / PancakeswapV3), per-(chain, protocol) subgraph URL routing, per-chain public RPCs and block times. `App.tsx` multi-tab routing via `activeTabId` state, `TopBar` shell with tab pills, route-aware `SettingsMenu`.

### Telemetry — `src/lib/telemetry.ts` + `src-tauri/src/commands/telemetry.rs`

Cross-cutting IPC tracer. Frontend hub (397 lines) exposes `record(eventName, payload)`, `recordError`, `recordIpcStart`, `recordIpcEnd`, `recordIpcError`, `flush()`, and a `useTelemetrySnapshot(pageName, state)` React hook. Backend pair persists buffered events to `~/Library/Logs/com.ataca.aurix/last-session.json` (macOS user-logs convention) via the `telemetry_persist` Tauri command; file cleared each app boot. Dotted hierarchical event names (`lp.pipeline.start`, `lp.pipeline.chain-head-fetched`, `settings.toggle`) make `jq` filtering trivial. Replaces an earlier screenshot-based diagnostic loop; 14 `console.log + eslint-disable` pairs in `LpBacktestPage.tsx` were converted to `telemetry.record` calls in commit `b01b4f4`. The 2026-05-04 audit cycle was driven by exactly this pattern — the `LpSettingsForm` crash diagnosis came from reading `last-session.json` post-hoc with `jq`, no screenshots required.

### Cross Runtime Contract — `src-tauri/src/market/types.rs` + `src/features/{arbitrage,lp-backtest}/types.ts` + inline DTOs in `commands/lp.rs`

Manual Rust↔TS mirror via `serde(rename_all = "camelCase")` on every IPC-crossing Rust type. Tab 1 contract has two types (`PriceSnapshot` with 8 fields; `MarketOverview` with 5 fields including `venues: Vec<PriceSnapshot>`). Tab 2 contract is much larger — `src/features/lp-backtest/types.ts` is 217 lines mirroring `PoolMetadataDto`, `FirstSwapInfo`, `TokenPricesDto`, `BacktestResponse`, `IngestionReport`, `AttemptedSource`, `PositionConfig`, `RebalanceRule`, `PositionRunSummary`, `EquityCurvePoint`, `StrategyResultRow`, `HeadlineMonthlyRow`, `BenchmarkPoint`, `GridConfig`, `HeadlineConfig`, `HeadlineMonthlyInput`, `PoolMetadata`. The standard error envelope `CommandError { message: String, key_required: Option<String> }` lets the frontend prompt structurally rather than parsing error messages. No codegen (`ts-rs` / `specta`) — the implementing engineer updates `types.ts` whenever a Rust DTO changes. The IPC mismatch bug commit `391eadd` fixed (`PositionConfig` deserialising snake_case while TS sent camelCase, producing `missing field 'pool_address'` on every backtest call) is exactly this manual-sync gap manifesting.

### Tab 1 Data Pipeline + DEX Adapters — `src-tauri/src/commands/market.rs` + `dex/uniswap_v2.rs` + `dex/uniswap_v3.rs` + `ethereum/client.rs`

The original Aurix product, pre-Vector A. `fetch_market_overview` runs `tokio::join!` over 5 concurrent futures (V3 5bps, V3 30bps, V2, SushiSwap, gas_price), fail-fast on any error. V3 adapter: one `eth_call` to `slot0()` (`0x3850c7bd`), decodes the first 32-byte word as `sqrtPriceX96`, converts to USD via `BigUint` arithmetic then casts to `f64` at the IPC boundary. V2/SushiSwap adapter: three sequential `eth_call`s (`getPair(USDC, WETH)` → `token0()` → `getReserves()`), decodes reserves, applies 10^12 decimal correction. SushiSwap shares `fetch_v2_snapshot` with Uniswap V2 — only the factory address differs. The `dex_name` strings (`"Uniswap V3 5bps"`, `"Uniswap V3 30bps"`, `"Uniswap V2"`, `"SushiSwap"`) are the implicit identity key joining backend payloads to frontend `VENUES[]` and `SERIES_META{}` metadata tables.

### Tab 1 Analytics Engine + GUI Layout — `src/features/arbitrage/`

TypeScript analytical layer producing `InsightsViewModel { primary, secondary, events }`. Threshold constants: `GAS_UNITS_ESTIMATE = 220_000`, `SHORT_WINDOW = 5`, `BASELINE_WINDOW = 20`, `PERSISTENCE_WINDOW = 4`. Severity model has four levels (`info`/`watch`/`notable`/`actionable`) selected via a priority cascade in `buildPrimaryInsight`. Four-mode SVG chart (`raw`/`deviation`/`spread`/`gas`) at 960×320 viewBox — hand-rolled, no charting library. Plain CSS architecture — no Tailwind / shadcn / MUI / Chakra / styled-components. Component tree: `ArbitragePage` owns state; renders `PriceCard` (hero readout), `MarketChart` (pure), `InsightsPanel` (pure), and 4 inline `.exchange-card` venue lanes.

## Technologies and concepts demonstrated

### Languages

- **Rust** (edition 2021) — backend across `src-tauri/src/` (79 files, ~10,500 LoC per LifeOS Overview). Every Vector A subsystem (storage, math, ingest, backtest, strategies, benchmarks, headline, validation) is Rust; per-module `thiserror::Error` enums rather than a grand unified error type; `serde(rename_all = "camelCase")` as the only wire bridge; four-line `Inputs / Outputs / Errors / Side effects` docstring convention for non-trivial public functions; zero inline `WHY`/`NOTE`/`HACK`/`TODO`/`FIXME` annotations project-wide (rationale lives in `context/` docs instead).
- **TypeScript 5.8.3** — frontend across `src/` (45 .tsx + 18 .ts files, ~9,000 LoC). Strict React 19 + hand-rolled IPC client wrappers + 217-line manual Rust↔TS DTO mirror for Tab 2 alone + telemetry recorder hub (397 lines).
- **SQL** — embedded `refinery` migrations: `V001__initial.sql` (M2.0 — all Vector A tables: `swap_events`, `pool_events`, `gas`, `position_runs`, `equity_curve_points`, `strategy_results`, `benchmark_series`, `headline_runs`, `headline_monthly`, `ingest_state`, `snapshots`) and `V002__multi_asset_headline.sql` (M2.8 — multi-asset return columns).
- **CSS** — plain CSS only across `src/styles/{tokens.css, dashboard.css, components/* (12 files)}` (per LifeOS Architecture). CSS custom properties drive a dense monitoring/trading-terminal aesthetic.

### Frameworks and libraries

- **Tauri 2** (`src-tauri/Cargo.toml`) — desktop shell + IPC framework; binary ~5–15 MB versus Electron's 100–200 MB; capability-based security model.
- **React 19.1.0** (`package.json`) — frontend framework; multi-tab routing via local state, no React Router.
- **Vite 7.0.4** — frontend build tool.
- **Tokio 1** with `macros, rt-multi-thread, sync, time` features — async runtime backing the Rust backend.
- **reqwest 0.12** with `rustls-tls` feature — HTTP client; the rustls choice means no system OpenSSL dependency, simplifying cross-platform Tauri builds.
- **rusqlite 0.31** with `bundled` feature — SQLite compiled from source, reproducible across user machines without dependence on system libsqlite3.
- **tokio-rusqlite 0.5** — async SQLite writer (one dedicated thread serialising mutations).
- **r2d2 0.8 + r2d2_sqlite 0.24** — fixed-size reader connection pool (~4 connections).
- **refinery 0.8** — forward-only embedded SQL migrations.
- **num-bigint 0.4** — arbitrary-precision unsigned integer arithmetic for Q64.96 (`BigUint` rather than fixed-width `U256` from `ruint`; the "clean-room V3 port" framing is stronger when types are general-purpose Rust primitives).
- **once_cell 1** — `Lazy<BigUint>` constants + `Lazy<reqwest::Client>` hoists.
- **async-trait 0.1** — trait async methods for `ArchiveSource`.
- **chrono 0.4** with `clock, serde` features — datetime handling.
- **thiserror 1** — per-module error enum derivation.
- **proptest 1** (dev) — property-based testing.
- **tempfile 3** (dev) — temp-file fixtures for storage round-trip tests.
- **dotenvy** — `.env` file loading with fallback path `../.env`.

### Runtimes / engines / platforms

- **Tauri 2 IPC bridge** — JSON over Tauri's invoke mechanism; `serde(rename_all = "camelCase")` is the only wire-level bridge between Rust snake_case and TS camelCase; 19 IPC handlers registered in `lib.rs::run` (3 market, 14 lp, 2 telemetry).
- **macOS user-logs convention** — telemetry writes to `~/Library/Logs/com.ataca.aurix/last-session.json` via Tauri's `app_log_dir()`; file cleared at each app boot.
- **WAL-mode SQLite** — `journal_mode = WAL`, `synchronous = NORMAL`, `foreign_keys = ON`, `temp_store = MEMORY`; writer-on-its-own-thread isolates write contention from reads under WAL.

### Tools

- **Cargo** — Rust build, dependency, and test runner; backend tests: 139 pass / 0 fail / 3 ignored at audit baseline.
- **pnpm** — Node.js package manager (`pnpm-lock.yaml` per LifeOS Architecture).
- **`refinery` embedded migrations** — forward-only migrator running on every `Storage::open`.
- **`jq`-driven diagnostic pattern** — telemetry log structure (dotted event names + JSON file) is designed for `jq '.events[] | select(.eventName | startswith("lp.pipeline."))'`.
- **No Vitest yet** — frontend test infrastructure deferred at the 2026-05-04 audit; zero frontend tests today (per LifeOS Gaps).

### Domains and concepts demonstrated

- **Concentrated-liquidity AMMs (Uniswap V3)** — full clean-room port of TickMath, FullMath, LiquidityAmounts, SqrtPriceMath ported from Solidity using `BigUint`. The 20 magic constants in `tick_to_sqrt_price_x96` (Q128.128 representations of `1.0001^(2^k)` for `k = 0..19`) are bit-exact transcriptions, allocated once via `Lazy<[BigUint; 20]>`. Inverse `sqrt_price_x96_to_tick` uses f64-log-estimate plus ±2-tick refinement (faster than the Solidity bit-walking inverse; correctness pinned by round-trip tests).
- **Q64.96 fixed-point arithmetic** — all `sqrt_price` arithmetic in 96-bit-fractional `BigUint`; `mul_div(a, b, denom) = (a * b) / denom` direct port of `FullMath.mulDiv`; per-direction rounding via `mul_div_round_up` to preserve V3's "protocol never under-charges" invariant.
- **Impermanent loss closed forms** — V2 form `2 × sqrt(r) / (1 + r) - 1` for wide-range/V2 baseline; V3-concentrated form parameterised by tick range for true narrow-range IL. Closed forms derived in `context/references/v3-lp-profitability-literature.md` (Milionis-Moallemi-Roughgarden) per LifeOS Math system note.
- **Loss-Versus-Rebalancing (LVR)** — discrete approximation `LVR_step = 0.5 × Δsqrt² × L / (sqrt × Q96)` per Milionis-Moallemi-Roughgarden, used in-range with non-zero previous sqrtPrice. The continuous-time integral form requires per-pool arbitrage-rate parameters; discrete is the standard practical substitute.
- **Per-swap fee distribution** — `fee_share_token0/1 = swap_amount × fee_tier_bps / 1_000_000 × position_L / active_L` for in-range positions; out-of-range positions receive zero. Defensive clamp `position_L ≤ active_L` (real V3 pools enforce structurally; synthetic data can violate, hence the clamp).
- **Cartesian grid search over LP strategies** — `range × rule × deposit × period`, with per-chain block-time conversion so a "30-day backtest" on Arbitrum doesn't quietly become a 48× longer block range than on Ethereum.
- **Adaptive-tercile volatility regime classification** — per-month realised vol partitioned into low/mid/high terciles by observed distribution rather than fixed thresholds. Self-calibrates per asset (a "low-vol month" for ETH/USDC differs from a "low-vol month" for WBTC/USDC).
- **Three-variant LP comparison (best / naive / median)** — selection-bias-aware verdict framing. Earlier single-variant "best LP" implicitly reported "LP wins" because of selection bias; adding naive (no tuning) and median (typical user) reframes the question from "did the perfectly-tuned LP win?" to "would a typical LP have won?" Note: Bailey/de Prado deflated-Sharpe correction is **not yet** implemented; verdict's "best-cell beat all benchmarks" remains statistically optimistic (highest-impact open statistical gap).
- **Multi-tier free-data fallback chain** — Tier 1 Uniswap V3 hosted subgraph (or fork) → Tier 2 user's Alchemy archive RPC (mainnet only) → Tier 3 free public RPC for the chain → Tier 4 explicit empty state with `CommandError`. No paid options at any tier; no synthetic fallback in user-facing flow. `KeyRequired(name)` error variant surfaces "this path needs an API key" structurally rather than via message parsing.
- **Idempotent CRUD via domain-natural composite keys** — `(pool, block, log_index)` for events; `config_hash` (deterministic SHA over `PositionConfig` / `HeadlineConfig` excluding any wall-clock or non-determining field) for backtest runs; `(grid_id, cell_index)` for strategy cells. Verification pattern: every idempotency-claiming function has a `*_is_idempotent` test (e.g. `batch_insert_is_idempotent`, `re_persisting_same_hash_is_idempotent`, `simulate_persists_to_storage_and_is_idempotent`).
- **Address-case canonicalisation** — storage layer is the single normalisation point; lowercase on both insert and query (closes a real silent-empty-dashboard bug from mixed-case user input meeting lowercase Alchemy logs).
- **Synthetic-vs-live separation** — `SYNTHETIC_TX_HASH` sentinel constant tags synthetic rows for targeted purge without touching live data; collision probability with real tx hashes ~2⁻²²⁴.
- **WAL-checkpoint cadence** — 60-second Tokio interval task issuing `PRAGMA wal_checkpoint(TRUNCATE)` to bound WAL file growth under sustained ingest; checkpoint errors swallowed (next tick retries; SQLite's default 1000-page auto-truncate is safety net).
- **ETH archive log ingestion** — chunked `eth_getLogs` against Alchemy (`AlchemyArchiveSource`) or any URL (`with_rpc_url(public_rpc)`); ABI decoders for Swap/Mint/Burn/Collect events verifying `topic[0]` keccak hashes; `parse_int24_word` fast path reading 6 trailing hex chars via `u32::from_str_radix` (closes a 32-byte `Vec<u8>` per-call allocation found in the audit).
- **The Graph hosted-subgraph backfill** — single GraphQL query per `Ingester::backfill` call, no chunking, no auth on legacy hosted URL; gateway path supports `THE_GRAPH_API_KEY` for Uniswap V3 (gateway requires wallet which user does not have — legacy URL is the no-key path).
- **Cross-chain V3 + V3-forks routing** — `ChainId` × `Protocol` enums × per-(chain, protocol) subgraph URL; Sushi V3 and Pancake V3 share Uniswap V3's GraphQL schema, so the forks differ only by URL.
- **Tauri 2 IPC + manual Rust↔TS DTO mirror** — `serde(rename_all = "camelCase")` is the single bridge; 19 commands registered in `lib.rs::run`; standardised `CommandError { message, key_required }` error envelope.
- **React 18 StrictMode discipline** — gate **state setters** with the `mounted` flag, never the pipeline body; idempotency at storage makes the second mount a cache hit. The closed bug commit `43599ba` (an `initialised` ref short-circuit) left `busy=true` set forever on the second mount, hanging the page on "Pipeline running…".
- **`usePersistedState` shape-merge** — defends against persisted-shape drift when new fields are added to a persisted object; primitives keep their explicit values (12s arbitrage refresh default still respects existing `1000` in localStorage); objects pick up new fields' defaults on next load (LP settings get the new `chainId` field even on stale persistence).
- **Cross-cutting telemetry** — dotted hierarchical event names (`lp.pipeline.start`, `lp.pipeline.chain-head-fetched`, `settings.toggle`); buffered in-memory + flushed via `telemetry_persist` IPC + on `beforeunload`; persisted to per-session JSON file cleared each boot; designed for `jq` diagnostic queries.
- **Read-only DeFi posture** — Aurix never holds a key, never signs, never submits a transaction; the entire blast radius of any code bug is zero on-chain. Caller can run on a machine that also holds wallet keys without concern.

## Key technical decisions

LifeOS captures four foundational design principles plus several session-level direction calls.

### Foundational principles

- **Local-first** — all computation and storage on-device, no cloud backend, no server, no telemetry-to-third-party. Driven by privacy (DeFi monitoring patterns are commercially sensitive), latency (no network round-trip for analytical layer), zero operational cost, and offline resilience. Constrains history to device storage and forbids cross-machine state sharing without an explicit sync layer.
- **Zero-cost** — free public RPC endpoints only; no paid API keys; no premium data subscriptions. Driven by accessibility (Aurix must be runnable without an Alchemy paid plan), no wallet required to monitor (Caner himself has no wallet), and sustainability. Constrains polling cadence — free-tier RPC limits are sufficient at 12-second cadence but not at sub-block aggressive scanning. `ALCHEMY_API_KEY` is an optional fallback.
- **Read-only** — never submits a transaction, never holds a private key, never requests wallet connection. Driven by safety (cannot drain funds if it cannot write), scope clarity (analytics ≠ execution), and trust (user can run Aurix alongside a wallet without concern). Cannot be extended into an execution tool without explicit principle violation — that would require fork/rebrand, not increment.
- **Tab-scoped independence** — each tab is a self-contained feature module with its own types, backend commands, and analytical logic. Cross-tab state sharing is not permitted until explicitly designed. The 2026-05-03 multi-tab refactor preserves the principle: tabs share `Storage`, telemetry, and `EthereumRpcClient` (transport-only), but no feature-level state.

### Technology choices and rationale

- **Tauri over Electron.** Tauri's ~5-15 MB binary vs Electron's 100-200 MB; native webview (low memory) vs bundled Chromium (high memory); Rust backend (the *point*, not a compromise) vs Node.js; capability-based security vs full-Node-access default. The Rust backend is directly useful for `sqrtPriceX96` 256-bit big-integer arithmetic, which JavaScript's `BigInt` could handle but `num-bigint` makes ergonomic and precision-clear.
- **Plain CSS over a component library.** Dark, dense trading-terminal aesthetic targeted; generic component libraries default to light consumer styling and require significant overriding. No abstraction overhead; no unused component bundle. Trade-off accepted: no design system enforcing consistency across future tabs.
- **Rust backend for all on-chain decoding.** `sqrtPriceX96` requires 256-bit arithmetic; future tabs (risk modelling, LP backtesting) involve heavy numerical computation where Rust's performance is genuinely better. TypeScript becomes a pure display/interaction layer.
- **`num-bigint::BigUint` over `ruint::U256`.** The "no ethers-rs, clean-room port from Solidity" framing reads stronger when the underlying types are general-purpose Rust primitives. `ruint` is faster (fixed-width, stack-allocated) but the port-from-Solidity narrative is less direct.
- **f64-then-refine over Solidity's bit-walking inverse for `sqrt_price_x96_to_tick`.** Solidity's inverse is bit-precise but slow in BigUint arithmetic; f64 estimate gets within 2 ticks; refinement closes the gap via the exact `tick_to_sqrt_price_x96`. Documented in `context/references/v3-mathematics-deep-dive.md` (110KB reference).
- **Per-module `thiserror::Error` enums over a grand-unified Error type.** Each module owns its own enum; adapter errors wrap `EthereumRpcError` with `#[from]` and `#[error(transparent)]`. Modules remain independent units that can evolve their failure model without churning a shared type. Stringification belongs at the Tauri command boundary only.
- **`Inputs / Outputs / Errors / Side effects` four-line docstring contract.** Order matches how a reader reasons about a call site. Private helpers omit the block; a single-line summary is enough.
- **Zero inline `WHY`/`NOTE`/`HACK` annotations.** Repository-wide grep returns zero matches outside `context/` per LifeOS Conventions. Rationale lives in `context/notes/*.md` or `context/systems/*.md` Durable Notes sections. The convention prevents two canonical homes for the same knowledge.
- **`serde(rename_all = "camelCase")` manual mirror, no codegen.** Bridge is one Serde attribute on the Rust side. TypeScript interface uses camelCase names directly; no runtime transformation on the frontend. Currently 19 IPC commands and ~217 lines of TS mirrors — `ts-rs` migration target if drift becomes recurring or surface grows past ~30 commands / 2-level nesting.
- **TEXT decimal big-integer encoding over BLOB.** SQLite INTEGER is 64-bit signed; `sqrtPriceX96` is uint160, `liquidity` uint128, `amount0/amount1` int256. INTEGER loses precision above 2⁶³. TEXT decimal preserves range at the cost of per-read `BigUint::parse_bytes` (hoisted out of the per-loop body by the 2026-05-04 audit's `ParsedSwap` pre-parse).
- **`INSERT OR IGNORE` on domain-natural composite keys** vs synthetic auto-increment. Domain-natural keys make the idempotency contract work: re-running with the same semantic inputs hits the cache because the inputs determine the key. `config_hash` must hash only inputs that semantically determine the output — including `chrono::Utc::now()` or any wall-clock value defeats the cache.
- **`benchmark_series` uses `INSERT OR REPLACE`** as the sole exception. External providers (FRED) revise older series prints; replace-on-collision keeps the local cache aligned with provider truth.
- **`SYNTHETIC_TX_HASH` sentinel for synthetic-vs-live separation** vs separate tables. Simplifies schema to one `swap_events` table at the cost of needing `delete_synthetic_swaps_in_range` purge step before re-ingesting tweaked synthetic data.
- **Three-tier free-data fallback over Alchemy-only.** Earlier design used Alchemy-only, requiring users to bring an API key. Subgraph-first → user-Alchemy → public-RPC chain closes the no-API-key UX gap that blocks dashboard usability for users without a wallet (Caner's case).
- **No synthetic data in user-facing flows.** Synthetic ingest stays as IPC for tests/dev but auto-run pipeline does not call it. Caner's rationale at landing (verbatim from Decisions): *"imagine if an engineer from a company we applied to wanted to test it out and saw completely made up numbers, he'd think we fucked up somewhere. better off displaying no data and having me fetch api keys than calculating things w made up data."*
- **Tab 2 Vector A over Tab 2 Timeboost MEV analytics (2026-05-02).** A May 2026 status-decision session chose `revive` with **Vector A (V3 LP Backtester)** as immediate work. The competing Timeboost MEV direction (Arbitrum sequencer auction analytics) was preserved in `Aurix/Work/Tab 2 Timeboost MEV Analytics.md` as a historical alternative. Rationale: Tab 1 alone was *"barely a signal other than a pretty frontend"*; Vector A closes the resume credibility gap that the "Uniswap V3 LP backtesting" bullet promised; the M2.7 multi-asset benchmarks + M2.8 regime-conditional capital-allocation headline reframes the project from "another V3 backtester" to "investment-grade analysis framework"; 11 research papers (~17,000 lines, ~770KB) commissioned the day before the sprint provided the implementation foundation.
- **Solo-contributor master-only workflow.** Commits directly to master; no feature branches even for multi-week features. Commit messages and commit boundaries become the only review surface. The 2026-05-03 sprint shipped 21 commits across M2.0 → M2.8 + 4-tier extension all directly to master; the 2026-05-04 audit cycle shipped 4 more.
- **Stooq → FRED migration for S&P 500 benchmark.** Stooq silently added an API-key requirement; FRED `SP500.txt` is no-key and provides additional macro series (T-bill, gold) Aurix needs anyway. `stooq_voo` legacy series-key preserved but transparently routes to FRED SP500.
- **DefiLlama over individual lender APIs.** Aave / Compound / Lido each have their own APIs; DefiLlama aggregates them with one canonical normalisation and consistent decimal-rate semantics across providers.
- **`lookbackBlocks` over manual `fromBlock`/`toBlock` inputs (commit `53f99eb`).** Caner verbatim: *"I don't like the 'from' and 'to' block window, u should just do 'last 1k blocks' or smth that auto updates w every new block instead."* Chain head fetched as the second IPC call so window is always anchored to the **finalized** block.
- **Telemetry recorder over screenshot-based diagnostic loop.** Caner verbatim: *"every screenshot is so much heavier than a couple more lines in a json file."* Dotted event names + `jq`-friendly JSON file + per-session reset.
- **Default arbitrage refresh 1s → 12s (commit `de2a4c7`).** `slot0()` reads only change per block; sub-block polling repeated identical data 11 of every 12 ticks. Sub-block options (1s/2s/5s/10s) retained for users who want UX-feedback faster than block cadence; existing localStorage `1000` values keep their explicit choice (shape-merge applies to objects, not primitives).
- **Three audit findings deferred (2026-05-04).** `commands/lp.rs` folder split (628 lines, fan-out 16); `backtest/engine.rs` setup → step → summarise split; `LpBacktestPage.tsx` → `useLpPipeline` hook + Vitest setup. All three are HIGH-severity per the audit but explicitly *zero-functional-change* mechanical refactors — deferred while user was AFK because risk-vs-reward unfavourable without run-test.

## What is currently built

Vector A is **code-complete and audit-improved**, with live-data verification blocked. The honest current-state picture from LifeOS Overview's Tab Reality table:

| Tab | Feature | Status |
|-----|---------|--------|
| **1 — Cross-DEX arbitrage** | Multi-pair refactor done; pool-fee P/L done; card-grid dashboard done; 12s default; milestones 1.1-1.4 substantially complete | **Working** |
| **2 — Uniswap V3 LP backtesting** | Full Vector A stack (M2.0 storage → M2.8 capital-allocation headline) + 4-tier extension (cross-chain + V3 forks + non-USD pools) + 2026-05-04 audit perf wins | **Code-complete + audited; live-data verification pending Alchemy key restoration** |
| **3 — On-chain wallet tracker** | No wallet-address input, no position decoder | **Not started** |
| **4 — Gas price intelligence** | Gas read live but not persisted historically | **Not started** |
| **5 — Quantitative risk modelling** | No correlation / volatility / VaR code | **Not started** |

Activity signal per LifeOS Overview as of last verification (2026-05-13, HEAD `9085a82`):

- 40 total commits over the period (2026-03-04 → 2026-05-04).
- 295 total files in the repository.
- ~10,500 backend Rust LoC across 79 files.
- ~9,000 frontend TypeScript LoC across 45 .tsx + 18 .ts files.
- 139 backend tests pass / 0 fail / 3 ignored (live-Alchemy gated).
- 0 frontend tests (Vitest setup deferred from the 2026-05-04 audit).
- `context/` is 1.16 MB (54 files): `architecture.md`, 14 per-subsystem `systems/*.md`, 11 convention notes, plan files (vector-a + vector-b + vector-c + code-health-audit/), 11 research papers in `references/` (~17,000 lines, ~770 KB).
- `learning/` is 56 files across 7 top-level folders: `concepts/{foundations,core,domain-patterns,advanced}/`, `project/{architecture,decisions,evolution,systems,comparisons}/`, `exercises/{foundations,core,project,solutions}/`, `materials/`, `paths/{foundations,domain-theory,project-systems,vector-prep,interview-fluency}/`, `references/`.

The 2026-05-03 sprint moved the project from a `8 commits, 67 files, 0 tests` baseline (2026-04-22) to its current state across the M2.0 → M2.8 + 4-tier extension arc plus the 2026-05-04 audit + implementation cycle.

## Current state

Status `active` per LifeOS Overview frontmatter. Last meaningful commit 2026-05-04 (4 commits that day, all functional); HEAD `9085a82` unchanged at 2026-05-13 re-verification (no new commits during the 9-day window). The 2026-05-13 re-verification pass refreshed frontmatter on five files (Overview, Decisions, 3 Roadmap design-space notes, 2 Work files) without changing content; 24 of 29 files re-verified `likely_current` and 5 had frontmatter-only gaps.

In-flight items from LifeOS Work folder: README demo recording is still pending (no `media/` folder exists yet; Vector A LP backtester now the strongest demo candidate per the Work file's 2026-05-13 status note); the live-data path restoration is the binding next-session item.

## Gaps and known limitations

LifeOS Gaps maintains an authoritative gap inventory split into resolved, partially resolved, still open, and newly surfaced. Career-relevant honest gaps:

- **Gap A — Live-data path non-functional (CRITICAL).** `commands/lp.rs::run_lp_ingestion` fails on every tier: Tier 1 legacy hosted Uniswap subgraph URL `api.thegraph.com/subgraphs/name/uniswap/uniswap-v3` is deprecated (transport error); Tier 2 `.env` `ALCHEMY_API_KEY` is `QNZ1oqj_e9R6izhNcz_9X` (~21 chars vs typical 32+ — likely truncated, returns 429/400); Tier 3 public RPC returned "All RPCs are unreachable" in the last session. Tab 1 *appears* to work because `EthereumRpcClient` silently falls through to LlamaRPC — this is a documented diagnostic trap (`Tab 1 working ≠ Alchemy works`). User's Alchemy account login is also broken (verbatim: *"my alchemy login isn't working ... reset email keeps saying theres an issue"*), so the key can't be re-paired until login is restored. Blocks live verification of the entire 4-tier extension and the real-fixture validation.
- **Gap B — Three deferred audit findings (HIGH, modularisation hygiene).** `commands/lp.rs` (628 lines, fan-out 16) folder split; `backtest/engine.rs` setup → step → summarise split (488 lines post-audit); `LpBacktestPage.tsx` (668 lines mixing orchestration + JSX) → `useLpPipeline` hook + Vitest setup. All three are HIGH-severity per the audit but explicitly zero-functional-change mechanical refactors — deferred while user was AFK. The audit folder at `context/plans/code-health-audit/` is preserved (the 3 deferred findings keep it active); 11 of 14 actionable checkboxes ticked.
- **Gap C — Real-fixture validation pending (HIGH).** `validation/` has only synthetic round-trip coverage. Real-fixture infrastructure (subgraph query for known mainnet LP positions + burn-tx receipt parsing → ground-truth fees) is not started. Highest-impact validation gap; requires both live RPC access (blocked by Gap A) AND a curated set of mainnet LP positions.
- **Gap D — Deflated Sharpe correction missing (HIGH, statistical).** The strategies grid picks the best Sharpe out of 81 cells; selection bias inflates the expected value of "best LP wins" in the headline verdict. Bailey/de Prado's deflated-Sharpe form is in `references/backtest-statistical-methodology.md` but not in code. The verdict's "best-cell beat all benchmarks" claim is statistically optimistic.
- **Gap 2 — Tab 1 fail-fast error model (HIGH).** Tab 1's `fetch_market_overview` uses `.map_err` on every individual future, so any one venue's error rejects the whole tick. Tab 2 closes this structurally via the 3-tier ingest fallback. Tab 1 still affected; opportunistic per the 2026-05-02 direction decision.
- **Gap 6 — Frontend test coverage (HIGH).** Zero frontend tests today; Vitest setup deferred at the 2026-05-04 audit. Backend resolved (139 tests). Manual Rust↔TS DTO sync drift is the primary risk this would mitigate.
- **Gap 4 — Duplicated Tab 1 TypeScript primitives (MEDIUM, has drifted).** `formatUsd()` is implemented in 4 places with `insights.ts` already using `signDisplay: "exceptZero"` while the other three use default — cosmetic drift exists today; `median()`, `GAS_UNITS_ESTIMATE = 220_000`, and the gas-adjusted formula are duplicated in 3 places each but identical so far. Tab 1 polish opportunistic.
- **Gap 7 — Fixed 220k gas estimate (MEDIUM).** `GAS_UNITS_ESTIMATE = 220_000` undocumented; actual Uniswap V3 swap gas varies ~130k-300k depending on route, tick crossings, and approvals. Gates the `actionable` severity classification.
- **Gap 11 — No IPC contract check (MEDIUM).** Rust DTOs and TypeScript mirrors are manually kept in sync with no automated validation; the only protective convention is `serde(rename_all = "camelCase")` plus discipline. The commit `391eadd` IPC mismatch bug (`PositionConfig` deserialising snake_case while TS sent camelCase) is exactly this gap manifesting. Audit recommends Vitest contract pin or `ts-rs`/`specta` codegen migration.
- **Gap 5 (partial) — `f64` precision at IPC boundary (LOW now).** Math layer is fully fixed-point on `BigUint` Q64.96. `EquityCurvePoint` USD fields cross to TypeScript as f64; ULP drift over very long backtests bounded ~`1e-9` USD per the audit's analysis of the `fees_usd_acc` change. f64 LVR precision at extreme sqrtPrice ranges (rare on real ETH/USDC) is audit `potential-issues §4`.
- **Gap 8 — Stale "three venues" copy (TRIVIAL).** Three sites still say "three venues" although the backend returns four; plus Tauri+Vite starter scaffolding residue (`index.html` title `Tauri + React + Typescript`, `productName: "aurix"` lower-case, `description = "A Tauri App"`).
- **Gap 10 — Per-adapter timestamps (LOW).** Each Tab 1 `PriceSnapshot.fetched_at_unix_ms` set inside its adapter via `SystemTime::now()`; the four adapters run concurrently, so timestamps differ by tens of ms within the same tick. `MarketOverview.fetched_at_unix_ms` is copied from V3 5bps's snapshot.

## Direction (in-flight, not wishlist)

Active next-session items per LifeOS Vector A Sprint carry-forward + Work files:

- **Restore Alchemy account access; replace truncated `.env` key.** Then verify via telemetry that `lp.pipeline.ingest` event shows `report.sourceLabel = "alchemy:ethereum"` (or `"subgraph:Ethereum"`) and a non-zero `swapRowsPersisted`.
- **(Optional) sign up for The Graph Studio API key** to open Tier 1 directly. Email signup; wallet may or may not be required for gateway free-tier (Caner has no wallet, so this is uncertain).
- **Verify each of the four tiers from the 2026-05-03 sprint** on real data: cross-chain (Arbitrum / Optimism / Base / Polygon), V3 forks (Sushi V3 / Pancake V3 URLs which are educated guesses based on public hosted-service conventions), non-USD-quote pools (WBTC/ETH, LDO/ETH).
- **README demo recording is pending.** No `media/` folder exists yet. Demo target recommendation per the LifeOS Work file: record both Tab 1 arbitrage live feed AND Tab 2 LP backtester pipeline + headline verdict once live-data is restored — Tab 2 carries the heaviest hiring signal. NeuroDrive-style hero gif (~720 px width, ~10 s loop, `media/` folder at repo root, centred via `<p align="center">` at the very top of the README).
- **Three deferred audit findings** (lp.rs folder split, engine.rs setup→step→summarise split, LpBacktestPage → useLpPipeline + Vitest setup) — zero-functional-change refactors waiting for a focused session that can run-test the frontend.

Three vector plans exist in `context/plans/` as the menu of next directions per LifeOS Overview: **Vector A (V3 LP Backtester)** is shipped + audited (live-verification pending), **Vector B (Mempool MEV Detector)** is proposed (3-5 weeks, independent), **Vector C (ML Arbitrage-Survival Classifier)** is proposed (4-8 weeks, shares Vector A's storage layer — rare DeFi+ML cross-section). Recommended sequence if multiple are pursued is A → C (shares M2.0) → B (independent). Appetite for Vectors B and C after A's live-data verification is a separate decision.

Tabs 3-5 design-space notes (Wallet Tracker, Gas Intelligence, Risk Modelling) live in `Roadmap/`. They are preserved as design space because Vector A consumed the available build budget; persistence (Gap 1, formerly the primary blocker for all three) is now resolved via the M2.0 SQLite layer, so when these tabs are built, the storage layer + math primitives + 3-tier ingest pattern transfer cleanly.

## Demonstrated skills

What this project specifically proves Caner can do, drawn evidence-anchored from LifeOS:

- **Ship a complete production-grade subsystem stack in one focused two-day arc** — the 2026-05-03 sprint shipped 21 commits across M2.0 → M2.8 plus a 4-tier extension covering storage, math, ingest, backtest engine, validation, strategies, benchmarks, and headline. The 2026-05-04 audit + implementation cycle shipped 4 more commits including a code-health-audit driven implementation of 11 of 14 findings with explicit reasoned deferrals for the remaining 3.
- **Implement non-trivial protocol mathematics from primary sources, not SDKs.** Clean-room port of Uniswap V3's TickMath / FullMath / LiquidityAmounts / SqrtPriceMath from Solidity to Rust on `num-bigint::BigUint`. No `ethers-rs`, no third-party V3 SDK, no transitive crypto dependency. 30+ unit tests including round-trip pins (`tick → sqrtPrice → tick` recovers original tick) and bit-exact matches against Solidity reference values for the 20 magic constants.
- **Build a robust persistence layer with explicit invariants and idempotency.** SQLite + WAL + writer-thread + r2d2 reader pool + refinery forward-only migrations + 12 domain CRUD submodules. Four enforced conventions across every per-table CRUD: domain-natural composite keys + lowercase address normalisation + synthetic sentinel separation + TEXT decimal big-integer encoding. 18 round-trip + idempotency tests pinning the contract.
- **Make trade-off-driven engineering choices and document them.** Tauri over Electron with specific cited reasoning; plain CSS over component libraries with the design system trade-off acknowledged; `num-bigint::BigUint` over `ruint::U256` with the narrative-vs-speed trade-off named; per-module `thiserror` enums over grand-unified types with the independence reasoning; "zero inline rationale comments — rationale lives in `context/` docs" as a project-wide convention enforced and grep-verified.
- **Apply a code-health audit cycle end-to-end.** Two-pass audit producing 19 findings across 7 finding-bearing files; 11 implemented in the same session as the audit (math precompute, ingest int24 fast path, backtest `ParsedSwap` hoist + `HoldOnlyEvaluator` hoist + incremental `fees_usd_acc`, `lp.rs` `Lazy<reqwest::Client>` + structured `IngestionReport`, `lib.rs` WAL checkpoint task, frontend `console.log → telemetry.record` sweep, `usePersistedState` shape-merge crash fix, documentation rot via parallel upkeep-context skill); 3 explicitly deferred with documented rationale.
- **Design and ship a tiered free-data fallback chain for a domain where free data is fragile.** Three-tier `Subgraph → Alchemy → public RPC → empty state (no synthetic fallback)`. Each tier records `AttemptedSource`; `IngestionReport.source_label` reports which tier succeeded; `KeyRequired(name)` error variant lets the frontend prompt structurally rather than parsing error messages. The chain accommodates real provider drift (Stooq adding key requirement → swap to FRED `SP500.txt`; hosted Uniswap subgraph deprecation; user has no wallet so gateway path is unavailable).
- **Apply a research corpus to implementation, then trace the application back.** 11 research papers (~17,000 lines, ~770 KB) commissioned in `context/references/` the day before the sprint: `v3-mathematics-deep-dive.md` (110 KB) → Math system; `backtest-statistical-methodology.md` (87 KB) → Strategies + Headline; `tradfi-benchmark-data-sources.md` (77 KB) → Benchmarks tradfi providers; `sqlite-rust-production-patterns.md` (76 KB) → Storage topology; `defi-yield-data-sources.md` (69 KB) → Benchmarks DeFi providers; `v3-position-validation-methodology.md` (69 KB) → Validation real-fixture protocol; `lp-rebalancing-strategies.md` (65 KB) → Backtest Engine rebalance rules; `oss-v3-backtester-landscape.md` (56 KB) → competitive positioning; `ethereum-archive-log-ingestion.md` (53 KB) → Ingest; `v3-lp-profitability-literature.md` (51 KB) → Math IL closed forms (Milionis-Moallemi-Roughgarden LVR); `out-of-scope-risks-survey.md` (58 KB) → what is deliberately not modelled.
- **Build cross-runtime IPC at scale with manual discipline.** 19 Tauri IPC handlers; Tab 2 type mirror alone is 217 lines of `types.ts` covering 20+ Rust DTOs; `serde(rename_all = "camelCase")` as the only wire-level bridge; `CommandError { message, key_required }` standardised envelope; the audit's `cross-cutting.md` finding (`console.log + eslint-disable` → `telemetry.record`) and the IPC mismatch bug (`PositionConfig` snake_case vs camelCase) demonstrate both the conscious design and the cost of the convention.
- **Diagnose subtle React + IPC bugs from telemetry traces, not screenshots.** The 2026-05-04 audit cycle was driven by exactly this pattern. Telemetry log showed user clicked Settings on the LP tab at session-relative `t=12838ms`; `LpSettingsForm` rendered with stale persisted shape; `CHAIN_CONFIGS[undefined].label` threw a `TypeError`. Fixed structurally in `usePersistedState` shape-merge so any future field addition picks up its default on next load. Diagnosed without screenshots; validating moment for the 2026-05-03 telemetry-over-screenshots design choice.
- **React 18 StrictMode discipline in a complex auto-run pipeline.** Closed bug `43599ba` left `busy=true` set forever on the second mount because an `initialised` ref short-circuited the pipeline body. Fix gates state setters with the `mounted` flag while letting both StrictMode-mount invocations run the pipeline body — the second mount becomes a cache hit because storage keys runs by `config_hash`. The contract spans Engine + Grid + Headline + Ingester layers.
- **Apply quantitative finance primitives correctly.** Per-swap fee distribution with in-range gating; LVR discrete approximation per Milionis-Moallemi-Roughgarden; V2 + V3-concentrated IL closed forms; `il_usd = raw_position_value - hold_only_usd` corrected from an earlier fee-included form; adaptive-tercile vol regime classifier (self-calibrating per asset, replacing fixed 0.5%/2.0% thresholds that broke on DAI/USDC and WBTC/ETH); three-variant LP comparison (best / naive / median) reframing the verdict question from "did the perfectly-tuned LP win?" to "would a typical LP have won?"
- **Honest scope discipline.** The repository's status is `active` but Tabs 3-5 are documented as `not started`; the LP backtester is `code-complete + audited` but `live-data verification pending`; the three audit findings are deferred with explicit deferral rationale; the deflated-Sharpe gap is named openly. The Vector A Sprint note frames the project to a quant-LP-desk audience without overclaiming.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Aurix/_Overview.md | 212 | "#aurix #rust #defi #tauri #vector-a #project-active" |
| Projects/Aurix/Architecture.md | 388 | "#aurix #rust #tauri #architecture #vector-a" |
| Projects/Aurix/Conventions.md | 284 | "#aurix #conventions #rust #typescript #free-data #idempotency #rationale-capture" |
| Projects/Aurix/Decisions.md | 213 | "#aurix #decisions #audit #vector-a #performance #2026-05-04" |
| Projects/Aurix/Gaps.md | 296 | "#aurix #technical-debt #gaps #defi #vector-a" |
| Projects/Aurix/Vector A Sprint.md | 270 | "#aurix #vector-a #sprint #lp-backtester #2026-05-03 #2026-05-04 #audit #evolution" |
| Projects/Aurix/Systems/_Overview.md | 90 | "#aurix #systems #overview #vector-a" |
| Projects/Aurix/Systems/Storage.md | 226 | "#aurix #rust #sqlite #persistence #storage #vector-a #m2-0" |
| Projects/Aurix/Systems/Math.md | 167 | "#aurix #rust #uniswap #v3 #tick-math #q64-96 #vector-a #m2-2" |
| Projects/Aurix/Systems/Ingest.md | 175 | "#aurix #rust #ingest #eth_getlogs #subgraph #vector-a #m2-1" |
| Projects/Aurix/Systems/Backtest Engine.md | 199 | "#aurix #rust #backtest #lvr #vector-a #m2-3" |
| Projects/Aurix/Systems/Strategies.md | 116 | "#aurix #rust #strategies #grid-search #vector-a #m2-5" |
| Projects/Aurix/Systems/Benchmarks.md | 159 | "#aurix #rust #benchmarks #defillama #fred #alpha #vector-a #m2-7" |
| Projects/Aurix/Systems/Headline.md | 148 | "#aurix #rust #headline #capital-allocation #regime-classifier #vector-a #m2-8" |
| Projects/Aurix/Systems/Validation.md | 119 | "#aurix #rust #validation #vector-a #m2-4" |
| Projects/Aurix/Systems/Runtime Foundation.md | 213 | "#aurix #rust #tauri #runtime #shell #multi-tab" |
| Projects/Aurix/Systems/Telemetry.md | 145 | "#aurix #telemetry #diagnostics #cross-cutting" |
| Projects/Aurix/Systems/Cross Runtime Contract.md | 270 | "#aurix #rust #typescript #ipc #systems" |
| Projects/Aurix/Systems/DEX Adapters.md | 339 | "#aurix #defi #rust #uniswap #systems" |
| Projects/Aurix/Systems/Data Pipeline.md | 261 | "#aurix #architecture #systems #rust" |
| Projects/Aurix/Systems/Analytics Engine.md | 284 | "#aurix #typescript #analytics #systems #defi" |
| Projects/Aurix/Systems/GUI Layout.md | 332 | "#aurix #typescript #react #frontend #systems" |
| Projects/Aurix/Systems/LP Backtest GUI.md | 175 | "#aurix #react #typescript #lp-backtest #vector-a #frontend" |
| Projects/Aurix/Roadmap/Gas Intelligence.md | 174 | "#aurix #defi #roadmap #gas-intelligence" |
| Projects/Aurix/Roadmap/LP Backtesting.md | 167 | "#aurix #defi #uniswap #roadmap #lp-backtesting" |
| Projects/Aurix/Roadmap/Risk Modelling.md | 222 | "#aurix #defi #roadmap #risk-modelling #quant" |
| Projects/Aurix/Roadmap/Wallet Tracker.md | 164 | "#aurix #defi #roadmap #wallet-tracker" |
| Projects/Aurix/Work/README Demo.md | 58 | "- Cernio demo commit: `Capataina/Cernio` `4a93239`" |
| Projects/Aurix/Work/Tab 2 Timeboost MEV Analytics.md | 91 | "#aurix #work #defi #timeboost #mev #sequencer" |
