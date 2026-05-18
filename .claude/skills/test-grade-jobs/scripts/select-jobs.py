#!/usr/bin/env python3
"""
select-jobs.py — Stratified 60-job diversity sample for test-grade-jobs.

Picks 30 stress-pattern jobs (cluster A) + 30 stability-pattern jobs (cluster B)
from state/cernio.db. Writes per-agent manifests (13 agents) with randomised job
order per agent. Builds the coverage matrix.

The clusters are diversity samples, NOT expectation labels. The script does not
identify trigger cases or persist DB grades to disk — both encoded curated
answers the test must not consult.

Per the skill's inviolable rules: zero grade leakage to manifests. The script
selects jobs WITHOUT their DB grades; no DB-grade field appears in any output.

Usage:
    python3 select-jobs.py <run-id>

Writes to /tmp/test-grade-jobs-<run-id>/:
    jobs-all.json            — 60 full job records (no grades)
    cluster-a.json           — 30 cluster A job IDs
    cluster-b.json           — 30 cluster B job IDs
    coverage-matrix.json     — job_id -> list of agent_ids that will grade it
    manifest-agent-*.json    — per-agent manifest with randomised job order (13 files)

stdout: run-id confirmation, cluster sizes, zero-grade-leakage assertion.
"""

import json
import os
import random
import sqlite3
import sys
from pathlib import Path

# Stress patterns for cluster A — diversity-sampling anchors, not outcome expectations.
NARROW_FUNNEL_FIRMS = {
    "Jane Street", "Hudson River Trading", "HRT", "XTX Markets", "Citadel",
    "Citadel Securities", "Two Sigma", "DE Shaw", "Jump Trading",
    "Tower Research Capital", "Old Mission Capital", "Optiver", "Akuna Capital",
    "Squarepoint Capital", "Point72", "Cubist", "Point72 / Cubist", "Millennium",
    "G-Research", "Susquehanna", "SIG", "Belvedere Trading", "DRW", "IMC Trading",
    "Qube Research & Technologies", "QRT", "Man Group", "Man Group (AHL)",
    "Capula Investment Management",
}

# Stability patterns for cluster B — diversity-sampling anchors, not outcome expectations.
MID_TIER_FINTECH = {
    "Lendable", "Trainline", "Monzo", "Zopa", "Starling Bank", "Wise",
    "Cleo", "Plaid", "Revolut", "GoCardless", "Tide", "Curve", "Atom Bank",
    "OakNorth", "Checkout.com", "Yapily",
}

WIDE_FUNNEL_GRAD = {
    "Cloudflare", "Spotify", "Palantir", "Amazon", "Google", "Microsoft",
    "Meta", "Apple", "Netflix", "Twilio", "Datadog", "Snowflake",
    "B2C2", "GitLab", "Stripe", "Bloomberg", "Arm",
}

DB_PATH = "state/cernio.db"


def connect_db():
    if not os.path.exists(DB_PATH):
        print(f"FATAL: {DB_PATH} not found. Run from repo root.", file=sys.stderr)
        sys.exit(2)
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    return conn


def fetch_candidates(conn):
    """Pull all eligible jobs (description > 500 chars, not archived) with their
    context. Deliberately does NOT select j.grade or j.fit_assessment — those
    are external answers the test must not consult."""
    cur = conn.cursor()
    cur.execute("""
        SELECT j.id, j.title, j.url, j.location, j.remote_policy,
               j.raw_description, c.name AS company_name, c.what_they_do
        FROM jobs j JOIN companies c ON c.id = j.company_id
        WHERE j.evaluation_status <> 'archived'
          AND LENGTH(j.raw_description) > 500
    """)
    return [dict(r) for r in cur.fetchall()]


def stress_pattern_score(job):
    """Score a job by how many stress patterns it matches. Higher = more
    cluster-A-eligible. Score is for SAMPLING ONLY — it does NOT encode any
    grade expectation."""
    score = 0
    company = job["company_name"] or ""
    title = (job["title"] or "").lower()
    desc = (job["raw_description"] or "").lower()

    if company in NARROW_FUNNEL_FIRMS:
        score += 3
    if "engineer" in title and not any(t in title for t in ("graduate", "junior", "intern", "new grad")):
        # implicit-seniority disguise candidate: needs years floor in desc
        if any(p in desc for p in ("3+ years", "4+ years", "5+ years", "5-10 years",
                                    "extensive experience", "deep expertise", "expert-level")):
            score += 2
    if "intern" in title and any(t in desc for t in ("currently pursuing", "current student",
                                                       "expected graduation")):
        score += 2
    if any(t in title for t in ("analyst", "trader", "quantitative researcher", "data scientist")):
        score += 2
    if any(t in desc for t in ("security clearance", "sc clearance", "dv clearance",
                                "uk national required", "british national required")):
        score += 2
    if any(t in desc for t in ("customer engagement", "customer-facing", "interact with customers",
                                "forward-deployed")):
        score += 2
    # off-stack mid-level (Kotlin/Android, iOS, Salesforce at mid-level)
    if any(t in desc for t in ("kotlin", "android development", "ios development",
                                "swift development", "salesforce")):
        if any(p in desc for p in ("3+ years", "4+ years")):
            score += 1
    # staff-tier comp band disguise
    if "£200" in desc or "£250" in desc or "£300" in desc:
        if any(t in desc for t in ("lead the", "own the", "shape the", "principal")):
            score += 2
    return score


def stability_pattern_score(job):
    """Score a job by stability-pattern match. Higher = more cluster-B-eligible.
    Score is for SAMPLING ONLY — it does NOT encode any grade expectation."""
    score = 0
    company = job["company_name"] or ""
    title = (job["title"] or "").lower()
    desc = (job["raw_description"] or "").lower()
    location = (job["location"] or "").lower()

    # wide-funnel grad
    if company in WIDE_FUNNEL_GRAD and any(t in title for t in ("graduate", "new grad", "2026 grad")):
        score += 3
    # hard years floor (explicit)
    if any(p in desc for p in ("4+ years", "5+ years", "5-10 years", "6+ years", "7+ years")):
        score += 2
    # hard location exclusion candidate
    if any(loc in location for loc in ("bristol", "edinburgh", "manchester", "berlin",
                                         "dublin", "paris", "munich", "amsterdam")):
        score += 2
    # mid-tier fintech junior
    if company in MID_TIER_FINTECH and any(t in title for t in ("graduate", "junior", "associate",
                                                                  "engineer i", "engineer 1")):
        score += 2
    # off-stack mid-level (different from cluster A's off-stack — here it's a stability case
    # because the stack mismatch is clear and unambiguous)
    if any(t in desc for t in ("kotlin only", "android-first", "ios-only", "swift-only")):
        score += 1
    # standard junior engineering at recognised firm
    if any(t in title for t in ("graduate", "junior", "engineer i", "engineer 1", "associate")):
        if not (company in MID_TIER_FINTECH or company in WIDE_FUNNEL_GRAD):
            # mid-tier-recognised-firm signal
            if job["company_name"]:
                score += 1
    return score


def select_cluster(candidates, scorer, n, rng, exclude_ids=None):
    """Select up to n jobs sorted by scorer (descending). Ties broken by rng.
    Skips jobs in exclude_ids."""
    exclude_ids = exclude_ids or set()
    scored = [(scorer(j), rng.random(), j) for j in candidates if j["id"] not in exclude_ids]
    # Keep only jobs with score > 0
    scored = [s for s in scored if s[0] > 0]
    scored.sort(key=lambda x: (-x[0], x[1]))
    return [s[2] for s in scored[:n]]


def manifests_for_run(cluster_a, cluster_b, rng):
    """Build the 13 per-agent manifests + coverage matrix.

    Inventory:
      cluster-a-10job-1/2/3   (10 jobs each, A disjoint)
      cluster-a-15job-1/2     (15 jobs each, A disjoint)
      cluster-a-30job         (full cluster A)
      cluster-b-10job-1/2/3   (10 jobs each, B disjoint)
      cluster-b-30job         (full cluster B)
      cross-cluster            (15 from A + 15 from B)
      pairwise-1, pairwise-2  (~20 pairs each — different manifest shape)
    """
    a_ids = [j["id"] for j in cluster_a]
    b_ids = [j["id"] for j in cluster_b]
    by_id = {j["id"]: j for j in cluster_a + cluster_b}

    # Shuffle and disjoint-split cluster A into three 10-job pieces
    a_shuffled = list(a_ids)
    rng.shuffle(a_shuffled)
    a_10job_1 = a_shuffled[0:10]
    a_10job_2 = a_shuffled[10:20]
    a_10job_3 = a_shuffled[20:30]
    # Two 15-job splits (different shuffle)
    a_for_15 = list(a_ids)
    rng.shuffle(a_for_15)
    a_15job_1 = a_for_15[0:15]
    a_15job_2 = a_for_15[15:30]
    # Full cluster A
    a_30job = list(a_ids)
    rng.shuffle(a_30job)

    # Shuffle and disjoint-split cluster B into three 10-job pieces
    b_shuffled = list(b_ids)
    rng.shuffle(b_shuffled)
    b_10job_1 = b_shuffled[0:10]
    b_10job_2 = b_shuffled[10:20]
    b_10job_3 = b_shuffled[20:30]
    # Full cluster B
    b_30job = list(b_ids)
    rng.shuffle(b_30job)

    # Cross-cluster: 15 from A + 15 from B
    cross_a = list(a_ids); rng.shuffle(cross_a)
    cross_b = list(b_ids); rng.shuffle(cross_b)
    cross_cluster = cross_a[:15] + cross_b[:15]
    rng.shuffle(cross_cluster)

    # Pairwise manifests: build pairs from cluster A and cluster B mixed
    all_ids = a_ids + b_ids
    pairs_1 = []
    pairs_2 = []
    # 20 pairs each, random sampling without replacement within each agent
    for _ in range(20):
        a, b = rng.sample(all_ids, 2)
        pairs_1.append({"pair_id": f"p{len(pairs_1)+1:03}", "job_a": by_id[a], "job_b": by_id[b]})
        a, b = rng.sample(all_ids, 2)
        pairs_2.append({"pair_id": f"p{len(pairs_2)+1:03}", "job_a": by_id[a], "job_b": by_id[b]})

    grading_manifests = {
        "cluster-a-10job-1": a_10job_1,
        "cluster-a-10job-2": a_10job_2,
        "cluster-a-10job-3": a_10job_3,
        "cluster-a-15job-1": a_15job_1,
        "cluster-a-15job-2": a_15job_2,
        "cluster-a-30job":   a_30job,
        "cluster-b-10job-1": b_10job_1,
        "cluster-b-10job-2": b_10job_2,
        "cluster-b-10job-3": b_10job_3,
        "cluster-b-30job":   b_30job,
        "cross-cluster":     cross_cluster,
    }
    # Convert each ID list to a list of full job records (no grades, no fit_assessment)
    grading_manifests_full = {
        name: [{k: v for k, v in by_id[jid].items()
                if k not in ("grade", "fit_assessment")} for jid in ids]
        for name, ids in grading_manifests.items()
    }

    # Coverage matrix: which agents see each job
    coverage = {}
    for name, ids in grading_manifests.items():
        for jid in ids:
            coverage.setdefault(jid, []).append(f"agent-{name}")

    return grading_manifests_full, pairs_1, pairs_2, coverage


def main():
    if len(sys.argv) < 2:
        print("Usage: select-jobs.py <run-id>", file=sys.stderr)
        sys.exit(2)
    run_id = sys.argv[1]
    out_dir = Path(f"/tmp/test-grade-jobs-{run_id}")
    out_dir.mkdir(parents=True, exist_ok=True)

    rng = random.Random()  # timestamp-seeded; reproducibility not required per-run

    conn = connect_db()
    candidates = fetch_candidates(conn)
    print(f"Eligible candidates (desc > 500 chars, not archived): {len(candidates)}")

    cluster_a = select_cluster(candidates, stress_pattern_score, 30, rng)
    cluster_b = select_cluster(candidates, stability_pattern_score, 30, rng,
                                exclude_ids={j["id"] for j in cluster_a})
    print(f"Cluster A (stress patterns): {len(cluster_a)} jobs")
    print(f"Cluster B (stability patterns): {len(cluster_b)} jobs")
    print(f"Total selected: {len(cluster_a) + len(cluster_b)} jobs")

    # Write top-level files
    all_jobs = cluster_a + cluster_b
    # Strip grade/fit_assessment columns — defensive even though we didn't SELECT them
    for j in all_jobs:
        j.pop("grade", None)
        j.pop("fit_assessment", None)
    with open(out_dir / "jobs-all.json", "w") as f:
        json.dump(all_jobs, f, indent=2)
    with open(out_dir / "cluster-a.json", "w") as f:
        json.dump([j["id"] for j in cluster_a], f, indent=2)
    with open(out_dir / "cluster-b.json", "w") as f:
        json.dump([j["id"] for j in cluster_b], f, indent=2)

    # Build manifests
    grading, pairs_1, pairs_2, coverage = manifests_for_run(cluster_a, cluster_b, rng)
    for name, jobs in grading.items():
        with open(out_dir / f"manifest-agent-{name}.json", "w") as f:
            json.dump(jobs, f, indent=2)

    # Strip grade/fit_assessment from pairwise records too
    def _clean(j):
        return {k: v for k, v in j.items() if k not in ("grade", "fit_assessment")}
    pairs_1_clean = [{"pair_id": p["pair_id"], "job_a": _clean(p["job_a"]), "job_b": _clean(p["job_b"])}
                     for p in pairs_1]
    pairs_2_clean = [{"pair_id": p["pair_id"], "job_a": _clean(p["job_a"]), "job_b": _clean(p["job_b"])}
                     for p in pairs_2]
    with open(out_dir / "manifest-agent-pairwise-1.json", "w") as f:
        json.dump(pairs_1_clean, f, indent=2)
    with open(out_dir / "manifest-agent-pairwise-2.json", "w") as f:
        json.dump(pairs_2_clean, f, indent=2)

    with open(out_dir / "coverage-matrix.json", "w") as f:
        json.dump(coverage, f, indent=2)

    n_manifests = len(grading) + 2  # +2 pairwise
    print(f"Manifest files written: {n_manifests}")
    job_to_agents = {str(jid): aids for jid, aids in coverage.items()}
    if job_to_agents:
        coverage_counts = [len(aids) for aids in coverage.values()]
        print(f"Coverage: every job graded by {min(coverage_counts)}-{max(coverage_counts)} agents "
              f"(mean {sum(coverage_counts)/len(coverage_counts):.1f})")

    # Zero-grade-leakage assertion
    import subprocess
    grep_out = subprocess.run(
        ["grep", "-l", "-E", r'"grade"|"fit_assessment"|"grade_reasoning"',
         *[str(out_dir / f"manifest-agent-{name}.json") for name in grading],
         str(out_dir / "manifest-agent-pairwise-1.json"),
         str(out_dir / "manifest-agent-pairwise-2.json")],
        capture_output=True, text=True
    )
    if grep_out.stdout.strip():
        print(f"FATAL: grade leakage detected in {grep_out.stdout}", file=sys.stderr)
        sys.exit(3)
    print("Zero grade leakage confirmed across all manifests.")
    print(f"Run ID: {run_id}")
    print(f"Working dir: /tmp/test-grade-jobs-{run_id}")
    print("DONE.")


if __name__ == "__main__":
    main()
