---
name: Aurix
status: active
source_repo: https://github.com/Capataina/Aurix
lifeos_folder: Projects/Aurix
last_synced: 2026-04-26
sources_read: 15
---

# Aurix

## One-line summary

Local-first, zero-cost, read-only DeFi analytics desktop application built on Tauri 2 (Rust backend, React 19 / TypeScript frontend) that polls four Ethereum DEX venues at 1Hz, decodes Uniswap V3 `sqrtPriceX96` with `num-bigint`, and derives cross-DEX arbitrage insights entirely on-device.

## What it is

Aurix is designed as a five-tab DeFi monitoring platform whose first tab — cross-DEX arbitrage detection on WETH/USDC across Uniswap V3 5bps, Uniswap V3 30bps, Uniswap V2, and SushiSwap — is the only tab implemented and is itself partial (README milestones 1.2, 1.3, and 1.5 remain unchecked). The core premise codified in LifeOS Decisions: serious DeFi analytics tools either cost money (premium APIs) or require trusting a third party with sensitive query patterns; Aurix makes the computation local, the data source public free RPC, and the scope deliberately read-only — no wallet connection, no signing keys, no transaction-submission code path. The repository's design ambition is the full five-tab platform (LP backtesting, wallet position tracking, gas-price intelligence, quantitative risk modelling); the implemented scope today is one feature module, `src/features/arbitrage/`, comprising a 1Hz polling scanner with a 100-sample in-memory rolling history. The distinction between design ambition and implemented scope is load-bearing: LifeOS explicitly warns "Do not describe Aurix as a five-tab platform in any downstream context."

## Architecture

Aurix runs two separate runtimes communicating exclusively via Tauri's IPC bridge — no shared memory, no shared filesystem, no WebSocket. The boundary is strict and intentional.

```
┌─────────────────────────────────────────────────────────┐
│  FRONTEND RUNTIME  (Chromium WebView, src/)             │
│                                                         │
│  App.tsx                                                │
│    └── ArbitragePage.tsx                                │
│          ├── setInterval(loadSnapshot, 1000 ms)         │
│          ├── requestInFlight ref guards overlap         │
│          ├── invoke("fetch_market_overview")  ─────────►│
│          ├── PriceCard.tsx   (hero readout)             │
│          ├── MarketChart.tsx (SVG, 4 modes)             │
│          ├── InsightsPanel.tsx ← insights.ts            │
│          └── venue lanes × 4  (VENUES × overview.venues)│
└─────────────────────────────┬───────────────────────────┘
                              │  Tauri IPC (JSON, camelCase)
                              ▼
┌─────────────────────────────────────────────────────────┐
│  BACKEND RUNTIME  (Native Rust process, src-tauri/)     │
│                                                         │
│  #[tauri::command]                                      │
│  fetch_market_overview()                                │
│    ├── AppConfig::from_environment (dotenv Once)        │
│    ├── EthereumRpcClient::new (per invocation)          │
│    └── tokio::join!(                                    │
│          fetch_weth_usdc_snapshot         (V3 5bps)     │
│          fetch_weth_usdc_30bps_snapshot   (V3 30bps)    │
│          fetch_uniswap_v2_snapshot                      │
│          fetch_sushiswap_snapshot                       │
│          gas_price_gwei                                 │
│        )  ← all five concurrent                         │
│    └── any Err → whole command fails (fail-fast)        │
│    └── MarketOverview { chain, pair_label,              │
│                         fetched_at_unix_ms,             │
│                         gas_price_gwei, venues[4] }     │
│    └── serde → camelCase JSON → IPC bridge → frontend   │
└─────────────────────────────────────────────────────────┘
```

The Tauri IPC bridge is unidirectional per tick: the frontend invokes, Rust responds, no server-push. Rust uses `#[serde(rename_all = "camelCase")]` on `PriceSnapshot` and `MarketOverview` so field names cross the boundary unchanged; the TypeScript side has a manually mirrored interface in `src/features/arbitrage/types.ts` — no code generation, no shared schema, drift detected only at runtime. Error propagation: `Result::Err(String)` from Rust → rejected JS Promise → caught in `ArbitragePage.loadSnapshot()` → `setErrorMessage(message)` → error banner in `PriceCard`.

The `fetch_market_overview` IPC call is the single critical operation: polled at 1Hz, drives every rendered value, blast radius is the whole product surface. Single external dependency: all four venue reads and the gas-price read go through one `EthereumRpcClient` pointing at one URL. Provider rate-limiting, outage, or JSON-RPC semantic drift on the free Alchemy/Cloudflare tier silently fails every 1Hz tick simultaneously — no alternate path, no circuit breaker, no fallback endpoint, no caching.

State ownership boundaries:

| State | Owner | Lifetime |
|---|---|---|
| RPC URL | `AppConfig` (`src-tauri/src/config.rs`) | dotenv parsed once per process via `ENVIRONMENT_BOOTSTRAP: Once` |
| HTTP client | `EthereumRpcClient` | constructed per IPC call; `reqwest::Client` internally pooled |
| Market history | `ArbitragePage` React state | session-only, wiped on reload, `HISTORY_LIMIT = 100` |
| Venue metadata (`VENUES`) | `ArbitragePage.tsx:30-55` module constant | source-level; joined to backend payload by `dexName` string equality |
| Chart series metadata (`SERIES_META`) | `MarketChart.tsx:24-41` module constant | same `dexName` keyspace as `VENUES`, **separate copy** |

Tab-scoped independence is the architectural commitment: each tab is a self-contained feature module (`src/features/<tab>/`) with its own types, backend commands, and analytical logic. Cross-tab state sharing is not permitted until explicitly designed. Today only `arbitrage/` exists; tabs 2-5 have no folder. The architecture has no routing layer (`App.tsx` renders one page), no persistence layer, no shared state between tabs, no background daemon, no automated tests at any layer, no contract check between Rust and TypeScript types, and no per-venue error isolation.

## Subsystems and components

### Data Pipeline (Rust backend)

Per-tick flow: `setInterval` fires every 1000ms → `requestInFlight` guard checked (drops tick silently if previous still in-flight) → `invoke("fetch_market_overview")` → `AppConfig::from_environment()` resolves `MAINNET_RPC_URL` or constructs Alchemy URL from `ALCHEMY_API_KEY` → `EthereumRpcClient::new(url)` → `tokio::join!(5 futures)` runs four venue adapters and `gas_price_gwei` concurrently → each future `.map_err(|e| e.to_string())?` (fail-fast) → `MarketOverview` assembled with `fetched_at_unix_ms` copied from the V3 5bps snapshot → serde camelCase → JSON → IPC bridge → frontend appends to history (`.slice(-100)`) → `deriveInsightsView(history)` → React render. Per tick the system makes ~9 RPC requests (V3 5bps: 1; V3 30bps: 1; V2: 3 sequential; SushiSwap: 3 sequential; `eth_gasPrice`: 1). 60 ticks/minute = ~540 requests/minute. Polling interval is hard-coded (`REFRESH_INTERVAL_MS = 1_000`); not user-configurable.

### DEX Adapters (Rust backend, `src-tauri/src/dex/`)

Four adapters, two protocol families. **Uniswap V3** (5bps and 30bps pools) calls `slot0()` (selector `0x3850c7bd`) once, decodes the first 32-byte word as `sqrtPriceX96`, and converts to spot price using `num-bigint::BigUint` because `sqrtPriceX96^2` exceeds 2^192 and overflows `u128`. The conversion `numerator = 2^192 * 10^12`, `denominator = sqrtPriceX96^2`, `price = numerator / denominator` corrects for the USDC/WETH decimal asymmetry (`TOKEN0_DECIMALS = 6`, `TOKEN1_DECIMALS = 18`); these constants are hard-coded to USDC/WETH. **Uniswap V2** and **SushiSwap** share `fetch_v2_snapshot` (only the factory address differs), making three sequential RPC calls: `getPair(USDC, WETH)` on the factory → `token0()` on the pair → `getReserves()` on the pair. The `token0_is_usdc` check determines reserve ordering; price is `(reserve0 / reserve1) * 10^12` with the decimal correction. V2 reserves decode from a 32-byte word as `u128::from_str_radix(.., 16)` (safe because on-chain reserves are `u112`). Each adapter has its own `thiserror` enum (`UniswapV3Error`, `UniswapV2Error`) wrapping `EthereumRpcError` via `#[error(transparent)] #[from]`. Pool addresses for V3 are compile-time constants (not factory-resolved); V2 pair addresses are resolved at runtime per call.

### Cross-Runtime Contract (`src-tauri/src/market/types.rs` ↔ `src/features/arbitrage/types.ts`)

Two types cross the IPC boundary: `PriceSnapshot` (one per venue: `chain`, `dex_name`, `pair_label`, `price_usd: f64`, `pool_address`, `fee_tier_bps: u16`, `price_source_label`, `fetched_at_unix_ms: u64`) and `MarketOverview` (`chain`, `pair_label`, `fetched_at_unix_ms`, `gas_price_gwei: f64`, `venues: Vec<PriceSnapshot>`). The `dex_name` string is an **implicit identity key** — the only cross-system join key — used by both `VENUES[]` in `ArbitragePage.tsx` (price-binding lookup, accent class) and `SERIES_META{}` in `MarketChart.tsx` (chart line colour, legend swatch). Renaming any `dex_name` in the backend silently breaks the frontend in two different ways: `VENUES.find()` miss falls back to `$0.00`, `SERIES_META[name]` miss crashes the chart on `undefined.accentClassName`. The contract carries only raw observations; all derived values (median, spread, gas-adjusted profit) are computed on the frontend, deliberately keeping the Rust backend stateless per command.

### Analytics Engine (TypeScript frontend, `src/features/arbitrage/insights.ts`)

The analytical layer derives `InsightsViewModel` (one primary insight card + ≤4 secondary cards + ≤4 transition events) from the rolling `MarketOverview[]` history. Threshold constants: `GAS_UNITS_ESTIMATE = 220_000` (assumed swap gas cost, undocumented source), `SHORT_WINDOW = 5`, `BASELINE_WINDOW = 20`, `PERSISTENCE_WINDOW = 4`, `EVENT_LIMIT = 4`. Spread-elevation threshold is `averageSpread * 1.15` (15% above baseline). Severity model is a four-level type `"info" | "watch" | "notable" | "actionable"` mapped to UI styling classes. The primary card severity is chosen by a priority cascade: positive run length ≥ 4 → `actionable` ("Positive setup holding"); else `gasAdjustedUsd > 0` → `watch`; else elevated-spread run length ≥ 4 → `notable`; else `info`/`notable`. Five transition event types are detected by walking consecutive sample pairs: richest venue change, cheapest venue change, gas-adjusted turning positive, gas-adjusted falling back, spread crossing the elevated threshold. The engine has no memoisation — `derivedHistory.map(deriveSample)` runs every render at ~1000 operations per frame for the full 100-sample window.

### GUI Layout (TypeScript frontend, `src/features/arbitrage/components/`)

Single-page application. `App.tsx` renders `ArbitragePage` directly with no routing. `ArbitragePage` owns all state (snapshot, overview, history, error, loading, chart mode, showEvents toggle, requestInFlight ref) and composes `PriceCard` (hero readout: status pill, dexName pill, priceUsd, gas in gwei, error banner, refresh button), `MarketChart` (hand-rolled SVG, 960×320 viewBox, 4 modes: `raw` / `deviation` / `spread` / `gas`, `spread` is default), `InsightsPanel` (severity-coloured cards), and venue lanes (4 exchange-cards). CSS architecture: plain CSS in `theme.css` (CSS custom properties for colours, spacing, typography) and `dashboard.css` (layout classes); no Tailwind, no shadcn, no MUI, no styled-components. Accent palette (mint, sky, lilac, peach) maps directly to the four DEX venues. Responsive collapse below 1040px.

## Technologies and concepts demonstrated

### Languages

- **Rust** (edition 2021) — all backend code in `src-tauri/src/`: `commands/market.rs` (Tauri IPC entry point), `config.rs` (env resolution with `Once` guard), `ethereum/client.rs` (JSON-RPC client), `dex/uniswap_v3.rs` and `dex/uniswap_v2.rs` (protocol-specific decoders), `market/types.rs` (cross-runtime types). Approximately 850 lines across 12 `.rs` files per the LifeOS Overview scale section.
- **TypeScript 5.8.3** — all frontend code in `src/`. ~1000 lines across 6 feature files in `src/features/arbitrage/`; heaviest are `insights.ts` (14.9KB, 430+ lines) and `MarketChart.tsx` (13.4KB, 320+ lines).

### Frameworks and libraries

- **Tauri 2** — desktop shell binding the React frontend to the Rust backend via IPC; capability-based security model.
- **React 19.1.0** — frontend UI framework; component tree owns all UI state, no global store.
- **Vite 7.0.4** — frontend build tool.
- **Tokio 1** — async runtime in the Rust backend (features `macros`, `rt-multi-thread`); used by `tokio::join!` to fan out concurrent venue fetches.
- **reqwest 0.12** with **rustls-tls** (no system OpenSSL dependency; same binary across platforms) — HTTP client for JSON-RPC.
- **num-bigint 0.4** — required for `sqrtPriceX96` decoding because `sqrtPriceX96^2` exceeds 2^192 and overflows `u128`.
- **thiserror 1** — per-module error enums with `#[error(transparent)] #[from]` wrapping the shared `EthereumRpcError`.
- **dotenvy 0.15** — `.env` and `../.env` fallback loading, guarded by a `Once` so it parses once per process despite `AppConfig::from_environment()` being called every 1Hz tick.
- **serde 1 + serde_json 1** — `#[serde(rename_all = "camelCase")]` is the only wire-level bridge to TypeScript; no DTO layer, no codegen.
- **hex 0.4** — encoding `eth_call` calldata and decoding response words.

### Runtimes / engines / platforms

- **Tauri IPC bridge** — JSON, camelCase via serde rename, unidirectional per tick, error propagation as stringified `Result::Err(String)` → rejected JS Promise.
- **Ethereum JSON-RPC** — provider-agnostic client targeting Cloudflare or Alchemy free tier; all five concurrent calls per tick share one endpoint.

### Tools

- No automated tests at any layer (no `tests/`, no `#[cfg(test)]`, no Jest, no Vitest in `package.json`).
- No CI configuration captured in LifeOS source.

### Domains and concepts

- **DeFi protocol decoding** — Uniswap V3 `sqrtPriceX96` Q64.96 fixed-point arithmetic; Uniswap V2 / SushiSwap constant-product reserve-ratio pricing; ERC-20 decimal correction (`10^(18-6) = 10^12`); function selector encoding (first 4 bytes of keccak256 of the signature).
- **Big-integer arithmetic at protocol boundaries** — retaining `BigUint` through V3 decode to avoid overflow, with an explicit `f64` precision drop documented at the serialisation boundary; the boundary is a deliberate choice with named consequences (~15 significant digits sufficient for display, insufficient for tick-level LP backtesting).
- **Concurrent fan-out with fail-fast aggregation** — `tokio::join!` of 5 futures with per-future `.map_err(|e| e.to_string())?` so any single failure rejects the whole tick; documented as a current limitation with a named fix direction (per-venue `Result<PriceSnapshot, _>` aggregation).
- **Cross-runtime contract management without codegen** — manual TypeScript mirror of Rust serde structs, with the documented hazard that field renames produce silent `undefined` reads with no compile-time signal.
- **Analytical primitives over rolling windows** — short-window slope (5 samples), baseline mean (20 samples), persistence detection by trailing run length (4-sample threshold), elevated-regime detection (1.15× baseline), transition-event detection by walking consecutive sample pairs.
- **Severity-tiered insight model** — four-level severity (`info` / `watch` / `notable` / `actionable`) chosen by a priority cascade with documented thresholds and named magic numbers.
- **Hand-rolled SVG chart rendering** — 960×320 viewBox, four mutually exclusive modes (`raw` / `deviation` / `spread` / `gas`), event-marker overlay; no charting library dependency.
- **Documentation-as-rationale convention** — zero `WHY`/`NOTE`/`HACK`/`TODO`/`FIXME`/`SAFETY` annotations in source; design rationale lives in `context/` and the LifeOS vault, not in inline comments. This is a deliberate two-canonical-homes-prevention discipline, codified in `context/notes/no-inline-rationale.md`.

## Key technical decisions

- **Tauri over Electron.** Binary size (~5-15 MB vs 100-200 MB), memory footprint (low native webview vs bundled Chromium), backend language (Rust vs Node.js), capability-based security model. The Rust backend is the point, not a compromise — `num-bigint` precision and Rust's performance profile are directly useful for on-chain decoding and the future heavy numerical computation that LP backtesting and risk modelling will require.
- **Rust backend over pure TypeScript.** `sqrtPriceX96` decoding requires 256-bit unsigned integer arithmetic; JavaScript's `BigInt` could handle it but `num-bigint` is more ergonomic and the precision guarantees are clearer. Future tabs (risk modelling, LP backtesting) involve heavy numerical computation where Rust's performance profile is a genuine advantage. Trade-off acknowledged: the TypeScript layer becomes a pure display layer, so any analytical primitive used in both runtimes (median, spread) must be kept in sync manually — already manifesting as `formatUsd` drift in the frontend.
- **Plain CSS over a component library.** No Tailwind, no shadcn/ui, no Chakra. Aurix targets a dark, dense monitoring aesthetic (trading terminal, not consumer app); generic component libraries default to light consumer styling and require significant overriding. Trade-off accepted: no design system enforcing consistency across future tabs.
- **Local-first as a non-negotiable architectural commitment.** All computation and storage on-device. No cloud backend, no server, no telemetry. Privacy (DeFi query patterns are commercially valuable), zero operational cost, offline resilience once RPC is up. The constraint: history is bounded by device storage; cross-machine sync requires a sync layer that does not exist.
- **Zero-cost as a non-negotiable architectural commitment.** Free public RPC endpoints only, no paid API keys, no premium data subscriptions. The `ALCHEMY_API_KEY` fallback acknowledges users may want a free Alchemy key for better reliability without cost. The constraint: rate limits become a real concern at higher polling cadences or multi-pair coverage.
- **Read-only as a non-negotiable architectural commitment.** Aurix never submits a transaction, never holds a private key, never requests wallet connection. A tool that cannot write to chain cannot drain funds — blast radius of any bug is zero on-chain. Cannot be extended into an execution tool without violating the principle (a fork, not an increment).
- **Tab-scoped independence over premature shared abstractions.** Each tab has its own types, backend commands, analytical logic; cross-tab state sharing is not permitted until explicitly designed. The right abstraction for "price history" should emerge from two working implementations, not be designed upfront. Trade-off accepted: there will be duplication across tabs (similar RPC patterns, similar charting); deduplication is a refactoring task for when patterns have stabilised.
- **Minimal IPC contract — backend produces facts, frontend derives meaning.** All derived values (spread, gas-adjusted, median, severity, events) are computed on the TypeScript side. Cost: duplication of analytical primitives across three frontend files (`ArbitragePage.tsx`, `MarketChart.tsx`, `insights.ts`). Benefit: backend is stateless per command, testable independently, analytical definitions can change without touching Rust.
- **No inline-rationale comments.** Zero `WHY`/`NOTE`/`HACK`/`TODO`/`FIXME`/`SAFETY` annotations across the entire codebase. Design rationale lives in `context/` (in-repo) or in the LifeOS vault — never inline — to avoid two canonical homes for the same knowledge.
- **`SystemTime::now()` per-adapter timestamps over command-entry timestamp.** Each `PriceSnapshot` carries a `fetched_at_unix_ms` set inside its adapter at decode time; `MarketOverview.fetched_at_unix_ms` is then copied from the V3 5bps snapshot specifically (not a command-level clock read). The four adapters' timestamps differ by tens of milliseconds within the same tick. Documented as a known limitation with a named fix direction.

## What is currently built

**Per LifeOS Overview's scale section**: 8 commits total on master (first 2026-03-04 "Tauri scaffold"; latest 2026-04-22 "rename ARCHITECTURE.md → architecture.md"), 67 files (excluding `node_modules/`, lockfiles, generated schemas), ~850 lines of Rust across 12 `.rs` files, ~1000 lines of TypeScript across 6 feature files, ~11KB of plain CSS, zero tests, one IPC command (`fetch_market_overview`).

The code that exists today is the Tab 1 four-venue arbitrage scanner in partial form. The scanner polls Uniswap V3 5bps, Uniswap V3 30bps, Uniswap V2, and SushiSwap at 1Hz, all hard-coded to WETH/USDC on Ethereum mainnet, decodes prices using protocol-specific paths (V3 via `slot0()` + `BigUint` arithmetic, V2/Sushi via `getPair`/`token0`/`getReserves` reserve ratio), produces a `MarketOverview` per tick, ships it across the IPC boundary, and renders four UI surfaces: `PriceCard` (hero readout), `MarketChart` (4-mode SVG chart over 100-sample rolling window), `InsightsPanel` (severity-coloured insight cards plus events feed), and four venue lanes. The frontend maintains a 100-sample rolling React state array; closing the window wipes all history.

**What is NOT built** (per LifeOS Overview's "Five-Tab Vision vs Implementation Reality" table verbatim):

- **Tab 1 milestones 1.2, 1.3, and 1.5 are unchecked**: graceful failure handling for stale/failed RPC connections; opportunity logging above a configurable threshold; historical opportunity chart; demo recording; clean shutdown and restart without data loss.
- **Tab 2 (LP backtesting)**: no `src/features/lp-*` or `src-tauri/src/backtest/` folder exists.
- **Tab 3 (wallet tracker)**: no wallet-address input, no position decoder.
- **Tab 4 (gas intelligence)**: gas is read live but never persisted or analysed.
- **Tab 5 (risk modelling)**: no correlation, volatility, or VaR code.
- **Cross-cutting**: no SQLite/database layer, no tab shell or routing, no per-venue error isolation, no IPC contract checks, zero tests at any layer, manual TypeScript mirror with no codegen, three sites with stale "three venues" copy after the venue count expanded to four.

## Current state

**LifeOS frontmatter status:** `active-status-undecided` with `project-active` tag. The most recent code commit touching `src/` or `src-tauri/` is `7ed0e482` (2026-03-22 "added insights"); the last two commits on master are context-folder upkeep, not feature work. LifeOS records "no feature development for over a month" as of 2026-04-24, and a Status Decision file (`Work/Status Decision.md`, opened 2026-04-19) explicitly flags the revive / pause / decommission decision as still pending. A separate work file dated 2026-04-25 (`Work/Tab 2 Timeboost MEV Analytics.md`) proposes a renewed direction: complete in-flight Tab 1 milestones (1.2, 1.3, 1.5) and add a new Tab 2 focused on Arbitrum Timeboost auction analytics + sequencer-ordering simulation, deliberately reusing ~70-80% of Tab 1's substrate (JSON-RPC client, ABI decoder, Q96 fixed-point arithmetic). Tabs 3-5 (LP backtesting, wallet tracker, gas, risk) are explicitly demoted to "optional roadmap" in this proposal.

## Gaps and known limitations

LifeOS captures eleven known gaps as of 2026-04-24, in rough order of structural impact:

| # | Gap | Urgency | What it blocks |
|---|---|---|---|
| 1 | No persistence — 100-sample in-memory React array, wiped on reload; no SQLite, no file write | Critical | Tabs 2-5, Tab 1 milestone 1.5 |
| 2 | Fail-fast error model — `tokio::join!` with per-future `?` makes any one venue failure reject the whole tick | High | Resilience, Tab 1 milestone 1.2, per-venue health UI |
| 3 | Hard-coded WETH/USDC — pool addresses, token addresses, decimal assumptions are compile-time constants | Medium | Multi-pair, Tab 5 correlation |
| 4 | Duplicated analytical primitives — `median()` × 3, `formatUsd()` × 4 (already drifted: `signDisplay: "exceptZero"` in `insights.ts` only), `GAS_UNITS_ESTIMATE = 220_000` × 3, gas-adjusted formula × 3 | Medium | Refactor safety, visual consistency |
| 5 | `f64` precision at the serialisation boundary — `BigUint` decoded but downcast to `f64` before IPC | Low (now) / High (Tab 2) | Tick-level LP backtesting accuracy |
| 6 | No tests — zero coverage across both runtimes; `sqrtPriceX96` decode has no fixture tests; V2 token0-ordering decision has no tests | High | All refactors |
| 7 | Fixed `220_000` gas estimate undocumented and unsourced; actual V3 swap gas varies 130k-300k | Medium | Spread-accuracy trust, `actionable` severity threshold |
| 8 | Stale "three venues" UI copy in three coordinated sites + Tauri/Vite scaffolding residue (productName "aurix" lower-case, "A Tauri App" description, starter SVGs bundled) | Trivial | Polish; signals inattention |
| 9 | No tab shell — `App.tsx` renders one page directly; the five-tab roadmap has no UI home | High | Tabs 2-5 |
| 10 | Per-adapter timestamps captured inside each adapter via `SystemTime::now()`; tick-level "simultaneous" sample is actually skewed by tens of ms | Low | Backtesting precision |
| 11 | No IPC contract check — Rust types and TypeScript mirror kept in sync manually with no automated validation | Medium | Safe renames; confidence extending the payload |

The deterministic-order quirk in `insights.ts` secondary cards: when `history.length < 5`, the array briefly contains 5 items before `slice(0, 4)` drops the `caution` card — meaning the caution card never appears in practice. LifeOS documents this as either a subtle bug or deliberate graceful degradation.

## Direction (in-flight, not wishlist)

**Active near-term work** per the 2026-04-25 work file:

- Complete Tab 1 milestone 1.2: graceful failure handling for stale or failed RPC connections — scanner survives a deliberately-killed RPC connection without crashing, surfaces failure in the dashboard with a "venue down" indicator, auto-retries with exponential backoff.
- Complete Tab 1 milestone 1.3: opportunity logging above a configurable user-set USD-spread threshold — persisted with timestamp, venues, spread, gas-adjusted profit; threshold configurable via a Settings UI.
- Complete Tab 1 milestone 1.5: historical opportunity chart over 1h / 24h / 7d windows + demo-recording artefact (~60s) for the README.
- Build Tab 2: Timeboost auction analytics on Arbitrum, measuring express-lane auction outcomes (the 200ms advantage that went live 17 April 2025). Reuses Tab 1's substrate (JSON-RPC client, ABI decoder, Q96 fixed-point) for ~70-80% of the work; the new content is Timeboost-specific RPC calls and auction-analytics computation.
- Sequencer-ordering simulator: replay historical Arbitrum blocks under alternative ordering policies (vanilla first-come-first-served vs Timeboost vs Chainlink FSS-style) and report MEV-redistribution differences.
- A paper-grade write-up framing the Timeboost-analytics findings (Methods → simulator implementation by file path; Results → measured Timeboost-redistribution numbers; Discussion → descriptive framing, not prescriptive "MEV-resistant" claims).

**Open underlying decision:** the Status Decision file (`Work/Status Decision.md`) remains unresolved — `Projects/Aurix/Overview.md` and `Projects/Index.md` need a current explicit status (`active` / `paused with resume trigger` / `decommissioned with reason`). The 2026-04-25 Tab 2 proposal implies revival but the formal status declaration is still pending.

## Demonstrated skills

- Implements a complete Uniswap V3 spot-price decoder from raw `eth_call` data: function-selector encoding, ABI argument padding, response-word slicing, `sqrtPriceX96` extraction, `BigUint` arithmetic to convert `sqrtPriceX96^2 / 2^192 * 10^12` into a USD price — without any DeFi SDK or web3 library dependency on the decode path.
- Implements a Uniswap V2 / SushiSwap reserve-ratio decoder with runtime pair-address resolution via factory `getPair`, `token0()` ordering check, `getReserves()` decode, and decimal correction — sharing one parameterised function across two protocols that have identical contract interfaces, deliberately not pre-emptively splitting them.
- Builds a Tauri 2 application with strict two-runtime separation: Rust backend, React 19 / TypeScript 5.8 frontend, Tauri IPC as the only crossing point, manual `serde rename_all = "camelCase"` ↔ TypeScript-mirror contract, with explicit awareness of the silent-drift hazard and a documented fix direction (`ts-rs` / `specta` codegen).
- Designs and operates a 1Hz concurrent fan-out pipeline (`tokio::join!` of 5 futures) with explicit fail-fast semantics, documented blast radius (single external RPC dependency, no fallback, no circuit breaker), and a named fix direction (per-venue `Result<PriceSnapshot, _>` aggregation surfacing partial successes).
- Captures architectural rationale in dedicated documentation rather than inline comments, with a codified convention preventing two canonical homes for the same knowledge — and grounds every architectural claim in verified file/line references with commit-pinned evidence.
- Writes a hand-rolled SVG chart engine (960×320 viewBox, four mutually exclusive modes — raw price, percent deviation, USD spread, gas-adjusted USD — with event-marker overlays in `gas` mode only) without a charting library dependency.
- Implements a multi-window analytical engine: short-window slope (5 samples), baseline mean (20 samples), trailing-run-length persistence detection (4-sample threshold), elevated-regime detection (1.15× baseline), transition-event walker with five typed event categories, and a four-level severity model chosen by priority cascade.
- Demonstrates conscious precision-trade-off engineering: retains `BigUint` through V3 decode arithmetic to avoid `u128` overflow at `sqrtPriceX96^2 > 2^192`, downcasts to `f64` only at the serialisation boundary, names the precision drop as the limiting factor for tick-level LP backtesting, and chooses the boundary location deliberately rather than uniformly.
- Engineers around free-tier RPC constraints: rate-limit awareness (~540 requests/minute against ~300M Alchemy CU/month free tier), `Once`-guarded dotenv parsing to avoid 1Hz re-parsing, `requestInFlight` ref guard to drop overlapping ticks rather than queue them, `rustls-tls` choice to remove the system OpenSSL dependency.
- Builds in honest awareness of what is missing: ships an eleven-item gap inventory naming each gap with concrete file/line evidence, named blast radius, and documented fix directions — including the "stale UI copy in three coordinated sites" trivial gap that signals attention to detail across runtime boundaries.
- Designs roadmap-as-design-space: each unimplemented tab (LP backtesting, wallet tracker, gas intelligence, risk modelling) has a complete LifeOS design note covering required mathematics (V3 tick-to-price, IL formula, Pearson correlation, parametric vs historical-simulation VaR, Multicall3 batching pattern, log-return rolling volatility), data-source trade-offs against the zero-cost principle, schema sketches, and named gap dependencies — demonstrating the analytical depth the implementation work would draw on.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Aurix/Overview.md | 170 | "#aurix #rust #defi #tauri #project-active" |
| Projects/Aurix/Architecture.md | 246 | "#aurix #rust #tauri #architecture" |
| Projects/Aurix/Decisions.md | 119 | "#aurix #decisions #architecture #defi" |
| Projects/Aurix/Gaps.md | 230 | "#aurix #technical-debt #gaps #defi" |
| Projects/Aurix/Systems/Analytics Engine.md | 284 | "#aurix #typescript #analytics #systems #defi" |
| Projects/Aurix/Systems/Cross Runtime Contract.md | 239 | "#aurix #rust #typescript #ipc #systems" |
| Projects/Aurix/Systems/DEX Adapters.md | 339 | "#aurix #defi #rust #uniswap #systems" |
| Projects/Aurix/Systems/Data Pipeline.md | 261 | "#aurix #architecture #systems #rust" |
| Projects/Aurix/Systems/GUI Layout.md | 329 | "#aurix #typescript #react #frontend #systems" |
| Projects/Aurix/Roadmap/Gas Intelligence.md | 166 | "#aurix #defi #roadmap #gas-intelligence" |
| Projects/Aurix/Roadmap/LP Backtesting.md | 150 | "#aurix #defi #uniswap #roadmap #lp-backtesting" |
| Projects/Aurix/Roadmap/Risk Modelling.md | 209 | "#aurix #defi #roadmap #risk-modelling #quant" |
| Projects/Aurix/Roadmap/Wallet Tracker.md | 152 | "#aurix #defi #roadmap #wallet-tracker" |
| Projects/Aurix/Work/Status Decision.md | 24 | "_(none yet)_" |
| Projects/Aurix/Work/Tab 2 Timeboost MEV Analytics.md | 74 | "#aurix #work #defi #timeboost #mev #sequencer" |
