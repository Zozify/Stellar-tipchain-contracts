# Stellar-tipchain-contracts

A Rust + Soroban smart contract for a Stellar-based tipping application. Supporters can tip creators with any Stellar token, funds are held in contract escrow, and creators withdraw at any time. Every tip and withdrawal is recorded on-chain via events.

> ⚠️ **Work in Progress — ~30% complete.** Only contract initialisation is implemented. See the [progress tracker](#progress-tracker) below.

---

## Progress Tracker

| # | Feature | Status |
|---|---|---|
| 1 | Workspace & crate scaffolding | ✅ Done |
| 2 | `DataKey` storage enum | ✅ Done |
| 3 | `init(token)` — one-time token config | ✅ Done |
| 4 | `tip(sender, creator, amount)` — escrow transfer + storage update | 🔲 TODO |
| 5 | `get_total_tips(creator)` — read cumulative total | 🔲 TODO |
| 6 | `withdraw(creator)` — release escrowed balance | 🔲 TODO |
| 7 | On-chain events for tip and withdraw | 🔲 TODO |
| 8 | Full unit test suite | 🔲 TODO |
| 9 | Testnet deploy script | 🔲 TODO |

---

## Table of Contents

- [Overview](#overview)
- [How It Works](#how-it-works)
- [Repository Structure](#repository-structure)
- [Contract Functions](#contract-functions)
- [Storage Model](#storage-model)
- [Events](#events)
- [Authorization Model](#authorization-model)
- [Prerequisites](#prerequisites)
- [Build](#build)
- [Test](#test)
- [Deploy (Testnet)](#deploy-testnet)
- [Interacting with the Contract](#interacting-with-the-contract)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

Stellar-tipchain-contracts is a Soroban smart contract written in Rust that enables a creator tipping economy on the Stellar network. It compiles to a WASM binary deployed on-chain and interacted with via the Stellar CLI or any Soroban-compatible client.

The contract acts as a **non-custodial escrow**: when a supporter tips a creator, tokens move from the supporter's wallet into the contract's own on-chain account. The contract records how much of that pool belongs to each creator. When a creator is ready to collect, they call `withdraw` and the contract releases exactly their share — no admin, no intermediary.

Key design properties:

- **Non-custodial escrow** — tipped tokens are held by the contract itself, not any third-party wallet.
- **Any Stellar token** — configured at deploy time with any SEP-41 compliant token address (XLM, USDC, or custom tokens).
- **Dual balance tracking** — each creator has two independent counters: a withdrawable balance (resets on withdrawal) and a cumulative all-time total (never decremented). This lets you show lifetime earnings separately from available balance.
- **On-chain events** — `tip` and `withdraw` each emit a structured Soroban event, making it straightforward for off-chain indexers, explorers, and notification services to track activity.
- **No admin key** — once initialised, no privileged account exists. Only the creator can withdraw their own funds. The token address cannot be changed.
- **Multi-creator** — a single deployed contract handles any number of creators simultaneously. Each creator's balance is stored independently under their address.

---

## How It Works

### Full Lifecycle

```
1. Deploy contract to Stellar network
        ↓
2. init(token_address)
   - Stores the token contract address in instance storage
   - Can only be called once; panics with "already initialised" on repeat calls
   - No auth required — first caller wins
        ↓
3. tip(sender, creator, amount)                          ← 🔲 TODO
   - sender must sign the transaction (require_auth)
   - Validates amount > 0, panics with "amount must be positive" otherwise
   - Calls token.transfer(sender → contract, amount)
   - Reads CreatorBalance[creator] from persistent storage (default 0)
   - Reads CreatorTotal[creator] from persistent storage (default 0)
   - Writes CreatorBalance[creator] += amount
   - Writes CreatorTotal[creator]   += amount
   - Emits event: topic=("tip", creator), data=(sender, amount)
        ↓
4. withdraw(creator)                                     ← 🔲 TODO
   - creator must sign the transaction (require_auth)
   - Reads CreatorBalance[creator]; panics with "nothing to withdraw" if 0
   - Writes CreatorBalance[creator] = 0  (zeroed before transfer — reentrancy safe)
   - Calls token.transfer(contract → creator, balance)
   - Emits event: topic=("withdraw", creator), data=amount
   - CreatorTotal[creator] is NOT changed — historical record preserved
        ↓
5. get_total_tips(creator) → i128                        ← 🔲 TODO
   - Read-only, no auth required
   - Returns CreatorTotal[creator] from persistent storage
   - Returns 0 if creator has never been tipped
```

### Token Flow

```
Supporter wallet
      │
      │  tip(sender, creator, 500)
      │  token.transfer(sender → contract, 500)
      ▼
Contract escrow account
      │  CreatorBalance[creator] = 500
      │  CreatorTotal[creator]   = 500
      │
      │  withdraw(creator)
      │  token.transfer(contract → creator, 500)
      ▼
Creator wallet
      │  CreatorBalance[creator] = 0
      │  CreatorTotal[creator]   = 500  ← unchanged
```

### Dual Balance Design

The contract maintains two separate counters per creator:

| Storage key | Tracks | On tip | On withdraw |
|---|---|---|---|
| `CreatorBalance(addr)` | Withdrawable escrow balance | `+= amount` | reset to `0` |
| `CreatorTotal(addr)` | All-time cumulative tips | `+= amount` | unchanged |

This separation is intentional. A creator who has received 10,000 tokens across 50 tips and withdrawn twice still shows 10,000 as their lifetime total, while their current withdrawable balance reflects only what has accumulated since their last withdrawal.

### Storage Tiers

Soroban has three storage tiers. This contract uses two:

- **Instance storage** — used for `DataKey::Token`. Tied to the contract instance lifetime, cheap to access, appropriate for a single value that never changes after `init`.
- **Persistent storage** — used for `CreatorBalance` and `CreatorTotal`. Survives ledger expiry and is the correct choice for per-user data that must be retained indefinitely across many ledger windows.

---

## Repository Structure

```
Stellar-tipchain-contracts/
├── Cargo.toml                  # Workspace manifest (soroban-sdk 22.0.8)
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── contracts/
│   └── tipjar/
│       ├── Cargo.toml          # cdylib + rlib, soroban-sdk dep
│       └── src/
│           └── lib.rs          # Contract logic + unit tests
└── scripts/
    └── deploy.sh               # Testnet deploy helper (not yet implemented)
```

The workspace uses a single `[workspace.dependencies]` entry to pin `soroban-sdk` to `22.0.8` across all crates. The contract crate declares `crate-type = ["cdylib", "rlib"]` so it can compile to both WASM (for on-chain deployment) and a native library (for unit tests).

---

## Contract Functions

### `init(token: Address)` ✅ Implemented

Stores the token contract address used for all future transfers. Guards against re-initialisation.

```rust
pub fn init(env: Env, token: Address) {
    if env.storage().instance().has(&DataKey::Token) {
        panic!("already initialised");
    }
    env.storage().instance().set(&DataKey::Token, &token);
}
```

```bash
stellar contract invoke --id <CONTRACT_ID> -- init --token <TOKEN_ADDRESS>
```

---

### `tip(sender: Address, creator: Address, amount: i128)` 🔲 TODO

Moves `amount` tokens from `sender` into the contract's escrow and credits the creator's balances.

Planned behaviour:
- `sender.require_auth()` — the sender's wallet must sign
- Panics `"amount must be positive"` if `amount <= 0`
- Calls `token::Client::transfer(sender → contract, amount)`
- Increments `CreatorBalance[creator]` and `CreatorTotal[creator]` in persistent storage
- Emits `("tip", creator)` event with data `(sender, amount)`

```bash
stellar contract invoke --id <CONTRACT_ID> -- tip \
  --sender <SENDER_ADDRESS> \
  --creator <CREATOR_ADDRESS> \
  --amount 500
```

---

### `get_total_tips(creator: Address) → i128` 🔲 TODO

Returns the all-time cumulative tips received by `creator`. Read-only, no auth required. Returns `0` for a creator who has never been tipped.

```bash
stellar contract invoke --id <CONTRACT_ID> -- get_total_tips \
  --creator <CREATOR_ADDRESS>
```

---

### `withdraw(creator: Address)` 🔲 TODO

Releases the creator's entire escrowed balance to their wallet.

Planned behaviour:
- `creator.require_auth()` — the creator's wallet must sign
- Panics `"nothing to withdraw"` if `CreatorBalance[creator]` is `0`
- Zeroes `CreatorBalance[creator]` before the transfer (prevents reentrancy)
- Calls `token::Client::transfer(contract → creator, balance)`
- Emits `("withdraw", creator)` event with data `amount`
- `CreatorTotal[creator]` is not modified

```bash
stellar contract invoke --id <CONTRACT_ID> -- withdraw \
  --creator <CREATOR_ADDRESS>
```

---

## Storage Model

| Key | Storage tier | Value type | Description |
|---|---|---|---|
| `DataKey::Token` | Instance | `Address` | Token contract address set at init; never changes |
| `DataKey::CreatorBalance(Address)` | Persistent | `i128` | Current withdrawable balance for a creator |
| `DataKey::CreatorTotal(Address)` | Persistent | `i128` | All-time cumulative tips received by a creator |

`i128` is used for token amounts because Stellar token balances are represented as signed 128-bit integers in the Soroban token interface.

---

## Events

> 🔲 Not yet implemented — planned for `tip()` and `withdraw()`.

Soroban events have a **topic** (used for filtering/indexing) and a **data** payload.

| Topic | Data | Emitted by |
|---|---|---|
| `("tip", creator: Address)` | `(sender: Address, amount: i128)` | `tip()` |
| `("withdraw", creator: Address)` | `amount: i128` | `withdraw()` |

Off-chain consumers (explorers, notification bots, analytics dashboards) can subscribe to these events by filtering on the contract ID and topic.

---

## Authorization Model

Soroban uses explicit per-call authorization — each function that moves funds calls `require_auth()` on the account that must sign.

| Function | Who must sign | Why |
|---|---|---|
| `init` | No one (open) | First caller wins; no funds involved |
| `tip` | `sender` | Tokens leave sender's wallet |
| `withdraw` | `creator` | Tokens are released to creator's wallet |

There is no contract owner or admin role. After `init`, the contract is fully autonomous — no account can change the token, freeze funds, or redirect withdrawals.

---

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)
- Soroban WASM target:

```bash
rustup target add wasm32v1-none
```

- A funded Stellar testnet account (for deployment):

```bash
stellar keys generate default --network testnet --fund
```

---

## Build

```bash
cargo build -p tipjar --target wasm32v1-none --release
```

Output: `target/wasm32v1-none/release/tipjar.wasm`

---

## Test

```bash
cargo test -p tipjar
```

Tests run in Soroban's in-process test environment — no network or CLI required.

| Test | What it covers | Status |
|---|---|---|
| `test_init` | `init()` stores token address without panic | ✅ Passes |
| `test_init_twice_panics` | Second `init()` call panics with `"already initialised"` | ✅ Passes |
| `test_tip_and_totals` | Tips accumulate in both balance and total | 🔲 TODO |
| `test_withdraw` | Creator receives correct token amount after withdrawal | 🔲 TODO |
| `test_invalid_tip_amount` | Zero/negative tip amount panics correctly | 🔲 TODO |

---

## Deploy (Testnet)

> 🔲 Deploy script not yet implemented.

```bash
bash scripts/deploy.sh
```

Manual steps (once script is ready):

```bash
# 1. Build
cargo build -p tipjar --target wasm32v1-none --release

# 2. Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/tipjar.wasm \
  --source default \
  --network testnet

# 3. Initialise
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source default \
  --network testnet \
  -- init \
  --token $(stellar contract id asset --asset native --network testnet)
```

---

## Interacting with the Contract

> 🔲 Only `init` is callable. `tip`, `get_total_tips`, and `withdraw` panic with `unimplemented!` until implemented.

```bash
# Initialise (one time only)
stellar contract invoke --id <CONTRACT_ID> -- init --token <TOKEN_ADDRESS>
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for branching strategy, coding standards, test requirements, and the pull request checklist.

---

## License

MIT
