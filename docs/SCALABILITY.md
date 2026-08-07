# Scalability & Performance Guide

## Overview

This document addresses scalability considerations for TrustForge contracts as the protocol grows. It covers optimization strategies, performance bottlenecks, and production-grade scaling patterns.

## Current Architecture Limitations

### Single-Bond-Per-Contract Model

**Current Design:**
- Each identity deploys its own bond contract instance
- Registry maintains identity → contract mappings
- Provides strong isolation but higher deployment costs

**Limitations:**
- ❌ High deployment cost per identity (gas + storage)
- ❌ Registry becomes bottleneck for discovery
- ❌ No native batch queries across identities
- ⚠️ `get_all_identities()` is unbounded (O(n) storage read)

**Migration Path:**

For high-scale deployments (>10,000 identities), consider:

1. **Sharded Registry Pattern**
   ```
   Registry_Shard_0 → Identities [0-9999]
   Registry_Shard_1 → Identities [10000-19999]
   Registry_Shard_N → Identities [N*10000 - (N+1)*10000-1]
   ```

2. **Multi-Identity Bond Contract** (v2.0 planned)
   - Store multiple bonds in single contract
   - Reduces deployment costs by 90%+
   - Requires careful access control per bond
   - See [docs/multi-identity-bonds.md](multi-identity-bonds.md)

### Unbounded Iteration

**Problem:**
```rust
// Current implementation - unbounded
pub fn get_all_identities(e: Env) -> Vec<Address> {
    e.storage().instance().get(&DataKey::RegisteredIdentities)
        .unwrap_or(Vec::new(&e))
    // Returns ALL identities - O(n) read
}
```

**Impact:**
- Transaction timeout risk with >1000 identities
- High gas costs
- Poor UX for large datasets

**Solutions:**

1. **Use Event-Based Indexing (Recommended)**
   ```rust
   // Index identity_registered events off-chain
   // Query backend API instead of contract
   GET /api/v1/identities?page=1&limit=100
   ```

2. **Implement Pagination**
   ```rust
   pub fn get_identities_paginated(
       e: Env,
       start_index: u32,
       page_size: u32
   ) -> (Vec<Address>, u32) {
       let all = e.storage().instance()
           .get(&DataKey::RegisteredIdentities)
           .unwrap_or(Vec::new(&e));
       
       let total = all.len();
       let end = min(start_index + page_size, total);
       let page = all.slice(start_index..end);
       
       (page, total)
   }
   ```

3. **Cursor-Based Pagination (Best for Growing Lists)**
   ```rust
   pub fn get_identities_cursor(
       e: Env,
       cursor: Option<Address>,
       limit: u32
   ) -> (Vec<Address>, Option<Address>) {
       // Returns (results, next_cursor)
       // Client uses next_cursor for subsequent requests
   }
   ```

## Performance Optimization

### Storage Tier Selection

Choose the right storage tier for each data type:

| Data Type | Recommended Tier | Rationale |
|-----------|------------------|-----------|
| **Config** (admin, thresholds) | Instance | Never archived, always needed |
| **Active bonds** | Persistent | Long-lived, needs archival protection |
| **Slash history** | Persistent | Immutable audit trail |
| **Temporary state** (nonces) | Temporary | Short-lived, can be regenerated |
| **Cache** (computed values) | Temporary | Recomputable from source |

**Impact:**
- Instance storage: Most expensive, never archived
- Persistent storage: Moderate cost, requires TTL bumps
- Temporary storage: Cheapest, auto-archived after TTL

### Gas Optimization Patterns

#### 1. Batch Operations

**Before (inefficient):**
```rust
// Multiple transactions
for identity in identities {
    client.create_bond(identity, amount, duration);
    // Gas cost: N × (base_cost + bond_creation)
}
```

**After (optimized):**
```rust
// Single batch transaction
pub fn create_bonds_batch(
    e: Env,
    bonds: Vec<BondCreation>
) -> Vec<Result<(), ContractError>> {
    bonds.iter().map(|b| {
        create_bond_internal(&e, b.identity, b.amount, b.duration)
    }).collect()
}
// Gas cost: base_cost + (N × bond_creation)
// Saves ~30% on base transaction overhead
```

#### 2. Lazy Computation

**Before (eager):**
```rust
// Compute tier on every read
pub fn get_bond(e: Env) -> IdentityBond {
    let mut bond = read_bond(&e);
    bond.tier = compute_tier(&e, bond.bonded_amount); // Recomputed every time
    bond
}
```

**After (cached):**
```rust
// Compute tier only when amount changes
pub fn get_bond(e: Env) -> IdentityBond {
    read_bond(&e) // Tier stored with bond
}

pub fn top_up(e: Env, amount: i128) {
    let mut bond = read_bond(&e);
    bond.bonded_amount += amount;
    bond.tier = compute_tier(&e, bond.bonded_amount); // Only recomputed here
    write_bond(&e, bond);
}
```

#### 3. Early Returns

```rust
// Check cheapest conditions first
pub fn slash_bond(e: Env, admin: Address, amount: i128) -> IdentityBond {
    // 1. Auth check (cheap)
    admin.require_auth();
    require_admin(&e, &admin);
    
    // 2. Amount validation (cheap)
    if amount <= 0 {
        panic_with_error!(&e, ContractError::InvalidAmount);
    }
    
    // 3. Storage read (expensive) - only if above checks pass
    let bond = read_bond(&e);
    
    // 4. Business logic
    let available = bond.bonded_amount - bond.slashed_amount;
    require!(amount <= available, ContractError::SlashExceedsBond);
    
    // ... rest of logic
}
```

### Event Optimization

**Efficient Event Design:**

```rust
// ✅ Good: Indexed topics for filtering, data for details
e.events().publish(
    (
        symbol_short!("slashed"),  // Topic 0: event type
        identity.clone(),           // Topic 1: indexed entity
        slash_amount,               // Topic 2: indexed amount
    ),
    (
        total_slashed,              // Data: additional context
        timestamp,
        is_full_slash,
    )
);

// ❌ Bad: Everything in data (not filterable)
e.events().publish(
    (symbol_short!("event"),),
    (event_type, identity, amount, total, timestamp)
);
```

**Indexing Strategy:**
- Topic 0: Event type (for filtering by event)
- Topic 1: Primary entity (identity, contract)
- Topic 2: Key metric (amount, status)
- Data: Remaining context

## Indexer Architecture

### Off-Chain Indexing (Required for Production)

**Components:**

```
Stellar RPC → Event Stream → Indexer → PostgreSQL → API → Frontend
              ↓
         Filter relevant
         contract events
```

**Database Schema:**

```sql
-- Bonds table (derived from events)
CREATE TABLE bonds (
    identity_address TEXT PRIMARY KEY,
    contract_address TEXT NOT NULL,
    bonded_amount NUMERIC(38,0) NOT NULL,
    slashed_amount NUMERIC(38,0) NOT NULL,
    tier VARCHAR(20) NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    INDEX idx_tier (tier),
    INDEX idx_active (is_active)
);

-- Slash history (immutable audit log)
CREATE TABLE slash_events (
    id SERIAL PRIMARY KEY,
    identity_address TEXT NOT NULL,
    slash_amount NUMERIC(38,0) NOT NULL,
    total_slashed NUMERIC(38,0) NOT NULL,
    admin_address TEXT NOT NULL,
    reason TEXT,
    timestamp TIMESTAMP NOT NULL,
    tx_hash TEXT NOT NULL,
    INDEX idx_identity (identity_address),
    INDEX idx_timestamp (timestamp)
);

-- Attestations
CREATE TABLE attestations (
    id BIGINT PRIMARY KEY,
    subject_address TEXT NOT NULL,
    attester_address TEXT NOT NULL,
    data TEXT NOT NULL,
    weight INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP,
    INDEX idx_subject (subject_address),
    INDEX idx_attester (attester_address)
);
```

**Indexer Implementation (Pseudocode):**

```typescript
// Event handler
async function handleBondCreated(event: BondCreatedV2Event) {
    await db.bonds.upsert({
        identity_address: event.topics[1],
        contract_address: event.contract,
        bonded_amount: event.data.amount,
        slashed_amount: 0,
        tier: calculateTier(event.data.amount),
        is_active: true,
        created_at: event.data.start_ts,
        updated_at: event.data.start_ts,
    });
}

async function handleBondSlashed(event: BondSlashedV2Event) {
    const identity = event.topics[1];
    
    // Update bonds table
    await db.bonds.update({
        where: { identity_address: identity },
        data: {
            slashed_amount: event.data.total_slashed,
            updated_at: event.data.timestamp,
        }
    });
    
    // Append to slash history
    await db.slash_events.insert({
        identity_address: identity,
        slash_amount: event.topics[2],
        total_slashed: event.data.total_slashed,
        admin_address: event.topics[5],
        reason: event.data.reason,
        timestamp: event.data.timestamp,
        tx_hash: event.tx_hash,
    });
}
```

### API Layer

**REST API Design:**

```typescript
// Paginated identity list
GET /api/v1/identities
  ?page=1
  &limit=100
  &tier=gold
  &active=true

// Response
{
    "data": [
        {
            "identity": "GABC...",
            "contract": "CDEF...",
            "bonded_amount": "5000000000000000000000",
            "tier": "gold",
            "is_active": true
        }
    ],
    "pagination": {
        "page": 1,
        "limit": 100,
        "total": 1523,
        "has_next": true
    }
}

// Single identity details
GET /api/v1/identities/:address
{
    "identity": "GABC...",
    "contract": "CDEF...",
    "bonded_amount": "5000000000000000000000",
    "slashed_amount": "0",
    "tier": "gold",
    "is_active": true,
    "created_at": "2026-01-15T10:30:00Z",
    "slash_history": [...],
    "attestations": [...]
}

// Aggregated statistics
GET /api/v1/stats
{
    "total_identities": 1523,
    "total_bonded": "15000000000000000000000000",
    "by_tier": {
        "bronze": 1200,
        "silver": 250,
        "gold": 60,
        "platinum": 13
    },
    "slash_events_24h": 3
}
```

## Caching Strategy

### Contract-Level Caching

```rust
// Cache computed tier thresholds
pub fn get_tier_cached(e: &Env, amount: i128) -> BondTier {
    let cache_key = DataKey::TierCache(amount / CACHE_GRANULARITY);
    
    if let Some(tier) = e.storage().temporary().get(&cache_key) {
        return tier;
    }
    
    let tier = compute_tier(e, amount);
    e.storage().temporary().set(&cache_key, &tier);
    e.storage().temporary().extend_ttl(&cache_key, 100, 1000);
    
    tier
}
```

### API-Level Caching

```typescript
// Redis cache for hot queries
const cacheKey = `identity:${address}`;
const cached = await redis.get(cacheKey);

if (cached) {
    return JSON.parse(cached);
}

const data = await db.bonds.findUnique({ where: { identity_address: address }});
await redis.setex(cacheKey, 60, JSON.stringify(data)); // 60s TTL
return data;
```

**Cache Invalidation:**
- Invalidate on `bond_slashed`, `bond_withdrawn`, `bond_created` events
- Use pub/sub to notify API servers of invalidations

## Load Testing

### Test Scenarios

1. **High Bond Creation Rate**
   - 100 bonds/minute
   - Verify registry doesn't timeout
   - Check gas costs remain stable

2. **Mass Slashing Event**
   - 50 simultaneous slash proposals
   - Verify governance voting scales
   - Check event emission performance

3. **Large Withdrawal Queue**
   - 200 concurrent withdrawal requests
   - Verify cooldown logic performs
   - Check treasury depletion handling

### Benchmarking

```bash
# Gas benchmarks (see contracts/trustforge_bond/benches/)
cargo bench --features gas-bench -p trustforge_bond

# Load test with k6
k6 run scripts/load-test.js \
  --vus 100 \
  --duration 5m \
  --out influxdb=http://localhost:8086/k6
```

## Scaling Checklist

Before deploying to production with >1000 identities:

- [ ] **Off-chain indexer deployed** and syncing events
- [ ] **API layer implemented** with pagination
- [ ] **Caching enabled** (Redis/Memcached)
- [ ] **Load testing completed** at 2x expected volume
- [ ] **Monitoring configured** for key metrics
- [ ] **Alert thresholds set** for performance degradation
- [ ] **Database optimized** with proper indexes
- [ ] **CDN configured** for static assets
- [ ] **Rate limiting enabled** on public APIs
- [ ] **Backup strategy** for indexer database

## Performance Metrics

### Target Performance (Mainnet)

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Bond creation latency | <5s | >10s |
| Slash execution latency | <3s | >7s |
| API response time (p95) | <200ms | >500ms |
| Event indexing lag | <30s | >2min |
| Database query time (p95) | <50ms | >200ms |
| Contract gas cost | <0.1 XLM | >0.5 XLM |

### Monitoring Queries

```sql
-- Slow queries
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
WHERE mean_exec_time > 100
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Index usage
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0
ORDER BY pg_relation_size(indexrelid) DESC;

-- Table sizes
SELECT tablename,
       pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

## Future Scaling Roadmap

### Phase 1: Current (0-10K identities)
- ✅ Single-bond-per-contract
- ✅ Event-based indexing
- ✅ Basic pagination

### Phase 2: Medium Scale (10K-100K identities)
- 🔄 Sharded registry
- 🔄 Cursor-based pagination
- 🔄 Read replicas for indexer DB
- 🔄 Advanced caching (multi-layer)

### Phase 3: Large Scale (100K-1M identities)
- 📋 Multi-identity bond contracts
- 📋 Horizontal scaling of indexers
- 📋 GraphQL API with DataLoader
- 📋 Materialized views for analytics

### Phase 4: Massive Scale (1M+ identities)
- 📋 Fully sharded architecture
- 📋 Separate read/write paths
- 📋 CQRS pattern
- 📋 Event sourcing for audit trail

## Additional Resources

- [Multi-Identity Bonds Design](multi-identity-bonds.md)
- [Event Indexing Guide](event-indexing.md)
- [Architecture Overview](architecture.md)
- [Performance Benchmarks](bond_gas_benchmarks.md)

---

**Last Updated**: January 2026  
**Maintainer**: TrustForge Engineering Team
