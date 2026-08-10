use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Admin,
    Token,
    Attester(Address),
    Attestation(u64),
    AttestationCounter,
    SubjectAttestations(Address),
    Locked,
    AcceptedTokens,
}

pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&DataKey::Admin)
}

// `set_admin`, `get_token`, `set_token`, `is_locked`, and `set_lock` (formerly here)
// were unused duplicates: `initialize()` sets DataKey::Admin directly, real token
// storage goes through `token_integration::{get,set}_token`, and the reentrancy
// lock lib.rs actually checks is a separate Symbol-keyed flag, not DataKey::Locked.
// Removed 2026-08-10 rather than `#[allow(dead_code)]`d, since nothing referenced
// them even transitively — see docs/ORPHANED_MODULES.md for the broader audit this
// was found during.

pub fn get_accepted_tokens(e: &Env) -> Vec<Address> {
    e.storage()
        .instance()
        .get(&DataKey::AcceptedTokens)
        .unwrap_or_else(|| Vec::new(e))
}

pub fn set_accepted_tokens(e: &Env, tokens: &Vec<Address>) {
    e.storage().instance().set(&DataKey::AcceptedTokens, tokens);
}

pub fn is_token_accepted(e: &Env, token: &Address) -> bool {
    let accepted = get_accepted_tokens(e);
    accepted.iter().any(|t| t == *token)
}
