# sXLM Protocol — Liquid Staking for Stellar

**First liquid staking primitive on Stellar**

Stake XLM → Receive **sXLM** → Earn real DeFi yield (Soroswap LP, future strategies)

Built using **Soroban smart contracts** and deployed live on **Stellar Testnet**.

---

## 🔗 Live Links

- **Frontend (Vercel):** https://sxlm-protocol.vercel.app  
- **Demo Video & Screenshots:**  
  https://drive.google.com/drive/folders/1NczpFcz6QtTxHKyBg41CnNVptjTE2UMG

---

## 📌 Overview

`sXLM` is the **first liquid staking protocol on Stellar**, allowing users to earn yield on their XLM **without locking liquidity**.

Users deposit XLM into a vault and receive **sXLM**, a yield-bearing receipt token whose value increases as yield accrues.  
sXLM remains fully composable across the Stellar DeFi ecosystem.

---

## 🚀 Why This Project Matters

Stellar currently has **no native liquid staking primitive**.

`sXLM` introduces:

- ✅ First liquid staking solution on Stellar  
- ✅ Converts idle XLM into productive DeFi capital  
- ✅ Fully composable receipt token (LPs, lending, derivatives)  
- ✅ Drives TVL growth for Stellar & Soroban  
- ✅ Foundation for advanced yield strategies  

**This protocol is a core DeFi building block for Stellar.**

---

## ✨ Key Features

- **Liquid Staking** — Deposit XLM, receive sXLM instantly  
- **Yield-Bearing Token** — sXLM value increases over time  
- **Instant Withdrawals** — No lock-ups or unbonding period  
- **Slippage Protection** — Optional `min_out` parameters  
- **Upgradeable Contracts** — Admin-controlled upgrades  
- **Security First** — Reentrancy guards, caps, pausing  

---

## 🧠 High-Level Architecture

            ┌────────────┐
            │   User     │
            └─────┬──────┘
                  │ deposit XLM
                  ▼
           ┌──────────────┐
           │    Vault     │ ─────► Yield Strategies
           └─────┬────────┘        (Soroswap LP, Blend*)
                 │ mint / burn
                 ▼
            ┌──────────┐
            │  sXLM    │  Yield-bearing receipt token
            └──────────┘
                 ▲
                 │ withdraw
                 ▼
              XLM + Yield


---

## ⚙️ How It Works

1. User deposits XLM into the **Vault**
2. Vault mints sXLM based on the current exchange rate
3. XLM is deployed to yield-generating strategies
4. Yield accrues → exchange rate increases
5. User burns sXLM → receives XLM + proportional yield

---

## 📈 Exchange Rate Example

| Time     | Total sXLM | Total XLM | Exchange Rate | 100 sXLM Worth |
|----------|------------|-----------|---------------|----------------|
| Day 0    | 100        | 100       | 1.00          | 100 XLM        |
| Month 6  | 100        | 105       | 1.05          | 105 XLM        |
| Year 1   | 100        | 110       | 1.10          | 110 XLM        |

---

## 🧪 Live Testnet Deployment

### Contracts

| Contract | Address |
|--------|--------|
| **Vault** | `CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV` |
| **sXLM Token** | `CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF` |
| **XLM SAC** | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |

### Explorer Links

- Vault: https://stellar.expert/explorer/testnet/contract/CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV  
- sXLM: https://stellar.expert/explorer/testnet/contract/CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF  

---

## 📊 Protocol Status (Testnet)

- TVL: Live and updating  
- Exchange Rate: Auto-increasing (simulated yield)  
- Wallet Support: Freighter  
- Transactions: Real Soroban transactions on testnet  
- Tests: **18/18 passing** (100% core logic coverage)

---

## 🖼️ dApp Screenshots

### Wallet Connection & Dashboard
<img width="2938" src="https://github.com/user-attachments/assets/98735a3a-33f1-4708-a9df-10c974da3e30" />

### Unstake Screen
<img width="2940" src="https://github.com/user-attachments/assets/c8ffed64-6420-4f67-9589-2c67735f70e1" />

---

## 🛠️ Quick Start

### Prerequisites

- Rust (`wasm32-unknown-unknown`)
- Stellar CLI v25+
- Freighter Wallet
- Node.js (for frontend)

---

## 🛠️ Build Contracts

Clone the repository and compile the Soroban smart contracts.

```bash
git clone https://github.com/ANSHSINGH5999/sxlm-liquid-staking.git
cd sxlm-liquid-staking
```

Build the contracts for the wasm32-unknown-unknown target:
```
cargo build --release --target wasm32-unknown-unknown
```
Build the contracts for the wasm32-unknown-unknown target:
```
cargo build --release --target wasm32-unknown-unknown
```

Optimize the compiled WASM files for deployment:
```
stellar contract optimize \
  --wasm target/wasm32-unknown-unknown/release/sxlm_token.wasm

stellar contract optimize \
  --wasm target/wasm32-unknown-unknown/release/vault.wasm
```
---
## 🧪 Run Tests

Run the full test suite for the protocol:
```
cargo test
```

18 tests passing

100% coverage on core vault logic
---
## 🌐 Run Frontend Locally

Navigate to the frontend directory and start the development server.
```
cd frontend
npm install
npm run dev
```

Open the app in your browser:
```
http://localhost:3000
```
---
## 🧩 CLI Usage

Interact with the protocol directly using the provided scripts.

View protocol statistics:
```
./scripts/interact.sh stats
```

Deposit XLM into the vault:
```
./scripts/interact.sh deposit-xlm 100
```

Check balances:
```
./scripts/interact.sh balance
```

Withdraw sXLM from the vault:
```
./scripts/interact.sh withdraw-xlm 50
```

Add yield to the vault (admin only):
```
./scripts/interact.sh add-yield 100000000
```
---
## Contract Interfaces 

| Function                    | Description                       |
| --------------------------- | --------------------------------- |
| deposit(user, amount)       | Deposit XLM and receive sXLM      |
| deposit_with_min_out        | Deposit with slippage protection  |
| withdraw(user, sxlm_amount) | Burn sXLM and receive XLM         |
| withdraw_with_min_out       | Withdraw with slippage protection |
| preview_deposit             | Preview sXLM output               |
| preview_withdraw            | Preview XLM output                |
| get_exchange_rate           | Current sXLM → XLM rate           |
| get_total_assets            | Total XLM in vault                |
| add_yield                   | Add yield (admin)                 |
| pause / unpause             | Emergency controls                |
| set_max_deposit             | Set deposit cap                   |
| upgrade                     | Upgrade contract                  |

---

## sXLM Token Contract (SEP-41)

| Function     | Description            |
| ------------ | ---------------------- |
| mint         | Mint sXLM (vault only) |
| burn         | Burn sXLM (vault only) |
| total_supply | Total sXLM supply      |
---

## 🔐 Security Features

Reentrancy protection

Deposit caps and TVL limits

Minimum deposit and withdrawal checks

Optional slippage protection

Pausable contracts for emergencies

Checked arithmetic to prevent overflows

Automatic TTL extension for storage

---
## 🔮 Future Roadmap

Stellar mainnet deployment

Soroswap LP integration

Blend protocol integration

Lending using sXLM as collateral

DAO governance for yield strategies

Mobile wallet support

Cross-chain yield opportunities
---
🏁 Goal

Become the default liquid staking primitive on Stellar.
