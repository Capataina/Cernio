#!/usr/bin/env python3
"""Split all pending/evaluating jobs into N stripes for parallel grading.

Within each company-grade tier, jobs are pre-sorted by a title-keyword
"high-promise" score so the highest-signal jobs land in the first slot of each
batch (each batch gets a similar tier distribution but the top of each batch is
front-loaded with grad / intern / rust / compiler / trading keywords).

Usage:
    python3 scripts/split_grading_batches.py            # default: 8 stripes
    python3 scripts/split_grading_batches.py --n 5      # fewer / larger
    python3 scripts/split_grading_batches.py --out /path/to/dir
"""

from __future__ import annotations

import argparse
import sqlite3
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DB = PROJECT_ROOT / "state" / "cernio.db"

HIGH_KEYWORDS = {
    "graduate", "grad", "new grad", "junior", "entry", "intern",
    "rust", "compiler", "runtime", "low-latency", "systems", "infrastructure",
    "platform", "trading", "quant", "quantitative", "hft",
}


def title_score(title: str) -> int:
    t = title.lower()
    return sum(1 for kw in HIGH_KEYWORDS if kw in t)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--n", type=int, default=8, help="number of stripes (default 8)")
    parser.add_argument("--out", type=Path, default=Path("/tmp"), help="output directory (default /tmp)")
    args = parser.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(DB)
    cur = conn.cursor()
    cur.execute(
        """
        SELECT j.id, c.name AS company_name, c.grade AS company_grade,
               j.title, COALESCE(j.location, '') AS loc,
               COALESCE(j.url, '') AS url,
               COALESCE(j.raw_description, '') AS desc
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE j.evaluation_status = 'pending' OR j.evaluation_status = 'evaluating'
        ORDER BY
            CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END,
            j.id ASC
        """
    )
    jobs = cur.fetchall()
    print(f"Total pending jobs: {len(jobs)}")

    def sort_key(j):
        cg = j[2] or "D"
        tier = {"S": 1, "A": 2, "B": 3, "C": 4, "D": 5}.get(cg, 5)
        return (tier, -title_score(j[3]), j[0])
    jobs.sort(key=sort_key)

    batches: list[list] = [[] for _ in range(args.n)]
    for i, j in enumerate(jobs):
        batches[i % args.n].append(j)

    for bi, batch in enumerate(batches):
        path = args.out / f"grading_batch_{bi:02d}.md"
        with open(path, "w", encoding="utf-8") as out:
            out.write(f"# Grading Batch {bi:02d} — {len(batch)} jobs\n\n")
            for job in batch:
                jid, cname, cgrade, title, loc, url, desc = job
                d = desc.strip()
                if len(d) > 4000:
                    d = d[:4000] + "\n[...truncated for batch file; agent should WebFetch url for full description if needed]"
                out.write(f"---\n\n## Job id={jid}\n")
                out.write(f"- **Company**: {cname} (company_grade={cgrade or 'NULL'})\n")
                out.write(f"- **Title**: {title}\n")
                out.write(f"- **Location**: {loc}\n")
                out.write(f"- **URL**: {url}\n")
                if not d or len(d) < 100:
                    out.write("- **Description**: [MISSING — fetch via WebFetch on URL above]\n\n")
                else:
                    out.write(f"\n### Description\n\n{d}\n\n")
        print(f"  {path}: {len(batch)} jobs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
