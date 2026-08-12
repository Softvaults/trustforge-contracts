# `trustforge_bond` Internal Coherence Review

**Date:** 2026-08-12
**Reviewer:** Claude (Anthropic), acting as a coding agent for this repository's maintainer
**Scope requested:** `QUALITY_UPGRADE_ROADMAP.md` Phase 6 — "one full, deliberate re-read of
`trustforge_bond/src/` end-to-end by a single maintainer... who signs off on it as a unit,"
recorded as a dated review note.

## What this is, and isn't

This is a genuine, deliberate read of the contract's public entrypoints and their call
graphs — not a mechanical scan. It found real, previously-undocumented defects, listed below
with file:line evidence and the reasoning that confirmed each one.

**This is not a substitute for the human sign-off the roadmap item asks for**, and it is not a
third-party audit. An AI agent read this code once, in one session, without the institutional
context a maintainer has (why decisions were made, what's mid-refactor, what's already known
internally). Treat every finding below as "worth independently verifying," not as settled fact
— though each one is backed by concrete evidence, and the top finding was confirmed by cross-
referencing four independent code paths, not a single read-through impression.

**Coverage:** all `#[contractimpl]` entrypoints in `lib.rs` (2,661 lines) were read in full,
plus `validation.rs`, `token_integration.rs`, `slashing.rs`, `claims.rs` (partially), and
`safe_token.rs`. Smaller modules (`nonce.rs`, `pausable.rs`, `emergency.rs`,
`emergency_drain.rs`, `upgrade_auth.rs`, `parameters.rs`, and others) were not read line-by-line
this pass — see [Not Covered](#not-covered-this-pass) at the end.

---

## Critical: the normal withdrawal and fee-collection paths never transfer tokens

**This is the headline finding.** In the currently deployed contract, calling `withdraw()`,
`withdraw_bond()`, or `collect_fees()` on a real, token-backed deployment updates internal
accounting to say funds left the bond, but **no tokens are ever transferred to anyone.** The
funds stay in the contract's custody, permanently, with no recorded claim and no way to
retrieve them through any public entrypoint found in this review.

### Evidence

`token_integration::transfer_from_contract` — the helper that actually moves tokens out of
the contract — is called from exactly one place in the entire `#[contractimpl]` block:
inside `withdraw_early` (`lib.rs:1462` and `:1472`). It is **not** called from:

- `withdraw()` (`lib.rs:1312-1374`) — decrements `bond.bonded_amount`, persists the updated
  bond, returns it. No transfer call anywhere in the function body.
- `withdraw_bond()` (`lib.rs:1682-1746`) — computes `withdraw_amount = bonded_amount -
  slashed_amount`, zeroes `bonded_amount`, marks the bond inactive, fires an optional
  `on_withdraw` callback (see next finding), and **returns `withdraw_amount` as the function's
  return value** without ever transferring it.
- `collect_fees()` (`lib.rs:1835-1873`) — reads and zeroes the `"fees"` counter, fires an
  optional `on_collect` callback, returns the amount. No transfer.

This is not a case of funds moving through an alternate path. The claims/pull-payment system
in `claims.rs` (772 lines, a real, working implementation — its `process_claims` function does
call `TokenClient::transfer` at `claims.rs:482`) is the only other mechanism in this contract
that pays anyone. It is used for exactly one thing: crediting the admin a 10% reward claim on
slashed funds (`slashing.rs:172-181`, via `claims::add_pending_claim`). Neither `withdraw()`,
`withdraw_bond()`, nor `collect_fees()` calls `claims::add_pending_claim` — grep confirms
`add_pending_claim` has exactly one call site in the whole compiled surface, and it isn't any
of these three functions.

**Worse: even the one claim that does get created has no way to be paid out.**
`claims::process_claims` — the function that actually calls `TokenClient::transfer` — has
**zero callers anywhere in `lib.rs`**. There is no `#[contractimpl]` entrypoint that invokes
it. The admin's 10% slash-reward claim is recorded in storage (readable via
`get_pending_claims_page` / `get_claims_summary`) but can never actually be collected through
any public method this review found.

This went unflagged by Phase 3's dead-code cleanup because `claims.rs` carries a blanket
`#![allow(dead_code)]` at the top of the file — `process_claims` being uncalled never
surfaces as a clippy warning.

### Why this isn't a "pull payment by design" pattern

`liquidate()`'s own doc comment (`lib.rs:2036-2041`) describes the intended design explicitly:
> "once a rolling bond's lock-up is over the keeper drives it through `withdraw_bond` instead,
> which already cleanly closes the position."

That's a direct statement that `withdraw_bond` is expected to be a complete, self-sufficient
way to close out and settle a bond — not a bookkeeping-only step in some other flow. The
`claims.rs` module header describes itself as a pull-payment pattern specifically "to prevent
griefing attacks and failed transfers due to recipient contract fallback behavior" — a rationale
that makes sense for admin reward payouts (many small claims, batchable) but there's no
evidence the identity owner's own withdrawal was ever meant to route through it, and no code
does so.

### Suggested severity and next step

Treat as blocking for any deployment with a real token configured. Before writing a fix,
confirm with whoever has the most context on this contract's history whether this is a
regression (something moved/broken during a refactor — plausible, this repo already has one
confirmed incident of exactly this shape, see `docs/ORPHANED_MODULES.md`) or whether an
intended settlement path exists that this review missed. Either way, this needs a real fix
and real test coverage (a test asserting the caller's on-chain token balance actually
increases after `withdraw_bond`) before it can be trusted — the deleted `test_withdraw_bond.rs`
would have been exactly that test, had it ever been compiled; see
`docs/ORPHANED_MODULES.md`.

---

## High: `set_callback` has no access control and can permanently break `withdraw_bond`, `slash_bond`, and `collect_fees`

`set_callback(e: Env, addr: Address)` (`lib.rs:2119-2123`) has no `require_auth()` call and no
admin check — any address can call it. Its own doc comment says "Register a callback contract
for **testing hooks**," confirming it's test instrumentation that was not feature-gated out of
the production build.

Once a callback address is registered, `withdraw_bond` (`lib.rs:1737-1742`),
`slash_bond` — the lib.rs-local one, see next finding — (`lib.rs:1818-1823`), and
`collect_fees` (`lib.rs:1864-1869`) all unconditionally call
`e.invoke_contract::<Val>(&cb_addr, &fn_name, args)` (not `try_invoke_contract`), which
propagates a panic from the callee up through the whole transaction. Anyone can call
`set_callback` with an address that isn't a contract, or a contract that panics on
`on_withdraw`/`on_slash`/`on_collect`, and permanently break those three entrypoints for that
contract instance. Given the single-bond-per-contract-instance architecture, blast radius is
one identity's instance at a time — but zero auth on an entrypoint that can disable value-
movement functions is a real, trivially-exploitable griefing vector, not a theoretical one.

**Fix shape:** gate `set_callback` behind `#[cfg(any(test, feature = "testutils"))]` so it
can't exist in the release WASM at all (consistent with how other test-only helpers in this
crate are already gated — see the `#[cfg(any(test, feature = "testutils"))] mod batch;` line
at the top of `lib.rs`), or add an admin check if it's meant to be a real production feature.

---

## Medium: two divergently-named `slash` entrypoints, only one of which pays the treasury

`TrustForgeBond::slash(admin, amount)` (`lib.rs:1574-1576`) delegates to
`slashing::slash_bond()`, which **does** correctly transfer slashed funds to the configured
treasury (`slashing.rs:195`, `transfer_slashed_funds_to_treasury`).

`TrustForgeBond::slash_bond(admin, slash_amount, idempotency_salt)` (`lib.rs:1760-1827`) is a
**separate, inline implementation** in `lib.rs` — not a call to `slashing::slash_bond`. It adds
real, useful features the first one lacks (idempotency-salt replay protection, an explicit
reentrancy lock) but **does not transfer slashed funds anywhere.** It only updates
`bond.slashed_amount` and fires the (unauthenticated, see above) `on_slash` callback.

An admin who calls `slash_bond()` — the more fully-featured-looking name, with the safety
properties an operator would want — instead of `slash()`, gets a function that succeeds,
returns a plausible-looking cumulative slashed total, and never moves the slashed capital to
the treasury. The two functions' names don't signal this difference at all.

**Fix shape:** either make `slash_bond()` call `transfer_slashed_funds_to_treasury` too (and
decide whether idempotency/reentrancy protection should also be added to `slash()`), or
deprecate one of the two in favor of the other so there's a single slash path.

---

## Medium: `create_bond()` has no `BondAlreadyExists` guard; a second call silently overwrites the bond

`ContractError::BondAlreadyExists = 217` exists in `trustforge_errors`, documented as
"Triggered by: create_bond called for an identity that already has an active bond" — and
`batch.rs`'s internal batch-creation path (`batch.rs:154`) does check for and raise it. But the
primary `TrustForgeBond::create_bond()` entrypoint (`lib.rs:628-671`) has no such check: it
unconditionally does `e.storage().instance().set(&key, &bond)`, overwriting whatever
`IdentityBond` (if any) was already stored — including its `slashed_amount` history — for
whichever `identity` the new caller supplies. Given the single-bond-per-contract-instance
model, the assumption is presumably that `create_bond` is called exactly once per deployed
instance via the registry flow, but nothing in the contract itself enforces that.

**Fix shape:** add the same `has(&DataKey::Bond)` check `batch.rs` already has, raising
`BondAlreadyExists`.

---

## Medium: `create_bond()`'s duration/notice-period validation is fully absent, masked by ~20 passing tests of a disconnected, uncalled function

The real `TrustForgeBond::create_bond()` entrypoint validates only `amount` (via
`validation::validate_bond_amount`) and leverage. It does **not** call
`validation::validate_bond_duration`, and does not check `notice_period_duration` against
`duration` at all. `validation.rs` says so explicitly in its own doc comment (`validation.rs`
lines above `validate_bond_duration`): *"this helper has no production caller today... only its
own tests exercise it."*

Separately, `lib.rs:2412-2443` defines a free-standing `pub fn create_bond(...) ->
Result<Bond, ContractError>` — operating on an unrelated `Bond` struct that exists nowhere else
in the file — which **does** validate zero duration, zero notice period for rolling bonds, and
notice-period-exceeds-duration. This free function has its own ~20-test suite immediately below
it (`create_bond_rejects_zero_duration`, `create_bond_rejects_notice_greater_than_duration`,
etc.), all of which pass. But this function and its `Bond` type are never referenced by the
real contractimpl `create_bond`, or by anything else in the file — they're a fully parallel,
disconnected implementation. The passing test suite creates a false impression that duration
and notice-period validation is tested and working for the deployed contract; it tests a
different function entirely.

Practical consequence: as far as this review found, **the live contract currently accepts
`duration = 0`, and accepts any `notice_period_duration` value for a rolling bond — including
one that exceeds `duration`, or is `0`.** `THREATS.md`'s T-019 ("Set notice period > bond
duration... Notice never clears; funds locked indefinitely") cites exactly this scenario as
"✅ Covered" by `test_rolling_bond.rs::test_notice_period_bounded` — but `test_rolling_bond.rs`
was one of the ~70 orphaned files deleted this session (`docs/ORPHANED_MODULES.md`); it was
never compiled before deletion either, so this mitigation was never actually verified, and per
this finding, may not exist in the deployed contract at all.

**Fix shape:** call `validation::validate_bond_duration` and add explicit notice-period checks
inside the real `create_bond`, then either delete the disconnected free function and its test
suite or repurpose them as the real validation path.

---

## Low: stale, non-compiling files remain in `contracts/trustforge_bond/tests/`

`Cargo.toml`'s `autotests = false` (with a comment explaining why) already correctly prevents
`tests/test_fee_on_transfer_rejection.rs` from being compiled — verified during this review
that it targets an abandoned API (`initialize(admin)` with one arg instead of two,
`create_bond_with_rolling`, `set_usdc_token` with a network-string arg that doesn't match the
current signature, an undefined `identity` variable). This is already correctly neutralized,
not a live risk.

Two more files in the same directory have the same problem and are *also* silently excluded by
`autotests = false`, but aren't mentioned in that comment and weren't confirmed broken until
this review: `tests/migration_test.rs` (uses `Address::random`, a pattern that doesn't fit an
integration-test file — written as if meant to be included via `mod` from `src/`) and
`tests/indexer_replay.rs` (327 lines, otherwise well-documented and substantive — tests that
off-chain indexers can correctly replay bond state from events — but confirmed via a temporary
build attempt to fail compilation: `initialize(&admin)` missing the `registry` argument, and an
undefined `identity` variable at four call sites, the same typo pattern as the other stale
file).

**Fix shape:** either delete these two files (matching the treatment given to the orphaned
`src/` files) or fix and wire them in properly — `indexer_replay.rs` in particular looks like
it would be a valuable, real test suite for the event-replay guarantees `docs/EVENTS.md` and
`docs/indexer-replay-contract.md` already document, if fixed rather than deleted.

---

## Not covered this pass

Read in full: `lib.rs` (all entrypoints), `validation.rs`, `token_integration.rs`,
`slashing.rs`, `safe_token.rs`. Read partially: `claims.rs`.

Not read line-by-line this pass: `nonce.rs`, `pausable.rs`, `emergency.rs`,
`emergency_drain.rs`, `upgrade_auth.rs` (790 lines — the largest unread file, and
`upgrade_auth.rs` is flagged elsewhere in this repo at 0% test coverage per `STATUS.md`, which
makes it a strong candidate for the next read), `parameters.rs`, `events.rs`, `batch.rs`
(beyond the one `BondAlreadyExists` check cited above), `idempotency.rs`, `invariants.rs`,
`leverage.rs`, `migration.rs`, `normalization.rs`, `rolling_bond.rs`,
`same_ledger_liquidation_guard.rs`, `slash_history.rs`, `tiered_bond.rs`,
`weighted_attestation.rs`, `fee.rs`, `fork_divergent.rs`, `types/`.

Given what turned up in the ~40% of the crate that was read, treating this as "done" rather
than "the highest-risk slice was read once" would be its own overclaiming problem. The
`upgrade_auth.rs` / 0%-coverage combination is the most likely candidate to contain something
comparable to the findings above.
