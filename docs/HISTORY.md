# Repository History

**Purpose:** `QUALITY_UPGRADE_ROADMAP.md` Phase 6 flagged this repository's contribution
pattern — 197 contributors over roughly six months — as worth being transparent about, rather
than leaving newcomers to guess why the author list looks the way it does. This document
records what's observable from `git log` as of 2026-08-12. It intentionally does not
speculate about *why* the project was built this way (a bounty program, a hackathon, an
outsourced effort, or something else) — that context belongs to the people who ran it, not to
an inference from commit metadata. If someone with that context wants to add it, this is the
right place.

## Observable facts (as of 2026-08-12)

- **982 commits** across the repository's full history.
- **197 distinct contributor identities** (by `git shortlog` author name/email — see the
  caveat below on what "distinct" means here).
- **Time span:** first commit 2026-02-18, most recent 2026-08-12 — about six months.
- **Commit volume by month:**

  | Month | Commits |
  |---|---:|
  | 2026-02 | 294 |
  | 2026-03 | 97 |
  | 2026-04 | 201 |
  | 2026-05 | 82 |
  | 2026-06 | 249 |
  | 2026-07 | 10 |
  | 2026-08 | 49 (partial month) |

  Activity is uneven rather than steady — three months (Feb, Apr, Jun) account for the
  large majority of commits, with July nearly silent.
- **Contribution concentration:** the top contributor (`Baskar`) accounts for 218 commits
  (~22% of all commits); the top 5 identities together account for roughly 37%. 81 of the 197
  identities have exactly one commit; 75 have between 2 and 5.
- **Doc-only commits:** roughly 427 of 982 commits (~43%) touched only `.md` files, by a
  simple "every changed path ends in `.md`" check. This is an approximation — it doesn't
  distinguish a genuine typo fix from a doc commit that happened to bundle with nothing else,
  and doesn't catch doc changes bundled into a larger code commit.

## A caveat on "197 contributors"

`git shortlog` groups by the commit author's recorded name and email, which is
self-reported and not verified identity. Two rows in the shortlog output (`Baskar`,
218 commits, and `Baskarayelu`, 59 commits) could plausibly be the same person under two
git configurations, or two different people — this document doesn't resolve that, and
neither does raw `git log`. Treat "197" as "197 distinct author identities recorded in
history," not as a verified headcount of 197 different humans.

## What this document is not

It is not an explanation of motive, process, or quality control during this period — see
[`QUALITY_UPGRADE_ROADMAP.md`](../QUALITY_UPGRADE_ROADMAP.md) for the broader effort to bring
the codebase's engineering and security posture up to the standard its documentation claims,
and [`CONTRIBUTING.md`](../CONTRIBUTING.md)'s Review Tiers section and
[`.github/CODEOWNERS`](../.github/CODEOWNERS) for how contribution review works going forward.
