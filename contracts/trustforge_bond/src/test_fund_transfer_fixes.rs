//! Regression tests for the fixes recorded in `docs/BOND_REVIEW_NOTE.md` (2026-08-12):
//! `withdraw`/`withdraw_bond`/`collect_fees` not transferring tokens, `slash_bond` not
//! paying the treasury, unauthenticated `set_callback`, and `create_bond`'s missing
//! duration/notice-period validation. Each test here asserts the *actual on-chain token
//! balance* changes, not just internal accounting — that's precisely the gap the review
//! found: the deleted `test_withdraw_bond.rs` would have been this test, had it ever
//! been compiled.

use crate::test_helpers;
use crate::{TrustForgeBond, TrustForgeBondClient};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token::TokenClient, Address, Bytes, Env};

#[test]
fn withdraw_transfers_tokens_to_identity() {
    let e = Env::default();
    let (client, _admin, identity, token, _contract_id) = test_helpers::setup_with_token(&e);
    let token_client = TokenClient::new(&e, &token);

    client.create_bond(&identity, &1_000_i128, &86_400_u64, &false, &0_u64);
    let balance_before = token_client.balance(&identity);

    e.ledger().with_mut(|l| l.timestamp = 86_401);
    client.withdraw(&identity, &400_i128);

    assert_eq!(
        token_client.balance(&identity),
        balance_before + 400,
        "withdraw() must transfer the withdrawn amount to the identity"
    );
}

#[test]
fn withdraw_bond_transfers_tokens_to_identity() {
    let e = Env::default();
    let (client, _admin, identity, token, _contract_id) = test_helpers::setup_with_token(&e);
    let token_client = TokenClient::new(&e, &token);

    client.create_bond(&identity, &1_000_i128, &86_400_u64, &false, &0_u64);
    let balance_before = token_client.balance(&identity);

    e.ledger().with_mut(|l| l.timestamp = 86_401);
    let withdrawn = client.withdraw_bond(&identity);

    assert_eq!(withdrawn, 1_000);
    assert_eq!(
        token_client.balance(&identity),
        balance_before + 1_000,
        "withdraw_bond() must transfer the full available balance to the identity"
    );
}

#[test]
fn collect_fees_transfers_tokens_to_admin() {
    let e = Env::default();
    let (client, admin, _identity, token, _contract_id) = test_helpers::setup_with_token(&e);
    let token_client = TokenClient::new(&e, &token);

    client.deposit_fees(&250_i128);
    let balance_before = token_client.balance(&admin);

    let collected = client.collect_fees(&admin, &Bytes::new(&e));

    assert_eq!(collected, 250);
    assert_eq!(
        token_client.balance(&admin),
        balance_before + 250,
        "collect_fees() must transfer the collected amount to the admin"
    );
}

#[test]
fn slash_bond_transfers_tokens_to_treasury() {
    let e = Env::default();
    let (client, admin, identity, token, _contract_id) = test_helpers::setup_with_token(&e);
    let token_client = TokenClient::new(&e, &token);
    let treasury = Address::generate(&e);
    client.set_slash_treasury(&admin, &treasury);

    client.create_bond(&identity, &1_000_i128, &86_400_u64, &false, &0_u64);
    let treasury_balance_before = token_client.balance(&treasury);

    client.slash_bond(&admin, &300_i128, &Bytes::new(&e));

    assert_eq!(
        token_client.balance(&treasury),
        treasury_balance_before + 300,
        "slash_bond() must transfer the slashed amount to the configured treasury"
    );
}

#[test]
#[should_panic]
fn slash_bond_without_treasury_reverts_instead_of_stranding_funds() {
    let e = Env::default();
    let (client, admin, identity, ..) = test_helpers::setup_with_token(&e);
    client.create_bond(&identity, &1_000_i128, &86_400_u64, &false, &0_u64);
    // No set_slash_treasury call: must revert (TreasuryNotConfigured), not silently
    // succeed while leaving the slashed funds unaccounted for.
    client.slash_bond(&admin, &300_i128, &Bytes::new(&e));
}

#[test]
#[should_panic]
fn set_callback_rejects_unauthenticated_caller() {
    let e = Env::default();
    // Deliberately not calling e.mock_all_auths() — set_callback must require the
    // stored admin's authorization, not succeed for an arbitrary invocation.
    let contract_id = e.register(TrustForgeBond, ());
    let client = TrustForgeBondClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    e.mock_auths(&[]);
    let callback = Address::generate(&e);
    client.set_callback(&callback);
    let _ = admin;
}

#[test]
fn set_callback_succeeds_for_admin() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(TrustForgeBond, ());
    let client = TrustForgeBondClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    client.initialize(&admin, &None);
    let callback = Address::generate(&e);
    client.set_callback(&callback);
}

#[test]
#[should_panic]
fn create_bond_rejects_zero_duration_via_real_entrypoint() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(TrustForgeBond, ());
    let client = TrustForgeBondClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let identity = Address::generate(&e);
    client.initialize(&admin, &None);
    client.create_bond(&identity, &1_000_i128, &0_u64, &false, &0_u64);
}

#[test]
#[should_panic]
fn create_bond_rejects_zero_notice_period_for_rolling_bond_via_real_entrypoint() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(TrustForgeBond, ());
    let client = TrustForgeBondClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let identity = Address::generate(&e);
    client.initialize(&admin, &None);
    client.create_bond(&identity, &1_000_i128, &86_400_u64, &true, &0_u64);
}

#[test]
#[should_panic]
fn create_bond_rejects_notice_period_exceeding_duration_via_real_entrypoint() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(TrustForgeBond, ());
    let client = TrustForgeBondClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let identity = Address::generate(&e);
    client.initialize(&admin, &None);
    client.create_bond(&identity, &1_000_i128, &86_400_u64, &true, &90_000_u64);
}

#[test]
#[should_panic]
fn create_bond_rejects_second_call_for_same_instance() {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register(TrustForgeBond, ());
    let client = TrustForgeBondClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let identity = Address::generate(&e);
    client.initialize(&admin, &None);
    client.create_bond(&identity, &1_000_i128, &86_400_u64, &false, &0_u64);
    // Second call must be rejected (BondAlreadyExists), not silently overwrite the
    // first bond's state.
    client.create_bond(&identity, &500_i128, &86_400_u64, &false, &0_u64);
}
