# Finding: `upgrade_auth.rs` is implemented but not wired to any entrypoint

Discovered 2026-08-10 while auditing `trustforge_bond`'s `unwrap()`/`expect()`/`panic!()`
calls for [QUALITY_UPGRADE_ROADMAP.md](../QUALITY_UPGRADE_ROADMAP.md) Phase 3.

## What's there

`contracts/trustforge_bond/src/upgrade_auth.rs` (799 lines) is a complete, well-documented
UUPS-style upgrade authorization system: role-based authorization (`Upgrader`/`Proposer`),
a proposal/multi-approval workflow for executing upgrades, an upgrade history log, and a
two-step upgrade-admin transfer. It is declared as `mod upgrade_auth;` in `lib.rs` and does
compile into the crate.

## What's missing

No `#[contractimpl]` method in `lib.rs` calls into `upgrade_auth::` at all — not
`propose_upgrade`, not `execute_upgrade`, not `grant_upgrade_auth`, none of it. The module
was invisible to normal dead-code analysis because of a blanket `#![allow(dead_code)]` at
the top of the file (removed as part of the same session that found this, replaced with a
comment pointing here). It also sits at 0.00% test coverage (see
[`VERIFICATION.md`](../VERIFICATION.md) §10) — not one test exercises it, wired-in or not.

## Why this matters

[`docs/UPGRADE.md`](UPGRADE.md) documents a real upgrade procedure — build the new wasm,
deploy it, queue the call through the Timelock contract, and once the delay passes,
"anyone can execute the queued operation through the Timelock. This effectively delegates
the call to the proxy's `execute_upgrade` method" (line 63), with a CLI example that
literally invokes `execute_upgrade` (line 85).

**That procedure cannot run against the current contract.** There is no `execute_upgrade`
entrypoint, no `propose_upgrade` entrypoint, and no way to grant anyone upgrade
authorization in the first place. As shipped, `trustforge_bond` has no upgrade path at
all — not this UUPS mechanism, and no simpler fallback either (there's no direct
`update_current_contract_wasm` call anywhere in `lib.rs` either). Combined with the wasm
size finding in `VERIFICATION.md` §9 (137KB vs. the 64KB deploy limit), the contract
currently can neither be deployed nor, if it somehow were, be upgraded to fix that or any
other defect found later — including in an eventual third-party audit.

## What was done about it (2026-08-10)

Per explicit decision, this session did **not** wire the module in — adding ~8 new public
entrypoints to a value-custody contract is a real API-surface decision needing its own
design review, not something to bundle into a panic-cleanup pass (same reasoning as the
orphaned-modules restoration in [`ORPHANED_MODULES.md`](ORPHANED_MODULES.md), which was
also deferred rather than rushed).

What *was* done: all 31 `unwrap()`/`expect()`/`panic!()` calls in the file were converted
to typed `trustforge_errors::ContractError` variants (115–134), so the module is
ready to wire in — cleanly, with proper error codes — whenever that becomes a deliberate
decision, rather than needing a second pass through it later.

## Recommendation

Treat wiring this in as a dedicated, reviewed piece of work, not a quick addition:

1. Decide whether the intended upgrade flow is really UUPS-via-Timelock as documented, or
   something simpler — and update `docs/UPGRADE.md` to match reality either way in the
   meantime (it currently documents a procedure that doesn't work).
2. If UUPS is the right design, design the `#[contractimpl]` wrapper methods
   (`propose_upgrade`, `approve_upgrade_proposal`, `execute_upgrade`,
   `grant_upgrade_auth`, `revoke_upgrade_auth`, `transfer_upgrade_admin`,
   `accept_upgrade_admin`, plus read views) with the same care given to any other
   admin-facing entrypoint — in particular how `initialize_upgrade_auth` gets called
   (today nothing calls it either, so `UpgradeKey::Admin` is never set).
3. Add real tests before considering this done — it's currently the single largest gap in
   `trustforge_bond`'s coverage precisely because nothing reaches it.
4. This is exactly the kind of surface a third-party audit (Phase 5 of the quality
   roadmap) should see *after* it's wired in and tested, not before.
