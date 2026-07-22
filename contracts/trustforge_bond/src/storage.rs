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

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_token(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::Token)
        .expect("token not initialized")
}

pub fn set_token(e: &Env, token: &Address) {
    e.storage().instance().set(&DataKey::Token, token);
}

pub fn is_locked(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::Locked)
        .unwrap_or(false)
}

pub fn set_lock(e: &Env, locked: bool) {
    e.storage().instance().set(&DataKey::Locked, &locked);
}

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
