---
name: Aurix
status: paused
source_repo: https://github.com/Capataina/Aurix
lifeos_folder: Projects/Aurix
last_synced: 2026-04-29
sources_read: 16
---

# Aurix

> [!note] Status note — schema enum mapping
> LifeOS records this project as `active-status-undecided` (no feature commits since 2026-03-23 — over a month). Mapped to schema-conforming `paused` for the frontmatter. The `Work/Status Decision.md` file in LifeOS captures the pending revive/pause/decommission decision since 2026-04-19.

## One-line summary

Local-first, zero-cost, read-only DeFi analytics desktop app in Tauri 2 + React 19 + Rust — currently a working four-venue WETH/USDC arbitrage scanner against free public Ethereum RPC, with four additional analytical tabs (LP backtesting, wallet tracking, gas intelligence, risk modelling) designed but not implemented.

## What it is

Aurix is a local-first DeFi analytics desktop application built with Tauri 2, pairing a React 19 TypeScript frontend with a Rust backend to monitor and analyse decentralised exchange markets entirely on-device — no cloud, no wallet, no transaction capability. The core premise: serious DeFi analytics tooling either costs money (premium APIs, subscriptions) or requires trusting a third party with your activity. Aurix makes the computation local, the data source public (free on-chain RPC), and the scope deliberately read-only — the user can run it indefinitely for free, privately, without risk of accidental transaction submission. The README describes a five-tab platform; only Tab 1 (cross-DEX arbitrage detection) is partially implemented. Tabs 2–5 exist as design-space notes in LifeOS `Roadmap/` but no source code. The last feature commit was 2026-03-22; the project has been dormant for over a month at the time of LifeOS verification.

## Architecture

Two-runtime model (Rust backend + React frontend) communicating across the Tauri IPC boundary, with `serde` `rename_all = "camelCase"` as the cross-runtime contract. A single IPC command (`fetch_market_overview`) is exposed today.

```
┌──────────────────────────────────────────────────────────────────┐
│ React 19 + Vite 7 + TypeScript 5.8 frontend                       │
│  • src/features/arbitrage/ — ArbitragePage, MarketChart, insights│
│  • Plain CSS (no component library, no utility framework)        │
│  • 100-sample rolling window in React state, no persistence       │
└──────────────────────┬───────────────────────────────────────────┘
                       │ Tauri IPC (camelCase serde)
                       ▼
┌──────────────────────────────────────────────────────────────────┐
│ Rust backend (~850 lines, 12 .rs files)                           │
│  src-tauri/src/                                                   │
│  ├── lib.rs               # invoke_handler: fetch_market_overview │
│  ├── commands/market.rs   # 1Hz polling, four-venue scan          │
│  ├── ethereum/client.rs   # rustls reqwest, gas_price_gwei        │
│  ├── dex/                                                         │
│  │   ├── uniswap_v2.rs    # 5.4KB pool decoder                    │
│  │   └── uniswap_v3.rs    # 4.1KB sqrtPriceX96 + tick decoder     │
│  └── error handling: thiserror + ApiError                         │
└──────────────────────────────────────────────────────────────────┘
```

Four venues polled at 1Hz: Uniswap V3 5bps, Uniswap V3 30bps, Uniswap V2, SushiSwap — all WETH/USDC hard-coded. Pool addresses compiled in as constants. One pair, one chain (Ethereum mainnet).

## Subsystems and components

| Subsystem | Responsibility | Key files |
|---|---|---|
| **Data Pipeline** | Per-tick backend ingestion path; 1Hz poll loop; four-venue parallel fetch; rolling window | `src-tauri/src/commands/market.rs` |
| **DEX Adapters** | Per-venue price decoders — Uniswap V3 sqrtPriceX96 + tick math via num-bigint, Uniswap V2 reserve ratio, SushiSwap V2-fork | `src-tauri/src/dex/uniswap_v2.rs`, `uniswap_v3.rs` |
| **Cross Runtime Contract** | `PriceSnapshot` / `MarketOverview` types with `serde rename_all="camelCase"`; mirrored in TypeScript | `src-tauri/src/types.rs`, `src/features/arbitrage/types.ts` |
| **Analytics Engine** | TypeScript insight derivation, severity model, chart modes, primitive duplication (TS-side stats vs Rust-side stats) | `src/features/arbitrage/insights.ts` (14.9KB) |
| **GUI Layout** | Component structure, CSS architecture, VENUES/SERIES_META dual-key contract for venue labels + chart series | `src/features/arbitrage/`, `src/styles/` |
| **Roadmap design space** (LifeOS only) | Per-tab design notes for unimplemented Tabs 2–5: LP Backtesting, Wallet Tracker, Gas Intelligence, Risk Modelling | `Roadmap/*.md` in LifeOS |

## Technologies and concepts demonstrated

### Languages
- **Rust 2021** — backend, ~850 lines across 12 files; Uniswap V3 sqrtPriceX96 decode via num-bigint.
- **TypeScript 5.8** — frontend, ~1000 lines in `src/features/arbitrage/`; insights.ts is the analytical core.

### Frameworks
- **Tauri 2** — desktop shell, single IPC command exposed.
- **React 19** — frontend; functional components + hooks.
- **Vite 7** — build tool.

### Libraries
- **Tokio** (macros, rt-multi-thread) — async runtime for the 1Hz poll loop.
- **reqwest + rustls-tls** — HTTP client with no system OpenSSL dependency.
- **num-bigint 0.4** — Uniswap V3 sqrtPriceX96 decoding (Q64.96 fixed-point math via BigUint).
- **serde + serde_json** with `rename_all="camelCase"` — cross-runtime contract.
- **thiserror 1** — error taxonomy.
- **dotenvy 0.15** — env-based RPC config.

### Domains and concepts
- **DeFi market microstructure** — Uniswap V2 constant-product reserve math; Uniswap V3 sqrtPriceX96 tick math; cross-DEX arbitrage detection; gas-adjusted spread analysis.
- **Local-first / zero-cost architecture** — free public RPC only, no paid APIs, no wallet, never submits transactions.
- **Cross-runtime serde contracts** — `rename_all="camelCase"` as the boundary; TypeScript mirror types maintained alongside Rust types.
- **Read-only as a design constraint** — deliberately does not have wallet integration or transaction capability; the constraint is the security feature.

## Key technical decisions

- **Local-first, zero-cost, read-only, tab-scoped** — the four non-negotiable architectural commitments.
- **rustls over OpenSSL** — no system dependency for cross-platform builds.
- **Plain CSS, no component library** — same decision echoed across Aurix, Flat Browser, NeuroDrive, Cernio.
- **No-inline-rationale convention** — the codebase has zero `WHY/NOTE/HACK/IMPORTANT/TODO/FIXME/SAFETY/XXX` annotations. All design rationale lives in `context/` (in-repo) and LifeOS Decisions.md, not in inline code comments.
- **Pool addresses + venue list compiled in as constants** — accepts the trade-off that adding a new venue is a code change; eliminates runtime config surface for Tab 1.

## What is currently built

- 8 commits total on master (first 2026-03-04, latest 2026-04-22 — non-feature commits in April).
- 67 files in repo (excluding node_modules); 12 Rust files (~850 lines), 6 frontend feature files (~1000 lines).
- Tab 1 partial: live four-venue scanner, opportunity feed UI, per-DEX stats — feed/threshold flagging, historical opportunity chart, export, clean shutdown all unchecked per README's own milestone tickboxes.
- Tabs 2–5: zero source code (no `src/features/lp-*`, `src-tauri/src/backtest/`, wallet decoder, gas history table, correlation/VaR code).
- Zero tests in any form (no `tests/`, no `#[cfg(test)]`, no Jest/Vitest).
- One IPC command exposed (`fetch_market_overview`).
- 100-sample rolling window in React state (no persistence — window closes → history wiped).

## Current state

Paused (LifeOS: `active-status-undecided`). Last feature commit `7ed0e482` (2026-03-22). Subsequent commits are docs-only. `Work/Status Decision.md` captures the pending revive/pause/decommission decision since 2026-04-19. No feature development for over a month at LifeOS verification.

## Gaps and known limitations

- **README scope vs implementation scope diverge ~20× on ambition**.
- **Milestone 1 (Tab 1) itself incomplete** — opportunity-flagging, historical chart, export, clean shutdown all unchecked.
- **Zero tests** across both runtimes.
- **No persistence** — window close wipes history.
- **`Tab 2 Timeboost MEV Analytics.md` Work file** captures a future work item in the MEV space but no scaffolding exists yet.

## Direction (in-flight, not wishlist)

The pending status decision dominates direction. If revived, the first work is finishing Tab 1's milestones (opportunity flagging, historical persistence, clean shutdown) before opening Tab 2.

## Demonstrated skills

- **Tauri 2 + React 19 + Rust desktop architecture** — same stack as Image Browser, Cernio (TUI variant), NeuroDrive (Bevy variant); stack proficiency demonstrated across 4 projects.
- **DeFi market microstructure decoding in Rust** — Uniswap V3 sqrtPriceX96 + tick math via num-bigint Q64.96 fixed-point; Uniswap V2 reserve-ratio decode; SushiSwap V2-fork compatibility.
- **Cross-runtime serde contract design** — `rename_all="camelCase"`; TypeScript mirror types; ApiError discriminated union pattern.
- **Local-first / zero-cost / read-only product framing** — runs indefinitely against free public RPC, never submits transactions, no third-party trust required.
- **rustls-tls / no-system-OpenSSL discipline** — cross-platform Tauri build story without platform-specific TLS dependencies.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Aurix/_Overview.md | 181 | "- `ba6b971` (2026-04-15) — fix YAML frontmatter: quote values that break Obsidian's parser" |
| Projects/Aurix/Architecture.md | 246 | "#aurix #rust #tauri #architecture" |
| Projects/Aurix/Decisions.md | 119 | "#aurix #decisions #architecture #defi" |
| Projects/Aurix/Gaps.md | 230 | "#aurix #technical-debt #gaps #defi" |
| Projects/Aurix/Roadmap/Gas Intelligence.md | 166 | "#aurix #defi #roadmap #gas-intelligence" |
| Projects/Aurix/Roadmap/LP Backtesting.md | 150 | "#aurix #defi #uniswap #roadmap #lp-backtesting" |
| Projects/Aurix/Roadmap/Risk Modelling.md | 209 | "#aurix #defi #roadmap #risk-modelling #quant" |
| Projects/Aurix/Roadmap/Wallet Tracker.md | 152 | "#aurix #defi #roadmap #wallet-tracker" |
| Projects/Aurix/Systems/_Overview.md | 42 | "- [[Projects/Aurix/Roadmap]] — direction-of-travel" |
| Projects/Aurix/Systems/Analytics Engine.md | 284 | "#aurix #typescript #analytics #systems #defi" |
| Projects/Aurix/Systems/Cross Runtime Contract.md | 239 | "#aurix #rust #typescript #ipc #systems" |
| Projects/Aurix/Systems/DEX Adapters.md | 339 | "#aurix #defi #rust #uniswap #systems" |
| Projects/Aurix/Systems/Data Pipeline.md | 261 | "#aurix #architecture #systems #rust" |
| Projects/Aurix/Systems/GUI Layout.md | 329 | "#aurix #typescript #react #frontend #systems" |
| Projects/Aurix/Work/Status Decision.md | 24 | "_(none yet)_" |
| Projects/Aurix/Work/Tab 2 Timeboost MEV Analytics.md | 74 | "#aurix #work #defi #timeboost #mev #sequencer" |
