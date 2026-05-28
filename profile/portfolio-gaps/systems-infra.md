---
title: Portfolio Gaps — Systems / Infra OSS
lane: systems-infra
last_updated: 2026-05-29
seed_source: legacy profile/portfolio-gaps.md
---

# Portfolio Gaps — Systems / Infra OSS lane

> Seeded from legacy portfolio-gaps.md. Will be overwritten by grade-jobs Phase 3 on next canonical run.

## Open gaps

- **Production OSS contribution velocity** — burn PR #4894 is APPROVED (+1864 LOC) and burn #4938 is draft; tinygrad #16119 minimal. Increasing contribution velocity (3-5 merged PRs across 2-3 upstreams) strengthens the OSS-anchored Systems / Infra signal.
- **Compiler internals depth** — Xyntra is typed-compiler-IR primitives, zero-deps; doesn't yet reach a working interpreter or a small JIT. Closure would be a working evaluation backend.
- **Database internals (B-tree / LSM / WAL beyond consumer use)** — Cernio + Image Browser + Aurix use SQLite + WAL extensively but database internals at the SQLite-fork-or-write level are undemonstrated.
- **Distributed systems consensus implementations** — Raft / Paxos / multi-paxos at implementation level (not just consumer use). Reflected in Apache Cassandra / Cockroach DB / TiDB role gaps.

## Confirmed strengths

- **Rust depth across systems-shaped projects** — Cernio (~14k LOC, 6 ATS providers, modular Ratatui) · Nyquestro (deterministic matching engine, zero `unsafe`) · Image Browser (28 backend files, dual-connection SQLite, ort production) · Xyntra (typed compiler IR primitives, zero deps).
- **SQLite WAL production discipline** — Cernio idempotent migrations + manual table-rebuild pattern; Image Browser writer/reader split closing 22-second `ipc.get_images` freeze; Aurix refinery migrations + 60s checkpoint_truncate task.
- **Lock-free engineering** — Nyquestro deterministic price-time-priority matching engine.
- **HTTP/1.1 from raw TCP** — Zyphos (from raw `std::net`, `panic::catch_unwind` isolation, atomic connection counter).
- **OSS engagement track established** — `Capataina/OpenSourceContributions` umbrella with per-repo `contribution-culture.md` + `repo-conventions.md` notes; substantive shipped contributions across burn (Rust ML), tinygrad (Python), alloy (Rust ETH).

## Closure prescriptions

1. **Increase OSS PR velocity.** Target 1 merged substantive PR per month across the existing engagement set (burn, tinygrad, alloy). Each PR is the load-bearing visible signal for Systems / Infra OSS lane.
2. **Take Xyntra past IR-primitives scaffold to a working evaluation backend.** Even a small interpreter would close the "compiler depth?" question and create a concrete artefact.
3. **OSS-aligned Cloudflare / Arm / MongoDB / Datadog application track** — these companies value the GitHub-portfolio signal directly; the OSS umbrella IS the application material for them.

## Pinnacle-relevant evidence

- Cloudflare — Rust-heavy edge infra; OSS-aligned culture; sponsor-reliable.
- Arm (Cambridge) — systems / OSS adjacent; profile-aligned location.
- MongoDB / Datadog / Snowflake — database / observability infrastructure; profile-aligned skill domains.
- Anysphere (Cursor) — devtools / systems blend; Rust-heavy.

## Lane-internal calibration notes

Systems / Infra OSS lane uniquely rewards the visible GitHub portfolio. Closure prescriptions are weighted toward visible OSS work, not closed-source project artefacts. The OSS umbrella architecture is itself a meta-signal that this lane values.
