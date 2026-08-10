# Finding: Orphaned Modules in `trustforge_bond` (discovered 2026-08-10)

## Summary

`contracts/trustforge_bond/src/lib.rs` does not declare `mod` for roughly 60 of the
`.rs` files physically present in `contracts/trustforge_bond/src/`. Undeclared files are
never compiled — not into the release WASM, not into `cargo test`, not into anything.
This was discovered while investigating why `cargo clippy` reported ~20-30 functions as
dead code: their only real callers turned out to live in test files that are themselves
never compiled.

This is **not** a `known-simplifications.md`-style intentional limitation. Git evidence
(below) shows these files were live, wired-in, and under active bugfix at some point,
and were dropped from the module tree — most likely during
[`a8cdf7b1`](../../commit/a8cdf7b1) ("fix: resolve pre-existing merge corruption
blocking the workspace build") or the rebrand commits (`39cea808`, `fe4449c6`) that
immediately followed it in the same session.

**No restoration has been done.** This document exists so the finding isn't lost; see
[Recommendation](#recommendation) for why a mechanical restore is the wrong next step.

## Orphaned production modules (8 files, ~1,955 lines)

None of these are declared as `mod` anywhere in `lib.rs`. Each is a complete,
documented, self-consistent feature — not a stub:

| File | Lines | What it implements |
|---|---|---|
| `verifier.rs` | 419 | Verifier registration/staking system |
| `liquidation_scanner.rs` | 380 | Keeper-driven liquidation candidate scanning, with real bugfix history (see below) |
| `governance_approval.rs` | 320 | Multi-sig governance voting on slash proposals, with delegation |
| `evidence.rs` | 292 | On-chain evidence-hash storage linked to slash proposals |
| `access_control.rs` | 290 | Role-based access control (admin/verifier/identity-owner) modifiers |
| `pausable.rs`'s multisig extension (already declared, but see below) | — | Threshold pause-signer proposal flow — module *is* compiled, but its multisig functions (`set_pause_signer`, `approve_pause_proposal`, `execute_pause_proposal`, `require_not_paused`) have zero live callers because `test_pausable.rs` (their only caller) is orphaned |
| `cooldown.rs` | 85 | Withdrawal request/cooldown/execute/cancel flow |
| `fees.rs` | 86 | Bond-creation fee mechanism (treasury + bps, waiver support) |
| `status_snapshot.rs` | 83 | Read-only backend-friendly bond status snapshot |

## Orphaned test files (~53 files, ~16,600 lines)

Includes `test_pausable.rs`, `test_verifier.rs`, `test_math.rs`, `test_slashing.rs`,
`test_events.rs` (+ `_v2`, `_schema`, `_ordering`), `test_reentrancy.rs` (+
`_bug_exploration`, `_preservation`), `test_tier_boundary_fuzz.rs`,
`test_liquidation_scanner.rs`, `test_governance_approval.rs`, `test_evidence.rs`,
`test_cooldown.rs`, `test_fees.rs`, `test_withdraw_bond.rs`, `test_create_bond.rs`,
and ~40 more. Full list captured during investigation; ask if you need the exact
inventory re-generated (`comm -23` between files-on-disk and `mod` declarations in
`lib.rs` reproduces it).

These inflate the *appearance* of test coverage in the roadmap's baseline ("test
discipline (design): 8/10") without contributing any actual verification — they are not
compiled, so `cargo test` neither runs nor fails them.

## Evidence this was a regression, not a design choice

- `liquidation_scanner.rs` has real, targeted bugfix history from when it was live:
  `1616d1cb fix(liquidation): scan per-identity bond state, not global DataKey::Bond —
  scan_liquidation_candidates now evaluates each registered identity's own bond and
  removes the dead shadowed ratio computation. (#547)`
- `git log -p -- contracts/trustforge_bond/src/lib.rs` shows `pub mod access_control;`,
  `mod fees;`, `pub mod liquidation_scanner;`, `pub mod verifier;`, `mod test_pausable;`,
  `mod test_liquidation_scanner;`, and others being **added** at one point in history and
  **removed** later, i.e. they were part of the module tree before they were cut.
- `a8cdf7b1`'s own commit message describes finding "two competing implementations of
  the same functions concatenated together" in `lib.rs` and other files after a run of
  `git merge -X theirs`-style conflict resolution, and says it "removed the dead/
  incompatible side of each, keeping the version that matches what the rest of the crate
  and its tests actually exercise." It's plausible the module declarations for these
  files were caught in that cleanup as collateral damage rather than deliberately kept
  disabled — the fix's own stated goal (make main compile again) wouldn't have required
  dropping working modules like `liquidation_scanner`.

## What restoring this actually requires

An attempt to restore was made on 2026-08-10 (see git history around this doc's commit
for the attempt, which was reverted). Adding the `mod` declarations back and running
`cargo check --tests --all-features` surfaced **1,279 compile errors**, dominated by:

- **~45 distinct missing `#[contractimpl]` entrypoint methods** on `TrustForgeBond`
  itself — e.g. `register_verifier`, `set_usdc_token`, `set_emergency_config`,
  `submit_evidence`, `governance_vote`, `propose_slash`, `execute_slash_with_governance`,
  `set_cooldown_period`, `request_cooldown_withdrawal`, `execute_cooldown_withdrawal`,
  `scan_liquidation_candidates`, `get_bond_status_snapshot`, plus ~15 fee/threshold/
  leverage getters and setters. The module *logic* exists and is self-consistent; the
  public contract methods that would call into it do not.
- Signature mismatches between what the orphaned tests call and what the (also
  orphaned) module functions currently accept — some tests may predate the last
  refactor of the types they exercise (`IdentityBond`, `BondTier`).
- A handful of mechanical issues (`extern crate std;` missing in a few test files,
  `serde_json` not in dev-dependencies, `BondTier: Copy` pattern-match errors) that are
  trivial on their own.

In other words: this is not "reconnect a few wires." It is **adding ~45 new public
entrypoints to a contract that custodies staked value**, each needing a real decision
about auth checks, error types, and parameter shape — exactly the kind of change that
should go through design review and, ultimately, the third-party audit in
[Phase 5](../QUALITY_UPGRADE_ROADMAP.md#phase-5--security-get-to-a-real-audit) of the
quality roadmap, not be pushed through under a "make CI green" banner.

## Recommendation

- Do not restore mechanically. Treat this as its own scoped project: go
  method-by-method through the ~45 missing entrypoints, decide (with intent) whether
  each should exist at all in the current design, and if so what its auth/error
  contract should be.
- Prioritize by risk: `verifier.rs` (staking) and `governance_approval.rs` (slash
  voting) touch value custody and dispute outcomes directly — treat these with at least
  as much care as the arbitrator-weight and multisig-expiry items in
  [`known-simplifications.md`](known-simplifications.md).
- Until a method is deliberately restored, leave its module and test file undeclared
  rather than silencing the resulting dead-code warnings — an undeclared module makes
  the gap visible; an `#[allow(dead_code)]` on live-looking code hides it.
- Re-run the `comm -23` inventory (see above) after any restoration work to confirm the
  orphaned set is shrinking, and delete this doc's claims about specific files once they
  are addressed rather than leaving it to drift.
