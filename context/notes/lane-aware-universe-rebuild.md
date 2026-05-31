# Lane-aware universe rebuild (2026-05-29 → 2026-05-31)

> Captures the coherent multi-commit move from a flat-grade single-tier universe to a sponsor-only, lane-tagged, lane-relatively-graded universe. The work spans the preferences refactor, ethical-exclusion deletions, 9-agent discovery expansion, and the grade wipe. Treat as one rebuild, not four unrelated changes.

## Why the rebuild

Cernio's grading semantic changed with the lane-based-relativity refactor (`context/plans/cernio-full-refactor.md`, design-locked 2026-05-28). Companies are now graded **relative to their lane's pinnacle distribution**, not on a flat S/A/B/C scale that conflated Google with a 20-person AI startup. Several upstream rebuilds had to land before the first lane-aware regrade could produce a coherent calibration.

## What changed, in order

### 1. Preferences refactor — mechanical-only TOML (commit `72006d3`, 2026-05-29)

`preferences.toml` had drifted into a snapshot of taste — `tech_preferred`, `sectors_preferred`, `exclude_role_types`, positive/negative signals, seniority bands, remote preference, company size, `[hard].locations`. All duplicated content the grading skills already infer from `profile/` (career-goals.md, lifestyle-preferences.md, skills/, projects/). The duplication was the snapshot anti-pattern the Living System Philosophy explicitly opposes.

**After:** `preferences.toml` holds only what the Rust pipeline needs at runtime — include/exclude keyword filters, the shared location-pattern list, `min_company_grade`, cleanup thresholds. `[hard]` and `[soft]` sections deleted entirely. The substance moved to `profile/career-goals.md` as prose: the 8 active lanes, role-truth-at-hire hard rule, sponsor-only universe rule, ethical-exclusions hard rule, and the Tier 1/2/3 location table.

**Guard:** a `no_hard_or_soft_sections_present` regression test in `tests/preferences_integrity.rs` prevents the anti-pattern from quietly returning.

### 2. Shared location list (commit `f592ca2`, 2026-05-29)

The six per-provider `[search_filters.locations.<provider>]` subtables were 100% duplicated content — every provider matched the same UK + remote vocabulary case-insensitively, and the format differences (workable `"GB"` vs smartrecruiters `"gb"` vs greenhouse `"London, England"`) were already handled by the shared lowercased `contains()` match in `src/config.rs`. Collapsed to a single `[search_filters.locations] patterns = [...]` list.

**Rust changes:** `LocationConfig` flattened from `HashMap<String, LocationConfig>` to a single struct with a `patterns: Vec<String>`. `passes_location(&self, provider: &str, locs: &[String])` → `passes_location(&self, locs: &[String])` (provider parameter dropped). Three callsites in `src/pipeline/search.rs` updated. The `per_provider_isolation` unit test was removed (no longer meaningful).

**Patterns expanded:** Tier 1/2/3 cities from `career-goals.md` added so `cernio search` actually catches Reading, Oxford, Milton Keynes, Brighton, Guildford, Watford, Stevenage, Luton, Newbury, Bristol, Bath, Birmingham, Coventry, Northampton, Sheffield, Nottingham, Derby, Edinburgh, Glasgow. Previously only London/Cambridge/UK patterns were live; tier cities only passed if their JD location string happened to contain "UK".

### 3. `remove_job_grades` → `archive_job_grades` (commit `aebd701`)

A pure rename for honesty — `cernio clean` archives jobs (`evaluation_status = 'archived'` + `archived_at = now`); a separate 14-day expiry sweep later deletes archived jobs. The original name suggested deletion was happening on every clean. The new name matches reality. Field is in `preferences.toml [cleanup]`.

### 4. Ethical-exclusion deletions (commit `dc7d718`)

The new `career-goals.md §Hard rules` ethical-exclusion clause covers gambling, adtech, and consumer-crypto as categorical no-gos. 19 companies in those three categories had already entered the DB via earlier discovery passes and had to be removed:

| Category | Count | Companies |
|---|---|---|
| Gambling | 5 | Betfred Technology (Sharp Gaming), Black Cow Technology, EveryMatrix Technology UK, Genius Sports, Smarkets |
| Adtech | 7 | Captify Technologies, Cheq, Moloco, StackAdapt, Tubular Labs, Wunderkind, Zilch Technology |
| Consumer-crypto | 7 | 0xA Technologies, Argent, Blockchain (GB), Bron Labs, Lattice Labs, OKX UK FinTech, XDEFI Wallet |

Cascade-deleted 20 jobs + 11 `company_portals`. Zero `user_decisions` affected.

> **Explicit override of archival doctrine.** Cernio's general principle is "archive, never delete" (preserves grading history; prevents re-discovery and re-grading). For these 19 companies the user opted to **delete** instead — they should never have entered the DB and re-discovery is the desired behaviour if the policy ever changes. Archival would have made them invisible-but-resurrectable; deletion clears the slate. This is the only known intentional exception to the archival rule.

Tonk Labs and Yaspa kept active despite surfacing in the scan — Tonk is B2B infra research mistagged "consumer-crypto"; Yaspa is FCA-regulated open-banking payments with a casino-payments vendor relationship, not consumer-gambling enablement. The crypto-mm lane remains in scope per the consumer-vs-institutional carveout.

### 5. 9-agent parallel discovery (commit `d6256f6`, 2026-05-31)

8 per-lane agents + 1 non-obvious-sources agent dispatched in parallel. Each agent read the profile, received the existing universe for dedup, and wrote finds to `companies/discovery-{lane}-2026-05-31.md`. Orchestrator imported via `cernio import`.

**Outcome:**

- 212 net-new candidates surfaced across all lanes (out of ~230 emitted; 10 auto-deduped by website constraint, 7 invalid, 24 cross-name URL-format duplicates deleted via name-canonical dedup pass).
- Final state: **892 active companies** (up from 687), all sponsor-only.
- Sparse lanes lifted: bank-strats 18 → ~46, crypto-mm 42 → ~69, devtools 56 → ~105, big-tech 66 → ~97.
- Codeplay (Intel) had its UK Skilled Worker licence revoked between Feb–Apr 2026 per the systems-infra agent's flagging. `sponsors_uk` flipped to `no` and the company archived per sponsor-only universe rule. (Living-system principle in action: even existing companies must stay verified sponsors.)

**Known follow-up — `cernio import` parser bug.** 226 newly-imported companies had `NULL` lanes + `NULL` sponsor because the parser only recognises `Website`/`What they do`/`Why relevant`/`Source` fields — `Lane` and `Sponsor` markdown fields are silently dropped. Backfilled via `/tmp/cernio-backfill-lanes.py` (parses each discovery file: lane from filename for per-lane files, per-entry `Lane` field for the non-obvious file; UPDATEs matching DB rows by lowercased name). The parser at `src/pipeline/import.rs:143` (`parse_potential_md`) should be extended to recognise `Lane:` and `Sponsor:` so future discovery imports skip the backfill step.

### 6. Grade wipe (commit `c8dc8e6`, 2026-05-31)

Cleared `grade`, `grade_reasoning`, `graded_at`, `pinnacle_status_per_lane` on every active company. Lanes, `sponsors_uk`, `status`, `location`, `sector_tags`, ATS resolution data all preserved.

**Pre-wipe:** 598 graded / 89 ungraded. **Post-wipe:** 0 / 687. Lanes preserved: 687/687. Sponsor field preserved: 687/687.

**Why selective wipe instead of full reset:** lanes are expensive to assign (the discovery agents do it; backfill is the alternative); sponsor status is expensive to verify (per-company research); ATS resolution requires probe-runs against six providers per company. None of that needs to change with the grading semantic. Only the grade column itself is calibration-dependent — wipe it, preserve everything else.

## Grade-wipe philosophy (durable lesson)

> **When a grading semantic changes, wipe only the grade-derived columns; preserve everything that took human or AI labour to assign.** The expensive-to-rederive fields (lanes, sponsor verification, ATS slugs, location, sector tags) survive every regrade. The cheap-to-rederive fields (grade, grade reasoning, graded-at timestamp, pinnacle status) reset cleanly. This is the **same principle as the archival-not-deletion default**: preserve history, lose only what is fastest to regenerate from the new semantic.

The cost asymmetry is roughly 30 seconds (one Claude grading call) vs. 5–30 minutes (probe ATS providers, verify sponsor status, classify lane, source what-they-do prose). Wipe the seconds-side; preserve the minutes-side.

## What still needs to happen

1. Run `grade-companies` against the 892 ungraded set (lane-aware, lane-relatively calibrated). This is the first proper lane-relative calibration the system has ever produced.
2. Extend `src/pipeline/import.rs::parse_potential_md` to recognise `Lane:` and `Sponsor:` fields so future discovery imports don't need the backfill script.
3. Review `context/plans/cernio-full-refactor.md` against current state — substantial parts of the design have shipped; the plan file should reflect what is and isn't done.

## See also

- `context/plans/cernio-full-refactor.md` — the original design that drove this rebuild
- `profile/career-goals.md` — the canonical home for the moved-from-preferences content
- `profile/preferences.toml` — mechanical-only TOML after the refactor
- `context/notes/profile-system.md` — Cernio-native vs LifeOS-synced files
- `companies/discovery-*-2026-05-31.md` — the 9-agent discovery output (per-lane files preserved as research artefacts)
- Commits `72006d3`, `f592ca2`, `aebd701`, `c3c9ca2`, `dc7d718`, `d6256f6`, `c8dc8e6` — the rebuild's full git trail
