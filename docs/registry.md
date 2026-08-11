# TrustForge Registry

The registry contract maps identity addresses to their bond contract addresses, enabling efficient forward and reverse lookups across the TrustForge trust protocol.

## Uniqueness Invariants

The registry enforces strict uniqueness at two levels:

| Invariant | Error code | Description |
|---|---|---|
| Identity uniqueness | `#400 IdentityAlreadyRegistered` | Each identity address may appear at most once in the forward mapping |
| Bond contract uniqueness | `#401 BondContractAlreadyRegistered` | Each bond contract address may appear at most once in the reverse mapping |

Both checks are enforced atomically inside `register`. A deactivated entry still occupies its storage slot and therefore still blocks re-registration — use `remove` to free the slot.

## Operations

### `register(identity, bond_contract, allow_non_interface) → RegistryEntry`

Creates a new identity → bond mapping. Requires admin auth.

- Panics `#400` if `identity` already has an entry (active **or** deactivated).
- Panics `#401` if `bond_contract` is already mapped to any identity.
- Panics if `allow_non_interface = false` and the bond contract does not implement `supports_interface(IFACE_TRUSTFORGE_BOND_V1)`.

### `deactivate(identity)`

Soft-deletes the entry by setting `active = false`. The storage keys remain; the identity and bond contract are **not** freed for re-use. Requires admin auth.

- Panics `#402` if identity is not registered.
- Panics `#404` if already deactivated.

### `reactivate(identity)`

Restores a soft-deleted entry to `active = true`. Requires admin auth.

- Panics `#402` if identity is not registered.
- Panics `#405` if already active.

### `remove(identity)`

Hard-deletes the entry. Removes both the forward mapping (`identity → bond`) and the reverse mapping (`bond → identity`) from storage, and removes the identity from the `RegisteredIdentities` list. After removal both the identity and the bond contract are free to be re-registered. Requires admin auth.

- Panics `#402` if identity is not registered.
- Works on both active and deactivated entries.

### `get_bond_contract(identity) → RegistryEntry`

Forward lookup. Panics `#402` if not registered.

### `get_identity(bond_contract) → Address`

Reverse lookup. Panics `#403` if not registered.

### `is_registered(identity) → bool`

Returns `true` only if the entry exists **and** `active = true`.

### `get_identities_page(offset, limit) → Vec<Address>`

Bounded, paginated read of registered identity addresses, in insertion order. `limit` is
capped at `MAX_IDENTITIES_PAGE_SIZE` (200) regardless of the requested value. `offset >=`
the current identity count returns an empty vec rather than panicking, so callers can page
until they see fewer than `limit` results without a separate count check.

```
get_identities_page(0, 50)    → first 50 identities
get_identities_page(50, 50)   → next 50
```

### `get_all_identities() → Vec<Address>` (deprecated)

Returns every registered identity in one unbounded call. `#[deprecated]` since it will
eventually exceed Soroban's per-transaction resource limits as the registry grows — use
`get_identities_page` for bounded reads, or listen for `identity_registered` events for
off-chain, event-based discovery instead of polling either call.

## Remove / Reinsert Semantics

```
register(id, bond_A)   → ok
deactivate(id)         → soft-delete; id still blocks re-registration
register(id, bond_B)   → ERROR #400  (entry still in storage)

remove(id)             → hard-delete; both id and bond_A are freed
register(id, bond_B)   → ok  (fresh entry, new timestamp)
register(id2, bond_A)  → ok  (bond_A is also free again)
```

## Error Code Reference

| Code | Name | Trigger |
|---|---|---|
| `#400` | `IdentityAlreadyRegistered` | `register` called with an identity that already has a storage entry |
| `#401` | `BondContractAlreadyRegistered` | `register` called with a bond contract already mapped to another identity |
| `#402` | `IdentityNotRegistered` | `get_bond_contract`, `deactivate`, `reactivate`, or `remove` called for an unknown identity |
| `#403` | `BondContractNotRegistered` | `get_identity` called for an unknown bond contract |
| `#404` | `AlreadyDeactivated` | `deactivate` called on an already-inactive entry |
| `#405` | `AlreadyActive` | `reactivate` called on an already-active entry |

Part of the TrustForge protocol contracts.

## Known Simplifications

- There is no cross-contract binding between bond and registry at initialization.

`get_all_identities()`'s lack of pagination is resolved — see `get_identities_page` above.
See [known-simplifications.md](known-simplifications.md) for the full list and other contracts' production paths.
