# Portfolio Gap Analysis

> A living document tracking what the job market consistently asks for vs what the profile currently offers. Updated as Claude evaluates jobs and spots patterns.

---

## Current Strengths

- **From-scratch implementation depth**: Every major project builds core functionality from first principles (matching engine, PPO, CLIP inference pipeline, DeFi analytics). This is rare at entry level and directly demonstrates understanding rather than framework usage.
- **Observability and instrumentation**: NeuroDrive has a comprehensive analytics pipeline (16 tick fields, 25 episode aggregates, diagnostic markdown reports, profiling infrastructure). Nyquestro has HDR latency histograms with hardware perf counters. This demonstrates production-minded engineering.
- **Performance engineering**: Flat storage 43x improvement in NeuroDrive, lock-free structures in Nyquestro, frame budget management with amortised training. Concrete evidence of profiling-driven optimisation.
- **Multi-domain fluency**: Systems, ML, finance, and desktop application development across a single portfolio. Unusual breadth at entry level.

## Known Gaps

- **No CI/CD in any project**: No GitHub Actions, no automated testing pipelines, no deployment workflows visible in any repo. This is one of the most commonly asked-about skills at entry level.
- **No containerisation**: No Dockerfiles, no container-based development or deployment. Docker/Kubernetes appear in a large percentage of backend and infrastructure roles.
- **No cloud experience**: No AWS, GCP, or Azure usage. Many entry-level roles expect at least basic cloud familiarity.
- **No testing visible in project entries**: NeuroDrive's `cargo test` passes per architecture.md, but testing isn't highlighted in any profile entry. Test-driven development is a common interview topic.
- **No collaborative development evidence**: All projects are solo. No evidence of code review, PR workflows, or team development beyond the university team project.

## Gap Closure Opportunities

- **Add GitHub Actions CI to NeuroDrive or Nyquestro**: A `cargo test` + `cargo clippy` pipeline takes 30 minutes to set up and immediately addresses the CI/CD gap. Nyquestro is the best candidate since it's the most engineering-focused project.
- **Add a Dockerfile to one project**: Even a simple dev container for Nyquestro or NeuroDrive demonstrates containerisation familiarity. Low effort, high signal.
- **Highlight existing tests in profile entries**: If tests exist (NeuroDrive passes `cargo test`), make them visible in the technical highlights. Don't undersell what already exists.
- **Land a merged open source contribution**: The tinygrad PR was closed for line-count reasons. A merged contribution to any well-known project would be a significant signal. Bevy, ratatui, or reqwest are all projects already in the stack.

## Patterns from Job Evaluations

<!-- Recurring observations from evaluating job descriptions. Updated after evaluation sessions. -->

