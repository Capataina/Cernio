---
title: Portfolio Gaps — Fintech / Payments
lane: fintech
last_updated: 2026-05-29
seed_source: legacy portfolio-gaps.md
---

# Portfolio Gaps — Fintech / Payments lane

> Seeded from legacy portfolio-gaps.md. Will be overwritten by grade-jobs Phase 3 on next canonical run.

## Open gaps

- **Payments-protocol fluency** — PSD2, SCA, idempotency keys, dispute mechanics, settlement reconciliation. Self-teachable but not currently demonstrated.
- **Financial-systems regulatory awareness** — FCA permissions, payment-institution status, AML/KYC integration points. Background reading.
- **Production payment-rails integration** — Stripe / Adyen / GoCardless integration depth beyond consumer use.

## Confirmed strengths

- **Aurix DeFi microstructure** — directly transferable to fintech payment / settlement / liquidity engineering. Aurix is named verbatim at B2C2 in the rubric for exactly this reason.
- **SQLite WAL production discipline** — Cernio + Image Browser + Aurix all demonstrate transactional integrity at scale. Fintech engineers care about exactly this.
- **Reproducibility + idempotency** — Aurix StrictMode discipline + idempotency-at-storage layer making the second mount a cache hit; Cernio `INSERT OR IGNORE` on UNIQUE constraints as dedup mechanism. Both are the canonical fintech reliability patterns.
- **TypeScript-Rust polyglot** — Stripe / Adyen / Wise stack TypeScript-heavy with Rust services; portfolio breadth aligns.

## Closure prescriptions

1. **Build a small payment-mechanics demonstration.** A double-entry ledger with idempotency-keyed transfers (in Rust) demonstrates the reliability-pattern fluency that fintech screens for.
2. **Apply to Stripe / Adyen / Wise / Monzo / Revolut / Plaid / GoCardless.** Wide-funnel fintech grad / SDE-1 pipelines; sponsor-reliable; visa-friendly.
3. **Read up on PSD2 + SCA basics** — a single weekend covers the regulatory-context interview screen.

## Pinnacle-relevant evidence

- Stripe / Adyen — pinnacle Fintech employers; sponsor-reliable; brand-asset durable.
- Wise / Monzo / Revolut / Plaid — strong-secondary fintech; UK-native sponsor-friendly.

## Lane-internal calibration notes

Fintech lane benefits from Aurix's DeFi microstructure work — the cross-domain transfer is recognised by the calibration anchors (B2C2 cited Aurix verbatim). Lane-internal SS is accessible via the wide-funnel Stripe / Adyen grad pipelines.
