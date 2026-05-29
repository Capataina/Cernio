# Job Lane vs Company Lane — Worked Examples

The job's lane is the role's *function*, classified from title + JD. The
company's lane is the firm's *primary business*. These often match — but
they don't have to, and the divergence is information, not error.

This file is mandatory-read alongside `grading-rubric.md` during Phase 1.

## The rule, in one sentence

| Source | Drives |
|---|---|
| Title + JD content | The job's lane (`jobs.lanes`) |
| Company's primary business | The company's lane (`companies.lanes`) |
| Company's `pinnacle_status_per_lane` | The Q3 within-lane position the job is graded against |

When title + JD make the role function clear, that wins. The company lane
is consulted only for Q3 pinnacle position and as a tiebreaker when title +
JD are too thin to classify the role.

## Divergence cases — concrete

These are real patterns the agent will hit. The "job lane" column is the
correct classification. Anchor to these; do not auto-inherit company lane.

| Title (excerpt) | Company | Company lane | Job lane | Why divergent |
|---|---|---|---|---|
| Platform Engineer / Site Reliability Engineer | Monzo | `fintech` | `systems-infra` | Role is infra-team, not product-payments |
| Senior ML Research Engineer — multimodal | Google | `big-tech` | `ai-ml` | Role is ML research, not generalist SWE |
| Compiler Engineer | Citadel | `hft` | `systems-infra` | Compiler tooling team, not trading-system |
| ML Performance Engineer — CUDA kernels | Jane Street | `hft` | `ai-ml` | ML systems team, not strats |
| Quant Developer — equity strats | Goldman Sachs | `bank-strats` | `bank-strats` | Matches (no divergence) |
| Software Engineer, Vault Engineering | JPMorgan | `bank-strats` | `systems-infra` | Internal infra platform team, not strats |
| Developer Relations Engineer | Stripe | `fintech` | `devtools` | DX-facing role on the API team |
| Backend Engineer, Payments | Stripe | `fintech` | `fintech` | Matches |
| Distributed Systems Engineer | Bloomberg | `big-tech` | `systems-infra` | Bloomberg's terminal/data infra team |
| Trading Systems Engineer | Two Sigma | `hft` | `hft` | Matches |
| Research Engineer, Alignment | Anthropic | `ai-ml` | `ai-ml` | Matches |
| Compiler Backend Engineer | Apple | `big-tech` | `systems-infra` | Swift compiler team |
| Junior Software Engineer | a generic fintech with sparse JD | `fintech` | `fintech` (fallback) | Title + JD too thin — fall back to company lane |
| Engineering Manager, Crypto Liquidity | Coinbase | `crypto-mm` | `crypto-mm` | Matches |
| Smart Contract Engineer | a generic crypto firm | `crypto-mm` | `systems-infra` | Solidity/EVM is systems work, not market-making |
| Senior Rust Engineer | Cloudflare | `big-tech` | `systems-infra` | Cloudflare workers / edge infra |
| Frontend Engineer | Datadog | `devtools` | `devtools` | Matches |

## Anti-patterns

| Pattern | Why it's wrong |
|---|---|
| Auto-inheriting company lane on every job without reading title/JD | Defeats the whole point of separating the two fields. Produces a near-degenerate `jobs.lanes` distribution and erases the Monzo-systems / Google-ML signal. |
| Classifying job lane purely from the title without reading the JD | Title can mislead: "Software Engineer" at Monzo could be payments-rail (fintech) or platform (systems-infra). The JD body is the disambiguator. |
| Using company `pinnacle_status_per_lane.<job_lane>` when the company isn't tagged with that lane | If the job is `ai-ml` but the company is not tagged `ai-ml`, the company's pinnacle position in `ai-ml` is undefined. Treat as `adjacent` or lower, not pinnacle, unless the company explicitly tags that lane. |

## What "fallback to company lane" actually means

The fallback fires when, after reading title + JD, the agent cannot
confidently pick a lane. Examples that trigger fallback:

- A 1-line description with a generic title ("Software Engineer", "Engineer
  III") and no team / domain signal.
- A description that's purely company boilerplate (`About us`, benefits,
  diversity statement) with no role-specific content.
- Multiple lanes equally plausible AND no JD detail to break the tie.

In all other cases — including when the role spans two lanes — classify
from title + JD. Multi-tag is supported (`["systems-infra", "devtools"]`
for a developer-tooling platform role); primary lane goes first.

## Multi-tag examples

When a role genuinely spans two function areas, multi-tag with the more
load-bearing lane first.

| Title | Lanes |
|---|---|
| Developer Tools Platform Engineer | `["devtools", "systems-infra"]` |
| ML Infrastructure Engineer | `["systems-infra", "ai-ml"]` |
| Quant Strategy Developer (Python + C++) | `["bank-strats", "systems-infra"]` |
| Trading Systems / Low-Latency Engineer | `["hft", "systems-infra"]` |
| AI Safety Researcher (Engineering-track) | `["ai-ml", "systems-infra"]` |

Multi-tag is for genuine span, not for hedging. If the role's primary
function is clear, single-tag.

## Cross-check in Phase 2

The Phase 2 within-lane relativity pass should sanity-check job-lane
classifications, not just grades. If the same agent has classified five
Monzo Platform Engineers as `fintech` and one as `systems-infra`, surface
the inconsistency and re-classify the outliers (in this case, *the five
are wrong* — Monzo platform is `systems-infra`).
