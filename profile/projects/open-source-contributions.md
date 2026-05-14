---
name: Open Source Contributions
status: active
source_repo: https://github.com/Capataina/OpenSourceContributions (umbrella, private)
lifeos_folder: Projects/Open Source Contributions
last_synced: 2026-05-13
sources_read: 18
---

# Open Source Contributions

## One-line summary

A managed engagement track across 9 vetted Rust and ML-infrastructure upstreams, running out of a private umbrella repo (`Capataina/OpenSourceContributions`) that hosts durable per-repo culture/conventions notes and a local `scout-issues` skill, with active PRs into burn (one APPROVED, one draft) and tinygrad (one open) plus an interest comment posted on alloy.

## What it is

The OSS track is a deliberate, structured engagement programme — not opportunistic drive-by contributions. The vetting layer picked 9 upstream targets based on stack alignment (Rust ML/infra), AI-policy compatibility, and review culture. Each target has a per-repo notes file capturing maintainers, review dynamics, claim-then-implement conventions, AI-policy posture, and a contribution history. The umbrella repo (`Capataina/OpenSourceContributions`, private, 386 KB, 11 commits over 2 days, 64 markdown files in `Notes/`) holds working memory; the vault folder `Projects/Open Source Contributions/` holds the durable narrative. The umbrella ships its own scout-issues skill that runs across all 9 vetted projects in parallel, producing a ranked top-10 list per scout pass.

Two contributions have shipped substantive code so far: burn PR #4894 (A-FINE image-quality metric, +1864 / -0 across 10 files, APPROVED 2026-05-07 awaiting tracel-team merge) and tinygrad PR #16119 (minimal LSTM, +14 tokenised lines, open since 2026-05-09). Two more are in flight: burn PR #4938 (TensorContainer downcast panic fix, open as draft 2026-05-11) and burn issue #4519 (fold4d / Col2Im, informally claimed 2026-05-10).

## Architecture

The umbrella repo uses a **whitelist-style `.gitignore`** that excludes everything by default and explicitly re-includes `CLAUDE.md`, `README.md`, `Notes/`, `.claude/skills/scout-issues/`. Cloned upstreams (`tinygrad/`, `burn/`, etc.) live as sibling subdirectories inside the umbrella but are gitignored — they're working surfaces, not tracked content. Each cloned upstream additionally has a `.research/` subfolder inside its own clone (gitignored via `.git/info/exclude`) holding implementation notes that get extracted to the umbrella's `Notes/<repo>/` on session close.

`Notes/` is two-layered:

- **Universal layer:** speech-patterns.md (voice rules, commit-trailer omission, AI-tell anti-patterns), contribution-history.md (every attempt with lessons), possible-projects.md (the 9-project ranked list), and `_issue-scout-2026-05-10.md` (cross-project synthesis).
- **Per-project layer:** one subfolder per upstream (alloy/, burn/, candle/, mistral.rs/, ratatui/, tauri/, tinygrad/, tokio/, tract/) with `contribution-culture.md`, `repo-conventions.md`, and a session-dated `_issue-triage-2026-05-10.md`. Deep per-issue research lives in further subfolders (e.g. `Notes/burn/fold4d/` has 8 files covering math spec, ONNX Col2Im-18, reference implementations, burn-internal surface, 6-commit plan, testing strategy; `Notes/burn/2924-tensor-container-panic/` has 5 files including timeline, current code state, directional options, reproducer).

The vault folder `Projects/Open Source Contributions/` mirrors the cross-cutting layer as durable narrative — Architecture, Workflow, Speech Patterns, Project Portfolio, Scout Methodology, Decisions, Roadmap, Gaps, _Overview — plus a `Repos/` subfolder with one file per upstream capturing engagement timeline, claim status, and sequencing. The vault per-repo files are canonical for **what happened**; OSS-repo per-project notes are canonical for **what we learned in detail**.

The source-of-truth hierarchy is explicit: live PR/issue state on GitHub > vault per-repo file > OSS-repo per-project notes > cloned upstream's CONTRIBUTING.md/README.

## Subsystems and components

### tracel-ai/burn — Rust-first deep learning framework

Most active engagement. Three concurrent threads is the ceiling (Decisions §D10).

- **PR #4894 — A-FINE image-quality metric.** +1864 / -0 across 10 files. Adaptive Fidelity-Naturalness Evaluator, no-reference (blind) IQA metric proposed by torsteingrindvik on the image-quality-metrics meta-issue (#4312). Implementation includes an inlined CLIP ViT backbone with PyTorch-weight loader, 5 evaluator heads (technical, structural, aesthetic, authenticity, overall), end-to-end regression tests against the reference implementation, and a `forward_with_features` refactor preserving CLS-token output as a reusable feature-extraction surface. APPROVED 2026-05-07 by laggui (*"LGTM!"*) after a single CHANGES_REQUESTED round on `forward_with_features` returning a `ClipOutput { features, cls: Option }` struct. Awaiting tracel-team merge; no Caner-side action.
- **PR #4938 — TensorContainer downcast panic fix.** +291 / -50 across 5 files, branch `fix/tensor-container-downcast-panic` off `upstream/main` commit `1e289582e`. Closes issues #2924 (VirtualNonsense, Mar 2025) and #3969 (lucasmdjl, Nov 2025) — same root cause confirmed by laggui. Two reporters 8 months apart; VirtualNonsense's PR #2965 went stale on an unanswered direction question (Result-API fix vs Backend-generic refactor). This PR chose the smaller path: Result-API stays internal to the container layer, public `GradientsParams::get`/`remove` keep their existing `Option` return type, `NotFound` maps to `None`, `TypeMismatch` panics with a descriptive message. Bonus correctness in `remove`: peek-before-remove via `Any::is::<T>()` so a wrong-backend `remove` no longer leaks the tensor. 7 tests added (4 regression + 3 adversarial). Marked draft 2026-05-11; PR body explicitly invites direction call between this path and nathanielsimard's larger Backend-generic refactor. Workspace tests + `clippy -D warnings` + fmt + rustdoc all clean.
- **Issue #4519 — fold4d / Col2Im operator.** Informally claimed via antimora 👍 on 2026-05-10. ONNX `Col2Im` operator support, the inverse of `unfold4d`. Lives as a new `ModuleOps` trait method in `crates/burn-backend/src/backend/ops/modules/`. Two equivalent forward strategies (scatter-add vs conv_transpose2d-identity); backward is FREE (decomposes into already-differentiable primitives). ~460 LOC across 11 files, 6–8 focused hours estimated. Scoping comment NOT yet posted (agent draft ready). Queued behind PR #4894 review.
- **Issue #4716 — PytorchStore non-contiguous layer index bug.** Top scout pick #2 from 2026-05-10. Zero claimers, antimora-authored. Sits in the exact `burn-store` surface PR #4894 just shipped 1864 LOC into — 0-day ramp. Queued for natural baton-pass when PR #4938 lands or stalls.

burn's review culture is active and friendly: laggui (tracel-ai MEMBER) triages meta-issues, confirms claims within hours, gives implementation hints, uses short low-ceremony confirmations (*"Yeah it should be up for graps!"*). The PR-claim convention is "claim-then-implement": comment on the meta-issue, wait for MEMBER confirmation, then open the PR. Precedent-driven implementation is the norm — every closed item on a meta-issue links the merged PR as a template.

### tinygrad/tinygrad — minimalist deep learning framework

- **PR #16119 — minimal LSTM operator.** Open since 2026-05-09. +14 tokenised lines (line-budget-aware), forward + reverse + bidirectional directions, passes Silero VAD parity. **2026-05-10: geohot (project owner) commented asking about ONNX test suite LSTM tests; reply sent confirming 2 of 4 upstream tests pass with this scope, soft-checking on whether to enable them in this PR.** Awaiting chenyuxyz response on the soft-check.
- **Historical: PR #15453.** Closed 2026-03-25 (~6h 28m alive) by chenyuxyz with the stated reason *"+78 lines is too much"*. Original was +146 / -0 across 2 files; tests don't count against the core budget, but the `tinygrad/nn/onnx.py` core delta of +78 hit the budget limit. The resurrection effort built the minimal +14-line version after extensive postmortem work; a force-push attempt to reopen #15453 was blocked by GitHub policy, so #16119 is a fresh PR.

tinygrad is governed by an aggressive **line-budget philosophy** — low total line count in the core `tinygrad/` folder is treated as a first-class metric, enforced by a bot comment on every PR reporting `sz.py` tokenised-line delta. Written policy: *"If your PR looks complex, is a big diff, or adds lots of lines, it won't be reviewed or merged."*

### alloy-rs/alloy — modern Rust-first Ethereum library

- **Issue #1156 — JSON-RPC recursion limit.** Interest comment posted 2026-05-10. Top cross-project pick from the 2026-05-10 scout. ~30-LOC patch already drafted in-thread by ZzPoLariszZ; DaniPopes (active reviewer) endorsed in-tree fix. Awaiting joneskm (original 14-month-silent volunteer) or maintainer response.

alloy's AI-policy is best-in-class for the candidate set. mattsse (lead maintainer) trailers his own commits with `🤖 Generated with [Claude Code]`. Externals (Elena343-ai, stevencartavia, DivooOliver, anim001k, dhai, James Prestwich) carry the same trailer openly. CONTRIBUTING.md is welcoming (*"It doesn't matter if you are just getting started with Rust or are the most weathered expert."*). Conventional commits + signed commits required, nightly toolchain for fmt/clippy. Median merged-PR LOC: 42. Sister-repo discipline matters — sol! / ABI bugs belong in `alloy-rs/core`, not the consumer repo.

### Not-yet-engaged candidates (vetted, queued)

- **tauri-apps/tauri** (Tier 1) — explicit pragmatic AI policy + `ai-slop` enforcement; daily Caner stack (Image Browser, Aurix).
- **sonos/tract** (Tier 2) — ONNX/NNEF inference runtime; Sonos commercial backing. Note: tract Claude-trailers visible in `git log` are by external czoli1976, not maintainer kali (corrected in the 2026-05-10 scout).
- **EricLBuehler/mistral.rs** (Tier 2) — LLM serving; cleanest first contact for LLM-infra learning.
- **huggingface/candle** (Tier 2) — HF Rust ML framework; silent permissive AI-policy.
- **ratatui/ratatui** (Tier 3) — formal AI policy added 2026-05; daily Caner stack (Cernio, Nyquestro).
- **tokio-rs/tokio** (Tier 3) — aspirational; one merged PR is worth ten elsewhere.

## Technologies and concepts demonstrated

### Languages

- **Rust** — used substantively in burn contributions (1864 + 291 LOC = 2155 LOC across two PRs). Trait-based design (new `ModuleOps` method), generic over backend types, `Any::is::<T>()` for type-safe peek-before-remove, workspace-wide `clippy -D warnings` discipline.
- **Python** — used in the tinygrad LSTM implementation. ONNX operator semantics modelled in pure Python following tinygrad's `tinygrad/nn/onnx.py` conventions.

### Frameworks and libraries

- **burn** (deep-learning framework) — implemented a full image-quality metric (5 heads + CLIP ViT backbone + PyTorch-weight loader) inside the framework's perceptual-metrics directory convention. Following 4 precedent PRs (LPIPS, DISTS, Gram Matrix, FID) as templates.
- **tinygrad** — implemented ONNX operator following the codebase's tokenised-line-budget conventions.

### Runtimes / engines / platforms

- **ONNX runtime semantics** — modelled Col2Im-18 specification, LSTM operator with forward/reverse/bidirectional directions, regression-tested against ONNX reference outputs.
- **PyTorch weight import** — recurring pattern across LPIPS/DISTS/FID precedents; A-FINE loader mirrors the same approach.

### Tools

- **Git / GitHub** — claim-then-implement workflow with meta-issue comments, draft PRs as direction-forcing artefacts, force-push attempts on closed PRs (blocked by policy), informal-claim via 👍 reaction tracking.
- **`gh` CLI** — `gh api repos/<owner>/<repo>/pulls/<n>` is the authoritative source for live PR/issue state per the source-of-truth hierarchy.
- **`scout-issues` skill** — local Claude Code skill in the umbrella repo running parallel agents across all 9 vetted projects, producing ranked top-10 picks per pass.

### Domains and concepts

- **Image-quality metrics** — perceptual metrics with pretrained features; CLIP ViT-based no-reference IQA; 5-head fidelity-naturalness composition.
- **ONNX operator semantics** — Col2Im as the inverse of `unfold4d`; LSTM with multiple directions; regression-test parity against ONNX Runtime.
- **Maintainer-direction forcing** — draft-PR pattern that puts code on the table to force the direction answer that long-running issues never got (PR #4938 designed explicitly to avoid PR #2965's stale-on-direction-question fate).
- **Cross-repo AI-policy mapping** — explicit enforcement (tauri's `ai-slop`), formal allow (alloy via mattsse's own trailers, ratatui post-2026-05), silent permissive (candle), strict line-budget (tinygrad).
- **Speech patterns for upstream comms** — voice rules: no em-dashes, no slashes-as-prose-glue, plain text over decorated formatting, omit Co-Authored-By Claude trailer from commits (umbrella convention).

## Key technical decisions

- **D6 — Vault per-repo files are canonical narrative; OSS-repo per-project files are working memory.** When they diverge, the vault wins on "what happened" and the OSS-repo wins on "what we learned in detail."
- **D10 — Three concurrent burn threads is the ceiling.** Currently at 2 actively Caner-side (fold4d scoping pending, PR #4938 draft awaiting maintainer); A-FINE doesn't count (ball with tracel-team merge); PytorchStore stays parked.
- **Umbrella `.gitignore` is whitelist-style** — exclude everything by default, explicitly include `CLAUDE.md`, `README.md`, `Notes/`, `.claude/skills/scout-issues/`. Prevents cloned-upstream binaries and venvs from leaking into commits.
- **Two-layer Notes:** universal cross-repo material (speech-patterns, contribution-history, possible-projects) separated from per-project subfolders. Reduces cross-contamination when working in one upstream.
- **Draft-PR pattern as direction forcer** — PR #4938 marked draft until laggui or nathanielsimard responds; PR body explicitly invites the direction call.
- **Commit-trailer omission rule (umbrella convention)** — do NOT include `Co-Authored-By: Claude` trailers on upstream commits in repos without explicit AI-policy permission. alloy and ratatui have formal/observed permission; others are silent or restrictive.

## What is currently built

The umbrella repo (`Capataina/OpenSourceContributions`) has 386 KB tracked content as of 2026-05-10: CLAUDE.md (146 lines universal personality + workflow), README (26 lines), whitelist `.gitignore`, the `.claude/skills/scout-issues/` local skill, 4 universal Notes files (speech-patterns.md 28.3 KB, contribution-history.md 16.8 KB, possible-projects.md 14.8 KB, `_issue-scout-2026-05-10.md`), and 9 per-project subfolders covering all 9 vetted candidates with `contribution-culture.md`, `repo-conventions.md`, and `_issue-triage-2026-05-10.md`. Deep per-issue research lives in further subfolders: `Notes/burn/fold4d/` (8 files) and `Notes/burn/2924-tensor-container-panic/` (5 files) plus `Notes/tinygrad/` (7 files including `pr-15453-postmortem.md`, `silero-vad-spec.md`, `minimum-lstm-implementation.md`).

Two PRs are open or approved on upstream repos: burn PR #4894 (APPROVED) and PR #4938 (draft). One PR open on tinygrad (#16119). One interest comment on alloy (#1156). Two informal claims on burn (#4519, #4716 queued). The shipped LOC across opened PRs: 2169 (1864 A-FINE + 291 TensorContainer + 14 tinygrad LSTM).

## Current state

Active. Most recent commit on umbrella repo: 2026-05-10 04:15 UTC (`b7e08c2`). Most recent upstream activity: burn PR #4938 opened 2026-05-11 (draft); tinygrad PR #16119 received a geohot comment 2026-05-10 with reply sent same day. Active engagements: 6 (4 ball-on-others, 2 awaiting merge or direction).

## Gaps and known limitations

- **scout-issues skill output is unaudited** — the 2026-05-10 scout ran across 9 projects in parallel and surfaced a top-10 picks list, but only one of those picks (alloy #1156) has been engaged. The remainder are queued; whether the skill's ranking holds up against actual engagement quality is not yet validated.
- **No merged PR yet** — burn PR #4894 is APPROVED but unmerged (2026-05-13: 6 days past approval); tinygrad PR #16119 is awaiting maintainer; burn PR #4938 is draft awaiting direction. The contribution track has substantive shipped work but no merged-into-mainline outcome to point at on the profile yet.
- **Three-thread burn ceiling is a soft limit** — currently honoured but no mechanism enforces it beyond the agent's discipline.
- **AI-policy posture across candidates is mostly inference, not explicit text** — only alloy, tauri, and ratatui (post-2026-05) have observable or formal AI-policy signals. Others (candle, mistral.rs, tract, tokio) are inferred-silent-permissive.
- **Game-mod contributions** — the GitHub README lists 150,000+ aggregate mod downloads across RimWorld, Minecraft, Terraria, and Escape from Tarkov, but no LifeOS source documents these. Per the anti-puffing rule, this paragraph names the gap rather than synthesising content the LifeOS source does not support.

## Direction (in-flight, not wishlist)

- **burn PR #4894 merge** — ball is with tracel-team merge-rights holders; no Caner-side action.
- **burn PR #4938 direction call** — awaiting laggui or nathanielsimard response on Result-API vs Backend-generic refactor before clicking "Ready for review".
- **burn issue #4519 scoping comment** — agent draft ready with 2 directional questions on default-impl strategy + output_size param; pending post.
- **tinygrad PR #16119 follow-through** — soft-check sent on enabling ONNX test suite LSTM tests; awaiting chenyuxyz.

## Demonstrated skills

- **Substantial upstream Rust ML contribution.** burn PR #4894 (+1864 / -0 across 10 files, APPROVED) demonstrates the ability to implement a complete domain-specific metric inside a Rust deep-learning framework — inlined CLIP ViT, PyTorch-weight loader, 5 evaluator heads, end-to-end regression suite.
- **Type-correctness work in shared-state container internals.** burn PR #4938 demonstrates designing the smaller-path fix (Result-API internal to container, public `Option` API preserved) plus catching the latent leak in `remove` (peek-before-remove via `Any::is::<T>()`).
- **Surviving and shipping under aggressive line-budget review.** tinygrad PR #16119 is +14 tokenised lines (down from #15453's +146); the resurrection required postmortem-then-minimisation against a codebase that closes PRs on `+78 lines is too much`.
- **Maintainer-direction-forcing via draft PRs.** PR #4938's draft framing is deliberate — putting working code with tests on the table to force the direction call that killed PR #2965.
- **Reading upstream conventions before contributing.** Per-repo `contribution-culture.md` + `repo-conventions.md` files document maintainer voice, review cadence, AI-policy posture, claim conventions, commit-trailer rules — captured before opening PRs.
- **Cross-repo orchestration via local skills.** The `scout-issues` skill runs parallel agents across all 9 vetted projects and produces a ranked top-10 picks list per pass — local skill authorship demonstrated in service of OSS work.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Open Source Contributions/_Overview.md | 161 | "- [[Profile/Professional/Interests\|Interests]] — OSS as an interest territory" |
| Projects/Open Source Contributions/Architecture.md | 137 | "- [[Projects/_Overview\|Projects Overview]] — peer to all Caner-owned projects" |
| Projects/Open Source Contributions/Decisions.md | 179 | "- [[Projects/Open Source Contributions/Project Portfolio\|Project Portfolio]] — input to D10 sequencing" |
| Projects/Open Source Contributions/Gaps.md | 124 | "- [[Projects/Open Source Contributions/Decisions\|Decisions]] — G9 reasons the deliberate-skip below" |
| Projects/Open Source Contributions/Project Portfolio.md | 176 | "- [[Profile/Professional/Interests\|Interests]] — Rust systems, AI infrastructure, low-latency engines" |
| Projects/Open Source Contributions/Roadmap.md | 139 | "- [[Projects/Open Source Contributions/Repos/Burn\|Burn]] / [[Projects/Open Source Contributions/Repos/Tinygrad\|Tinygrad]] — engagement queue" |
| Projects/Open Source Contributions/Scout Methodology.md | 148 | "- [[Projects/Open Source Contributions/Architecture\|Architecture]] — the skill lives at `.claude/skills/scout-issues/` in the umbrella" |
| Projects/Open Source Contributions/Speech Patterns.md | 195 | "- [[Projects/Open Source Contributions/Decisions\|Decisions]] — the no-em-dashes / no-slashes / plain-text rules" |
| Projects/Open Source Contributions/Workflow.md | 145 | "- [[Projects/Open Source Contributions/Roadmap\|Roadmap]] — sequencing of next contributions" |
| Projects/Open Source Contributions/Repos/Alloy.md | 87 | "- [[Profile/Professional/Interests\|Interests]] — DeFi infrastructure as an interest territory" |
| Projects/Open Source Contributions/Repos/Burn.md | 183 | "- [[Profile/Professional/Experience\|Experience]] — counts as external open-source engagement with a Rust deep-learning framework maintainer team" |
| Projects/Open Source Contributions/Repos/Candle.md | 147 | "- [[Profile/Professional/Interests\|Interests]] — Rust ML / Hugging Face ecosystem" |
| Projects/Open Source Contributions/Repos/Mistral.rs.md | 164 | "- [[Profile/Professional/Interests\|Interests]] — LLM infrastructure as career territory" |
| Projects/Open Source Contributions/Repos/Ratatui.md | 134 | "- [[Profile/Professional/Interests\|Interests]] — Rust foundational libraries" |
| Projects/Open Source Contributions/Repos/Tauri.md | 150 | "- [[Profile/Professional/Interests\|Interests]] — local-first product engineering as career territory" |
| Projects/Open Source Contributions/Repos/Tinygrad.md | 150 | "- [[Projects/Open Source Contributions/Repos/Tract\|Tract]] / [[Projects/Open Source Contributions/Repos/Tauri\|Tauri]] — queued candidates" |
| Projects/Open Source Contributions/Repos/Tokio.md | 196 | "- [[Profile/Professional/Interests\|Interests]] — Rust async / foundational systems" |
| Projects/Open Source Contributions/Repos/Tract.md | 154 | "- [[Profile/Professional/Interests\|Interests]] — edge ML / on-device inference as career territory" |
