#![no_std]
#![allow(
    deprecated,
    unused_imports,
    unused_variables,
    dead_code,
    unused_assignments,
    unused_mut,
    mismatched_lifetime_syntaxes,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::restriction
)]

use soroban_sdk::{contracterror, contracttype};
/// Project-wide version constant.
pub const VERSION: &str = "0.1.0";

/// @title  Role
/// @notice Coarse admin/user classification returned by read-only role checks
///         (e.g. `is_admin`) across contracts. Crosses the contract ABI
///         boundary, so it is a `#[contracttype]`.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

/// @title  ErrorCategory
/// @notice Groups errors by domain for monitoring, alerting, and dashboards.
/// @dev    Off-chain consumers should switch on this value first, then on the
///         specific `ContractError` code for fine-grained handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Contract setup and initialization errors (codes 1-99).
    Initialization,
    /// Caller identity and permission errors (codes 100-199).
    Authorization,
    /// Bond lifecycle errors (codes 200-299).
    Bond,
    /// Attestation errors (codes 300-399).
    Attestation,
    /// Registry identity/contract errors (codes 400-499).
    Registry,
    /// Delegation errors (codes 500-599).
    Delegation,
    /// Treasury proposal and balance errors (codes 600-699).
    Treasury,
    /// Safe-math errors (codes 700-799).
    Arithmetic,
}

/// @title  ContractError
/// @notice Canonical error enum shared by all TrustForge smart contracts.
/// @dev    Codes are wire-stable. Never renumber a variant after deployment.
///         Append new variants at the end of their category block only.
///         Use the ErrorExt trait to retrieve the category and description.
///
/// Error Code Layout:
///   1  -  99  : Initialization
///   100 - 199 : Authorization
///   200 - 299 : Bond
///   300 - 399 : Attestation
///   400 - 499 : Registry
///   500 - 599 : Delegation
///   600 - 699 : Treasury
///   700 - 799 : Arithmetic
// Keep conversions generated, but do not export this utility enum as contract
// spec metadata. The shared enum has more variants than Soroban's current
// exported error-enum case vector limit supports, and this crate is not a
// deployed contract interface.
#[contracterror(export = false)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    // --- Initialization (1-99) ---
    /// Contract has not been initialized yet.
    /// Replaces: panic!("not initialized")
    /// Contracts: bond, registry, delegation, treasury
    /// Wire-stable: do not renumber this error code.
    NotInitialized = 1,

    /// Contract has already been initialized and cannot be re-initialized.
    /// Replaces: panic!("already initialized")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    AlreadyInitialized = 2,

    // --- Authorization (100-199) ---
    /// Caller is not the admin.
    /// Replaces: panic!("not admin")
    /// Contracts: bond, registry, delegation
    /// Wire-stable: do not renumber this error code.
    NotAdmin = 100,

    /// Caller is not the bond owner.
    /// Replaces: panic!("not bond owner")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NotBondOwner = 101,

    /// Caller is not an authorized attester for this bond.
    /// Replaces: panic!("unauthorized attester")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UnauthorizedAttester = 102,

    /// Caller is not the original attester who created the attestation.
    /// Replaces: panic!("only original attester can revoke")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NotOriginalAttester = 103,

    /// Caller is not a registered multi-sig signer.
    /// Replaces: panic!("only signer can propose withdrawal")
    ///           panic!("only signer can approve")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    NotSigner = 104,

    /// Caller is neither the admin nor an authorized depositor.
    /// Replaces: panic!("only admin or authorized depositor can receive_fee")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    UnauthorizedDepositor = 105,

    /// Contract is currently paused and does not allow state mutations.
    /// Replaces: panic!("contract is paused")
    /// Contracts: bond, registry, treasury
    /// Wire-stable: do not renumber this error code.
    ContractPaused = 106,

    /// Pause proposal action value is invalid.
    /// Replaces: panic!("invalid pause action")
    /// Contracts: registry, treasury
    /// Wire-stable: do not renumber this error code.
    InvalidPauseAction = 107,

    /// Not enough approvals to execute the proposal.
    /// Replaces: panic!("insufficient signatures to execute"), panic!("insufficient approvals")
    /// Contracts: multisig, treasury
    /// Wire-stable: do not renumber this error code.
    InsufficientSignatures = 108,

    /// The target admin is currently suspended (suspended_until > now).
    /// Used by suspend_admin when `until_ts` is not strictly in the future,
    /// and by callers that detect a suspended admin attempting a privileged
    /// action.
    /// Contracts: admin
    /// Wire-stable: do not renumber this error code.
    AdminSuspended = 113,

    // --- Bond (200-299) ---
    /// No bond exists for the given address or key.
    /// Replaces: panic!("no bond")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    BondNotFound = 200,

    /// Bond is not in the active state required for this operation.
    /// Replaces: panic!("bond not active")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    BondNotActive = 201,

    /// Caller balance is insufficient for the requested withdrawal.
    /// Replaces: panic!("insufficient balance for withdrawal")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InsufficientBalance = 202,

    /// The slash amount exceeds the bonded amount.
    /// Replaces: panic!("slashed amount exceeds bonded amount")
    ///           panic!("slash exceeds bond")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    SlashExceedsBond = 203,
    /// Storage cap for attestations or slash history reached.
    /// Replaces: panic!("storage cap reached")
    StorageCapReached = 224,

    /// Bond lock-up period has not yet expired.
    /// Replaces: panic!("use withdraw for post lock-up")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    LockupNotExpired = 204,

    /// Operation requires a rolling bond but this bond is not rolling.
    /// Replaces: panic!("not a rolling bond")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NotRollingBond = 205,

    /// A withdrawal has already been requested for this bond.
    /// Replaces: panic!("withdrawal already requested")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    WithdrawalAlreadyRequested = 206,

    /// Reentrancy was detected; the call is rejected.
    /// Replaces: panic!("reentrancy detected")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    ReentrancyDetected = 207,

    /// Nonce is invalid - either replayed or out of order.
    /// Replaces: panic!("invalid nonce: replay or out-of-order")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InvalidNonce = 208,

    /// Signature/operation deadline has passed (now > deadline + grace).
    /// Contracts: bond, delegation
    SignatureExpired = 222,

    /// Attester stake would go negative, which is not permitted.
    /// Replaces: panic!("attester stake cannot be negative")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NegativeStake = 209,

    /// Early-exit configuration has not been set for this bond.
    /// Replaces: panic!("early exit config not set")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    EarlyExitConfigNotSet = 210,

    /// Penalty basis-points value must be in the range 0-10000.
    /// Replaces: panic!("penalty_bps must be <= 10000")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InvalidPenaltyBps = 211,

    /// Resulting leverage exceeds the configured maximum.
    /// Replaces: panic!("leverage exceeds maximum")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    LeverageExceeded = 212,

    /// Token transfer resulted in different amount than requested (fee-on-transfer tokens).
    /// Replaces: panic!("unsupported token: transfer amount mismatch")
    /// Contracts: bond, dispute_resolution, fixed_duration_bond
    /// Wire-stable: do not renumber this error code.
    UnsupportedToken = 213,

    /// Token decimals are outside the supported range used for normalization.
    /// Triggered by token ingress when a configured token reports unsupported decimals.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UnsupportedDecimals = 229,

    /// Bond amount must be strictly positive (> 0).
    /// Triggered by: create_bond called with amount <= 0
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InvalidBondAmount = 214,

    /// Bond duration must be strictly positive (> 0).
    /// Triggered by: create_bond called with duration == 0
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InvalidBondDuration = 215,

    /// Rolling-bond notice_period_duration must be > 0 and <= duration.
    /// Triggered by: create_bond called with is_rolling=true and notice_period_duration == 0
    ///               or notice_period_duration > duration
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InvalidNoticePeriod = 216,

    /// Bond already exists for this identity.
    /// Triggered by: create_bond called for an identity that already has an active bond
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    BondAlreadyExists = 217,

    /// Token address is not in the set of accepted tokens.
    /// Triggered by: initialize called with a token not in the accepted tokens set
    /// Contracts: bond
    /// Reassigned from a colliding 218 (originally shared with `InvariantViolation`,
    /// which is wire-pinned by `tests/error_codes_wire.rs`) during the
    /// duplicate-discriminant cleanup; never deployed at 218.
    UnauthorizedToken = 230,
    /// Post-write invariant self-check detected bond or attestation accounting drift.
    /// Triggered by: `invariants::assert_self_consistent` after a bond-module write
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InvariantViolation = 218,

    /// Slash treasury address has not been configured.
    /// Triggered by: `slash_bond` when `DataKey::SlashTreasury` is absent.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    TreasuryNotConfigured = 223,

    /// Pagination cursor is out of range (cursor >= registry_slots).
    /// Triggered by: `scan_liquidation_candidates` when the supplied cursor
    /// equals or exceeds the current registry slot count. Accepting
    /// cursor == registry_slots would silently return a done=true result,
    /// allowing a malicious keeper to synthesize a completed-scan response
    /// without actually scanning any positions.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    CursorOutOfRange = 226,

    /// Batch input exceeds the maximum allowed size constant.
    /// Prevents a single transaction from exhausting CPU/ledger budgets.
    /// Replaces: panic!("batch too large")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    BatchTooLarge = 227,

    /// Batch input is empty (len == 0) when at least one item is required.
    /// Replaces: panic!("empty batch")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    EmptyBatch = 228,

    /// Idempotency key has already been used for this operation.
    /// Triggered by: duplicate submissions with the same idempotency key
    /// (actor, operation, salt) arriving via webhook retries.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    DuplicateIdempotencyKey = 231,

    /// A rolling-bond withdrawal was attempted before `request_withdrawal`
    /// (or equivalent) was called to start the notice period.
    /// Replaces: panic!("withdrawal not requested")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    WithdrawalNotRequested = 232,

    /// A rolling-bond withdrawal was attempted before the notice period
    /// (measured from the withdrawal request) has elapsed.
    /// Replaces: panic!("notice period not elapsed")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NoticePeriodNotElapsed = 233,

    /// Bond does not meet either liquidation eligibility condition: it is
    /// not fully slashed, and (for non-rolling bonds) its lock-up has not
    /// expired without renewal.
    /// Replaces: panic!("bond is not eligible for liquidation: ...")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    BondNotEligibleForLiquidation = 234,

    /// `add_pending_claim` was called with a non-positive amount.
    /// Replaces: panic!("claim amount must be positive")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    ClaimAmountMustBePositive = 235,

    /// `process_claims` was called for a user with no pending claims at all.
    /// Replaces: panic!("no pending claims")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NoPendingClaims = 236,

    /// `process_claims` found pending claims but none were payable this call
    /// (all filtered out, already processed, or expired).
    /// Replaces: panic!("no valid claims to process")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NoValidClaimsToProcess = 237,

    /// No claim exists with the given claim ID.
    /// Replaces: panic!("claim not found")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    ClaimNotFound = 238,

    /// The contract's bond token (`DataKey::BondToken`) has not been
    /// configured, so a token-moving operation cannot proceed.
    /// Replaces: expect("token not configured")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    BondTokenNotConfigured = 239,

    /// A governance-controlled protocol parameter setter (`parameters`
    /// module) received a value outside that parameter's configured
    /// min/max bounds.
    /// Replaces: panic!("{parameter}_out_of_bounds")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    ParameterOutOfBounds = 240,

    /// The `GovernanceApproval` passed to a parameter setter was signed by
    /// an address other than the caller.
    /// Replaces: panic!("governance approver mismatch")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    GovernanceApproverMismatch = 241,

    /// The `GovernanceApproval` passed to a parameter setter has a non-zero
    /// `expires_at` that has already passed.
    /// Replaces: panic!("governance approval expired")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    GovernanceApprovalExpired = 242,

    /// The `GovernanceApproval` passed to a parameter setter was scoped to
    /// a different parameter category than the one being set.
    /// Replaces: panic!("governance approval category mismatch")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    GovernanceApprovalCategoryMismatch = 243,

    /// `set_usdc_token` was called with a network label other than
    /// `"mainnet"` or `"testnet"`.
    /// Replaces: panic!("unsupported stellar network")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UnsupportedNetwork = 244,

    /// A token-moving helper (`transfer_into_contract`/`transfer_from_contract`)
    /// was called with a negative amount.
    /// Replaces: panic!("amount must be non-negative")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NegativeTransferAmount = 245,

    /// The token owner's allowance for the contract is less than the
    /// amount a transfer-in operation requires.
    /// Replaces: panic!("insufficient token allowance")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InsufficientAllowance = 246,

    /// The underlying `try_transfer`/`try_transfer_from` call to the token
    /// contract returned an error.
    /// Replaces: panic!("token transfer failed")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    TokenTransferFailed = 247,

    /// `slash_bond`/`unslash_bond` was called with a negative amount.
    /// Replaces: panic!("slash amount must be non-negative")
    ///           panic!("unslash amount must be non-negative")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NegativeSlashAmount = 248,

    /// `unslash_bond`'s amount is greater than the bond's currently
    /// recorded `slashed_amount`, which would drive it below zero.
    /// Replaces: panic!("unslashing would reduce below 0")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UnslashExceedsSlashedAmount = 249,

    // --- Attestation (300-399) ---
    /// An attestation already exists from this attester for this bond.
    /// Replaces: panic!("duplicate attestation")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    DuplicateAttestation = 300,

    /// No attestation was found for the given key.
    /// Replaces: panic!("attestation not found")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    AttestationNotFound = 301,

    /// Attestation has already been revoked.
    /// Replaces: panic!("attestation already revoked")
    /// Contracts: bond, delegation
    /// Wire-stable: do not renumber this error code.
    AttestationAlreadyRevoked = 302,

    /// Attestation weight must be a positive value.
    /// Replaces: panic!("attestation weight must be positive")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    InvalidAttestationWeight = 303,

    /// Attestation weight exceeds the configured maximum.
    /// Replaces: panic!("attestation weight exceeds maximum")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    AttestationWeightExceedsMax = 304,

    /// The same attester appears more than once within a single batch
    /// submission.
    /// Replaces: panic!("duplicate attester in batch")
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    DuplicateAttesterInBatch = 305,

    // --- Registry (400-499) ---
    /// Identity has already been registered in the registry.
    /// Replaces: panic!("identity already registered")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    IdentityAlreadyRegistered = 400,

    /// Bond contract address has already been registered.
    /// Replaces: panic!("bond contract already registered")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    BondContractAlreadyRegistered = 401,

    /// Identity is not registered in the registry.
    /// Replaces: panic!("identity not registered")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    IdentityNotRegistered = 402,

    /// Bond contract is not registered in the registry.
    /// Replaces: panic!("bond contract not registered")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    BondContractNotRegistered = 403,

    /// Identity or bond contract is already in the deactivated state.
    /// Replaces: panic!("already deactivated")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    AlreadyDeactivated = 404,

    /// Identity or bond contract is already in the active state.
    /// Replaces: panic!("already active")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    AlreadyActive = 405,

    /// Provided contract address is not a deployed contract.
    /// Replaces: panic!("invalid contract address")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    InvalidContractAddress = 406,

    /// Contract code hash verification failed during trustless registration.
    /// The calling contract's WASM code hash does not match the expected bond code hash.
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    ContractCodeVerificationFailed = 407,

    /// Bond contract does not support required interface.
    /// Replaces: panic!("bond contract does not support required interface")
    /// Contracts: registry
    /// Wire-stable: do not renumber this error code.
    UnsupportedInterface = 408,
    // --- Delegation (500-599) ---
    /// Delegation expiry timestamp must be in the future.
    /// Replaces: panic!("expiry must be in the future")
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    ExpiryInPast = 500,

    /// No delegation record was found for the given key.
    /// Replaces: panic!("delegation not found")
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    DelegationNotFound = 501,

    /// Delegation has already been revoked.
    /// Replaces: panic!("already revoked")
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    AlreadyRevoked = 502,

    /// Delegation expiry timestamp exceeds the maximum allowed lifetime.
    /// Triggered by: expires_at > now + MAX_DELEGATION_DURATION
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    DelegationExpiryTooLong = 503,
    // Note: DomainMismatch (218), OwnerMismatch (219), TargetMismatch (220),
    // ContractIdMismatch (221), and SignatureExpired (222) are shared Bond/Delegation
    // variants defined in the Bond section above.
    /// Unknown or unsupported signature scheme tag.
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    UnknownScheme = 504,

    /// Verifier already registered for the given scheme tag.
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    VerifierAlreadyRegistered = 505,

    /// No verifier registered for the given scheme tag.
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    VerifierNotRegistered = 506,

    /// Signature verification failed for the given scheme and payload.
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    VerificationFailed = 507,

    /// Post-expiry revocation attempted outside the configured grace window.
    /// Triggered when `revocation_grace_period > 0` and
    /// `now > expires_at + revocation_grace_period`.
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    RevocationGraceExpired = 508,

    /// Cleanup attempted on a delegation that is not expired yet.
    /// Contracts: delegation
    /// Wire-stable: do not renumber this error code.
    DelegationNotExpired = 509,

    // --- Shared Bond/Delegation payload mismatch errors (218-221) ---
    // Wire-stable: codes documented in the note above; kept distinct from the
    // delegation scheme/verifier errors (504-507).
    DomainMismatch = 225,
    OwnerMismatch = 219,
    TargetMismatch = 220,
    ContractIdMismatch = 221,

    // --- Admin Transfer (109-112) ---
    /// No pending admin transfer exists.
    NoPendingAdmin = 109,

    /// Proposed admin is the zero/identity address.
    InvalidAdminAddress = 110,

    /// Proposed admin is the same as the current admin.
    AdminUnchanged = 111,

    /// Timelock delay has not yet elapsed.
    TimelockNotReady = 112,

    /// Emergency drain is not permitted: contract must be paused and timelock window must have elapsed.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    /// Reassigned from a colliding 113 (originally shared with `AdminSuspended`)
    /// during the duplicate-discriminant cleanup; never deployed at 113.
    EmergencyDrainNotPermitted = 114,

    // --- Upgrade Authorization (115-135) ---
    // Distinct from the regular admin-transfer codes above (109-112): these
    // govern the separate upgrade-admin/upgrader-role system in
    // `trustforge_bond::upgrade_auth`, kept as its own error family so an
    // off-chain consumer can tell "wrong admin for a normal admin action"
    // apart from "wrong admin for an upgrade action."
    /// Upgrade authorization has already been initialized for this contract.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeAuthAlreadyInitialized = 115,

    /// Upgrade authorization has not been initialized for this contract.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeAuthNotInitialized = 116,

    /// This address already holds an active upgrade authorization; revoke it
    /// before granting a new one.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    AlreadyAuthorizedUpgrader = 117,

    /// An upgrade admin may not grant themselves a role equal to or higher
    /// than their current one.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    CannotGrantHigherUpgradeRoleToSelf = 118,

    /// This address does not hold an upgrade authorization (or it was
    /// already revoked/never granted).
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NotAuthorizedUpgrader = 119,

    /// Cannot revoke the last remaining address with the Upgrader role —
    /// doing so would leave the contract with no path to authorize a future
    /// upgrade.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    CannotRevokeLastUpgrader = 120,

    /// Caller is not the upgrade admin.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NotUpgradeAdmin = 121,

    /// Caller is not authorized to perform an upgrade (not an active,
    /// unexpired Upgrader).
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UnauthorizedUpgrade = 122,

    /// Caller's upgrade authorization exists but is not currently active.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeAuthorizationNotActive = 123,

    /// Caller's upgrade authorization has expired.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeAuthorizationExpired = 124,

    /// No upgrade proposal exists with the given ID.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeProposalNotFound = 125,

    /// The upgrade proposal is not in `Pending` status (already approved,
    /// executed, rejected, or expired).
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeProposalNotPending = 126,

    /// This address has already approved the upgrade proposal.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    AlreadyApprovedUpgradeProposal = 127,

    /// The upgrade proposal has not accumulated enough approvals to execute.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeProposalNotApproved = 128,

    /// The implementation address passed to `execute_upgrade` does not match
    /// the one recorded on its approved proposal.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    UpgradeImplementationMismatch = 129,

    /// No current implementation address is recorded for this contract.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NoCurrentImplementation = 130,

    /// The proposed new implementation is identical to the current one.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    SameImplementation = 131,

    /// The new upgrade admin must differ from the current upgrade admin.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NewUpgradeAdminMustDiffer = 132,

    /// No pending upgrade-admin transfer exists to accept.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NoPendingUpgradeAdmin = 133,

    /// Caller is not the address nominated as the pending upgrade admin.
    /// Contracts: bond
    /// Wire-stable: do not renumber this error code.
    NotPendingUpgradeAdmin = 134,

    // --- Treasury (600-699) ---
    /// Amount argument must be strictly positive (> 0).
    /// Replaces: panic!("amount must be positive")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    AmountMustBePositive = 600,

    /// Approval threshold cannot exceed the current number of signers.
    /// Replaces: panic!("threshold cannot exceed signer count")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    ThresholdExceedsSigners = 601,

    /// Treasury balance is insufficient for the requested withdrawal.
    /// Replaces: panic!("insufficient treasury balance")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    InsufficientTreasuryBalance = 602,

    /// Withdrawal proposal was not found for the given id.
    /// Replaces: panic!("proposal not found")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    ProposalNotFound = 603,

    /// Withdrawal proposal has already been executed.
    /// Replaces: panic!("proposal already executed")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    ProposalAlreadyExecuted = 604,

    /// Proposal does not yet have enough approvals to execute.
    /// Replaces: panic!("insufficient approvals to execute")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    InsufficientApprovals = 605,

    /// Flashloan callback returned an invalid magic value.
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    InvalidFlashLoanCallback = 606,

    /// Flashloan principal plus fee was not fully repaid.
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    FlashLoanRepaymentFailed = 607,

    /// Withdrawal proposal has expired and can no longer be approved or executed.
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    ProposalExpired = 608,

    /// Settled withdrawal amount fell below the caller's `min_amount_out`
    /// slippage bound. Distinct from `InsufficientTreasuryBalance`: the treasury
    /// had funds, but the realized amount tripped the caller's slippage guard.
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    SlippageExceeded = 609,

    // --- Arithmetic (700-799) ---
    /// Integer overflow detected during a checked arithmetic operation.
    /// Replaces: .expect("... overflow")
    /// Contracts: bond, treasury
    /// Wire-stable: do not renumber this error code.
    Overflow = 700,

    /// Integer underflow detected during a checked arithmetic operation.
    /// Replaces: .expect("... underflow")
    /// Contracts: treasury
    /// Wire-stable: do not renumber this error code.
    Underflow = 701,

    /// Division (or remainder) by a zero denominator was attempted.
    /// Replaces: panic!("...") in the safe-math div/ceil_div helpers when `b == 0`.
    /// Contracts: math, bond
    /// Wire-stable: do not renumber this error code.
    DivisionByZero = 702,
}

/// @title  ErrorExt
/// @notice Provides category(), description(), and is_recoverable() on every
///         ContractError variant.
/// @dev    Use this for structured logging, monitoring, and off-chain display.
///
/// `is_recoverable()` classifies an error as recoverable when the
/// caller can fix their input or wait for state to change and retry
/// the same kind of operation successfully (e.g. `AlreadyInitialized`,
/// `LockupNotExpired`, `InsufficientSignatures`). It returns `false`
/// for **fatal** errors that indicate either a code-level fault
/// (`Overflow`, `Underflow`, `InvariantViolation`), a security halt
/// (`ReentrancyDetected`), a cryptographic failure
/// (`VerificationFailed`), or a payload binding mismatch
/// (`DomainMismatch`, `OwnerMismatch`, `TargetMismatch`,
/// `ContractIdMismatch`). Off-chain clients (indexers, admin CLI,
/// alerting) should use this signal to decide between
/// "retry/ignore" vs "alert/halt".
///
/// `is_recoverable()` is metadata only: it does not panic, does not
/// allocate, and does not touch storage. It does not change any
/// wire codes, categories, or description strings.
///
/// New `ContractError` variants must be added with an explicit
/// classification - the matching `impl` is exhaustive and the test
/// suite forces a decision for every variant (see `test_is_recoverable_exhaustive`).
pub trait ErrorExt {
    /// @return The ErrorCategory bucket this error belongs to.
    fn category(&self) -> ErrorCategory;

    /// @return A static string description safe for logging or display.
    fn description(&self) -> &'static str;

    /// @return `true` if a caller can fix their input or wait for state to
    ///         change and retry the same operation successfully;
    ///         `false` if the error indicates a code-level fault, security
    ///         halt, or payload-binding mismatch where blind retry will not
    ///         help.
    fn is_recoverable(&self) -> bool;
}

impl ErrorExt for ContractError {
    fn category(&self) -> ErrorCategory {
        match self {
            ContractError::NotInitialized | ContractError::AlreadyInitialized => {
                ErrorCategory::Initialization
            }
            ContractError::NotAdmin
            | ContractError::NotBondOwner
            | ContractError::UnauthorizedAttester
            | ContractError::NotOriginalAttester
            | ContractError::NotSigner
            | ContractError::UnauthorizedDepositor
            | ContractError::ContractPaused
            | ContractError::InvalidPauseAction
            | ContractError::InsufficientSignatures
            | ContractError::AdminSuspended => ErrorCategory::Authorization,

            ContractError::BondNotFound
            | ContractError::BondNotActive
            | ContractError::InsufficientBalance
            | ContractError::SlashExceedsBond
            | ContractError::LockupNotExpired
            | ContractError::NotRollingBond
            | ContractError::WithdrawalAlreadyRequested
            | ContractError::ReentrancyDetected
            | ContractError::InvalidNonce
            | ContractError::SignatureExpired
            | ContractError::NegativeStake
            | ContractError::EarlyExitConfigNotSet
            | ContractError::InvalidPenaltyBps
            | ContractError::LeverageExceeded
            | ContractError::UnsupportedToken
            | ContractError::UnsupportedDecimals
            | ContractError::InvalidBondAmount
            | ContractError::InvalidBondDuration
            | ContractError::InvalidNoticePeriod
            | ContractError::BondAlreadyExists
            | ContractError::UnauthorizedToken => ErrorCategory::Bond,
            ContractError::StorageCapReached
            | ContractError::TreasuryNotConfigured
            | ContractError::CursorOutOfRange
            | ContractError::BatchTooLarge
            | ContractError::EmptyBatch
            | ContractError::DuplicateIdempotencyKey
            | ContractError::InvariantViolation
            | ContractError::WithdrawalNotRequested
            | ContractError::NoticePeriodNotElapsed
            | ContractError::BondNotEligibleForLiquidation
            | ContractError::ClaimAmountMustBePositive
            | ContractError::NoPendingClaims
            | ContractError::NoValidClaimsToProcess
            | ContractError::ClaimNotFound
            | ContractError::BondTokenNotConfigured
            | ContractError::ParameterOutOfBounds
            | ContractError::GovernanceApproverMismatch
            | ContractError::GovernanceApprovalExpired
            | ContractError::GovernanceApprovalCategoryMismatch
            | ContractError::UnsupportedNetwork
            | ContractError::NegativeTransferAmount
            | ContractError::InsufficientAllowance
            | ContractError::TokenTransferFailed
            | ContractError::NegativeSlashAmount
            | ContractError::UnslashExceedsSlashedAmount => ErrorCategory::Bond,

            ContractError::DuplicateAttestation
            | ContractError::AttestationNotFound
            | ContractError::AttestationAlreadyRevoked
            | ContractError::InvalidAttestationWeight
            | ContractError::AttestationWeightExceedsMax
            | ContractError::DuplicateAttesterInBatch => ErrorCategory::Attestation,

            ContractError::IdentityAlreadyRegistered
            | ContractError::BondContractAlreadyRegistered
            | ContractError::IdentityNotRegistered
            | ContractError::BondContractNotRegistered
            | ContractError::AlreadyDeactivated
            | ContractError::AlreadyActive
            | ContractError::InvalidContractAddress
            | ContractError::ContractCodeVerificationFailed
            | ContractError::UnsupportedInterface => ErrorCategory::Registry,

            ContractError::ExpiryInPast
            | ContractError::DelegationNotFound
            | ContractError::AlreadyRevoked
            | ContractError::DelegationExpiryTooLong
            | ContractError::UnknownScheme
            | ContractError::VerifierAlreadyRegistered
            | ContractError::VerifierNotRegistered
            | ContractError::VerificationFailed
            | ContractError::RevocationGraceExpired
            | ContractError::DelegationNotExpired => ErrorCategory::Delegation,

            ContractError::AmountMustBePositive
            | ContractError::ThresholdExceedsSigners
            | ContractError::InsufficientTreasuryBalance
            | ContractError::ProposalNotFound
            | ContractError::ProposalAlreadyExecuted
            | ContractError::InsufficientApprovals
            | ContractError::InvalidFlashLoanCallback
            | ContractError::FlashLoanRepaymentFailed
            | ContractError::ProposalExpired
            | ContractError::SlippageExceeded => ErrorCategory::Treasury,

            ContractError::Overflow | ContractError::Underflow | ContractError::DivisionByZero => {
                ErrorCategory::Arithmetic
            }
            ContractError::NoPendingAdmin
            | ContractError::InvalidAdminAddress
            | ContractError::AdminUnchanged
            | ContractError::TimelockNotReady
            | ContractError::EmergencyDrainNotPermitted
            | ContractError::UpgradeAuthAlreadyInitialized
            | ContractError::UpgradeAuthNotInitialized
            | ContractError::AlreadyAuthorizedUpgrader
            | ContractError::CannotGrantHigherUpgradeRoleToSelf
            | ContractError::NotAuthorizedUpgrader
            | ContractError::CannotRevokeLastUpgrader
            | ContractError::NotUpgradeAdmin
            | ContractError::UnauthorizedUpgrade
            | ContractError::UpgradeAuthorizationNotActive
            | ContractError::UpgradeAuthorizationExpired
            | ContractError::UpgradeProposalNotFound
            | ContractError::UpgradeProposalNotPending
            | ContractError::AlreadyApprovedUpgradeProposal
            | ContractError::UpgradeProposalNotApproved
            | ContractError::UpgradeImplementationMismatch
            | ContractError::NoCurrentImplementation
            | ContractError::SameImplementation
            | ContractError::NewUpgradeAdminMustDiffer
            | ContractError::NoPendingUpgradeAdmin
            | ContractError::NotPendingUpgradeAdmin => ErrorCategory::Authorization,
            ContractError::DomainMismatch
            | ContractError::OwnerMismatch
            | ContractError::TargetMismatch
            | ContractError::ContractIdMismatch => ErrorCategory::Delegation,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ContractError::NotInitialized => "Contract has not been initialized",
            ContractError::AlreadyInitialized => "Contract has already been initialized",
            ContractError::NotAdmin => "Caller is not the admin",
            ContractError::NotBondOwner => "Caller is not the bond owner",
            ContractError::UnauthorizedAttester => "Caller is not an authorized attester",
            ContractError::NotOriginalAttester => "Only the original attester can revoke",
            ContractError::NotSigner => "Caller is not a registered multi-sig signer",
            ContractError::UnauthorizedDepositor => {
                "Caller is neither admin nor an authorized depositor"
            }
            ContractError::ContractPaused => "Contract is paused",
            ContractError::InvalidPauseAction => "Pause proposal action is invalid",
            ContractError::InsufficientSignatures => "Not enough approvals to execute proposal",
            ContractError::AdminSuspended => "Admin is currently suspended",
            ContractError::BondNotFound => "No bond found for the given key",
            ContractError::BondNotActive => "Bond is not in an active state",
            ContractError::InsufficientBalance => "Insufficient balance for withdrawal",
            ContractError::SlashExceedsBond => "Slash amount exceeds the bonded amount",
            ContractError::LockupNotExpired => "Lock-up period has not yet expired",
            ContractError::NotRollingBond => "Bond is not configured as a rolling bond",
            ContractError::WithdrawalAlreadyRequested => {
                "A withdrawal has already been requested for this bond"
            }
            ContractError::ReentrancyDetected => "Reentrancy detected; call rejected",
            ContractError::InvalidNonce => "Nonce is replayed or out of order",
            ContractError::SignatureExpired => "Signature/operation deadline has passed",
            ContractError::NegativeStake => "Attester stake cannot be negative",
            ContractError::EarlyExitConfigNotSet => {
                "Early-exit configuration has not been set for this bond"
            }
            ContractError::InvalidPenaltyBps => "Penalty bps must be in range 0-10000",
            ContractError::LeverageExceeded => "Resulting leverage exceeds the configured maximum",
            ContractError::UnsupportedToken => "Token transfer resulted in different amount than requested (fee-on-transfer tokens not supported)",
            ContractError::UnsupportedDecimals => "Token decimals are outside the supported normalization range",
            ContractError::InvalidBondAmount => "Bond amount must be strictly positive (> 0)",
            ContractError::InvalidBondDuration => "Bond duration must be strictly positive (> 0)",
            ContractError::InvalidNoticePeriod => "Rolling-bond notice_period_duration must be > 0 and <= duration",
            ContractError::BondAlreadyExists => "Bond already exists for this identity",
            ContractError::UnauthorizedToken => "Token address is not in the set of accepted tokens",
            ContractError::StorageCapReached => "Storage cap for attestations or slash history reached",
            ContractError::TreasuryNotConfigured => "Slash treasury address has not been configured",
            ContractError::CursorOutOfRange => "Pagination cursor is out of range (cursor >= registry_slots)",
            ContractError::BatchTooLarge => "Batch input exceeds the maximum allowed size",
            ContractError::EmptyBatch => "Batch input is empty; at least one item is required",
            ContractError::DuplicateIdempotencyKey => "Idempotency key has already been used for this operation",
            ContractError::InvariantViolation => {
                "Bond storage drift detected; bonded/slashed or attestation counters inconsistent"
            }
            ContractError::WithdrawalNotRequested => {
                "Rolling-bond withdrawal was not requested; call request_withdrawal first"
            }
            ContractError::NoticePeriodNotElapsed => "Rolling-bond notice period has not yet elapsed",
            ContractError::BondNotEligibleForLiquidation => {
                "Bond must be fully slashed, or expired and non-rolling, to be liquidated"
            }
            ContractError::ClaimAmountMustBePositive => "Claim amount must be greater than zero",
            ContractError::NoPendingClaims => "User has no pending claims",
            ContractError::NoValidClaimsToProcess => {
                "User has pending claims, but none are currently payable"
            }
            ContractError::ClaimNotFound => "No claim exists with the given claim ID",
            ContractError::BondTokenNotConfigured => "Bond token has not been configured",
            ContractError::ParameterOutOfBounds => {
                "Parameter value is outside its configured min/max bounds"
            }
            ContractError::GovernanceApproverMismatch => {
                "Governance approval was signed by a different address than the caller"
            }
            ContractError::GovernanceApprovalExpired => "Governance approval has expired",
            ContractError::GovernanceApprovalCategoryMismatch => {
                "Governance approval category does not match the parameter being set"
            }
            ContractError::UnsupportedNetwork => {
                "Network label must be \"mainnet\" or \"testnet\""
            }
            ContractError::NegativeTransferAmount => "Transfer amount must be non-negative",
            ContractError::InsufficientAllowance => {
                "Token owner's allowance for the contract is less than the transfer amount"
            }
            ContractError::TokenTransferFailed => "The underlying token contract call failed",
            ContractError::NegativeSlashAmount => "Slash/unslash amount must be non-negative",
            ContractError::UnslashExceedsSlashedAmount => {
                "Unslash amount exceeds the bond's currently slashed amount"
            }
            ContractError::DuplicateAttestation => "Attestation already exists from this attester",
            ContractError::AttestationNotFound => "No attestation found for the given key",
            ContractError::AttestationAlreadyRevoked => "Attestation has already been revoked",
            ContractError::InvalidAttestationWeight => "Attestation weight must be positive",
            ContractError::DuplicateAttesterInBatch => {
                "The same attester appears more than once in this batch"
            }
            ContractError::AttestationWeightExceedsMax => {
                "Attestation weight exceeds the configured maximum"
            }
            ContractError::IdentityAlreadyRegistered => {
                "Identity has already been registered in the registry"
            }
            ContractError::BondContractAlreadyRegistered => {
                "Bond contract address has already been registered"
            }
            ContractError::IdentityNotRegistered => "Identity is not registered in the registry",
            ContractError::BondContractNotRegistered => {
                "Bond contract is not registered in the registry"
            }
            ContractError::AlreadyDeactivated => "Record is already in the deactivated state",
            ContractError::AlreadyActive => "Record is already in the active state",
            ContractError::InvalidContractAddress => {
                "Provided contract address is not a deployed contract"
            }
            ContractError::ContractCodeVerificationFailed => {
                "Contract code hash verification failed during trustless registration"
            }
            ContractError::ExpiryInPast => "Delegation expiry must be in the future",
            ContractError::DelegationNotFound => "No delegation found for the given key",
            ContractError::AlreadyRevoked => "Delegation has already been revoked",
            ContractError::DelegationExpiryTooLong => {
                "Delegation expiry exceeds the maximum allowed lifetime"
            }
            ContractError::UnknownScheme => "Unknown or unsupported signature scheme tag",
            ContractError::VerifierAlreadyRegistered => {
                "Verifier already registered for the given scheme tag"
            }
            ContractError::VerifierNotRegistered => {
                "No verifier registered for the given scheme tag"
            }
            ContractError::VerificationFailed => {
                "Signature verification failed for the given scheme and payload"
            }
            ContractError::RevocationGraceExpired => {
                "Post-expiry revocation attempted outside the configured grace window"
            }
            ContractError::DelegationNotExpired => {
                "Cleanup attempted on a delegation that is not expired yet"
            }
            ContractError::AmountMustBePositive => "Amount must be strictly positive (> 0)",
            ContractError::ThresholdExceedsSigners => {
                "Threshold cannot exceed the current signer count"
            }
            ContractError::InsufficientTreasuryBalance => {
                "Treasury balance is insufficient for withdrawal"
            }
            ContractError::ProposalNotFound => "Withdrawal proposal not found",
            ContractError::ProposalAlreadyExecuted => {
                "Withdrawal proposal has already been executed"
            }
            ContractError::InsufficientApprovals => {
                "Proposal does not have enough approvals to execute"
            }
            ContractError::InvalidFlashLoanCallback => {
                "Flashloan callback returned an invalid magic value"
            }
            ContractError::FlashLoanRepaymentFailed => {
                "Flashloan principal plus fee was not fully repaid"
            }
            ContractError::ProposalExpired => "Withdrawal proposal has expired",
            ContractError::SlippageExceeded => {
                "Settled withdrawal amount fell below the caller's minimum (slippage)"
            }
            ContractError::Overflow => "Integer overflow in checked arithmetic",
            ContractError::NoPendingAdmin => "No pending admin transfer exists",
            ContractError::DomainMismatch => "Payload domain tag does not match expected",
            ContractError::OwnerMismatch => "Payload owner does not match expected caller",
            ContractError::TargetMismatch => "Payload target does not match expected action",
            ContractError::ContractIdMismatch => "Payload contract_id does not match current contract",
            ContractError::InvalidAdminAddress => "Proposed admin is the zero or identity address",
            ContractError::AdminUnchanged => "Proposed admin is the same as the current admin",
            ContractError::TimelockNotReady => "Timelock delay has not yet elapsed",
            ContractError::EmergencyDrainNotPermitted => "Emergency drain requires contract to be paused and timelock window to have elapsed",
            ContractError::UpgradeAuthAlreadyInitialized => "Upgrade authorization has already been initialized",
            ContractError::UpgradeAuthNotInitialized => "Upgrade authorization has not been initialized",
            ContractError::AlreadyAuthorizedUpgrader => "Address already holds an active upgrade authorization",
            ContractError::CannotGrantHigherUpgradeRoleToSelf => {
                "Cannot grant an equal or higher upgrade role to self"
            }
            ContractError::NotAuthorizedUpgrader => "Address does not hold an upgrade authorization",
            ContractError::CannotRevokeLastUpgrader => "Cannot revoke the last remaining upgrader",
            ContractError::NotUpgradeAdmin => "Caller is not the upgrade admin",
            ContractError::UnauthorizedUpgrade => "Caller is not authorized to perform an upgrade",
            ContractError::UpgradeAuthorizationNotActive => "Caller's upgrade authorization is not active",
            ContractError::UpgradeAuthorizationExpired => "Caller's upgrade authorization has expired",
            ContractError::UpgradeProposalNotFound => "Upgrade proposal not found",
            ContractError::UpgradeProposalNotPending => "Upgrade proposal is not pending",
            ContractError::AlreadyApprovedUpgradeProposal => "Caller has already approved this upgrade proposal",
            ContractError::UpgradeProposalNotApproved => "Upgrade proposal does not have enough approvals to execute",
            ContractError::UpgradeImplementationMismatch => {
                "Implementation address does not match the approved proposal"
            }
            ContractError::NoCurrentImplementation => "No current implementation address is recorded",
            ContractError::SameImplementation => "New implementation is identical to the current one",
            ContractError::NewUpgradeAdminMustDiffer => "New upgrade admin must differ from the current one",
            ContractError::NoPendingUpgradeAdmin => "No pending upgrade-admin transfer exists",
            ContractError::NotPendingUpgradeAdmin => "Caller is not the pending upgrade admin",
            ContractError::Underflow => "Integer underflow in checked arithmetic",
            ContractError::UnsupportedInterface => "Bond contract does not support required interface",
            ContractError::DivisionByZero => "Division by a zero denominator",
        }
    }

    fn is_recoverable(&self) -> bool {
        // Classification rule (informs every arm below):
        //   RECOVERABLE — caller can fix their own input or wait for state
        //                 they observe to change, then retry the same
        //                 kind of operation successfully without code/
        //                 deployment changes.
        //   FATAL       — retrying the same caller input is guaranteed
        //                 to fail, and the fix is not in caller's hands:
        //                 code-level impossibility, security halt,
        //                 cryptographic failure, or system capacity
        //                 reached. Indexers/admins should be alerted;
        //                 clients should NOT retry.
        // Per-arm rationale is the trailing `// ...` comment so reviewers
        // can audit each decision next to its arm. The `///` trait rustdoc
        // captures the rule globally.
        match self {
            // --- Initialization: caller fixes setup state. ---
            ContractError::NotInitialized | ContractError::AlreadyInitialized => true,

            // --- Authorization (100-199) + Admin Transfer (109-112):
            //     switch to the correct signer/role, or wait/correct
            //     payload/state. Caller-fixable in every case. ---
            ContractError::NotAdmin
            | ContractError::NotBondOwner
            | ContractError::UnauthorizedAttester
            | ContractError::NotOriginalAttester
            | ContractError::NotSigner
            | ContractError::UnauthorizedDepositor
            | ContractError::ContractPaused         // wait for unpause
            | ContractError::InvalidPauseAction     // correct action byte
            | ContractError::InsufficientSignatures // gather more approvals
            | ContractError::AdminSuspended         // wait for suspension
            | ContractError::NoPendingAdmin         // call begin_admin_transfer first
            | ContractError::InvalidAdminAddress
            | ContractError::AdminUnchanged
            | ContractError::TimelockNotReady
            | ContractError::EmergencyDrainNotPermitted => true, // wait for pause + timelock window

            // --- Upgrade Authorization (115-134): same shape as the rest of
            //     Authorization above — wrong signer, wrong state, wrong ID.
            //     Caller-fixable by switching signer, waiting, or retrying
            //     with correct arguments. ---
            ContractError::UpgradeAuthAlreadyInitialized  // idempotent to observe
            | ContractError::UpgradeAuthNotInitialized    // admin can initialize first
            | ContractError::AlreadyAuthorizedUpgrader    // revoke first, or no-op
            | ContractError::CannotGrantHigherUpgradeRoleToSelf // choose a lower role
            | ContractError::NotAuthorizedUpgrader        // admin can grant, then retry
            | ContractError::CannotRevokeLastUpgrader     // grant another upgrader first
            | ContractError::NotUpgradeAdmin              // switch to the upgrade admin
            | ContractError::UnauthorizedUpgrade          // switch to an authorized upgrader
            | ContractError::UpgradeAuthorizationNotActive
            | ContractError::UpgradeAuthorizationExpired  // admin can re-grant
            | ContractError::UpgradeProposalNotFound      // supply a valid proposal id
            | ContractError::UpgradeProposalNotPending
            | ContractError::AlreadyApprovedUpgradeProposal // idempotent
            | ContractError::UpgradeProposalNotApproved   // gather more approvals
            | ContractError::UpgradeImplementationMismatch // pass the address on the proposal
            | ContractError::NoCurrentImplementation      // admin can set one first
            | ContractError::SameImplementation           // pass a different implementation
            | ContractError::NewUpgradeAdminMustDiffer
            | ContractError::NoPendingUpgradeAdmin        // start a transfer first
            | ContractError::NotPendingUpgradeAdmin => true, // switch to the nominated address

            // --- Bond (200-299): most errors are caller-fixable. ---
            ContractError::BondNotFound                 // create_bond first
            | ContractError::BondNotActive
            | ContractError::InsufficientBalance        // top up
            | ContractError::SlashExceedsBond           // reduce slash amount
            | ContractError::LockupNotExpired           // wait for the lock-up
            | ContractError::NotRollingBond
            | ContractError::WithdrawalAlreadyRequested // wait for the existing request
            | ContractError::InvalidNonce               // bump nonce
            | ContractError::SignatureExpired           // re-sign with later deadline
            | ContractError::NegativeStake              // reduce the stake
            | ContractError::EarlyExitConfigNotSet      // configure early exit first
            | ContractError::InvalidPenaltyBps          // use 0..=10000
            | ContractError::LeverageExceeded           // reduce operation size
            | ContractError::UnsupportedToken           // use a safe token (e.g. SAC)
            | ContractError::UnsupportedDecimals        // use a token with supported decimals
            | ContractError::UnauthorizedToken          // use an accepted token
            | ContractError::InvalidBondAmount
            | ContractError::InvalidBondDuration
            | ContractError::InvalidNoticePeriod
            | ContractError::BondAlreadyExists
            | ContractError::BatchTooLarge         // reduce batch size
            | ContractError::EmptyBatch            // supply at least one item
            | ContractError::WithdrawalNotRequested // call request_withdrawal first
            | ContractError::NoticePeriodNotElapsed // wait for the notice period
            | ContractError::BondNotEligibleForLiquidation // wait for slash/lock-up expiry
            | ContractError::ClaimAmountMustBePositive // supply amount > 0
            | ContractError::NoPendingClaims          // wait for claims to accrue
            | ContractError::NoValidClaimsToProcess   // wait, or adjust the type filter
            | ContractError::ClaimNotFound            // supply a valid claim id
            | ContractError::BondTokenNotConfigured   // admin can configure the token then retry
            | ContractError::ParameterOutOfBounds     // supply a value within bounds
            | ContractError::GovernanceApproverMismatch // sign with the correct approver
            | ContractError::GovernanceApprovalExpired  // obtain a fresh approval
            | ContractError::GovernanceApprovalCategoryMismatch // use the matching category
            | ContractError::UnsupportedNetwork       // pass "mainnet" or "testnet"
            | ContractError::NegativeTransferAmount   // supply amount >= 0
            | ContractError::InsufficientAllowance    // owner can raise the allowance
            | ContractError::TokenTransferFailed      // owner can fix balance/trustline and retry
            | ContractError::NegativeSlashAmount      // supply amount >= 0
            | ContractError::UnslashExceedsSlashedAmount // reduce the unslash amount
            => true,

            // FATAL Bond: caller cannot directly fix any of these.
            ContractError::StorageCapReached => false,    // system capacity; only operator prune fixes it
            ContractError::TreasuryNotConfigured => true, // admin can configure treasury then retry
            ContractError::CursorOutOfRange => true,      // caller can supply a valid cursor in range
            ContractError::DuplicateIdempotencyKey => true, // idempotent - safe to retry with same key
            ContractError::ReentrancyDetected => false,   // SECURITY HALT: investigate, do not retry
            ContractError::InvariantViolation => false,   // post-write drift detection

            // FATAL Bond/Delegation payload binding mismatches (218/219/220/221).
            // Same payload will fail again; clients must not blindly retry.
            ContractError::DomainMismatch
            | ContractError::OwnerMismatch
            | ContractError::TargetMismatch
            | ContractError::ContractIdMismatch => false,

            // --- Attestation (300-399): all caller-fixable. ---
            ContractError::DuplicateAttestation
            | ContractError::AttestationNotFound
            | ContractError::AttestationAlreadyRevoked
            | ContractError::InvalidAttestationWeight
            | ContractError::AttestationWeightExceedsMax
            | ContractError::DuplicateAttesterInBatch => true, // drop the duplicate and retry

            // --- Registry (400-499): all caller-fixable. ---
            ContractError::IdentityAlreadyRegistered
            | ContractError::BondContractAlreadyRegistered
            | ContractError::IdentityNotRegistered
            | ContractError::BondContractNotRegistered
            | ContractError::AlreadyDeactivated
            | ContractError::AlreadyActive
            | ContractError::InvalidContractAddress
            | ContractError::ContractCodeVerificationFailed => true,

            // --- Delegation (500-599): mostly caller-fixable ---
            ContractError::ExpiryInPast                // supply a future expiry
            | ContractError::DelegationNotFound        // create the delegation first
            | ContractError::AlreadyRevoked            // idempotent
            | ContractError::DelegationExpiryTooLong   // shorten to MAX_DURATION
            | ContractError::VerifierAlreadyRegistered // idempotent
            | ContractError::VerifierNotRegistered
            | ContractError::DelegationNotExpired => true,

            // FATAL Delegation: caller cannot fix these.
            ContractError::UnknownScheme => false,         // scheme tag not supported by this build
            ContractError::VerificationFailed => false,    // crypto failure; same input will fail
            ContractError::RevocationGraceExpired => false,           // grace window is admin-controlled; expiry is terminal for the caller

            // --- Treasury (600-699): mostly caller-fixable ---
            ContractError::AmountMustBePositive            // supply amount > 0
            | ContractError::ThresholdExceedsSigners        // lower threshold to <= signer count
            | ContractError::InsufficientTreasuryBalance    // top up
            | ContractError::ProposalNotFound               // supply a valid proposal id
            | ContractError::ProposalAlreadyExecuted        // idempotent
            | ContractError::InsufficientApprovals          // collect more approvals
            | ContractError::ProposalExpired                // create a new proposal
            | ContractError::SlippageExceeded => true,      // retry with a looser min_amount_out

            // FATAL Treasury flashloan failures: callback contract misbehaved.
            ContractError::InvalidFlashLoanCallback => false, // bad magic value
            ContractError::FlashLoanRepaymentFailed => false, // principal+fee mismatch

            // --- Arithmetic (700-799): code-level impossibility. ---
            ContractError::Overflow
            | ContractError::Underflow
            | ContractError::DivisionByZero => false,
            ContractError::UnsupportedInterface => false,
        }
    }
}

#[cfg(test)]
mod test_errors;
