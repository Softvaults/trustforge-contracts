# Verification Log — Phase 2

Records what was actually run for [QUALITY_UPGRADE_ROADMAP.md](QUALITY_UPGRADE_ROADMAP.md)
Phase 2 ("Prove the Baseline"), when, and the real result — so "it works" is a checked
fact, not folklore. Session date: 2026-08-10.

## Summary

Phase 2's premise — "we rated this repo without being able to run `cargo build`/
`cargo test` in this environment" — turned out to be justified. Running the checks
surfaced two categories of problems: things that were simply never run (a stale test
snapshot, a huge slice of source that was never compiled at all), and one concrete,
unresolved blocker (the bond contract is over 2x the deployable size limit). None of
this was visible from reading the code or the existing docs; it only showed up by
actually executing the toolchain end to end.

## 1. `cargo build --workspace`

**Result: clean.** No errors, no warnings, ~2m38s cold build.

## 2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`

**Result: clean**, after fixing 33 dead-code/unused-import errors plus a handful of
unrelated lint errors (duplicated `cfg(test)` attributes, unnecessary casts, a
deprecated `register_contract` call, a genuine copy-pasted duplicate line in a test,
two `assert!` calls on compile-time constants). None of these were introduced this
session — the working tree already had partial, uncommitted fmt fixes in progress when
this session started; the remaining clippy failures were what was blocking CI.

Of the 33 dead-code items, only 6 (in `storage.rs` and `emergency_drain.rs`) were true
orphaned duplicates and were deleted outright. The rest were real, working code whose
only caller is either an orphaned test file (see §6) or nothing in production at all —
these were kept and marked `#[allow(dead_code)]` with a comment explaining why, rather
than deleted, per the principle in [docs/ORPHANED_MODULES.md](docs/ORPHANED_MODULES.md):
an undeclared module makes a gap visible; deleting working code to silence a lint hides
it. Two of those comments flag real functional gaps, not just lint noise — see §7.

## 3. `cargo fmt --all -- --check`

**Result: clean**, after running `cargo fmt --all` once (mostly the pre-existing
uncommitted formatting work already in the tree, plus reformatting of this session's
own edits).

## 4. `cargo test --workspace`

**Result: every test that finished running, passed. Zero failures.** But two proptest
files cannot be run to completion in an interactive session — see §5 — so "the full
suite passes" was verified in two pieces:

- Every test *except* `trustforge_bond::test_tier_fuzz` and the two `prop_*` tests in
  `trustforge_delegation/tests/nonce_replay.rs`: full pass, all crates.
- The three excluded proptest functions: confirmed passing at a reduced case count
  (`PROPTEST_CASES` doesn't apply to two of the three since their case count is
  hardcoded — reduced by running with `proptest::test_runner` case overrides directly),
  and one of the two `nonce_replay.rs` tests was allowed to run to completion at its
  full 10,000 cases and passed. See §5 for why the other two were not run to
  completion.

**One real, pre-existing test failure was found and fixed:**
`trustforge_delegation/tests/spec_xdr_regression.rs` had two failing tests
(`contract_spec_version_matches_pinned_manifest`, `contract_spec_xdr_is_pinned`). A
`version()` view function had been added to `trustforge_delegation`'s public contract
interface at some point without refreshing the pinned `contractspecv0` XDR snapshot
this test guards. Diffing the actual vs. pinned XDR confirmed the drift was exactly
that one additive, non-breaking function — not an unnoticed breaking change — so it was
safe to follow the test's own documented refresh procedure: regenerated
`tests/spec_xdr/trustforge_delegation.v1.hex` from the current build, bumped
`CONTRACT_SPEC_VERSION` 1→2 in `src/lib.rs`, and updated `EXPECTED_VERSIONED_MANIFEST`
to match. This was already broken before this session (present in the uncommitted-fmt
diff's parent state, not introduced by any change here) and had nothing to do with
`trustforge_bond`.

## 5. Finding: two proptest files make a full `cargo test --workspace` take ~2+ hours

`contracts/trustforge_bond/tests/proptest_tier.rs` sets
`ProptestConfig { cases: 10000, .. }` for `test_tier_fuzz`. Measured rate: ~0.73s/case
(50 cases → 36.48s), so 10,000 cases ≈ **2 hours** for this one test alone.
`contracts/trustforge_delegation/tests/nonce_replay.rs` has a `proptest! { #![...
with_cases(10_000)] }` block containing *two* property tests
(`prop_all_invalidated_nonces_rejected`, `prop_post_invalidation_nonce_accepted`),
similarly slow — one was run to completion (passed), the other was still running after
being let run far longer than its sibling before this log was written.

Neither is a correctness bug — every one of these that was run to completion or at a
reduced case count passed. But `.github/workflows/ci.yml`'s `test` job runs plain
`cargo test --all-targets` with no case-count override and no unusual
`timeout-minutes`, so on a fresh CI runner this job either takes multiple hours or is
being implicitly relied upon to finish within GitHub's 6-hour default job timeout —
neither of which anyone is likely to sit and watch. **This is almost certainly the
concrete mechanism behind the roadmap's baseline line "can't confirm tests actually
pass in CI."** Not fixed here (reducing case counts is a real coverage-vs-speed
tradeoff, not a mechanical cleanup); flagged for a deliberate decision — e.g. a lower
default `cases` count with a `PROPTEST_CASES`-driven "full fuzz" mode reserved for a
scheduled/nightly job rather than every push.

## 6. Finding: ~60 files / ~18,500 lines never compiled into `trustforge_bond`

Full writeup: [docs/ORPHANED_MODULES.md](docs/ORPHANED_MODULES.md). Summary: 8 real
feature modules (verifier staking, governance-voted slashing, evidence storage, access
control, cooldowns, fees, status snapshots, plus the multisig extension to the already-
compiled `pausable.rs`) and ~53 test files are never declared as `mod` in `lib.rs`, so
they're not part of the compiled crate at all — not in the release WASM, not in
`cargo test`. Git history shows they were live and under active bugfix before being
dropped, most likely during the `a8cdf7b1` merge-corruption cleanup or the rebrand that
followed it. An attempted restoration surfaced ~45 missing `#[contractimpl]`
entrypoints and 1,279 compile errors — this is a scoped project of its own (adding new
public methods to a value-custody contract), not a Phase 2 cleanup item, and was
reverted rather than rushed. Not restored; documented for deliberate follow-up.

## 7. Finding: two implemented-but-never-invoked production code paths

Discovered while triaging the clippy dead-code list in §2; flagged with
`#[allow(dead_code)]` comments in the source rather than silently fixed, since both are
judgment calls about intended behavior, not lint noise:

- **`normalization.rs`**: `normalize`/`denormalize`/`get_scale_info`/
  `can_normalize_safely` and the `NORMALIZED_DECIMALS`/`MIN_SUPPORTED_DECIMALS`
  constants are exercised only by a `cfg(test)` fuzz test. Nothing in the actual
  deposit/withdraw path scales amounts by token decimals — only
  `validate_supported_decimals` (which checks a token's decimals are ≤18, but doesn't
  scale) is called from production code. A token configured with fewer than 18
  decimals — e.g. real USDC, which has 6 — would currently have its raw amount used
  as-is rather than converted to the 18-decimal internal accounting the module's own
  doc comment says all internal accounting uses.
- **`pausable.rs`**: no production entrypoint calls `require_not_paused` or checks
  `is_paused()` before executing. `pause()`/`unpause()` currently only flip a storage
  flag with no effect on any other contract method — pause is not actually enforced
  anywhere. The full multisig pause-signer/proposal flow (`set_pause_signer` through
  `execute_pause_proposal`) is implemented and was working code, but its only exerciser
  (`test_pausable.rs`) is one of the orphaned files from §6.

Both are real design gaps for a contract handling staked value, not simplifications
already tracked in `docs/known-simplifications.md`. Recommended for Phase 4/5 review
alongside the arbitrator-weight and multisig-expiry items already there.

## 8. Unsafe code (`README.md`'s "no unsafe code" claim)

`cargo-geiger` could not be installed in this session — it pulls in a large dependency
tree (git, curl, tls bindings) and repeatedly exceeded reasonable install time.
Verified manually instead: `grep -rn "unsafe" contracts/*/src --include=*.rs` finds
exactly **two** `unsafe` blocks in the entire workspace
(`test_tier_boundary_fuzz.rs:123`, `fuzz/test_slashing_tier_invariants.rs:93`), both
`core::mem::transmute` calls in test files, and both are inside the orphaned-file set
from §6 — not compiled today. **The compiled surface (release WASM and everything
`cargo test` actually runs) contains zero unsafe code**, consistent with the README's
claim, though the claim should note it hasn't been re-verified by the actual
`cargo-geiger` tool that `security.yml` runs in CI.

## 9. `scripts/check_wasm_size.sh`

Built with `cargo build --target wasm32-unknown-unknown --release --locked` for the 7
packages `docs/DEPLOYMENT.md` lists, plus `trustforge_registry` (a real, complete,
deployable contract that is puzzlingly absent from both `docs/DEPLOYMENT.md`'s build
list and `scripts/wasm-size-budget.toml` — likely a documentation oversight, flagged
here rather than silently patched into those files).

```
[PASS] timelock:              16KB ( 16,549 bytes)
[PASS] trustforge_admin:      48KB ( 49,514 bytes)
[PASS] trustforge_arbitration:40KB ( 41,458 bytes)
[FAIL] trustforge_bond:      137KB (140,324 bytes)  <-- exceeds 64KB limit by >2x
[PASS] trustforge_delegation: 59KB ( 60,812 bytes)
[PASS] trustforge_multisig:   20KB ( 21,024 bytes)
[PASS] trustforge_registry:   20KB ( 20,778 bytes)
[PASS] trustforge_treasury:   31KB ( 32,522 bytes)
```

**`trustforge_bond.wasm` is 137KB against Soroban's ~64KB on-chain deploy limit — the
contract cannot currently be deployed to any Stellar network, testnet or mainnet, full
stop.** This is with the workspace's already-maximal size optimization profile
(`opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`) — there is
no cheap post-processing step being skipped; this is the real, final artifact size.
Given `trustforge_bond` is also the contract carrying the most functionality (batch
ops, claims, early-exit penalties, emergency drain, fees, idempotency, leverage,
migration, nonces, pausable multisig, rolling bonds, safe-token wrappers, same-ledger
liquidation guards, slash history, tiered bonds, token integration, upgrade auth,
validation, weighted attestation — 29 non-test modules), this likely needs either
splitting into multiple contracts or a deliberate feature-trim, not a quick fix. Not
attempted here; this is a hard blocker for Phase 5 (external audit) and any real
deployment, and should be escalated above the Phase 4 architecture items.

## 10. `cargo tarpaulin --workspace` → measured via `cargo llvm-cov` instead

`cargo-tarpaulin` failed to install (a transitive dependency needs rustc ≥1.91; this
repo pins 1.89.0 in `rust-toolchain.toml`). `.github/workflows/coverage.yml` doesn't
actually use tarpaulin either — it uses `cargo-llvm-cov` on a separately-installed
`stable` toolchain (not the pinned 1.89.0), which does not have this issue, so that's
what was used here, matching what CI actually runs.

CI's `coverage.yml` only measures 3 of the 8 deployable contracts
(`trustforge_bond`, `trustforge_delegation`, `timelock`) with a 95%-line
`--fail-under-lines` gate. Measured this session (bond and delegation with the two
10,000-case proptest functions skipped — see §5 — so these numbers exclude whatever
incremental coverage those specific fuzz tests would add on top of their sibling
boundary/property tests):

| Crate | Line coverage | vs. 95% CI gate |
|---|---|---|
| `timelock` | 95.80% | pass |
| `trustforge_delegation` | 95.85% | pass |
| `trustforge_bond` | **70.64%** | **fail** |

`trustforge_bond`'s shortfall is concentrated: `upgrade_auth.rs` is at **0.00%
coverage** (551/551 lines, 44/44 functions never executed by anything that ran) —
the contract-upgrade authorization path has no test exercising it at all. Also notably
low: `types/attestation.rs` (20.00%), `tiered_bond.rs` (34.09%),
`token_integration.rs` (67.36%). Not investigated further or fixed here — recorded as
the real number Phase 2 asked for, replacing the roadmap's "full test coverage"
assertion with a measured fact. `trustforge_registry`, `trustforge_treasury`,
`trustforge_admin`, `trustforge_multisig`, and `trustforge_arbitration` have no
coverage gate in CI at all and were not measured here.

## 11. CI status

See `STATUS.md` for the current live CI badge state after this session's commit.

## What Phase 2 didn't cover

- The two 10,000-case proptests were not run to full completion end-to-end together in
  a single session (see §5) — each was verified passing individually, at reduced case
  counts or to completion in isolation, not as part of one unified "green" run.
- `cargo-geiger` (the actual tool `security.yml` runs) was not run; §8's manual grep is
  a reasonable substitute but not identical verification.
- Coverage was not measured for 5 of 8 deployable contracts (no CI gate exists for
  them either).
