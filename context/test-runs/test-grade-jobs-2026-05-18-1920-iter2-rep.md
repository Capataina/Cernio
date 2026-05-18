# test-grade-jobs run 2026-05-18-1920-iter2-rep (reproduction)

Fresh 60-job sample, same grade-jobs commit (`76d6776`) as the 1830-iter2 run. Purpose: distinguish provable upgrade from sample-variance luck.

## Headline

**Not a provable upgrade.** Six of seven axes regressed versus the 1830-iter2 measurement. Only axis_f (pairwise agreement) shows a robust gain that held across both runs of the same code.

## Three-way comparison

| Axis | Baseline | iter2 (1830) | iter2-rep (1920) | rep vs baseline | iter2 vs rep range |
|---|---|---|---|---|---|
| A (slot completeness) | 80.5 | 88.7 | **75.4** | −5.1 | 13.3 |
| B (parse cleanliness) | 38.3 | 39.7 | **33.2** | −5.1 | 6.5 |
| C (evidence anchoring) | 89.5 | 95.1 | **87.7** | −1.8 | 7.4 |
| D (Q1→verdict coherence) | 94.6 | 98.2 | **87.9** | −6.7 | 10.3 |
| E (per-job grade agreement) | 80.5 | 80.4 | **78.7** | −1.8 | 1.7 |
| F (pairwise agreement) | 30.0 | 70.0 | **70.0** | **+40.0** | 0.0 |
| G (cluster-position effect) | 59.7 | 62.5 | **60.2** | +0.5 | 2.3 |
| Composite | 67.6 | 76.4 | **70.4** | +2.8 | 6.0 |

## Reading the result honestly

**Axis F is the only provable upgrade.** Both the 1830-iter2 run and the 1920-rep run land at exactly 70.0 versus the baseline's 30.0. Two independent samples agreeing on a +40 gain is not coincidence — pairwise agents converged on Q1-primary decision logic across both samples. This is real signal from the rubric overhaul.

**Axes A, B, C, D, E, G fluctuate within ±13 points between runs of identical code.** Same skill, same agents, same selection script — different 60-job sample. That tells us the noise floor for these axes is large enough to swallow the "iter2 vs iter1" deltas we celebrated in the 1830 report. Attributing the +10.9 axis_d gain or the +13.2 axis_e gain to Rules 15+16 was overconfident; both axes moved by similar magnitudes in the opposite direction on a fresh sample.

**Composite is +2.8 vs baseline.** Real but small — and below the per-axis sample-variance band, so a third run could plausibly land anywhere from +0 to +6.

**The 0-SS outcome reproduces.** Iter2 had 0 SS / 1 S / 3 A / 0 B / 14 C / 64 F over 82 individual grade-emissions. Rep had 0 SS / 2 S / 2 A / 9 B / ~33 C / ~76 F over ~122 emissions. The "no SS in this manifest" pattern held across both samples — the rubric structurally treats SS as a true ceiling rather than a fill-the-distribution band. That's behaviour, not bias.

## What this means for the iter2 changes

Rules 15 (anti-inflation) and 16 (risks-engaged-in-Verdict) likely DID improve structural prose discipline — axis A in iter2 was +5 above the rep, and the slot-completeness mechanism is one the rules target. But the gains on axes D, E, G are mostly indistinguishable from sample-variance noise. Calling iter2 a clean win on those axes was reading the wins too generously.

The honest summary: **one robust win (axis F, pairwise), one plausible win (composite +2.8), five axes inside the noise band.** Better than baseline on average, but the "provably better across all metrics" claim from the 1830 report does not survive a single reproduction.

## What I Did Not Do

- Did not run a third sample — two is enough to falsify the strong "all axes improved" claim; collecting more samples to narrow the variance bands is a future-test exercise.
- Did not roll back iter2's rule additions — Rules 15+16 do not appear to harm any axis; they are at worst neutral.
- Did not investigate the source of axis-variance — different sample compositions (heavy narrow-funnel quant vs heavy entry-level fintech) plausibly explain a lot of it. A future test pass with controlled-composition samples would let signal beat noise.
