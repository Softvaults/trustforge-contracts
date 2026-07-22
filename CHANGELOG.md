# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
