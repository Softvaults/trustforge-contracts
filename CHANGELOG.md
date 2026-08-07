# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-15

### Production Release 🎉

TrustForge v1.0.0 marks the first production-ready release of the protocol. All contracts have undergone comprehensive internal security review and are ready for mainnet deployment.

### Added

**Documentation & Production Readiness**
- Added comprehensive security audit documentation (SECURITY_AUDIT.md)
- Added mainnet deployment guide with pre-deployment checklist (docs/MAINNET_DEPLOYMENT.md)
- Added upgrade strategy with governance workflow (docs/UPGRADE_STRATEGY.md)
- Added production monitoring and observability guide (docs/MONITORING.md)
- Added scalability roadmap with three-phase optimization plan (docs/SCALABILITY.md)

**Features**
- **Timelock Timeout Test**: Added explicit timeout regression coverage for time-locked operation execution after the grace period (`timelock`).
- **Pause Signer Invariant**: Added invariant test for PauseSignerCount to prevent drift (`trustforge_delegation`).
- **Slash Bond Core**: Implemented admin-only `slash_bond` functionality with partial/full slashing and event emission.
- **Treasury Guardrails**: Added comprehensive tests and functionality for liquidity floor and slippage protection mechanisms in treasury withdrawals (`trustforge_treasury`).
- **Batch Bond Atomicity**: Enhanced batch operations with explicit empty batch handling and `MAX_BATCH_BOND_SIZE` enforcement (`trustforge_bond`).

### Changed

- **Version Bump**: All workspace packages upgraded from 0.1.0 to 1.0.0, marking production readiness
- **TrustForge rebrand**: Renamed the project from Credence to TrustForge across the entire workspace. Crate names (`credence_*` → `trustforge_*`), public contract types (`CredenceBond` → `TrustForgeBond`, `CredenceRegistry` → `TrustForgeRegistry`, `CredenceTreasury` → `TrustForgeTreasury`, `CredenceDelegation` → `TrustForgeDelegation`, `CredenceArbitration` → `TrustForgeArbitration`, `CredenceMultiSig` → `TrustForgeMultiSig`, and their generated `*Client` types), environment variables (`CREDENCE_*` → `TRUSTFORGE_*` in the admin CLI), and ~970 prose references across docs, comments, and CI text. Renamed `docs/credence-*.md` files to their `trustforge_*` equivalents and fixed all cross-references and anchor links.
- **SafeERC20 Migration**: Replaced direct `TokenClient` calls with safe wrapper functions to support non-compliant ERC20 tokens across the protocol.
- **Protocol Fixes**: Resolved compilation errors, completed `top_up` and `extend_duration` with overflow protection.
- **Event Indexing**: Migrated lifecycle events to V2 for optimized off-chain indexing.

### Fixed

- **Workspace build**: Resolved pre-existing merge corruption that was blocking the workspace build, as a prerequisite to the rebrand above.
- **CI**: Fixed the `cargo-geiger` unsafe-code scan glob in `security.yml`, which had silently stopped matching any crate after the package rename.
- **Storage TTL**: Tightened storage TTL bumps across all contracts to prevent silent archival of hot-path data (closes #570). Adds `bump_instance_ttl` to every public entrypoint in `trustforge_registry`, `trustforge_admin`, `trustforge_treasury`, `trustforge_arbitration`, `trustforge_multisig`, `timelock`, and `trustforge_delegation`; adds `extend_ttl` after every persistent write (and on reads) in `trustforge_bond` slash history, emergency audit trail, and claims modules.
- **Rebrand Artifacts**: Cleaned up all remaining Credence references in documentation, test data, and configuration files

### Removed

- Repository cleanup ahead of the first public release: removed 21 stray, never-meant-to-be-committed artifacts left over from local development (compiled test binaries, coverage dumps, scratch check/diff logs, a bogus Node lockfile, and 13 leftover PR-writeup/implementation-summary docs), plus an additional empty stray file and a leftover build log (containing a pre-rebrand local path) found during final release preparation. Added corresponding `.gitignore` rules to prevent recurrence.

### Documentation

- Corrected `docs/architecture.md` to remove two documented contracts (`dispute_resolution`, `fixed_duration_bond`) that do not exist as workspace members, and added the three previously-undocumented real crates (`templates`, `trustforge_admin_cli`, `testutils`) to the workspace crate table so it matches the root `Cargo.toml` members list exactly.
- Fixed stale GitHub repository references from `credenceprotocol/credence-contracts` and `CredenceOrg/Credence-Contracts` to `Softvaults/trustforge-contracts` throughout documentation.
- Added a "Reporting a Vulnerability" section to `SECURITY.md` (previously undocumented) pointing to the correct GitHub Security Advisories path.
- Added an `Apache-2.0` `LICENSE` file and `license` metadata to the workspace and all member `Cargo.toml` files.

### Security

- All contracts passed comprehensive internal security review
- Access control, reentrancy protection, and arithmetic safety validated
- Storage key stability ensured via DataKey fingerprint tests
- Emergency mechanisms and circuit breakers tested
- CEI (Checks-Effects-Interactions) pattern violations fixed

### Known Limitations

Documented in `docs/known-simplifications.md`:
1. Multisig proposals have no expiry (manual rejection required)
2. `get_all_identities()` unbounded (use event-based indexing for production)
3. Admin-assigned arbitrator weights (not stake-backed)

See [Security Audit Report](SECURITY_AUDIT.md) for detailed security assessment.

## [Unreleased]

### Changed

- **TrustForge rebrand**: Renamed the project from Credence to TrustForge across the entire workspace. Crate names (`credence_*` → `trustforge_*`), public contract types (`CredenceBond` → `TrustForgeBond`, `CredenceRegistry` → `TrustForgeRegistry`, `CredenceTreasury` → `TrustForgeTreasury`, `CredenceDelegation` → `TrustForgeDelegation`, `CredenceArbitration` → `TrustForgeArbitration`, `CredenceMultiSig` → `TrustForgeMultiSig`, and their generated `*Client` types), environment variables (`CREDENCE_*` → `TRUSTFORGE_*` in the admin CLI), and ~970 prose references across docs, comments, and CI text. Renamed `docs/credence-*.md` files to their `trustforge_*` equivalents and fixed all cross-references and anchor links.

### Fixed

- **Workspace build**: Resolved pre-existing merge corruption that was blocking the workspace build, as a prerequisite to the rebrand above.
- **CI**: Fixed the `cargo-geiger` unsafe-code scan glob in `security.yml`, which had silently stopped matching any crate after the package rename.
- Tighten storage TTL bumps across all contracts to prevent silent archival of hot-path data (closes #570). Adds `bump_instance_ttl` to every public entrypoint in `trustforge_registry`, `trustforge_admin`, `trustforge_treasury`, `trustforge_arbitration`, `trustforge_multisig`, `timelock`, and `trustforge_delegation`; adds `extend_ttl` after every persistent write (and on reads) in `trustforge_bond` slash history, emergency audit trail, and claims modules.

### Removed

- Repository cleanup ahead of the first public release: removed 21 stray, never-meant-to-be-committed artifacts left over from local development (compiled test binaries, coverage dumps, scratch check/diff logs, a bogus Node lockfile, and 13 leftover PR-writeup/implementation-summary docs), plus an additional empty stray file and a leftover build log (containing a pre-rebrand local path) found during final release preparation. Added corresponding `.gitignore` rules to prevent recurrence.

### Documentation

- Corrected `docs/architecture.md` to remove two documented contracts (`dispute_resolution`, `fixed_duration_bond`) that do not exist as workspace members, and added the three previously-undocumented real crates (`templates`, `trustforge_admin_cli`, `testutils`) to the workspace crate table so it matches the root `Cargo.toml` members list exactly.
- Fixed a stale `Credence-Contracts` clone URL and directory name in `CONTRIBUTING.md`, and a stale security-advisory link in the issue template, both left over from before the GitHub-level org/repo rename to `Softvaults/trustforge-contracts`.
- Added a "Reporting a Vulnerability" section to `SECURITY.md` (previously undocumented) pointing to the correct GitHub Security Advisories path.
- Added an `Apache-2.0` `LICENSE` file and `license` metadata to the workspace and all member `Cargo.toml` files.

### Added

- **Timelock Timeout Test**: Added explicit timeout regression coverage for time-locked operation execution after the grace period (`timelock`).
- **Pause Signer Invariant**: Added invariant test for PauseSignerCount to prevent drift (`trustforge_delegation`).
- **Slash Bond Core**: Implemented admin-only `slash_bond` functionality with partial/full slashing and event emission.
- **Treasury Guardrails**: Added comprehensive tests and functionality for liquidity floor and slippage protection mechanisms in treasury withdrawals (`trustforge_treasury`).
- **Batch Bond Atomicity**: Enhanced batch operations with explicit empty batch handling and `MAX_BATCH_BOND_SIZE` enforcement (`trustforge_bond`).

### Changed

- **SafeERC20 Migration**: Replaced direct `TokenClient` calls with safe wrapper functions to support non-compliant ERC20 tokens across the protocol.
- **Protocol Fixes**: Resolved compilation errors, completed `top_up` and `extend_duration` with overflow protection.
- **Event Indexing**: Migrated lifecycle events to V2 for optimized off-chain indexing.
