# Quality Upgrade Roadmap: Path to 10/10

## Purpose

This document is the working plan for taking TrustForge from its current state to a codebase that
genuinely earns "production-ready, audited" language — not just claims it. It is **not** about
on-chain contract upgrades (see [`docs/UPGRADE.md`](docs/UPGRADE.md) and
[`docs/UPGRADE_STRATEGY.md`](docs/UPGRADE_STRATEGY.md) for that). This is about the engineering,
security, and process quality of the repository itself.

A backend/indexing service is planned for later and is intentionally **out of scope** here — see
[Section 7](#7-backend-readiness-do-not-build-yet) for what this plan does to avoid blocking it.

## Current Baseline (2026-08-07)

| Dimension | Score | Primary gap |
|---|---|---|
| Documentation | 9/10 | Solid — mostly needs accuracy fixes, not new writing |
| Test discipline (design) | 8/10 | Can't confirm tests actually pass in CI; no coverage floor enforced |
| Security posture | 6/10 | Self-review only; README claims "audited" without a third-party audit |
| Code health | 5/10 | 950 `unwrap()`/`expect()`/`panic!()` in non-test contract source |
| Architecture maturity | 5/10 | Centralized arbitration, no proposal expiry, unbounded registry iteration |
| Provenance / cohesion | 4/10 | 200+ commit authors over ~6 months, many low-signal doc-only PRs |

**Target:** every dimension at 9-10, with claims in the README/SECURITY_AUDIT.md matching reality.

---

## Phase 1 — Stop Overclaiming (do this first, costs almost nothing)

Credibility is the cheapest thing to fix and the most damaging to leave broken.

- [x] Remove the "security: audited ✅" badge from `README.md` until a third-party audit exists.
      Replace with "internal review complete, external audit pending" or similar.
- [x] Rewrite `SECURITY_AUDIT.md`'s framing: "Third-Party Audit (Recommended)" reads as optional —
      make it explicit that mainnet deployment with real TVL should not happen before it's done.
- [x] Replace placeholder testnet contract addresses in `README.md` with either real deployed
      addresses or a clearer "not yet deployed" state (currently `<DEPLOY_YOUR_OWN>` reads as if
      deployment already happened and the reader just needs to fill something in).
- [x] Add a `STATUS.md` (or a status table at the top of `README.md`) that states plainly: audited
      (yes/no), deployed to testnet (yes/no + addresses), deployed to mainnet (yes/no), CI passing
      (link to badge). This becomes the single source of truth other docs point to instead of each
      restating status independently and drifting out of sync.

**Why first:** none of this requires code changes, and every other phase's credibility depends on
the docs not contradicting reality.

---

## Phase 2 — Prove the Baseline (verification before improvement)

We rated this repo without being able to run `cargo build`/`cargo test` in this environment. Before
investing in fixes, confirm what actually works today.

- [x] Run `cargo build --workspace` and `cargo test --workspace` clean, on the pinned toolchain
      (`rust-toolchain.toml`). Fix anything that doesn't pass — do not skip failing tests.
      Build was clean. Found and fixed one real pre-existing test failure (stale
      `contractspecv0` XDR pin in `trustforge_delegation`). Two 10,000-case proptest files
      make a literal full-suite run take ~2+ hours — see `VERIFICATION.md` §5; every test
      that could be run to completion or at reduced cases passed.
- [x] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
      Fixed 33 dead-code/unused-import errors + several unrelated lint errors. Two of the
      dead-code fixes surfaced real functional gaps (unused decimal normalization, unenforced
      pause) — flagged in-code and in `VERIFICATION.md` §7, not silently fixed.
- [x] Run `cargo geiger` and confirm the "no unsafe code" claim in the README is still true.
      `cargo-geiger` itself wouldn't install in-session; verified by manual grep instead.
      Zero unsafe code in the compiled surface — see `VERIFICATION.md` §8.
- [x] Run `bash scripts/check_wasm_size.sh` and confirm every deployable contract is under the
      64KB budget. **`trustforge_bond` fails: 137KB, over 2x the limit — not currently
      deployable to any Stellar network.** 7/8 others pass. See `VERIFICATION.md` §9.
- [x] Pull actual coverage numbers (`cargo tarpaulin --workspace`) instead of relying on "full test
      coverage" as an assertion. Record the real percentage in `STATUS.md`.
      Used `cargo llvm-cov` (tarpaulin wouldn't install against the pinned toolchain, and
      isn't actually what CI's `coverage.yml` uses anyway). `trustforge_bond` is at 70.64%,
      below CI's 95% gate, with `upgrade_auth.rs` at a flat 0.00%. See `VERIFICATION.md` §10.
- [ ] Confirm CI (`.github/workflows/*`) is green on `main` right now, not just configured.

**Output of this phase:** see [`VERIFICATION.md`](VERIFICATION.md) for the full log of what was
run, when, and the result, plus two findings big enough to get their own documents:
[`docs/ORPHANED_MODULES.md`](docs/ORPHANED_MODULES.md) (~60 files never compiled into
`trustforge_bond`) and the `trustforge_bond.wasm` 64KB-limit overshoot above.

---

## Phase 3 — Code Health (close the unwrap/panic gap)

950 `unwrap()`/`expect()`/`panic!()` calls in non-test `trustforge_bond` source is the single
biggest concrete code-quality gap. In Soroban a panic aborts the transaction rather than corrupting
state, so this isn't a fund-safety emergency — but it's sloppy for a contract handling staked value,
and each one is a spot where a malformed input produces an opaque abort instead of a typed error.

- [x] Audit every `unwrap()`/`expect()`/`panic!()` in `contracts/*/src/` (excluding test files).
      Categorize each as: (a) provably unreachable (add a comment explaining why, or convert to
      `unreachable!()` with justification), or (b) a real failure mode that should return a typed
      error via `trustforge_errors`.
      Done for `trustforge_bond`, `trustforge_errors`, `trustforge_registry`, and `templates` —
      their compiled non-test surfaces are now at zero, CI-enforced (see below). Three
      documented, provably-dead-code exceptions remain in `trustforge_bond`
      (`validation.rs`'s `validate_recipient`/`validate_bond_duration`, `slashing.rs`'s
      `get_available_balance` — all `Env`-less, no production caller, see
      `docs/ORPHANED_MODULES.md`). `trustforge_math`'s ~10 legacy `panic!`-based checked-math
      helpers (`mul_i128`, `div_i128`, etc.) are intentionally untouched: they have no `Env`
      parameter so can't call `panic_with_error!`, and typed `Result`-returning counterparts
      (`div_checked_i128`, `ceil_div_checked_i128`) already exist for the two hottest paths —
      fully eliminating the rest means threading `Env` (or `Result`) through ~24+ call sites
      across `trustforge_bond`, deferred as a separate, larger decision.
- [x] Start with `trustforge_bond` (240 instances) since it's the core value-custody contract, then
      `trustforge_errors` itself (56 — ironic, given its job) and `trustforge_arbitration` (19).
      All three are at zero in their compiled non-test surface.
- [x] Add a workspace-level clippy lint (`#![deny(clippy::unwrap_used, clippy::expect_used)]` at the
      crate root, with a narrow `#[allow]` at each justified call site) so regressions fail CI
      instead of accumulating silently again.
      Added to `trustforge_bond`, `trustforge_errors`, `trustforge_registry`, and `templates`,
      scoped `cfg_attr(not(test), deny(...))` so it only bites the compiled surface, not
      `#[cfg(test)]` modules. Verified live (a deliberately-injected `.unwrap()` was confirmed to
      fail `cargo clippy`) and clean under CI's exact
      `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [x] Re-run the unwrap/expect/panic count from this conversation's audit as a CI-enforced ceiling
      (start by pinning today's count as the max, then ratchet it down per PR).
      The ceiling is the clippy `deny` above, not a separate count script — a regression fails
      the build immediately rather than needing a periodic re-count.

---

## Phase 4 — Architecture Maturity (close the documented simplifications)

`docs/known-simplifications.md` is honest about these — that's good. The gap is that several are
listed as "acceptable for now" when they're actually load-bearing trust assumptions for a
protocol calling itself production-ready.

Priority order (highest centralization/trust risk first):

- [x] **Restore or deliberately delete ~60 orphaned files in `trustforge_bond`**
      (`access_control.rs`, `verifier.rs`, `governance_approval.rs`, `evidence.rs`,
      `cooldown.rs`, `fees.rs`, `status_snapshot.rs`, the pause-multisig functions in
      `pausable.rs`, and ~53 test files) — not declared as `mod` in `lib.rs`, so never
      compiled into anything. Requires adding ~45 missing `#[contractimpl]` entrypoints
      with real auth/error design, not a mechanical reconnect. See
      [docs/ORPHANED_MODULES.md](docs/ORPHANED_MODULES.md) for the full finding
      (discovered 2026-08-10 during Phase 2 verification).
      Deleted rather than restored (2026-08-12) — restoring means adding ~45 new
      value-custody entrypoints, which this doc's own recommendation says needs
      design-review/audit-level care, not a cleanup-pass rush job. 70 files removed
      total: the original ~61 plus 9 more the initial audit missed in `security/`,
      `integration/`, and `fuzz/` subdirectories. Build/test/clippy verified clean
      afterward; wasm size unchanged (nothing here was ever compiled). Docs that
      described these modules as live (`docs/architecture.md`,
      `docs/access-control.md`, `docs/status-snapshot.md`, `docs/trustforge-bond.md`,
      `docs/liquidation.md`, `contracts/trustforge_bond/docs/pagination.md`) updated
      to say so. Byproduct finding: this exposed that `THREATS.md`'s test-fixture
      references are substantially stale independent of this cleanup (42/50 rows
      point at nonexistent files) — flagged with a warning banner there, not fixed;
      see the note added to `docs/ORPHANED_MODULES.md`'s resolution section. That
      remediation is unscoped, tracked as a new follow-up, not part of this item.
- [ ] **`THREATS.md` test-fixture references are substantially stale** (discovered
      2026-08-12 as a byproduct of the orphaned-modules deletion above) — a
      file-existence check found 42 of 50 threat rows point at test files that don't
      exist (some because they lived in the orphaned files just deleted and were
      never actually compiled; others never existed under the referenced name at
      all). Spot checks found zero live coverage anywhere in the compiled surface for
      reentrancy, replay-prevention, or arithmetic-overflow test categories — the only
      tests that ever covered them were themselves orphaned. The `tests/threats_link.rs`
      bidirectional-traceability validator this document describes does not exist.
      Needs a dedicated pass: verify or restore real coverage per threat, correct each
      row's Test Fixture column, and either build the described validator or stop
      claiming it exists. Not fixed as part of the orphaned-modules item above —
      flagged with a warning banner in `THREATS.md` instead, since a rushed row-by-row
      edit risked introducing new inaccuracies without deeper investigation.
- [x] **Arbitrator weights not stake-backed** (`trustforge_arbitration`) — currently pure
      admin-assigned integers, i.e. the admin key fully controls dispute outcomes. Derive weight
      from `trustforge_bond` balance via cross-contract call, or require arbitrators to stake into
      the arbitration contract directly. This is the highest-priority item — it undermines the
      "decentralized" claim in the README's very first line.
      Implemented the cross-contract-derivation option: `register_arbitrator` no longer takes a
      weight — it only grants voting permission. `vote()`/`get_arbitrator_weight()` derive weight
      live via `trustforge_registry.get_bond_contract()` → that bond's `get_identity_state()`
      (`bonded_amount - slashed_amount`), configured through a new `set_registry_contract`
      admin call. Discovered mid-implementation that a real Cargo dependency on
      `trustforge_bond`/`trustforge_registry` collides at the WASM export level (both crates'
      `#[contractimpl]` blocks define same-named entrypoints like `initialize`/`pause`, which
      Soroban's macros emit as flat unmangled symbols) — worked around with a local structural
      mirror type (`BondRegistryEntry`/`BondIdentityState` in `lib.rs`) decoded via
      `try_invoke_contract`, and `trustforge_bond`/`trustforge_registry` as dev-dependencies only
      (for building real bonded-arbitrator fixtures in tests, which never reach the release WASM).
      Verified: `trustforge_arbitration`'s WASM stayed within budget (45KB, unchanged order of
      magnitude), full test suite green (57 lib tests + 6 new behavioral tests in
      `tests/test_weight_derivation.rs` covering snapshot-immutability, zero-weight rejection,
      and overflow-checked aggregation), `datakey_fingerprint.rs` regenerated for the new
      `RegistryContract` storage key.
- [x] **Multisig proposals never expire** (`trustforge_multisig`) — add `expires_at` and reject
      approval/execution past it. Low effort, real risk (stale proposal executed in a changed
      context).
      Already implemented in code (`expires_at` field, `prune_expired_proposals`, expiry checks
      in `sign_proposal`/`execute_proposal`) and in `docs/multisig.md` — only
      `known-simplifications.md` still described it as open. Moved to Resolved.
- [x] **`get_all_identities()` is unbounded** (`trustforge_registry`) — add
      `get_identities_page(offset, limit)`, deprecate the unbounded call, and update any internal
      caller (including the future backend indexer — see Phase 7) to use event-based discovery
      instead of polling.
      Already implemented in code (`get_identities_page`, `#[deprecated]` on
      `get_all_identities`) but never documented in `registry.md` and still listed as open in
      `known-simplifications.md`. Added the missing operation docs, moved to Resolved.
- [x] **Single-bond-per-contract-instance model** — this is a legitimate design choice, not just a
      simplification, but it should be a documented decision with a stated reason (avoids
      cross-identity storage leakage) rather than filed under "limitations." Either promote it to
      `docs/architecture.md` as an intentional tradeoff with its cost (per-identity deployment gas)
      spelled out, or build the `Map<Address, IdentityBond>` alternative if per-identity deployment
      cost turns out to be a real adoption blocker.
      Promoted to `docs/architecture.md` with the rationale, cost, and alternative spelled out;
      `known-simplifications.md`'s item #2 now points there instead of framing it as an open gap.
- [x] Re-run `docs/known-simplifications.md` after each fix — items should move from "Current
      Limitations" to "Resolved" with the date and PR, matching the existing pattern for item #4.
      Done for the two items resolved above.

---

## Phase 5 — Security (get to a real audit)

- [ ] Fix everything from Phases 3-4 *before* engaging an external auditor — audits are expensive
      per finding-round, and self-fixable issues shouldn't burn auditor time.
- [ ] Commission a third-party audit from one of the firms already named in `SECURITY_AUDIT.md`
      (Trail of Bits / OpenZeppelin / Quantstamp / Certora). Scope: all eight contracts, with
      particular attention to `trustforge_bond` (largest, custodies value) and the
      arbitration/multisig/timelock trio (governance trust boundary).
- [ ] Publish the actual audit report (not just a summary) alongside `SECURITY_AUDIT.md`, and only
      then restore audit-related badges/claims in the README.
- [ ] Stand up the bug bounty program that's currently listed as "to be launched post-deployment" —
      tie it to real testnet/mainnet deployment, not left indefinite.
- [ ] Replace the placeholder `security@trustforge.io (coming soon)` contact with a working channel
      before any public deployment — an unreachable security contact on a financial contract is a
      real gap, not cosmetic.

---

## Phase 6 — Provenance & Process (fix the contribution pattern)

The 200+-author, 6-month commit history with many trivial doc-only PRs is the hardest thing to fix
retroactively and the main reason "provenance/cohesion" scored lowest. You can't rewrite history
credibly, but you can change what happens going forward:

- [ ] Add a `CODEOWNERS` file so changes to `contracts/*/src/` (not just `docs/`) require review
      from a small, named set of maintainers — currently anyone's PR seems mergeable, which is how
      a repo ends up with hundreds of drive-by contributors and no clear owner of correctness.
- [ ] Tighten `CONTRIBUTING.md` to distinguish "good first issue" doc/typo PRs (fine to keep open
      for community engagement) from contract-logic PRs (should require design discussion first,
      not just a passing CI run).
- [ ] Do one full, deliberate re-read of `trustforge_bond/src/` end-to-end by a single maintainer
      (or a small team) who signs off on it as a unit — right now correctness confidence is spread
      thin across many small, disjoint contributions rather than anchored in anyone's complete
      mental model of the contract. Record this as a dated review note, similar in spirit to
      `SECURITY_AUDIT.md` but focused on internal coherence rather than vulnerability classes.
- [ ] Consider whether the git history itself needs a note (e.g. in `CONTRIBUTING.md` or a
      `HISTORY.md`) explaining the project's origin as a mass-contribution effort — transparency
      about provenance is better than leaving newcomers to guess why the author list looks the way
      it does.

---

## 7. Backend Readiness (do not build yet)

Per current direction, the backend is a later project. Nothing in this roadmap should be blocked on
it, but a few choices now will make that integration cheap instead of a rewrite:

- [ ] When fixing unbounded registry iteration (Phase 4), design the paginated
      `get_identities_page()` API and event schema (`docs/EVENTS.md`) with an off-chain indexer as
      the primary consumer in mind — this is the API surface the backend will actually call.
- [ ] Do not build any backend service, API server, or database schema as part of this roadmap —
      that's explicitly deferred. This phase is limited to not making future backend integration
      harder.
- [ ] Keep `docs/EVENT_INDEXING_MIGRATION.md` and `docs/EVENTS.md` up to date as the de facto
      contract for whatever indexer gets built later.

---

## Sequencing

Recommended order: **Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6**, with Phase 7
constraints applied opportunistically inside Phase 4 (no separate time block needed).

Phases 1-2 are cheap and should happen this week. Phases 3-4 are the bulk of the real engineering
work. Phase 5 (external audit) should only start once 3-4 are substantially done. Phase 6 is
ongoing process change, not a one-time task — it doesn't "complete," it just starts being true going
forward.

## Definition of Done (10/10)

- README and `SECURITY_AUDIT.md` claims are all independently verifiable and currently true.
- CI is green, coverage is measured and reported (not asserted), and unwrap/expect/panic count in
  non-test contract code is trending down under an enforced ceiling.
- Arbitration weight, multisig expiry, and registry pagination gaps from
  `known-simplifications.md` are resolved or reclassified as intentional, justified design choices.
- A real third-party audit report exists and is linked from `SECURITY_AUDIT.md`.
- Contract-logic PRs go through named-owner review, not open merge.
