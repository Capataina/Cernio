---
name: Open Source Contributions
status: active
source_repo: https://github.com/Capataina/OpenSourceContributions
lifeos_folder: Projects/Open Source Contributions
last_synced: 2026-05-31
sources_read: 18
---

# Open Source Contributions

## One-line summary

Multi-project upstream-contribution programme to Rust ML, deep-learning, and systems repositories (burn, tinygrad, alloy, Tauri, tract, mistral.rs, candle, ratatui, tokio), structured around a private umbrella repo, a local parallel-agent issue-scouting skill, and a portfolio of in-flight engagements headlined by a +1864-line CLIP-ViT-backed image-quality metric merged-track in tracel-ai/burn.

## What it is

A deliberate, evidence-driven OSS-engagement programme run out of a single private umbrella folder at `~/Documents/Programming-Projects/OpenSourceContributions/` (GitHub: `Capataina/OpenSourceContributions`, 386 KB, 11 commits, 64 markdown notes + 1 Python fixture-capture script). The umbrella holds workflow rules, per-repo culture notes, contribution history, a local `scout-issues` Claude Code skill that dispatches one Opus background agent per pre-vetted target in parallel, and durable per-engagement working memory. Cloned upstream repositories sit as gitignored siblings inside the umbrella (whitelist-style `.gitignore`), so adding a new clone is zero-config and never absorbs upstream code into the umbrella's history. The portfolio currently tracks 9 vetted candidate projects across 4 tiers and 6 active engagements (4 ball-on-others, 2 ball-on-Caner) with a stated 6-month target of 3-5 merged PRs across 3-4 different projects.

The programme exists to convert deep technical practice into visible, maintainer-approved upstream contributions. Each engagement is research-led: precedent PRs are read end-to-end before any code is written, scoping comments are posted before implementation begins where the repo culture rewards it (burn A-FINE locked direction with a five-question comment before the 1864 lines were written), and rejection histories are mined before READMEs.

## Architecture

The programme is organised as an umbrella-folder architecture, not a code project. The structural map (sourced from `Architecture.md` and `_Overview.md`):

```
~/Documents/Programming-Projects/OpenSourceContributions/      # umbrella
├── CLAUDE.md                       # universal personality + workflow rules (146 lines)
├── README.md                       # 26-line GitHub description
├── .gitignore                      # whitelist style: ignore *, allow only managed paths
├── .claude/
│   └── skills/scout-issues/        # local skill, parallel-agent OSS issue scouting
│       ├── SKILL.md                # 19.4 KB
│       └── references/             # output-formats.md + per-agent-prompt.md
│
├── Notes/                          # durable, tracked content (66 markdown files)
│   ├── universal layer:
│   │   ├── speech-patterns.md      # 28.3 KB - voice rules
│   │   ├── contribution-history.md # 16.8 KB - every attempt + lessons
│   │   ├── possible-projects.md    # 14.8 KB - 9 projects ranked
│   │   └── _issue-scout-2026-05-10.md
│   │
│   └── per-project layer:
│       alloy/, burn/ (24+ files inc. fold4d/ + 2924-tensor-container/),
│       candle/, mistral.rs/, ratatui/, tauri/, tinygrad/ (7 files inc.
│       pr-15453-postmortem + silero-vad-spec + minimum-lstm-impl),
│       tokio/, tract/
│
└── <cloned-upstream>/              # GITIGNORED (e.g. tinygrad/, burn/)
    └── .research/                  # GITIGNORED via <clone>/.git/info/exclude
```

Three load-bearing architectural decisions:

- **D1 — single umbrella over per-project repos.** Cross-project synthesis (the `scout-issues` cross-project ranking) is trivial because everything is one folder away; institutional memory survives clone deletion.
- **D2 — gitignore-whitelist.** `.gitignore` ignores `*` and explicitly allows only `!.gitignore`, `!CLAUDE.md`, `!Notes/`, `!Notes/**`, `!README.md`, `!.claude/`, `!.claude/**`. Cloning a 600 MB upstream into the umbrella requires zero `.gitignore` changes.
- **D3 — from-root orchestration only.** Caner never `cd`s into a cloned project to start a session. Claude operates on clones through `cd`-prefixed Bash commands or absolute paths. All Claude-facing memory lives in the umbrella; the clone holds only what upstream sees plus a `.research/` scratch folder added to `<clone>/.git/info/exclude`.

`Notes/` is split into a **universal layer** (cross-repo material that applies to every contribution: voice rules, cross-attempt lessons, the 9-project ranking) and a **per-project layer** (one subfolder per upstream with `contribution-culture.md`, `repo-conventions.md`, dated `_issue-triage-YYYY-MM-DD.md`, and per-issue subfolders for active deep research). Promotion to the universal layer requires the rule to apply if the next project tomorrow were Python rather than Rust.

The source-of-truth hierarchy is explicit: live PR/issue state on GitHub > vault per-repo files (durable narrative) > OSS-repo per-project notes (deep working memory) > cloned upstream's CONTRIBUTING.md (stated rules, cross-checked against observed merge behaviour).

## Subsystems and components

### scout-issues skill (the only local skill)

The umbrella's `.claude/skills/scout-issues/` is the single local Claude Code skill, scoped to OSS-contribution work and pre-loaded with the 9 vetted candidate projects. It dispatches one Opus background agent per project in a single message (9 agents in one Claude turn), applies cleanliness filters (no claimer in issue thread, no in-flight PR cross-linked, no prior Capataina engagement, >=3 comments or untouched-and-actionable, aligned with the project's surface per per-repo notes), and ranks surviving picks by cleanliness > scope > alignment-to-Caner's-career-angle.

Each agent produces a per-project `Notes/<repo>/_issue-triage-YYYY-MM-DD.md` (top-10 picks); the orchestrator weaves these into a cross-project `Notes/_issue-scout-YYYY-MM-DD.md` synthesis (top-10 ranked across the entire pool). Side-effect updates to `contribution-culture.md` and `repo-conventions.md` are written whenever a scout pass observes durable culture shifts.

First all-projects invocation: 2026-05-10. Total surfaced: 81 picks (10 each from 8 projects; ratatui returned only 5 and tinygrad only 3, pruned hard by their own contributor environments). Output: top-10 cross-project synthesis at `Notes/_issue-scout-2026-05-10.md`; two high-confidence AI-policy corrections surfaced (ratatui now has a formal AI section in CONTRIBUTING.md; tract's 3 Claude-trailered commits in the last 100 are by external contributor czoli1976, not maintainer kali).

### A-FINE image-quality metric (tracel-ai/burn PR #4894)

The flagship in-flight engagement and the largest single piece of upstream code shipped. Implementation lives in `crates/burn-train/src/metric/vision/afine/` following the existing perceptual-metrics directory convention. Scope per laggui's locked-in 2026-04-23 guidance:

- Single PR (not a backbone-first PR followed by A-FINE), matching the inlined-backbone pattern used for LPIPS (#4403), DISTS (#4574), Gram Matrix (#4595), and FID (#4644).
- CLIP ViT inlined into the A-FINE module rather than carved out as a reusable component (a future CLIP-based metric can refactor it out).
- Five A-FINE heads + a PyTorch-weight loader + regression tests all shipping in the one PR.

PR shape: `+1864 / -0 across 10 files`. The `forward_with_features` refactor laggui requested on 2026-05-06 (returning a `ClipOutput { features, cls: Option }` struct) was pushed same-day; approval came 2 hours later (2026-05-07 15:27 UTC, "LGTM!"). Status: APPROVED 2026-05-07, awaiting tracel-team merge (no Caner-side action; ball with merge-rights holders).

Reference upstream materials cited in LifeOS: paper at arXiv:2503.11221, project page at `tianhewu.github.io/A-FINE-page.github.io`, reference implementation at `ChrisDud0257/AFINE` (Apache 2.0, compatible with burn's dual MIT/Apache-2.0 licence). PyTorch weight-import path mirrors the loader patterns already used by LPIPS / DISTS / FID.

### TensorContainer downcast-panic fix (tracel-ai/burn PR #4938)

Draft PR opened 2026-05-11 fixing a downcast panic in `TensorContainer::get`/`remove` on backend mismatch (caller's `B` ≠ stored `B`). Closes two long-standing issues 8 months apart (#2924 from VirtualNonsense, March 2025; #3969 from lucasmdjl, November 2025) — same root cause confirmed by laggui.

Diff: `+291 / -50 across 5 files` on branch `fix/tensor-container-downcast-panic` off upstream/main commit `1e289582e`. Implementation chose the smaller path: a Result-API stays internal to the container layer; public `GradientsParams::get`/`remove` and `Gradients::get`/`remove` keep their existing `Option` return type; `NotFound` maps to `None`; `TypeMismatch` panics with a descriptive message. Bonus correctness in `remove`: peek-before-remove via `Any::is::<T>()` so a wrong-backend `remove` no longer leaks the tensor (the old code did `HashMap::remove` then `downcast.unwrap()`, dropping the entry on the failure path before panicking).

7 tests added (4 regression + 3 adversarial). All workspace tests + `clippy -D warnings` + fmt + rustdoc clean. PR body explicitly invites maintainer direction between Result-API (this PR's path) and nathanielsimard's larger Backend-generic refactor; the draft framing is designed to force the direction answer that timed out and killed VirtualNonsense's PR #2965.

### ONNX LSTM operator (tinygrad/tinygrad PR #15453 closed → PR #16119 active)

Original PR #15453 (2026-03-24, +146 / -0 across 2 files; +78 tokenised core lines per `sz.py`) closed by chenyuxyz at 2026-03-25 00:38 UTC (~6h 28m alive) with the one-line verdict "+78 lines is too much". Original implementation covered: forward, reverse, bidirectional directions; full gate decomposition (i/o/f/c split of W, R, Wb, Rb per direction); optional inputs defaulted to zero tensors when absent (B, initial_h, initial_c); explicit `NotImplementedError` for `sequence_lens` and peephole weights P; attributes for `activation_alpha`, `activation_beta`, `activations`, `clip`, `direction`, `hidden_size`, `input_forget`, `layout`; gate clipping applied to all four pre-activations; `input_forget=1` → `f_t = 1.0 - i_t` coupled-input-forget-gate shortcut; outputs Y / Y_h / Y_c with layout permutation when `layout=1`; an `_apply_rnn_activation` helper supporting all 11 ONNX RNN activations.

Resurrection (PR #16119, 2026-05-09): a 14-tokenised-line minimum LSTM, scoped down to exactly what loading Silero VAD v5 through tinygrad's ONNX frontend actually needs (forward-only, default Sigmoid+Tanh+Tanh activations; no clip, peephole, bidirectional, input_forget; single regression test against Silero parity). Force-pushing onto #15453's head branch was blocked by GitHub policy, so the resurrection shipped as a fresh PR with `Notes/tinygrad/pr-15453-postmortem.md` capturing the closure forensics and `Notes/tinygrad/minimum-lstm-implementation.md` documenting the rebuild. Status 2026-05-10: geohot (project owner, not chenyuxyz) commented asking whether tinygrad's `test/external/external_test_onnx_backend.py:129` `backend_test.exclude('test_lstm_*')` could be replaced — background-agent research confirmed the minimal LSTM passes 2 of 4 upstream ONNX-conformance LSTM tests (`test_lstm_defaults`, `test_lstm_with_initial_bias`) and fails the other 2 by intention (peephole + `layout=1`, both intentionally out of the Silero scope). Reply sent confirming the breakdown; awaiting geohot follow-up.

### Other vetted upstream repositories (not yet engaged with code; deep research staged)

Each of the following has a per-repo vault narrative + an OSS-repo `Notes/<repo>/contribution-culture.md` + `repo-conventions.md` + `_issue-triage-2026-05-10.md` triage file already in place.

- **alloy-rs/alloy** — Modern Rust-first Ethereum library (successor to ethers-rs). Interest comment posted 2026-05-10 on issue #1156 (`alloy_json_rpc::Response` recursion-limit deserialisation). DaniPopes endorsed the in-tree fix; ZzPoLariszZ drafted a ~30-LOC patch in the thread; original volunteer joneskm has been silent for 14+ months. AI-policy environment is best-in-class for the candidate set: lead maintainer Matthias Seitz openly trailers his own Claude commits.
- **tauri-apps/tauri** — Rust-based local-first application framework Caner uses in production (Aurix, Image Browser). Tier 1. Explicit pragmatic AI policy with `ai-slop` enforcement (the label has been used multiple times in the last 30 days). Median external merged-PR LOC ≈ 49. Top scout pick: issue #14154 (async commands re-invoked on webview reload, ~30-LOC diff already sketched in-thread by lucasfernog). Pre-requisite for any PR: GPG-signed commits configured (Legend-Master blocks merges on this even for one-line PRs).
- **sonos/tract** — Sonos's Rust-based ONNX/NNEF/TensorFlow inference runtime for on-device AI. Tier 2. Commercially-backed (~20 crates organised by concern: op support, IR/runtime, backends, surface, test infra). Sonos-employee : single-trusted-external : dependabot ratio ≈ 38 : 9 : 3 in last 50 merges. 0 external rejections in 100 most-recent closed PRs (all 8 closed-not-merged were maintainer self-cancellations). Backward-compat golden rule: `models serialized with tract 0.x.y should work with tract 0.x.z where z >= y`. Top scout pick: issue #1775 (SimplifiedLayerNormalization ONNX op) — ONNX op work transfers directly from the tinygrad LSTM PR.
- **EricLBuehler/mistral.rs** — Eric Buehler's Rust-based LLM serving framework. Tier 2. Silent permissive with maintainer leading (Eric uses Claude himself on merged commits for DRY sampler logic, FP8 handling, prompt-truncation panic). Approval is fast and one-line ("Thank you @user!"). Median merged external PR < 50 lines net. Top scout pick: issue #2098 (gemma4 GGUF panic→error; identical pattern to merged #1916 by n-engine which Eric approved one-line).
- **huggingface/candle** — Hugging Face's Rust-first ML framework. Tier 2. Silent permissive (no CONTRIBUTING.md, no PR template, no Claude trailers visible — but AI-disclosed bug reports accepted without pushback). Deep PR review backlog (#3318 sits 4 months unreviewed). Top scout pick: issue #3522 (Metal SDPA panic in quantized LLaMA GQA — Caner has the Mac to reproduce).
- **ratatui/ratatui** — Terminal-UI library Caner uses in production (NeuroDrive, Nyquestro, Cernio). Tier 3. Formal AI-tools section added to CONTRIBUTING.md in 2026-05 (disclose AI use, keep AI-touched chunks small, review every line; AI commit trailers are expected to stay on for ratatui PRs). The `Good First Issue` label is unreliable — of nine currently-labelled issues, zero are actually unclaimed and unblocked. Top scout pick: issue #1259 (deprecate `Buffer::set_line` / `set_span`, joshka explicitly +1'd).
- **tokio-rs/tokio** — Foundational Rust async runtime Caner uses in Cernio, Nyquestro, Aurix. Tier 3. The highest review bar of any project in the candidate set. Mandatory: tests (integration preferred), loom tests for sync/runtime changes, miri tests for unsafe paths, spellcheck, `rustfmt --check --edition 2021` (not `cargo fmt`), version-pinned clippy, MSRV awareness (6-month-old Rust). Feature PRs opened cold get closed within a day; cleanest first-contact path is bug-discovery-in-own-work → file issue with minimal reproducer → wait for Darksonn (Alice Ryhl) → PR with regression test. Top scout pick: issue #7445 (tokio-test Mock deadlock docs warning, maintainer ADD-SP endorsed adding a short docs note and suggested wording direction).

### Game-mod aggregate downloads (no source evidence in LifeOS)

The Cernio profile README anchors mention "RimWorld, Minecraft, Terraria, Escape from Tarkov game mods — 150,000+ aggregate downloads". **No source evidence in LifeOS** for any of these entries: no per-mod folder, no platform IDs, no per-mod download counts, no patch histories, no listing under `Projects/Open Source Contributions/` or any other examined `Projects/` subtree. The LifeOS `Open Source Contributions/` folder is entirely focused on Rust/ML/Python upstream-contribution work for the 9 vetted candidate projects; game-mod activity is not represented in the source material available for this synthesis.

## Technologies and concepts demonstrated

### Languages

- **Rust** — Primary language for all 9 vetted upstream targets except tinygrad. Used at depth across deep-learning framework internals (burn `crates/burn-train/`, `burn-store`, `burn-backend` `ModuleOps` trait surface), LLM-serving infrastructure (mistral.rs subsystems including gguf, metal, cuda, gemma4), ONNX inference runtime internals (tract `onnx/src/ops/`, `linalg/`, `hir/`), async runtime internals (tokio M-time, M-sync, M-runtime modules with loom/miri test obligations), TUI library internals (ratatui Buffer / widget / deprecation surface), Ethereum tooling (alloy JSON-RPC deserialisation), and local-first application framework internals (Tauri IPC plumbing, command macros, AppHandle, plugin system, ACL/capabilities).
- **Python** — Used for tinygrad upstream contributions (PRs #15453 and #16119 to `tinygrad/nn/onnx.py` and `test/external/external_test_onnx_ops.py`). Also used in `Notes/burn/capture_afine_fixtures.py` (2.5 KB A-FINE fixture-capture utility, the single Python file in the umbrella).

### Frameworks and libraries (upstream targets, not consumed dependencies)

- **burn (tracel-ai/burn)** — Rust-first deep learning framework. Caner has shipped 1864 LOC into `crates/burn-train/src/metric/vision/afine/` (A-FINE perceptual metric with inlined CLIP ViT backbone + PyTorch-weight loader + regression tests) and +291/-50 LOC into the `TensorContainer` downcast-panic fix surface.
- **tinygrad** — Minimalist deep-learning framework. Caner has shipped a 14-tokenised-line minimum ONNX LSTM operator in `tinygrad/nn/onnx.py` (forward-only, Silero VAD parity-tested via the ONNX-Runtime regression harness sketched in `Notes/tinygrad/silero-vad-spec.md`).
- **alloy** — Modern Rust-first Ethereum library (successor to ethers-rs). Workspace split across `alloy-rs/alloy` (core) and `alloy-rs/core` (sol! / ABI primitives). Engagement scope: JSON-RPC response-deserialisation recursion-limit plumbing behind a feature gate.
- **Tauri 2** — Rust local-first application framework. Multi-repo project (sister repos: `tauri-apps/wry` for webview, `tauri-apps/tao` for event-loop). Production usage in Aurix + Image Browser; upstream-engagement scope is IPC plumbing / command macros / AppHandle / plugin system / bundler / CLI / ACL.
- **tract** — Sonos's Rust ONNX/NNEF/TensorFlow inference runtime. Workspace of ~20 crates. Engagement scope: ONNX op implementations transferring conceptually from the tinygrad LSTM PR.
- **mistral.rs** — Rust LLM serving framework. Engagement scope: GGUF arch validation, sampling-parameter extensions, encoder-skip flags for VRAM-constrained deployments.
- **candle** — Hugging Face's Rust-first ML framework. Sister repos: `huggingface/safetensors`, `huggingface/tokenizers`.
- **ratatui** — Rust terminal-UI library. Engagement scope: widget API deprecation and Buffer-API ergonomics.
- **tokio** — Foundational Rust async runtime. Sister-repo boundaries explicit (mio for OS-level event-loop primitives, axum for HTTP, tracing for structured logging, console for runtime debugger, bytes for `Bytes`/`BytesMut`).

### Runtimes / engines / platforms

- **CLIP ViT** — Vision Transformer backbone inlined inside burn's A-FINE module (per laggui's locked-in single-PR strategy matching LPIPS / DISTS / FID precedents).
- **ONNX Runtime conformance suite** — Used as the regression-test target for the tinygrad LSTM operator (4 upstream LSTM conformance tests: `test_lstm_defaults`, `test_lstm_with_initial_bias`, `test_lstm_with_peepholes`, `test_lstm_batchwise`).
- **Silero VAD v5** — The real-world ONNX model whose loading failure originally surfaced the tinygrad LSTM gap (Quantizr's 2025-06-20 issue #10897); used as the parity-test target for the minimal LSTM rebuild.
- **PyTorch** — Source of pre-trained weights for burn's perceptual-metrics family (LPIPS / DISTS / FID / Gram Matrix / A-FINE all import PyTorch weights via the `burn-store` PytorchStore surface).

### Tools

- **gh (GitHub CLI)** — Used for all upstream interactions: PR creation, issue comments, PR-state polling (`gh api repos/<owner>/<repo>/pulls/<n>`), fork-without-clone (`gh repo fork <owner>/<repo> --clone=false`).
- **Claude Code background subagents** — The `scout-issues` skill dispatches one Opus background agent per vetted project in a single message; 9 agents fire in one Claude turn, return roughly synchronously, outputs woven into a single synthesis file.
- **`.git/info/exclude`** — Per-clone, untracked, never-pushed exclusion file used for `.research/` scratch state inside cloned upstreams (per hard rule D4: never edit `.gitignore` in a cloned project for local tooling).
- **loom** — Concurrency-permutation tester required for any tokio sync/runtime contribution (`LOOM_MAX_PREEMPTIONS=1 LOOM_MAX_BRANCHES=10000 RUSTFLAGS="--cfg loom -C debug_assertions" cargo test --lib --release --features full -- --test-threads=1 --nocapture`).
- **miri** — Used for unsafe-code-path verification in tokio contributions (`MIRIFLAGS="-Zmiri-disable-isolation -Zmiri-strict-provenance" cargo +nightly miri test --features full --lib --tests`).
- **Covector** — Tauri's required changelog mechanism (`.changes/` directory with YAML frontmatter listing affected packages and bump levels; covector bot comments on every PR with a clickable "Add another change file through the GitHub UI" pre-fill).

### Domains and concepts

- **No-reference image quality assessment** — A-FINE (Adaptive Fidelity-Naturalness Evaluator) is a blind image-quality metric using pretrained perceptual features. Five evaluator heads + a CLIP-ViT backbone + a PyTorch-weight loader.
- **ONNX operator implementation** — LSTM (forward, reverse, bidirectional) with full gate decomposition, attribute handling, optional-input defaulting, layout-permutation, and per-direction dispatch. Conceptually transferable to fold4d (Col2Im inverse-of-unfold4d operator) and SimplifiedLayerNormalization (tract).
- **Backend-generic tensor containers** — TensorContainer downcast-panic-vs-typed-Error API design space; trade-off between a small Result-API fix that stays internal to the container layer (keeping public `Option` return types) and a larger Backend-generic refactor (nathanielsimard's path).
- **Lock-free + concurrency-correctness verification** — loom-permutation testing as a tokio review prerequisite for any sync/runtime changes.
- **Line-budget-driven design** — tinygrad's enforcement of a tokenised line count via `sz.py` as a first-class merge filter; designing operators against the budget from the start rather than after (the LSTM rebuild went from +78 to +14 by scoping down to exactly what Silero VAD needed).
- **Backward-compatibility serialisation rules** — tract's golden rule (`models serialized with tract 0.x.y should work with tract 0.x.z where z >= y`); additive-only NNEF/OPL changes within a 0.x line.
- **AI-policy taxonomy** — Spectrum from lead-maintainer-trailers-own-AI-commits (alloy's Matthias Seitz) through explicit-allow-or-visible-AI-commits (burn, Tauri, mistral.rs, tract, ratatui) through silent-permissive (candle, tokio) to insider-OK / external-hostile (tinygrad with externally-applied `ai-slop`-equivalent rejection despite geohot himself authoring multiple Claude-trailered commits).
- **Multi-agent parallel issue scouting** — The `scout-issues` pattern: single-message dispatch of N background Opus agents (one per vetted project), each applying project-agnostic cleanliness filters (no claimer, no in-flight PR, no prior engagement, ≥3 comments OR untouched-and-actionable, aligned with project surface) and ranking by cleanliness > scope > alignment.

## Key technical decisions

The standalone `Decisions.md` enumerates 11 design choices (D1–D11). The career-relevant ones:

- **D1 — Umbrella folder, not per-project repos.** Rejected alternatives: one Caner-side repo per upstream (would force copying speech-patterns and contribution-history into every repo), notes inside each fork (merge conflicts on rebase, notes show in PR diffs, forks get deleted), notes in a vault folder only (loses the clone-orchestration benefits). Single umbrella wins on institutional memory surviving clone deletion + zero-config new clones + trivial cross-project synthesis.
- **D2 — Gitignore-whitelist, not blacklist.** A blacklist would need a new line for every clone and would silently absorb any new clone that nobody remembered to add. The whitelist is zero-config: `git clone tracel-ai/burn .` from inside the umbrella works without touching `.gitignore`; umbrella stays at 386 KB regardless of how much upstream code sits next to it.
- **D3 — From-root orchestration only.** No `cd` into a cloned project to start a session. Consequence: no per-project `CLAUDE.md`, no per-project `Notes/` inside the clone — both would cause review noise (untracked) or PR diff pollution (tracked). Forking history stays clean; PR diffs stay clean; per-project memory persists across clone deletion.
- **D4 — `.git/info/exclude` for clone-internal scratch, never `.gitignore`.** Hard rule. `.gitignore` edits show up as unwanted diffs in any PR; `.git/info/exclude` is per-clone, untracked, invisible upstream.
- **D5 — Notes/ has two layers (universal + per-project).** Promotion bar for the universal layer: "would this apply to a Python project tomorrow as easily as a Rust one today?" Prevents premature universalisation.
- **D6 — Vault per-repo files are canonical narrative; OSS-repo per-project files are working memory.** Intentional duplication: vault Burn.md mentions "fold4d deep research lives in `Notes/burn/fold4d/`" without reproducing the math spec; the OSS-repo folder doesn't replicate the vault narrative.
- **D8 — Speech patterns must mature on real Caner-typed samples.** Until `Notes/speech-patterns.md` accumulates ≥3 real upstream messages Caner himself typed (not Claude-drafted-then-approved), every outbound message goes through draft → Caner edits or rewrites → send. Even when Caner says "just send it", the draft still gets shown first. Currently 0 verified Caner-typed upstream samples.
- **D9 — Default to omit AI commit trailers in upstream commits.** Caner is sole author; the `🤖 Generated with [Claude Code]` footer text is a hard avoid in any upstream context. Exceptions: repos where the lead maintainer trailers their own commits (alloy), internal/personal repos (LifeOS, the OSS umbrella itself), and repos with explicit AI-disclosure requirements (ratatui as of 2026-05).
- **D10 — Three concurrent burn threads is the ceiling.** Cross-thread context-switching slows Caner down past 3; maintainer relationship starts to feel demanding past 3.
- **D11 — One concern per PR, even when touching adjacent code feels efficient.** Hard rule. Review surface multiplies non-linearly with PR scope. Multi-concern PRs confuse the audit trail when regressions surface months later.

A-FINE-specific decision (locked 2026-04-23 with laggui's confirmation): single PR not backbone-first-PR-then-A-FINE; CLIP ViT inlined rather than carved out; port target `crates/burn-train/src/metric/vision/afine/`; five A-FINE heads + loader + tests all shipping together. This conversation-before-code pattern is what made the 1864-line PR manageable.

tinygrad-specific decision: line-budget design from the start. The PR #15453 closure produced a postmortem (`Notes/tinygrad/pr-15453-postmortem.md`) that drove the 14-token-line rebuild scope (forward-only, default activations, no peephole, no `layout=1`, no bidirectional, no `input_forget`, Silero-parity-only).

TensorContainer-specific decision: chose the smaller Result-API path over the architectural Backend-generic refactor; framed the PR as a draft explicitly inviting maintainer direction so the same unanswered-question pattern that killed VirtualNonsense's PR #2965 does not repeat.

## What is currently built

The umbrella repo at HEAD `b7e08c2` (2026-05-10) contains: umbrella CLAUDE.md (146 lines, universal personality + workflow rules), README, whitelist `.gitignore`, the `.claude/skills/scout-issues/` skill (19.4 KB SKILL.md + 2 references), 4 universal notes (`speech-patterns.md` 28.3 KB, `contribution-history.md` 16.8 KB, `possible-projects.md` 14.8 KB, `_issue-scout-2026-05-10.md` 11.6 KB), and 9 per-project subfolders covering all 9 vetted candidates with `contribution-culture.md` + `repo-conventions.md` + `_issue-triage-2026-05-10.md`. Total tracked content: 66 markdown files + 1 Python script + 1 `.gitignore`. Total repo size: 386 KB.

Upstream code currently in-flight or shipped:

| Upstream | PR | Lines | Files | State |
|---|---|---|---|---|
| tracel-ai/burn | #4894 (A-FINE) | +1864 / -0 | 10 | APPROVED 2026-05-07, awaiting tracel-team merge |
| tracel-ai/burn | #4938 (TensorContainer fix) | +291 / -50 | 5 | Open as draft, awaiting maintainer direction |
| tinygrad/tinygrad | #16119 (minimum LSTM) | +14 tokenised lines | 2 | Open, awaiting geohot follow-up on enabling 2 ONNX-conformance tests |
| tinygrad/tinygrad | #15453 (original LSTM) | +146 / -0 | 2 | CLOSED 2026-03-25 (+78 tokenised core lines, chenyuxyz "+78 lines is too much") |

In-flight issue engagements without PRs yet:

- tracel-ai/burn #4519 (fold4d / Col2Im ONNX operator) — informally claimed via antimora 👍 on 2026-05-10. Deep research staged at `Notes/burn/fold4d/`: 8 files including math spec, ONNX Col2Im-18 reproduction, reference implementations across PyTorch / ONNX-Runtime / candle / tract, burn-internal surface analysis, 6-commit bisectable implementation plan, testing strategy. Estimated effort: 6-8 focused hours; ~460 LOC across 11 files. Backward is FREE (decomposes into already-differentiable primitives).
- tracel-ai/burn #4716 (PytorchStore non-contiguous layer index bug) — scout pick #2 from 2026-05-10. Zero claimers, zero comments, antimora-authored. 0-day ramp on the fix (same `burn-store` surface as A-FINE PR #4894).
- alloy-rs/alloy #1156 (JSON-RPC recursion-limit deserialisation) — interest comment posted 2026-05-10. ~30-LOC patch already drafted in-thread by ZzPoLariszZ; DaniPopes endorsed in-tree fix path; original volunteer joneskm has been silent 14+ months.

Cross-project synthesis output (2026-05-10 first all-projects scout run): top-10 ranked across the entire 81-pick pool; two high-confidence AI-policy corrections (ratatui formal AI section added; tract Claude-trailers attributed to external czoli1976 not maintainer kali).

## Current state

Active. Umbrella repo bootstrapped 2026-05-09; 11 commits in 2 days; most recent commit 2026-05-10 04:15 UTC. Six active engagements: 4 ball-on-others (A-FINE awaiting merge, TensorContainer fix awaiting direction, tinygrad LSTM awaiting geohot, alloy #1156 awaiting joneskm/maintainer), 2 ball-on-Caner (fold4d implementation queued behind A-FINE; PytorchStore queued behind TensorContainer). Three-concurrent-burn-threads ceiling (D10) is at 2 actively Caner-side. No new project starts until at least one of the active threads moves.

## Gaps and known limitations

- **G1 — Zero verified Caner-typed upstream samples.** `Notes/speech-patterns.md`'s bootstrap observations come from chat-with-Claude register, not from public upstream comments Caner himself authored. The mandatory draft-then-edit loop will stay in force until 3 real upstream samples accumulate.
- **G2 — No baseline measurements for sustainable contribution velocity.** The stated 6-month target of 3-5 merged PRs across 3-4 projects is aspiration without empirical grounding. 2 days into umbrella-repo existence; needs 3-4 months of observed output to tune the queue depth.
- **G3 — burn fold4d implementation has not started.** Informally claimed via antimora 👍 on 2026-05-10; 8-file deep research plan staged; no code yet. Informal-claim decay clock: 2026-05-10 + ~2 weeks → re-evaluate by 2026-05-24.
- **G4 — burn TensorContainer scoping has no maintainer reply yet.** Same direction-question that killed PR #2965 (2026-04-14 → 2026-06-18 silent-bot-close). The draft framing of PR #4938 is the second attempt to force the answer.
- **G5 — Tauri requires GPG-signed commits, not yet configured.** Becomes a hard blocker the moment a Tauri PR is opened. Configure when Tauri becomes imminent (after alloy lands), not now.
- **G6 — `Notes/speech-patterns.md` Edit Deltas section is empty.** The strongest training signal for refining the upstream-OSS register; closes when Caner non-trivially rewrites a Claude-drafted upstream message and the before/after pair gets saved.
- **G7 — No `oss-workflow.md` / `communication.md` / `git-etiquette.md` / `pre-pr-checklist.md` universal notes yet.** Patterns haven't crossed the universal bar yet (would-apply-to-a-Python-project-tomorrow); will accrete as engagements multiply.
- **G8 — Vault doesn't track "ball is on whom" cleanly across active engagements in real time.** Snapshot at vault-write time; live state shifts in hours. Workaround: the morning-brew session-start synthesis includes burn / tinygrad / alloy state.

## Direction (in-flight, not wishlist)

Sequenced engagement queue from `Roadmap.md`:

1. **Now → next 1-2 weeks.** Wait for A-FINE PR #4894 merge; wait for chenyuxyz on tinygrad PR #16119; wait for maintainer direction on TensorContainer PR #4938. No new project starts.
2. **After A-FINE lands.** burn issue #4716 (PytorchStore non-contiguous layer index bug) — natural next burn contribution; same `burn-store` surface; 0-day ramp; antimora-authored.
3. **After alloy first PR.** Tauri first PR (target: issue #14154 IPC bug). Pre-requisite: GPG-signed commits configured.
4. **After Tauri first PR.** burn issue #4164 (Wgpu safetensors segfault) — same loader/store surface as #4716.
5. **After Tauri lands.** tract or candle first PR — pick whichever territory is calling louder at the time.

Background work throughout: tokio's `docs/contributing/` so when a bug-discovery moment arises organically in Caner's own work (Cernio, Nyquestro, Aurix), the file-an-issue-with-regression-test path is muscle memory.

## Demonstrated skills

Specific, evidence-anchored, drawn from the LifeOS contribution record:

- **End-to-end implementation of a published-paper computer-vision metric inside a production Rust deep-learning framework**, including inlining a CLIP ViT backbone, porting five A-FINE evaluator heads, building a PyTorch-weight-import loader, and writing end-to-end regression tests against the reference Python implementation. 1864 lines across 10 files, approved by tracel-ai maintainer laggui after a single-round refactor (`forward_with_features` returning a `ClipOutput { features, cls: Option }` struct).
- **Implementation of an ONNX operator from spec to merged-track in a strictly line-budget-enforced framework.** First attempt (+78 tokenised core lines) covered forward / reverse / bidirectional directions, full gate decomposition, all 11 ONNX RNN activations, gate clipping, layout permutation, coupled-input-forget shortcut, and four-test regression harness — closed on diff size alone. Second attempt scoped down to a 14-tokenised-line minimum LSTM (forward-only, default activations, Silero-parity-only) demonstrating the ability to redesign an operator against a hard line-budget constraint without losing the loading-correctness path.
- **Designing PR framings that force unanswered maintainer questions to get answered.** The TensorContainer-fix PR #4938 was opened as a draft with the body explicitly inviting maintainer direction between a small Result-API fix and an architectural Backend-generic refactor — the exact direction question that timed out and killed VirtualNonsense's PR #2965 over a 65-day silent-bot-close window.
- **Cross-backend tensor-container API design.** Authored a fix to `TensorContainer::get`/`remove` that keeps public `GradientsParams::get`/`remove` and `Gradients::get`/`remove` API-stable on their existing `Option` return type while internally distinguishing `NotFound` (maps to `None`) from `TypeMismatch` (descriptive panic). Bonus correctness: peek-before-remove via `Any::is::<T>()` so a wrong-backend `remove` no longer leaks the tensor.
- **Multi-agent parallel orchestration for OSS issue scouting.** Authored the `scout-issues` local Claude Code skill: single-message dispatch of one Opus background agent per vetted project (9 agents in one Claude turn), each applying project-agnostic cleanliness filters and ranking by cleanliness > scope > alignment, with cross-project synthesis producing top-10 picks ranked across the entire 81-pick pool. First all-projects invocation surfaced two high-confidence AI-policy corrections as a side-effect.
- **Repository-architecture design for a non-code, version-controlled working environment.** Whitelist-style `.gitignore` (ignore-`*` plus explicit allow-list) enabling zero-config cloning of arbitrary upstreams as gitignored siblings; from-root orchestration model keeping per-project Claude memory in the umbrella while leaving clones byte-for-byte identical to upstream; per-clone `.git/info/exclude` for scratch state; two-layer `Notes/` structure with a documented promotion bar.
- **Reading rejection histories before READMEs.** Mining a project's closed-not-merged PRs to surface the hidden filters before sinking implementation time. tinygrad's `sz.py` tokenised-line-budget filter became visible from reading recent closures; burn's inlined-backbone-precedent pattern became visible from reading the LPIPS / DISTS / FID / Gram-Matrix merged PRs end-to-end before A-FINE.
- **Cross-attempt lesson extraction.** Maintaining `Notes/contribution-history.md` (16.8 KB) as durable cross-attempt synthesis: pick by rejection-history not star-count; read 5-10 recent merged PRs before writing any code; use `Fixes #N` not prose; open a draft PR or interim status comment within 1-2 weeks of claiming; scope conversations before code; match the maintainer's tone; pivot fast on cultural mismatch.
- **Voice-discipline across two registers.** Documented and enforced rules for translating between Caner's chat-with-Claude register (shortenings, run-ons, Turkish-leak typos uncorrected) and an upstream-OSS register (same shape — run-ons, soft framing, no em dashes, no AI tells — but cleaned to full words and correct apostrophes). Explicit anti-pattern list ("customer service rep" test).
- **Multi-project, multi-stack engagement portfolio management.** Tier system across 9 vetted candidates (4 tiers: ship-ready Tier 1; strong-fit-with-caveats Tier 2; worth-knowing-higher-friction Tier 3; already-in-pipeline Tier 4); explicit ceilings (3 concurrent burn threads); sequenced queue with trigger-to-pick-up conditions; explicit-parked list with reasons; future-targets list with career-angle reasoning.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Open Source Contributions/_Overview.md | 161 | `- [[Profile/Professional/Interests\|Interests]] — OSS as an interest territory` |
| Projects/Open Source Contributions/Architecture.md | 137 | `- [[Projects/_Overview\|Projects Overview]] — peer to all Caner-owned projects` |
| Projects/Open Source Contributions/Decisions.md | 179 | `- [[Projects/Open Source Contributions/Project Portfolio\|Project Portfolio]] — input to D10 sequencing` |
| Projects/Open Source Contributions/Gaps.md | 124 | `- [[Projects/Open Source Contributions/Decisions\|Decisions]] — G9 reasons the deliberate-skip belongs with design rationale` |
| Projects/Open Source Contributions/Project Portfolio.md | 176 | `- [[Profile/Professional/Interests\|Interests]] — Rust systems, AI infrastructure, low-latency engines, local-first apps, DeFi infrastructure — the territories that filter the candidate pool` |
| Projects/Open Source Contributions/Roadmap.md | 139 | `- [[Projects/Open Source Contributions/Repos/Burn\|Burn]] / [[Projects/Open Source Contributions/Repos/Tinygrad\|Tinygrad]] / [[Projects/Open Source Contributions/Repos/Alloy\|Alloy]] — per-engagement narrative` |
| Projects/Open Source Contributions/Scout Methodology.md | 148 | `- [[Projects/Open Source Contributions/Architecture\|Architecture]] — the skill lives at `.claude/skills/scout-issues/` in the umbrella architecture` |
| Projects/Open Source Contributions/Speech Patterns.md | 195 | `- [[Projects/Open Source Contributions/Decisions\|Decisions]] — the no-em-dashes / no-slashes / plain-language choices belong with the design rationale` |
| Projects/Open Source Contributions/Workflow.md | 145 | `- [[Projects/Open Source Contributions/Roadmap\|Roadmap]] — sequencing of next contributions` |
| Projects/Open Source Contributions/Repos/Alloy.md | 87 | `- [[Profile/Professional/Interests\|Interests]] — DeFi infrastructure as an interest territory` |
| Projects/Open Source Contributions/Repos/Burn.md | 183 | `- [[Profile/Professional/Experience\|Experience]] — counts as external open-source engagement with a Rust deep-learning framework maintainer team` |
| Projects/Open Source Contributions/Repos/Candle.md | 147 | `- [[Profile/Professional/Interests\|Interests]] — Rust ML / Hugging Face ecosystem` |
| Projects/Open Source Contributions/Repos/Mistral.rs.md | 164 | `- [[Profile/Professional/Interests\|Interests]] — LLM infrastructure as career territory` |
| Projects/Open Source Contributions/Repos/Ratatui.md | 134 | `- [[Profile/Professional/Interests\|Interests]] — Rust foundational libraries` |
| Projects/Open Source Contributions/Repos/Tauri.md | 150 | `- [[Profile/Professional/Interests\|Interests]] — local-first product engineering as career territory` |
| Projects/Open Source Contributions/Repos/Tinygrad.md | 150 | `- [[Projects/Open Source Contributions/Repos/Tract\|Tract]] / [[Projects/Open Source Contributions/Repos/Candle\|Candle]] — ONNX op work transfers from this LSTM experience` |
| Projects/Open Source Contributions/Repos/Tokio.md | 196 | `- [[Profile/Professional/Interests\|Interests]] — Rust async / foundational systems` |
| Projects/Open Source Contributions/Repos/Tract.md | 154 | `- [[Profile/Professional/Interests\|Interests]] — edge ML / on-device inference as career territory` |
