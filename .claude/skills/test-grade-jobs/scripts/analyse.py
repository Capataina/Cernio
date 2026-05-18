#!/usr/bin/env python3
"""
analyse.py — Multi-axis structural measurement of grade-jobs' output.

Parses per-agent markdown outputs from a test-grade-jobs run, extracts
structured Q-slots and grade letters via prose parsing, and computes seven
structural axes:

  A. Format adherence       — does output follow rubric's slot structure
  B. Reasoning specificity  — citation density vs generic-phrase rate
  C. Q3a/Q3b differentiation — distinctness of stack-overlap vs career-axis slots
  D. Internal consistency   — Verdict↔Grade alignment, Q-slot coherence
  E. Inter-agent variance   — exact-letter agreement, within-1
  F. Pairwise consistency   — cross-agent pair agreement, transitivity
  G. Risk acknowledgment    — risk-naming density, risk-direction correlation

None of these metrics reference any external "correct answer." The script
does not assert which grades are right; it measures grade-jobs' own
structural properties and reports the numbers as-is.

Cross-run regression diff: if context/test-runs/baseline.json exists, the
script computes per-axis deltas and writes the new scores to that file,
appending the prior baseline to baseline-history.json.

Usage:
    python3 analyse.py <run-id>

Writes intermediate JSONs to /tmp/test-grade-jobs-<run-id>/computed-*.json
and updates context/test-runs/baseline.json + baseline-history.json.

stdout: per-axis scores + regression-diff summary.
"""

import json
import os
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

LETTER_TO_NUM = {"SS": 5, "S": 4, "A": 3, "B": 2, "C": 1, "F": 0}
NUM_TO_LETTER = {v: k for k, v in LETTER_TO_NUM.items()}

# Axis A: banned strings inside Q-slot prose (per grade-jobs' no-verdict-enums rule)
BANNED_IN_SLOT_PROSE = [
    "cleared-decisively", "cleared decisively",
    "cleared-with-friction", "cleared with friction",
    "real-headwind", "real headwind",
    "hard-fail",
    # Arrow shorthand
    "→ A", "→ B", "→ C", "→ S", "→ SS", "→ F",
    "-> A", "-> B", "-> C", "-> S", "-> SS", "-> F",
    # Label-only Q reporting
    "Q3 moderate", "Q3 strong", "Q3 weak",
    "Q2 strong", "Q2 moderate", "Q2 weak",
    "Q1 cleared", "Q5 ✓", "Q4 ✓",
]

GENERIC_PHRASES = [
    "good company", "decent fit", "relevant experience", "strong tech stack",
    "great role", "good role", "good fit", "decent role", "solid choice",
    "broadly relevant", "interesting role", "reasonable match",
    "strong company", "decent company", "fine fit",
]

CAREER_AXIS_TERMS = [
    "on-axis", "off-axis", "adjacent", "trajectory", "career-axis",
    "career axis", "build toward", "career launch", "axis bet",
    "specialism", "career trajectory", "kind of engineer",
]

HARD_FLOOR_PATTERNS = [
    r"\b(5\+|6\+|7\+|8\+|9\+|10\+)\s*years\b",
    r"\b[5-9]-\d+\s*years\b",
    r"\bstaff-level\b", r"\bprincipal-level\b",
    r"£200\s*-?\s*\d+k", r"£250\s*-?\s*\d+k", r"£300\s*-?\s*\d+k",
    r"\bsenior staff\b", r"\bsenior principal\b",
    r"\bdistinguished engineer\b",
]

VERDICT_POSITIVE = [
    "axis bet", "career launch", "make the cut", "compelling",
    "strong pull", "would make the cut",
]
VERDICT_NEGATIVE = [
    "does not make the cut", "deadweight",
    "not worth", "would not make the cut",
]

RISK_PHRASES = [
    "friction", "gap", "off-axis", "stretch", "concern",
    "headwind", "soft floor", "narrow funnel", "selectivity",
    "credential floor", "stack mismatch", "career-axis mismatch",
    "trade-off", "tradeoff",
]

STOPWORDS = set("the a an and or of in on for to with from by as is are was were be been being it that this its their there here have has had do does did but not no so if when while which who whom whose what where why how all any some many more most much".split())


def get_project_filenames():
    project_dir = Path("profile/projects")
    if not project_dir.exists():
        return []
    return [p.stem for p in project_dir.glob("*.md") if not p.stem.startswith("_")]


def get_skills_entries():
    skills_path = Path("profile/skills.md")
    if not skills_path.exists():
        return []
    text = skills_path.read_text()
    m = re.search(r"##\s*Concepts and Domains.*?(?=^##|\Z)", text, re.S | re.M)
    if not m:
        return []
    section = m.group(0)
    entries = re.findall(r"\|\s*([A-Z][A-Za-z0-9 \-/&,]+?)\s*\|", section)
    return list(set(e.strip() for e in entries if 4 <= len(e.strip()) <= 60))


def extract_batch_size(agent_id):
    if "10job" in agent_id:
        return 10
    if "15job" in agent_id:
        return 15
    if "30job" in agent_id:
        return 30
    if "cross-cluster" in agent_id:
        return 30
    return None


def extract_cluster_scope(agent_id):
    if "cluster-a" in agent_id:
        return "a-only"
    if "cluster-b" in agent_id:
        return "b-only"
    if "cross-cluster" in agent_id:
        return "cross"
    return "unknown"


def parse_q_slots(sect):
    """Extract Q1, Q2, Q3a, Q3b, Q4, Q5, Verdict prose from a job section.

    Robust to multiple header styles agents may use:
      ## Q1 — ...   /  ### Q1 — ...   /  **Q1 — ...**
    """
    slots = {f"q{n}_prose": "" for n in ["1", "2", "3a", "3b", "4", "5"]}
    slots["verdict_prose"] = ""

    # Heading-line regex: matches ##, ###, or **bold** styles
    # for a given Q-slot name. Returns the content from the heading to the next
    # Q-slot heading or the Grade line.
    def heading_pattern(slot_name, next_names):
        # Build alternation for next-section headers
        next_alts = []
        for n in next_names:
            next_alts.append(rf"^#{{2,3}}\s*{re.escape(n)}\b")
            next_alts.append(rf"^\*\*\s*{re.escape(n)}\b")
        next_alts.append(r"^Grade\s*:")
        next_alts.append(r"^evidence_basis\s*:")
        next_alt_re = "|".join(next_alts)
        # Header for this slot can be:
        #   ##\s*Q1   /  ###\s*Q1  /  **Q1...** (bold)
        header_re = rf"(?:^#{{2,3}}\s*{re.escape(slot_name)}\b[^\n]*\n|^\*\*\s*{re.escape(slot_name)}\b[^\n*]*\*\*\s*)"
        return rf"{header_re}(.+?)(?={next_alt_re}|\Z)"

    q_seq = [
        ("Q1", "q1_prose", ["Q2", "Q3a", "Q3", "Verdict"]),
        ("Q2", "q2_prose", ["Q3a", "Q3", "Q4", "Verdict"]),
        ("Q3a", "q3a_prose", ["Q3b", "Q4", "Verdict"]),
        ("Q3b", "q3b_prose", ["Q4", "Verdict"]),
        ("Q3", "q3a_prose", ["Q4", "Verdict"]),  # Some agents use unified Q3
        ("Q4", "q4_prose", ["Q5", "Verdict"]),
        ("Q5", "q5_prose", ["Verdict"]),
        ("Verdict", "verdict_prose", ["Summary table", "Job"]),
    ]
    for slot_name, key, nexts in q_seq:
        # Only write if slot is still empty (preserve Q3a if Q3 unified parse hits later)
        if slots.get(key):
            continue
        pat = heading_pattern(slot_name, nexts)
        m = re.search(pat, sect, re.M | re.S)
        if m:
            slots[key] = m.group(1).strip()

    return slots


def parse_grade_letter(sect):
    m = re.search(r"^\s*\**\s*Grade:\s*\**\s*([SAFBC]{1,2})\s*\**\s*$", sect, re.M)
    if m and m.group(1) in LETTER_TO_NUM:
        return m.group(1)
    m = re.search(r"Grade:\s*\**\s*([SAFBC]{1,2})\b", sect)
    if m and m.group(1) in LETTER_TO_NUM:
        return m.group(1)
    if re.search(r"Grade:\s*\**\s*(NULL|N/A)\b", sect, re.I):
        return None
    return None


def parse_evidence_basis(sect):
    m = re.search(r"evidence_basis:\s*\**\s*(jd|semantic|insufficient)\b", sect, re.I)
    if m:
        return m.group(1).lower()
    return None


def parse_pairwise_output(text, agent_id):
    pairs = []
    sections = re.split(r"^---+\s*$", text, flags=re.M)
    for sect in sections:
        m = re.match(r"\s*##\s*Pair\s+(\S+)\s*:", sect, re.M)
        if not m:
            continue
        pair_id = m.group(1).rstrip(":")
        winner_m = re.search(r"\*\*Winner:?\*\*\s*([abAB]|tie)", sect)
        winner = winner_m.group(1).lower() if winner_m else None
        pairs.append({"pair_id": pair_id, "winner": winner, "section": sect})
    return {
        "agent_id": agent_id, "role": "pairwise", "batch_size": None,
        "cluster_scope": None, "pairs": pairs, "n_pairs": len(pairs),
    }


def parse_agent_output(path):
    if not path.exists():
        return None
    text = path.read_text()
    agent_id = path.stem.replace("agent-", "")
    role = "pairwise" if "pairwise" in agent_id else "core-grading"

    if role == "pairwise":
        return parse_pairwise_output(text, agent_id)

    batch_size = extract_batch_size(agent_id)
    cluster_scope = extract_cluster_scope(agent_id)
    sections = re.split(r"^---+\s*$", text, flags=re.M)
    assessments = []
    for sect in sections:
        # Accept multiple Job-header styles: `## Job 592:`, `## Job 592 —`,
        # `### Job 592:`, `## Job 750` (no separator + title on next line), etc.
        m = re.search(r"^#{2,3}\s*Job\s+(\d+)\s*([:\-—–]\s*(.+?))?\s*$", sect, re.M)
        if not m:
            continue
        job_id = int(m.group(1))
        header = (m.group(3) or "").strip()
        q_slots = parse_q_slots(sect)
        grade = parse_grade_letter(sect)
        evidence_basis = parse_evidence_basis(sect)
        assessments.append({
            "job_id": job_id, "header": header,
            "grade": grade, "evidence_basis": evidence_basis,
            **q_slots, "full_text": sect,
        })

    return {
        "agent_id": agent_id, "role": role,
        "batch_size": batch_size, "cluster_scope": cluster_scope,
        "assessments": assessments, "n_assessments": len(assessments),
    }


def quoted_substring_present(slot_text, jd, min_len=12):
    quotes = re.findall(r'"([^"]+)"|\'([^\']+)\'', slot_text)
    flat = [q[0] or q[1] for q in quotes]
    for q in flat:
        if len(q) >= min_len and q.lower() in jd.lower():
            return True
    return False


def axis_a_format(agent_data, project_names, run_dir):
    if agent_data["role"] != "core-grading":
        return None
    n = len(agent_data["assessments"])
    if n == 0:
        return {"score": 0, "n_assessments": 0}

    results = Counter()
    jobs_all_path = run_dir / "jobs-all.json"
    jd_by_id = {}
    if jobs_all_path.exists():
        for j in json.loads(jobs_all_path.read_text()):
            jd_by_id[j["id"]] = j.get("raw_description", "") or ""

    q1_quote_eligible = 0
    q3a_quote_eligible = 0

    for asmt in agent_data["assessments"]:
        if all(asmt.get(f"q{x}_prose") for x in ["1", "2", "3a", "3b", "4", "5"]) and asmt.get("verdict_prose"):
            results["all_seven_slots"] += 1
        if asmt["grade"] is not None or asmt["evidence_basis"] == "insufficient":
            results["grade_line"] += 1
        if asmt["evidence_basis"] in ("jd", "semantic", "insufficient"):
            results["evidence_basis_set"] += 1
        any_project = any(
            p.lower() in (asmt.get("q3a_prose", "") + " " + asmt.get("q3b_prose", "")).lower()
            for p in project_names
        )
        if any_project:
            results["project_anchor"] += 1
        slot_text = " ".join(asmt.get(f"q{x}_prose", "") for x in ["1", "2", "3a", "3b", "4", "5"])
        slot_text += " " + asmt.get("verdict_prose", "")
        if not any(b.lower() in slot_text.lower() for b in BANNED_IN_SLOT_PROSE):
            results["no_banned"] += 1
        if asmt["evidence_basis"] == "jd":
            q1_quote_eligible += 1
            q3a_quote_eligible += 1
            jd = jd_by_id.get(asmt["job_id"], "")
            if jd and quoted_substring_present(asmt.get("q1_prose", ""), jd):
                results["q1_jd_quote"] += 1
            if jd and quoted_substring_present(asmt.get("q3a_prose", ""), jd):
                results["q3a_jd_quote"] += 1

    pct = {
        "all_seven_slots": results["all_seven_slots"] / n,
        "grade_line": results["grade_line"] / n,
        "project_anchor": results["project_anchor"] / n,
        "no_banned": results["no_banned"] / n,
        "evidence_basis": results["evidence_basis_set"] / n,
        "q1_jd_quote": (results["q1_jd_quote"] / q1_quote_eligible) if q1_quote_eligible else 1.0,
        "q3a_jd_quote": (results["q3a_jd_quote"] / q3a_quote_eligible) if q3a_quote_eligible else 1.0,
    }
    weights = {"all_seven_slots": 0.20, "grade_line": 0.10, "project_anchor": 0.15,
               "no_banned": 0.20, "evidence_basis": 0.10,
               "q1_jd_quote": 0.125, "q3a_jd_quote": 0.125}
    score = sum(pct[k] * weights[k] for k in pct) * 100
    return {
        "score": round(score, 1),
        "details": {k: round(v * 100, 1) for k, v in pct.items()},
        "n_assessments": n,
    }


def axis_b_specificity(agent_data, project_names, skills_entries):
    if agent_data["role"] != "core-grading":
        return None
    n = len(agent_data["assessments"])
    if n == 0:
        return {"score": 0}
    total_generic = 0
    total_words = 0
    total_specific_refs = 0
    project_diversity = set()
    for asmt in agent_data["assessments"]:
        text = asmt.get("full_text", "")
        total_words += len(re.findall(r"\b\w+\b", text))
        for gp in GENERIC_PHRASES:
            total_generic += text.lower().count(gp)
        quotes = re.findall(r'"([^"]{8,})"', text)
        total_specific_refs += len(quotes)
        for proj in project_names:
            if proj.lower() in text.lower():
                total_specific_refs += 1
                project_diversity.add(proj)
        for entry in skills_entries:
            if entry.lower() in text.lower():
                total_specific_refs += 1
    specificity_density = (total_specific_refs / total_words * 100) if total_words else 0
    generic_per_asmt = total_generic / n
    project_diversity_score = min(len(project_diversity) / 10.0, 1.0)
    raw = (specificity_density * 0.6) + (project_diversity_score * 30) + (max(0, 1 - generic_per_asmt) * 10)
    score = min(raw, 100)
    return {
        "score": round(score, 1),
        "specificity_density": round(specificity_density, 2),
        "generic_per_assessment": round(generic_per_asmt, 2),
        "project_diversity": len(project_diversity),
        "n_assessments": n,
    }


def axis_c_differentiation(agent_data):
    if agent_data["role"] != "core-grading":
        return None
    n_with_both = 0
    total_overlap = 0
    q3b_career_axis = 0
    q3a_jd_tech = 0
    for asmt in agent_data["assessments"]:
        q3a = asmt.get("q3a_prose", "")
        q3b = asmt.get("q3b_prose", "")
        if not (q3a and q3b):
            continue
        n_with_both += 1
        a_words = set(w.lower() for w in re.findall(r"\b\w{4,}\b", q3a) if w.lower() not in STOPWORDS)
        b_words = set(w.lower() for w in re.findall(r"\b\w{4,}\b", q3b) if w.lower() not in STOPWORDS)
        if a_words | b_words:
            total_overlap += len(a_words & b_words) / len(a_words | b_words)
        if any(t in q3b.lower() for t in CAREER_AXIS_TERMS):
            q3b_career_axis += 1
        if re.search(r'"[^"]{4,}"', q3a):
            q3a_jd_tech += 1
    if n_with_both == 0:
        return {"score": 0, "n_with_both_slots": 0}
    mean_overlap = total_overlap / n_with_both
    q3b_career_pct = q3b_career_axis / n_with_both
    q3a_jd_tech_pct = q3a_jd_tech / n_with_both
    score = (1 - mean_overlap) * 40 + q3b_career_pct * 30 + q3a_jd_tech_pct * 30
    return {
        "score": round(score, 1),
        "mean_overlap_jaccard": round(mean_overlap, 3),
        "q3b_career_axis_pct": round(q3b_career_pct * 100, 1),
        "q3a_jd_tech_pct": round(q3a_jd_tech_pct * 100, 1),
        "n_with_both_slots": n_with_both,
    }


def axis_d_consistency(agent_data):
    if agent_data["role"] != "core-grading":
        return None
    n = len(agent_data["assessments"])
    if n == 0:
        return {"score": 0}
    verdict_grade_aligned = 0
    q1_hardfloor_coherent = 0
    q1_hardfloor_eligible = 0
    risk_engaged = 0
    risk_eligible = 0
    n_graded = 0
    for asmt in agent_data["assessments"]:
        verdict = asmt.get("verdict_prose", "").lower()
        grade = asmt["grade"]
        if grade is None:
            continue
        n_graded += 1
        verdict_pos = any(p in verdict for p in VERDICT_POSITIVE)
        verdict_neg = any(p in verdict for p in VERDICT_NEGATIVE)
        grade_high = grade in ("SS", "S", "A")
        grade_low = grade in ("C", "F")
        if (verdict_pos and grade_high) or (verdict_neg and grade_low) or (not verdict_pos and not verdict_neg):
            verdict_grade_aligned += 1
        q1 = asmt.get("q1_prose", "").lower()
        if any(re.search(p, q1) for p in HARD_FLOOR_PATTERNS):
            q1_hardfloor_eligible += 1
            if grade == "F":
                q1_hardfloor_coherent += 1
        q3b = asmt.get("q3b_prose", "").lower()
        if any(r in q3b for r in RISK_PHRASES):
            risk_eligible += 1
            if any(r in verdict for r in RISK_PHRASES) or any(t in verdict for t in ["pushback", "trade-off", "tradeoff"]):
                risk_engaged += 1
    if n_graded == 0:
        return {"score": 0}
    verdict_pct = verdict_grade_aligned / n_graded
    q1_pct = (q1_hardfloor_coherent / q1_hardfloor_eligible) if q1_hardfloor_eligible else 1.0
    risk_pct = (risk_engaged / risk_eligible) if risk_eligible else 1.0
    # Weights sum to 100, percentages are 0-1 → score is 0-100 directly
    score = verdict_pct * 40 + q1_pct * 30 + risk_pct * 30
    return {
        "score": round(score, 1),
        "verdict_grade_aligned_pct": round(verdict_pct * 100, 1),
        "q1_hardfloor_coherent_pct": round(q1_pct * 100, 1),
        "q1_hardfloor_eligible_n": q1_hardfloor_eligible,
        "risk_engaged_pct": round(risk_pct * 100, 1),
        "risk_eligible_n": risk_eligible,
    }


def axis_e_inter_agent(agents):
    import itertools
    core = [a for a in agents if a["role"] == "core-grading"]
    job_grades = defaultdict(list)
    for agent in core:
        for asmt in agent["assessments"]:
            if asmt["grade"]:
                job_grades[asmt["job_id"]].append(asmt["grade"])
    exact_matches = 0
    within_1_matches = 0
    total_shared_pairs = 0
    for a1, a2 in itertools.combinations(core, 2):
        a1_grades = {a["job_id"]: a["grade"] for a in a1["assessments"] if a["grade"]}
        a2_grades = {a["job_id"]: a["grade"] for a in a2["assessments"] if a["grade"]}
        shared = set(a1_grades) & set(a2_grades)
        for j in shared:
            total_shared_pairs += 1
            if a1_grades[j] == a2_grades[j]:
                exact_matches += 1
            if abs(LETTER_TO_NUM[a1_grades[j]] - LETTER_TO_NUM[a2_grades[j]]) <= 1:
                within_1_matches += 1
    if total_shared_pairs == 0:
        return {"score": 0, "total_shared_pairs": 0, "per_job_grades": {}}
    exact_pct = exact_matches / total_shared_pairs
    within1_pct = within_1_matches / total_shared_pairs
    job_ranges = {}
    for jid, grades in job_grades.items():
        if len(grades) >= 2:
            nums = [LETTER_TO_NUM[g] for g in grades]
            job_ranges[jid] = max(nums) - min(nums)
    # Weights are percentages (60+40=100) and pcts are 0-1 → 0-100 directly
    score = exact_pct * 60 + within1_pct * 40
    return {
        "score": round(score, 1),
        "exact_match_pct": round(exact_pct * 100, 1),
        "within_1_letter_pct": round(within1_pct * 100, 1),
        "total_shared_pairs": total_shared_pairs,
        "mean_per_job_range": round(sum(job_ranges.values()) / len(job_ranges), 2) if job_ranges else 0,
        "max_per_job_range": max(job_ranges.values()) if job_ranges else 0,
        "per_job_grades": {str(jid): grades for jid, grades in job_grades.items()},
    }


def axis_f_pairwise(agents):
    pw_agents = [a for a in agents if a["role"] == "pairwise"]
    if len(pw_agents) < 2:
        return {"score": 0, "available": False, "reason": "fewer than 2 pairwise agents"}
    pw1 = {p["pair_id"]: p["winner"] for p in pw_agents[0]["pairs"]}
    pw2 = {p["pair_id"]: p["winner"] for p in pw_agents[1]["pairs"]}
    shared = set(pw1) & set(pw2)
    cross_agree = sum(1 for p in shared if pw1[p] == pw2[p] and pw1[p] is not None)
    cross_pct = (cross_agree / len(shared)) if shared else None
    total_decisions = len(pw_agents[0]["pairs"]) + len(pw_agents[1]["pairs"])
    ties = sum(1 for a in pw_agents for p in a["pairs"] if p["winner"] == "tie")
    tie_rate = ties / total_decisions if total_decisions else 0
    if cross_pct is not None:
        score = cross_pct * 70 + (1 - tie_rate) * 30
    else:
        score = (1 - tie_rate) * 70
    return {
        "score": round(score, 1),
        "cross_agent_agreement_pct": round(cross_pct * 100, 1) if cross_pct is not None else None,
        "shared_pairs_n": len(shared),
        "tie_rate_pct": round(tie_rate * 100, 1),
        "total_decisions": total_decisions,
    }


def axis_g_risk(agent_data):
    if agent_data["role"] != "core-grading":
        return None
    n = len(agent_data["assessments"])
    if n == 0:
        return {"score": 0}
    asmts_with_risk = 0
    total_risks = 0
    risks_in_q3b = 0
    grades_with_risk = []
    grades_without_risk = []
    for asmt in agent_data["assessments"]:
        text = asmt.get("full_text", "").lower()
        q3b = asmt.get("q3b_prose", "").lower()
        risk_count = sum(text.count(p) for p in RISK_PHRASES)
        total_risks += risk_count
        if risk_count > 0:
            asmts_with_risk += 1
        if any(p in q3b for p in RISK_PHRASES):
            risks_in_q3b += 1
        if asmt["grade"] is not None:
            if risk_count > 0:
                grades_with_risk.append(LETTER_TO_NUM[asmt["grade"]])
            else:
                grades_without_risk.append(LETTER_TO_NUM[asmt["grade"]])
    risk_named_pct = asmts_with_risk / n
    risks_in_q3b_pct = risks_in_q3b / n
    mean_grade_with = sum(grades_with_risk) / len(grades_with_risk) if grades_with_risk else None
    mean_grade_without = sum(grades_without_risk) / len(grades_without_risk) if grades_without_risk else None
    risk_direction_delta = (mean_grade_without - mean_grade_with) if (mean_grade_with is not None and mean_grade_without is not None) else None
    delta_normalised = 0.5
    if risk_direction_delta is not None:
        delta_normalised = max(0, min(1, (risk_direction_delta + 1) / 2))
    # Weights sum to 100; percentages are 0-1 → score is 0-100 directly
    score = risk_named_pct * 40 + risks_in_q3b_pct * 30 + delta_normalised * 30
    return {
        "score": round(score, 1),
        "risk_named_pct": round(risk_named_pct * 100, 1),
        "risks_in_q3b_pct": round(risks_in_q3b_pct * 100, 1),
        "mean_risks_per_assessment": round(total_risks / n, 2),
        "risk_direction_delta_letters": round(risk_direction_delta, 2) if risk_direction_delta is not None else None,
    }


def main():
    if len(sys.argv) < 2:
        print("Usage: analyse.py <run-id>", file=sys.stderr)
        sys.exit(2)
    run_id = sys.argv[1]
    run_dir = Path(f"/tmp/test-grade-jobs-{run_id}")
    if not run_dir.exists():
        print(f"FATAL: {run_dir} not found.", file=sys.stderr)
        sys.exit(2)

    agents = []
    for agent_md in sorted(run_dir.glob("agent-*.md")):
        parsed = parse_agent_output(agent_md)
        if parsed:
            agents.append(parsed)
    n_grading = sum(1 for a in agents if a["role"] == "core-grading")
    n_pairwise = sum(1 for a in agents if a["role"] == "pairwise")
    print(f"Agents parsed: {len(agents)} ({n_grading} grading + {n_pairwise} pairwise)")

    project_names = get_project_filenames()
    skills_entries = get_skills_entries()
    print(f"Project anchors loaded: {len(project_names)} | Skills entries: {len(skills_entries)}")

    per_axis = {}
    per_agent_scores = {}
    for agent in agents:
        if agent["role"] == "pairwise":
            continue
        per_agent_scores[agent["agent_id"]] = {
            "axis_a": axis_a_format(agent, project_names, run_dir),
            "axis_b": axis_b_specificity(agent, project_names, skills_entries),
            "axis_c": axis_c_differentiation(agent),
            "axis_d": axis_d_consistency(agent),
            "axis_g": axis_g_risk(agent),
        }
    for axis_key in ("axis_a", "axis_b", "axis_c", "axis_d", "axis_g"):
        scores = [v[axis_key]["score"] for v in per_agent_scores.values()
                  if v.get(axis_key) and v[axis_key].get("score") is not None]
        per_axis[axis_key] = round(sum(scores) / len(scores), 1) if scores else 0

    e = axis_e_inter_agent(agents)
    per_axis["axis_e"] = e["score"]
    f_score = axis_f_pairwise(agents)
    per_axis["axis_f"] = f_score["score"]

    composite = round(sum(per_axis.values()) / len(per_axis), 1)

    with open(run_dir / "computed-per-agent.json", "w") as fh:
        json.dump(per_agent_scores, fh, indent=2, default=str)
    e_disk = {k: v for k, v in e.items() if k != "per_job_grades"}
    e_disk["per_job_grades_count"] = len(e.get("per_job_grades", {}))
    with open(run_dir / "computed-axis-e.json", "w") as fh:
        json.dump(e_disk, fh, indent=2, default=str)
    with open(run_dir / "computed-axis-f.json", "w") as fh:
        json.dump(f_score, fh, indent=2, default=str)
    with open(run_dir / "computed-axes-summary.json", "w") as fh:
        json.dump({**per_axis, "composite": composite}, fh, indent=2)
    with open(run_dir / "computed-per-job.json", "w") as fh:
        json.dump(e["per_job_grades"], fh, indent=2, default=str)

    baseline_path = Path("context/test-runs/baseline.json")
    baseline_history_path = Path("context/test-runs/baseline-history.json")
    baseline_path.parent.mkdir(parents=True, exist_ok=True)

    diff_summary = {}
    if baseline_path.exists():
        try:
            baseline = json.loads(baseline_path.read_text())
            baseline_run_id = baseline.get("run_id", "unknown")
            for axis_key in ("axis_a", "axis_b", "axis_c", "axis_d", "axis_e", "axis_f", "axis_g"):
                current = per_axis.get(axis_key, 0)
                prior = baseline.get(axis_key, 0)
                delta = round(current - prior, 1)
                direction = "improved" if delta > 0.5 else "regressed" if delta < -0.5 else "stable"
                diff_summary[axis_key] = {"current": current, "prior": prior, "delta": delta, "direction": direction}
            composite_prior = baseline.get("composite", 0)
            diff_summary["composite"] = {
                "current": composite, "prior": composite_prior,
                "delta": round(composite - composite_prior, 1),
                "direction": "improved" if composite - composite_prior > 0.5 else "regressed" if composite - composite_prior < -0.5 else "stable",
            }
            print(f"\nRegression diff vs baseline (run {baseline_run_id}):")
            for k, v in diff_summary.items():
                print(f"  {k}: {v['current']} (was {v['prior']}) Delta {v['delta']:+} [{v['direction']}]")
            history = []
            if baseline_history_path.exists():
                try:
                    history = json.loads(baseline_history_path.read_text())
                except Exception:
                    history = []
            history.append(baseline)
            with open(baseline_history_path, "w") as fh:
                json.dump(history, fh, indent=2)
        except Exception as exc:
            print(f"WARNING: baseline parse failed: {exc}. Treating as first run.")
            diff_summary = {"first_run": True}
    else:
        print("\nNo baseline found. This run becomes the new baseline.")
        diff_summary = {"first_run": True}

    new_baseline = {"run_id": run_id, "composite": composite, **per_axis}
    with open(baseline_path, "w") as fh:
        json.dump(new_baseline, fh, indent=2)
    with open(run_dir / "computed-regression-diff.json", "w") as fh:
        json.dump(diff_summary, fh, indent=2)

    print(f"\n=== Per-axis scores (run {run_id}) ===")
    for k in ("axis_a", "axis_b", "axis_c", "axis_d", "axis_e", "axis_f", "axis_g"):
        print(f"  {k}: {per_axis[k]}")
    print(f"  composite: {composite}")
    print(f"\nIntermediate JSONs written to {run_dir}/computed-*.json")
    print(f"Baseline updated at {baseline_path}")
    print("DONE.")


if __name__ == "__main__":
    main()
