# Audit Readiness Package

**Purpose:** everything a prospective third-party auditor (or TrustForge, when scoping a
request for proposal) needs to size and price an engagement, without having to re-derive it
from the repository from scratch. This is a scoping aid, not a substitute for the audit
itself — see [`SECURITY_AUDIT.md`](../SECURITY_AUDIT.md) for overall audit status and
[`QUALITY_UPGRADE_ROADMAP.md`](../QUALITY_UPGRADE_ROADMAP.md) Phase 5 for how this fits into
the broader plan.

**What this document is not:** it is not an audit, a security certification, or a substitute
for engaging and paying one of the firms listed below. Commissioning a real audit is a
business decision outside the scope of what this repository's automation can do — this
package exists to make that engagement fast and well-scoped once your team is ready to start
it.

_Compiled: 2026-08-12._

---

## 1. Contract Inventory

8 deployable contracts (each with a Soroban `#[contract]` entrypoint and its own WASM size
budget in [`scripts/wasm-size-budget.toml`](../scripts/wasm-size-budget.toml)), plus
`trustforge_registry` (deployable but currently missing from that budget file — worth asking
the auditor to flag as a process gap), and 2 shared libraries.

| Contract | LOC (`src/`) | Files | Role | WASM budget status |
|---|---:|---:|---|---|
| `trustforge_bond` | 15,885 | 54 | Core value-custody contract: bond lifecycle, attestations, slashing, tiers | ❌ **129KB measured 2026-08-12 vs. 64KB budget — not currently deployable.** See §3. |
| `trustforge_delegation` | 8,199 | 22 | Delegated attestation rights, nonce/domain-separated replay prevention | ✅ 60KB, within budget |
| `trustforge_admin` | 4,235 | 14 | System-wide role hierarchy (SuperAdmin/Admin/Operator) | Not re-measured this pass — see [`docs/admin-roles.md`](admin-roles.md) |
| `trustforge_treasury` | 3,855 | 12 | Fee accounting, withdrawal guardrails, flash-loan fee enforcement | Not re-measured this pass |
| `trustforge_arbitration` | 2,560 | 8 | Dispute resolution; voting weight now derived from bonded stake (2026-08-12) | 45KB per commit history, not re-measured this pass |
| `trustforge_multisig` | 2,341 | 5 | Multi-signature proposal/execution, now with proposal expiry | Not re-measured this pass |
| `trustforge_registry` | 1,134 | 3 | Identity → bond-contract discovery, paginated reads | **Missing from `wasm-size-budget.toml`** — not currently CI-checked |
| `timelock` | 541 | 1 | Time-delayed execution for governance actions | Not re-measured this pass |
| `templates` | 489 | 2 | Contract scaffolding/example — confirm with the team whether this is in scope for audit or purely developer tooling | Not re-measured this pass |
| `trustforge_errors` (library) | 3,042 | 2 | Shared typed-error definitions, no independent WASM | N/A |
| `trustforge_math` (library) | 520 | 1 | Shared checked-arithmetic helpers; ~11 legacy `panic!`-based functions with no `Env` param, by design (no typed-error path without threading `Env` through ~24+ call sites) | N/A |

Re-run `bash scripts/check_wasm_size.sh` after a fresh `cargo build --release --target
wasm32-unknown-unknown -p <contract>` per contract (a full-workspace release build currently
fails — see §3) to get current numbers for the "not re-measured this pass" rows before
sending this to a firm.

## 2. Priority Order for Audit Scope

Per `QUALITY_UPGRADE_ROADMAP.md`'s existing guidance, highest risk first:

1. **`trustforge_bond`** — largest contract, custodies value directly. Also the contract
   with the most historical churn (the deleted orphaned-modules incident, see §4) and the
   only one currently over its WASM budget.
2. **The arbitration / multisig / timelock trio** — governance trust boundary. Arbitration's
   voting-weight derivation and multisig's proposal expiry were both changed on 2026-08-12;
   these are recent enough to warrant focused attention rather than being assumed stable.
3. **`trustforge_delegation` and `trustforge_registry`** — replay/domain-separation
   correctness and discovery/pagination correctness respectively.
4. **`trustforge_admin` and `trustforge_treasury`** — role hierarchy and fee/withdrawal
   guardrail logic.
5. **`timelock` and `templates`** — smaller surface area; confirm `templates`' audit
   relevance with the team first (see inventory note above).

## 3. Known Open Issues (disclose these up front)

Reporting these proactively saves auditor time — they shouldn't have to rediscover what's
already known. All verified 2026-08-12 unless noted.

| Issue | Detail | Reference |
|---|---|---|
| `trustforge_bond` exceeds WASM size budget | 129KB release build vs. 64KB Soroban budget (roadmap's 2026-08-07 baseline measured 137KB — improved but still not deployable) | [`VERIFICATION.md`](../VERIFICATION.md) §9 |
| `trustforge_bond` test coverage below CI gate | 70.64% measured vs. 95% gate; `upgrade_auth.rs` at 0.00% — no test exercises contract-upgrade authorization at all | [`VERIFICATION.md`](../VERIFICATION.md) §10 |
| `THREATS.md` test-fixture references are substantially stale | 42 of 50 threat-registry rows point at test files that don't exist. Spot checks found **zero live automated coverage** for reentrancy, replay-prevention, or arithmetic-overflow in the compiled test suite — their only historical tests were dead code (never compiled, never run in CI). This is the single most audit-relevant open item: it means these threat categories currently rely entirely on manual review, not regression tests. | Warning banner at top of `THREATS.md`; tracked in `QUALITY_UPGRADE_ROADMAP.md` Phase 4 |
| ~70 files of dead code deleted from `trustforge_bond` | Never compiled (undeclared `mod`s in `lib.rs`) — included a verifier-staking system, governance slash-voting, evidence storage, cooldown withdrawals, fees, a liquidation scanner, and read-only status snapshots. Deleted rather than restored; if any of this functionality is wanted, it needs deliberate redesign with real auth/error decisions, which an auditor should review at design time, not after implementation. | [`docs/ORPHANED_MODULES.md`](ORPHANED_MODULES.md) |
| `pausable.rs` multisig pause-flow has zero live callers | `set_pause_signer`, `approve_pause_proposal`, `execute_pause_proposal`, `require_not_paused` are compiled and correct but nothing in `lib.rs` calls them — only `pause`/`unpause`/`is_paused` are wired to production entrypoints | `contracts/trustforge_bond/src/pausable.rs` (documented inline) |
| Full-workspace `--release` WASM build fails | `cargo build --release --target wasm32-unknown-unknown --workspace` errors on `serde_json`/`rand` inside `soroban-sdk`'s `testutils` — a `testutils` feature-unification issue across the workspace, not specific to any one contract's logic. Per-contract builds (`-p <contract>`) work fine and are what `scripts/check_wasm_size.sh` actually needs. | Discovered 2026-08-12 |
| `trustforge_registry` missing from WASM size budget | It has a `#[contract]` entrypoint but no entry in `scripts/wasm-size-budget.toml`, so its size isn't CI-checked the way the other 8 are | `scripts/wasm-size-budget.toml` |
| CI green-on-main not independently reconfirmed since 2026-08-10 | `QUALITY_UPGRADE_ROADMAP.md` Phase 2's last box. Multiple commits have landed since; re-check `.github/workflows/*` status before or during audit kickoff | `QUALITY_UPGRADE_ROADMAP.md` Phase 2 |

## 4. Prior Review Artifacts (hand these to the auditor)

- [`VERIFICATION.md`](../VERIFICATION.md) — full log of `cargo build`/`test`/`clippy`/
  `llvm-cov`/wasm-size runs from Phase 2's internal verification pass (2026-08-10), including
  findings and fixes.
- [`docs/known-simplifications.md`](known-simplifications.md) — current, maintained list of
  intentional design tradeoffs vs. resolved former limitations.
- [`docs/ORPHANED_MODULES.md`](ORPHANED_MODULES.md) — the dead-code finding and deletion
  above, including the git-history evidence for how ~70 files ended up uncompiled.
- [`docs/architecture.md`](architecture.md) — per-contract module breakdown.
- `THREATS.md` (repo root) — threat registry, **read with its 2026-08-12 accuracy warning**;
  do not treat its "✅ Covered" markers as verified.
- [`STATUS.md`](../STATUS.md) (repo root) — single source of truth for audited/deployed/CI
  status, kept in sync with this document.

## 5. RFP Outline

When ready to request quotes from the firms below, a request for proposal should include:

1. **Scope**: link to §1 (inventory) and §2 (priority order) above.
2. **Codebase size**: ~45,000 lines of Rust across `src/` (contracts + libraries combined,
   per §1's LOC column), Soroban/Stellar smart contracts, `no_std`.
3. **Known issues**: §3 above, disclosed up front so the quote reflects real remaining risk
   rather than issues the team already knows about.
4. **Timeline constraint**: none currently fixed — no testnet or mainnet deployment date is
   set (see [`STATUS.md`](../STATUS.md)); audit timeline should drive the deployment
   timeline, not the reverse.
5. **Deliverable requirement**: a publishable report, not just a summary — per
   `SECURITY_AUDIT.md`, the actual report must be published alongside it before any
   audit-related badge or claim is restored to the README.
6. **Suggested/candidate firms** (already named in `SECURITY_AUDIT.md`, not vetted or
   contacted as part of this pass): Trail of Bits, OpenZeppelin, Quantstamp, Certora
   (formal verification — likely best suited to the arithmetic/invariant-heavy paths in
   `trustforge_math` and `trustforge_bond`'s slashing logic specifically, less to the
   broader access-control surface).

## 6. What This Package Does Not Cover

- It does not commission, pay for, or schedule an audit — that requires your organization's
  direct engagement with a firm.
- It does not include a bug bounty program or a live security contact inbox — see
  [`docs/SECURITY_CONTACT_PLAN.md`](SECURITY_CONTACT_PLAN.md) for the latter.
- It does not re-verify every number in §1's inventory table for rows marked "not
  re-measured this pass" — re-run the measurements noted there before finalizing an RFP.
