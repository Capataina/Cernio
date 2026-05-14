---
title: "LeetCode — Practice Profile"
tags:
  - profile
  - professional
  - leetcode
  - interview-prep
domain: profile/professional
status: active
created: 2026-04-29
last_verified: 2026-04-29
evidence_basis: "LeetCode GraphQL API (matchedUser + recentAcSubmissionList + languageStats + userCalendar) fetched 2026-04-29 01:50 BST"
---

# LeetCode

Caner's LeetCode practice — solved problems, language mix, topic coverage, calibration data. Public profile: <https://leetcode.com/Capataina/>.

## Headline numbers (as of 2026-04-29)

| Metric | Value |
|---|---|
| Total problems solved | **34** (out of 38 attempted; 80 total submissions) |
| Easy | 29 / 32 attempted |
| Medium | 5 / 6 attempted |
| Hard | 0 / 0 |
| Global ranking | 3,093,038 |
| Account name | `Capataina` (real name: Ata Caner Cetinkaya) |
| Active years | 2022, 2024, 2025 |
| Total active days (2025) | 4 |
| Current streak | 1 |

The 34 solved / 80 submission ratio means ~2.4 attempts per solve on average, with most failed attempts concentrated on Medium-tier problems (16 failed Medium submissions out of 22 = ~73% retry rate on Mediums vs ~76% first-pass on Easy 33/58). Lifetime active days = 4 means practice is intense-but-rare: a few focused sessions rather than daily grind.

## Languages used

| Language | Problems solved | Notes |
|---|---|---|
| Pandas | 15 | SQL/data-manipulation track (LeetCode's Pandas Study Plan) |
| Python3 | 10 | General algorithmic problems |
| Rust | 6 | Recent shift — most July 2025 submissions in Rust |
| JavaScript | 2 | Older work |
| C# | 1 | Single problem |

The Pandas concentration is unusual for a general LeetCode profile and reflects deliberate data-engineering practice. The Rust shift in mid-2025 aligns with the broader Rust-as-primary-language pattern across Caner's projects (Cernio, NeuroDrive, Nyquestro, Vynapse, Xyntra, Zyphos).

## Topic coverage

### Fundamental (per LeetCode's tag taxonomy)

| Tag | Solved |
|---|---|
| Array | 7 |
| String | 4 |
| Sorting | 2 |
| Linked List | 2 |
| Two Pointers | 2 |
| Simulation | 2 |

### Intermediate

| Tag | Solved |
|---|---|
| Math | 4 |
| Hash Table | 3 |
| Greedy | 2 |
| Binary Search | 2 |
| Recursion | 2 |
| Bit Manipulation | 2 |

### Advanced

| Tag | Solved |
|---|---|
| Dynamic Programming | 2 |
| Divide and Conquer | 1 |

The distribution reads as "solid fundamentals, beginning intermediate, just touched advanced." DP at 2 solved is the natural next-growth area for interview prep at SS/S/A-tier targets — most senior interview loops include 1-2 DP problems.

## Recent submissions (last 7 visible via API)

LeetCode's public GraphQL endpoint exposes only the most recent 15 accepted submissions to non-authenticated callers. As of 2026-04-29 the API returns 7 entries; the remaining 27 of 34 solved problems are not enumerable via this endpoint without authenticated cookies.

| Date | Language | Problem | Slug |
|---|---|---|---|
| 2025-07-23 | Python3 | Reverse Linked List | `reverse-linked-list` |
| 2025-07-21 | Rust | Contains Duplicate | `contains-duplicate` |
| 2025-07-19 | Rust | Jump Game | `jump-game` |
| 2025-07-19 | Rust | Pascal's Triangle | `pascals-triangle` |
| 2025-07-13 | Rust | Merge Nodes in Between Zeros | `merge-nodes-in-between-zeros` |
| 2025-07-13 | Rust | Find the Maximum Achievable Number | `find-the-maximum-achievable-number` |
| 2025-07-13 | Rust | Score of a String | `score-of-a-string` |

The pattern across these 7: a focused Rust-practice burst over 11 days (2025-07-13 → 2025-07-23) covering linked lists, dynamic programming entry-points (Jump Game, Pascal's Triangle), hash-set fundamentals (Contains Duplicate), and a few warmup-tier problems (Score of a String, Find the Maximum Achievable Number).

## Gap: full solved-problem list (27 problems not API-accessible)

The remaining 27 of 34 solved problems are visible on the authenticated LeetCode UI under "Submissions" but not via the public GraphQL endpoint. Three paths to extract them:

1. **Browser-cookie scrape** — fetch with an authenticated `LEETCODE_SESSION` cookie via `curl --cookie`. Caner can paste the cookie value into a one-off command; the script can iterate the paginated submissions API and emit the full list.
2. **Manual paste** — copy the Submissions table from LeetCode's UI into this file. Lossy on metadata but quickest if cookie-fetching feels heavy.
3. **Defer + grow forward** — leave the historical 27 unenumerable, log every new submission via the public API (which always shows the most recent 15) on a periodic script. Future-only, not retroactive.

No path picked yet — surfaced for Caner to choose.

## Cross-references

- `Profile/Professional/Personal.md` § LeetCode — single-line URL pointer; redundant once this file is canonical (consider removing on next Profile pass)
- `Projects/Cernio/Roadmap.md` — references "LeetCode-style TDD problems tuned to target companies" as a Cernio capability (the [[Projects/Cernio]] interview-prep curriculum design)
- `Projects/Cernio/Gaps.md` — interview-prep curriculum design exists in `Context/notes/interview-prep-design.md` per Cernio's gaps
- LeetCode public profile: <https://leetcode.com/Capataina/>

## Source data — extraction trail

| Source | What it gave | When |
|---|---|---|
| `https://leetcode.com/graphql/` `matchedUser` query | profile + total submission counts by difficulty | 2026-04-29 01:48 BST |
| `https://leetcode.com/graphql/` `recentAcSubmissionList` query (limit=50) | 7 most recent accepted submissions | 2026-04-29 01:48 BST |
| `https://leetcode.com/graphql/` `languageStats` query | language + topic-tag breakdown | 2026-04-29 01:48 BST |
| `https://leetcode.com/graphql/` `userProfileCalendar` query (year=2025) | active years + total active days + streak | 2026-04-29 01:50 BST |
| Public profile HTML at `https://leetcode.com/Capataina/` | blocked with 403 to `WebFetch` (anti-scraping) | 2026-04-29 01:48 BST |
| `gh repo list Capataina` | scanned 60+ repos for `*leetcode*` / `algo*` / `dsa*` — no match | 2026-04-29 01:50 BST |
| Local `~` + `~/Documents` filesystem search for `*leetcode*` directories | no results | 2026-04-29 01:50 BST |

Source data was extracted 2026-04-29 from quick-notes inbox item *"extract my leetcode things here"*.
