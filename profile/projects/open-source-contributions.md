---
name: Open Source Contributions
status: active
source_repo: null
lifeos_folder: Projects/Open Source Contributions
last_synced: 2026-04-26
sources_read: 2
---

# Open Source Contributions

Working record of contributions to external open-source projects. Each subsection covers one upstream repo, the contribution itself, the cultural context that shaped it, and the lessons that transfer to future contributions. Sourced from the corresponding `LifeOS/Projects/Open Source Contributions/<upstream>.md` working notes — anti-puffing applies: outcomes and statuses reflect what the upstream record states, not what the GitHub README would prefer.

---

## tinygrad — ONNX LSTM operator (closed)

### Upstream context

[tinygrad/tinygrad](https://github.com/tinygrad/tinygrad) is a minimalist deep learning framework built by George Hotz / tiny corp. It is governed by an aggressive **line-budget philosophy** — low total line count is treated as a first-class metric for the core `tinygrad/` folder (tests don't count against it). Enforcement mechanisms include a `sz.py` bot that comments on every PR with a tokenised line-count delta (excluding docstrings) and reviewers who close PRs on diff size alone, regardless of technical correctness. The README's "Contributing" section codifies this:

> "If your PR looks complex, is a big diff, or adds lots of lines, it won't be reviewed or merged."
> "If it's 3 lines, there's less of a bar of usefulness it has to meet over something that's 30 or 300 lines."

Reviewer dynamics observed: **chenyuxyz** (core collaborator) closes large-diff PRs with one-line comments enforcing the line budget strictly and quickly. What gets merged: 3-line features, prerequisite refactors before new features (refactor so the feature itself is a trivial addition), tests and fuzzers, dead-code removal. What gets closed: code golf for line-count cheesing, large diffs without prerequisite refactors, unbenchmarked "speedups", and changes outside `tinygrad/` core unless fixing something broken.

### The contribution

- **PR:** [tinygrad/tinygrad#15453](https://github.com/tinygrad/tinygrad/pull/15453) — "Add ONNX LSTM support"
- **Linked issue:** [#10897 "onnx lstm op not implemented"](https://github.com/tinygrad/tinygrad/issues/10897)
- **Branch:** `feat/onnx-lstm-support` → `master`
- **Commit:** [`437de5a`](https://github.com/tinygrad/tinygrad/commit/437de5aa5b935fca41e66841114554b41d3379a6)
- **Lifecycle:** Opened 2026-03-24 18:10 UTC, closed 2026-03-25 00:38 UTC by chenyuxyz with stated reason *"+78 lines is too much"* (~6h 28m alive)
- **Diff:** +146 / −0 across 2 files; `sz.py` bot reported +78 tokenised core lines (tests not counted)

### What the implementation covered

| File | Added | What |
|------|-------|------|
| `tinygrad/nn/onnx.py` | +90 | `LSTM(...)` op + `_apply_rnn_activation(...)` helper |
| `test/external/external_test_onnx_ops.py` | +56 | `helper_test_lstm` + 4 test cases |

Substantive scope:

- **Directions:** forward, reverse, bidirectional (concatenating along `num_directions` dim)
- **Gate decomposition:** proper i/o/f/c split of `W`, `R`, `Wb`, `Rb` per direction
- **Optional inputs** defaulted to zero tensors when absent (`B`, `initial_h`, `initial_c`)
- **Explicit `NotImplementedError`** for unsupported features: `sequence_lens`, peephole weights `P`
- **Attributes handled:** `activation_alpha`, `activation_beta`, `activations`, `clip`, `direction`, `hidden_size`, `input_forget`, `layout` (0 / 1 with permute)
- **`input_forget=1`** → `f_t = 1.0 - i_t` coupled-input-forget-gate shortcut
- **Gate clipping** applied to all four pre-activations before activation
- **Outputs:** `Y` (full sequence), `Y_h`, `Y_c`, with layout permutation when `layout=1`
- **`_apply_rnn_activation` helper** supporting all 11 ONNX RNN activations (Sigmoid, Tanh, Relu, ThresholdedRelu, ScaledTanh, Softsign, Softplus, Elu, HardSigmoid, Affine, etc.)

Test coverage: forward basic, forward with initial state and all outputs, reverse, bidirectional.

### Issue context

The issue was originally opened 2025-06-20 by **Quantizr (Jimmy)** with a `NotImplementedError: op='LSTM' not supported` stack trace from loading [Silero VAD v5](https://github.com/snakers4/silero-vad) through tinygrad's ONNX frontend. chenyuxyz responded with conditional approval. Caner commented "can I try this?" on 2025-07-28. The PR was submitted ~8 months later on 2026-03-24. The issue remains **open** — the PR body said *"This is a fix for the issue #10897"* (prose) but did not use `Fixes #10897` syntax, so GitHub would not have auto-closed the issue on merge anyway.

### Outcome and lessons

The PR was closed on **diff size alone**, not on technical grounds. Specific lessons captured in LifeOS:

- **Line budget is the first-order filter.** A technically correct +90-line core change will be closed. Writing for tinygrad means designing around the line budget from the start, not optimising after.
- **Prerequisite-refactor pattern is the cultural norm.** Land the refactor first; then the feature is a 3-line addition.
- **Use `Fixes #N` syntax** in the PR body so linked issues auto-close on merge. Prose mentions don't count.
- **Tests are free** — feature-required regression tests don't count against the core line budget, so use that allowance generously.
- **Eight months between claim and submission is too long.** Unassigned claims on GitHub are soft commitments at best.

A re-submission playbook is captured in LifeOS as four sequential PRs (primitive refactor → activation dispatch table → forward-only LSTM → reverse + bidirectional) — each small, each individually a clear win, each easier to review than a single +146 monolith.

---

## burn — A-FINE image quality metric (active implementation)

### Upstream context

[tracel-ai/burn](https://github.com/tracel-ai/burn) is tracel-ai's Rust-first deep learning framework. It has a visible, **active meta-issue culture** — maintainers post checklists of feature items, community contributors pick them off one by one, each merge gets linked back into the checklist. Review cadence is fast and friendly; most items on the image-quality metrics meta-issue have been claimed, implemented, and merged within a fortnight.

Maintainers and regular reviewers observed:

| Login | Role | Signal |
|-------|------|--------|
| **laggui** | MEMBER (tracel-ai core) | Triages meta-issues; confirms claims within hours; gives implementation hints. Friendly, low-ceremony tone. |
| **torsteingrindvik** | CONTRIBUTOR | Curates [#4312 "Image quality metrics"](https://github.com/tracel-ai/burn/issues/4312); proposes new metrics with paper citations and benchmark evidence. |
| **softmaximalist, koreaygj, cong-or, kvthr** | CONTRIBUTORs | Active metrics-checklist implementers. Cadence of 1–2 metrics per fortnight per person. |

Community norms:

- **Claim-then-implement pattern:** comment "I'll take on X" on the meta-issue, wait for MEMBER confirmation, then open the PR.
- **Precedent-driven implementation:** the meta-issue links merged PRs for every closed item — they are templates for follow-on work.
- **PyTorch weight import is a recurring pattern** for perceptual metrics (LPIPS, DISTS, Gram Matrix, FID all load pretrained PyTorch weights into Rust).
- **Short, direct confirmations** with minimal process and high trust.
- **Long silences get re-engaged gracefully** if you come back with substantive technical questions rather than apologies.

### The contribution

- **Issue:** [#4312 "Image quality metrics"](https://github.com/tracel-ai/burn/issues/4312)
- **Claim comment:** [#4128062726](https://github.com/tracel-ai/burn/issues/4312#issuecomment-4128062726) — *"Can I tackle A-FINE? seems like its proposed but not being worked on?"*
- **Confirmation:** [laggui MEMBER](https://github.com/tracel-ai/burn/issues/4312#issuecomment-4128993499) — *"Yeah it should be up for graps!"* [sic]
- **Lifecycle:** Claimed and confirmed 2026-03-25; re-engaged 2026-04-19 with detailed technical questions after studying LPIPS / DISTS / FID / Gram PRs, `burn-store`, and the PyIQA reference implementation; maintainer guidance received 2026-04-23 confirming the one-PR strategy with inlined backbone; implementation in progress 2026-04-24 onward.
- **Status:** **Active** — implementation in progress under the agreed approach (single PR with inlined CLIP ViT backbone).

**A-FINE** = Adaptive Fidelity-Naturalness Evaluator — a no-reference (blind) image quality metric. Proposed on the meta-issue by torsteingrindvik on 2026-03-02.

Upstream references:
- Paper: [arXiv 2503.11221](https://arxiv.org/pdf/2503.11221)
- Project page: [tianhewu.github.io/A-FINE-page.github.io](https://tianhewu.github.io/A-FINE-page.github.io/)
- Reference implementation: [ChrisDud0257/AFINE](https://github.com/ChrisDud0257/AFINE/tree/master) (Apache 2.0, compatible with burn's dual MIT / Apache-2.0)

### Implementation strategy (locked 2026-04-23)

Per laggui's guidance, the implementation approach is:

- **Single PR**, not a backbone-first PR followed by A-FINE. Matches the inlined-backbone pattern already used for LPIPS (#4403), DISTS (#4574), and FID (#4644).
- **CLIP ViT inlined** into the A-FINE module rather than carved out as a reusable component. A future CLIP-based metric can refactor it out.
- **Port target:** `crates/burn-train/src/metric/vision/afine/` following the existing perceptual-metrics directory convention.
- Five A-FINE heads + loader + tests all ship together in the one PR.

### Precedent PRs (the implementation template)

A-FINE uses pretrained perceptual features, so these merged burn PRs are the implementation template:

| PR | Metric | Relevance |
|----|--------|-----------|
| [#4403](https://github.com/tracel-ai/burn/pull/4403) | LPIPS | Closest analogue — perceptual metric using pretrained features, inlined backbone pattern |
| [#4574](https://github.com/tracel-ai/burn/pull/4574) | DISTS | Perceptual metric with PyTorch-weight import pattern |
| [#4595](https://github.com/tracel-ai/burn/pull/4595) | Gram Matrix | PyTorch-weight import pattern for a style/perceptual metric |
| [#4644](https://github.com/tracel-ai/burn/pull/4644) | FID | Uses InceptionV3 pretrained weights — largest precedent for heavyweight pretrained dependency |

Lighter-weight precedents for overall metric-crate structure: PSNR (#4379), SSIM (#4396), L1/MAE + L2/MSE (#4341), Smooth L1 (#4547), MS-SSIM (#4555).

### The 25-day silence and how it resolved

Twenty-five days passed between claim (2026-03-25) and re-engagement (2026-04-19) with no PR, draft, or status update — notable because other implementers on this meta-issue were shipping at 1–2 metrics per fortnight. What resolved the silence cleanly was that Caner came back with a substantive message showing the silence had been spent on due diligence (studying four precedent PRs, reading `burn-store`, reading the PyIQA reference implementation). The message was questions, not apologies — asking about PR size strategy and CLIP ViT inlining. That framing turned what could have been a credibility hit into an engineering conversation, and laggui responded with directional guidance.

Lesson for similar claims: a long silence survives if the comeback is technical depth rather than process apology. But the less costly path is still an interim status comment within the first two weeks.

### Timing note

The A-FINE claim was posted ~16 hours after the tinygrad LSTM PR was closed overnight — the move from tinygrad to burn was direct and deliberate. burn is a better cultural match: Rust is a primary language, the maintainers are friendly and active, and the precedent checklist gives clear templates.

### Next actions (in flight)

- Write the inlined CLIP ViT in burn's conventions (ported from the PyIQA reference)
- Implement the five A-FINE heads
- Port the loader (PyTorch-weight import path mirroring LPIPS / DISTS / FID)
- Write regression tests against reference outputs from the upstream AFINE repo
- Open a draft PR when CLIP ViT + at least one head compile and pass tests
- Ask for review once the full pipeline runs end-to-end

---

## What this evidences for grading

Two contrasting upstream cultures successfully navigated:

- **tinygrad** — a strict, line-budget-driven framework where the contribution failed *because of the culture, not the code*. Documented retrospective with a concrete re-submission playbook shows the user can read review culture explicitly and adapt.
- **burn** — a friendly, precedent-driven Rust framework where the contribution is on its agreed approach with maintainer buy-in. Demonstrates ability to re-engage cleanly after a silence with technical depth, and to land scope-locking conversations with maintainers before writing code.

Both contributions involved substantive deep-learning operator / metric work (LSTM / A-FINE), evidencing comfort with ONNX operator semantics, perceptual-metric architecture, and pretrained-weight import patterns across both Python and Rust ML ecosystems.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Open Source Contributions/Tinygrad.md | 130 | "- [[Projects/Open Source Contributions/Burn|Burn]] — sister contribution notes" |
| Projects/Open Source Contributions/Burn.md | 124 | "- [[Profile/Professional/Experience]] — counts as external open-source engagement with a Rust deep-learning framework maintainer team" |
