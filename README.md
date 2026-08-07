# TrustForge Contracts

**Production-grade Soroban smart contracts for the TrustForge economic trust protocol.**

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/Softvaults/trustforge-contracts/releases)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Security](https://img.shields.io/badge/security-audited-success.svg)](SECURITY_AUDIT.md)

## About

TrustForge is a decentralized identity bond and reputation system built on Stellar's Soroban platform. Users stake tokens as collateral to establish trustworthiness, with attestations, delegations, and dispute resolution mechanisms.

### Key Features

- ✅ **Identity Bonds**: Fixed-duration and rolling bonds with tiered reputation
- ✅ **Slashing**: Governance-controlled penalties for misconduct
- ✅ **Attestations**: Weighted trust signals from verified attesters
- ✅ **Delegation**: Granular rights management with replay protection
- ✅ **Treasury**: Multi-sig fee collection and withdrawal management
- ✅ **Arbitration**: Weighted-vote dispute resolution
- ✅ **Emergency Controls**: Circuit breakers and dual-auth recovery

### Production Status

**v1.0.0** - Production Ready ✅

- Comprehensive internal security review completed
- Full test coverage (unit, integration, fuzzing, property-based)
- Production documentation and deployment guides
- Monitoring and observability framework
- Scalability roadmap established

See [SECURITY_AUDIT.md](SECURITY_AUDIT.md) for detailed security assessment.

## Quick Start

### Prerequisites

- **Rust 1.85.1+** (pinned in [`rust-toolchain.toml`](rust-toolchain.toml))
- **Soroban CLI** ([installation guide](https://developers.stellar.org/docs/smart-contracts/getting-started/setup))
- **Stellar Account** with testnet XLM ([Friendbot](https://friendbot.stellar.org))

### Build

```bash
# Clone repository
git clone https://github.com/Softvaults/trustforge-contracts.git
cd trustforge-contracts

# Build all contracts
cargo build --workspace

# Build WASM for deployment
cargo build --target wasm32-unknown-unknown --release --locked \
  -p trustforge_bond \
  -p trustforge_delegation \
  -p trustforge_registry \
  -p trustforge_treasury \
  -p trustforge_arbitration \
  -p trustforge_admin \
  -p trustforge_multisig \
  -p timelock
```

### Test

```bash
# Run all tests
cargo test --workspace

# Run tests for specific contract
cargo test -p trustforge_bond

# Run with coverage
cargo tarpaulin --workspace --out Html
```

### Deploy to Testnet

```bash
# Configure Soroban CLI for testnet
soroban network add testnet \
  --rpc-url https://soroban-rpc.testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Deploy bond contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/trustforge_bond.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

**For complete deployment instructions**, see [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

## Documentation

### Getting Started
- [API Reference for Integrators](docs/API_REFERENCE.md) - Complete API documentation
- [Architecture Overview](docs/architecture.md) - System design and contract interactions
- [Bond State Transitions](docs/bond-state-transitions.md) - Bond lifecycle diagrams

### Deployment
- [Testnet Deployment Guide](docs/DEPLOYMENT.md) - Step-by-step testnet deployment
- [Mainnet Deployment Guide](docs/MAINNET_DEPLOYMENT.md) - Production deployment checklist
- [Upgrade Strategy](docs/UPGRADE_STRATEGY.md) - Upgrade procedures and governance

### Operations
- [Monitoring & Observability](docs/MONITORING.md) - Production monitoring setup
- [Scalability Roadmap](docs/SCALABILITY.md) - Performance targets and optimization plans
- [Security Audit Report](SECURITY_AUDIT.md) - Security assessment and findings

### Features
- [Bond Management](docs/trustforge-bond.md) - Creating, topping up, withdrawing bonds
- [Tier System](docs/tier-system.md) - Bronze, Silver, Gold, Platinum tiers
- [Slashing Mechanism](docs/slashing.md) - Penalties and governance approval
- [Rolling Bonds](docs/rolling-bonds.md) - Auto-renewing bonds with notice periods
- [Early Exit](docs/early-exit.md) - Pre-lockup withdrawal with penalties
- [Attestations](docs/attestations.md) - Trust signals and reputation
- [Delegation](docs/delegation.md) - Rights management and relayed execution
- [Emergency Procedures](docs/emergency.md) - Circuit breakers and recovery

### Development
- [Testing Guide](docs/testing.md) - Test patterns and best practices
- [Known Limitations](docs/known-simplifications.md) - Current constraints and workarounds
- [WASM Reproducibility](docs/wasm-reproducibility.md) - Deterministic builds

## Project Structure

```
trustforge-contracts/
├── contracts/
│   ├── trustforge_bond/          # Core identity bond contract
│   ├── trustforge_registry/      # Identity→bond address mapping
│   ├── trustforge_delegation/    # Attestation delegation
│   ├── trustforge_treasury/      # Fee accounting and withdrawals
│   ├── trustforge_arbitration/   # Dispute resolution
│   ├── trustforge_admin/         # Role management
│   ├── trustforge_multisig/      # Multi-signature proposals
│   ├── timelock/                 # Time-delayed operations
│   ├── trustforge_errors/        # Shared error types
│   └── trustforge_math/          # Arithmetic utilities
├── docs/                         # Comprehensive documentation
├── crates/
│   ├── trustforge_admin_cli/     # Operator CLI tool
│   └── testutils/                # Shared test utilities
└── .github/workflows/            # CI/CD pipelines
```

## Testing

### Test Coverage

- **Unit tests**: Contract logic and edge cases
- **Integration tests**: Cross-contract interactions
- **Property-based tests**: Invariant validation with proptest
- **Fuzz tests**: Random input testing
- **Gas benchmarks**: Performance regression detection

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test -p trustforge_bond test_create_bond

# Run property-based tests
cargo test -p trustforge_bond proptest

# Run benchmarks (requires nightly + gas-bench feature)
cargo bench -p trustforge_bond --features gas-bench
```

### Continuous Integration

Every PR runs:
- ✅ Unit and integration tests
- ✅ Clippy linting (`-D warnings`)
- ✅ Format checking (`cargo fmt --check`)
- ✅ Security audit (`cargo audit`)
- ✅ WASM size budget enforcement
- ✅ Unsafe code detection (`cargo geiger`)
- ✅ Test coverage reporting

See [.github/workflows/](.github/workflows/) for CI configuration.

## Contract Addresses

### Testnet

```
Network: Stellar Testnet
RPC: https://soroban-rpc.testnet.stellar.org
Passphrase: Test SDF Network ; September 2015

trustforge_bond:        <DEPLOY_YOUR_OWN>
trustforge_registry:    <DEPLOY_YOUR_OWN>
trustforge_treasury:    <DEPLOY_YOUR_OWN>
trustforge_delegation:  <DEPLOY_YOUR_OWN>
trustforge_arbitration: <DEPLOY_YOUR_OWN>
trustforge_admin:       <DEPLOY_YOUR_OWN>
trustforge_multisig:    <DEPLOY_YOUR_OWN>
timelock:               <DEPLOY_YOUR_OWN>

See docs/DEPLOYMENT.md for deployment instructions.
```

### Mainnet

**Status**: Ready for deployment (pending governance approval)

For mainnet deployment, see [docs/MAINNET_DEPLOYMENT.md](docs/MAINNET_DEPLOYMENT.md).

## Performance

### Current Throughput (v1.0.0)

| Operation | Gas Cost | Latency (p95) | Throughput |
|-----------|----------|---------------|------------|
| Create bond | ~800k | 3-5s | 10 TPS |
| Top-up | ~400k | 2-3s | 20 TPS |
| Withdraw | ~500k | 2-4s | 15 TPS |
| Attestation | ~300k | 1-2s | 30 TPS |

See [docs/SCALABILITY.md](docs/SCALABILITY.md) for optimization roadmap.

## Security

### Reporting Vulnerabilities

**DO NOT** open public issues for security vulnerabilities.

Report security issues via:
- **GitHub Security Advisories**: [Create Advisory](https://github.com/Softvaults/trustforge-contracts/security/advisories/new)
- **Email**: security@trustforge.io (coming soon)

For detailed vulnerability disclosure process, see [SECURITY.md](SECURITY.md).

### Security Features

- ✅ Admin-only functions properly gated
- ✅ Checks-Effects-Interactions pattern enforced
- ✅ Overflow-safe arithmetic throughout
- ✅ Replay protection with nonces
- ✅ Storage key stability (fingerprint tests)
- ✅ Multi-sig pause mechanism
- ✅ Emergency recovery procedures
- ✅ Comprehensive event logging

## Contributing

We welcome contributions! Please read our contributing guidelines (coming soon) before submitting PRs.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test --workspace`)
5. Run lints (`cargo fmt && cargo clippy`)
6. Commit your changes (`git commit -m 'feat: add amazing feature'`)
7. Push to your branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Code Standards

- Follow Rust standard style (`rustfmt`)
- No clippy warnings (`-D warnings`)
- All public functions documented
- Tests for new features
- No unsafe code (verified by `cargo geiger`)

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

**WASM Size Budget**: Each contract must stay under 64KB (enforced in CI).

See [docs/wasm-size-budget.md](docs/wasm-size-budget.md) for details.

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

- **Documentation**: [Full Documentation](docs/)
- **API Reference**: [Integration Guide](docs/API_REFERENCE.md)
- **GitHub Issues**: [Report Bugs](https://github.com/Softvaults/trustforge-contracts/issues)
- **GitHub Discussions**: [Ask Questions](https://github.com/Softvaults/trustforge-contracts/discussions)
- **Security**: security@trustforge.io (coming soon)

---

**Built with ❤️ by the TrustForge Team**

## Prerequisites

- Rust 1.85.1+ (pinned in [`rust-toolchain.toml`](rust-toolchain.toml)); the WASM target is included
- [Soroban CLI](https://developers.stellar.org/docs/smart-contracts/getting-started/setup) (`cargo install soroban-cli`)

## Setup

From the repo root:

```bash
cargo build
```

For Soroban (WASM) build:

```bash
cargo build --target wasm32-unknown-unknown --release --locked -p trustforge_bond -p trustforge_delegation
```

For the reproducibility check and the CI hash comparison, see [docs/wasm-reproducibility.md](docs/wasm-reproducibility.md).

## Tests

Run all workspace tests:

```bash
cargo test --workspace
```

Run specific contract tests:

```bash
cargo test -p trustforge_bond
cargo test -p trustforge_delegation
```

The dedicated CI workflow at `.github/workflows/contracts-tests.yml` runs the full workspace tests on every PR.

## Linting

Run the contracts-only formatting and lint checks locally before opening a PR:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The dedicated CI workflow at `.github/workflows/contracts-lints.yml` runs the same checks.

## Security scanning

Pull requests run `cargo audit --deny warnings`; dependency vulnerabilities are surfaced in a sticky PR comment and the full JSON report is uploaded as a workflow artifact. See [docs/SECURITY_SCANNING.md](docs/SECURITY_SCANNING.md) for the local command and triage flow.

## Release profile — WASM size

The workspace release profile is tuned to minimize WASM binary size:

```toml
[profile.release]
opt-level = "z"       # Optimize for size
lto = "fat"           # Full link-time optimisation across all crates
codegen-units = 1     # Single codegen unit for maximum inlining
strip = "symbols"     # Strip debug symbols
panic = "abort"       # Omit panic unwind machinery
```

- `opt-level = "z"` — instructs `rustc` to optimise for size rather than speed.
- `lto = "fat"` — enables full cross-crate LTO so the linker can eliminate dead code and inline across crate boundaries.
- `codegen-units = 1` — prevents the compiler from splitting a crate into multiple compilation units, giving the optimiser a whole-crate view.
- `strip = "symbols"` — removes the symbol table from the final `.wasm`.
- `panic = "abort"` — replaces panic unwind landing pads with an immediate `wasm32::unreachable`, saving hundreds of bytes per panic site.

These settings apply workspace-wide. Individual contracts can override them in their own `Cargo.toml` if needed.

## WASM size budget

Release Wasm for every deployable contract must stay within per-contract size ceilings enforced in CI. See [docs/wasm-size-budget.md](docs/wasm-size-budget.md) for the enforced limits and [`.github/workflows/wasm-size.yml`](.github/workflows/wasm-size.yml) for the gate.

## Project layout

- `contracts/trustforge_bond/` — Identity bond contract
  - `create_bond()` / `top_up()` / `withdraw()` / `withdraw_early()`
  - Rolling bonds: `request_withdrawal()` and `renew_if_rolling()`
  - Tiering: `get_tier()` with auto-upgrade/downgrade events
  - Slashing: `slash()` with available-balance enforcement
  - Emergency: `set_emergency_config()`, `set_emergency_mode()`, `emergency_withdraw()`
  - Emergency audit: `get_latest_emergency_record_id()`, `get_emergency_record()`
  - Lifecycle: [bond state transitions](docs/bond-state-transitions.md)
- `contracts/trustforge_delegation/` — Delegation contract
- `docs/` — Feature docs (`EVENTS.md`, `rolling-bonds.md`, `early-exit.md`, `slashing.md`, `tier-system.md`, `delegation.md`, `emergency.md`, `UPGRADE.md`)

**Known simplifications:** See [docs/known-simplifications.md](docs/known-simplifications.md) for a complete list of intentional limitations and production paths.

## Deploy (Soroban CLI)

Configure network and deploy:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/trustforge_bond.wasm \
  --source <SECRET_KEY> \
  --network <NETWORK>
```

See [Stellar Soroban docs](https://developers.stellar.org/docs/smart-contracts) for auth and network setup.

For the full testnet deploy and cross-contract wiring runbook, see [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
---

**Version 1.0.0** | [Changelog](CHANGELOG.md) | [Security](SECURITY_AUDIT.md) | [API Docs](docs/API_REFERENCE.md)
