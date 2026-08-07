# Mainnet Deployment Guide

**WARNING**: This document covers mainnet deployment with real funds at risk. Follow each step carefully and verify all configurations before proceeding.

## Pre-Deployment Checklist

Before deploying to mainnet, ensure:

### ✅ Security
- [ ] All contracts have passed internal security review (see [SECURITY_AUDIT.md](../SECURITY_AUDIT.md))
- [ ] Third-party audit completed (recommended) or risk acknowledged
- [ ] Multi-sig wallet set up for admin operations
- [ ] Hardware wallets configured for all signers
- [ ] Emergency response plan documented
- [ ] Bug bounty program ready to launch

### ✅ Testing
- [ ] Full testnet deployment completed successfully
- [ ] Integration tests run against testnet deployment
- [ ] User acceptance testing completed
- [ ] Load testing performed on testnet
- [ ] Upgrade procedures tested on testnet
- [ ] Emergency procedures tested on testnet

### ✅ Documentation
- [ ] All API documentation up to date
- [ ] Integration guides written for partners
- [ ] Frontend SDK deployed and tested
- [ ] Monitoring dashboards configured
- [ ] Alerting rules defined
- [ ] Runbooks for common operations written

### ✅ Legal & Compliance
- [ ] Terms of service reviewed
- [ ] Privacy policy updated
- [ ] Regulatory compliance verified for target jurisdictions
- [ ] Insurance coverage evaluated (if applicable)

### ✅ Operations
- [ ] 24/7 on-call rotation established
- [ ] Incident response team assigned
- [ ] Communication channels set up (status page, Discord, Twitter)
- [ ] Backup admin keys secured in cold storage
- [ ] Key rotation procedures documented

## Mainnet vs Testnet Differences

| Aspect | Testnet | Mainnet |
|--------|---------|---------|
| **Network ID** | `Test SDF Network ; September 2015` | `Public Global Stellar Network ; September 2015` |
| **Horizon URL** | `https://horizon-testnet.stellar.org` | `https://horizon.stellar.org` |
| **Friendbot** | Available | Not available |
| **Token** | Test USDC | Production USDC (GDQOE23CFSUMSVQK4Y5JHPPYK73VYCNHZHA7ENKCV37P6SUEO6XQBKPP) |
| **Deployment cost** | Testnet XLM (free) | Real XLM (purchase required) |
| **Risk** | Zero | Real funds at risk |
| **Recovery** | Can redeploy freely | Immutable, requires migration |

## Deployment Steps

### 1. Prepare Mainnet Environment

```bash
# Configure Soroban CLI for mainnet
soroban network add mainnet \
  --rpc-url https://soroban-rpc.mainnet.stellar.org \
  --network-passphrase "Public Global Stellar Network ; September 2015"

# Generate or import admin keys
soroban keys generate mainnet-admin --no-fund

# Fund admin account with XLM for deployment (minimum 100 XLM recommended)
# Purchase XLM from exchange and send to generated address

# Verify balance
soroban keys address mainnet-admin
stellar account --network mainnet --account <address>
```

### 2. Configure Multi-Sig Admin

**CRITICAL**: Never use a single EOA as admin on mainnet. Always use multi-sig.

```bash
# Set up multi-sig signers (3-of-5 recommended for production)
export SIGNER_1="<hardware-wallet-1-address>"
export SIGNER_2="<hardware-wallet-2-address>"
export SIGNER_3="<hardware-wallet-3-address>"
export SIGNER_4="<hardware-wallet-4-address>"
export SIGNER_5="<hardware-wallet-5-address>"
export MULTISIG_THRESHOLD=3
```

### 3. Build Production WASM

```bash
# Clean build to ensure no dev artifacts
cargo clean

# Build with locked dependencies for reproducibility
cargo build \
  --target wasm32-unknown-unknown \
  --release \
  --locked \
  -p trustforge_bond \
  -p trustforge_delegation \
  -p trustforge_treasury \
  -p trustforge_admin \
  -p trustforge_multisig \
  -p trustforge_arbitration \
  -p trustforge_registry \
  -p timelock

# Verify WASM sizes
bash scripts/check_wasm_size.sh

# Generate and verify checksums
bash scripts/generate_checksums.sh > MAINNET_CHECKSUMS.txt
```

### 4. Deploy Core Contracts

Follow the same order as testnet deployment (see [DEPLOYMENT.md](DEPLOYMENT.md)), but with mainnet configuration:

```bash
export NETWORK="mainnet"
export ADMIN_KEY="mainnet-admin"
export USDC_TOKEN_ADDRESS="GDQOE23CFSUMSVQK4Y5JHPPYK73VYCNHZHA7ENKCV37P6SUEO6XQBKPP"

# Deploy each contract (see DEPLOYMENT.md for full steps)
# 1. trustforge_admin
# 2. timelock  
# 3. trustforge_multisig
# 4. trustforge_arbitration
# 5. trustforge_registry (NEW for mainnet)
# 6. trustforge_treasury
# 7. trustforge_bond
# 8. trustforge_delegation
```

**IMPORTANT**: Save all contract IDs immediately after deployment:

```bash
# Store in secure location (encrypted vault, hardware wallet storage)
cat > mainnet_contract_ids.txt <<EOF
ADMIN_CONTRACT_ID=$ADMIN_CONTRACT_ID
TIMELOCK_CONTRACT_ID=$TIMELOCK_CONTRACT_ID
MULTISIG_CONTRACT_ID=$MULTISIG_CONTRACT_ID
ARBITRATION_CONTRACT_ID=$ARBITRATION_CONTRACT_ID
REGISTRY_CONTRACT_ID=$REGISTRY_CONTRACT_ID
TREASURY_CONTRACT_ID=$TREASURY_CONTRACT_ID
BOND_CONTRACT_ID=$BOND_CONTRACT_ID
DELEGATION_CONTRACT_ID=$DELEGATION_CONTRACT_ID
DEPLOYED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
DEPLOYER_ADDRESS=$(soroban keys address mainnet-admin)
EOF

# Encrypt and back up this file
gpg --encrypt --recipient <your-gpg-key> mainnet_contract_ids.txt
```

### 5. Configure Production Parameters

```bash
# Set production fee parameters
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source mainnet-admin \
  --network mainnet \
  -- set_fee_config \
  --admin "$MULTISIG_CONTRACT_ID" \
  --treasury "$TREASURY_CONTRACT_ID" \
  --fee_bps 50  # 0.5% creation fee

# Set early exit penalty (higher on mainnet to discourage gaming)
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source mainnet-admin \
  --network mainnet \
  -- set_early_exit_config \
  --admin "$MULTISIG_CONTRACT_ID" \
  --treasury "$TREASURY_CONTRACT_ID" \
  --penalty_bps 1000  # 10% penalty

# Set tier thresholds (mainnet values)
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source mainnet-admin \
  --network mainnet \
  -- set_tier_thresholds \
  --admin "$MULTISIG_CONTRACT_ID" \
  --bronze_max 1000000000000000000000     # 1,000 USDC
  --silver_max 5000000000000000000000     # 5,000 USDC
  --gold_max 20000000000000000000000      # 20,000 USDC

# Set supply cap (start conservative)
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source mainnet-admin \
  --network mainnet \
  -- set_supply_cap \
  --admin "$MULTISIG_CONTRACT_ID" \
  --cap 100000000000000000000000  # 100,000 USDC initial cap
```

### 6. Transfer Admin to Multi-Sig

**CRITICAL STEP**: Transfer admin control from deployment EOA to multi-sig wallet.

```bash
# Initiate ownership transfer
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source mainnet-admin \
  --network mainnet \
  -- transfer_ownership \
  --caller "$DEPLOYER_ADDRESS" \
  --new_owner "$MULTISIG_CONTRACT_ID"

# Repeat for all contracts
# Then have multi-sig accept ownership
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source mainnet-multisig-signer-1 \
  --network mainnet \
  -- accept_ownership \
  --caller "$MULTISIG_CONTRACT_ID"

# Verify ownership transfer
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --network mainnet \
  -- get_admin
# Should return: $MULTISIG_CONTRACT_ID
```

### 7. Verify Deployment

Run comprehensive verification:

```bash
# Run verification script
bash scripts/verify_mainnet_deployment.sh

# Manual spot checks
soroban contract invoke --id "$BOND_CONTRACT_ID" --network mainnet -- get_admin
soroban contract invoke --id "$BOND_CONTRACT_ID" --network mainnet -- get_supply_cap
soroban contract invoke --id "$TREASURY_CONTRACT_ID" --network mainnet -- get_threshold
soroban contract invoke --id "$REGISTRY_CONTRACT_ID" --network mainnet -- get_admin

# Verify event indexing is working
# Check that initialization events are being captured by your indexer
```

### 8. Lock Deployment Keys

After successful deployment and ownership transfer:

```bash
# Remove deployment keys from hot storage
soroban keys remove mainnet-admin

# Store encrypted backup in multiple secure locations:
# - Hardware wallet
# - Encrypted cloud storage  
# - Physical paper backup in safe deposit box

# Document key recovery procedures
# Ensure 3+ team members can access keys in emergency
```

## Post-Deployment

### Enable Monitoring

1. **Event Indexing**
   - Deploy event indexer
   - Verify all v2 events being captured
   - Set up database backups

2. **Metrics & Alerts**
   - Total bonded amount
   - Active bonds count
   - Slashing events
   - Treasury balance
   - Failed transactions
   - Gas usage spikes

3. **Dashboards**
   - Public protocol stats
   - Admin dashboard for operations
   - Security monitoring (unusual slashing patterns)

### Launch Checklist

- [ ] Announce contract addresses via official channels
- [ ] Update frontend to use mainnet contracts
- [ ] Enable contract verification on Stellar Explorer
- [ ] Publish integration guides with mainnet addresses
- [ ] Launch bug bounty program
- [ ] Begin social media monitoring for issues
- [ ] Schedule first post-deployment review (24h after launch)

### First 24 Hours

- [ ] Monitor all transactions closely
- [ ] Watch for unusual patterns
- [ ] Verify first bond creation works end-to-end
- [ ] Test withdrawal flow with small amount
- [ ] Verify fee collection
- [ ] Check event indexing completeness
- [ ] Review gas costs vs estimates

### First Week

- [ ] Daily team sync on metrics
- [ ] Review all slashing proposals (if any)
- [ ] Monitor supply cap utilization
- [ ] Collect user feedback
- [ ] Document any unexpected behavior
- [ ] Plan first parameter adjustments (if needed)

### First Month

- [ ] Publish transparency report
- [ ] Review and adjust supply cap
- [ ] Evaluate fee parameters
- [ ] Plan feature enhancements
- [ ] Conduct post-mortem on any incidents
- [ ] Schedule third-party audit (if not done pre-launch)

## Emergency Procedures

### Circuit Breaker Activation

If critical issue detected:

```bash
# Pause all contracts (requires multi-sig threshold)
# Each signer must call:
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source <signer-key> \
  --network mainnet \
  -- propose_pause \
  --proposer "<signer-address>" \
  --action "pause" \
  --target_ledger <current-ledger + 100>

# Once threshold reached, pause activates automatically
```

### Emergency Withdrawal (Last Resort)

Only use if contract exploit detected and user funds at risk:

```bash
# Requires dual-auth from governance + admin
soroban contract invoke \
  --id "$BOND_CONTRACT_ID" \
  --source governance-key \
  --network mainnet \
  -- emergency_withdraw \
  --governance "<governance-address>" \
  --admin "<admin-address>" \
  --identity "<affected-user>" \
  --amount <amount-to-rescue> \
  --reason "Critical vulnerability mitigation"

# This creates immutable audit trail
# All emergency withdrawals are publicly logged
```

### Communication Template

```markdown
🚨 TRUSTFORGE PROTOCOL NOTICE

Status: [INVESTIGATING / MITIGATED / RESOLVED]
Impact: [Brief description]
Action: [What we're doing]
User Action Required: [What users should do]
Timeline: [Expected resolution]

We will provide updates every [frequency].

Details: [link to status page]
```

## Rollback & Migration

**Important**: Soroban contracts are immutable. "Rollback" means deploying new contracts and migrating state.

### Migration Steps

1. Deploy new contract versions
2. Pause old contracts
3. Snapshot state from old contracts
4. Initialize new contracts with snapshot data
5. Update registry mappings
6. Redirect frontend to new contracts
7. Communicate migration to users
8. Set old contracts to permanently paused

See [UPGRADE.md](UPGRADE.md) for detailed migration procedures.

## Mainnet Contract Addresses

**Official mainnet deployment** (update after deployment):

```
Network: Stellar Mainnet (pubnet)
Deployment Date: TBD
Deployer: TrustForge Team

trustforge_admin:       <CONTRACT_ID>
trustforge_bond:        <CONTRACT_ID>
trustforge_registry:    <CONTRACT_ID>
trustforge_treasury:    <CONTRACT_ID>
trustforge_delegation:  <CONTRACT_ID>
trustforge_arbitration: <CONTRACT_ID>
trustforge_multisig:    <CONTRACT_ID>
timelock:               <CONTRACT_ID>

Mainnet USDC: GDQOE23CFSUMSVQK4Y5JHPPYK73VYCNHZHA7ENKCV37P6SUEO6XQBKPP

Checksums: See MAINNET_CHECKSUMS.txt
```

## Support & Incident Response

- **Security Issues**: security@trustforge.io (use GPG key)
- **Operational Issues**: ops@trustforge.io  
- **Status Page**: status.trustforge.io
- **Discord**: discord.gg/trustforge (verify official link)
- **On-Call**: PagerDuty rotation (internal)

## Additional Resources

- [Testnet Deployment Guide](DEPLOYMENT.md)
- [Security Audit Report](../SECURITY_AUDIT.md)
- [Known Limitations](known-simplifications.md)
- [Upgrade Procedures](UPGRADE.md)
- [Architecture Overview](architecture.md)

---

**Remember**: Mainnet deployment is irreversible. When in doubt, test on testnet first.
