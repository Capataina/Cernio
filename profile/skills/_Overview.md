---
title: Skills Overview
parent: Profile
last_updated: 2026-05-31
---

# Skills Overview

> Derived from `profile/projects/*.md` per the `populate-from-lifeos` skills-derivation rubric (Phase 5).
> Evidence-anchored, not LOC-based. Proficiency bands: **Proficient** | **Comfortable** | **Familiar**.
> Last synced: 2026-05-31.

This folder contains the canonical, calibrated view of what the portfolio demonstrates. Each file is shaped for grading subagents (`grade-companies`, `grade-jobs`, `prepare-applications`) to consume directly. The bands reflect what the project files surface, not what the user has been exposed to.

## File map

| File | Contents | Most important for |
|------|----------|--------------------|
| [`languages.md`](languages.md) | Programming languages, with cross-domain transfer notes | Q1 (achievability) / Q2 (CV-value) grading at the language-stack-fit layer |
| [`frameworks.md`](frameworks.md) | Application-shaping scaffolds (entry-point structure, threading model, lifecycle) | Stack-overlap-with-role-requirements grading |
| [`libraries.md`](libraries.md) | Discrete libraries the program calls into | Granular fit assessment |
| [`engines-runtimes.md`](engines-runtimes.md) | Heavyweight execution environments (ONNX Runtime, SQLite WAL, Bevy ECS, Tauri WebView, Ethereum JSON-RPC) | Engine-specific role requirements (e.g. ML inference role, game engine role) |
| [`tools-platforms.md`](tools-platforms.md) | Developer tooling, version control, infra, observability | Tool-fluency surface |
| [`concepts-domains.md`](concepts-domains.md) | **Most important for grading fit.** Domain expertise demonstrated, separate from any specific tool | Cross-stack concept transfer (lock-free, market microstructure, RL, DeFi, ML inference, compiler IR) |
| [`methodologies-soft.md`](methodologies-soft.md) | Working-style patterns (milestone-driven dev, docs-as-infrastructure, anti-puffing, parallel-agent orchestration, decision archaeology) | Cultural-fit signals beyond technical stack |
| [`lane-affinity.md`](lane-affinity.md) | **Pre-resolved per-lane evidence pack.** Pinnacle + supporting + skills + honest gaps for each of the 8 active lanes | Direct grading of role-lane fit; embedded verbatim by grading agents |

## Per-table band counts

| Table | Total entries | Proficient | Comfortable | Familiar |
|-------|---------------|------------|-------------|----------|
| Languages | 7 | 1 (Rust) | 3 (TypeScript, Python, C#) | 3 (C++, MDX, SQL) |
| Frameworks | 14 | 2 (Tauri 2, Tokio) | 4 (Bevy, Ratatui, React 19, Cargo workspaces) | 8 |
| Libraries | 26 | 0 | 9 (rusqlite, serde, reqwest, num-bigint, thiserror, chrono, ort, rand family, MonoMod/Cecil) | 17 |
| Engines & Runtimes | 11 | 1 (SQLite WAL) | 4 (ONNX Runtime, Tauri WebView, Bevy ECS, Ethereum JSON-RPC) | 6 |
| Tools & Platforms | 24 | 3 (Git/GitHub, Cargo, Claude Code skills native) | 5 (gh, CMake, Vite, Catch2/xUnit/vitest, Multi-agent subagents, TUI infographics) | 16 |
| Concepts & Domains | 24 | 5 (Local-first architecture, Reinforcement learning, Documentation-first iteration, Desktop application engineering Tauri 2, plus per-lane skills) | 14 | 5 (Cross-chain DeFi, Compiler/IR design, Content-addressed storage/VCS, Network protocols/HTTP server, Stochastic synthetic flow) |

## Lane affinity summary

8 lanes extracted from `profile/career-goals.md`. For each, see [`lane-affinity.md`](lane-affinity.md) for pinnacle + supporting + cross-cutting skills + honest gaps.

| Lane | Pinnacle | Supporting count | Headline gap |
|------|----------|------------------|--------------|
| `big-tech` | no pinnacle | 5 | No Cloud/K8s/Docker/Terraform; no distributed-systems-at-scale |
| `ai-ml` | NeuroDrive | 6 | No production-scale ML / CUDA-kernel work; no published research |
| `hft` | Nyquestro | 3 | No C++ at depth; no actual lock-free engine (foundation only); no kernel-bypass |
| `crypto-mm` | Aurix | 2 | Read-only by design; no live-trading PnL; no production CEX MM engine |
| `bank-strats` | no pinnacle | 4 | No e-trading platform internals; no kdb+/q; 2:2 credential filter |
| `systems-infra` | Cernio | 8 | No distributed-database tenure; no kernel-bypass; no production distributed-systems |
| `devtools` | Cernio | 4 | No widely-used external developer-tool releases; no LSP / IDE / language-server work |
| `fintech` | no pinnacle | 5 | No payments-rail / KYC / ledger experience; no production fintech product engineering |

## Calibration anchors (what the bands mean here)

- **Proficient** at the language level (Rust) means: central to 5+ projects across 5+ distinct domains (TUI + DB + game engine + ML + DeFi + HFT-style + desktop apps), with documented research-level technique work in at least one (handwritten PPO + biology-inspired learner in NeuroDrive; clean-room V3 math port in Aurix; safe-Rust deterministic matching engine in Nyquestro).
- **Comfortable** means: working competence demonstrated in a substantial context, can be used in production-like work without supervision, project files show concrete technique work.
- **Familiar** means: working knowledge demonstrated in a limited or single context; could pick up in a new project; depth bounded by source evidence.

The portfolio is research-heavy and systems-leaning. Strongest signals are in **reinforcement learning** (NeuroDrive depth is exceptional for the junior phase), **safe-Rust systems engineering** (Nyquestro + Image Browser + Cernio + Aurix + NeuroDrive), **local-first architecture** (5+ projects with deliberate cross-project pattern), and **documentation-as-infrastructure** (every project carries substantial context, with anti-puffing reconciliation as a deliberate discipline). Weakest signals are **cloud / distributed infrastructure at scale** (the #1 portfolio-gap across 5+ Cernio grading batches), **production-scale ML / CUDA-kernel engineering**, and **C++ at depth** (the #1 hft-lane blocker on 7+ roles in 2026-04-29 batch).
