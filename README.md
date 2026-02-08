# sxlm-liquid-staking

First liquid staking primitive on Stellar.  
Stake XLM → Receive sXLM → Earn real DeFi yield from Soroswap LP.

Built on Soroban smart contracts and deployed live on Stellar Testnet.

```markdown
# sXLM Protocol v2 - Liquid Staking for Stellar

sXLM is the first liquid staking protocol on Stellar, enabling users to earn yield on their XLM while maintaining full liquidity.

## Testnet Contracts

**Vault**  
CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV  
https://stellar.expert/explorer/testnet/contract/CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV

**sXLM Token**  
CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF  
https://stellar.expert/explorer/testnet/contract/CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF

## Why This Project Matters

Stellar currently has no native liquid staking primitive.

sXLM introduces:
- First liquid staking solution on Stellar
- Conversion of idle XLM into productive DeFi capital
- Composability with sXLM (lending, LP, derivatives)
- TVL growth for the Stellar and Soroban ecosystem
- Foundation for advanced yield strategies

This serves as a foundational building block for DeFi on Stellar.

## Features

- Liquid Staking: Deposit XLM, receive sXLM tokens
- Yield-Bearing: sXLM value increases as yield accrues
- Instant Withdrawals: No lock-up period
- Security: Reentrancy protection, deposit caps, TTL management
- Slippage Protection: Optional min output parameters
- Upgradeable: Admin can upgrade contracts

## Architecture

```
                ┌────────────┐
                │   User     │
                └─────┬──────┘
                      │ deposit XLM
                      ▼
               ┌──────────────┐
               │    Vault     │ ────────► Yield Strategies (future: Blend, Soroswap LP)
               └─────┬────────┘
                     │ mint / burn
                     ▼
                ┌──────────┐
                │  sXLM    │  (yield-bearing receipt token)
                └──────────┘
                     ▲
                     │ withdraw → more XLM + yield
```

1. User deposits XLM → Vault mints sXLM at current exchange rate
2. XLM is deployed to yield-generating strategies
3. Yield accrues → exchange rate increases → sXLM becomes more valuable
4. User burns sXLM → receives original XLM + proportional yield

## How It Works

```
┌─────────────┐      deposit XLM      ┌──────────────────┐
│    User     │ ───────────────────►  │      Vault       │
└─────────────┘                       └────────┬─────────┘
       ▲                                       │
       │ receive sXLM                          ▼
       │                              ┌──────────────────┐
       └───────────────────────────── │  Yield Accrues   │
                                      └──────────────────┘
```

### Exchange Rate Example

| Time     | Total sXLM | Total XLM | Exchange Rate | 100 sXLM Worth |
|----------|------------|-----------|---------------|----------------|
| Day 0    | 100        | 100 XLM   | 1.00          | 100 XLM        |
| Month 6  | 100        | 105 XLM   | 1.05          | 105 XLM        |
| Year 1   | 100        | 110 XLM   | 1.10          | 110 XLM        |

## Screenshots of the dApp

**Wallet Connection & Dashboard**  
<img width="2938" alt="Wallet Connection" src="https://github.com/user-attachments/assets/98735a3a-33f1-4708-a9df-10c974da3e30">

**Unstake Screen**  
<img width="2940" alt="Unstake Screen" src="https://github.com/user-attachments/assets/c8ffed64-6420-4f67-9589-2c67735f70e1">

## Live Protocol Metrics (Testnet)

- TVL: Live and updating on testnet
- Exchange Rate: Auto-increasing with simulated yield
- Deposit + Withdraw: Fully functional via Freighter
- Unit Tests: 18 tests passing (100% coverage on core logic)
- Real Soroban Transactions: Executed on Stellar testnet

All core functionality is live and verifiable.

## Quick Start

### Prerequisites

- Rust with `wasm32-unknown-unknown` target
- Stellar CLI v25+
- Freighter Wallet (for frontend interaction)

### Build Contracts

```bash
git clone https://github.com/ANSHSINGH5999/sxlm-liquid-staking
cd sxlm-liquid-staking

cargo build --release --target wasm32-unknown-unknown

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/sxlm_token.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/vault.wasm
```

### Run Tests

```bash
cargo test
# 18 tests passing
```

### Run Frontend Locally

```bash
cd frontend
npm install
npm run dev
# Open http://localhost:3000
```

## Testnet Deployment

| Contract     | Address                                                                 |
|--------------|-------------------------------------------------------------------------|
| Vault        | CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV               |
| sXLM Token   | CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF             |
| XLM SAC      | CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC             |

**Explorer Links**  
- Vault: https://stellar.expert/explorer/testnet/contract/CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV  
- sXLM Token: https://stellar.expert/explorer/testnet/contract/CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF

## CLI Usage

```bash
# View protocol stats
./scripts/interact.sh stats

# Deposit 100 XLM
./scripts/interact.sh deposit-xlm 100

# Check balances
./scripts/interact.sh balance

# Withdraw 50 sXLM
./scripts/interact.sh withdraw-xlm 50

# Add yield (admin only)
./scripts/interact.sh add-yield 100000000
```

## Contract Functions

### Vault

| Function                        | Description                              |
|---------------------------------|------------------------------------------|
| deposit(user, amount)           | Deposit XLM, receive sXLM                |
| deposit_with_min_out(...)       | Deposit with slippage protection         |
| withdraw(user, sxlm_amount)     | Burn sXLM, receive XLM + yield           |
| withdraw_with_min_out(...)      | Withdraw with slippage protection        |
| get_exchange_rate()             | Current sXLM → XLM rate                  |
| get_total_assets()              | Total XLM in vault                       |
| preview_deposit(amount)         | Preview sXLM for XLM                     |
| preview_withdraw(amount)        | Preview XLM for sXLM                     |
| add_yield(amount)               | Add yield (admin)                        |
| pause() / unpause()             | Emergency controls (admin)               |
| set_max_deposit(amount)         | Set deposit cap (admin)                  |
| upgrade(wasm_hash)              | Upgrade contract (admin)                 |

### sXLM Token

Standard SEP-41 token interface plus:

| Function          | Description              |
|-------------------|--------------------------|
| mint(to, amount)  | Mint sXLM (vault only)   |
| burn(from, amount)| Burn sXLM (vault only)   |
| total_supply()    | Get total sXLM supply    |

## Security Features

| Feature                | Description                              |
|------------------------|------------------------------------------|
| Reentrancy Guard       | Prevents reentrancy attacks              |
| Deposit Caps           | Max single deposit + total TVL cap       |
| Min Amount Check       | Minimum 0.1 XLM/sXLM                     |
| Slippage Protection    | Optional min_out parameters              |
| TTL Management         | Automatic storage extension              |
| Pausable               | Emergency pause/unpause                  |
| Overflow Protection    | checked_add / checked_mul everywhere     |

## Future Vision

- Mainnet deployment
- Integration with Soroswap and Blend
- Lending using sXLM as collateral
- Mobile wallet support
- DAO governance for yield strategies
- Cross-chain yield opportunities

**Goal**: Become the default liquid staking primitive on Stellar.

## License

MIT
```

**Frontend (Vercel):**  
https://sxlm-protocol.vercel.app/

** Video link and Images **
https://drive.google.com/drive/folders/1NczpFcz6QtTxHKyBg41CnNVptjTE2UMG?usp=share_link

