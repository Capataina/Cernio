#!/usr/bin/env python3
"""Split all active+graded companies and jobs into N stripes for parallel re-grade.

Each batch gets a similar tier mix (companies striped by sort-position so batch 0
gets the 1st, 11th, 21st... company across S/A/B/C sorted order; same for jobs).
Output goes to /tmp/regrade_{company,job}_batch_NN.md as per-stripe brief files
ready to hand to parallel re-grade agents.

Usage:
    python3 scripts/split_regrade_batches.py            # default: 10 stripes
    python3 scripts/split_regrade_batches.py --n 5      # fewer / larger stripes
    python3 scripts/split_regrade_batches.py --out /path/to/dir
"""

from __future__ import annotations

import argparse
import sqlite3
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DB = PROJECT_ROOT / "state" / "cernio.db"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--n", type=int, default=10, help="number of stripes (default 10)")
    parser.add_argument("--out", type=Path, default=Path("/tmp"), help="output directory (default /tmp)")
    parser.add_argument("--skip-companies", action="store_true", help="only split jobs")
    parser.add_argument("--skip-jobs", action="store_true", help="only split companies")
    args = parser.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(DB)
    cur = conn.cursor()

    if not args.skip_companies:
        cur.execute(
            """
            SELECT id, name, COALESCE(website,'') AS website, COALESCE(careers_url,'') AS careers_url,
                   COALESCE(grade,'') AS grade, COALESCE(grade_reasoning,'') AS grade_reasoning,
                   COALESCE(why_relevant,'') AS why_relevant, COALESCE(what_they_do,'') AS what_they_do,
                   COALESCE(location,'') AS location, COALESCE(sector_tags,'') AS sector_tags,
                   status
            FROM companies
            WHERE status IN ('resolved', 'bespoke')
              AND grade IS NOT NULL
            ORDER BY CASE grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 WHEN 'C' THEN 4 END, id ASC
            """
        )
        companies = cur.fetchall()
        print(f"Total active+graded companies to re-grade: {len(companies)}")
        batches = [[] for _ in range(args.n)]
        for i, c in enumerate(companies):
            batches[i % args.n].append(c)
        for bi, batch in enumerate(batches):
            path = args.out / f"regrade_company_batch_{bi:02d}.md"
            with open(path, "w", encoding="utf-8") as out:
                out.write(f"# Company Re-Grade Batch {bi:02d} — {len(batch)} companies\n\n")
                for co in batch:
                    cid, name, web, careers, grade, reason, why, what, loc, tags, status = co
                    out.write(f"---\n\n## Company id={cid}\n")
                    out.write(f"- **Name**: {name}\n")
                    out.write(f"- **Current grade**: {grade}\n")
                    out.write(f"- **Status**: {status}\n")
                    out.write(f"- **Website**: {web}\n")
                    out.write(f"- **Careers URL**: {careers}\n")
                    out.write(f"- **Location**: {loc}\n")
                    out.write(f"- **Sector tags**: {tags}\n\n")
                    out.write(f"### What they do (current)\n\n{what}\n\n")
                    out.write(f"### Current grade_reasoning (likely stale)\n\n{reason}\n\n")
                    out.write(f"### Current why_relevant (likely stale)\n\n{why}\n\n")
            print(f"  {path}: {len(batch)} companies")

    if not args.skip_jobs:
        cur.execute(
            """
            SELECT j.id, c.name AS company_name, c.grade AS company_grade,
                   j.title, COALESCE(j.location,'') AS loc, COALESCE(j.url,'') AS url,
                   COALESCE(j.grade,'') AS current_grade, COALESCE(j.fit_assessment,'') AS current_fit,
                   COALESCE(SUBSTR(j.raw_description, 1, 3500), '') AS desc
            FROM jobs j
            JOIN companies c ON c.id = j.company_id
            WHERE j.evaluation_status != 'archived'
              AND j.grade IS NOT NULL
            ORDER BY
                CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END,
                CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3 WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6 END,
                j.id ASC
            """
        )
        jobs = cur.fetchall()
        print(f"\nTotal active+graded jobs to re-grade: {len(jobs)}")
        job_batches = [[] for _ in range(args.n)]
        for i, j in enumerate(jobs):
            job_batches[i % args.n].append(j)
        for bi, batch in enumerate(job_batches):
            path = args.out / f"regrade_job_batch_{bi:02d}.md"
            with open(path, "w", encoding="utf-8") as out:
                out.write(f"# Job Re-Grade Batch {bi:02d} — {len(batch)} jobs\n\n")
                for job in batch:
                    jid, cname, cgrade, title, loc, url, current_grade, current_fit, desc = job
                    d = desc.strip()
                    if len(d) > 3500:
                        d = d[:3500] + "\n[...truncated]"
                    out.write(f"---\n\n## Job id={jid}\n")
                    out.write(f"- **Company**: {cname} (company_grade={cgrade or 'NULL'})\n")
                    out.write(f"- **Title**: {title}\n")
                    out.write(f"- **Location**: {loc}\n")
                    out.write(f"- **URL**: {url}\n")
                    out.write(f"- **Current job grade**: {current_grade}\n\n")
                    out.write(f"### Current fit_assessment (likely stale)\n\n{current_fit}\n\n")
                    if d and len(d) >= 100:
                        out.write(f"### Description\n\n{d}\n\n")
                    else:
                        out.write(f"### Description\n\n[MISSING — WebFetch the URL above if needed; otherwise grade from title+company+current_fit signals if confident]\n\n")
            print(f"  {path}: {len(batch)} jobs")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
