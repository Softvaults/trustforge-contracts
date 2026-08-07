# Production Monitoring & Observability Guide

## Overview

This guide establishes production-grade monitoring for TrustForge contracts, covering metrics collection, alerting, dashboards, and incident response.

## Architecture

```
┌─────────────────┐
│ Stellar Network │
│  (Soroban RPC)  │
└────────┬────────┘
         │ Events & Transactions
         ▼
┌─────────────────┐
│ Event Indexer   │◄─── Historical data sync
│  (PostgreSQL)   │
└────────┬────────┘
         │ Normalized events
         ▼
┌─────────────────┐     ┌──────────────┐
│   Prometheus    │────▶│   Grafana    │
│   (Metrics)     │     │ (Dashboards) │
└────────┬────────┘     └──────────────┘
         │
         ▼
┌─────────────────┐     ┌──────────────┐
│   AlertManager  │────▶│  PagerDuty   │
│                 │     │ Slack/Discord│
└─────────────────┘     └──────────────┘
```

## Event Indexing

### Events to Index

All v2 events are designed for efficient indexing:

**Bond Lifecycle:**
```
bond_created_v2(identity, amount, start_ts, duration, is_rolling, end_ts)
bond_increased_v2(identity, added, new_total, ts, tier_changed, new_tier)
bond_withdrawn_v2(identity, withdrawn, remaining, ts, is_early, penalty)
bond_slashed_v2(identity, slash_amt, total_slashed, ts, admin, reason, is_full_slash)
tier_changed_v2(identity, old_tier, new_tier, timestamp)
```

**Attestations:**
```
attestation_added(subject, id, attester, data)
attestation_revoked(subject, id, attester)
```

**Treasury:**
```
fee_received(source, amount, fee_type)
withdrawal_proposed(proposal_id, recipient, amount, proposer)
withdrawal_executed(proposal_id, recipient, amount)
```

**Arbitration:**
```
dispute_opened(dispute_id, parties, subject)
vote_cast(dispute_id, arbitrator, vote, weight)
dispute_resolved(dispute_id, outcome, timestamp)
```

### Indexer Schema

```sql
-- Core bond tracking
CREATE TABLE bonds (
    identity TEXT PRIMARY KEY,
    contract_id TEXT NOT NULL,
    bonded_amount NUMERIC(38,0) NOT NULL,
    slashed_amount NUMERIC(38,0) DEFAULT 0,
    available_amount NUMERIC(38,0) NOT NULL,
    tier TEXT NOT NULL,
    is_rolling BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,
    bond_start BIGINT NOT NULL,
    bond_end BIGINT NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_bonds_tier ON bonds(tier);
CREATE INDEX idx_bonds_active ON bonds(is_active);
CREATE INDEX idx_bonds_created ON bonds(created_at);

-- Bond history (all state changes)
CREATE TABLE bond_events (
    id SERIAL PRIMARY KEY,
    identity TEXT NOT NULL,
    event_type TEXT NOT NULL,
    amount NUMERIC(38,0),
    timestamp BIGINT NOT NULL,
    ledger_sequence BIGINT NOT NULL,
    transaction_hash TEXT NOT NULL,
    event_data JSONB NOT NULL,
    indexed_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_bond_events_identity ON bond_events(identity);
CREATE INDEX idx_bond_events_type ON bond_events(event_type);
CREATE INDEX idx_bond_events_timestamp ON bond_events(timestamp);

-- Slashing history
CREATE TABLE slash_events (
    id SERIAL PRIMARY KEY,
    identity TEXT NOT NULL,
    slash_amount NUMERIC(38,0) NOT NULL,
    total_slashed NUMERIC(38,0) NOT NULL,
    reason TEXT NOT NULL,
    admin TEXT NOT NULL,
    is_full_slash BOOLEAN NOT NULL,
    timestamp BIGINT NOT NULL,
    ledger_sequence BIGINT NOT NULL,
    transaction_hash TEXT NOT NULL
);

CREATE INDEX idx_slash_events_identity ON slash_events(identity);
CREATE INDEX idx_slash_events_timestamp ON slash_events(timestamp);

-- Attestations
CREATE TABLE attestations (
    id BIGINT PRIMARY KEY,
    subject TEXT NOT NULL,
    attester TEXT NOT NULL,
    data TEXT NOT NULL,
    contract_id TEXT NOT NULL,
    deadline BIGINT,
    created_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_attestations_subject ON attestations(subject);
CREATE INDEX idx_attestations_attester ON attestations(attester);
CREATE INDEX idx_attestations_active ON attestations(is_active);

-- Protocol metrics (aggregated)
CREATE TABLE protocol_metrics (
    timestamp TIMESTAMP PRIMARY KEY,
    total_bonded NUMERIC(38,0) NOT NULL,
    total_slashed NUMERIC(38,0) NOT NULL,
    active_bonds INTEGER NOT NULL,
    bronze_count INTEGER NOT NULL,
    silver_count INTEGER NOT NULL,
    gold_count INTEGER NOT NULL,
    platinum_count INTEGER NOT NULL,
    total_attestations INTEGER NOT NULL,
    treasury_balance NUMERIC(38,0) NOT NULL
);

CREATE INDEX idx_metrics_timestamp ON protocol_metrics(timestamp);
```

## Metrics Collection

### Key Metrics

**Protocol Health:**
- `trustforge_total_bonded_amount` - Total value locked
- `trustforge_active_bonds_count` - Number of active bonds
- `trustforge_supply_cap_utilization` - Percentage of supply cap used
- `trustforge_treasury_balance` - Total treasury holdings

**Bond Distribution:**
- `trustforge_bonds_by_tier{tier="bronze|silver|gold|platinum"}` - Count per tier
- `trustforge_rolling_bonds_count` - Active rolling bonds
- `trustforge_fixed_bonds_count` - Active fixed bonds

**Operations:**
- `trustforge_bond_creations_total` - Counter
- `trustforge_withdrawals_total{type="normal|early"}` - Counter
- `trustforge_slashing_events_total` - Counter
- `trustforge_attestations_total` - Counter

**Performance:**
- `trustforge_transaction_success_rate` - Success percentage
- `trustforge_transaction_gas_used` - Histogram
- `trustforge_indexer_lag_seconds` - Event processing delay

**Security:**
- `trustforge_slashing_amount_total` - Total slashed value
- `trustforge_emergency_events_total` - Emergency activations
- `trustforge_failed_transactions_total` - Failed tx count
- `trustforge_pause_events_total` - Circuit breaker activations

### Prometheus Exporter

Example Python exporter:

```python
from prometheus_client import start_http_server, Gauge, Counter
import psycopg2
import time

# Define metrics
total_bonded = Gauge('trustforge_total_bonded_amount', 'Total bonded amount in protocol')
active_bonds = Gauge('trustforge_active_bonds_count', 'Number of active bonds')
bonds_by_tier = Gauge('trustforge_bonds_by_tier', 'Bonds by tier', ['tier'])
slashing_events = Counter('trustforge_slashing_events_total', 'Total slashing events')

def collect_metrics():
    conn = psycopg2.connect("dbname=trustforge user=indexer")
    cur = conn.cursor()
    
    # Total bonded amount
    cur.execute("SELECT SUM(bonded_amount) FROM bonds WHERE is_active = true")
    total_bonded.set(cur.fetchone()[0] or 0)
    
    # Active bonds
    cur.execute("SELECT COUNT(*) FROM bonds WHERE is_active = true")
    active_bonds.set(cur.fetchone()[0])
    
    # Bonds by tier
    cur.execute("SELECT tier, COUNT(*) FROM bonds WHERE is_active = true GROUP BY tier")
    for tier, count in cur.fetchall():
        bonds_by_tier.labels(tier=tier).set(count)
    
    cur.close()
    conn.close()

if __name__ == '__main__':
    start_http_server(8000)
    while True:
        collect_metrics()
        time.sleep(15)  # Collect every 15 seconds
```

## Dashboards

### Executive Dashboard

**Key Metrics (Single Numbers):**
- Total Value Locked (TVL)
- Active Bonds Count
- 24h Bond Creation Volume
- Treasury Balance
- Supply Cap Utilization %

**Charts:**
- TVL over time (line chart, 30 days)
- Bonds by tier (pie chart)
- Daily bond creations (bar chart, 7 days)
- Slashing events timeline (event markers)

### Operations Dashboard

**System Health:**
- Transaction success rate (gauge, target >99%)
- Event indexer lag (gauge, target <30s)
- Average gas per operation (line chart)
- Error rate by contract (heatmap)

**Activity Monitoring:**
- Bond creations/hour (line chart)
- Withdrawals/hour (line chart)
- Attestations/hour (line chart)
- Top-ups/hour (line chart)

**Treasury:**
- Fee collection by source (stacked area)
- Withdrawal proposals pending (counter)
- Treasury balance trend (line chart)

### Security Dashboard

**Threat Detection:**
- Unusual slashing activity (anomaly detection)
- Large withdrawals (>10K USDC, list)
- Failed admin operations (counter)
- Emergency mode activations (timeline)

**Audit Trail:**
- Recent slashing events (table)
- Admin operations log (table)
- Multi-sig proposal status (table)
- Pause events (timeline)

## Alerting Rules

### Critical Alerts (PagerDuty)

```yaml
groups:
  - name: trustforge_critical
    interval: 30s
    rules:
      - alert: TrustForgeContractPaused
        expr: trustforge_pause_events_total > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "TrustForge contract has been paused"
          description: "Emergency pause activated. Immediate investigation required."
      
      - alert: TrustForgeHighTransactionFailureRate
        expr: rate(trustforge_failed_transactions_total[5m]) / rate(trustforge_transactions_total[5m]) > 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Transaction failure rate exceeds 5%"
          description: "{{ $value | humanizePercentage }} of transactions failing."
      
      - alert: TrustForgeIndexerLagHigh
        expr: trustforge_indexer_lag_seconds > 300
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Event indexer is severely lagging"
          description: "Indexer is {{ $value }} seconds behind ledger."
      
      - alert: TrustForgeEmergencyWithdrawal
        expr: increase(trustforge_emergency_withdrawals_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Emergency withdrawal executed"
          description: "Emergency withdrawal detected. Verify legitimacy immediately."
```

### Warning Alerts (Slack/Discord)

```yaml
  - name: trustforge_warnings
    interval: 1m
    rules:
      - alert: TrustForgeSupplyCapNearLimit
        expr: trustforge_supply_cap_utilization > 90
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Supply cap utilization above 90%"
          description: "{{ $value | humanizePercentage }} of supply cap used. Consider increasing."
      
      - alert: TrustForgeLargeSlashing
        expr: increase(trustforge_slashing_amount_total[1h]) > 10000
        for: 0m
        labels:
          severity: warning
        annotations:
          summary: "Large slashing event detected"
          description: "{{ $value }} USDC slashed in the last hour."
      
      - alert: TrustForgeTreasuryBalanceLow
        expr: trustforge_treasury_balance < 1000
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Treasury balance below minimum threshold"
          description: "Treasury has {{ $value }} USDC. Top up recommended."
      
      - alert: TrustForgeUnusualWithdrawalVolume
        expr: rate(trustforge_withdrawals_total[1h]) > 2 * rate(trustforge_withdrawals_total[1h] offset 24h)
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Withdrawal volume 2x normal"
          description: "Unusual withdrawal activity detected."
```

## Incident Response

### Severity Levels

| Level | Response Time | Escalation | Example |
|-------|---------------|------------|---------|
| **Critical (P0)** | <15 minutes | Immediate page | Contract exploit, funds at risk |
| **High (P1)** | <1 hour | Page if no response | High tx failure rate, indexer down |
| **Medium (P2)** | <4 hours | Slack alert | Supply cap near limit, high gas |
| **Low (P3)** | <24 hours | Ticket | Documentation issue, minor UI bug |

### Incident Playbooks

#### Playbook 1: High Transaction Failure Rate

**Symptoms:**
- `trustforge_failed_transactions_total` spiking
- User reports of failed bond creations/withdrawals

**Investigation:**
1. Check Stellar network status: https://status.stellar.org
2. Query recent failed transactions:
   ```sql
   SELECT * FROM transactions 
   WHERE success = false 
   ORDER BY timestamp DESC LIMIT 20;
   ```
3. Check contract event logs for error codes
4. Verify gas parameters haven't changed
5. Check if specific contract paused

**Resolution:**
- If Stellar network issue: Wait for resolution, communicate status
- If gas issue: Update gas estimates
- If contract bug: Prepare emergency patch
- If supply cap: Increase cap via governance

#### Playbook 2: Indexer Lag Spike

**Symptoms:**
- `trustforge_indexer_lag_seconds` > 300
- Dashboards showing stale data
- API queries returning outdated state

**Investigation:**
1. Check indexer service health: `systemctl status trustforge-indexer`
2. Check database load: `SELECT * FROM pg_stat_activity;`
3. Check Soroban RPC connectivity
4. Review indexer logs for errors

**Resolution:**
- Restart indexer service if hung
- Scale database if CPU/memory maxed
- Clear old processed events if disk full
- Optimize slow queries

#### Playbook 3: Suspicious Slashing Activity

**Symptoms:**
- Multiple slashing events in short time
- Large slashing amounts
- Unknown admin address

**Investigation:**
1. Query recent slashing events:
   ```sql
   SELECT * FROM slash_events 
   WHERE timestamp > extract(epoch from now() - interval '1 hour')
   ORDER BY slash_amount DESC;
   ```
2. Verify admin addresses are legitimate
3. Check if governance proposals exist for slashes
4. Review slash reasons/evidence

**Resolution:**
- If legitimate: Document and communicate
- If unauthorized: Activate emergency pause immediately
- Prepare incident report and compensation plan if needed

## Log Management

### Structured Logging

All services should emit structured JSON logs:

```json
{
  "timestamp": "2026-01-15T10:30:45.123Z",
  "level": "INFO",
  "service": "event-indexer",
  "message": "Indexed bond_created_v2 event",
  "context": {
    "identity": "GABC...",
    "amount": "1000000000000000000000",
    "tier": "bronze",
    "ledger_sequence": 12345678,
    "transaction_hash": "abc123..."
  }
}
```

### Log Retention

| Log Type | Retention | Storage |
|----------|-----------|---------|
| Application logs | 30 days | CloudWatch/Loki |
| Audit logs | 2 years | S3/Archive |
| Transaction logs | 1 year | Database |
| Security events | 5 years | Immutable storage |

### Log Queries

**Recent errors:**
```
level="ERROR" | json | filter timestamp > 1h ago
```

**Slashing events:**
```
message="slash_bond executed" | json | select identity, amount, reason
```

**Failed transactions by user:**
```
success=false | json | stats count by identity
```

## Performance Benchmarks

### Expected Latency

| Operation | p50 | p95 | p99 |
|-----------|-----|-----|-----|
| Bond creation | <3s | <5s | <8s |
| Withdrawal | <2s | <4s | <6s |
| Attestation | <1s | <2s | <3s |
| Query (indexed) | <50ms | <200ms | <500ms |

### Throughput Targets

- Sustained: 10 TPS
- Peak: 50 TPS
- Indexer: <30s lag at peak load

## Maintenance Windows

**Scheduled Maintenance:**
- Weekly indexer database maintenance: Sundays 02:00-04:00 UTC
- Monthly contract parameter review: First Monday of month
- Quarterly disaster recovery drill: TBD

**During Maintenance:**
- Post status page update 24h in advance
- Ensure read-only mode for queries
- Keep alert channels active
- Document any issues encountered

## Runbook Repository

Maintain updated runbooks at:
- `/docs/runbooks/high-tx-failure-rate.md`
- `/docs/runbooks/indexer-lag-spike.md`
- `/docs/runbooks/suspicious-slashing.md`
- `/docs/runbooks/emergency-pause-activation.md`
- `/docs/runbooks/supply-cap-increase.md`

## Contact Information

**On-Call Rotation:**
- Primary: ops-oncall@trustforge.io
- Secondary: engineering-oncall@trustforge.io
- Escalation: cto@trustforge.io

**External Vendors:**
- Stellar Support: https://stellar.org/support
- Database Provider: [your provider]
- Cloud Infrastructure: [your provider]

---

**Last Updated**: January 2026  
**Owner**: TrustForge Operations Team
