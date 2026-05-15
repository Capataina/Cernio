#!/usr/bin/env python3
"""
analyse.py — Ingest per-agent outputs from the test-grade-jobs run and
compute intermediate analysis tables.

Reads:
    /tmp/test-grade-jobs-<run-id>/agent-*.md (the 20 per-agent outputs)
    /tmp/test-grade-jobs-<run-id>/coverage-matrix.json
    /tmp/test-grade-jobs-<run-id>/cluster-a.json
    /tmp/test-grade-jobs-<run-id>/cluster-b.json
    /tmp/test-grade-jobs-<run-id>/trigger-cases.json
    /tmp/test-grade-jobs-<run-id>/db-grades.json
    /tmp/test-grade-jobs-<run-id>/jobs-all.json

Writes (consumed by the agent to compose the final report):
    /tmp/test-grade-jobs-<run-id>/computed-per-job.json
    /tmp/test-grade-jobs-<run-id>/computed-agreement.json
    /tmp/test-grade-jobs-<run-id>/computed-batch-effect.json
    /tmp/test-grade-jobs-<run-id>/computed-cluster-position.json
    /tmp/test-grade-jobs-<run-id>/computed-trigger.json
    /tmp/test-grade-jobs-<run-id>/computed-q1-consistency.json
    /tmp/test-grade-jobs-<run-id>/computed-blind-comparison.json
    /tmp/test-grade-jobs-<run-id>/computed-anchor-effect.json
    /tmp/test-grade-jobs-<run-id>/computed-pairwise.json

stdout: number of grades parsed, agents ingested, jobs covered, plus
       any agents whose output was missing or malformed.
"""

import itertools
import json
import re
import statistics
import sys
from collections import Counter, defaultdict
from pathlib import Path


LETTER_TO_NUM = {"SS": 5, "S": 4, "A": 3, "B": 2, "C": 1, "F": 0}
NUM_TO_LETTER = {v: k for k, v in LETTER_TO_NUM.items()}

Q1_VERDICTS = {"cleared-decisively", "cleared-with-friction", "real-headwind", "hard-fail"}


def parse_grading_agent(path):
    """Parse a core / blind / anchor-injected agent's markdown output.

    Looks for the summary table and per-job grade letters. Returns a list of:
        {"job_id": int, "grade": str, "q1_verdict": str|None}
    """
    with open(path) as f:
        content = f.read()

    # Strategy: parse the markdown summary table at the end. It has
    # the columns: job_id | company | title | grade | Q1-verdict | reasoning
    # (or for blind: job_id | company | title | grade | reasoning)

    grades = []
    # Find the summary-table section
    m = re.search(r"##\s+Summary\s+(?:table|Table)(.*?)(?:\n##|\Z)", content, re.DOTALL)
    if not m:
        # Fallback: try to find per-job "Grade: X" lines and Q1 tags
        for job_match in re.finditer(
            r"##\s+Job\s+(\d+)[:\s].*?\*\*Grade:?\*\*\s*([A-Z]{1,2})", content, re.DOTALL
        ):
            grades.append({
                "job_id": int(job_match.group(1)),
                "grade": job_match.group(2),
                "q1_verdict": None,
            })
        return grades

    table = m.group(1)
    # Parse table rows like: | 2447 | HRT | SWE | C | real-headwind | ... |
    for row in re.finditer(r"^\s*\|\s*(\d+)\s*\|([^|]+)\|([^|]+)\|\s*([A-Z]{1,2})\s*\|(.*?)\|", table, re.MULTILINE):
        job_id = int(row.group(1))
        grade = row.group(4).strip()
        if grade not in LETTER_TO_NUM:
            continue  # skip header rows or invalid grades
        rest = row.group(5).strip().lower()
        q1_verdict = None
        for v in Q1_VERDICTS:
            if v in rest:
                q1_verdict = v
                break
        grades.append({
            "job_id": job_id,
            "grade": grade,
            "q1_verdict": q1_verdict,
        })

    # Also look for Q1 verdicts in the per-job assessments (not just the summary table)
    # Some agents may put Q1 in the per-job section but not in the table column
    if any(g["q1_verdict"] is None for g in grades):
        q1_map = {}
        for job_match in re.finditer(
            r"##\s+Job\s+(\d+)[:\s].*?Q1:\s*([a-z-]+)", content, re.DOTALL
        ):
            q1_map[int(job_match.group(1))] = job_match.group(2).strip()
        for g in grades:
            if g["q1_verdict"] is None and g["job_id"] in q1_map:
                g["q1_verdict"] = q1_map[g["job_id"]]

    return grades


def parse_pairwise_agent(path):
    """Parse a pairwise agent's output.

    Returns a list of:
        {"pair_id": str, "job_a_id": int, "job_b_id": int, "winner": str}
    """
    with open(path) as f:
        content = f.read()

    pairs = []
    # Look for the summary table
    m = re.search(r"##\s+Summary\s+(?:table|Table)(.*?)(?:\n##|\Z)", content, re.DOTALL)
    if not m:
        return pairs

    table = m.group(1)
    # Row shape: | p001 | job_a (company, title) | job_b (company, title) | winner | decisive Q |
    # The job-cell shape varies; we extract IDs from the per-pair sections instead
    pair_id_to_jobs = {}
    for pair_section in re.finditer(
        r"##\s+Pair\s+(p\d+):.*?(?=##\s+Pair|\Z)",
        content, re.DOTALL
    ):
        pid = pair_section.group(1)
        section_text = pair_section.group(0)
        # Look for Winner: a / b / tie
        wm = re.search(r"\*\*Winner:?\*\*\s*([abt][a-z]*)", section_text, re.IGNORECASE)
        if not wm:
            continue
        winner = wm.group(1).lower()
        if winner.startswith("a"):
            winner = "a"
        elif winner.startswith("b"):
            winner = "b"
        elif winner.startswith("t"):
            winner = "tie"

        pair_id_to_jobs[pid] = winner

    # We don't have job_id from the markdown directly; the orchestrator
    # has the manifest, so it can cross-reference. We just return what
    # we can parse.
    for pid, winner in pair_id_to_jobs.items():
        pairs.append({"pair_id": pid, "winner": winner})

    return pairs


def compute_per_job(grades_by_agent, coverage, db_grades, trigger_cases, jobs):
    """For each job, build the grade distribution across agents that graded it."""
    by_job = {}
    job_info = {str(j["id"]): j for j in jobs}

    for job_id_str, agent_ids in coverage.items():
        grades = []
        q1_verdicts = []
        for aid in agent_ids:
            agent_grades = grades_by_agent.get(aid, [])
            for g in agent_grades:
                if str(g["job_id"]) == job_id_str:
                    grades.append(g["grade"])
                    if g["q1_verdict"]:
                        q1_verdicts.append(g["q1_verdict"])
                    break

        if not grades:
            continue

        # Compute mode + range + agreement %
        try:
            mode = Counter(grades).most_common(1)[0][0]
        except IndexError:
            mode = None

        nums = [LETTER_TO_NUM[g] for g in grades if g in LETTER_TO_NUM]
        if len(nums) >= 2:
            grade_range = max(nums) - min(nums)
        else:
            grade_range = 0

        agreement = grades.count(mode) / len(grades) if mode else 0.0

        info = job_info.get(job_id_str, {})
        by_job[job_id_str] = {
            "job_id": int(job_id_str),
            "company": info.get("company_name", ""),
            "title": info.get("title", ""),
            "db_grade": db_grades.get(job_id_str),
            "grades_observed": grades,
            "mode": mode,
            "range_letters": grade_range,
            "agreement_pct": round(agreement * 100, 1),
            "q1_verdicts": q1_verdicts,
            "is_trigger_case": int(job_id_str) in trigger_cases,
        }

    return by_job


def compute_agreement(grades_by_agent, core_agent_ids):
    """Pairwise % exact match and % within-1-letter across core grading agents."""
    pairs = []
    for a1, a2 in itertools.combinations(core_agent_ids, 2):
        g1 = {g["job_id"]: g["grade"] for g in grades_by_agent.get(a1, [])}
        g2 = {g["job_id"]: g["grade"] for g in grades_by_agent.get(a2, [])}
        shared = set(g1.keys()) & set(g2.keys())
        if not shared:
            continue
        exact = sum(1 for j in shared if g1[j] == g2[j])
        within = sum(1 for j in shared
                     if g1[j] in LETTER_TO_NUM and g2[j] in LETTER_TO_NUM
                     and abs(LETTER_TO_NUM[g1[j]] - LETTER_TO_NUM[g2[j]]) <= 1)
        pairs.append({
            "agent_a": a1,
            "agent_b": a2,
            "shared_jobs": len(shared),
            "exact_match_pct": round(exact / len(shared) * 100, 1),
            "within_1_pct": round(within / len(shared) * 100, 1),
        })

    if not pairs:
        return {"mean_exact": 0.0, "mean_within_1": 0.0, "pairs": []}

    mean_exact = round(statistics.mean(p["exact_match_pct"] for p in pairs), 1)
    mean_within = round(statistics.mean(p["within_1_pct"] for p in pairs), 1)

    return {
        "mean_exact_match_pct": mean_exact,
        "mean_within_1_letter_pct": mean_within,
        "pairs": pairs,
    }


def compute_batch_effect(grades_by_agent, agent_meta):
    """Mean grade per job by batch-size bucket."""
    buckets = defaultdict(lambda: defaultdict(list))  # batch_size -> job_id -> [grades]
    for agent_id, grades in grades_by_agent.items():
        meta = agent_meta.get(agent_id, {})
        if meta.get("role") != "core-grading":
            continue
        batch = meta.get("batch_size")
        if not batch:
            continue
        for g in grades:
            buckets[batch][g["job_id"]].append(g["grade"])

    bucket_summary = {}
    for batch, jobs in buckets.items():
        per_job_means = []
        for job_id, grades in jobs.items():
            nums = [LETTER_TO_NUM[g] for g in grades if g in LETTER_TO_NUM]
            if nums:
                per_job_means.append(statistics.mean(nums))
        if per_job_means:
            bucket_summary[batch] = {
                "mean": round(statistics.mean(per_job_means), 2),
                "std": round(statistics.stdev(per_job_means) if len(per_job_means) > 1 else 0, 2),
                "n_jobs": len(per_job_means),
            }

    return bucket_summary


def compute_cluster_position(grades_by_agent, agent_meta, cluster_a, cluster_b):
    """For each job, compare grades from same-cluster agents vs cross-cluster agents."""
    by_job = {}
    a_set = set(cluster_a)
    b_set = set(cluster_b)

    for agent_id, grades in grades_by_agent.items():
        meta = agent_meta.get(agent_id, {})
        scope = meta.get("scope")
        for g in grades:
            jid = g["job_id"]
            grade_num = LETTER_TO_NUM.get(g["grade"])
            if grade_num is None:
                continue
            if jid not in by_job:
                by_job[jid] = {"same_cluster": [], "cross_cluster": [], "full": []}

            if scope == "cross":
                by_job[jid]["cross_cluster"].append(grade_num)
            elif scope == "a-only" and jid in a_set:
                by_job[jid]["same_cluster"].append(grade_num)
            elif scope == "b-only" and jid in b_set:
                by_job[jid]["same_cluster"].append(grade_num)
            elif scope == "full":
                by_job[jid]["full"].append(grade_num)

    result = {}
    for jid, data in by_job.items():
        if data["same_cluster"] and data["cross_cluster"]:
            same_mean = statistics.mean(data["same_cluster"])
            cross_mean = statistics.mean(data["cross_cluster"])
            result[str(jid)] = {
                "same_cluster_mean": round(same_mean, 2),
                "cross_cluster_mean": round(cross_mean, 2),
                "delta": round(cross_mean - same_mean, 2),
            }

    return result


def compute_trigger_correction(grades_by_agent, trigger_cases, agent_meta):
    """% of trigger cases that aggregated to C or F across core agents."""
    by_trigger = {}
    for tjid in trigger_cases:
        new_grades = []
        for agent_id, grades in grades_by_agent.items():
            meta = agent_meta.get(agent_id, {})
            if meta.get("role") != "core-grading":
                continue
            for g in grades:
                if g["job_id"] == tjid:
                    new_grades.append(g["grade"])
                    break
        if new_grades:
            corrected = sum(1 for g in new_grades if g in ("C", "F"))
            by_trigger[str(tjid)] = {
                "new_grades": new_grades,
                "mode": Counter(new_grades).most_common(1)[0][0],
                "corrected_count": corrected,
                "correction_rate": round(corrected / len(new_grades), 3),
            }

    if not by_trigger:
        return {"overall_correction_rate": None, "per_trigger": {}}

    overall = round(statistics.mean(d["correction_rate"] for d in by_trigger.values()), 3)
    return {"overall_correction_rate": overall, "per_trigger": by_trigger}


def compute_q1_consistency(per_job):
    """For each job, classify the (letter-agreement, Q1-agreement) cell."""
    cells = {
        "letter-agree_q1-agree": 0,
        "letter-agree_q1-disagree": 0,
        "letter-disagree_q1-agree": 0,
        "letter-disagree_q1-disagree": 0,
    }
    examples = {k: [] for k in cells}

    for jid, data in per_job.items():
        grades = data["grades_observed"]
        q1s = data["q1_verdicts"]
        if not grades or not q1s:
            continue
        letter_agreement = len(set(grades)) == 1
        q1_agreement = len(set(q1s)) == 1
        cell = f"letter-{'agree' if letter_agreement else 'disagree'}_q1-{'agree' if q1_agreement else 'disagree'}"
        cells[cell] += 1
        if len(examples[cell]) < 5:
            examples[cell].append({
                "job_id": jid,
                "company": data["company"],
                "grades": grades,
                "q1_verdicts": q1s,
            })

    return {"counts": cells, "examples": examples}


def compute_blind_comparison(grades_by_agent, agent_meta):
    """Compare rubric-blind distribution against rubric-loaded full-60 agents."""
    blind = None
    full60 = []
    for agent_id, grades in grades_by_agent.items():
        meta = agent_meta.get(agent_id, {})
        if meta.get("role") == "rubric-blind":
            blind = [g["grade"] for g in grades]
        elif meta.get("role") == "core-grading" and meta.get("batch_size") == 60:
            full60.extend(g["grade"] for g in grades)

    if not blind:
        return {"available": False, "reason": "rubric-blind agent produced no parseable output"}

    blind_dist = Counter(blind)
    full60_dist = Counter(full60)

    delta = {}
    for g in ("SS", "S", "A", "B", "C", "F"):
        delta[g] = blind_dist.get(g, 0) - full60_dist.get(g, 0)

    return {
        "available": True,
        "blind_distribution": dict(blind_dist),
        "full60_distribution": dict(full60_dist),
        "delta": delta,
    }


def compute_anchor_effect(grades_by_agent, agent_meta):
    """Compare anchor-injected distribution against plain full-60 agents."""
    anchor = None
    full60 = []
    for agent_id, grades in grades_by_agent.items():
        meta = agent_meta.get(agent_id, {})
        if meta.get("role") == "anchor-injected":
            anchor = [g["grade"] for g in grades]
        elif meta.get("role") == "core-grading" and meta.get("batch_size") == 60:
            full60.extend(g["grade"] for g in grades)

    if not anchor:
        return {"available": False, "reason": "anchor-injected agent produced no parseable output"}

    anchor_dist = Counter(anchor)
    full60_dist = Counter(full60)

    delta = {}
    for g in ("SS", "S", "A", "B", "C", "F"):
        delta[g] = anchor_dist.get(g, 0) - full60_dist.get(g, 0)

    return {
        "available": True,
        "anchor_distribution": dict(anchor_dist),
        "full60_distribution": dict(full60_dist),
        "delta": delta,
    }


def compute_pairwise_consistency(pairs_by_agent, per_job):
    """Compare pairwise winners against letter-grade ordering."""
    disagreements = []
    agreements = 0
    total = 0

    for agent_id, pairs in pairs_by_agent.items():
        for p in pairs:
            # We only have pair_id and winner from the markdown; we'd need
            # the manifest to map pair_id to job_a/job_b ids. For now, count
            # the parseable results and let the agent's report explain.
            total += 1
            # The full cross-check requires the orchestrator to load the
            # manifest; this script just emits raw counts.

    return {
        "total_pairs_parsed": total,
        "note": "Full letter-vs-pair cross-check is done by the report-composing agent using the manifests; this script just confirms the pairwise outputs were parseable.",
    }


def main():
    if len(sys.argv) != 2:
        print("Usage: analyse.py <run-id>", file=sys.stderr)
        sys.exit(2)
    run_id = sys.argv[1]
    working_dir = Path(f"/tmp/test-grade-jobs-{run_id}")

    if not working_dir.exists():
        print(f"FATAL: {working_dir} not found", file=sys.stderr)
        sys.exit(2)

    # Load support files
    with open(working_dir / "coverage-matrix.json") as f:
        coverage = json.load(f)
    with open(working_dir / "cluster-a.json") as f:
        cluster_a = json.load(f)
    with open(working_dir / "cluster-b.json") as f:
        cluster_b = json.load(f)
    with open(working_dir / "trigger-cases.json") as f:
        trigger_cases = json.load(f)
    with open(working_dir / "db-grades.json") as f:
        db_grades = json.load(f)
    with open(working_dir / "jobs-all.json") as f:
        jobs = json.load(f)

    # Define agent metadata (drives the analysis)
    agent_meta = {
        "cluster-a-10job-1": {"role": "core-grading", "batch_size": 10, "scope": "a-only"},
        "cluster-a-10job-2": {"role": "core-grading", "batch_size": 10, "scope": "a-only"},
        "cluster-a-10job-3": {"role": "core-grading", "batch_size": 10, "scope": "a-only"},
        "cluster-a-15job-1": {"role": "core-grading", "batch_size": 15, "scope": "a-only"},
        "cluster-a-15job-2": {"role": "core-grading", "batch_size": 15, "scope": "a-only"},
        "cluster-a-30job": {"role": "core-grading", "batch_size": 30, "scope": "a-only"},
        "cluster-b-10job-1": {"role": "core-grading", "batch_size": 10, "scope": "b-only"},
        "cluster-b-10job-2": {"role": "core-grading", "batch_size": 10, "scope": "b-only"},
        "cluster-b-10job-3": {"role": "core-grading", "batch_size": 10, "scope": "b-only"},
        "cluster-b-15job-1": {"role": "core-grading", "batch_size": 15, "scope": "b-only"},
        "cluster-b-15job-2": {"role": "core-grading", "batch_size": 15, "scope": "b-only"},
        "cluster-b-30job": {"role": "core-grading", "batch_size": 30, "scope": "b-only"},
        "cross-cluster-1": {"role": "core-grading", "batch_size": 30, "scope": "cross"},
        "cross-cluster-2": {"role": "core-grading", "batch_size": 30, "scope": "cross"},
        "full-60-1": {"role": "core-grading", "batch_size": 60, "scope": "full"},
        "full-60-2": {"role": "core-grading", "batch_size": 60, "scope": "full"},
        "rubric-blind": {"role": "rubric-blind", "batch_size": 60, "scope": "full"},
        "anchor-injected": {"role": "anchor-injected", "batch_size": 60, "scope": "full"},
        "pairwise-1": {"role": "pairwise", "batch_size": 20, "scope": "pairs"},
        "pairwise-2": {"role": "pairwise", "batch_size": 20, "scope": "pairs"},
    }

    # Parse every per-agent output file
    grades_by_agent = {}
    pairs_by_agent = {}
    parse_failures = []

    for agent_id, meta in agent_meta.items():
        path = working_dir / f"agent-{agent_id}.md"
        if not path.exists():
            parse_failures.append({"agent_id": agent_id, "reason": "file missing"})
            continue
        try:
            if meta["role"] == "pairwise":
                parsed = parse_pairwise_agent(path)
                pairs_by_agent[agent_id] = parsed
            else:
                parsed = parse_grading_agent(path)
                grades_by_agent[agent_id] = parsed
        except Exception as e:
            parse_failures.append({"agent_id": agent_id, "reason": f"parse error: {e}"})

    total_grades = sum(len(g) for g in grades_by_agent.values())
    total_pairs = sum(len(p) for p in pairs_by_agent.values())
    unique_jobs_covered = set()
    for grades in grades_by_agent.values():
        for g in grades:
            unique_jobs_covered.add(g["job_id"])

    print(f"Agents ingested: {len(grades_by_agent)} grading + {len(pairs_by_agent)} pairwise")
    print(f"Grades parsed: {total_grades}")
    print(f"Pairs parsed: {total_pairs}")
    print(f"Unique jobs covered: {len(unique_jobs_covered)}")
    if parse_failures:
        print(f"Parse failures: {len(parse_failures)}")
        for pf in parse_failures:
            print(f"  - {pf['agent_id']}: {pf['reason']}")

    # Run computations
    core_agent_ids = [aid for aid, meta in agent_meta.items() if meta["role"] == "core-grading"]

    per_job = compute_per_job(grades_by_agent, coverage, db_grades, trigger_cases, jobs)
    agreement = compute_agreement(grades_by_agent, core_agent_ids)
    batch_effect = compute_batch_effect(grades_by_agent, agent_meta)
    cluster_position = compute_cluster_position(grades_by_agent, agent_meta, cluster_a, cluster_b)
    trigger = compute_trigger_correction(grades_by_agent, trigger_cases, agent_meta)
    q1_consistency = compute_q1_consistency(per_job)
    blind = compute_blind_comparison(grades_by_agent, agent_meta)
    anchor = compute_anchor_effect(grades_by_agent, agent_meta)
    pairwise = compute_pairwise_consistency(pairs_by_agent, per_job)

    # Write intermediate files
    outputs = {
        "computed-per-job.json": per_job,
        "computed-agreement.json": agreement,
        "computed-batch-effect.json": batch_effect,
        "computed-cluster-position.json": cluster_position,
        "computed-trigger.json": trigger,
        "computed-q1-consistency.json": q1_consistency,
        "computed-blind-comparison.json": blind,
        "computed-anchor-effect.json": anchor,
        "computed-pairwise.json": pairwise,
        "computed-parse-failures.json": parse_failures,
    }
    for name, data in outputs.items():
        with open(working_dir / name, "w") as f:
            json.dump(data, f, indent=2)
        print(f"Wrote {name}")

    print("DONE.")


if __name__ == "__main__":
    main()
