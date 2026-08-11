//! Stake-derived arbitrator weight — behavioral tests.
//!
//! An external integration test (like `datakey_fingerprint.rs`), so it can't see
//! `src/test_support.rs` (that's `#[cfg(test)]`-gated, only compiled when
//! `trustforge_arbitration` itself is the crate under test — not when it's
//! linked as a dependency by a separate `tests/*.rs` binary). Self-contained
//! instead, duplicating the small mock-token + bond/registry setup rather than
//! promoting `trustforge_bond`/`trustforge_registry` to non-dev dependencies,
//! which would reintroduce the WASM export-symbol collision documented on
//! `trustforge_arbitration::BondRegistryEntry`.

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use trustforge_arbitration::status::ArbitrationError;
use trustforge_arbitration::{TrustForgeArbitration, TrustForgeArbitrationClient};
use trustforge_bond::{TrustForgeBond, TrustForgeBondClient};
use trustforge_registry::{TrustForgeRegistry, TrustForgeRegistryClient};

const ONE_TOKEN: i128 = 1_000_000_000_000_000_000;

#[contract]
struct MockStellarAsset;

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
    pub fn approve(_e: Env, _from: Address, _spender: Address, _amount: i128, _expiration: u32) {}
    pub fn mint(e: Env, to: Address, amount: i128) {
        let current = Self::balance(e.clone(), to.clone());
        e.storage().instance().set(&to, &(current + amount));
    }
    pub fn set_authorized(_e: Env, _id: Address, _auth: bool) {}
}

fn deploy_registry(e: &Env, admin: &Address) -> Address {
    let contract_id = e.register(TrustForgeRegistry, ());
    TrustForgeRegistryClient::new(e, &contract_id).initialize(admin);
    contract_id
}

/// Bond `identity` (freshly generated) for `bond_amount`, minting `mint_amount`
/// (>= `bond_amount`, leaving `mint_amount - bond_amount` spare balance for
/// top-up tests) and registering the pair in `registry`. Returns
/// `(identity, bond_contract, bond_admin)`.
fn setup_bonded_arbitrator(
    e: &Env,
    registry: &Address,
    mint_amount: i128,
    bond_amount: i128,
) -> (Address, Address, Address) {
    let bond_contract_id = e.register(TrustForgeBond, ());
    let bond_client = TrustForgeBondClient::new(e, &bond_contract_id);
    let bond_admin = Address::generate(e);
    let identity = Address::generate(e);

    bond_client.initialize(&bond_admin, &None);

    let stellar_asset = e.register(MockStellarAsset, ());
    let stellar_client = StellarAssetClient::new(e, &stellar_asset);
    stellar_client.set_authorized(&identity, &true);
    stellar_client.mint(&identity, &mint_amount);

    let token_client = TokenClient::new(e, &stellar_asset);
    let expiration = e.ledger().sequence().saturating_add(10_000);
    token_client.approve(&identity, &bond_contract_id, &mint_amount, &expiration);

    let mut accepted_tokens = Vec::new(e);
    accepted_tokens.push_back(stellar_asset.clone());
    bond_client.set_accepted_tokens(&bond_admin, &accepted_tokens);
    bond_client.set_token(&bond_admin, &stellar_asset);

    bond_client.create_bond(&identity, &bond_amount, &86_400_u64, &false, &0_u64);

    let registry_client = TrustForgeRegistryClient::new(e, registry);
    registry_client.register(&identity, &bond_contract_id, &true);

    (identity, bond_contract_id, bond_admin)
}

fn advance(e: &Env, secs: u64) {
    e.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: e.ledger().timestamp() + secs,
        protocol_version: 22,
        sequence_number: e.ledger().sequence() + 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 16,
        max_entry_ttl: 1000,
    });
}

/// Admin cannot set arbitrary weight (`register_arbitrator` takes no weight
/// argument at all) — weight is derived purely from the arbitrator's bonded
/// amount, so two arbitrators bonded for different amounts get different
/// weights with no admin input beyond granting voting permission.
#[test]
fn test_weight_derived_from_bond_not_admin() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let contract_id = e.register(TrustForgeArbitration, ());
    let client = TrustForgeArbitrationClient::new(&e, &contract_id);
    client.initialize(&admin);
    let registry = deploy_registry(&e, &admin);
    client.set_registry_contract(&admin, &registry);

    let (arb_small, ..) = setup_bonded_arbitrator(&e, &registry, ONE_TOKEN * 3, ONE_TOKEN * 3);
    let (arb_large, ..) = setup_bonded_arbitrator(&e, &registry, ONE_TOKEN * 9, ONE_TOKEN * 9);
    client.register_arbitrator(&arb_small);
    client.register_arbitrator(&arb_large);

    // register_arbitrator's only argument is the address — there is no weight
    // parameter to pass an admin-chosen number through, by construction.
    assert_eq!(client.get_arbitrator_weight(&arb_small), ONE_TOKEN * 3);
    assert_eq!(client.get_arbitrator_weight(&arb_large), ONE_TOKEN * 9);
}

/// An arbitrator with no discoverable bonded stake (never registered in the
/// configured `trustforge_registry`) cannot vote — permission to participate
/// (`register_arbitrator`) is separate from having weight to back a vote.
#[test]
fn test_zero_weight_arbitrator_cannot_vote() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let creator = Address::generate(&e);
    let contract_id = e.register(TrustForgeArbitration, ());
    let client = TrustForgeArbitrationClient::new(&e, &contract_id);
    client.initialize(&admin);
    let registry = deploy_registry(&e, &admin);
    client.set_registry_contract(&admin, &registry);

    // Permitted to vote, but never bonded — no registry entry to resolve.
    let unbonded_arb = Address::generate(&e);
    client.register_arbitrator(&unbonded_arb);

    let dispute_id = client.create_dispute(&creator, &String::from_str(&e, "zero weight"), &3600);
    let err = client
        .try_vote(&unbonded_arb, &dispute_id, &1)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ArbitrationError::ArbitratorNotBonded);

    // The tally is untouched — the rejected vote never got recorded.
    assert_eq!(client.get_tally(&dispute_id, &1), 0);
}

/// A vote's recorded weight is a snapshot at the moment it was cast: topping up
/// a bond *after* voting must not retroactively change the already-cast vote's
/// contribution to the tally.
#[test]
fn test_weight_snapshot_immutable() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let creator = Address::generate(&e);
    let contract_id = e.register(TrustForgeArbitration, ());
    let client = TrustForgeArbitrationClient::new(&e, &contract_id);
    client.initialize(&admin);
    let registry = deploy_registry(&e, &admin);
    client.set_registry_contract(&admin, &registry);

    // Mint double the initial bond so there's spare balance to top up with.
    let (arb, bond_contract, _) =
        setup_bonded_arbitrator(&e, &registry, ONE_TOKEN * 20, ONE_TOKEN * 10);
    client.register_arbitrator(&arb);

    let dispute_id = client.create_dispute(&creator, &String::from_str(&e, "snapshot"), &3600);
    client.vote(&arb, &dispute_id, &1);
    assert_eq!(client.get_tally(&dispute_id, &1), ONE_TOKEN * 10);

    // Top up well after the vote was cast.
    let bond_client = TrustForgeBondClient::new(&e, &bond_contract);
    bond_client.top_up(&arb, &(ONE_TOKEN * 10));
    assert_eq!(client.get_arbitrator_weight(&arb), ONE_TOKEN * 20);

    // The already-cast vote's contribution to the tally is unchanged.
    assert_eq!(client.get_tally(&dispute_id, &1), ONE_TOKEN * 10);
}

/// Symmetric to the top-up case: slashing an arbitrator's bond *after* they've
/// voted must not retroactively reduce the already-cast vote's tally
/// contribution — only future votes see the reduced weight.
#[test]
fn test_slashed_arbitrator_mid_dispute() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let creator = Address::generate(&e);
    let contract_id = e.register(TrustForgeArbitration, ());
    let client = TrustForgeArbitrationClient::new(&e, &contract_id);
    client.initialize(&admin);
    let registry = deploy_registry(&e, &admin);
    client.set_registry_contract(&admin, &registry);

    let (arb, bond_contract, bond_admin) =
        setup_bonded_arbitrator(&e, &registry, ONE_TOKEN * 10, ONE_TOKEN * 10);
    client.register_arbitrator(&arb);

    let dispute_id = client.create_dispute(&creator, &String::from_str(&e, "slash mid"), &3600);
    client.vote(&arb, &dispute_id, &1);
    assert_eq!(client.get_tally(&dispute_id, &1), ONE_TOKEN * 10);

    // Slashing is rejected in the same ledger as the bond's creation (guards
    // against same-ledger collateral manipulation) — advance one ledger first.
    advance(&e, 5);
    let bond_client = TrustForgeBondClient::new(&e, &bond_contract);
    // slash() requires a configured slash treasury (funds are swept there).
    let treasury = Address::generate(&e);
    bond_client.set_slash_treasury(&bond_admin, &treasury);
    bond_client.slash(&bond_admin, &(ONE_TOKEN * 5));

    // The already-cast vote's tally contribution is unchanged.
    assert_eq!(client.get_tally(&dispute_id, &1), ONE_TOKEN * 10);

    // But a *fresh* weight query now reflects the reduced stake.
    assert_eq!(client.get_arbitrator_weight(&arb), ONE_TOKEN * 5);
}

/// Sanity regression: the dispute status machine (Voting → Resolving →
/// Resolved) is unaffected by deriving weight from stake instead of an
/// admin-set number — exhaustive transition coverage lives in
/// `src/test_lifecycle.rs`, this just checks the common path end-to-end.
#[test]
fn test_status_machine_preserved() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let creator = Address::generate(&e);
    let contract_id = e.register(TrustForgeArbitration, ());
    let client = TrustForgeArbitrationClient::new(&e, &contract_id);
    client.initialize(&admin);
    let registry = deploy_registry(&e, &admin);
    client.set_registry_contract(&admin, &registry);

    let (arb, ..) = setup_bonded_arbitrator(&e, &registry, ONE_TOKEN * 10, ONE_TOKEN * 10);
    client.register_arbitrator(&arb);

    let dispute_id = client.create_dispute(&creator, &String::from_str(&e, "status"), &3600);
    assert_eq!(
        client.get_dispute(&dispute_id).status,
        trustforge_arbitration::status::DisputeStatus::Voting
    );

    client.vote(&arb, &dispute_id, &1);
    advance(&e, 3601);
    let winner = client.resolve_dispute(&dispute_id);
    assert_eq!(winner, 1);
    assert_eq!(
        client.get_dispute(&dispute_id).status,
        trustforge_arbitration::status::DisputeStatus::Resolved
    );
}

/// The tally's `checked_add` (vote()'s `current_tally.checked_add(weight)`)
/// must panic with a typed `Overflow` rather than silently wrapping when two
/// large stakes voting for the same outcome would overflow `i128`.
#[test]
#[should_panic(expected = "Error(Contract, #700)")]
fn test_weight_aggregation_checked_arithmetic() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let creator = Address::generate(&e);
    let contract_id = e.register(TrustForgeArbitration, ());
    let client = TrustForgeArbitrationClient::new(&e, &contract_id);
    client.initialize(&admin);
    let registry = deploy_registry(&e, &admin);
    client.set_registry_contract(&admin, &registry);

    let (arb, ..) = setup_bonded_arbitrator(&e, &registry, ONE_TOKEN, ONE_TOKEN);
    client.register_arbitrator(&arb);

    let dispute_id = client.create_dispute(&creator, &String::from_str(&e, "overflow"), &3600);

    // Realistic bonded amounts can't reach i128::MAX through real votes —
    // trustforge_bond's own MAX_BOND_AMOUNT ceiling (1e26) sits far below it —
    // so this seeds an already-near-overflow tally directly in storage and lets
    // one ordinary vote push it over, exercising vote()'s
    // `current_tally.checked_add(weight)` overflow path (ContractError::Overflow
    // = 700) the same way it would fire if that ceiling were ever raised.
    e.as_contract(&contract_id, || {
        let mut votes: soroban_sdk::Map<u32, i128> = soroban_sdk::Map::new(&e);
        votes.set(1u32, i128::MAX - 1);
        e.storage().instance().set(
            &trustforge_arbitration::DataKey::DisputeVotes(dispute_id),
            &votes,
        );
    });

    client.vote(&arb, &dispute_id, &1); // tally overflows here
}
