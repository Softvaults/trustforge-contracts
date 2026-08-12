# TrustForge Contracts

**Soroban smart contracts for the TrustForge economic trust protocol — pre-audit, not yet deployed anywhere.**

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Security](https://img.shields.io/badge/security-internal--review--only-yellow.svg)](SECURITY_AUDIT.md)
[![CI](https://github.com/Softvaults/trustforge-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/ci.yml)
[![Contracts Tests](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-tests.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/contracts-tests.yml)
[![Coverage](https://github.com/Softvaults/trustforge-contracts/actions/workflows/coverage.yml/badge.svg)](https://github.com/Softvaults/trustforge-contracts/actions/workflows/coverage.yml)

## What TrustForge is

TrustForge is a decentralized identity-bond and reputation protocol on Stellar's Soroban
platform. The core mechanic: an identity locks ("bonds") tokens as collateral, and that stake
backs their trustworthiness on-chain. Around that core sit five supporting systems:

- **Attestations** — registered attesters issue weighted trust signals against a subject's
  bond, accumulated as reputation.
- **Tiers** — an identity's bonded amount (net of any slashing) maps to a Bronze/Silver/Gold/
  Platinum tier, read by any integrator that wants a coarse trust signal without interpreting
  raw attestation data.
- **Slashing** — the protocol admin can penalize a bond for misconduct. This is **admin-controlled
  today, not a governance vote** — see [docs/known-simplifications.md](docs/known-simplifications.md)
  for why, and [docs/trustforge-bond.md](docs/trustforge-bond.md) for what was tried and removed.
- **Delegation** — a separate contract lets an identity grant another address the right to act
  on its behalf (e.g. relay an attestation), with nonce- and domain-separated replay protection.
- **Arbitration** — disputes are resolved by weighted vote, where an arbitrator's voting weight
  is derived live from their own bonded stake in a `trustforge_bond` instance (not admin-assigned
  — see [docs/arbitration.md](docs/arbitration.md)).

Governance-style controls (multi-signature approval, time-delayed execution) and operational
plumbing (fee treasury, cross-contract identity discovery) exist as their own contracts so each
has a narrow, auditable responsibility — see [Monorepo Layout](#monorepo-layout) below for the
full map, and [docs/architecture.md](docs/architecture.md) for how they call into each other.

## Current status

**Not audited. Not deployed anywhere — no testnet instance, no mainnet instance.**
[STATUS.md](STATUS.md) is the single source of truth for audit/deployment/CI/coverage state and
is updated whenever any of that changes; this README doesn't restate those numbers so they can't
drift out of sync with it. In short, as of this writing:

- Internal review has been done (including an AI-conducted line-by-line pass over part of
  `trustforge_bond` that found and fixed a critical fund-custody bug — see
  [docs/BOND_REVIEW_NOTE.md](docs/BOND_REVIEW_NOTE.md)); no independent third-party audit has
  happened yet. See [SECURITY_AUDIT.md](SECURITY_AUDIT.md).
- `trustforge_bond` is not currently deployable to any network: its release WASM exceeds
  Soroban's ~64KB size limit even with maximal size optimization. The other 7 deployable
  contracts are within budget.
- Mainnet deployment is blocked on a completed third-party audit and governance approval —
  see [docs/AUDIT_READINESS.md](docs/AUDIT_READINESS.md) for the scoping package prepared for
  whichever firm gets engaged.

## Monorepo Layout

This is a single Cargo workspace containing **13 crates** — 11 under `contracts/`, 2 under
`crates/`. They're related but independently deployable/publishable; there is no separate
repository this one depends on or is split from.

### Deployable contracts (8, each with its own release-WASM size budget)

| Crate | ~LOC | Role |
|---|---:|---|
| `contracts/trustforge_bond` | 15,900 | Core value-custody contract: bond lifecycle, attestations, slashing, tiers. The largest and highest-risk contract — see [docs/trustforge-bond.md](docs/trustforge-bond.md) and [docs/BOND_REVIEW_NOTE.md](docs/BOND_REVIEW_NOTE.md). |
| `contracts/trustforge_delegation` | 8,200 | Delegated attestation rights with nonce/domain-separated replay protection. See [docs/delegation.md](docs/delegation.md). |
| `contracts/trustforge_admin` | 4,200 | System-wide role hierarchy (SuperAdmin/Admin/Operator). See [docs/admin-roles.md](docs/admin-roles.md). |
| `contracts/trustforge_treasury` | 3,900 | Fee accounting, withdrawal guardrails, flash-loan fee enforcement. |
| `contracts/trustforge_arbitration` | 2,600 | Dispute resolution; voting weight derived live from bonded stake. See [docs/arbitration.md](docs/arbitration.md). |
| `contracts/trustforge_multisig` | 2,300 | Multi-signature proposal/execution with proposal expiry. See [docs/multisig.md](docs/multisig.md). |
| `contracts/timelock` | 540 | Time-delayed execution for governance-style actions. |
| `contracts/templates` | 490 | **Not a product contract** — a canonical, documented scaffold for starting a new contract in this workspace (patterns: `DataKey` storage, admin-gated init, `require_auth`, event emission). Copy and rename it rather than starting from scratch. |

### Deployable, but not yet size-budgeted (a real gap, not an oversight to copy)

| Crate | ~LOC | Role |
|---|---:|---|
| `contracts/trustforge_registry` | 1,100 | Identity → bond-contract-address discovery, with trustless self-registration and paginated reads. Has a `#[contract]` entrypoint like the eight above, but is currently **missing from `scripts/wasm-size-budget.toml`**, so its WASM size isn't CI-checked the way the others are. See [docs/registry.md](docs/registry.md). |

### Shared libraries (not independently deployable — no `#[contract]`)

| Crate | ~LOC | Role |
|---|---:|---|
| `contracts/trustforge_errors` | 3,000 | Canonical, wire-stable `ContractError` enum shared by every contract above — numeric error codes are permanent once shipped. See [docs/errors.md](docs/errors.md). |
| `contracts/trustforge_math` | 520 | Shared checked-arithmetic helpers for financial calculations. |

### Tooling

| Crate | ~LOC | Role |
|---|---:|---|
| `crates/trustforge_admin_cli` | 580 | `trustforge-admin` — the operator's command-line interface for administrative actions against deployed contracts, built on `soroban-client`/`stellar-baselib`. See [docs/admin-cli.md](docs/admin-cli.md). This crate's host-side dependencies are why the workspace's pinned Rust toolchain is newer than the contracts themselves strictly need — see the comment in [`rust-toolchain.toml`](rust-toolchain.toml). |
| `crates/testutils` | — | Shared test-only helpers (e.g. canonical `admin`/`attacker`/`user` test addresses) used across multiple contracts' test suites. |

## Quick Start

### Prerequisites

| Requirement | Version / source |
|---|---|
| Rust toolchain | **`1.89.0`**, pinned in [`rust-toolchain.toml`](rust-toolchain.toml) (raised from an earlier 1.85.1 pin — see that file's comment for why) |
| Target | `wasm32-unknown-unknown` (installed automatically by `rustup` from the toolchain file) |
| Components | `rustfmt`, `clippy`, `llvm-tools-preview` |
| Soroban CLI | [installation guide](https://developers.stellar.org/docs/smart-contracts/getting-started/setup) — needed for deployment, not for building/testing |

### Build

```bash
git clone https://github.com/Softvaults/trustforge-contracts.git
cd trustforge-contracts

# Build the whole workspace (all 13 crates, dev profile)
cargo build --workspace

# Build optimized release WASM for the 8 size-budgeted deployable contracts
# (trustforge_registry is deployable too, but excluded here pending its
# wasm-size-budget.toml entry -- see "Monorepo Layout" above)
cargo build \
  --target wasm32-unknown-unknown \
  --release \
  --locked \
  -p trustforge_bond \
  -p trustforge_delegation \
  -p trustforge_treasury \
  -p trustforge_arbitration \
  -p trustforge_admin \
  -p trustforge_multisig \
  -p timelock \
  -p templates

# Verify WASM sizes are within budget (see the CI section below --
# this script exists and works, but nothing currently runs it in CI)
bash scripts/check_wasm_size.sh
```

### Test

```bash
# Run all workspace tests
cargo test --workspace

# Run tests for one crate
cargo test -p trustforge_bond

# Measure real coverage (CI's actual tool -- not cargo-tarpaulin, which
# doesn't install cleanly against the pinned toolchain in this workspace)
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo llvm-cov --package trustforge_bond --fail-under-lines 95
```

### Deploy to testnet

```bash
soroban network add testnet \
  --rpc-url https://soroban-rpc.testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/trustforge_bond.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

**For complete deployment instructions** (including cross-contract wiring), see
[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

## Documentation

### Start here
- [STATUS.md](STATUS.md) — audit/deployment/CI/coverage status at a glance, kept current
- [Architecture Overview](docs/architecture.md) — per-crate module breakdown and how contracts call each other
- [API Reference for Integrators](docs/API_REFERENCE.md)
- [Bond State Transitions](docs/bond-state-transitions.md)

### Security & review
- [SECURITY_AUDIT.md](SECURITY_AUDIT.md) — security assessment, known issues, resolved issues
- [docs/AUDIT_READINESS.md](docs/AUDIT_READINESS.md) — scoping package for the pending third-party audit
- [docs/BOND_REVIEW_NOTE.md](docs/BOND_REVIEW_NOTE.md) — dated internal-coherence review of `trustforge_bond`, including a critical finding that was found and fixed
- [docs/ORPHANED_MODULES.md](docs/ORPHANED_MODULES.md) — ~70 files of dead code found and deleted from `trustforge_bond`, and why restoring instead of deleting was rejected
- [docs/SECURITY_CONTACT_PLAN.md](docs/SECURITY_CONTACT_PLAN.md) — what exists today (GitHub Security Advisories) vs. what a dedicated contact channel would still need
- [SECURITY.md](SECURITY.md) — how to report a vulnerability
- [THREATS.md](THREATS.md) — threat registry (read its accuracy warning at the top before trusting any row's "✅ Covered")

### Deployment & operations
- [Testnet Deployment Guide](docs/DEPLOYMENT.md)
- [Mainnet Deployment Guide](docs/MAINNET_DEPLOYMENT.md)
- [Upgrade Strategy](docs/UPGRADE_STRATEGY.md)
- [Monitoring & Observability](docs/MONITORING.md)
- [Scalability Roadmap](docs/SCALABILITY.md)
- [Admin CLI](docs/admin-cli.md)

### Protocol features
- [Bond Management](docs/trustforge-bond.md) — creating, topping up, withdrawing bonds
- [Tier System](docs/tier-system.md)
- [Slashing Mechanism](docs/slashing.md)
- [Rolling Bonds](docs/rolling-bonds.md)
- [Early Exit](docs/early-exit.md)
- [Attestations](docs/attestations.md)
- [Delegation](docs/delegation.md)
- [Arbitration](docs/arbitration.md)
- [Multisig](docs/multisig.md)
- [Registry](docs/registry.md)
- [Emergency Procedures](docs/emergency.md)

### Process & provenance
- [QUALITY_UPGRADE_ROADMAP.md](QUALITY_UPGRADE_ROADMAP.md) — the working plan for closing the gaps this README used to gloss over, phase by phase, with what's actually done vs. still open
- [CONTRIBUTING.md](CONTRIBUTING.md) — workflow, CI gates, review tiers (doc/typo PRs vs. contract-logic PRs)
- [.github/CODEOWNERS](.github/CODEOWNERS) — who must review contract-logic-tier changes
- [docs/HISTORY.md](docs/HISTORY.md) — factual record of this repository's contribution history
- [docs/known-simplifications.md](docs/known-simplifications.md) — current intentional design tradeoffs vs. resolved former limitations

### Development
- [Testing Guide](docs/testing.md)
- [WASM Reproducibility](docs/wasm-reproducibility.md)
- [WASM Size Budget](docs/wasm-size-budget.md)

## Testing

- **Unit tests**: per-crate contract logic and edge cases, in each crate's `#[cfg(test)]` modules
- **Integration tests**: cross-contract interactions, in each crate's `tests/` directory
- **Property-based tests**: invariant validation via `proptest` (e.g. `trustforge_bond`'s bond-tier fuzzer)
- **Fuzz tests**: randomized input sequences against core accounting invariants
- **Gas/storage-cost regression**: `trustforge_bond`'s `tests/test_cost_regression.rs` fails the build if any tracked entrypoint's measured cost moves more than 5% from the committed baseline

```bash
cargo test --workspace
cargo test --workspace -- --nocapture
cargo test -p trustforge_bond test_create_bond
cargo test -p trustforge_bond fuzz::test_bond_fuzz -- --nocapture
cargo bench -p trustforge_bond --features gas-bench   # requires the gas-bench feature
```

## Continuous Integration

Every PR runs, via [.github/workflows/](.github/workflows/):

- ✅ Unit and integration tests (`contracts-tests.yml`, `ci.yml`)
- ✅ Clippy linting, `-D warnings` (`contracts-lints.yml`)
- ✅ Format checking, `cargo fmt --check` (`contracts-lints.yml`)
- ✅ Dependency vulnerability scan, `cargo audit` (`security.yml`)
- ✅ Unsafe-code detection, `cargo geiger` (`security.yml`)
- ✅ Coverage measurement and reporting, `cargo llvm-cov` (`coverage.yml`)

**Not currently true, despite the tooling existing:** `scripts/check_wasm_size.sh` and
`scripts/wasm-size-budget.toml` implement real per-contract WASM size enforcement, and
`docs/wasm-size-budget.md` describes it as CI-enforced — but no workflow in
`.github/workflows/` actually invokes the script. `trustforge_bond` is currently 131KB against
its 64KB budget and this would not be caught by CI today. Run the script locally
(`bash scripts/check_wasm_size.sh`) until this is wired in.

## Contract Addresses

### Testnet

**Status: not currently deployed.** No instance of these contracts is live on testnet right now
— the table below is the deployment layout you get *after* running through
[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md), not a set of existing addresses.

```
Network: Stellar Testnet
RPC: https://soroban-rpc.testnet.stellar.org
Passphrase: Test SDF Network ; September 2015

trustforge_bond:        (not deployed — run docs/DEPLOYMENT.md, then fill in)
trustforge_registry:    (not deployed — run docs/DEPLOYMENT.md, then fill in)
trustforge_treasury:    (not deployed — run docs/DEPLOYMENT.md, then fill in)
trustforge_delegation:  (not deployed — run docs/DEPLOYMENT.md, then fill in)
trustforge_arbitration: (not deployed — run docs/DEPLOYMENT.md, then fill in)
trustforge_admin:       (not deployed — run docs/DEPLOYMENT.md, then fill in)
trustforge_multisig:    (not deployed — run docs/DEPLOYMENT.md, then fill in)
timelock:               (not deployed — run docs/DEPLOYMENT.md, then fill in)
```

### Mainnet

**Status**: Not deployed. Blocked on a completed third-party audit (see
[SECURITY_AUDIT.md](SECURITY_AUDIT.md)) and governance approval — do not treat "pending
governance approval" as the only remaining step.

For the mainnet deployment procedure (once unblocked), see
[docs/MAINNET_DEPLOYMENT.md](docs/MAINNET_DEPLOYMENT.md).

## Performance

There is no independently-verified throughput/latency benchmark for this protocol, and an
earlier version of this README asserted specific TPS/gas figures that could not be traced to
any committed measurement — they've been removed rather than left to mislead. What's real and
reproducible instead:

- `contracts/trustforge_bond/cost_baseline.json` — committed, per-entrypoint CPU/memory/storage
  cost measurements (`create_bond`, `top_up`, `withdraw`, `withdraw_early`, `slash_bond`,
  `add_attestation`), regenerated via `cargo run -p trustforge_bond --features gas-bench --bin
  update-cost-baseline` whenever a change intentionally moves the cost, and gated by
  `tests/test_cost_regression.rs` (5% tolerance) on every run.
- [docs/SCALABILITY.md](docs/SCALABILITY.md) — known architectural bottlenecks (e.g. the
  single-bond-per-contract-instance model's per-identity deployment cost) and mitigation/future
  options, not throughput claims.

## Security

### Reporting vulnerabilities

**Do not** open public issues for security vulnerabilities. Report via
[GitHub Security Advisories](https://github.com/Softvaults/trustforge-contracts/security/advisories/new)
— this is a real, working channel, not a placeholder. See [SECURITY.md](SECURITY.md) for the
full disclosure process and [docs/SECURITY_CONTACT_PLAN.md](docs/SECURITY_CONTACT_PLAN.md) for
what a dedicated monitored inbox would additionally require if the project adds one later.

### Security posture, honestly

- ✅ Checks-effects-interactions pattern used on fund-moving paths, with reentrancy guards on
  every entrypoint that makes an external token call
- ✅ Overflow-checked arithmetic on financial calculations (`checked_add`/`checked_sub` or a
  typed error, not silent wraparound)
- ✅ Replay protection via nonces (attestations) and domain separation (delegation)
- ✅ Storage-key stability enforced by fingerprint tests, so a renamed/reshaped `DataKey`
  variant fails CI instead of silently orphaning ledger state
- ✅ Zero `unwrap()`/`expect()`/`panic!()` in the compiled non-test surface of `trustforge_bond`,
  `trustforge_errors`, `trustforge_registry`, and `templates`, clippy-enforced so it can't
  regress
- ⚠️ Multi-sig pause exists per-contract, not uniformly — e.g. `trustforge_bond`'s
  threshold pause-signer flow is implemented but has zero live callers today (see
  `docs/BOND_REVIEW_NOTE.md`)
- ❌ No independent third-party audit yet
- ❌ `trustforge_bond`'s threat registry (`THREATS.md`) has substantially stale test-fixture
  references — read its warning banner before trusting its coverage claims

See [SECURITY_AUDIT.md](SECURITY_AUDIT.md) for the full, itemized assessment including
currently-open and resolved findings.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development workflow, every CI gate and its
local-equivalent command, and code conventions. In short:

1. Fork the repository and create a branch (`<type>/<short-description>`, e.g. `feat/slash-bond-core`)
2. Make your changes, following [CONTRIBUTING.md § Review Tiers](CONTRIBUTING.md#review-tiers) —
   doc/typo PRs stay easy to merge; anything touching `contracts/**/src/**` needs a design
   discussion first, not just a passing CI run
3. Run the local CI-gate commands (`cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace`)
4. Open a PR against `main` using conventional-commit-style titles (`feat:`, `fix:`, `docs:`, ...)
5. Contract-logic-tier changes require review from a [`.github/CODEOWNERS`](.github/CODEOWNERS)-listed maintainer

## Release Profile

The workspace is optimized for minimal WASM size:

```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = "fat"           # Full link-time optimization
codegen-units = 1     # Single codegen unit
strip = "symbols"     # Strip debug symbols
panic = "abort"       # Omit panic unwind
```

**WASM size budget**: each size-budgeted contract must stay under 64KB per
[`scripts/wasm-size-budget.toml`](scripts/wasm-size-budget.toml) — see
[docs/wasm-size-budget.md](docs/wasm-size-budget.md) for the full budget table and how to check
it locally. As noted in [Continuous Integration](#continuous-integration) above, this is not yet
wired into any CI workflow, and `trustforge_bond` is currently over budget.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history and migration guides.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

```
Copyright 2026 TrustForge Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## Support

- **Documentation**: [docs/](docs/)
- **API Reference**: [docs/API_REFERENCE.md](docs/API_REFERENCE.md)
- **Bugs / features**: [GitHub Issues](https://github.com/Softvaults/trustforge-contracts/issues)
- **Questions**: [GitHub Discussions](https://github.com/Softvaults/trustforge-contracts/discussions)
- **Security vulnerabilities**: [GitHub Security Advisories](https://github.com/Softvaults/trustforge-contracts/security/advisories/new) — see [Security](#security) above, do not use public issues

---

**Version 1.0.0** | [Changelog](CHANGELOG.md) | [Status](STATUS.md) | [Security](SECURITY_AUDIT.md) | [API Docs](docs/API_REFERENCE.md)
