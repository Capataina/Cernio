---
title: Portfolio Gaps — AI/ML
lane: ai-ml
last_updated: 2026-05-29
seed_source: legacy profile/portfolio-gaps.md
---

# Portfolio Gaps — AI/ML lane

> Seeded from legacy portfolio-gaps.md. Will be overwritten by grade-jobs Phase 3 on next canonical run.

## Open gaps

- **Production-scale ML — petabyte / 10K-GPU / cloud-training.** Named at top AI labs (Anthropic infrastructure, OpenAI scaling). Distinct from "ML projects exist" — this is hyperscale training infrastructure specifically.
- **PyTorch Distributed + DeepSpeed + Megatron-LM** — distributed training framework fluency.
- **MLOps stack** — Kubeflow, MLflow, Weights & Biases at production scale; experiment tracking discipline.
- **Foundation-model fine-tuning at scale** — LoRA / PEFT / RLHF pipelines on >7B parameter models.
- **Specific HuggingFace ecosystem depth** — tokenizers, datasets, accelerate, transformers internals beyond consumer use.

## Confirmed strengths

- **End-to-end ML system construction** — NeuroDrive (handwritten PPO with no external ML libs; 8 Bevy plugins; dual GEMM backend with Accelerate AMX FFI).
- **ONNX-Runtime production deployment** — Image Browser (three encoder graph families, M2-tuned `Session` builder, real-input pre-warm eliminating first-call latency spike, embedding-pipeline versioning).
- **From-scratch evolutionary algorithms** — AsteroidsAI (GA, CMA-ES, NEAT speciation, GNN-SAC with PyTorch + torch_geometric).
- **CLIP / SigLIP-2 / DINOv2 production integration** — Image Browser (tokenizer plumbing for BPE + SentencePiece, output-name defensive cascade, embedding cache invalidation).
- **Multi-LLM orchestration** — Consilium (provider-agnostic abstraction, three-state compose/run/result flow, structured-state schema with strict-key validators).

## Closure prescriptions

1. **Hyperscale distributed training proof-point.** Train something non-trivial on >1 GPU with PyTorch Distributed; ideally a from-scratch transformer training run. Closes the "I can run training, not just inference" gap.
2. **Anthropic Fellows / DeepMind Scholars application track preparation** — re-read OpenAI / Anthropic papers; build a small research-track artefact (eval harness, mechanistic-interpretability tool).
3. **NeuroDrive past M6 substrate validation** — the SideBySide behavioural acceptance run that's been gated; closes the M6-substrate-complete-but-unvalidated open thread from the 2026-04-30 NeuroDrive session.

## Pinnacle-relevant evidence

- Anthropic / OpenAI / DeepMind / Isomorphic Labs — pinnacle AI/ML employers; selection brutal.
- Wayve / Anthropic-adjacent — strong AI/ML with autonomous-systems flavour where NeuroDrive evidence transfers directly.
- Cohere / Mistral / Adept — strong AI/ML with smaller-company access patterns.

## Lane-internal calibration notes

AI/ML lane includes both Research (PhD-track, harder for Caner's 2:2 York profile) and Engineering (more accessible). Default lane assignment is Engineering unless JD explicitly says Research / Scientist function.
