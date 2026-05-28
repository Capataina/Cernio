---
title: Skills
last_updated: 2026-05-29
maintained_by: populate-from-lifeos (per cernio-full-refactor.md §5.4)
---

# Skills — Per-Group Folder

> Derived from project files in `profile/projects/`. Last synced: 2026-05-29.
> Proficiency bands: Proficient | Comfortable | Familiar | Beginner.
> Per `populate-from-lifeos` rubric — evidence-anchored, not LOC-based.

This folder replaces the legacy single `profile/skills.md` flat file per the
lane-based-relativity refactor (cernio-full-refactor.md §5.4). Per-group split
because per-lane would duplicate (Rust appears in 5 lanes; lane-relevance
annotations live within each skill's evidence text rather than the file
boundary).

## Group files

| File | Group | Lane relevance (illustrative) |
|---|---|---|
| `languages.md` | Programming Languages | Rust → hft, crypto-mm, systems-infra, devtools, fintech · Python → ai-ml, fintech · C++ → hft, systems-infra |
| `frameworks.md` | Frameworks | Tauri → devtools · React → big-tech, fintech, devtools · Bevy → ai-ml (simulation) |
| `libraries.md` | Libraries | hdrhistogram → hft · ort + ONNX → ai-ml · tokio-tungstenite → hft, crypto-mm |
| `engines-runtimes.md` | Engines and Runtimes | ONNX Runtime → ai-ml · SQLite WAL → systems-infra, fintech · Ollama / Gemini → ai-ml |
| `tools-platforms.md` | Tools and Platforms | Git/GitHub → universal · Cargo → systems-infra · CMake → hft, systems-infra |
| `concepts-domains.md` | Concepts and Domains | Lock-free → hft · DeFi microstructure → crypto-mm · ECS architecture → ai-ml |
| `methodologies-soft.md` | Methodologies and Soft Skills | universal across lanes |

The per-group split is the canonical taxonomy chosen by populate-from-lifeos at
profile/skills/ folder regeneration time. New skills are inserted into the
appropriate group file by populate-from-lifeos; manual edits are valid but the
skill is the canonical maintenance path.

## Project anchors

Project evidence anchors live inside each group file alongside the skill claim.
This is the load-bearing distinction from the legacy flat skills.md: the
evidence and the claim are co-located in the per-group file, so a grading
agent reading `languages.md` for Rust evidence doesn't need to cross-reference
the projects/ folder separately.

Projects covered (per-row evidence): Cernio · Aurix · NeuroDrive · Nyquestro ·
Image Browser · Tessarix · Vynapse · Xyntra · Zyphos · Chrona (paused) ·
Tectra (dormant) · AsteroidsAI (dormant) · Consilium (dormant) · Open Source
Contributions (umbrella).
