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

- [ ] Run `cargo build --workspace` and `cargo test --workspace` clean, on the pinned toolchain
      (`rust-toolchain.toml`). Fix anything that doesn't pass — do not skip failing tests.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- [ ] Run `cargo geiger` and confirm the "no unsafe code" claim in the README is still true.
- [ ] Run `bash scripts/check_wasm_size.sh` and confirm every deployable contract is under the
      64KB budget.
- [ ] Pull actual coverage numbers (`cargo tarpaulin --workspace`) instead of relying on "full test
      coverage" as an assertion. Record the real percentage in `STATUS.md`.
- [ ] Confirm CI (`.github/workflows/*`) is green on `main` right now, not just configured.

**Output of this phase:** a short `VERIFICATION.md` note (or a section in `STATUS.md`) recording
what was actually run, when, and the result — so "it works" is a checked fact, not folklore.

---

## Phase 3 — Code Health (close the unwrap/panic gap)

950 `unwrap()`/`expect()`/`panic!()` calls in non-test `trustforge_bond` source is the single
biggest concrete code-quality gap. In Soroban a panic aborts the transaction rather than corrupting
state, so this isn't a fund-safety emergency — but it's sloppy for a contract handling staked value,
and each one is a spot where a malformed input produces an opaque abort instead of a typed error.

- [ ] Audit every `unwrap()`/`expect()`/`panic!()` in `contracts/*/src/` (excluding test files).
      Categorize each as: (a) provably unreachable (add a comment explaining why, or convert to
      `unreachable!()` with justification), or (b) a real failure mode that should return a typed
      error via `trustforge_errors`.
- [ ] Start with `trustforge_bond` (240 instances) since it's the core value-custody contract, then
      `trustforge_errors` itself (56 — ironic, given its job) and `trustforge_arbitration` (19).
- [ ] Add a workspace-level clippy lint (`#![deny(clippy::unwrap_used, clippy::expect_used)]` at the
      crate root, with a narrow `#[allow]` at each justified call site) so regressions fail CI
      instead of accumulating silently again.
- [ ] Re-run the unwrap/expect/panic count from this conversation's audit as a CI-enforced ceiling
      (start by pinning today's count as the max, then ratchet it down per PR).

---

## Phase 4 — Architecture Maturity (close the documented simplifications)

`docs/known-simplifications.md` is honest about these — that's good. The gap is that several are
listed as "acceptable for now" when they're actually load-bearing trust assumptions for a
protocol calling itself production-ready.

Priority order (highest centralization/trust risk first):

- [ ] **Arbitrator weights not stake-backed** (`trustforge_arbitration`) — currently pure
      admin-assigned integers, i.e. the admin key fully controls dispute outcomes. Derive weight
      from `trustforge_bond` balance via cross-contract call, or require arbitrators to stake into
      the arbitration contract directly. This is the highest-priority item — it undermines the
      "decentralized" claim in the README's very first line.
- [ ] **Multisig proposals never expire** (`trustforge_multisig`) — add `expires_at` and reject
      approval/execution past it. Low effort, real risk (stale proposal executed in a changed
      context).
- [ ] **`get_all_identities()` is unbounded** (`trustforge_registry`) — add
      `get_identities_page(offset, limit)`, deprecate the unbounded call, and update any internal
      caller (including the future backend indexer — see Phase 7) to use event-based discovery
      instead of polling.
- [ ] **Single-bond-per-contract-instance model** — this is a legitimate design choice, not just a
      simplification, but it should be a documented decision with a stated reason (avoids
      cross-identity storage leakage) rather than filed under "limitations." Either promote it to
      `docs/architecture.md` as an intentional tradeoff with its cost (per-identity deployment gas)
      spelled out, or build the `Map<Address, IdentityBond>` alternative if per-identity deployment
      cost turns out to be a real adoption blocker.
- [ ] Re-run `docs/known-simplifications.md` after each fix — items should move from "Current
      Limitations" to "Resolved" with the date and PR, matching the existing pattern for item #4.

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
