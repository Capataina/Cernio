#!/usr/bin/env python3
"""Find exclude_keyword candidates: tokens hitting C/F job titles with zero B+ hits.

Strict rule: any keyword Caner adds to `preferences.toml [search_filters].exclude_keywords`
MUST hit only C/F titles, never B+ (SS/S/A/B). This script searches the post-regrade
DB for unigrams, bigrams, and trigrams that appear ≥N times in C/F titles AND zero
times in B+ titles, and aren't already filtered.

Usage:
    python3 scripts/find_filter_candidates.py                 # default: ≥5 C/F hits
    python3 scripts/find_filter_candidates.py --min 3         # widen the net
    python3 scripts/find_filter_candidates.py --min 10        # stricter signal
    python3 scripts/find_filter_candidates.py --no-trigrams   # uni + bi only
    python3 scripts/find_filter_candidates.py --examples 5    # show 5 example titles per candidate
    python3 scripts/find_filter_candidates.py --toml          # output TOML-diff form

The default output shows: token, C/F hits, B+ hits (always 0), and up to 3 example
titles per candidate.

Re-run after every grading batch or sector-archival pass — the C/F vs B+ distribution
shifts as the universe re-grades, so new patterns surface periodically.
"""

from __future__ import annotations

import argparse
import re
import sqlite3
from collections import defaultdict
from collections.abc import Iterable
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DB = PROJECT_ROOT / "state" / "cernio.db"
PREFERENCES = PROJECT_ROOT / "profile" / "preferences.toml"


def load_existing_excludes() -> set[str]:
    excludes: set[str] = set()
    in_list = False
    with open(PREFERENCES, "r", encoding="utf-8") as f:
        for line in f:
            stripped = line.strip()
            if "exclude_keywords" in stripped and "=" in stripped:
                in_list = True
                continue
            if in_list:
                if stripped.startswith("]"):
                    in_list = False
                    continue
                for m in re.finditer(r'"([^"]+)"', stripped):
                    excludes.add(m.group(1).lower())
    return excludes


def tokenize(title: str, max_ngram: int) -> Iterable[str]:
    """Yield unigrams, bigrams, ..., max_n-grams (whitespace tokens, lowercased,
    punctuation-stripped). Yields each token at most once per title — duplicates
    within a title don't inflate the count."""
    cleaned = re.sub(r"[^a-zA-Z0-9 ]+", " ", title or "").lower()
    tokens = [t for t in cleaned.split() if t]
    seen: set[str] = set()
    for n in range(1, max_ngram + 1):
        for i in range(len(tokens) - n + 1):
            ng = " ".join(tokens[i : i + n])
            if ng not in seen:
                seen.add(ng)
                yield ng


# Tokens too generic to ever serve as exclude_keywords.
STOPWORDS = {
    "a", "an", "the", "of", "in", "on", "at", "for", "to", "and", "or",
    "with", "by", "from", "as", "is", "it", "be", "are", "all", "our",
    "we", "us", "their", "your", "you", "i", "this", "that", "these",
    "those", "my", "any", "but", "not", "no", "do", "does", "did",
    "co", "ltd", "inc", "llc", "uk", "us",
    # Engineering universals — must not be excluded.
    "engineer", "engineering", "software", "developer", "developing",
    "development", "programmer",
}


def is_meaningful(tok: str, excludes: set[str]) -> bool:
    if tok in STOPWORDS:
        return False
    if len(tok) < 3:
        return False
    if tok.isdigit():
        return False
    if tok in excludes:
        return False
    # Bigrams/trigrams composed entirely of already-excluded words are redundant.
    if " " in tok:
        words = tok.split()
        if all(w in excludes for w in words):
            return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--min", type=int, default=5, dest="min_cf",
                        help="minimum C/F hit count (default 5)")
    parser.add_argument("--top", type=int, default=80,
                        help="top N candidates to show (default 80)")
    parser.add_argument("--examples", type=int, default=3,
                        help="example titles per candidate (default 3)")
    parser.add_argument("--ngram-max", type=int, default=3,
                        help="max n-gram length (default 3 = unigrams + bigrams + trigrams)")
    parser.add_argument("--no-trigrams", action="store_true",
                        help="shorthand for --ngram-max 2")
    parser.add_argument("--toml", action="store_true",
                        help="emit TOML-diff form ready to paste into preferences.toml")
    args = parser.parse_args()

    if args.no_trigrams:
        args.ngram_max = 2

    excludes = load_existing_excludes()

    conn = sqlite3.connect(DB)
    cur = conn.cursor()
    cur.execute(
        """
        SELECT title, grade
        FROM jobs
        WHERE evaluation_status != 'archived'
          AND grade IS NOT NULL
          AND title IS NOT NULL
        """
    )
    rows = cur.fetchall()

    cf_titles: list[str] = []
    bplus_titles: list[str] = []
    for title, grade in rows:
        if grade in ("C", "F"):
            cf_titles.append(title)
        elif grade in ("SS", "S", "A", "B"):
            bplus_titles.append(title)

    # Tally each token in each pool, also tracking example titles for C/F hits.
    cf_counts: dict[str, int] = defaultdict(int)
    bp_counts: dict[str, int] = defaultdict(int)
    cf_examples: dict[str, list[str]] = defaultdict(list)
    for title in cf_titles:
        for tok in tokenize(title, args.ngram_max):
            cf_counts[tok] += 1
            if len(cf_examples[tok]) < args.examples:
                cf_examples[tok].append(title)
    for title in bplus_titles:
        for tok in tokenize(title, args.ngram_max):
            bp_counts[tok] += 1

    # Candidates: ≥min C/F hits AND zero B+ hits AND meaningful.
    candidates: list[tuple[str, int]] = []
    for tok, cf_count in cf_counts.items():
        if cf_count < args.min_cf:
            continue
        if bp_counts.get(tok, 0) > 0:
            continue
        if not is_meaningful(tok, excludes):
            continue
        candidates.append((tok, cf_count))
    candidates.sort(key=lambda r: (-r[1], r[0]))

    # Render.
    print(f"Universe (post-regrade):")
    print(f"  C/F titles : {len(cf_titles)}")
    print(f"  B+ titles  : {len(bplus_titles)}")
    print(f"  Currently excluded: {len(excludes)} keywords")
    print()
    print(f"Candidates ({len(candidates)}) — ≥{args.min_cf} C/F hits AND zero B+ hits AND not already excluded:")
    print()

    if args.toml:
        print("# Paste into preferences.toml [search_filters].exclude_keywords:")
        print()
        for tok, cf in candidates[: args.top]:
            print(f'    "{tok.title()}",  # {cf} C/F hits, 0 B+ hits')
        return 0

    header = f"{'#':>3}  {'Token':<35} {'C/F':>4}  {'Examples'}"
    print(header)
    print("-" * len(header))
    for i, (tok, cf) in enumerate(candidates[: args.top], 1):
        ex = " | ".join(cf_examples[tok][: args.examples])
        if len(ex) > 95:
            ex = ex[:92] + "..."
        print(f"{i:>3}  {tok:<35} {cf:>4}  {ex}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
