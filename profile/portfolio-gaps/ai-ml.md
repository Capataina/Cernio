---
title: Portfolio Gaps — AI/ML
lane: ai-ml
last_updated: 2026-06-01
seed_source: grade-jobs Phase 3 (2026-06-01) — 358 graded jobs
distribution: { S: 25, A: 87, B: 55, C: 81, F: 110 }
---

# Portfolio Gaps — AI/ML lane

Regenerated from the 358-job ai-ml grading pool (2026-06-01). Pool spans Anthropic / Meta / Waymo / Wayve / Anthropic Fellows / DeepMind-pedigree spinouts (Hiverge, Latent Labs, Magentic) / Arm AI/ML / Doubleword / PhysicsX / Granola / Speechmatics / ElevenLabs / Encord / SKL Robotics / Scale AI / Graphcore / PostHog / Cognition / Wayve. Keyword density across the pool: PyTorch (61), GPU (71), CUDA (13), Triton (14), JAX (6), TPU (31), RLHF (38), distributed training (15), Kubernetes (16), fine-tuning (22), evals (65), agentic (233), embeddings/vectorDB (20), diffusion (18), interpretability/mech-interp (68), publications/papers (75).

---

## Open gaps

Ranked by S/A-tier frequency × landability lift.

### 1. Distributed training fluency — PyTorch DDP / FSDP / DeepSpeed / Megatron-LM
**Surfaced by:** Anthropic RE Pretraining (id=71, 72), Anthropic TPU Kernel Engineer (id=86), Anthropic Performance Engineer (id=64), Anthropic Fellows ML Systems & Performance (id=55), Anthropic Environment Scaling (id=65), Periodic Labs Distributed Training Engineer (id=706), PhysicsX MLSE Research (id=714 — explicitly names "data parallelism, parameter server… multi-node/multi-GPU"), Plumerai (id=730), Recraft (id=819), Graphcore Triton/PyTorch/ML Kernels (id=380, 382, 383), SKL Robotics NN Performance (id=861), Scale AI (id=870, 874), Speechmatics ×4 (id=903–906), Synthesia Infra (id=987).
**Count in S/A:** ~30 roles name distributed training, multi-node GPU, or framework-internals fluency as a stack expectation. 15 explicit "distributed training" mentions in the pool.
**Profile reality:** `projects/neurodrive.md` proves end-to-end ML systems with custom GEMM backends and a 21× perf engineering story on M2 — but on a single Apple Silicon machine, not on >1 GPU. No PyTorch DDP / FSDP / DeepSpeed / Megatron / `torch.distributed` evidence anywhere in `skills.md`. Closes the "I can build ML, but at single-host scale" gap that every pinnacle-lab Research Engineer role implicitly tests.
**Closure cost:** ~2–3 weeks. Train a from-scratch small transformer on 2× GPU (rented A10 / 4090 box or Colab) using `torch.distributed.DistributedDataParallel`; document the DDP-vs-DataParallel choice, gradient bucketing, and per-rank seeding. Then redo with `FSDP` on a >1B-param checkpoint. Outcome: a third project alongside NeuroDrive that does for distributed training what NeuroDrive does for single-host RL.

### 2. NVIDIA accelerator stack — CUDA / Triton / cuBLAS / TensorRT / vLLM
**Surfaced by:** Anthropic TPU Kernel Engineer (id=86), Anthropic Performance Engineer (id=64), Graphcore 2026 Graduate — ML Kernels & Runtime (id=380), Graphcore PyTorch (id=382), Graphcore Triton (id=383), Doubleword/TitanML Batched Inference (id=1214), Doubleword/TitanML LLM Inference Systems (id=1215), Arm AI/ML SE (id=1188), SKL Robotics NN Performance (id=861), Plumerai Deep Learning Research Engineer (id=730), Recraft (id=819), Nscale Specialised AI Engineer (id=654).
**Count in S/A:** 14 Triton mentions, 13 CUDA mentions, 18 inference-server / vLLM / TensorRT mentions.
**Profile reality:** `projects/neurodrive.md` has Apple AMX FFI via Accelerate with a SIMD fallback — exceptional evidence for *Apple* accelerator engineering, but Apple AMX is non-transferable signal for NVIDIA-stack hiring. `projects/image-browser.md` lists CUDA as "target-gated on non-macOS with CPU fallback" — built but not exercised. ONNX Runtime experience is portable; raw CUDA kernel authoring is not present.
**Closure cost:** ~3–4 weeks. Write one CUDA kernel (matmul tiled, or softmax fused) and one Triton kernel (same op, attention-style) with benchmark vs cuBLAS/PyTorch reference and a writeup. The artefact pattern is the same as NeuroDrive's GEMM backend — "I can author the kernel, not just call the library". Targets: Doubleword MTS, Anthropic Performance/TPU Kernel, Graphcore Triton/Kernels graduate roles.

### 3. RLHF / preference modelling / reward modelling pipelines
**Surfaced by:** Anthropic RE Machine Learning RL (id=67), Anthropic RL Velocity (id=68, 69), Anthropic Reward Models Platform (id=74), Anthropic Pretraining Scaling (id=72), Anthropic Environment Scaling (id=65), Anthropic Fellows RL (id=56), Lovable Labs Researcher Post-Training (id=599), JetBrains Founding ML Engineer (id=536), JetBrains Research Engineer Kineto (id=542), SKL Robotics RL Manipulation / Locomanipulation (id=847, 850), Recraft (id=819).
**Count in S/A:** 38 RLHF/RL mentions across S/A.
**Profile reality:** RL is `Proficient` in `skills.md` — handwritten PPO, GAE, PopArt, GNN-SAC. **This is the strongest single concept in the portfolio.** The gap is specifically *preference/reward-model* RL: paired-comparison datasets, Bradley-Terry loss, KL-regularised PPO against a reference policy, reward hacking diagnostics — the LLM-flavour of RL rather than the control-flavour. NeuroDrive RL is dense-reward driving simulation, not preference learning over text.
**Closure cost:** ~2 weeks. Reproduce a small RLHF loop — TRL or trlx on a 1B-param model with the Anthropic HH-RLHF dataset — and document the divergence between dense-reward PPO (NeuroDrive) and KL-regularised RLHF-PPO. Strongest single lever: **NeuroDrive already proves the RL substrate, so a small RLHF reproduction project is a one-step extension, not a from-scratch effort.** This unlocks the entire Anthropic RL-stream funnel (id=56, 67, 68, 69, 74).

### 4. JAX / TPU pipeline
**Surfaced by:** Anthropic TPU Kernel Engineer (id=86) explicitly, plus the broader DeepMind/Anthropic/Google pipeline assumption.
**Count in S/A:** 31 TPU mentions, 6 JAX mentions.
**Profile reality:** Zero JAX evidence in `skills.md`; PyTorch is `Familiar`, PyTorch Geometric is `Familiar`. JAX is the differentiator for DeepMind / Anthropic-TPU / Isomorphic-Labs / Latent-Labs (AlphaFold-pedigree) pipelines specifically.
**Closure cost:** Low — port one NeuroDrive PPO step from PyTorch-equivalent into JAX with `jit`/`vmap`/`pmap`, document the shape vs eager difference. Even a 200-line proof-point is sufficient to claim JAX `Familiar` honestly. This unlocks the AlphaFold-pedigree spinout funnel (Hiverge id=440, Latent Labs id=579) and the Anthropic TPU stream (id=86).

### 5. Production LLM eval frameworks — published harness pattern
**Surfaced by:** Granola AI Engineer (id=369), PostHog AI Research Engineer (id=753), JetBrains Research Engineer Agentic Models (id=541), JetBrains Kineto (id=542), Cognition Special Projects (id=158, 161), Anthropic Model Evaluations (id=70), Mantic LLM Agent (id=1312), HashiCorp Applied AI Vault (id=1316), Gradient Labs (id=363), DRW AI Engineer (id=189).
**Count in S/A:** 65 explicit "evals" / "evaluation framework" mentions. This is the second-most-cited concept after "agentic".
**Profile reality:** `projects/tessarix.md` has empirical hallucination testing across 4 candidate Ollama models — this is real eval work, but framed as project-internal model selection rather than as a reusable harness with a published artefact. `projects/consilium.md` has multi-LLM debate with structured-state validators. Neither anchors a *named* eval framework the way `lm-evaluation-harness`, `OpenAI evals`, `inspect-ai`, or `promptfoo` do.
**Closure cost:** ~1 week. Extract the Tessarix empirical-hallucination test into a standalone open-source repo (`tessarix-evals` or similar) with a JSON-schema-defined task spec, scoring rubric, and reproducible CLI — pitch as "the eval harness Tessarix used in production". Combined with one PR upstream into `inspect-ai` or `lm-evaluation-harness`, this lands a published eval-engineering artefact.

### 6. Agentic-systems portfolio anchor
**Surfaced by:** Cognition Special Projects (id=158, 161), Magentic Labs FDE/Product Engineer (id=624, 626), Mantic Technologies LLM Agent (id=1312), Lovable Labs (id=599–602), Granola (id=369–373), JetBrains Agentic Models (id=541), Encord Physical AI (id=297), Palantir FDE AI (id=688), Redpanda AI Agent (id=826), Parity (id=703).
**Count in S/A:** 233 mentions of "agent" / "agentic" — by far the dominant theme of the lane.
**Profile reality:** `projects/consilium.md` (multi-LLM orchestration) is the closest anchor but is `dormant` per project status. `projects/cernio.md` has parallel-agent orchestration (`discover-companies`, `grade-jobs`) but the agents are Claude Code sub-agents — not LLM-driven autonomous agents with tool use / browsing / planning loops. Tessarix has three LLM commands but no agentic loop.
**Closure cost:** ~2–3 weeks. Build one substantial autonomous-agent project: tool-using ReAct/PAL loop, structured action space, an environment (browser, code execution, or filesystem) with reward signal. Pattern after NeuroDrive's discipline — from-scratch loop, not an `langgraph`/`crewai` wrapper. This is the highest-leverage single addition for the lane: 233 agent mentions × the project would be the lane's anchor.

### 7. HuggingFace ecosystem depth beyond consumer use
**Surfaced by:** Plumerai (id=730), Recraft (id=819), Speechmatics (id=903–906), Scale AI ML Fellow (id=870), TikTok Multimodal LLM (id=1335).
**Profile reality:** `tokenizers = "0.22.2"` in Image Browser is the only HF-stack evidence. No `transformers`, `datasets`, `accelerate`, `peft`, `trl`, or `diffusers` evidence — all of which are table-stakes for fine-tuning-flavoured roles.
**Closure cost:** Folds into closures #1 (DDP), #3 (RLHF), and #5 (evals). Each of those naturally exercises a different HF library; producing all three closes this gap implicitly.

### 8. Diffusion-model engineering
**Surfaced by:** Recraft (id=819), Latent Labs (id=579 — "proprietary diffusion and reasoning models"), ElevenLabs Research (id=276), Speechmatics FutureVoices (id=905, 906), Sony Interactive Game Tech (id=897, 899).
**Count in S/A:** 18 diffusion mentions.
**Profile reality:** Zero diffusion-model evidence. Closure is only worth doing if explicitly targeting Latent Labs / Recraft / ElevenLabs Research — not a blanket-lever the way DDP or RLHF is.

### 9. Mechanistic interpretability research signal
**Surfaced by:** Anthropic Fellows AI Safety (id=53), Anthropic Universes (id=77), Anthropic Knowledge Team (id=66), Anthropic Alignment Science (id=76), Anthropic Model Evaluations (id=70). Anthropic Fellows program explicitly expects "a public output (e.g. a paper submission)" — 80% of fellows produce papers.
**Count in S/A:** 68 interpretability / mech-interp mentions.
**Profile reality:** Zero interpretability evidence. This is the gating gap for the *research-track* Anthropic Fellows / DeepMind Scholars / Cohere Research Internship pipelines, not the engineering-track ones. The engineering-track Anthropic streams (id=55, 67, 82, 83) do *not* need this.

---

## Confirmed strengths

Each strength is mapped to specific S/A roles that named it.

### A. End-to-end RL from first principles — `Proficient`
**Anchor:** `projects/neurodrive.md` (active) — handwritten PPO with no external ML libs; asymmetric 2×64 actor / 2×128 critic; GAE γ=0.995 / λ=0.95; clipped surrogate ε=0.2; tanh-squashed actions with Jacobian-correction; PopArt adaptive critic-target normalisation; AdamW decoupled weight decay; target-KL early stop; orthogonal init with 0.01× scaling. `projects/asteroidsai.md` (dormant) — GA + diagonal CMA-ES + NEAT speciation + GNN-SAC with twin critics + Polyak averaging + auto-entropy + AGC over `torch_geometric` GATv2Conv backbones. **Cross-method comparability on a single substrate (`population_evaluator.py`) is the differentiating evidence.**
**Maps to S/A roles:** Anthropic RE-ML-RL (id=67), Anthropic RL Velocity London (id=69), Anthropic Fellows RL (id=56), Anthropic Environment Scaling (id=65), Anthropic Reward Models Platform (id=74), SKL Robotics RL Manipulation / Locomanipulation (id=847, 850), Waymo MLE (id=1096), Wayve MLE (cited inside id=1096 reasoning), Sony Interactive ML (id=899), JetBrains Agentic Models (id=541). **This is the portfolio's load-bearing ai-ml signal.**

### B. ONNX Runtime production deployment + multi-encoder retrieval
**Anchor:** `projects/image-browser.md` (active) — three ONNX encoder families (CLIP ViT-B/32, DINOv2-Base, SigLIP-2 Base 256) with M2-tuned `Session` (Level3 + `intra_threads(4)` + `inter_threads(1)` + Phase 12 dynamic intra-split), CoreML-disabled-for-transformer-ops finding, output-name defensive cascade (`text_embeds → pooler_output → sentence_embedding`), real-input pre-warm eliminating first-call latency spike, `embedding_pipeline_version` migration wiping legacy rows on bump, Reciprocal Rank Fusion per Cormack-Clarke-Büttcher SIGIR 2009 with `k_rrf = 60`, single-flight orchestration with RAII `RunningGuard(Drop)`.
**Maps to S/A roles:** Doubleword/TitanML Batched Inference (id=1214), Doubleword/TitanML LLM Inference Systems (id=1215), Arm AI/ML SE (id=1188), Meta SWE-ML (id=1233), TikTok Multimodal LLM (id=1335), Monzo ML Platform (id=644), Plumerai (id=730), Recraft (id=819), SKL Robotics NN Performance (id=861), Vivacity Edge AI CV (id=1091), Encord Physical AI (id=297). **CLIP + DINOv2 + SigLIP-2 in production is the literal stack TikTok Multimodal LLM Graduate (id=1335) hires for — strongest portfolio-to-role concept-fit in the entire batch per the grader's own words.**

### C. Multi-LLM orchestration + structured LLM I/O
**Anchor:** `projects/tessarix.md` (active) — three Tauri commands (`llm_chat_complete`/`llm_chat_stream`/`llm_chat_json`); SSE streaming via `tauri::ipc::Channel<StreamEvent>` + `futures-util::StreamExt`; JSON-schema-constrained mode; empirical hallucination testing across `llama3.2:3b` vs `llama3.2:1b` vs `qwen2.5:3b` vs `gemma2:2b` with explicit-loser elimination; temperature 0.2 / top_p 0.9 / ban-list anti-evaluative contract. `projects/consilium.md` (dormant) — provider-agnostic LLM abstraction (`langchain_ollama` + `langchain_google_genai` adapters); three-state compose → run → result flow; 12.6 KB strict-key/strict-slot structured-state validator with tolerant per-field coercion.
**Maps to S/A roles:** Granola AI Engineer (id=369), Granola Product Engineer Full Stack/Backend (id=372, 373), Mantic Technologies LLM Agent (id=1312), PostHog AI Research Engineer (id=753), HashiCorp Applied AI Vault (id=1316), Magentic Labs FDE/PE (id=624, 626), Cognition Special Projects (id=158, 161), Lovable Labs Enterprise (id=601), Palantir FDE AI (id=688), DRW AI Engineer (id=189), Latent Labs FDAE (id=579), Luminance AI Engineer (id=602), Bending Spoons Graduate AI SWE (id=1428).

### D. ML-systems performance engineering with custom accelerator backends
**Anchor:** `projects/neurodrive.md` (active) — dual GEMM backend with Apple Accelerate AMX FFI vs SIMD fallback; flat row-major weight storage replacing `Vec<Vec<f32>>` cache-miss pattern; pre-allocated `BatchScratch` for zero training-loop allocations; iterator-based inner loops for LLVM auto-vectorisation; 21× frame-time improvement on M2 documented end-to-end. `projects/image-browser.md` `select_nth_unstable_by` partial-sort O(N) replacing O(N log N) — 2.53× measured speedup at N=10000 (audit `c6551e2`).
**Maps to S/A roles:** Anthropic Fellows ML Systems & Performance (id=55), Anthropic Performance Engineer (id=64), Anthropic TPU Kernel Engineer (id=86), Doubleword/TitanML (id=1214, 1215), Arm AI/ML SE (id=1188), Graphcore Kernels/PyTorch/Triton (id=380, 382, 383), SKL Robotics NN Performance (id=861), Plumerai (id=730).

### E. Tool-platform / internal-tooling engineering for AI orgs
**Anchor:** `projects/cernio.md` (active) — 14 k LOC Rust, lib+bin split, 6 ATS provider fetchers, 5-table SQLite with WAL, modular Ratatui v5 across 26 files, 346 tests, 9 native Claude Code skills with mandatory-read protocols and Tier-3 evidence-anchored quality checklists, parallel-agent dispatch architecture. `projects/tessarix.md` (active) — Tauri 2 + React 19 + MDX + KaTeX learning platform.
**Maps to S/A roles:** Anthropic Safeguards Foundations Internal Tooling (id=82), Anthropic Safeguards Infrastructure (id=83), Anthropic Applied AI Engineer (id=59), Anthropic Data Engineer Safeguards (id=61), Hiverge Product Engineer (id=440), Lovable Labs SE Enterprise (id=601), JetBrains roles (id=536, 541, 542), Encord Product Engineer (id=298), CuspAI Product Engineer (id=186, 187), V7 Labs Product Engineer (id=1049), Scale AI Software Engineer Enterprise (id=874). **This is what makes the Anthropic non-research SWE funnel (id=82, 83) realistically landable.**

### F. Open-source ML-infra engagement track
**Anchor:** Burn PR #4894 (+1864 LOC across 10 files) APPROVED — A-FINE no-reference IQA metric with inlined CLIP ViT backbone + PyTorch-weight loader + 5 evaluator heads + end-to-end regression tests. Burn PR #4938 draft. tinygrad PR #16119 minimal LSTM (line-budget-aware after #15453 postmortem closed for `+78 lines is too much`). `Capataina/OpenSourceContributions` private umbrella across 9 vetted Rust + ML-infra upstreams.
**Maps to S/A roles:** Graphcore PyTorch / Triton / Kernels Graduate (id=380, 382, 383), Doubleword (id=1214, 1215), Arm AI/ML SE (id=1188), Bending Spoons Graduate AI SWE (id=1428), Plumerai (id=730). **Burn-approved-PR is the kind of signal Anthropic Fellows / DeepMind Scholars committees explicitly look for ("public output… paper submission").**

---

## Pinnacle anchors

### Anthropic (engineering track) — id=55, 67, 69, 75, 82, 83 + A-tier id=53–86 family
**What lands:** Strengths A (RL Proficient) + D (ML-systems performance) + E (internal tooling) + F (OSS-engagement) directly anchor:
- **RE-ML-RL (id=67), RL Velocity London (id=69), Fellows RL (id=56)** — NeuroDrive handwritten PPO is exactly the differentiating evidence vs typical RE applicants who use stable-baselines3.
- **Fellows ML Systems & Performance (id=55), Performance Engineer (id=64), TPU Kernel Engineer (id=86)** — NeuroDrive dual GEMM backend + Nyquestro HDR-histogram tail-latency tracking + Image Browser ONNX Session tuning is the systems-performance triple.
- **Safeguards Foundations Internal Tooling (id=82), Safeguards Infrastructure (id=83)** — Cernio (14 k LOC Rust internal-tooling platform) is the literal portfolio anchor; software-engineering filter is meaningfully narrower than RE filter.
- **Anthropic Fellows AI Safety (id=53), Fellows AI Security (id=54)** — landable via Burn-approved-PR OSS-engagement signal.

**What's missing for Research Engineer (research-track) specifically:**
- **Zero published papers.** The Fellows program explicitly notes 80% of fellows produce paper submissions; Anthropic RE-Research track presumes either a publication record, a co-authored arxiv preprint, or a mech-interp / RLHF reproduction artefact that reads like a paper.
- **Zero mech-interp evidence.** No SAEs, no activation steering, no probing classifiers, no circuit analysis. This is the single research-track gating gap.
- **Zero RLHF reproduction.** The Anthropic RE-RL stream (id=67, 69) reads NeuroDrive favourably, but a *reward-model-trained PPO* artefact (Bradley-Terry loss, KL-regularised against reference policy) would be the precise bridge from "RL from first principles" to "RLHF as Anthropic does it".

**Concrete prescription:** A single 2-week project — small-model RLHF reproduction on the HH-RLHF dataset using TRL — closes both the RLHF gap and the published-artefact gap if pushed to a public repo with a writeup, and unlocks five S-tier Anthropic roles simultaneously.

### DeepMind / DeepMind-pedigree spinouts — Hiverge (id=440), Latent Labs (id=579)
**What lands:** Strength A (RL Proficient) and Strength F (OSS) read. Strength C (multi-LLM orchestration) lands for Hiverge's program-synthesis platform engineering.
**Missing:** JAX experience for any genuine DeepMind core role; diffusion-model evidence for Latent Labs Research track. Forward-Deployed AI Engineer at Latent Labs (id=579) is the most-accessible path — does not require JAX or diffusion, only engineering + customer-integration competence, which lands cleanly today.

### Meta AI (id=1233, 1234)
**What lands:** Strength B (ONNX in production), Strength D (perf engineering), Strength F (OSS PRs). Meta SWE-ML at IC1-IC2 band is realistically landable via portfolio depth in the standard SWE pipeline.
**Missing:** Distributed-training proof-point. Meta's ML SWE pipeline assumes PyTorch fluency end-to-end including distributed; single-host PyTorch (`Familiar` in skills.md) doesn't yet match the implicit expectation.

### OpenAI / xAI
**No graded London roles in this batch.** Not actively recruiting in the lane's sponsorship-capable UK geography at present. The closure pattern that lands Anthropic engineering-track will transfer if/when these open.

### Waymo (id=1096) / Wayve
**What lands:** Strength A (RL Proficient — NeuroDrive driving simulation is on-axis). NeuroDrive's empirical validation (8/8 cars complete Monaco loop, 96% of crashes anticipated by pre-impact throttle release >0.25s) reads as the AV-engineering analogue.
**Missing:** No commercial AV experience; the gap is closeable only by the role itself.

### Anthropic Fellows engineering streams + Cohere / Mistral / Adept
The Fellows ML-Systems-and-Performance stream (id=55) is the single best engineering-track entry into the frontier-AI-lab funnel because it explicitly accepts portfolio-only candidates ("regardless of previous experience") on the ML-systems axis where the portfolio is strongest.

---

## Closure prescriptions — ranked

1. **RLHF reproduction project** (~2 weeks). Closes Open Gap #3, contributes to Open Gap #5, unlocks 5+ Anthropic S-tier roles. Single highest-leverage lever in the lane.
2. **Distributed-training proof-point** (~2–3 weeks). Closes Open Gap #1. Unlocks Anthropic Performance/TPU streams, PhysicsX MLSE Research, Periodic Labs, Graphcore graduate ML roles. Lowers the implicit floor across most Meta / Waymo / Wayve roles.
3. **One CUDA + one Triton kernel** with benchmark writeup (~3–4 weeks). Closes Open Gap #2. Unlocks Doubleword MTS, Graphcore Triton/Kernels, Anthropic Performance/TPU Kernel, Arm AI/ML SE, SKL NN Performance, Plumerai. Aligns the existing NeuroDrive GEMM-backend evidence with the NVIDIA-stack hiring world.
4. **One autonomous LLM-agent project** (~2–3 weeks). Closes Open Gap #6. 233 agent mentions in the pool make this the lane's biggest single-addition lever for *breadth* even though RLHF is biggest for *Anthropic-specifically*. Targets: Cognition, Magentic, Mantic, Lovable, Granola, JetBrains.
5. **Extract Tessarix evals into standalone harness + one upstream PR into `inspect-ai` or `lm-evaluation-harness`** (~1 week). Closes Open Gap #5. Unlocks PostHog AI RE, Granola AI, Anthropic Model Evaluations, JetBrains Kineto. Single-week win.
6. **Small JAX port of one NeuroDrive component** (~1 week). Closes Open Gap #4. Unlocks Anthropic TPU stream + DeepMind / AlphaFold-pedigree pipelines.
7. **NeuroDrive M6 substrate validation** (carried over from prior gaps file). The SideBySide behavioural acceptance run that has been gated since the 2026-04-30 NeuroDrive session — closes a known open thread without adding new scope.

**Sequenced ordering:** 1 → 5 → 6 → 2 → 3 → 4. RLHF first because it extends an existing `Proficient` skill and unlocks the densest pinnacle cluster (Anthropic). Evals + JAX next because each is ~1 week and adds named ecosystem fluency. Distributed training + CUDA/Triton are the larger time investments and gate the systems-performance pinnacle roles. Agents project last because it lands a broad set of B/A-tier startups but doesn't unlock S-tier the way #1 does.

---

## Lane internal calibration — 358-job pool

**Tier distribution:**

| Tier | Count | Share |
|------|-------|-------|
| S | 25 | 7.0% |
| A | 87 | 24.3% |
| B | 55 | 15.4% |
| C | 81 | 22.6% |
| F | 110 | 30.7% |

**Top of S band (engineering-accessible, on-axis, sponsor-capable):**

| ID | Company / role | Why it tops the band |
|----|----------------|----------------------|
| 1335 | TikTok ML Engineer Graduate — CV/NLP/Multimodal LLM | Strongest portfolio-to-role concept-fit in the entire batch (CLIP+DINOv2+SigLIP-2 is the literal stack) + wide-funnel Graduate programme + Tier 1 sponsor |
| 67, 69 | Anthropic RE-ML-RL / RL Velocity London | NeuroDrive handwritten PPO is the differentiating evidence vs stable-baselines3 applicants; London Tier 1 |
| 82, 83 | Anthropic Safeguards Foundations / Infrastructure (SWE) | SWE filter narrower-than-RE; Cernio is the literal internal-tooling-platform portfolio anchor |
| 55 | Anthropic Fellows ML Systems & Performance | Stream-specific filter pulls toward systems-engineering profiles; Fellows accepts portfolio-only |
| 1214, 1215 | Doubleword/TitanML MTS Batched Inference / LLM Inference Systems | Portfolio-perfect cross-lane bridge (ai-ml × systems-infra); London hybrid; sponsor verified |
| 1233 | Meta SWE-ML | IC1-IC2 portfolio path realistic; brand routes everything afterwards |
| 753 | PostHog AI Research Engineer | Devtools + ai-ml double-pinnacle; Tessarix + Cernio direct anchor; remote-first UK |
| 1316 | HashiCorp Applied AI Vault | Triple-axis (ai-ml + systems-infra + devtools); confirmed sponsor |
| 1188 | Arm AI/ML SE | Cambridge Tier 1; ONNX-on-hardware portfolio match exact |

**Biggest single-addition lever:** RLHF reproduction (~2 weeks) — promotes Caner from "ai-ml engineering candidate" to "ai-ml engineering candidate Anthropic recruiters actively call back". The portfolio is already inside the Anthropic engineering-track realistic primary-target pool today; RLHF closes the specific gap that lets the Fellows committee read NeuroDrive as RLHF-ready rather than RL-adjacent.

**Lane band note:** Default ai-ml lane assignment remains Engineering unless the JD explicitly names Research / Scientist function. The Research-Engineer-research-track filter (id=70, 71, 73, 76, 77, 715) is materially narrower than the Research-Engineer-engineering-track filter (id=55, 67, 69, 82, 83); the personality grades both but the closure prescriptions weight engineering-track because that is where the portfolio currently lands cleanly.

---

**Gaps:** distributed training (DDP/FSDP/DeepSpeed/Megatron), NVIDIA accelerator stack (CUDA/Triton/cuBLAS/TensorRT/vLLM), RLHF/preference modelling, JAX/TPU, production eval frameworks, autonomous-agent project, HuggingFace ecosystem depth, diffusion models, mechanistic interpretability. **Strengths:** RL from first principles (Proficient, NeuroDrive — load-bearing lane signal), ONNX + multi-encoder retrieval (Image Browser CLIP+DINOv2+SigLIP-2 + RRF), multi-LLM orchestration (Tessarix + Consilium), ML-systems performance engineering (NeuroDrive AMX dual backend + 21× perf story), internal-tooling platforms for AI orgs (Cernio 14 k LOC + 9 skills), OSS ML-infra engagement (Burn PR #4894 APPROVED + tinygrad PR + Capataina/OpenSourceContributions umbrella). **Key recommendation:** ship a 2-week RLHF reproduction project on HH-RLHF using TRL — it extends NeuroDrive's existing `Proficient` RL substrate by one step, closes the single Anthropic-funnel-gating gap, contributes a published artefact toward the Fellows-paper expectation, and unlocks five S-tier Anthropic roles (id=56, 67, 69, 74, 75) simultaneously.
