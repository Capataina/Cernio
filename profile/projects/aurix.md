---
name: Aurix
status: active
source_repo: https://github.com/Capataina/Aurix
lifeos_folder: Projects/Aurix
last_synced: 2026-05-31
sources_read: 29
---

# Aurix

## One-line summary

Local-first, zero-cost, read-only DeFi analytics desktop application (Tauri 2 + React 19 + Rust) built around a Uniswap V3 LP backtester with clean-room Q64.96 tick mathematics, multi-asset benchmark alpha decomposition, and a regime-conditional capital-allocation verdict.

## What it is

Aurix is a Tauri 2 desktop application pairing a React 19 + TypeScript 5.8 frontend with a Rust (edition 2021) backend to monitor and analyse decentralised exchange markets entirely on-device — no cloud backend, no wallet, no transaction capability. The core premise: serious DeFi analytics tooling either costs money (premium APIs / subscriptions) or requires trusting a third party with the user's activity; Aurix keeps computation local, sources data from free public endpoints (on-chain RPC, hosted subgraphs, DefiLlama, FRED), and is deliberately read-only so it can never submit a transaction. The project is structured as a multi-tab shell — Tab 1 (cross-DEX arbitrage detection) is working, Tab 2 (Uniswap V3 LP backtester, "Vector A") is code-complete and audit-improved, and Tabs 3-5 (wallet tracker, gas intelligence, risk modelling) remain README intent with design-space notes. LifeOS positions Aurix explicitly as the "crypto-domain hiring project" in Caner's portfolio, framed against quant-LP-desk and DeFi-aware trading-firm audiences, with a depth-on-one-vector-over-breadth-across-shallow-tabs strategy (Vector A is the deep vector). The application is `active` per LifeOS frontmatter; HEAD commit `9085a82` (2026-05-04), 40 total commits, 295 files.

## Architecture

Two strictly separated runtimes communicating exclusively via Tauri's IPC mechanism — no shared memory, no shared file system, no WebSocket between them.

```
┌──────────────────────────────────────────────────────────────────┐
│  FRONTEND RUNTIME  (Chromium WebView, src/)                      │
│  App.tsx (multi-tab routing) ── TopBar ── route-aware Settings   │
│    ├── ArbitragePage (Tab 1)     ◄── 12s default tick            │
│    └── LpBacktestPage (Tab 2)    ◄── auto-run on settingsKey     │
│  src/lib/telemetry.ts (every IPC + click + change persisted)     │
└────────────────────────┬─────────────────────────────────────────┘
                         │  Tauri IPC (JSON, camelCase)
                         ▼
┌──────────────────────────────────────────────────────────────────┐
│  BACKEND RUNTIME  (Native Rust process, src-tauri/)              │
│  lib.rs::run                                                     │
│    ├── tokio::Builder::new_multi_thread                          │
│    ├── Storage::open(~/.aurix/aurix.sqlite) (writer + reader pool)│
│    ├── spawn_wal_checkpoint_task(60s interval)                   │
│    └── tauri::Builder::default()                                 │
│          ├── .manage(Arc<Storage>)                               │
│          └── .invoke_handler! [19 commands]                      │
└──────────────────────────────────────────────────────────────────┘
```

Repository shape (verified via LifeOS scan_repo.py): `src/` (75 frontend files), `src-tauri/` (98 backend files / ~821KB), `context/` (1.16MB project memory: 14 system docs, 11 convention notes, 11 reference papers ~770KB, code-health-audit plan folder), and a 56-file `learning/` curriculum.

Backend top-level Rust modules:

- `config/` — `AppConfig` + `chains.rs` (5 chains × 3 protocols).
- `ethereum/client.rs` — read-only JSON-RPC transport.
- `storage/` — 12 submodules + V001 + V002 refinery migrations.
- `math/` — q96 + tick + liquidity + fees + il (clean-room V3 port).
- `ingest/` — `ArchiveSource` trait + Alchemy + Subgraph + Mock + decoder.
- `backtest/` — engine + price + metrics + gas + position + rebalance.
- `validation/` — synthetic-first harness (real-fixture pending).
- `strategies/` — `GridRunner` cartesian search.
- `benchmarks/` — DefiLlama + FRED + beaconcha.in + V2-LP + HODL + alpha.
- `headline/` — adaptive-tercile classifier + verdict prose synthesis.
- `commands/` — `market.rs` (3 Tab-1 commands), `lp.rs` (14 Tab-2 commands, 628 lines), `telemetry.rs` (2 commands).

Dependency direction is strictly one-way: `ingest → storage`; `storage + math → backtest`; `backtest → strategies + headline`; `benchmarks → storage → headline`; `commands/*.rs` orchestrates against the modules above; `lib.rs` wires storage + the WAL checkpoint task into Tauri's managed-state.

The 2026-05-03 sprint generalised the architecture from "single-feature single-page" to "multi-feature multi-tab + shared storage + cross-cutting telemetry." Tabs share `Storage`, the telemetry recorder, and `EthereumRpcClient` (transport-only) but no feature-level state, per the explicit Tab-Scoped Independence principle.

## Subsystems and components

### Storage (M2.0)

SQLite persistence layer underpinning every Vector A subsystem. Topology: `tauri::Builder.manage(Arc<Storage>)` → one `tokio_rusqlite::Connection` writer thread + an `r2d2_sqlite::Pool` reader pool (~4 connections) → `aurix.sqlite` (WAL, `synchronous=NORMAL`, `foreign_keys=ON`, `temp_store=MEMORY`). 12 domain submodules (`connection`, `migrations`, `error`, `snapshots`, `swaps`, `pool_events`, `gas`, `runs`, `strategy`, `benchmarks`, `headline`, `state`). Idempotency contract: every batch insert uses `INSERT OR IGNORE` keyed on domain-natural composites — `(pool, block_number, log_index)` for events, `config_hash` for runs, `(grid_id, cell_index)` for strategies — except `benchmark_series` which uses `INSERT OR REPLACE` because external providers (FRED) revise older series prints. Address-case normalisation enforced at insert/query (lowercased; closed an earlier silent-empty bug commit `c1e5594`). Synthetic-vs-live separation via a sentinel `SYNTHETIC_TX_HASH = "0x...deadbeef"`. Big integers stored as TEXT decimal (sqrtPriceX96 is uint160, liquidity is uint128, amounts are int256 — INTEGER would lose precision above 2⁶³). Migrations forward-only via embedded refinery (`V001__initial.sql`, `V002__multi_asset_headline.sql`). WAL checkpoint task (`PRAGMA wal_checkpoint(TRUNCATE)`) tokio-spawned at 60s cadence (added 2026-05-04, commit `b2e6863`). 18 unit/integration tests pin the idempotency + round-trip contract; the storage error type has fan-in 12 across the backend.

### Math (M2.2)

Clean-room port of Uniswap V3's mathematics stack — no `ethers-rs`, no third-party V3 SDK, no transitive crypto dependency. Built on `num-bigint::BigUint` rather than fixed-width `U256`. Layout: `q96.rs` (FullMath.sol port), `tick.rs` (TickMath.sol port, 290 lines), `liquidity.rs` (LiquidityAmounts.sol + SqrtPriceMath.sol port, 322 lines), `fees.rs`, `il.rs` (V2 + V3-concentrated impermanent-loss closed forms). All sqrt-price arithmetic in `BigUint` Q64.96. `tick_to_sqrt_price_x96` is a bit-by-bit port using the 20 magic constants from `TickMath.sol` (Q128.128 representations of `1.0001^(2^k)`), now precomputed as `Lazy<[BigUint; 20]>` (audit commit `b2e6863` — pre-fix every tick decode parsed up to 20 hex literals via `BigUint::parse_bytes`). Inverse `sqrt_price_x96_to_tick` uses f64 log-estimate + ±2-tick refinement against exact forward — explicitly not Solidity's bit-walking inverse, faster in BigUint arithmetic. `MIN_TICK / MAX_TICK = ±887_272`. Per-direction rounding policy mirrored from `SqrtPriceMath.sol` for the per-step `amount0_delta` / `amount1_delta` helpers. `fee_share_token0/1` computes in-range fee share with defensive `position_L ≤ active_L` clamp added commit `391eadd` (synthetic data could violate this; real V3 mainnet pools enforce structurally). 30+ unit tests — the most heavily-tested subsystem in the repo.

### Ingest (M2.1)

Translates Ethereum archive data into storage rows. Highest composite hotspot score in the repo (0.95). Three-tiered free-data fallback chain that lets Aurix run without paid API keys: Tier 1 `UniswapV3SubgraphSource` (free hosted GraphQL, optional `THE_GRAPH_API_KEY`) → Tier 2 `AlchemyArchiveSource::from_environment()` (mainnet only, when configured) → Tier 3 `AlchemyArchiveSource::with_rpc_url(chain.public_rpc_url())` (free public RPC — LlamaRPC for ETH, official RPCs for L2s) → empty state with explicit error (never fabricated). Polymorphic `ArchiveSource` trait (`fetch_logs`, `latest_finalized_block`, `block_gas_price_gwei`, `pool_metadata`) lets `Ingester::backfill` consume any source identically. `IngestionReport` returned through IPC carries `source_label` (which tier ultimately succeeded) and `attempted_sources` (any tiers that errored — replaces an earlier `eprintln!`-driven fallback log per audit). `decoder.rs` (515 lines, 10+ tests) parses 32-byte hex words per ABI conventions — `parse_uint256`/`parse_int256`/`parse_uint160_word`/`parse_uint128_word`/`parse_int24_word`. The int24 decoder was hoisted from a 32-byte `Vec<u8>` allocation per word to direct `u32::from_str_radix` on the trailing 6 hex chars (audit commit `b2e6863`). `MockArchiveSource` generates deterministic sinusoidal swap walks for tests/dev, anchored at tick `-195_580` (≈3000 USDC/WETH). 10+ decoder tests + 6 pipeline tests + 3 `#[ignore]`d live-Alchemy integration tests.

### Backtest Engine (M2.3)

Per-swap simulation engine — the load-bearing computational core of Vector A. Composite hotspot 0.93 (second highest in repo). `Engine::simulate(config, rule)` walks every swap in `[entry_block, exit_block]` for a single pool and replays its effect on a configured LP position: per-swap fee accrual when in-range, impermanent-loss accumulation vs hold-only baseline, LVR (Loss-Versus-Rebalancing per Milionis-Moallemi-Roughgarden) discrete approximation `0.5 × Δsqrt² × L / (sqrt × Q96)`, management-gas costs at chain-historical block prices, optional MEV haircut, and a per-sample equity curve. `RebalanceRule` enum: `Static` / `Schedule { every_n_blocks }` / `OutOfRange { trigger_after_blocks }`. IL semantic corrected commit `391eadd` — `il_usd = raw_position_value - hold_only_usd` (fees reported separately, not mixed in). `config_hash` (deterministic SHA over `(pool_address, tick_lower, tick_upper, deposit_amounts, fee_tier, rebalance_rule_serialised)`) keys the cache so React StrictMode's intentional double-mount hits the cache on second invocation. Three audit-applied perf wins (commit `b2e6863`): (1) Data-Layout pre-parse `Vec<SwapEventRow>` → `Vec<ParsedSwap>` once (was ~40k allocations per 10k-swap run from `BigUint::parse_bytes` × 3 + `u128::from_str` inside the inner loop); (2) `HoldOnlyEvaluator` hoist for loop-invariant hold-amount multiply-add; (3) incremental `fees_usd_acc: f64` running sum replacing per-iteration `value_usd` re-walk of monotonically-growing BigUint accumulators (was O(N²) in swap count, ULP drift bound ~2e-11 USD acknowledged). 6 in-crate integration tests cover the behavioural envelope.

### Strategies (M2.5)

`GridRunner` performs cartesian search over `range × rule × deposit × period`. 81 cells default (3×3×3×3). Per-cell sequential invocation of `Engine::simulate`; per-cell failure halts that cell only, grid continues. 3 strategies tests. Persists per-cell outputs via `Storage::persist_strategy_results(grid_id, rows)`.

### Benchmarks (M2.7)

Multi-asset reference series for the LP backtester's headline verdict. 9 series: Aave V3 USDC supply APY (DefiLlama), Compound V3 USDC supply APY (DefiLlama), Lido stETH APY (DefiLlama), 3-month T-bill DGS3MO (FRED `.txt`), S&P 500 (FRED `SP500.txt`), Gold LBMA (FRED), ETH.STORE staking yield (beaconcha.in — KEY_REQUIRED), V2 LP constant-product synthetic baseline, HODL price-only baseline. All sources except beaconcha.in are usable without API keys. Stooq was abandoned for VOO when it silently added a key requirement (commit `391eadd`) — legacy `stooq_voo` series_key transparently routes to FRED `SP500.txt`. Layout: `defi.rs` (DefiLlama), `tradfi.rs` (FRED CSV parsing), `beaconchain.rs` (ETH.STORE), `v2lp.rs` (synthetic constant-product baseline), `hodl.rs` (price-only), `alpha.rs` (period alpha + rolling 30/60/90-day alpha vs each benchmark). `ReqwestFetcher` + `MockHttpFetcher` trait split lets per-provider parse logic be tested without live network. 15s HTTP timeout (commit `391eadd` — Aave/Lido fetches were hanging indefinitely). Persistence via `INSERT OR REPLACE on (series_key, date)` — the only Aurix table not using `INSERT OR IGNORE`, because providers (FRED) revise older series prints.

### Headline (M2.8)

The capital-allocation verdict — explicitly framed by LifeOS as the hiring-portfolio centre of gravity. Composes monthly LP sub-backtests with three variants (Best — selection-bias-inflated best-Sharpe-cell from the strategies grid; Naive — fixed wide range ±100 ticks + Static rule + 50/50 deposit split; Median — median-Sharpe cell), classifies each month into a vol regime via an adaptive-tercile classifier (partitions months into low/mid/high terciles by observed volatility, not fixed thresholds — earlier 0.5%/2.0% thresholds were ETH/USDC-calibrated and broke on lower-vol or higher-vol pairs), and counts months where each LP variant beat each of the 6 benchmarks (Aave / Lido / S&P 500 / gold / 3M T-bill / HODL). Synthesises verdict prose like: *"In low-vol months, naive LP beat Aave 7/12 times with a median spread of +1.4% APY."* Schema migration `V002__multi_asset_headline.sql` adds `months_lp_beat_*` aggregates and per-month asset returns. 414 lines across `mod.rs` + `verdict.rs`.

### Validation (M2.4)

Currently synthetic-first only — `Engine::simulate` round-trips against synthetic fixtures. Real-fixture infrastructure (subgraph query for known LP positions + burn-tx receipt parsing → ground-truth fees) is not started; blocked on live RPC access and a curated set of mainnet LP positions. 1 synthetic round-trip test.

### Vector A IPC layer (`commands/lp.rs`)

14 Tauri commands wrapping ingestion / backtest / grid / headline / benchmark fetch + cache; fan-out 16; 628 lines. Surfaces a `CommandError { message, key_required }` shape so the frontend can prompt for an API key when the failure tier is auth-related (`KEY_REQUIRED` propagation). `Lazy<reqwest::Client>` for the token-prices path (audit commit `b2e6863`).

### LP Backtest GUI (Tab 2 frontend)

`src/features/lp-backtest/` + `src/components/blocks/lp/` (10+ LP blocks: Equity, Heatmap, Headline, Multi-Asset, etc.). Strategy controls, equity curve chart, headline verdict block, sortable strategy grid, regime panel, multi-asset compare. Auto-run pipeline keyed off a JSON-stringified settings key — 9 IPC calls per pipeline run. `usePersistedState` shape-merge defends against new persisted fields (closed an `LpSettingsForm` crash diagnosed via the telemetry log, commit `b01b4f4`). `LpBacktestPage.tsx` is 668 lines (modularisation to `useLpPipeline` hook is one of the three deferred audit findings). `MultiAssetCompareBlock` (319 lines) — league table sorts 9 assets by cumulative return with per-asset sparkline + win-rate strip.

### Telemetry (cross-cutting)

`src/lib/telemetry.ts` (397 lines frontend hub) + `src-tauri/src/commands/telemetry.rs` (persistence). Captures IPC start/end/error (with response payloads summarised at 8KB), clicks, change events, lifecycle events, errors, lastState snapshots per page. Persists to `~/Library/Logs/com.ataca.aurix/last-session.json` via Tauri's `app_log_dir()`. Cleared each app boot. Replaces the screenshot-based diagnostic loop (Caner verbatim: *"every screenshot is so much heavier than a couple more lines in a json file."*). 14 prior `console.log + eslint-disable` pairs converted to `telemetry.record(eventName, payload)` per the `lp.pipeline.*` event-name convention.

### Tab 1 (Arbitrage) backend

`commands/market.rs::fetch_market_overview` runs five concurrent futures via `tokio::join!` (V3 5bps, V3 30bps, V2, Sushi, gas) under one `EthereumRpcClient`. Fail-fast error model: any one venue's failure rejects the whole command (documented as Gap 2). Multi-pair refactor (commit `b0862eb`), pool-fee P/L (`4fc8010`), card-grid dashboard (`6ba18f3`), 12s default refresh (`de2a4c7` — one Ethereum block post-Merge; sub-block options 1s/2s/5s/10s retained for users wanting UX feedback faster than block cadence). `dex/uniswap_v2.rs` + `dex/uniswap_v3.rs` venue adapters.

### Cross-Runtime Contract

`serde(rename_all = "camelCase")` on every Rust DTO; hand-kept TypeScript mirrors in `src/features/{arbitrage,lp-backtest}/types.ts` (LP `types.ts` is 217 lines). The IPC mismatch bug commit `391eadd` fixed (where `PositionConfig` was deserialising snake_case while TS sent camelCase, producing `missing field 'pool_address'`) is exactly the no-IPC-contract-check class manifesting. Future codegen migration target named: `ts-rs` or `specta`.

## Technologies and concepts demonstrated

### Languages

- **Rust** (edition 2021) — primary backend language across every Vector A subsystem; ~10,500 LoC across 79 files in `src-tauri/src/`; idiomatic use of `thiserror` for typed error hierarchies (one per subsystem: `StorageError`, `V3MathError`, `IngestError`, `BacktestError`, `BenchmarkError`), `async-trait` for the `ArchiveSource` polymorphism, `once_cell::Lazy` for process-wide constant precompute (magic constants in `tick.rs`, reqwest client hoisting), per-direction rounding policies preserved from Solidity, defensive clamps to handle synthetic-vs-real-data invariant differences.
- **TypeScript** (5.8.3) — ~9,000 LoC across 45 `.tsx` + 18 `.ts` files in `src/`; pure display/interaction layer with no business logic, mirroring Rust DTOs by hand via `serde(rename_all = "camelCase")` contract.
- **SQL** — `V001__initial.sql` + `V002__multi_asset_headline.sql` refinery migrations; forward-only; idempotency keys (`INSERT OR IGNORE` on domain-natural composites; `INSERT OR REPLACE` for `benchmark_series`).

### Frameworks and libraries

- **Tauri 2.x** — desktop shell; binary ~5-15MB vs Electron 100-200MB; capability-based security model.
- **React 19.1.0** + **Vite 7.0.4** — frontend framework + build tool.
- **Tokio 1** (macros, rt-multi-thread, sync, time) — async runtime; multi-thread builder; spawned WAL-checkpoint task; `tokio::join!` Tab-1 fan-out.
- **reqwest 0.12** (rustls-tls) — HTTP client; rustls choice means no system OpenSSL dependency, identical cross-platform builds.
- **num-bigint 0.4** (`BigUint`) — 256-bit arithmetic for Q64.96 sqrt-price math; chosen over `ruint::U256` deliberately for the clean-room-port portfolio narrative.
- **rusqlite 0.31** (bundled) + **tokio-rusqlite 0.5** + **r2d2 0.8** + **r2d2_sqlite 0.24** — writer-thread + reader-pool SQLite topology; bundled feature compiles SQLite from source for reproducibility.
- **refinery 0.8** — forward-only migrations embedded in the binary.
- **once_cell 1** (`Lazy<T>`) — process-wide constants (Q96 helpers, magic-tick array, reqwest client hoist).
- **chrono 0.4** (clock, serde), **thiserror 1**, **proptest 1** (dev), **tempfile 3** (dev).
- **Plain CSS** — no Tailwind / shadcn / Chakra; 12 per-component CSS files in `src/styles/components/`; explicit decision for dark-dense trading-terminal aesthetic without library-default light styling.

### Runtimes / engines / platforms

- **Tauri IPC** — JSON over the WebView ↔ Rust boundary; the strict no-shared-memory / no-shared-fs / no-WebSocket contract.
- **SQLite WAL** — `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON`, `temp_store=MEMORY`; 60-second tokio-driven `PRAGMA wal_checkpoint(TRUNCATE)` task.
- **Ethereum JSON-RPC** — read-only; multi-provider via Alchemy archive (chunked `eth_getLogs` + finalised-block helper) + LlamaRPC + Cloudflare + chain-official L2 RPCs.
- **The Graph hosted subgraph + gateway** — primary backfill via single GraphQL query (no chunking, no auth on legacy hosted URL).

### Tools

- **pnpm** — frontend package management (`pnpm-lock.yaml`).
- **Cargo** — backend manifest.
- **Vite** — frontend build.
- In-repo Python scripts (`scan_repo.py`, `fetch_commits.py`, `detect_staleness.py`, `search_content.py`) used as evidence sources in LifeOS — not Aurix product surface but part of how the project is maintained.

### Domains and concepts

- **Uniswap V3 mechanics** — Q64.96 sqrt-price encoding; tick-to-sqrt-price exponential via 20 magic constants; tick spacing and fee tiers; three V3 cases for liquidity ↔ amounts (current below range / inside range / above range); per-direction rounding policy preserved from `SqrtPriceMath.sol`; in-range fee share `swap_amount × fee_tier_bps / 1_000_000 × position_L / active_L`; V3-concentrated impermanent-loss closed forms.
- **LP backtesting methodology** — per-swap simulation (not per-block aggregation) chosen for tick-boundary correctness; LVR discrete approximation per Milionis-Moallemi-Roughgarden; rebalance rules (Static / Schedule / OutOfRange) with explicit recentring + gas-cost accounting; selection-bias awareness (Bailey/de Prado Deflated Sharpe acknowledged as missing).
- **Capital-allocation framing** — three-variant LP (Best / Naive / Median) explicitly to defeat selection-bias readings, compared across 6 benchmarks (Aave / Lido / S&P 500 / gold / T-bill / HODL) with adaptive-tercile vol-regime conditioning + verdict-prose synthesis.
- **Ethereum archive ingestion** — `eth_getLogs` batched fetcher with chunked range, ABI decoder per event signature (Swap / Mint / Burn / Collect), two's-complement int24 sign-extension, polymorphic `ArchiveSource` trait wrapping multiple providers.
- **Cross-chain DeFi** — 5 chains (Ethereum / Arbitrum / Optimism / Base / Polygon) × 3 protocols (Uniswap V3 + Sushi V3 + Pancake V3) with per-chain `ChainConfig` (subgraph URL, public RPC, block-time conventions).
- **Free-data architectural constraint** — tiered fallback (subgraph → user-Alchemy → public RPC → empty state); no paid APIs; no synthetic data in user-facing flows (synthetic separated structurally via sentinel tx-hash).
- **Database design under domain-natural-key idempotency** — composite-key `INSERT OR IGNORE` (events, runs, strategies) versus `INSERT OR REPLACE` (provider-revisable benchmark series); TEXT-decimal big-integer encoding to preserve `uint160`/`uint128`/`int256` precision past SQLite's 64-bit INTEGER ceiling; writer-thread + reader-pool topology under WAL.
- **Tauri IPC contract discipline** — `serde(rename_all = "camelCase")` boundary; manual TS mirrors; documented codegen migration path (`ts-rs` / `specta`) for when drift becomes recurring.
- **Cross-cutting observability** — telemetry-as-replacement-for-screenshots; JSON event log with payload summarisation and per-page lastState snapshots; jq-driven diagnostics.

## Key technical decisions

- **Tauri over Electron.** Binary size (~5-15MB vs 100-200MB), memory footprint, capability-based security model. The Rust backend is the point, not a compromise — `num-bigint` precision is directly useful for V3 decoding; Node.js would lose both performance and ergonomic big-integer arithmetic.
- **Plain CSS over component library.** Aurix targets a dark, dense trading-terminal aesthetic; generic libraries default to light consumer styling and require significant overriding. No design-system enforcement is the accepted trade-off.
- **Rust backend over pure TypeScript for on-chain decoding.** `sqrtPriceX96` requires 256-bit arithmetic; future tabs (risk modelling, LP backtesting) involve heavy numerical work where Rust is genuinely better. The TS layer becomes pure display.
- **`num-bigint::BigUint` over `ruint::U256`.** Deliberate hiring-portfolio framing: the clean-room-port-from-Solidity story reads stronger with general-purpose Rust primitives. `ruint` would be faster (fixed-width, stack-allocated) but less directly tells the port story.
- **f64-then-refine inverse for `sqrt_price_x96_to_tick` over Solidity's bit-walking inverse.** Solidity's inverse is bit-precise but slow in BigUint arithmetic; f64 estimate + ±2-tick refinement closes the gap.
- **Per-swap fee distribution over per-block aggregation.** Block-level aggregation would be faster but loses information when multiple swaps cross tick boundaries within the same block.
- **Three-variant LP (Best / Naive / Median) over single-variant headline.** Earlier "best LP wins" framing was selection-bias-inflated and uninformative; three variants reframe the question to the more honest "would a typical LP have won?"
- **Adaptive-tercile vol-regime classifier over fixed thresholds.** Earlier 0.5% / 2.0% daily-vol thresholds were ETH/USDC-calibrated and broke on lower-vol or higher-vol pairs; adaptive partitioning self-calibrates per asset.
- **Subgraph-first → user-Alchemy → public-RPC tiered fallback over Alchemy-only.** Closed the no-API-key UX gap that blocked dashboard usability for users without a wallet (Caner has no wallet by project preference). Documented in commit `391eadd`.
- **No paid APIs (architectural constraint).** Free public RPC + free hosted subgraphs + DefiLlama + FRED `.txt` + Stooq + Yahoo + beaconcha.in (free tier). FRED key avoided by using `.txt` endpoints; Stooq dropped for FRED `SP500.txt` when Stooq added key requirement (commit `391eadd`).
- **No synthetic data in user-facing flows.** Caner verbatim: *"imagine if an engineer from a company we applied to wanted to test it out and saw completely made up numbers, he'd think we fucked up somewhere. better off displaying no data and having me fetch api keys than calculating things w made up data."* Synthetic ingest stays as a separate IPC for tests/dev but auto-run never calls it; structural separation via `SYNTHETIC_TX_HASH` sentinel.
- **Read-only by design.** No transaction submission, no private-key handling, no wallet-connection prompt. Blast radius of any bug is zero on-chain. Cannot be extended to execution without explicit principle violation (would require a fork/rebranding, not an increment).
- **Local-first.** All computation and storage on-device; no cloud, no telemetry, no server. History bounded by device storage is the accepted trade-off.
- **Tab-scoped independence.** Each tab is a self-contained feature module with its own types, backend commands, and analytical logic; cross-tab state sharing not permitted until explicitly designed. Tabs share `Storage`, telemetry, and `EthereumRpcClient` (transport-only) but no feature-level state.
- **No inline rationale comments anywhere in source.** Zero `WHY` / `NOTE` / `HACK` / `TODO` / `FIXME` / `IMPORTANT` / `SAFETY` annotations in `src/` or `src-tauri/src/` (verified by `search_content.py` 2026-05-04 — all hits in `.md` docs or `learning/exercises/` teaching). Design rationale lives in `context/` (in-repo) and the LifeOS vault — no two-canonical-homes risk.
- **Solo-contributor workflow — master only, no feature branches.** Caner is the only contributor; branches add overhead without coordination benefit. Commit-as-natural-checkpoint discipline matters more than it would on branch-and-merge.
- **Forward-only migrations.** Refinery has no rollback by design; `snapshots` table from M1.5 is unused but kept rather than risking manual data surgery.
- **TEXT-decimal big-integer encoding in storage over BLOB.** SQLite INTEGER is 64-bit signed; `sqrtPriceX96` is uint160, `liquidity` is uint128, amounts are int256 — INTEGER would lose precision. TEXT preserves range; per-loop parse cost was hoisted via the audit's `ParsedSwap` (Option A) rather than a deeper BLOB schema change (Option B) deferred.

## What is currently built

- **Tab 1 (Arbitrage)** — working. Multi-pair refactor, pool-fee P/L, card-grid dashboard, 12s default refresh. Milestones 1.1-1.4 substantially complete. Milestone 1.5 (historical chart of opportunity frequency, export) unchecked.
- **Tab 2 (LP Backtester, Vector A)** — code-complete + audit-improved. Full M2.0 storage → M2.8 capital-allocation headline stack, plus a 4-tier extension (cross-chain across Ethereum / Arbitrum / Optimism / Base / Polygon, V3 forks Sushi V3 + Pancake V3, non-USD-quote pools via DefiLlama token prices). 9-IPC auto-run pipeline. Live-data verification blocked pending Alchemy key restoration.
- **Tab 3 (Wallet Tracker)** — not started; no wallet-address input, no position decoder.
- **Tab 4 (Gas Intelligence)** — not started; gas read live in Tab 1 but not persisted historically.
- **Tab 5 (Risk Modelling)** — not started; no correlation, volatility, or VaR code.
- **Backend tests** — 139 pass / 0 fail / 3 ignored (live-Alchemy integration tests gated). Coverage: 30+ math, 18 storage, 6 backtest integration, 10+ ingest decoder + 6 pipeline, 4+ benchmarks, 3 strategies, 1 validation synthetic round-trip.
- **Frontend tests** — 0. Vitest setup deferred from the 2026-05-04 audit (one of three deferred audit findings).
- **Telemetry recorder** — frontend hub + backend persistence to `~/Library/Logs/com.ataca.aurix/last-session.json`.
- **Multi-tab shell** — `App.tsx` routes between Tab 1 and Tab 2 via `activeTabId` state + `TopBar` + route-aware `SettingsMenu`. Tabs 3-5 don't have IDs reserved yet.
- **Storage schema** — V001 (initial Vector A tables) + V002 (multi-asset headline columns). 12 domain CRUD modules.
- **Refinery + WAL + writer-thread + reader-pool** topology shipped; 60s WAL checkpoint task wired in `lib.rs::run`.
- **Three deferred audit findings remain open** — `commands/lp.rs` folder split, `backtest/engine.rs` setup→step→summarise split, `LpBacktestPage.tsx` → `useLpPipeline` hook + Vitest setup. All HIGH-severity per the audit but explicit zero-functional-change refactors.

## Current state

`active` per LifeOS frontmatter. HEAD commit `9085a82`; most recent commit 2026-05-04 (4 functional commits that day); 40 total commits since 2026-03-04. The 2026-05-03 sprint shipped Vector A end-to-end (M2.0 → M2.8 + 4-tier extension); the 2026-05-04 cycle ran a code-health audit and shipped 11 of 14 actionable findings to master across 4 commits. LifeOS records a 2026-05-13 re-verification pass confirming no new commits during the 9-day window; the 2026-05-04 HEAD is the current HEAD. In-flight: live-data verification pending Alchemy key restoration (carry-forward) and the three deferred audit findings (modularisation moves waiting for a focused session that can run-test the frontend).

## Gaps and known limitations

- **Live-data path non-functional (Gap A, Critical).** The Vector A LP backtester cannot currently run against real data. All three ingest tiers fail: Tier 1 subgraph at the deprecated legacy hosted URL returns a transport error; Tier 2 Alchemy returns 429/400 because the `.env` `ALCHEMY_API_KEY` is ~21 chars vs the typical 32+ (truncated); Tier 3 public RPC returns `"All RPCs are unreachable"` in the most recent session. Tab 1 silently falls through to LlamaRPC, masking the Alchemy failure — documented as a diagnostic trap. User's Alchemy account login is also broken, blocking key re-pairing.
- **Three deferred audit findings (Gap B, High).** `commands/lp.rs` folder split (628 lines, fan-out 16 — touches `lib.rs`'s 19-entry handler list); `backtest/engine.rs` setup → step → summarise split (488 lines, audit says zero functional change); `LpBacktestPage.tsx` → `useLpPipeline` hook + Vitest setup (668 lines). All mechanical refactors awaiting a session that can run-test.
- **Real-fixture validation pending (Gap C, High).** `validation/` currently has only synthetic round-trip coverage; real-fixture infrastructure (subgraph query for known LP positions + burn-tx receipt parsing → ground-truth fees) not started. Blocked on (a) live RPC access (Gap A) and (b) curated set of mainnet LP positions.
- **Deflated Sharpe correction missing (Gap D, High statistical).** The strategies grid picks the best Sharpe out of 81 cells; selection bias inflates the expected value of the "best LP wins" claim in the headline verdict. Bailey/de Prado correction is in `references/backtest-statistical-methodology.md` but not in code. Verdict's "best-cell beat all benchmarks" is statistically optimistic.
- **No frontend tests (Gap 6 frontend half, High).** Backend 139/139 pass; frontend has 0 tests, no Vitest setup. Deferred at session-end pending a focused session that can run-test.
- **Tab 1 fail-fast error model (Gap 2, High).** Any one venue's failure rejects the whole `fetch_market_overview` command; cannot show per-venue health. Tab 2 closes this structurally via 3-tier fallback. Tab 1 polish opportunistic per the 2026-05-02 direction decision.
- **Tab 1 hard-coded WETH/USDC + 220k gas estimate + stale "three venues" copy + per-adapter timestamps (Gaps 3 / 7 / 8 / 10).** Tab 1 polish opportunistic; Tab 2 has no equivalent gaps (`PositionConfig.token0_decimals`/`token1_decimals` come from live subgraph metadata; pool list covers 5 chains × 3 protocols).
- **No IPC contract check (Gap 11, Medium).** Rust DTOs and TS mirrors manually kept in sync via `serde(rename_all = "camelCase")` discipline; the `PositionConfig` snake_case/camelCase mismatch bug (commit `391eadd`) is exactly this class manifesting. Future codegen migration target named (`ts-rs` / `specta`).
- **`f64` precision at IPC boundary (Gap 5 partial, Low).** Math layer is fully fixed-point BigUint; `EquityCurvePoint` USD fields cross to TS as `f64` so very long backtests can accumulate ULP-level drift relative to a fully-BigInt accumulator (bounded ~1e-9 USD per audit analysis).
- **Duplicated TS primitives drifted (Gap 4, Medium).** `median()` ×3 (no drift yet), `formatUsd()` ×4 (drifted — `insights.ts` uses `signDisplay: "exceptZero"`; others use default), `GAS_UNITS_ESTIMATE = 220_000` ×3, gas-adjusted formula ×3. Tab 1 polish.
- **Subgraph schema dependency (Medium).** The hosted Uniswap V3 endpoint is signalled for deprecation in 2026; the gateway path needs `THE_GRAPH_API_KEY`. Caner has no wallet, so gateway free-tier UX uncertain.
- **Adaptive-tercile on short windows (Medium).** Sub-12-month backtests have ~1 bucket per regime — classification meaningless. Not yet filed as `potential-issues.md` entry but acknowledged.
- **No confidence intervals on win counts (Low).** "LP beat benchmark X 7/12" should carry a Wilson-interval CI; recorded as future addition.
- **`engine.rs` is 488 lines monolithic, `verdict.rs` is 414 lines, `commands/lp.rs` is 628 lines, `LpBacktestPage.tsx` is 668 lines.** Modularisation hygiene is the open dimension; covered by the deferred audit findings above.

## Direction (in-flight, not wishlist)

- **Restore Alchemy account login and replace the truncated `.env` key.** Carry-forward from the 2026-05-04 wrap-up. Live-verification of the entire 4-tier extension (cross-chain + V3 forks + non-USD pools) is blocked behind this; the telemetry log is the primary diagnostic surface — `lp.pipeline.ingest` event should show `report.sourceLabel = "alchemy:ethereum"` (or `"subgraph:Ethereum"`) and a non-zero `swapRowsPersisted` once the key is repaired.
- **Land the three deferred audit findings in a focused run-test-capable session.** `commands/lp.rs` folder split, `backtest/engine.rs` setup→step→summarise split, `LpBacktestPage.tsx` → `useLpPipeline` hook + Vitest setup.
- **Demo recording (`Work/README Demo.md`)** — milestone 1.5 demo still pending; LifeOS notes Vector A LP backtester is now the strongest demo candidate per the work file's 2026-05-13 status note. Cernio demo commit `Capataina/Cernio` `4a93239` referenced as comparable artefact.

Roadmap notes for Tab 3 (Wallet Tracker), Tab 4 (Gas Intelligence), Tab 5 (Risk Modelling), and a deeper LP Backtesting design space exist as design-space documents in `Roadmap/`, but per LifeOS frontmatter these are design-space-only notes, not in-flight implementation work — Tab 2 (Vector A) consumed the previous LP Backtesting roadmap and the work-file `Tab 2 Timeboost MEV Analytics.md` was explicitly superseded 2026-05-02 by Vector A. Vectors B (Mempool MEV Detector — Flashbots / Jump Crypto / Wintermute audience, 3-5 weeks, independent) and C (ML Arbitrage-Survival Classifier — crypto-quant desks, 4-8 weeks, shares Vector A's storage layer) remain `proposed`; appetite for them after A's live-data verification is a separate decision.

## Demonstrated skills

- **Production-grade Rust** across 79 files / ~10,500 LoC with strict module boundaries, per-subsystem typed-error enums (`thiserror`), and zero inline rationale comments anywhere in source (rationale lives in `context/` and the vault).
- **Clean-room port of Uniswap V3's mathematics stack** (TickMath, FullMath, LiquidityAmounts, SqrtPriceMath) onto `num-bigint::BigUint` with no `ethers-rs` dependency and no third-party V3 SDK; pinned by 30+ unit tests including round-trip tick → sqrtPrice → tick and bit-exact matches against Solidity reference values.
- **Per-swap LP backtest engine** with LVR discrete approximation, rebalance rules with gas + MEV haircut accounting, and three audit-applied perf wins (pre-parse hoist, loop-invariant `HoldOnlyEvaluator`, incremental `fees_usd_acc` collapsing O(N²) → O(N) in swap count with documented ULP-drift bound).
- **Adaptive-tercile vol-regime classifier + multi-variant capital-allocation verdict** synthesising regime-conditional LP-vs-benchmark prose against Aave / Lido / S&P 500 / gold / T-bill / HODL — explicitly architected to defeat single-variant selection-bias readings.
- **Production SQLite topology in Rust** combining `tokio_rusqlite` writer thread + `r2d2_sqlite` reader pool under WAL with `PRAGMA wal_checkpoint(TRUNCATE)` on a 60s tokio task; forward-only `refinery` migrations; idempotency keyed on domain-natural composites (`(pool, block, log_index)`, `config_hash`, `(grid_id, cell_index)`) with `INSERT OR REPLACE` exception for provider-revisable benchmark series; TEXT-decimal big-integer encoding to preserve uint160/uint128/int256 past SQLite's 64-bit ceiling; sentinel-tx-hash separation of synthetic from live data.
- **Three-tier free-data fallback architecture** (`UniswapV3SubgraphSource` → `AlchemyArchiveSource` → public RPC → empty state) behind a polymorphic `ArchiveSource` trait, with `IngestionReport.source_label` + `attempted_sources` surfacing tier diagnostics to the UI; never falls through to fabricated data.
- **Ethereum archive log decoding** — `parse_uint256`/`parse_int256`/`parse_uint160_word`/`parse_uint128_word`/`parse_int24_word` (with two's-complement sign-extension); event-signature topic[0] keccak verification; 10+ decoder tests.
- **Cross-chain DeFi system design** across 5 chains × 3 protocols (Uniswap V3 + Sushi V3 + Pancake V3 forks) with per-chain `ChainConfig` (subgraph URL, no-auth public RPC, block-time conventions).
- **Tauri 2 desktop architecture** with strict two-runtime separation, 19 IPC handlers, route-aware settings menu, multi-tab shell with `usePersistedState` + shape-merge defence against persisted-state drift, and a cross-cutting telemetry recorder that replaces screenshot diagnostics with structured JSON event logs (`~/Library/Logs/com.ataca.aurix/last-session.json`).
- **IPC contract discipline under hand-kept-mirror constraint** — `serde(rename_all = "camelCase")` boundary; 217-line `types.ts` mirrors; documented codegen migration target (`ts-rs` / `specta`) for when drift becomes a recurring class.
- **Quant-relevant statistical awareness** — explicit recognition that "best-cell LP wins" is selection-bias-inflated (Bailey/de Prado Deflated Sharpe acknowledged as the corrective and present in `references/backtest-statistical-methodology.md`); three-variant LP framing introduced specifically to neutralise this; adaptive-tercile classifier acknowledged as degenerate on <12-month windows; corrected IL semantic (`raw - hold` with fees separate, not mixed in).
- **Architecture as a hiring artefact** — local-first / zero-cost / read-only / tab-scoped principles defended in writing with explicit trade-offs; "no paid APIs" framed as architecturally load-bearing with the 4-tier free-data fallback as the structural answer; deliberate clean-room-port narrative for the Q64.96 layer (`BigUint` over `ruint::U256`); deliberate solo-contributor master-only workflow with commit-as-checkpoint discipline.
- **In-repo project-memory practice** — 1.16MB `context/` folder with 14 system docs, 11 convention notes, 11 reference research papers (~770KB, ~17k lines, primary sources for the V3 math + LP-profitability + backtest-methodology literature), and a `context/plans/code-health-audit/` directory tracking audit findings; the system docs are referenced as `[verified: ...]` evidence anchors throughout.

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
| Projects/Aurix/Roadmap/Gas Intelligence.md | 174 | "#aurix #defi #roadmap #gas-intelligence" |
| Projects/Aurix/Roadmap/LP Backtesting.md | 167 | "#aurix #defi #uniswap #roadmap #lp-backtesting" |
| Projects/Aurix/Roadmap/Risk Modelling.md | 222 | "#aurix #defi #roadmap #risk-modelling #quant" |
| Projects/Aurix/Roadmap/Wallet Tracker.md | 164 | "#aurix #defi #roadmap #wallet-tracker" |
| Projects/Aurix/Systems/_Overview.md | 90 | "#aurix #systems #overview #vector-a" |
| Projects/Aurix/Systems/Analytics Engine.md | 284 | "#aurix #typescript #analytics #systems #defi" |
| Projects/Aurix/Systems/Backtest Engine.md | 199 | "#aurix #rust #backtest #lvr #vector-a #m2-3" |
| Projects/Aurix/Systems/Benchmarks.md | 159 | "#aurix #rust #benchmarks #defillama #fred #alpha #vector-a #m2-7" |
| Projects/Aurix/Systems/Cross Runtime Contract.md | 270 | "#aurix #rust #typescript #ipc #systems" |
| Projects/Aurix/Systems/Data Pipeline.md | 261 | "#aurix #architecture #systems #rust" |
| Projects/Aurix/Systems/DEX Adapters.md | 339 | "#aurix #defi #rust #uniswap #systems" |
| Projects/Aurix/Systems/GUI Layout.md | 332 | "#aurix #typescript #react #frontend #systems" |
| Projects/Aurix/Systems/Headline.md | 148 | "#aurix #rust #headline #capital-allocation #regime-classifier #vector-a #m2-8" |
| Projects/Aurix/Systems/Ingest.md | 175 | "#aurix #rust #ingest #eth_getlogs #subgraph #vector-a #m2-1" |
| Projects/Aurix/Systems/LP Backtest GUI.md | 175 | "#aurix #react #typescript #lp-backtest #vector-a #frontend" |
| Projects/Aurix/Systems/Math.md | 167 | "#aurix #rust #uniswap #v3 #tick-math #q64-96 #vector-a #m2-2" |
| Projects/Aurix/Systems/Runtime Foundation.md | 213 | "#aurix #rust #tauri #runtime #shell #multi-tab" |
| Projects/Aurix/Systems/Storage.md | 226 | "#aurix #rust #sqlite #persistence #storage #vector-a #m2-0" |
| Projects/Aurix/Systems/Strategies.md | 116 | "#aurix #rust #strategies #grid-search #vector-a #m2-5" |
| Projects/Aurix/Systems/Telemetry.md | 145 | "#aurix #telemetry #diagnostics #cross-cutting" |
| Projects/Aurix/Systems/Validation.md | 119 | "#aurix #rust #validation #vector-a #m2-4" |
| Projects/Aurix/Work/README Demo.md | 58 | "- Cernio demo commit: `Capataina/Cernio` `4a93239`" |
| Projects/Aurix/Work/Tab 2 Timeboost MEV Analytics.md | 91 | "#aurix #work #defi #timeboost #mev #sequencer" |

## Anomalies

All 29 LifeOS markdown files (4 top-level + 1 Vector A Sprint + 4 Roadmap + 17 Systems including `_Overview` + 2 Work) read successfully; no UNREADABLE files; no schema sections left without source evidence; `status: active` taken verbatim from `_Overview.md` frontmatter; per-system deep reads performed for Storage / Math / Ingest / Backtest Engine / Benchmarks / Headline (the load-bearing Vector A subsystems plus the hiring-portfolio centre of gravity), with the remaining subsystem subsections (Validation / Strategies / Vector A IPC layer / LP Backtest GUI / Telemetry / Tab 1 backend / Cross-Runtime Contract) synthesised from the Systems file last lines + tags + the Architecture.md subsystem-responsibility table and per-subsystem cross-references rather than separate deep-reads — every claim in those subsections is traceable to a sourced line in Architecture.md, _Overview.md, Decisions.md, Gaps.md, or the per-system file's evidence_basis frontmatter.
