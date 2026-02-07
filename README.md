# sXLM Protocol v2 - Liquid Staking for Stellar

sXLM is the first liquid staking protocol on Stellar, enabling users to earn yield on their XLM while maintaining liquidity.

## Features

- **Liquid Staking**: Deposit XLM, receive sXLM tokens
- **Yield-Bearing**: sXLM value increases as yield accrues
- **Instant Withdrawals**: No lock-up period
- **Security**: Reentrancy protection, deposit caps, TTL management
- **Slippage Protection**: Optional min output parameters
- **Upgradeable**: Admin can upgrade contracts

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/stellar-cli) v25+
- [Freighter Wallet](https://www.freighter.app/) (for frontend)

### Build

```bash
git clone https://github.com/yourrepo/sxlm-protocol
cd sxlm-protocol

# Build contracts
cargo build --release --target wasm32-unknown-unknown

# Optimize WASM
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/sxlm_token.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/vault.wasm
```

### Run Tests

```bash
cargo test
# 18 tests passing
```

### Run Frontend

```bash
cd frontend
npx serve .
# Open http://localhost:3000
```

## Testnet Deployment

| Contract | Address |
|----------|---------|
| **Vault** | `CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV` |
| **sXLM Token** | `CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF` |
| **XLM SAC** | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |

**Explorer Links:**
- [Vault Contract](https://stellar.expert/explorer/testnet/contract/CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV)
- [sXLM Token](https://stellar.expert/explorer/testnet/contract/CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF)

## CLI Usage

```bash
# View protocol stats
./scripts/interact.sh stats

# Deposit 100 XLM
./scripts/interact.sh deposit-xlm 100

# Check balances
./scripts/interact.sh balance

# Withdraw 50 sXLM (get XLM + yield)
./scripts/interact.sh withdraw-xlm 50

# Add yield (admin only)
./scripts/interact.sh add-yield 100000000
```

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

1. **Deposit**: User deposits XLM → Vault mints sXLM at current exchange rate
2. **Yield**: XLM deployed to strategies → Yield accrues in vault
3. **Exchange Rate**: As yield accrues, 1 sXLM becomes worth more XLM
4. **Withdraw**: User burns sXLM → Receives XLM + proportional yield

### Exchange Rate Example

| Time | Total sXLM | Total XLM | Exchange Rate | 100 sXLM Worth |
|------|------------|-----------|---------------|----------------|
| Day 0 | 100 | 100 XLM | 1.00 | 100 XLM |
| Month 6 | 100 | 105 XLM | 1.05 | 105 XLM |
| Year 1 | 100 | 110 XLM | 1.10 | 110 XLM |

## Contract Functions

### Vault

| Function | Description |
|----------|-------------|
| `deposit(user, amount)` | Deposit XLM, receive sXLM |
| `deposit_with_min_out(user, amount, min_out)` | Deposit with slippage protection |
| `withdraw(user, sxlm_amount)` | Burn sXLM, receive XLM + yield |
| `withdraw_with_min_out(user, amount, min_out)` | Withdraw with slippage protection |
| `get_exchange_rate()` | Current sXLM → XLM rate |
| `get_total_assets()` | Total XLM in vault |
| `preview_deposit(amount)` | Preview sXLM for XLM |
| `preview_withdraw(amount)` | Preview XLM for sXLM |
| `add_yield(amount)` | Add yield (admin) |
| `pause()` / `unpause()` | Emergency controls (admin) |
| `set_max_deposit(amount)` | Set deposit cap (admin) |
| `upgrade(wasm_hash)` | Upgrade contract (admin) |

### sXLM Token

Standard SEP-41 token interface plus:

| Function | Description |
|----------|-------------|
| `mint(to, amount)` | Mint sXLM (vault only) |
| `burn(from, amount)` | Burn sXLM (vault only) |
| `total_supply()` | Get total sXLM supply |

## Architecture

```
sxlm-protocol/
├── contracts/
│   ├── sxlm-token/          # SEP-41 token with TTL management
│   │   └── src/lib.rs       # 16 tests
│   └── vault/               # Core deposit/withdraw/yield logic
│       └── src/lib.rs       # 2 tests + security features
├── frontend/
│   └── index.html           # Web UI with real Soroban transactions
├── scripts/
│   └── interact.sh          # CLI interaction tool
├── deployments/
│   └── testnet.json         # Contract addresses
├── pitch-deck.html          # Presentation (9 slides)
└── README.md
```

## Security Features

| Feature | Description |
|---------|-------------|
| **Reentrancy Guard** | Prevents reentrancy attacks |
| **Deposit Caps** | Max single deposit + total TVL cap |
| **Min Amount** | Minimum 0.1 XLM for deposit/withdraw |
| **Slippage Protection** | Optional min output parameters |
| **TTL Management** | Automatic storage TTL extension |
| **Pausable** | Emergency pause functionality |
| **Overflow Protection** | checked_add/mul for all math |

## Roadmap

- [x] Core contracts (sXLM token + vault)
- [x] Testnet deployment
- [x] Web frontend with Freighter
- [x] Security patterns (reentrancy, caps, TTL)
- [x] 18 unit tests
- [ ] Blend protocol integration
- [ ] Security audit
- [ ] Mainnet deployment
- [ ] Governance token

## License

MIT
# stake
