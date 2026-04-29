---
name: Open Source Contributions
status: active
source_repo: null
lifeos_folder: Projects/Open Source Contributions
last_synced: 2026-04-29
sources_read: 3
---

# Open Source Contributions

## One-line summary

Per-upstream notes on Caner's external open-source engagement — `tinygrad/tinygrad` ONNX LSTM operator (PR #15453, closed on `sz.py` line-budget despite technical correctness) and `tracel-ai/burn` A-FINE no-reference image-quality metric (issue #4312, claim re-engaged 2026-04-24 after 25-day silence with maintainer-confirmed inlined-CLIP-ViT implementation in progress).

## What it is

This file aggregates Caner's external open-source engagement across multiple upstream repositories. It captures repo culture, review protocols, contribution history, what landed and what was rejected. Source: every file in `Projects/Open Source Contributions/` in LifeOS, including `_Overview.md`, `Tinygrad.md`, and `Burn.md`. Two upstreams currently tracked. The resume's Open Source Contributions section anchors here.

---

## tinygrad (tinygrad/tinygrad) — closed

### Repo culture

tinygrad is a minimalist deep-learning framework built by George Hotz / tiny corp. It is governed by an aggressive **line-budget philosophy** — low total line count is treated as a first-class metric for the core `tinygrad/` folder (tests don't count against it). Enforced by:

- A bot comment on every PR reporting line-count delta via `sz.py` (tokenised, excluding docstrings).
- Reviewers who close PRs on diff size alone regardless of technical correctness.
- Written policy in the README "Contributing" section: *"If your PR looks complex, is a big diff, or adds lots of lines, it won't be reviewed or merged."*

### Review dynamics

| Reviewer | Role | Signal |
|---|---|---|
| **chenyuxyz** | Core collaborator, prolific reviewer | Closes large-diff PRs with one-line comments ("+78 lines is too much"). Enforces line budget strictly and quickly. |

### Contribution: ONNX LSTM operator (PR #15453)

| Field | Value |
|---|---|
| PR | tinygrad/tinygrad#15453 |
| Linked issue | #10897 ("onnx lstm op not implemented") |
| Branch | `feat/onnx-lstm-support` → `master` |
| Commit | `437de5a` ("Add ONNX LSTM support") |
| Opened | 2026-03-24 18:10 UTC |
| Closed | 2026-03-25 00:38 UTC (~6h 28m alive) |
| Closed by | chenyuxyz |
| Stated reason | "+78 lines is too much" |
| Diff | +146 / −0 across 2 files |

**Diff composition:**

| File | Added | What |
|---|---|---|
| `tinygrad/nn/onnx.py` | +90 | `LSTM(...)` op + `_apply_rnn_activation(...)` helper |
| `test/external/external_test_onnx_ops.py` | +56 | `helper_test_lstm` + 4 test cases |

The `sz.py` bot reported **+78 tokenised core lines** — tests are not counted against the core budget.

### What the implementation covered

- **Directions**: forward, reverse, bidirectional (concatenating along `num_directions` dim).
- **Gate decomposition**: proper i/o/f/c split of `W`, `R`, `Wb`, `Rb` per direction.
- **Optional inputs defaulted to zero tensors** when absent (`B`, `initial_h`, `initial_c`).
- **Explicit `NotImplementedError`** for unsupported features: `sequence_lens`, peephole weights `P`.
- **Attributes handled**: `activation_alpha`, `activation_beta`, `activations`, `clip`, `direction`, `hidden_size`, `input_forget`, `layout` (0 / 1 with permute).
- **`input_forget=1` → `f_t = 1.0 - i_t`** coupled-input-forget-gate shortcut.
- **Gate clipping** applied to all four pre-activations before activation.
- **Outputs**: `Y` (full sequence), `Y_h`, `Y_c`, with layout permutation when `layout=1`.
- **`_apply_rnn_activation` helper** supporting all 11 ONNX RNN activations.

**Test coverage**: forward basic, forward with initial state and all outputs, reverse, bidirectional.

### Issue context

Originally opened 2025-06-20 by Quantizr (Jimmy) with a `NotImplementedError: op='LSTM' not supported` from loading Silero VAD v5 through tinygrad's ONNX frontend. chenyuxyz responded: *"probably fine to add. can you link an onnx model that has LSTM op that we can test?"* Caner commented "can I try this?" on 2025-07-28. PR submitted ~8 months later.

### Lessons learned (tinygrad)

- **Line budget is the first-order filter** — a technically correct +90-line core change will be closed.
- **Prerequisite-refactor pattern is the cultural norm** — *"If you can (cleanly) refactor to the point that the feature is a 3 line change, this is great."*
- **Use `Fixes #N` syntax** in PR body so linked issues auto-close.
- **Features must have regression tests, but test lines don't count against the budget.**
- **Eight months between claim and submission is too long** — chenyuxyz had moved on; nobody was holding the issue.

### Path to re-submission

If re-attempting LSTM, the playbook that matches tinygrad's culture:

1. **PR 1 — primitive refactor**: extract shared helper patterns (gate splitting, per-direction dispatch). Small, 3–10 lines, clear-win refactor.
2. **PR 2 — `_apply_rnn_activation` as a dict lookup**: collapse the 15-line if/elif chain to ~6 lines via dispatch table.
3. **PR 3 — forward-only LSTM using the helpers**: minimal LSTM, forward mode only, 3 canonical activations (Sigmoid, Tanh, Relu).
4. **PR 4 — reverse + bidirectional**: add reverse and bidirectional modes on top.

Each PR individually small, individually a clear win, individually easier to review than a single +146 monolith.

---

## tracel-ai/burn — active implementation

### Repo culture

`burn` is tracel-ai's Rust-first deep-learning framework. It has a visible, **active meta-issue culture** — maintainers post checklists of feature items, community contributors pick them off one by one, each merge gets linked back into the checklist. Review cadence is fast and friendly; most items on the image-quality metrics checklist (#4312) have been claimed, implemented, and merged within a fortnight.

### Maintainers and reviewers

| Login | Role | Signal |
|---|---|---|
| **laggui** | MEMBER (tracel-ai core) | Triages meta-issues; confirms claims within hours; gives implementation hints (LPIPS / DISTS / Gram matrix loss as precedents; PyTorch weight import is supported pattern). Friendly, low-ceremony tone. |
| torsteingrindvik | CONTRIBUTOR | Opens and curates #4312 ("Image quality metrics"); proposes new metrics with paper citations and benchmark evidence. |
| softmaximalist, koreaygj, cong-or, kvthr | CONTRIBUTORs | Active checklist implementers (1–2 metrics per fortnight per person during active periods). |

### Community norms

- **Claim-then-implement pattern**: comment "I'll take on X" on the meta-issue, wait for MEMBER confirmation, then open the PR.
- **Precedent-driven implementation**: the meta-issue links merged PRs for every closed item. Pattern-matching from prior PRs is the dominant style.
- **PyTorch weight import is a supported, recurring pattern** — perceptual metrics (LPIPS, DISTS, Gram Matrix, FID) all load pretrained PyTorch weights into Rust.
- **Short, direct confirmations** — laggui's response to Caner's claim was *"Yeah it should be up for graps!"* [sic] — minimal process, high trust.
- **Long silences get re-engaged gracefully** — Caner broke a 25-day silence with an open, detailed technical message and laggui replied within ~4 days with substantive guidance.

### Contribution: A-FINE image-quality metric (issue #4312)

| Field | Value |
|---|---|
| Issue | tracel-ai/burn#4312 ("Image quality metrics") |
| Claim comment | comment #4128062726 — *"Can I tackle A-FINE?"* |
| Confirmation | laggui MEMBER — *"Yeah it should be up for graps!"* |
| Claimed | 2026-03-25 |
| Confirmed | 2026-03-25 |
| Re-engaged | 2026-04-19 — detailed technical questions after studying LPIPS / DISTS / FID / Gram precedent PRs, `burn-store`, PyIQA reference impl |
| Maintainer response | 2026-04-23 — laggui confirmed one-PR strategy matching the inlined-backbone pattern used by LPIPS / DISTS / FID |
| Implementation status | 2026-04-24 — Caner confirmed: *"Perfect, I've already started the implementation and everything seems to align well. I'll change the vit implementation to inline as well"* |
| Status | **Active — implementation in progress under the agreed approach (single PR with inlined CLIP ViT backbone)** |

**A-FINE** = Adaptive Fidelity-Naturalness Evaluator — a no-reference (blind) image-quality metric. Proposed on the meta-issue by torsteingrindvik on 2026-03-02.

**Upstream references:**

- Paper: arXiv 2503.11221.
- Project page: tianhewu.github.io/A-FINE-page.github.io/.
- Reference implementation: ChrisDud0257/AFINE — Apache 2.0 (compatible with burn's dual MIT / Apache-2.0).

### Scope and implementation strategy (locked 2026-04-23)

Per laggui's guidance:

- **Single PR**, not a backbone-first PR followed by A-FINE. Matches the inlined-backbone pattern used for LPIPS (#4403), DISTS (#4574), FID (#4644).
- **CLIP ViT inlined** into the A-FINE module rather than carved out as a reusable component. Future CLIP-based metric can refactor it out.
- **Port target**: `crates/burn-train/src/metric/vision/afine/` following the existing perceptual-metrics directory convention.
- Five A-FINE heads + loader + tests all ship together in the one PR.

### Precedent PRs relevant to A-FINE

| PR | Metric | Relevance |
|---|---|---|
| #4403 | LPIPS | Closest analogue — perceptual metric using pretrained features, inlined backbone pattern |
| #4574 | DISTS | Perceptual metric with PyTorch-weight import pattern |
| #4595 | Gram Matrix | PyTorch-weight import pattern for style/perceptual metric |
| #4644 | FID | Uses InceptionV3 pretrained weights — largest precedent for heavyweight pretrained dependency |

Lighter precedents for overall metric-crate structure: #4341 (L1/L2), #4379 (PSNR), #4396 (SSIM), #4547 (Smooth L1), #4555 (MS-SSIM).

### Lessons learned (burn)

- **A long silence survives if the comeback is technical depth rather than process apology** — Caner's 25-day-silence message showed the silence had been spent on due diligence (4 precedent PRs studied, `burn-store` read, PyIQA reference implementation read).
- **The less-costly path is still an interim status comment within the first 2 weeks** — silence-then-substance recovered cleanly here, but is not the default playbook.
- **Pattern-matching from precedent PRs is the dominant burn style** — the LPIPS / DISTS / FID inlined-backbone pattern is the template.

### Timing note

The A-FINE claim was posted ~16 hours after the tinygrad LSTM PR was closed overnight — the move from tinygrad to burn was direct and deliberate. burn is a better cultural match for Caner (Rust is a primary language, maintainers are active and friendly, the precedent checklist gives clear templates).

---

## Cross-Vault Connections

- The `Resume.md` Open Source Contributions section anchors here — every resume claim about external OSS work traces to one of the per-upstream files.
- Source-of-truth hierarchy: per-contribution files canonical for repo culture and contribution history; the repos themselves are external authority on PR/issue status; this aggregation is the durable narrative + lessons-learned synthesis.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Open Source Contributions/_Overview.md | 40 | "- [[Profile/Professional/Interests]] — OSS as an interest territory" |
| Projects/Open Source Contributions/Burn.md | 124 | "- [[Profile/Professional/Experience]] — counts as external open-source engagement with a Rust deep-learning framework maintainer team" |
| Projects/Open Source Contributions/Tinygrad.md | 130 | "- [[Projects/Open Source Contributions/Burn|Burn]] — sister contribution notes" |
