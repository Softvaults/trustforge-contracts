# Status

Single source of truth for TrustForge's current audit/deployment state. Other docs
(`README.md`, `SECURITY_AUDIT.md`) should point here rather than restating these facts
independently, so they can't drift out of sync.

_Last updated: 2026-08-11._

| Question | Answer |
|---|---|
| Third-party audited? | **No.** Internal review only. See [SECURITY_AUDIT.md](SECURITY_AUDIT.md). |
| Deployed to testnet? | **No.** See [README.md § Testnet](README.md#testnet) for the deployment layout you get after following [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md). |
| Deployed to mainnet? | **No.** Blocked on a completed third-party audit and governance approval. |
| **Deployable at all?** | **`trustforge_bond` — no.** Its release WASM is 137KB against Soroban's ~64KB on-chain limit, more than double budget even with maximal size optimization already applied. The other 7 deployable contracts are within budget. See [VERIFICATION.md](VERIFICATION.md) §9. |
| CI passing? | See live status: [![CI](https://github.com/Softvaults/trustforge-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/ci.yml) [![Contracts Tests](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-tests.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-tests.yml) [![Contracts Lints](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-lints.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-lints.yml) [![Coverage](https://github.com/Softvaults/trustforge-contracts/actions/workflows/coverage.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/coverage.yml) [![Security Scanning](https://github.com/Softvaults/trustforge-contracts/actions/workflows/security.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/security.yml) |
| Test coverage (measured, not asserted) | **`timelock` 95.80%, `trustforge_delegation` 95.85%, `trustforge_bond` 70.64%** (below CI's 95% gate; `upgrade_auth.rs` is at 0.00% — no test exercises contract-upgrade authorization at all). These are the only 3 of 8 deployable contracts with a coverage gate. Measured 2026-08-10 via `cargo llvm-cov` (CI's actual tool) — see [VERIFICATION.md](VERIFICATION.md) §10. |
| `unwrap()`/`expect()`/`panic!()` in non-test contract code | **0** in the compiled surface of `trustforge_bond`, `trustforge_errors`, `trustforge_registry`, and `templates` (down from 950 at the 2026-08-07 baseline) — enforced by a crate-level `#[deny(clippy::unwrap_used, clippy::expect_used)]` so regressions fail CI. 3 documented exceptions remain, all provably-dead code with no production caller (see [docs/ORPHANED_MODULES.md](docs/ORPHANED_MODULES.md)). `trustforge_math` has ~11 by design — its checked-math helpers have no `Env` parameter so can't raise a typed `ContractError`; the 2 hottest call paths already have `Result`-returning counterparts. The ~60 orphaned (uncompiled) `trustforge_bond` modules still contain unconverted panics — out of scope until Phase 4 restores or deletes them. Tracked in [QUALITY_UPGRADE_ROADMAP.md](QUALITY_UPGRADE_ROADMAP.md) Phase 3. |
| Orphaned modules in `trustforge_bond` | **~60 files (~18,500 lines), including 8 real feature modules (verifier staking, governance-voted slashing, evidence storage, access control, cooldowns, fees, status snapshots) and ~53 test files, are not compiled — never declared as `mod` in `lib.rs`.** See [docs/ORPHANED_MODULES.md](docs/ORPHANED_MODULES.md) for the full finding and why it isn't a quick fix. |
| Unsafe code | Zero in the compiled surface (verified by manual grep; `cargo-geiger` itself wasn't run — see [VERIFICATION.md](VERIFICATION.md) §8). The only 2 `unsafe` blocks in the repo are in orphaned, uncompiled test files. |

## What "internal review" means today

- Self-review by contract authors, not an independent third party.
- `cargo build`, `cargo test`, `cargo clippy`, and `cargo llvm-cov` were run and independently
  verified as part of Phase 2 on 2026-08-10 — see [VERIFICATION.md](VERIFICATION.md) for the full
  log, including what was found broken (and fixed) and what's still outstanding. The CI badges
  above reflect the same commit going forward.

## Where this is going

See [QUALITY_UPGRADE_ROADMAP.md](QUALITY_UPGRADE_ROADMAP.md) for the phased plan to close these
gaps, and [SECURITY_AUDIT.md](SECURITY_AUDIT.md) for the detailed internal security assessment.
