# Status

Single source of truth for TrustForge's current audit/deployment state. Other docs
(`README.md`, `SECURITY_AUDIT.md`) should point here rather than restating these facts
independently, so they can't drift out of sync.

_Last updated: 2026-08-07._

| Question | Answer |
|---|---|
| Third-party audited? | **No.** Internal review only. See [SECURITY_AUDIT.md](SECURITY_AUDIT.md). |
| Deployed to testnet? | **No.** See [README.md § Testnet](README.md#testnet) for the deployment layout you get after following [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md). |
| Deployed to mainnet? | **No.** Blocked on a completed third-party audit and governance approval. |
| CI passing? | See live status: [![CI](https://github.com/Softvaults/trustforge-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/ci.yml) [![Contracts Tests](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-tests.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-tests.yml) [![Contracts Lints](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-lints.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-lints.yml) [![Coverage](https://github.com/Softvaults/trustforge-contracts/actions/workflows/coverage.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/coverage.yml) [![Security Scanning](https://github.com/Softvaults/trustforge-contracts/actions/workflows/security.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/security.yml) |
| Test coverage (measured, not asserted) | Not yet independently verified in this repo's current audit cycle — pending `cargo tarpaulin --workspace` run (see [QUALITY_UPGRADE_ROADMAP.md](QUALITY_UPGRADE_ROADMAP.md) Phase 2). |
| `unwrap()`/`expect()`/`panic!()` in non-test contract code | ~950, concentrated in `trustforge_bond` (240). Tracked in [QUALITY_UPGRADE_ROADMAP.md](QUALITY_UPGRADE_ROADMAP.md) Phase 3. |

## What "internal review" means today

- Self-review by contract authors, not an independent third party.
- No `cargo build`/`cargo test`/`cargo clippy`/`cargo tarpaulin` run has been independently
  re-verified as part of this status update — the CI badges above are the current source of
  truth for build/test health, not this document.

## Where this is going

See [QUALITY_UPGRADE_ROADMAP.md](QUALITY_UPGRADE_ROADMAP.md) for the phased plan to close these
gaps, and [SECURITY_AUDIT.md](SECURITY_AUDIT.md) for the detailed internal security assessment.
