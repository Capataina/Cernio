---
title: Portfolio Gaps — Crypto MM / DeFi
lane: crypto-mm
last_updated: 2026-05-29
seed_source: legacy profile/portfolio-gaps.md
---

# Portfolio Gaps — Crypto MM / DeFi Eng lane

> Seeded from legacy portfolio-gaps.md. Will be overwritten by grade-jobs Phase 3 on next canonical run.

## Open gaps

- **Real-exchange market-making operations exposure** — beyond Aurix's analytic Q64.96 Uniswap V3 port and Nyquestro's synthetic-feed matching engine. CEX integration (Binance / OKX / Coinbase via authenticated APIs) at production scale.
- **MEV / on-chain extraction techniques** — flashbots / Eden / private mempool experience. Aurix is analytics-side, not extraction-side.
- **Smart-contract engineering depth (Solidity)** — Aurix is clean-room Rust port reading Solidity reference; native Solidity authoring is undemonstrated.
- **Cross-venue arbitrage / settlement bridging** — DeFi-CeFi bridge mechanics.

## Confirmed strengths

- **Aurix → exactly the lane** — clean-room Uniswap V3 Q64.96 math on BigUint; multi-asset headline; named verbatim in B2C2 rubric reasoning. Aurix is the canonical lane-anchor evidence.
- **High-frequency synthetic-feed handling** — Nyquestro applies transferably; the matching-engine-side reasoning unlocks the maker-side framings at crypto MMs.
- **Production WebSocket discipline** — tokio-tungstenite with `native-tls` chosen empirically over `rustls`; transferable to Binance / Coinbase / Kraken venues.
- **Decimal big-integer encoding for sub-2⁶³ liquidity values** — Aurix uses TEXT decimal in SQLite preserving sqrtPriceX96 / liquidity range; directly transferable to crypto MM accounting layers.

## Closure prescriptions

1. **Extend Aurix from analytics to maker-bot prototype.** Build a paper-trading market-maker against Binance testnet using Aurix's microstructure understanding. Closes the analytics-vs-execution gap directly.
2. **Add Solidity contract to Aurix portfolio.** A small flash-loan-prevention or oracle-update mechanism in native Solidity demonstrates the language proficiency directly.
3. **Cross-venue arbitrage proof-of-concept** — even a paper-trading bot reading two exchanges and computing the arb signal demonstrates the cross-venue accounting + latency discipline.

## Pinnacle-relevant evidence

- Wintermute / B2C2 — London-based, sponsor-reliable, Aurix-evidence-aligned (B2C2 cited Aurix verbatim in 2026-04-29 grading reasoning).
- Flow Traders / Cumberland — adjacent; equivalent role-shape, different geographic footprint.
- GSR / Galaxy — strong-but-secondary; remote-friendly which aligns with Strategy A post-exit trajectory.

## Lane-internal calibration notes

Crypto MM is the lane with the strongest "already-remote" trajectory — closure prescriptions should be evaluated against the post-exit-leverage question (does this make me MORE valuable as an independent later, or is this lane the destination?). Per career-goals.md, Strategy A applies and prestige-build matters; for Crypto MM the prestige employer set is narrower than HFT but pinnacle quality is real.
