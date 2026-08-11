# Arbitration Voting System

The `TrustForgeArbitration` contract provides a weighted voting mechanism for dispute resolution, allowing authorized arbitrators to decide on outcomes.

## Overview

Disputes are created with a specific duration. During this time, registered arbitrators can cast weighted votes for different outcomes. Once the voting period ends, the dispute can be resolved, and the outcome with the highest total weight is declared the winner.

**Voting weight is derived from bonded stake, not admin-assigned.** `register_arbitrator` only grants voting *permission* — it takes no weight argument. A vote's weight is resolved live, at the moment `vote()` is called, via a cross-contract lookup: `trustforge_registry.get_bond_contract(voter)` finds the arbitrator's bond contract, then that contract's `get_identity_state()` supplies `bonded_amount - slashed_amount` as the weight. This means:

- The admin controls *who* may vote, not *how much* their vote counts — voting power is backed by real, slashable stake.
- An arbitrator must have an active registry entry pointing at an active bond with positive available balance to vote at all; otherwise `vote()` rejects with `ArbitratorNotBonded` (unregistered/deactivated entry or bond) or `WeightNotPositive` (bond fully slashed).
- A cast vote's weight is a **snapshot at cast time** — later top-ups or slashes of an arbitrator's bond do not retroactively change already-cast votes, since only the aggregated per-outcome tally (not a per-voter weight) is stored afterward.
- The arbitration contract must be pointed at a `trustforge_registry` instance via `set_registry_contract` before any vote can succeed.

See [`docs/known-simplifications.md`](known-simplifications.md) item 9 for the design history, and `contracts/trustforge_arbitration/tests/test_weight_derivation.rs` for the behavioral test coverage.

## Dispute Status Machine

Disputes follow a canonical status machine with enforced transitions:

```
Open ──────> Voting ──────> Resolving ──────> Resolved
  │            │              │
  └────────────┴──────────────┤──────────> Cancelled
                              ↓
                             Tied
```

### Valid Transitions

- `Open → Voting` — Voting period begins (implicit at creation)
- `Voting → Resolving` — Voting period ends, `resolve_dispute` called
- `Voting → Cancelled` — Dispute cancelled by creator or admin
- `Resolving → Resolved` — Outcome tallied with a clear winner
- `Resolving → Tied` — Outcome tallied with a tie (two or more outcomes have equal highest weight)
- `Open → Cancelled` — Cancelled before voting starts

All other transitions are rejected with `ArbitrationError::InvalidTransition`.

## Types

### DisputeStatus

| Status    | Value | Description                                       |
| --------- | ----- | ------------------------------------------------- |
| Open      | 0     | Initial state (immediately transitions to Voting) |
| Voting    | 1     | Arbitrators can cast votes                        |
| Resolving | 2     | Tallying votes (transient state)                  |
| Resolved  | 3     | Final outcome determined with a clear winner      |
| Cancelled | 4     | Dispute cancelled by creator or admin             |
| Tied      | 5     | Votes resulted in a tie (equal highest weights)   |

### Dispute

| Field        | Type          | Description                            |
| ------------ | ------------- | -------------------------------------- |
| id           | u64           | Unique identifier for the dispute      |
| creator      | Address       | Address that created the dispute       |
| description  | String        | Brief description of the dispute       |
| voting_start | u64           | Timestamp when voting begins           |
| voting_end   | u64           | Timestamp when voting ends             |
| status       | DisputeStatus | Current status in the lifecycle        |
| outcome      | u32           | The winning outcome (0 only when Tied) |

### ArbitrationError

| Error              | Code | Description                              |
| ------------------ | ---- | ---------------------------------------- |
| InvalidTransition  | 1    | Attempted an invalid status transition   |
| AlreadyInitialized | 2    | Contract already initialized             |
| NotInitialized     | 3    | Contract not initialized                 |
| NotAdmin           | 4    | Caller is not the admin                  |
| NotArbitrator      | 5    | Voter is not a registered arbitrator     |
| AlreadyVoted       | 6    | Arbitrator already voted on this dispute |
| VotingInactive     | 7    | Voting period is not active              |
| VotingNotEnded     | 8    | Voting period has not ended yet          |
| DisputeNotFound    | 9    | Dispute ID does not exist                |
| InvalidOutcome     | 10   | Outcome must be > 0                      |
| WeightNotPositive  | 11   | Arbitrator's derived weight (bonded_amount - slashed_amount) is not positive |
| NotAuthorized      | 12   | Caller not authorized for this action    |
| ReasonTooLong      | 14   | Cancellation reason exceeds the length limit |
| QuorumNotMet       | 13   | Resolution attempted before the configured weight/voter quorum is met |
| RegistryNotConfigured | 15 | `set_registry_contract` has not been called yet |
| ArbitratorNotBonded | 16  | No active, discoverable bonded stake for this arbitrator (unregistered/deactivated registry entry, or inactive/unreachable bond contract) |

## Contract Functions

### `initialize(admin: Address) -> Result<(), ArbitrationError>`

Sets the contract administrator. Can only be called once.

### `set_registry_contract(admin: Address, registry: Address) -> Result<(), ArbitrationError>`

Configures the `trustforge_registry` contract used to resolve an arbitrator's bond contract for weight derivation. Requires admin authorization. Must be called before any `vote()` can succeed.

### `get_registry_contract() -> Result<Address, ArbitrationError>`

Returns the configured `trustforge_registry` address, or `RegistryNotConfigured` if unset.

### `register_arbitrator(arbitrator: Address) -> Result<(), ArbitrationError>`

Grants `arbitrator` permission to vote. Requires admin authorization. Does **not** set a weight — voting weight is derived live from bonded stake at `vote()` time (see [Overview](#overview)).

### `unregister_arbitrator(arbitrator: Address) -> Result<(), ArbitrationError>`

Removes an arbitrator's voting rights. Requires admin authorization.

### `create_dispute(creator: Address, description: String, duration: u64) -> Result<u64, ArbitrationError>`

Creates a new dispute. Requires creator authorization. Returns the dispute ID. Status starts as `Voting`.

### `cancel_dispute(caller: Address, dispute_id: u64) -> Result<(), ArbitrationError>`

Cancels a dispute. Only the creator or admin may cancel. Valid from `Open` or `Voting` status.

### `vote(voter: Address, dispute_id: u64, outcome: u32) -> Result<(), ArbitrationError>`

Casts a weighted vote for an outcome. Requires voter authorization. Voter must be a registered arbitrator with a positive derived weight (see [Overview](#overview)). Dispute must be in `Voting` status.

### `resolve_dispute(dispute_id: u64) -> Result<u32, ArbitrationError>`

Resolves the dispute after the voting period has ended. Transitions `Voting → Resolving → Resolved` (or `Tied` if outcomes are tied). Calculates the winning outcome based on total weight. Returns the winning outcome if one is found, or 0 if a tie is detected. On tie, the dispute transitions to `Tied` status so consumers can distinguish it from a definite ruling.

### `get_dispute(dispute_id: u64) -> Result<Dispute, ArbitrationError>`

Retrieves the details of a specific dispute.

### `get_tally(dispute_id: u64, outcome: u32) -> i128`

Returns the current total weight for a specific outcome.

### `get_arbitrator_weight(arbitrator: Address) -> Result<i128, ArbitrationError>`

Returns `arbitrator`'s current derived weight (`bonded_amount - slashed_amount` from their registered bond), or `NotArbitrator`/`RegistryNotConfigured`/`ArbitratorNotBonded` if it can't be resolved. `i128`, not `u32`: weight is a raw token amount (typically 18-decimal), which routinely exceeds `u32::MAX` for realistic bonded amounts.

## Events

- `arbitrator_registered` — Emitted when an arbitrator is registered or updated
- `arbitrator_unregistered` — Emitted when an arbitrator is removed
- `dispute_created` — Emitted when a new dispute is opened
- `status_transition` — Emitted on every status change (from, to)
- `vote_cast` — Emitted when an arbitrator casts a vote
- `dispute_cancelled` — Emitted when a dispute is cancelled
- `dispute_resolved` — Emitted when a dispute is resolved with a clear winner
- `dispute_tied` — Emitted when a dispute resolution results in a tie

## Security

- Admin-only functions for arbitrator management and registry configuration
- Voting weight is derived from bonded stake via cross-contract lookup, not admin-assigned — see [Overview](#overview)
- Authorization required for creating disputes and casting votes
- Double-voting prevention
- Time-bound voting periods
- Overflow protection for weight tallies and counters
- Canonical status machine prevents invalid state transitions
- Result-based error handling for all state-changing operations

## Testing

The contract includes comprehensive test coverage:

- Basic arbitration flow (creation, voting, resolution)
- Tie scenarios
- Double-voting prevention
- Unauthorized voter rejection
- All valid status transitions
- All invalid status transitions (regression tests)
- Edge cases (outcome validation, quorum boundaries, etc.)
- Stake-derived weight behavior (`tests/test_weight_derivation.rs`): weight tracks bonded amount rather than an admin-set number, unbonded arbitrators can't vote, a cast vote's weight snapshot survives later top-ups/slashes, and the tally's overflow guard

Run tests:

```bash
cargo test -p trustforge_arbitration
```
