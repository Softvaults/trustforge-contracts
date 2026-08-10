use soroban_sdk::{Address, Env, Symbol};
use trustforge_errors::ContractError;

use crate::DataKey;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PauseAction {
    Pause = 1,
    Unpause = 2,
}

fn require_admin_auth(e: &Env, admin: &Address) {
    let stored_admin: Address = e
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| e.panic_with_error(ContractError::NotInitialized));
    if stored_admin != *admin {
        e.panic_with_error(ContractError::NotAdmin);
    }
    admin.require_auth();
}

pub fn is_paused(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

// NOTE: no production entrypoint in lib.rs calls `require_not_paused` (or checks
// `is_paused` at all) before executing — `pause()`/`unpause()` currently only flip
// a storage flag with no effect on any other method. Flagged for Phase 4/5 review
// rather than silently allowed away — see docs/ORPHANED_MODULES.md.
#[allow(dead_code)]
pub fn require_not_paused(e: &Env) {
    if is_paused(e) {
        e.panic_with_error(ContractError::ContractPaused);
    }
}

// The multisig pause-signer/proposal flow below (set_pause_signer through
// execute_pause_proposal) is fully implemented but has no production caller —
// its only exerciser, test_pausable.rs, is one of the orphaned files documented
// in docs/ORPHANED_MODULES.md.
#[allow(dead_code)]
pub fn set_pause_signer(e: &Env, admin: &Address, signer: &Address, enabled: bool) {
    require_admin_auth(e, admin);

    let key = DataKey::PauseSigner(signer.clone());
    let old_enabled: bool = e.storage().instance().get(&key).unwrap_or(false);

    if enabled {
        if !old_enabled {
            e.storage().instance().set(&key, &true);
            let count: u32 = e
                .storage()
                .instance()
                .get(&DataKey::PauseSignerCount)
                .unwrap_or(0);
            e.storage()
                .instance()
                .set(&DataKey::PauseSignerCount, &count.saturating_add(1));
        }
    } else if old_enabled {
        e.storage().instance().remove(&key);
        let count: u32 = e
            .storage()
            .instance()
            .get(&DataKey::PauseSignerCount)
            .unwrap_or(0);
        e.storage()
            .instance()
            .set(&DataKey::PauseSignerCount, &count.saturating_sub(1));

        let threshold: u32 = e
            .storage()
            .instance()
            .get(&DataKey::PauseThreshold)
            .unwrap_or(0);
        let new_count: u32 = e
            .storage()
            .instance()
            .get(&DataKey::PauseSignerCount)
            .unwrap_or(0);
        if threshold > new_count {
            e.storage()
                .instance()
                .set(&DataKey::PauseThreshold, &new_count);
        }
    }

    // Emit old and new values for auditability
    e.events().publish(
        (Symbol::new(e, "pause_signer_set"), signer.clone()),
        (old_enabled, enabled),
    );
}

#[allow(dead_code)]
pub fn set_pause_threshold(e: &Env, admin: &Address, threshold: u32) {
    require_admin_auth(e, admin);
    let count: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseSignerCount)
        .unwrap_or(0);
    if threshold > count {
        e.panic_with_error(ContractError::PauseThresholdExceedsSignerCount);
    }
    let old_threshold: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseThreshold)
        .unwrap_or(0);
    e.storage()
        .instance()
        .set(&DataKey::PauseThreshold, &threshold);

    // Emit old and new values for auditability
    e.events().publish(
        (Symbol::new(e, "pause_threshold_set"),),
        (old_threshold, threshold),
    );
}

fn require_pause_signer(e: &Env, signer: &Address) {
    signer.require_auth();
    let ok: bool = e
        .storage()
        .instance()
        .get(&DataKey::PauseSigner(signer.clone()))
        .unwrap_or(false);
    if !ok {
        e.panic_with_error(ContractError::NotPauseSigner);
    }
}

fn next_proposal_id(e: &Env) -> u64 {
    let id: u64 = e
        .storage()
        .instance()
        .get(&DataKey::PauseProposalCounter)
        .unwrap_or(0);
    let next = id
        .checked_add(1)
        .unwrap_or_else(|| e.panic_with_error(ContractError::Overflow));
    e.storage()
        .instance()
        .set(&DataKey::PauseProposalCounter, &next);
    id
}

fn record_approval(e: &Env, proposal_id: u64, signer: &Address) {
    let approval_key = DataKey::PauseApproval(proposal_id, signer.clone());
    if e.storage().instance().has(&approval_key) {
        return;
    }
    e.storage().instance().set(&approval_key, &true);
    let count: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseApprovalCount(proposal_id))
        .unwrap_or(0);
    let new_count = count
        .checked_add(1)
        .unwrap_or_else(|| e.panic_with_error(ContractError::Overflow));
    e.storage()
        .instance()
        .set(&DataKey::PauseApprovalCount(proposal_id), &new_count);
}

pub fn pause(e: &Env, caller: &Address) -> Option<u64> {
    let threshold: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseThreshold)
        .unwrap_or(0);
    if threshold == 0 {
        require_admin_auth(e, caller);
        do_pause(e, None);
        None
    } else {
        propose_action(e, caller, PauseAction::Pause)
    }
}

pub fn unpause(e: &Env, caller: &Address) -> Option<u64> {
    let threshold: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseThreshold)
        .unwrap_or(0);
    if threshold == 0 {
        require_admin_auth(e, caller);
        do_unpause(e, None);
        None
    } else {
        propose_action(e, caller, PauseAction::Unpause)
    }
}

fn propose_action(e: &Env, caller: &Address, action: PauseAction) -> Option<u64> {
    require_pause_signer(e, caller);

    let id = next_proposal_id(e);
    e.storage()
        .instance()
        .set(&DataKey::PauseProposal(id), &(action as u32));
    e.storage()
        .instance()
        .set(&DataKey::PauseApprovalCount(id), &0_u32);

    record_approval(e, id, caller);

    e.events()
        .publish((Symbol::new(e, "pause_proposed"), id), action as u32);

    Some(id)
}

#[allow(dead_code)]
pub fn approve_pause_proposal(e: &Env, signer: &Address, proposal_id: u64) {
    require_pause_signer(e, signer);

    let _action: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseProposal(proposal_id))
        .unwrap_or_else(|| e.panic_with_error(ContractError::PauseProposalNotFound));

    record_approval(e, proposal_id, signer);

    e.events().publish(
        (Symbol::new(e, "pause_approved"), proposal_id),
        signer.clone(),
    );
}

#[allow(dead_code)]
pub fn execute_pause_proposal(e: &Env, proposal_id: u64) {
    let action: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseProposal(proposal_id))
        .unwrap_or_else(|| e.panic_with_error(ContractError::PauseProposalNotFound));

    let threshold: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseThreshold)
        .unwrap_or(0);
    let approvals: u32 = e
        .storage()
        .instance()
        .get(&DataKey::PauseApprovalCount(proposal_id))
        .unwrap_or(0);

    if approvals < threshold {
        e.panic_with_error(ContractError::InsufficientSignatures);
    }

    match action {
        1 => do_pause(e, Some(proposal_id)),
        2 => do_unpause(e, Some(proposal_id)),
        _ => e.panic_with_error(ContractError::InvalidPauseAction),
    }

    e.storage()
        .instance()
        .remove(&DataKey::PauseProposal(proposal_id));
}

fn do_pause(e: &Env, proposal_id: Option<u64>) {
    e.storage().instance().set(&DataKey::Paused, &true);
    e.events().publish((Symbol::new(e, "paused"),), proposal_id);
}

fn do_unpause(e: &Env, proposal_id: Option<u64>) {
    e.storage().instance().set(&DataKey::Paused, &false);
    e.events()
        .publish((Symbol::new(e, "unpaused"),), proposal_id);
}
