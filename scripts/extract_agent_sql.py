#!/usr/bin/env python3
"""Extract UPDATE statements from parallel-agent JSONL transcripts.

When N agents run a re-grade / grading task in parallel, each emits a JSONL
transcript at /private/tmp/claude-XXX/.../tasks/{agent_id}.output. This script
walks all transcripts, extracts UPDATE statements with proper '' string-escape
handling, dedupes by (table, id) with last-write-wins (handles agent self-
corrections where the corrected statement is re-emitted after a broken one),
sanity-checks against the live DB, and emits a single BEGIN/COMMIT-wrapped
SQL file ready to apply with `sqlite3 state/cernio.db ".read /tmp/out.sql"`.

Companies and jobs are both handled in a single pass — the regex anchors on
the column order each agent prompt enforces (`grade = 'X', grade_reasoning =`
for companies; `grade = 'X', evaluation_status =` for jobs).

Usage:
    python3 scripts/extract_agent_sql.py
        # auto-detects the current Claude Code task dir; writes /tmp/all_agent_sql.sql
    python3 scripts/extract_agent_sql.py --task-dir /private/tmp/claude-501/.../tasks
    python3 scripts/extract_agent_sql.py --out /tmp/myrun.sql
    python3 scripts/extract_agent_sql.py --no-jobs
        # only extract company UPDATEs (and vice versa with --no-companies)

Manual patches: any agent self-flagged "the SQL says X but it should be Y"
case can be added via --patch-job ID:NEWGRADE:NEWSCORE (repeatable). The
hard-coded 2972 patch from the 2026-05-14 re-grade is retained as a literal
because the patch reflects a permanent grade decision, not a runtime tweak.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DB = PROJECT_ROOT / "state" / "cernio.db"

# SQL string fragment with proper '' escape support.
SQL_STRING = r"'(?:[^']|'')*'"

# Match a full UPDATE companies / UPDATE jobs statement, anchored on the
# column order the agent prompt enforces so we don't match prose snippets
# that contain `UPDATE jobs SET grade = 'B'` inline.
COMPANY_STMT_RE = re.compile(
    rf"UPDATE\s+companies\s+SET\s+grade\s*=\s*'[SABC]'\s*,\s*grade_reasoning\s*=\s*(?:[^';]|{SQL_STRING})+?\s+WHERE\s+id\s*=\s*(\d+);",
    re.DOTALL | re.IGNORECASE,
)
JOB_STMT_RE = re.compile(
    rf"UPDATE\s+jobs\s+SET\s+grade\s*=\s*'(?:SS|S|A|B|C|F)'\s*,\s*evaluation_status\s*=\s*(?:[^';]|{SQL_STRING})+?\s+WHERE\s+id\s*=\s*(\d+);",
    re.DOTALL | re.IGNORECASE,
)


def auto_detect_task_dir() -> Path | None:
    """Find the most-recently-modified Claude Code task directory under
    /private/tmp/claude-*/<project>/<session-uuid>/tasks/."""
    candidates: list[Path] = []
    for base in Path("/private/tmp").glob("claude-*"):
        for project in base.iterdir():
            if not project.is_dir():
                continue
            for session in project.iterdir():
                tasks = session / "tasks"
                if tasks.is_dir():
                    candidates.append(tasks)
    if not candidates:
        return None
    candidates.sort(key=lambda p: p.stat().st_mtime, reverse=True)
    return candidates[0]


def extract_text_from_jsonl(path: Path) -> str:
    """Pull assistant text + Write tool input.content + Edit new_string + Bash
    command from the transcript. Agents emit their SQL in any of these places
    depending on whether they inline it in chat or write it to a /tmp file."""
    parts: list[str] = []
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                try:
                    msg = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if not isinstance(msg, dict):
                    continue
                content = None
                if msg.get("type") == "assistant":
                    m = msg.get("message")
                    if isinstance(m, dict):
                        content = m.get("content")
                elif msg.get("role") == "assistant":
                    content = msg.get("content")
                if isinstance(content, list):
                    for item in content:
                        if isinstance(item, dict):
                            t = item.get("type")
                            if t == "text":
                                parts.append(item.get("text", ""))
                            elif t == "tool_use":
                                inp = item.get("input", {})
                                if isinstance(inp, dict):
                                    for key in ("content", "new_string", "command"):
                                        v = inp.get(key)
                                        if isinstance(v, str):
                                            parts.append(v)
                        elif isinstance(item, str):
                            parts.append(item)
                elif isinstance(content, str):
                    parts.append(content)
    except OSError as e:
        print(f"  warn: could not read {path}: {e}", file=sys.stderr)
    return "\n".join(parts)


def validate_company_stmt(stmt: str) -> bool:
    return (
        re.search(r"\bgrade\s*=\s*'[SABC]'", stmt) is not None
        and "grade_reasoning" in stmt
        and "why_relevant" in stmt
        and "graded_at" in stmt
    )


def validate_job_stmt(stmt: str) -> bool:
    if not (
        re.search(r"\bgrade\s*=\s*'(SS|S|A|B|C|F)'", stmt) is not None
        and "evaluation_status" in stmt
        and "fit_assessment" in stmt
        and "fit_score" in stmt
    ):
        return False
    # Reject placeholder fit_assessments (agents sometimes emit prose-level
    # corrections like "[same text as above, but replace X with Y]" that would
    # overwrite real data with junk).
    if re.search(r"fit_assessment\s*=\s*'\[same text as above", stmt, re.IGNORECASE):
        return False
    if re.search(r"fit_assessment\s*=\s*'\[(same|original|previous|prior)\b", stmt, re.IGNORECASE):
        return False
    return True


def db_sanity_check(table: str, where_clause: str, got_ids: set[int], label: str) -> None:
    res = subprocess.run(
        ["sqlite3", str(DB), f"SELECT id FROM {table} WHERE {where_clause} ORDER BY id;"],
        capture_output=True,
        text=True,
    )
    expected = set(int(x) for x in res.stdout.strip().split("\n") if x.strip())
    missing = expected - got_ids
    extra = got_ids - expected
    print(f"\n{label} expected: {len(expected)}, got: {len(got_ids)}")
    print(f"  missing: {len(missing)} — {sorted(missing)[:30]}{'...' if len(missing) > 30 else ''}")
    print(f"  extra:   {len(extra)} — {sorted(extra)[:30]}{'...' if len(extra) > 30 else ''}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--task-dir", type=Path, help="Claude Code task dir (auto-detected if omitted)")
    parser.add_argument("--out", type=Path, default=Path("/tmp/all_agent_sql.sql"), help="output SQL file")
    parser.add_argument("--no-companies", action="store_true", help="skip company UPDATE extraction")
    parser.add_argument("--no-jobs", action="store_true", help="skip job UPDATE extraction")
    parser.add_argument("--patch-job", action="append", default=[],
                        help="manual job grade override: ID:GRADE:SCORE (repeatable)")
    args = parser.parse_args()

    task_dir = args.task_dir or auto_detect_task_dir()
    if task_dir is None or not task_dir.is_dir():
        print(f"ERROR: could not locate task dir. Pass --task-dir explicitly.", file=sys.stderr)
        return 1
    print(f"Task dir: {task_dir}")

    company_updates: dict[int, str] = {}
    company_order: list[int] = []
    job_updates: dict[int, str] = {}
    job_order: list[int] = []

    files = sorted(task_dir.glob("*.output"))
    print(f"Scanning {len(files)} agent transcripts...")

    cfu = jfu = 0
    skip_c = skip_j = 0
    for path in files:
        text = extract_text_from_jsonl(path)
        if not text:
            continue
        seen_c = seen_j = False
        if not args.no_companies:
            for m in COMPANY_STMT_RE.finditer(text):
                stmt = m.group(0).strip()
                cid = int(m.group(1))
                if not validate_company_stmt(stmt):
                    skip_c += 1
                    continue
                if cid in company_updates:
                    company_order.remove(cid)
                company_updates[cid] = stmt
                company_order.append(cid)
                seen_c = True
        if not args.no_jobs:
            for m in JOB_STMT_RE.finditer(text):
                stmt = m.group(0).strip()
                jid = int(m.group(1))
                if not validate_job_stmt(stmt):
                    skip_j += 1
                    continue
                if jid in job_updates:
                    job_order.remove(jid)
                job_updates[jid] = stmt
                job_order.append(jid)
                seen_j = True
        cfu += int(seen_c)
        jfu += int(seen_j)

    print(f"  {cfu} files contributed company UPDATEs ({skip_c} stmts skipped as invalid)")
    print(f"  {jfu} files contributed job UPDATEs ({skip_j} stmts skipped as invalid)")
    print(f"Unique company UPDATEs: {len(company_updates)}")
    print(f"Unique job UPDATEs: {len(job_updates)}")

    # Apply --patch-job overrides.
    for spec in args.patch_job:
        try:
            jid_s, new_grade, new_score = spec.split(":")
            jid = int(jid_s)
            if jid not in job_updates:
                print(f"  warn: --patch-job {spec} skipped, id={jid} not in extracted set")
                continue
            before = job_updates[jid]
            patched = re.sub(r"grade\s*=\s*'[A-Z]+'", f"grade = '{new_grade.upper()}'", before, count=1)
            patched = re.sub(r"fit_score\s*=\s*[\d.]+", f"fit_score = {new_score}", patched, count=1)
            job_updates[jid] = patched
            print(f"  patched job id={jid}: grade→{new_grade}, fit_score→{new_score}")
        except ValueError:
            print(f"  warn: --patch-job {spec} malformed (expected ID:GRADE:SCORE)")

    # DB sanity-checks.
    if company_updates:
        db_sanity_check(
            "companies",
            "status IN ('resolved','bespoke') AND grade IS NOT NULL",
            set(company_updates),
            "Companies",
        )
    if job_updates:
        db_sanity_check(
            "jobs",
            "evaluation_status != 'archived' AND grade IS NOT NULL",
            set(job_updates),
            "Jobs",
        )

    # Emit aggregated SQL.
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as out:
        out.write(f"-- Agent SQL aggregated from {task_dir}\n")
        out.write(f"-- {len(company_updates)} company UPDATEs + {len(job_updates)} job UPDATEs\n")
        if args.patch_job:
            out.write(f"-- Manual patches: {' '.join(args.patch_job)}\n")
        out.write("\nBEGIN;\n\n")
        if company_updates:
            out.write("-- COMPANY UPDATES\n")
            for cid in company_order:
                out.write(company_updates[cid] + "\n")
        if job_updates:
            out.write("\n-- JOB UPDATES\n")
            for jid in job_order:
                out.write(job_updates[jid] + "\n")
        out.write("\nCOMMIT;\n")
    print(f"\nWrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
