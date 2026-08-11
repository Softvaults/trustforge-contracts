# Known Simplifications

## Current Limitations

1. **No appeal mechanism** — Decisions are final once finalized
2. **No timelocks** — Disputes resolve immediately upon quorum
3. **Single dispute per identity per epoch** — Cannot have overlappingtes
4. **No appeal bond** — Appeals are free (not yet implemented)
5. **Manual dispute initiation** — No automated trigger
6. **No graceful shutdown** — Cannot pause arbitration
7. **No fee split** — All slashed XLM goes to protocol
8. **No arbitrator rotation** — Same arbitrators in all disputes

## Resolved

### 4. Slashed Funds Are Now Transferred to Treasury

Slashed funds are now transferred to the configured slash treasury on every `slash()` call via `token_integration::transfer_from_contract`. Slashing reverts with `ContractError::TreasuryNotConfigured` if no treasury has been set via `set_slash_treasury(admin, treasury)`.

### 7. get_all_identities() Now Has a Paginated Alternative

`get_identities_page(offset, limit)` was added, bounded to `MAX_IDENTITIES_PAGE_SIZE` (200) regardless of the requested `limit`. `get_all_identities()` is now `#[deprecated(note = "Use get_identities_page for bounded pagination")]` rather than removed, so existing off-chain callers keep working while migrating. See [registry.md](registry.md).

### 11. Multisig Proposals Now Expire

Proposals carry an `expires_at: u64` field (`0` = no expiration). `sign_proposal` and `execute_proposal` both reject once `expires_at > 0 && now >= expires_at`, and the permissionless, bounded `prune_expired_proposals(start_id, max_iter)` reclaims storage for expired proposals and their signature keys. See [multisig.md](multisig.md).

### 9. Arbitration Voting Weights Are Now Stake-Backed

`register_arbitrator` no longer takes a weight argument — it only grants voting permission. Voting weight is derived live, at `vote()` time, from the arbitrator's bonded stake: a cross-contract lookup through a configured `trustforge_registry` (`set_registry_contract`) resolves the arbitrator's bond contract, whose `bonded_amount - slashed_amount` becomes the weight. An arbitrator with no discoverable active bond gets `ArbitratorNotBonded`; a fully-slashed one gets `WeightNotPositive`. A cast vote's weight is a snapshot — later top-ups or slashes don't retroactively change already-cast votes. See [arbitration.md](arbitration.md).

---

## 1. Token Transfer is Stubbed in trustforge_bond

**Where:** `contracts/trustforge_bond/src/`

**What:** The bond contract's token transfer calls (`transfer_from`, `transfer`) are wired to a Soroban token interface, but the reference implementation uses a mock/test token rather than a live USDC contract on mainnet. In tests, `Env::default()` with `mock_all_auths()` is used, meaning no real token approval or balance check occurs against a deployed token contract.

**Impact:** The accounting logic (bonded amounts, slashing, fees, penalties) is fully implemented and correct. Only the external token call is stubbed for testing purposes.

**Production path:** Configure a real USDC token address via `set_usdc_token(admin, token, network)` before deployment. The balance-delta check in `token_integration.rs` will then enforce transfer integrity against the live token. See [token-integration.md](token-integration.md).

---

## 2. Single-Bond-Per-Contract-Instance Storage Model

**Status:** intentional design decision, not a gap — see
[architecture.md](architecture.md#trustforge_bond) for the full rationale and the
cost/alternative it was weighed against. Kept here for discoverability since it still
shapes how the registry and backend integration work.

**Where:** `contracts/trustforge_bond/src/lib.rs`

**What:** The bond contract stores one bond per contract instance (keyed by a single storage slot), not a per-identity map. Each identity that wants a bond deploys its own contract instance.

**Impact:** This simplifies the storage model and avoids cross-identity data leakage, but it means the registry contract (`trustforge_registry`) is required to track which contract instance belongs to which identity. Batch operations across identities require iterating registry entries off-chain.

**If per-identity deployment cost becomes a real adoption blocker:** A multi-bond contract with a `Map<Address, IdentityBond>` storage layout would allow a single contract to serve many identities. The registry would still be useful for discovery but would not be strictly required for storage. See [registry.md](registry.md).

## Summary Table

| # | Simplification | Contract | Production Path |
|---|---------------|----------|-----------------|
| 1 | Token transfer stubbed in tests | trustforge_bond | Configure live USDC via `set_usdc_token` |
| 2 | Single-bond-per-contract-instance | trustforge_bond | Intentional — see [architecture.md](architecture.md#trustforge_bond) |
| 3 | Treasury is pure accounting, no token custody | trustforge_treasury | Add real token transfers on withdrawal |
| 4 | ~~Slashed funds not swept to treasury~~ | trustforge_bond | **Resolved** — see above |
| 6 | Early-exit penalty dropped if no treasury | trustforge_bond | Require treasury before `withdraw_early` |
| 7 | ~~`get_all_identities()` unbounded~~ | trustforge_registry | **Resolved** — `get_identities_page` added |
| 9 | ~~Arbitrator weights not stake-backed~~ | trustforge_arbitration | **Resolved** — weight derived from bond balance |
| 11 | ~~Multisig proposals have no expiry~~ | trustforge_multisig | **Resolved** — `expires_at` added |
