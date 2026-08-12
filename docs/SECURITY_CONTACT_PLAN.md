# Security Contact Plan

**Purpose:** `QUALITY_UPGRADE_ROADMAP.md` Phase 5 flags that an unreachable security contact
on a contract handling staked value is a real gap, not cosmetic. This document records what
currently exists, what's genuinely missing, and what stands up a more complete contact path
— it does not itself provision an inbox, a domain, or a monitoring process, since none of
that is something a repository change can create.

_Compiled: 2026-08-12._

## What already works today

[`SECURITY.md`](../SECURITY.md) has always pointed reporters to [GitHub Security
Advisories](https://github.com/Softvaults/trustforge-contracts/security/advisories/new) —
this is a real, functioning GitHub feature, not a placeholder. Anyone can open a private
advisory against this repository right now and it will reach whoever has admin/security
access on the `Softvaults/trustforge-contracts` GitHub org, no additional setup required.

[`SECURITY_AUDIT.md`](../SECURITY_AUDIT.md) previously duplicated this with a second,
non-functional contact — `security@trustforge.io (to be updated with actual contact)`. That
placeholder has been removed (2026-08-12) in favor of consistently pointing to the real GH
Advisories channel both documents already had access to. That fixes the "an unreachable
contact is listed" problem; it does not fix the separate question below.

## What's still genuinely open

GitHub Security Advisories being *reachable* is not the same as being *monitored with a
committed response time*. This document cannot verify or guarantee:

- **Who** on the team has notifications enabled for new security advisories on this repo.
- **How quickly** a report gets triaged — there is no published SLA anywhere in the repo
  today (`SECURITY.md` says "we will acknowledge new reports" with no timeframe).
- **Escalation path** if the primary reviewer is unavailable.

These are operational/staffing decisions, not code changes, and are outside what this
repository's automation can resolve on its own.

## If a dedicated channel is wanted beyond GitHub Advisories

Some reporters and auditors expect an email + PGP key as a matter of convention, independent
of whether GitHub Advisories is technically sufficient. If the team decides to add one:

1. **Domain and inbox**: requires an actual `trustforge.io` (or whatever domain the project
   settles on) mailbox that someone actively monitors — not a repository change.
2. **PGP key**: generate a dedicated key for the security inbox, publish its fingerprint in
   `SECURITY.md` and (optionally) a `security.txt` at `/.well-known/security.txt` per
   [RFC 9116](https://www.rfc-editor.org/rfc/rfc9116) — a repo-side change once the key
   exists, but the key itself must be generated and custodied by a real person/team.
3. **Response-time SLA**: pick and publish a number (e.g. "acknowledgment within 3 business
   days") in `SECURITY.md` — cheap to do, currently missing, doesn't require new
   infrastructure.
4. **Link it from `SECURITY_AUDIT.md`, `README.md`, and the bug bounty program** (see below)
   once it exists, so all three stay consistent rather than drifting the way the old
   placeholder email did.

## Relationship to the bug bounty program

`SECURITY_AUDIT.md`'s "Bug Bounty Program" section (`Status: To be launched post-deployment`)
was intentionally left untouched by this pass — standing up a real bounty program requires
committing real reward funds and choosing a platform (e.g. Immunefi, HackerOne, or a
self-run process via GitHub Advisories), which is a funding and business decision, not
something to draft speculatively without the team's input on budget and risk appetite. When
that decision is made, its disclosure channel should be the same one documented here rather
than a third, separate contact.

## Summary

| Item | Status |
|---|---|
| A real, reachable disclosure channel | ✅ Exists today — GitHub Security Advisories, documented in `SECURITY.md` |
| Placeholder/dead contact removed from `SECURITY_AUDIT.md` | ✅ Fixed 2026-08-12 |
| Published response-time SLA | ❌ Missing — cheap to add, needs a team decision on the number |
| Dedicated security email + PGP key | ❌ Missing — needs real infrastructure and a monitoring commitment |
| Bug bounty program | ❌ Not started — needs funding decision, out of scope here |
