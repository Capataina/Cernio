#!/usr/bin/env python3
"""
select-jobs.py — Stratified 60-job selection for test-grade-jobs.

Picks 30 stress-test jobs (cluster A) + 30 stability anchors (cluster B)
from state/cernio.db. Identifies trigger cases. Writes per-agent manifests
with randomised job order per agent. Builds the coverage matrix.

Per the skill's inviolable rules: zero grade leakage to manifests. The
script selects jobs WITH their DB grades for coverage statistics, then
strips the grade field from any output that any agent will read.

Usage:
    python3 select-jobs.py <run-id>

Writes to /tmp/test-grade-jobs-<run-id>/:
    jobs-all.json            — 60 full job records (no grades)
    cluster-a.json           — 30 cluster A job IDs
    cluster-b.json           — 30 cluster B job IDs
    trigger-cases.json       — subset of cluster A IDs with trigger-pattern hits
    coverage-matrix.json     — job_id -> list of agent_ids that will grade it
    db-grades.json           — job_id -> DB grade (for analysis only; NEVER in any manifest)
    manifest-agent-*.json    — per-agent manifest with randomised job order

stdout: run-id confirmation, cluster sizes, trigger-case count,
       zero-grade-leakage assertion.
"""

import json
import os
import random
import sqlite3
import sys
from pathlib import Path

# Stress-test patterns for cluster A
PRESTIGE_FIRMS = {
    "Jane Street", "Hudson River Trading", "HRT", "XTX Markets", "Citadel",
    "Two Sigma", "DE Shaw", "Jump Trading", "Tower Research Capital",
    "Old Mission Capital", "Optiver", "Akuna Capital", "Squarepoint Capital",
    "Point72", "Cubist", "Millennium", "G-Research", "Susquehanna",
    "SIG", "Belvedere Trading", "DRW", "IMC Trading",
}

TRIGGER_PHRASES = [
    "stretch", "lottery", "sub-1%", "sub 1%", "headwind", "prestige-trap",
    "prestige trap", "stretch-A", "A-stretch", "narrow-funnel",
    "narrow funnel", "brutal selectivity", "lottery ticket", "lottery band",
]

# Stability-anchor patterns for cluster B
MID_TIER_FINTECH = {
    "Lendable", "Trainline", "Monzo", "Zopa", "Starling Bank", "Wise",
    "Cleo", "Plaid", "Stripe", "Revolut", "GoCardless", "Tide",
    "Curve", "Atom Bank", "OakNorth",
}

WIDE_FUNNEL_GRAD = {
    "Cloudflare", "Spotify", "Palantir", "Amazon", "Google", "Microsoft",
    "Meta", "Apple", "Netflix", "Twilio", "Datadog", "Snowflake",
    "B2C2", "Squarepoint Capital",
}

LETTER_TO_NUM = {"SS": 5, "S": 4, "A": 3, "B": 2, "C": 1, "F": 0}

DB_PATH = "state/cernio.db"


def connect_db():
    if not os.path.exists(DB_PATH):
        print(f"FATAL: {DB_PATH} not found. Run from repo root.", file=sys.stderr)
        sys.exit(2)
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    return conn


def fetch_candidates(conn):
    """Pull all eligible jobs (description > 500 chars, not archived) with their context."""
    cur = conn.cursor()
    cur.execute(
        """
        SELECT j.id, j.title, j.url, j.location, j.remote_policy,
               j.raw_description, j.fit_assessment, j.grade AS db_grade,
               c.name AS company_name, c.what_they_do
        FROM jobs j JOIN companies c ON c.id = j.company_id
        WHERE j.evaluation_status <> 'archived'
          AND LENGTH(j.raw_description) > 500
        """
    )
    return [dict(r) for r in cur.fetchall()]


def classify_stress_patterns(job):
    """Return a list of stress-test patterns this job matches (may be empty)."""
    patterns = []
    title = (job["title"] or "").lower()
    desc = (job["raw_description"] or "").lower()
    company = (job["company_name"] or "")
    fit = (job["fit_assessment"] or "").lower()

    if company in PRESTIGE_FIRMS:
        patterns.append("prestige-firm")

    # Implicit-seniority disguise: SWE title + N+ years
    if any(k in title for k in ["software engineer", "software developer", "engineer"]):
        if "graduate" not in title and "junior" not in title and "intern" not in title:
            import re
            if re.search(r"\b[3-9]\+?\s*years?\b", desc) or re.search(r"\b\d+-\d+\s*years?\b", desc):
                patterns.append("seniority-disguise")

    # Currently-pursuing intern misfit
    if "intern" in title and ("currently pursuing" in desc or "current student" in desc or "expected graduation" in desc):
        patterns.append("intern-pursuing-misfit")

    # Role-type mismatch
    if any(k in title for k in ["analyst", "trader", "researcher", "data scientist"]):
        if "data scientist" not in title or "data scientist" in title:  # tag all of them; rubric will weigh
            patterns.append("role-type-mismatch")

    # Customer-facing borderline
    if "customer engagement" in desc or "interact with customers" in desc or "customer-engagement" in desc:
        patterns.append("customer-facing-borderline")

    # Stack-zero misfit (rough heuristic)
    if any(k in desc for k in ["kotlin", "android", "swift", "objective-c", "salesforce"]):
        if not any(k in desc for k in ["python", "typescript", "rust", "react"]):
            patterns.append("stack-zero-misfit")

    # Security clearance
    if any(k in desc for k in ["security clearance", "sc clearance", "dv clearance", "uk national"]):
        patterns.append("clearance-required")

    # Staff-tier comp band disguise
    # (look for >£200k or >$300k mention)
    import re
    if re.search(r"£\s*2\d\d,?\d{3}", desc) or re.search(r"£\s*[3-9]\d\d,?\d{3}", desc) or re.search(r"\$\s*[3-9]\d\d,?\d{3}", desc):
        if any(k in desc for k in ["lead", "own", "shape", "principal", "staff"]):
            patterns.append("staff-tier-disguise")

    # Trigger-phrase match in fit_assessment (the strongest signal)
    if any(phrase in fit for phrase in TRIGGER_PHRASES):
        patterns.append("trigger-phrase-in-assessment")

    return patterns


def classify_stability_patterns(job):
    """Return a list of stability-anchor patterns this job matches (may be empty)."""
    patterns = []
    title = (job["title"] or "").lower()
    desc = (job["raw_description"] or "").lower()
    company = (job["company_name"] or "")
    location = (job["location"] or "").lower()

    # Wide-funnel graduate
    if any(k in title for k in ["graduate", "new grad", "2026 grad"]):
        if company in WIDE_FUNNEL_GRAD:
            patterns.append("wide-funnel-grad")

    # Hard years-floor F
    import re
    if re.search(r"\b[5-9]\+?\s*years?\b", desc) or re.search(r"\b1\d\+?\s*years?\b", desc):
        patterns.append("hard-years-floor")

    # Location hard-fail (not London / Cambridge / Remote-UK)
    if location and not any(k in location for k in ["london", "cambridge", "remote", "uk"]):
        patterns.append("location-hard-fail")

    # Mid-tier UK fintech
    if company in MID_TIER_FINTECH:
        if any(k in title for k in ["graduate", "junior", "engineer"]):
            patterns.append("mid-tier-uk-fintech")

    # Off-stack mid-level
    if any(k in desc for k in ["android", "ios", "salesforce"]):
        patterns.append("off-stack-midlevel")

    # Legit-A engineering (graduate-explicit + portfolio stack)
    if any(k in title for k in ["graduate", "new grad", "junior"]):
        if any(k in desc for k in ["python", "rust", "typescript", "go"]):
            if company not in PRESTIGE_FIRMS:  # exclude trigger-trap firms
                patterns.append("legit-a-engineering")

    return patterns


def select_clusters(candidates, seed):
    """Stratified selection of 30 stress-test + 30 stability-anchor jobs."""
    random.seed(seed)

    # Classify every candidate
    enriched = []
    for job in candidates:
        stress = classify_stress_patterns(job)
        stability = classify_stability_patterns(job)
        enriched.append({
            **job,
            "stress_patterns": stress,
            "stability_patterns": stability,
        })

    # Cluster A: 30 jobs from stress-pattern matches
    stress_pool = [j for j in enriched if j["stress_patterns"]]
    # Cluster B: 30 jobs from stability-pattern matches
    stability_pool = [j for j in enriched if j["stability_patterns"] and not j["stress_patterns"]]

    # If pools are smaller than 30, augment from related buckets
    if len(stress_pool) < 30:
        # Augment with high-grade jobs that don't match stress patterns
        extras = [j for j in enriched if j["db_grade"] in ("SS", "S") and j not in stress_pool]
        random.shuffle(extras)
        stress_pool.extend(extras[: 30 - len(stress_pool)])

    if len(stability_pool) < 30:
        extras = [j for j in enriched if j["db_grade"] in ("B", "C") and j not in stability_pool and j not in stress_pool]
        random.shuffle(extras)
        stability_pool.extend(extras[: 30 - len(stability_pool)])

    random.shuffle(stress_pool)
    random.shuffle(stability_pool)

    cluster_a = stress_pool[:30]
    cluster_b = stability_pool[:30]

    # Ensure no overlap
    a_ids = {j["id"] for j in cluster_a}
    cluster_b = [j for j in cluster_b if j["id"] not in a_ids][:30]

    # If cluster_b shrunk, pad
    if len(cluster_b) < 30:
        all_used = a_ids | {j["id"] for j in cluster_b}
        extras = [j for j in enriched if j["id"] not in all_used]
        random.shuffle(extras)
        cluster_b.extend(extras[: 30 - len(cluster_b)])

    return cluster_a, cluster_b


def strip_grades(job):
    """Return a job record safe to put in any agent manifest — no grades, no assessments."""
    safe_keys = ["id", "title", "url", "location", "remote_policy",
                 "raw_description", "company_name", "what_they_do"]
    return {k: job[k] for k in safe_keys}


def write_manifest(working_dir, agent_id, job_records, seed):
    """Write a per-agent manifest with jobs in randomised order."""
    rng = random.Random(seed)
    shuffled = list(job_records)
    rng.shuffle(shuffled)
    stripped = [strip_grades(j) for j in shuffled]
    path = working_dir / f"manifest-agent-{agent_id}.json"
    with open(path, "w") as f:
        json.dump(stripped, f, indent=2)
    return path


def build_pairs(cluster_a_jobs, cluster_b_jobs, seed, count):
    """Build pair-up manifests for pairwise-ranking agents."""
    rng = random.Random(seed)
    pool = cluster_a_jobs + cluster_b_jobs
    pairs = []
    used = set()
    while len(pairs) < count and len(used) < len(pool) * 2:
        a, b = rng.sample(pool, 2)
        key = tuple(sorted([a["id"], b["id"]]))
        if key in used:
            continue
        used.add(key)
        pairs.append({
            "pair_id": f"p{len(pairs)+1:03d}",
            "job_a": strip_grades(a),
            "job_b": strip_grades(b),
        })
    return pairs


def main():
    if len(sys.argv) != 2:
        print("Usage: select-jobs.py <run-id>", file=sys.stderr)
        sys.exit(2)
    run_id = sys.argv[1]
    working_dir = Path(f"/tmp/test-grade-jobs-{run_id}")
    working_dir.mkdir(parents=True, exist_ok=True)

    # Seed the run from the run-id string (deterministic per run, fresh each run)
    seed_int = int.from_bytes(run_id.encode(), "big") % (2**31)

    conn = connect_db()
    candidates = fetch_candidates(conn)
    print(f"Eligible candidates (desc > 500 chars, not archived): {len(candidates)}")

    cluster_a, cluster_b = select_clusters(candidates, seed_int)
    print(f"Cluster A (stress tests): {len(cluster_a)} jobs")
    print(f"Cluster B (stability anchors): {len(cluster_b)} jobs")

    all_jobs = cluster_a + cluster_b
    print(f"Total selected: {len(all_jobs)} jobs")

    # Identify trigger cases (subset of cluster A)
    trigger_cases = [j for j in cluster_a if "trigger-phrase-in-assessment" in j["stress_patterns"]]
    print(f"Trigger cases identified: {len(trigger_cases)}")

    # Write the support files (with DB grades for analysis only)
    with open(working_dir / "jobs-all.json", "w") as f:
        json.dump([strip_grades(j) for j in all_jobs], f, indent=2)
    with open(working_dir / "cluster-a.json", "w") as f:
        json.dump([j["id"] for j in cluster_a], f, indent=2)
    with open(working_dir / "cluster-b.json", "w") as f:
        json.dump([j["id"] for j in cluster_b], f, indent=2)
    with open(working_dir / "trigger-cases.json", "w") as f:
        json.dump([j["id"] for j in trigger_cases], f, indent=2)

    # db-grades.json holds the DB grades for the analysis step (NEVER in any manifest)
    db_grades = {str(j["id"]): j["db_grade"] for j in all_jobs}
    with open(working_dir / "db-grades.json", "w") as f:
        json.dump(db_grades, f, indent=2)

    # Build per-agent manifests
    agent_assignments = {}  # agent_id -> list of jobs

    # Cluster A 10-job disjoint × 3
    a_chunks_10 = [cluster_a[i*10:(i+1)*10] for i in range(3)]
    for i, chunk in enumerate(a_chunks_10, 1):
        agent_id = f"cluster-a-10job-{i}"
        agent_assignments[agent_id] = chunk
        write_manifest(working_dir, agent_id, chunk, seed=seed_int + hash(agent_id))

    # Cluster A 15-job disjoint × 2
    a_chunks_15 = [cluster_a[i*15:(i+1)*15] for i in range(2)]
    for i, chunk in enumerate(a_chunks_15, 1):
        agent_id = f"cluster-a-15job-{i}"
        agent_assignments[agent_id] = chunk
        write_manifest(working_dir, agent_id, chunk, seed=seed_int + hash(agent_id))

    # Cluster A 30-job full × 1
    agent_id = "cluster-a-30job"
    agent_assignments[agent_id] = cluster_a
    write_manifest(working_dir, agent_id, cluster_a, seed=seed_int + hash(agent_id))

    # Cluster B 10-job disjoint × 3
    b_chunks_10 = [cluster_b[i*10:(i+1)*10] for i in range(3)]
    for i, chunk in enumerate(b_chunks_10, 1):
        agent_id = f"cluster-b-10job-{i}"
        agent_assignments[agent_id] = chunk
        write_manifest(working_dir, agent_id, chunk, seed=seed_int + hash(agent_id))

    # Cluster B 15-job disjoint × 2
    b_chunks_15 = [cluster_b[i*15:(i+1)*15] for i in range(2)]
    for i, chunk in enumerate(b_chunks_15, 1):
        agent_id = f"cluster-b-15job-{i}"
        agent_assignments[agent_id] = chunk
        write_manifest(working_dir, agent_id, chunk, seed=seed_int + hash(agent_id))

    # Cluster B 30-job full × 1
    agent_id = "cluster-b-30job"
    agent_assignments[agent_id] = cluster_b
    write_manifest(working_dir, agent_id, cluster_b, seed=seed_int + hash(agent_id))

    # Cross-cluster × 2 (different 15A + 15B per agent)
    rng = random.Random(seed_int + 12345)
    a_shuffled_1 = list(cluster_a)
    b_shuffled_1 = list(cluster_b)
    rng.shuffle(a_shuffled_1)
    rng.shuffle(b_shuffled_1)
    cross_1 = a_shuffled_1[:15] + b_shuffled_1[:15]
    agent_id = "cross-cluster-1"
    agent_assignments[agent_id] = cross_1
    write_manifest(working_dir, agent_id, cross_1, seed=seed_int + hash(agent_id))

    a_shuffled_2 = list(cluster_a)
    b_shuffled_2 = list(cluster_b)
    rng = random.Random(seed_int + 67890)
    rng.shuffle(a_shuffled_2)
    rng.shuffle(b_shuffled_2)
    cross_2 = a_shuffled_2[:15] + b_shuffled_2[:15]
    agent_id = "cross-cluster-2"
    agent_assignments[agent_id] = cross_2
    write_manifest(working_dir, agent_id, cross_2, seed=seed_int + hash(agent_id))

    # Full-60 × 2
    for i in range(1, 3):
        agent_id = f"full-60-{i}"
        agent_assignments[agent_id] = all_jobs
        write_manifest(working_dir, agent_id, all_jobs, seed=seed_int + hash(agent_id))

    # Rubric-blind × 1 (full 60)
    agent_id = "rubric-blind"
    agent_assignments[agent_id] = all_jobs
    write_manifest(working_dir, agent_id, all_jobs, seed=seed_int + hash(agent_id))

    # Anchor-injected × 1 (full 60)
    agent_id = "anchor-injected"
    agent_assignments[agent_id] = all_jobs
    write_manifest(working_dir, agent_id, all_jobs, seed=seed_int + hash(agent_id))

    # Pairwise × 2 (~20 pairs each)
    pairs_1 = build_pairs(cluster_a, cluster_b, seed=seed_int + 11111, count=20)
    with open(working_dir / "manifest-agent-pairwise-1.json", "w") as f:
        json.dump(pairs_1, f, indent=2)

    pairs_2 = build_pairs(cluster_a, cluster_b, seed=seed_int + 22222, count=20)
    with open(working_dir / "manifest-agent-pairwise-2.json", "w") as f:
        json.dump(pairs_2, f, indent=2)

    print(f"Manifest files written: {len(list(working_dir.glob('manifest-agent-*.json')))}")

    # Build coverage matrix (which agents see which job)
    coverage = {}
    for agent_id, jobs in agent_assignments.items():
        for j in jobs:
            coverage.setdefault(str(j["id"]), []).append(agent_id)

    # Also tag pairwise coverage
    for pair in pairs_1:
        coverage.setdefault(str(pair["job_a"]["id"]), []).append("pairwise-1")
        coverage.setdefault(str(pair["job_b"]["id"]), []).append("pairwise-1")
    for pair in pairs_2:
        coverage.setdefault(str(pair["job_a"]["id"]), []).append("pairwise-2")
        coverage.setdefault(str(pair["job_b"]["id"]), []).append("pairwise-2")

    with open(working_dir / "coverage-matrix.json", "w") as f:
        json.dump(coverage, f, indent=2)

    coverage_counts = [len(v) for v in coverage.values()]
    print(f"Coverage: every job graded by {min(coverage_counts)}-{max(coverage_counts)} agents (mean {sum(coverage_counts)/len(coverage_counts):.1f})")

    # Zero-grade-leakage assertion
    leakage_found = False
    for manifest_path in working_dir.glob("manifest-agent-*.json"):
        with open(manifest_path) as f:
            content = f.read()
        if '"grade"' in content or '"fit_assessment"' in content or '"grade_reasoning"' in content or '"fit_score"' in content:
            print(f"LEAKAGE DETECTED in {manifest_path.name}", file=sys.stderr)
            leakage_found = True

    if leakage_found:
        print("ABORT: grade leakage detected in manifests. Fix the script.", file=sys.stderr)
        sys.exit(3)

    print("Zero grade leakage confirmed across all manifests.")
    print(f"Run ID: {run_id}")
    print(f"Working dir: {working_dir}")
    print("DONE.")


if __name__ == "__main__":
    main()
