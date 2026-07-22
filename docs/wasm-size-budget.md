# Wasm Size Budget

Soroban contracts have a hard on-chain size limit of ~64 KiB per contract Wasm. Exceeding this limit prevents upgrades and can cause runtime failures.

## Enforced limits

CI enforces per-contract ceilings via [`.github/workflows/wasm-size.yml`](../.github/workflows/wasm-size.yml), which builds release Wasm and runs [`scripts/check_wasm_size.sh`](../scripts/check_wasm_size.sh). Budgets are defined in [`scripts/wasm-size-budget.toml`](../scripts/wasm-size-budget.toml).

| Contract | Budget (KiB) | Budget (bytes) |
| --- | ---: | ---: |
| `admin` | 64 | 65 536 |
| `trustforge_arbitration` | 64 | 65 536 |
| `trustforge_bond` | 64 | 65 536 |
| `trustforge_delegation` | 64 | 65 536 |
| `trustforge_multisig` | 64 | 65 536 |
| `trustforge_treasury` | 64 | 65 536 |
| `templates` | 64 | 65 536 |
| `timelock` | 64 | 65 536 |

The default ceiling for any contract not listed explicitly is **64 KiB** (65 536 bytes).

## How it works

1. The **WASM Size Budget** workflow builds all deployable contracts with `cargo build --target wasm32-unknown-unknown --release --locked -p <contract>…` (pinned toolchain from [`rust-toolchain.toml`](../rust-toolchain.toml)).
2. `scripts/check_wasm_size.sh` scans `target/wasm32-unknown-unknown/release/*.wasm` and fails if any artifact exceeds its configured budget.
3. Debug symbols are stripped via the workspace `profile.release.strip = "symbols"` setting, so only deployable code size is measured.

Pass a single argument to override all limits for a one-off local check, e.g. `./scripts/check_wasm_size.sh 60` for a 60 KiB ceiling.

## Local validation

```bash
cargo build --target wasm32-unknown-unknown --release --locked \
  -p admin -p trustforge_arbitration -p trustforge_bond -p trustforge_delegation \
  -p trustforge_multisig -p trustforge_treasury -p templates -p timelock
chmod +x scripts/check_wasm_size.sh
./scripts/check_wasm_size.sh
```

The script prints pass/fail for each contract. See also [wasm-reproducibility.md](wasm-reproducibility.md) for hash pinning of bond and delegation artifacts.

## Adjusting a budget

If a contract legitimately needs more space:

1. Raise its entry under `[contracts]` in `scripts/wasm-size-budget.toml`.
2. Update the table in this document.
3. Land the change in the same PR as the binary growth, with reviewer approval.
