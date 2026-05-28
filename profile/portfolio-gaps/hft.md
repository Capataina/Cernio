---
title: Portfolio Gaps — HFT
lane: hft
last_updated: 2026-05-29
seed_source: legacy profile/portfolio-gaps.md (batches 2026-04-09 → 2026-05-17)
---

# Portfolio Gaps — HFT lane

> Seeded from legacy portfolio-gaps.md. Will be overwritten by grade-jobs Phase 3 on next canonical run.

## Open gaps

- **C++ Familiar → Proficient.** Primary blocker on 7+ roles per batch (Apple JDK ×2, Apple Kafka, Citadel C++ SWE, Tower Quant Developer, QRT Low Latency Market Data, Wayve Robot Software, Wintermute C++ Quant Trading Platform). Current C++ self-rated Familiar via Chrona (paused) and Tectra (scaffold). Rust-to-C++ translation is undemonstrated.
- **CUDA / GPU-systems / PTX / SASS / CUTLASS / Triton / NCCL.** Newly named in 2026-04-29 batch from Jane Street ML Performance Engineer (F). Distinct from "production-scale ML" — this is GPU-kernel-engineering specifically.
- **Production-scale market-microstructure exposure beyond simulator.** Nyquestro is a synthetic-feed matching engine. Real-exchange data integration (feed handlers against real venues, beyond Coinbase WS) would harden the HFT signal.
- **Specific HFT vendor experience** — FIX protocol fluency, kdb+/q exposure, ITCH/OUCH protocol familiarity. Currently absent.

## Confirmed strengths

- **Lock-free / wait-free systems** — Nyquestro (deterministic price-time-priority matching engine, zero `unsafe`, four-phase `submit_limit`).
- **Hot-path latency engineering** — hdrhistogram-based latency observability with p99/p999/p9999 surfaced.
- **Deterministic simulator infrastructure** — Tectra `Clock` abstraction with `VirtualClock.advance/set_time` as testability foundation.
- **WebSocket + TLS production discipline** — Nyquestro Coinbase Advanced Trade feed via tokio-tungstenite with `native-tls` chosen for macOS Security.framework trust store.
- **Profile-aligned Rust depth** — cross-domain across 9+ projects including Nyquestro's sub-100µs concerns.

## Closure prescriptions

1. **Take Tectra past the Clock-interface scaffold into a working feed-handler + matching loop in C++.** Most leveraged closure: moves C++ from "Familiar" to "Proficient" via demonstrable production-shaped code AND closes the Rust-to-C++ translation gap simultaneously. Highest-priority portfolio investment for HFT-lane access.
2. **OR finish Chrona's commit DAG to a working `chrona init / commit / log` MVP in C++.** Alternative C++ closure path. Requires less domain-specific HFT framing but still moves the C++ proficiency band.
3. **CUDA kernel project** — custom GEMM, attention kernel, matmul tiling. Unlocks the ML-performance-engineering role family at HFT firms (Jane Street ML Perf Eng, Citadel Securities GPU teams, HRT ML).

## Pinnacle-relevant evidence

- HRT grad SWE — wide-funnel + reputation → SS-in-hft anchor (rubric calibration target).
- Jane Street Tech track — strong reputation + brutal selectivity → S-in-hft (selectivity vs reputation tension preserved per realism semantic).
- Maven Securities / Vivienne Court — adjacent in HFT; reachable Tier-2 firms.

## Lane-internal calibration notes

HFT-pinnacle position is determined by `grade-companies` Phase 2 relativity pass; this file is gap-content not anchor-content. The list of firms above is illustrative for closure-prescription targeting, not a hardcoded calibration anchor.
