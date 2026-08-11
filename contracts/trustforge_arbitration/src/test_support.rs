#![cfg(test)]
//! Shared fixtures for stake-derived-weight arbitration tests.
//!
//! Building a voting-eligible arbitrator now requires three real contracts in
//! the same `Env`: a mock token, a `trustforge_bond` instance holding the
//! stake, and a `trustforge_registry` entry linking the arbitrator's identity
//! to that bond contract. This module wires that up once so the individual
//! test files don't each reimplement it.
//!
//! `trustforge_bond`/`trustforge_registry` are dev-dependencies only (see
//! Cargo.toml) — they never ship in the release WASM, so this is safe even
//! though `lib.rs`'s production code path deliberately avoids depending on
//! either crate (see `BondRegistryEntry`'s doc comment there).

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use trustforge_bond::{TrustForgeBond, TrustForgeBondClient};
use trustforge_registry::{TrustForgeRegistry, TrustForgeRegistryClient};

/// Scale a whole-token count to the 18-decimal amount `trustforge_bond`
/// expects. Tests express relative weight in whole units, matching the old
/// admin-set integer weights this fixture replaces (e.g. `bond_units(10)`
/// roughly corresponds to the old `register_arbitrator(&arb, &10)`) — the
/// absolute scale doesn't matter to these tests, only the ratios between
/// arbitrators' stakes do, and 18-decimal scale keeps every value comfortably
/// above `trustforge_bond`'s `MIN_BOND_AMOUNT` production floor.
pub fn bond_units(units: i128) -> i128 {
    units * 1_000_000_000_000_000_000
}

/// Deterministic-failure-free mock SEP-41 token, minimal copy of
/// `trustforge_bond::test_helpers::MockStellarAsset` (not reusable directly:
/// that module is `#[cfg(test)]` inside `trustforge_bond` itself, so it isn't
/// part of the compiled artifact this crate can depend on).
#[contract]
pub struct MockStellarAsset;

#[contractimpl]
impl MockStellarAsset {
    pub fn decimals(_e: Env) -> u32 {
        18
    }
    pub fn balance(e: Env, id: Address) -> i128 {
        e.storage().instance().get(&id).unwrap_or(0)
    }
    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        let from_bal = Self::balance(e.clone(), from.clone());
        let to_bal = Self::balance(e.clone(), to.clone());
        e.storage().instance().set(&from, &(from_bal - amount));
        e.storage().instance().set(&to, &(to_bal + amount));
    }
    pub fn transfer_from(e: Env, _spender: Address, from: Address, to: Address, amount: i128) {
        Self::transfer(e, from, to, amount);
    }
    pub fn allowance(_e: Env, _from: Address, _spender: Address) -> i128 {
        i128::MAX
    }
    pub fn approve(_e: Env, _from: Address, _spender: Address, _amount: i128, _expiration: u32) {
        // no-op: allowance() always reports unlimited.
    }
    pub fn mint(e: Env, to: Address, amount: i128) {
        let current = Self::balance(e.clone(), to.clone());
        e.storage().instance().set(&to, &(current + amount));
    }
    pub fn set_authorized(_e: Env, _id: Address, _auth: bool) {
        // no-op
    }
}

/// Deploy a fresh `trustforge_registry`, initialized with `admin`.
pub fn deploy_registry(e: &Env, admin: &Address) -> Address {
    let contract_id = e.register(TrustForgeRegistry, ());
    let client = TrustForgeRegistryClient::new(e, &contract_id);
    client.initialize(admin);
    contract_id
}

/// Deploy a bonded identity — mock token + a `trustforge_bond` instance with
/// `bonded_amount` bonded for a freshly generated identity — and register that
/// identity/bond pair in `registry`. Returns the identity address, ready to
/// pass to `register_arbitrator`/`vote`.
///
/// `registry`'s `register` call is admin-gated; callers must have
/// `e.mock_all_auths()` set (as every test in this crate already does), since
/// this doesn't separately thread through registry's admin identity.
///
/// `trustforge_bond` doesn't implement `supports_interface`, so this always
/// registers with `allow_non_interface = true` — see
/// `docs/known-simplifications.md` ("no cross-contract binding between bond
/// and registry at initialization").
pub fn setup_registered_arbitrator(e: &Env, registry: &Address, bonded_amount: i128) -> Address {
    let bond_contract_id = e.register(TrustForgeBond, ());
    let bond_client = TrustForgeBondClient::new(e, &bond_contract_id);
    let bond_admin = Address::generate(e);
    let identity = Address::generate(e);

    bond_client.initialize(&bond_admin, &None);

    let stellar_asset = e.register(MockStellarAsset, ());
    let stellar_client = StellarAssetClient::new(e, &stellar_asset);
    stellar_client.set_authorized(&identity, &true);
    stellar_client.mint(&identity, &bonded_amount);

    let token_client = TokenClient::new(e, &stellar_asset);
    let expiration = e.ledger().sequence().saturating_add(10_000);
    token_client.approve(&identity, &bond_contract_id, &bonded_amount, &expiration);

    let mut accepted_tokens = Vec::new(e);
    accepted_tokens.push_back(stellar_asset.clone());
    bond_client.set_accepted_tokens(&bond_admin, &accepted_tokens);
    bond_client.set_token(&bond_admin, &stellar_asset);

    bond_client.create_bond(&identity, &bonded_amount, &86_400_u64, &false, &0_u64);

    let registry_client = TrustForgeRegistryClient::new(e, registry);
    registry_client.register(&identity, &bond_contract_id, &true);

    identity
}
