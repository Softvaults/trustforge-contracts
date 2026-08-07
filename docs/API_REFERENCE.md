# TrustForge API Reference for Integrators

## Overview

This document provides a complete API reference for developers integrating with TrustForge smart contracts. All contracts are deployed on Stellar's Soroban platform.

## Contract Addresses

### Testnet

```
Network: Stellar Testnet
RPC: https://soroban-rpc.testnet.stellar.org

trustforge_bond:        <TESTNET_CONTRACT_ID>
trustforge_registry:    <TESTNET_CONTRACT_ID>
trustforge_treasury:    <TESTNET_CONTRACT_ID>
trustforge_delegation:  <TESTNET_CONTRACT_ID>
trustforge_arbitration: <TESTNET_CONTRACT_ID>
trustforge_admin:       <TESTNET_CONTRACT_ID>
trustforge_multisig:    <TESTNET_CONTRACT_ID>
timelock:               <TESTNET_CONTRACT_ID>
```

### Mainnet

```
Network: Stellar Mainnet (Pubnet)
RPC: https://soroban-rpc.mainnet.stellar.org

Contract addresses: See MAINNET_DEPLOYMENT.md
```

## Quick Start

### Installation

```bash
# Install Soroban CLI
cargo install soroban-cli

# Or use Stellar SDK (JavaScript)
npm install @stellar/stellar-sdk

# Or use Soroban SDK (Rust)
cargo add soroban-sdk
```

### Basic Bond Creation (CLI)

```bash
# Create a fixed-duration bond
soroban contract invoke \
  --id <BOND_CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- create_bond \
  --identity <YOUR_ADDRESS> \
  --amount 1000000000000000000000 \
  --duration 86400 \
  --is_rolling false \
  --notice_period_duration 0
```

### Basic Bond Creation (JavaScript)

```javascript
const { Contract, SorobanRpc, Keypair, TransactionBuilder } = require('@stellar/stellar-sdk');

async function createBond() {
    const rpc = new SorobanRpc.Server('https://soroban-rpc.testnet.stellar.org');
    const keypair = Keypair.fromSecret('YOUR_SECRET_KEY');
    const contract = new Contract('<BOND_CONTRACT_ID>');
    
    const tx = new TransactionBuilder(await rpc.getAccount(keypair.publicKey()), {
        fee: '100',
        networkPassphrase: 'Test SDF Network ; September 2015'
    })
    .addOperation(contract.call(
        'create_bond',
        keypair.publicKey(),           // identity
        '1000000000000000000000',      // amount (1000 USDC with 18 decimals)
        86400,                          // duration (24 hours)
        false,                          // is_rolling
        0                               // notice_period_duration
    ))
    .setTimeout(30)
    .build();
    
    tx.sign(keypair);
    const result = await rpc.sendTransaction(tx);
    console.log('Transaction:', result);
}
```

## Core API: trustforge_bond

### Data Types

```rust
pub struct IdentityBond {
    pub identity: Address,
    pub bonded_amount: i128,
    pub slashed_amount: i128,
    pub bond_start: u64,
    pub bond_end: u64,
    pub active: bool,
    pub is_rolling: bool,
    pub withdrawal_requested_at: u64,
    pub notice_period_duration: u64,
}

pub enum BondTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

pub struct Attestation {
    pub id: u64,
    pub subject: Address,
    pub attester: Address,
    pub data: String,
    pub timestamp: u64,
    pub revoked: bool,
}
```

### Bond Management Functions

#### `create_bond`

Create a new identity bond.

```rust
pub fn create_bond(
    e: Env,
    identity: Address,
    amount: i128,
    duration: u64,
    is_rolling: bool,
    notice_period_duration: u64
) -> IdentityBond
```

**Parameters:**
- `identity`: Address - The identity creating the bond
- `amount`: i128 - Amount to bond in normalized 18-decimal format
- `duration`: u64 - Lock-up duration in seconds
- `is_rolling`: bool - Whether bond auto-renews
- `notice_period_duration`: u64 - Notice period for rolling bonds (0 for fixed)

**Returns:** `IdentityBond` struct

**Requirements:**
- Caller must authorize the transaction
- Amount must be positive
- Duration must be within allowed range
- If rolling, notice_period_duration must be > 0
- Supply cap must not be exceeded

**Events Emitted:**
- `bond_created_v2(identity, amount, start_ts, duration, is_rolling, end_ts)`

**Example:**
```bash
# Fixed 7-day bond with 1000 USDC
soroban contract invoke --id $BOND_ID -- create_bond \
  --identity $USER \
  --amount 1000000000000000000000 \
  --duration 604800 \
  --is_rolling false \
  --notice_period_duration 0

# Rolling bond with 30-day notice
soroban contract invoke --id $BOND_ID -- create_bond \
  --identity $USER \
  --amount 5000000000000000000000 \
  --duration 2592000 \
  --is_rolling true \
  --notice_period_duration 2592000
```

#### `top_up`

Add funds to an existing bond.

```rust
pub fn top_up(e: Env, identity: Address, amount: i128) -> IdentityBond
```

**Parameters:**
- `identity`: Address - Bond owner
- `amount`: i128 - Amount to add

**Returns:** Updated `IdentityBond`

**Events:** `bond_increased_v2(identity, added, new_total, ts, tier_changed, new_tier)`

#### `withdraw`

Withdraw funds after lock-up period.

```rust
pub fn withdraw(e: Env, identity: Address, amount: i128) -> IdentityBond
```

**Parameters:**
- `identity`: Address - Bond owner
- `amount`: i128 - Amount to withdraw

**Requirements:**
- Bond must be past lock-up period
- Amount must be ≤ available balance
- Cannot withdraw if in notice period (rolling bonds)

**Events:** `bond_withdrawn_v2(identity, withdrawn, remaining, ts, false, 0)`

#### `withdraw_early`

Withdraw before lock-up ends (penalty applied).

```rust
pub fn withdraw_early(e: Env, identity: Address, amount: i128) -> IdentityBond
```

**Parameters:**
- `identity`: Address - Bond owner  
- `amount`: i128 - Gross amount to withdraw

**Returns:** Updated bond with penalty deducted

**Penalty:** Configured by admin (typically 5-10% of withdrawn amount)

**Events:** `bond_withdrawn_v2(identity, withdrawn, remaining, ts, true, penalty)`

#### `request_withdrawal`

Request withdrawal for rolling bond (starts notice period).

```rust
pub fn request_withdrawal(e: Env, identity: Address) -> IdentityBond
```

**Parameters:**
- `identity`: Address - Bond owner

**Requirements:**
- Bond must be rolling
- Not already in notice period

**Effects:** Sets `withdrawal_requested_at` to current timestamp

#### `withdraw_bond`

Close bond and withdraw all available funds.

```rust
pub fn withdraw_bond(e: Env, identity: Address) -> i128
```

**Parameters:**
- `identity`: Address - Bond owner

**Returns:** Amount withdrawn

**Requirements:**
- Bond must be inactive (past lock-up) OR
- Notice period must have elapsed (rolling bonds)

**Events:** `bond_withdrawn_v2(...)` + sets `active = false`

### Query Functions

#### `get_bond`

Get current bond state.

```rust
pub fn get_bond(e: Env, identity: Address) -> IdentityBond
```

**Example:**
```bash
soroban contract invoke --id $BOND_ID -- get_bond --identity $USER
```

**Response:**
```json
{
  "identity": "GABC...",
  "bonded_amount": "1000000000000000000000",
  "slashed_amount": "0",
  "bond_start": 1705334400,
  "bond_end": 1705939200,
  "active": true,
  "is_rolling": false,
  "withdrawal_requested_at": 0,
  "notice_period_duration": 0
}
```

#### `get_tier`

Get current tier for a bond.

```rust
pub fn get_tier(e: Env, identity: Address) -> BondTier
```

**Returns:** `Bronze | Silver | Gold | Platinum`

**Tiers:**
- Bronze: 0 - 1,000 USDC
- Silver: 1,000 - 5,000 USDC
- Gold: 5,000 - 20,000 USDC
- Platinum: 20,000+ USDC

#### `get_supply_cap`

Get maximum total bonded amount allowed.

```rust
pub fn get_supply_cap(e: Env) -> i128
```

**Returns:** Supply cap (0 = uncapped)

#### `get_total_supply`

Get current total bonded amount across all bonds.

```rust
pub fn get_total_supply(e: Env) -> i128
```

### Attestation Functions

#### `add_attestation`

Add an attestation about a subject.

```rust
pub fn add_attestation(
    e: Env,
    attester: Address,
    subject: Address,
    data: String,
    contract_id: Address,
    deadline: u64,
    nonce: u64
) -> u64
```

**Parameters:**
- `attester`: Address - Must be registered attester
- `subject`: Address - Identity being attested about
- `data`: String - Attestation data (e.g., "kyc:verified")
- `contract_id`: Address - Bond contract ID
- `deadline`: u64 - Attestation expiry timestamp
- `nonce`: u64 - Replay prevention nonce

**Returns:** Attestation ID

**Requirements:**
- Attester must be registered (`register_attester` called by admin)
- Subject must have active bond

**Events:** `attestation_added(subject, id, attester, data)`

#### `get_attestations`

Get all attestations for a subject (paginated).

```rust
pub fn get_attestations(
    e: Env,
    subject: Address,
    start_index: u32,
    page_size: u32
) -> Vec<Attestation>
```

**Parameters:**
- `subject`: Address - Identity to query
- `start_index`: u32 - Starting index (0-based)
- `page_size`: u32 - Max results to return

**Returns:** Vector of attestations

**Example:**
```bash
# Get first 10 attestations
soroban contract invoke --id $BOND_ID -- get_attestations \
  --subject $USER \
  --start_index 0 \
  --page_size 10
```

#### `revoke_attestation`

Revoke a previously issued attestation.

```rust
pub fn revoke_attestation(e: Env, attester: Address, attestation_id: u64)
```

**Parameters:**
- `attester`: Address - Must be original attester
- `attestation_id`: u64 - ID from `add_attestation`

**Events:** `attestation_revoked(subject, attestation_id, attester)`

### Admin Functions

#### `slash_bond`

Slash a bond (admin only).

```rust
pub fn slash_bond(e: Env, admin: Address, amount: i128) -> IdentityBond
```

**Parameters:**
- `admin`: Address - Must be contract admin
- `amount`: i128 - Amount to slash

**Requirements:**
- Caller must be admin
- Amount ≤ available balance (bonded - slashed)

**Effects:**
- Increases `slashed_amount`
- Transfers slashed funds to treasury
- Creates immutable slash record

**Events:** `bond_slashed_v2(identity, amount, total_slashed, ts, admin, reason, is_full)`

#### `set_supply_cap`

Set maximum total bonded amount.

```rust
pub fn set_supply_cap(e: Env, admin: Address, cap: i128)
```

#### `set_tier_thresholds`

Update tier thresholds.

```rust
pub fn set_tier_thresholds(
    e: Env,
    admin: Address,
    bronze_max: i128,
    silver_max: i128,
    gold_max: i128
)
```

## Registry API: trustforge_registry

### `register_identity`

Register identity → bond contract mapping.

```rust
pub fn register_identity(
    e: Env,
    admin: Address,
    identity: Address,
    bond_contract: Address
) -> RegistryEntry
```

### `get_bond_contract`

Get bond contract address for an identity.

```rust
pub fn get_bond_contract(e: Env, identity: Address) -> Address
```

### `get_identities_page`

Get paginated list of registered identities.

```rust
pub fn get_identities_page(
    e: Env,
    start_index: u32,
    page_size: u32
) -> Vec<Address>
```

## Event Indexing

### Event Schemas

All v2 events include indexed fields for efficient querying:

**bond_created_v2:**
```
Topics: (symbol="bond_created_v2", identity, amount, start_ts)
Data: (duration, is_rolling, end_ts)
```

**bond_increased_v2:**
```
Topics: (symbol="bond_increased_v2", identity, added, new_total, ts)
Data: (tier_changed: bool, new_tier: BondTier)
```

**bond_slashed_v2:**
```
Topics: (symbol="bond_slashed_v2", identity, slash_amt, total_slashed, ts, admin)
Data: (reason: Symbol, is_full_slash: bool)
```

**tier_changed_v2:**
```
Topics: (symbol="tier_changed_v2", identity)
Data: (old_tier: BondTier, new_tier: BondTier, timestamp: u64)
```

### Querying Events

Use Soroban RPC to query events:

```javascript
const rpc = new SorobanRpc.Server('https://soroban-rpc.testnet.stellar.org');

// Query bond creation events
const events = await rpc.getEvents({
    filters: [{
        type: 'contract',
        contractIds: ['<BOND_CONTRACT_ID>'],
        topics: [['bond_created_v2']]
    }],
    startLedger: 1234567
});

console.log('Bond creations:', events.events);
```

## Error Handling

### Common Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | `InsufficientBalance` | Not enough balance for operation |
| 2 | `BondNotActive` | Bond is inactive or doesn't exist |
| 3 | `NotAdmin` | Caller is not admin |
| 4 | `NotAttester` | Caller is not registered attester |
| 5 | `InvalidAmount` | Amount is zero or negative |
| 6 | `InvalidDuration` | Duration outside allowed range |
| 7 | `SlashExceedsBond` | Slash amount > available balance |
| 8 | `SupplyCapExceeded` | Would exceed supply cap |
| 9 | `NotInNoticeor` | Rolling bond not in notice period |
| 10 | `LockUpNotExpired` | Cannot withdraw before lock-up end |

### Error Handling Examples

**Rust:**
```rust
match bond_client.try_create_bond(&identity, &amount, &duration, &false, &0) {
    Ok(bond) => println!("Bond created: {:?}", bond),
    Err(e) if e.code() == 8 => eprintln!("Supply cap exceeded"),
    Err(e) => eprintln!("Error: {:?}", e),
}
```

**JavaScript:**
```javascript
try {
    const result = await contract.call('create_bond', ...args);
    console.log('Success:', result);
} catch (error) {
    if (error.message.includes('Error(Contract, #8)')) {
        console.error('Supply cap exceeded');
    } else {
        console.error('Transaction failed:', error);
    }
}
```

## Rate Limits & Best Practices

### Rate Limits

- Stellar network: ~1000 TPS network-wide
- Per account: ~100 TPS recommended
- RPC endpoints: Vary by provider (check with your RPC provider)

### Best Practices

1. **Use Pagination**: Always use paginated queries for lists
2. **Index Events**: Don't query storage directly; use event indexer
3. **Handle Errors**: Always check for insufficient balance before operations
4. **Test on Testnet**: Deploy and test thoroughly before mainnet
5. **Monitor Gas**: Track gas costs and optimize batches
6. **Cache Queries**: Cache tier/balance queries client-side
7. **Use Batch Ops**: Batch multiple attestations when possible

### Example: Efficient Balance Check

```javascript
// ❌ BAD: Query storage on every check
async function hasEnoughBalance(identity, amount) {
    const bond = await contract.call('get_bond', identity);
    return bond.bonded_amount - bond.slashed_amount >= amount;
}

// ✅ GOOD: Query indexer with cached recent state
async function hasEnoughBalance(identity, amount) {
    const cached = cache.get(identity);
    if (cached && Date.now() - cached.timestamp < 60000) {
        return cached.balance >= amount;
    }
    const bond = await indexer.getBond(identity);  // From DB
    cache.set(identity, { balance: bond.available_amount, timestamp: Date.now() });
    return bond.available_amount >= amount;
}
```

## SDK Examples

### Complete Integration Example (JavaScript)

```javascript
const { Contract, SorobanRpc, Keypair, TransactionBuilder, Networks } = require('@stellar/stellar-sdk');

class TrustForgeClient {
    constructor(network = 'testnet') {
        this.network = network;
        this.rpcUrl = network === 'testnet' 
            ? 'https://soroban-rpc.testnet.stellar.org'
            : 'https://soroban-rpc.mainnet.stellar.org';
        this.rpc = new SorobanRpc.Server(this.rpcUrl);
        this.passphrase = network === 'testnet'
            ? Networks.TESTNET
            : Networks.PUBLIC;
    }
    
    async createBond(secretKey, bondContractId, amount, duration) {
        const keypair = Keypair.fromSecret(secretKey);
        const contract = new Contract(bondContractId);
        const account = await this.rpc.getAccount(keypair.publicKey());
        
        const tx = new TransactionBuilder(account, {
            fee: '1000000',  // 1 XLM max fee
            networkPassphrase: this.passphrase
        })
        .addOperation(contract.call(
            'create_bond',
            keypair.publicKey(),
            amount,
            duration,
            false,  // not rolling
            0       // no notice period
        ))
        .setTimeout(30)
        .build();
        
        tx.sign(keypair);
        const result = await this.rpc.sendTransaction(tx);
        
        // Wait for confirmation
        let status = await this.rpc.getTransaction(result.hash);
        while (status.status === 'PENDING') {
            await new Promise(resolve => setTimeout(resolve, 1000));
            status = await this.rpc.getTransaction(result.hash);
        }
        
        if (status.status === 'SUCCESS') {
            return status.returnValue;
        } else {
            throw new Error(`Transaction failed: ${status.status}`);
        }
    }
    
    async getBond(bondContractId, identity) {
        const contract = new Contract(bondContractId);
        const result = await this.rpc.simulateTransaction(
            new TransactionBuilder(
                new SorobanRpc.Account('GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF', '0'),
                { fee: '100', networkPassphrase: this.passphrase }
            )
            .addOperation(contract.call('get_bond', identity))
            .setTimeout(30)
            .build()
        );
        return result.result.returnValue;
    }
}

// Usage
const client = new TrustForgeClient('testnet');
const bond = await client.createBond(
    'YOUR_SECRET_KEY',
    '<BOND_CONTRACT_ID>',
    '1000000000000000000000',  // 1000 USDC
    86400  // 24 hours
);
console.log('Created bond:', bond);
```

## Support & Resources

- **Documentation**: https://docs.trustforge.io
- **GitHub**: https://github.com/Softvaults/trustforge-contracts
- **Discord**: https://discord.gg/trustforge
- **Security Issues**: security@trustforge.io (use PGP key)

---

**Last Updated**: January 2026  
**Version**: 1.0.0  
**Maintainer**: TrustForge Team
