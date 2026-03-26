#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, IntoVal, Symbol,
};

// Precision for exchange rate calculations (7 decimals)
const PRECISION: i128 = 1_0000000;

// Minimum deposit/withdraw amounts (0.1 XLM)
const MIN_AMOUNT: i128 = 1000000;

// Maximum single deposit (1M XLM) - can be adjusted by admin
const DEFAULT_MAX_DEPOSIT: i128 = 1_000_000_0000000;

// Storage TTL constants
const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS;
const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const PERSISTENT_LIFETIME_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    XlmToken,
    SxlmToken,
    TotalDeposits,
    YieldAccrued,
    Initialized,
    Paused,
    MaxDeposit,
    TotalCap,        // Maximum total TVL
    UserDeposit(Address), // Track per-user deposits
    Reentrancy,      // Reentrancy guard
}

#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum ReentrancyState {
    Unlocked,
    Locked,
}

// ============ Storage Helpers ============

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    if env.storage().persistent().has(key) {
        env.storage()
            .persistent()
            .extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }
}

fn get_xlm_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::XlmToken).unwrap()
}

fn get_sxlm_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::SxlmToken).unwrap()
}

fn get_total_deposits(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalDeposits)
        .unwrap_or(0)
}

fn set_total_deposits(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::TotalDeposits, &amount);
}

fn get_yield_accrued(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::YieldAccrued)
        .unwrap_or(0)
}

fn set_yield_accrued(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::YieldAccrued, &amount);
}

fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

fn get_max_deposit(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::MaxDeposit)
        .unwrap_or(DEFAULT_MAX_DEPOSIT)
}

fn get_total_cap(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalCap)
        .unwrap_or(i128::MAX)
}

fn get_user_deposit(env: &Env, user: &Address) -> i128 {
    let key = DataKey::UserDeposit(user.clone());
    let amount = env.storage().persistent().get(&key).unwrap_or(0);
    if amount > 0 {
        extend_persistent_ttl(env, &key);
    }
    amount
}

fn set_user_deposit(env: &Env, user: &Address, amount: i128) {
    let key = DataKey::UserDeposit(user.clone());
    env.storage().persistent().set(&key, &amount);
    extend_persistent_ttl(env, &key);
}

// ============ Reentrancy Guard ============

fn require_not_reentering(env: &Env) {
    let state: ReentrancyState = env
        .storage()
        .instance()
        .get(&DataKey::Reentrancy)
        .unwrap_or(ReentrancyState::Unlocked);

    if state == ReentrancyState::Locked {
        panic!("Reentrant call detected");
    }
}

fn lock_reentrancy(env: &Env) {
    env.storage()
        .instance()
        .set(&DataKey::Reentrancy, &ReentrancyState::Locked);
}

fn unlock_reentrancy(env: &Env) {
    env.storage()
        .instance()
        .set(&DataKey::Reentrancy, &ReentrancyState::Unlocked);
}

#[contract]
pub struct Vault;

#[contractimpl]
impl Vault {
    /// Initialize the vault
    pub fn initialize(
        env: Env,
        admin: Address,
        xlm_token: Address,
        sxlm_token: Address,
    ) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::XlmToken, &xlm_token);
        env.storage().instance().set(&DataKey::SxlmToken, &sxlm_token);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::MaxDeposit, &DEFAULT_MAX_DEPOSIT);
        env.storage().instance().set(&DataKey::Reentrancy, &ReentrancyState::Unlocked);
        set_total_deposits(&env, 0);
        set_yield_accrued(&env, 0);
        extend_instance_ttl(&env);
    }

    /// Deposit XLM and receive sXLM
    /// Returns the amount of sXLM minted
    pub fn deposit(env: Env, user: Address, amount: i128) -> i128 {
        extend_instance_ttl(&env);
        require_not_reentering(&env);
        lock_reentrancy(&env);

        // Validations
        user.require_auth();

        if is_paused(&env) {
            unlock_reentrancy(&env);
            panic!("Vault is paused");
        }

        if amount < MIN_AMOUNT {
            unlock_reentrancy(&env);
            panic!("Amount below minimum (0.1 XLM)");
        }

        if amount > get_max_deposit(&env) {
            unlock_reentrancy(&env);
            panic!("Amount exceeds maximum single deposit");
        }

        let total_after = get_total_deposits(&env).checked_add(amount).expect("Overflow");
        if total_after > get_total_cap(&env) {
            unlock_reentrancy(&env);
            panic!("Deposit would exceed total cap");
        }

        let xlm_token = get_xlm_token(&env);
        let sxlm_token = get_sxlm_token(&env);

        // Calculate sXLM to mint based on exchange rate
        let exchange_rate = Self::get_exchange_rate(env.clone());
        let sxlm_to_mint = amount
            .checked_mul(PRECISION)
            .expect("Overflow")
            .checked_div(exchange_rate)
            .expect("Division error");

        if sxlm_to_mint == 0 {
            unlock_reentrancy(&env);
            panic!("Deposit too small");
        }

        // Transfer XLM from user to vault
        let xlm_client = token::Client::new(&env, &xlm_token);
        xlm_client.transfer(&user, &env.current_contract_address(), &amount);

        // Update total deposits
        set_total_deposits(&env, total_after);

        // Track user deposit
        let user_total = get_user_deposit(&env, &user)
            .checked_add(amount)
            .expect("Overflow");
        set_user_deposit(&env, &user, user_total);

        // Mint sXLM to user
        env.invoke_contract::<()>(
            &sxlm_token,
            &Symbol::new(&env, "mint"),
            (user.clone(), sxlm_to_mint).into_val(&env),
        );

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "deposit"),),
            (user.clone(), amount, sxlm_to_mint),
        );

        unlock_reentrancy(&env);
        sxlm_to_mint
    }

    /// Deposit with slippage protection
    pub fn deposit_with_min_out(
        env: Env,
        user: Address,
        amount: i128,
        min_sxlm_out: i128,
    ) -> i128 {
        let sxlm_received = Self::deposit(env.clone(), user, amount);

        if sxlm_received < min_sxlm_out {
            panic!("Slippage: received {} but expected at least {}", sxlm_received, min_sxlm_out);
        }

        sxlm_received
    }

    /// Withdraw XLM by burning sXLM
    /// Returns the amount of XLM returned
    pub fn withdraw(env: Env, user: Address, sxlm_amount: i128) -> i128 {
        extend_instance_ttl(&env);
        require_not_reentering(&env);
        lock_reentrancy(&env);

        // Validations
        user.require_auth();

        if is_paused(&env) {
            unlock_reentrancy(&env);
            panic!("Vault is paused");
        }

        if sxlm_amount < MIN_AMOUNT {
            unlock_reentrancy(&env);
            panic!("Amount below minimum");
        }

        let xlm_token = get_xlm_token(&env);
        let sxlm_token = get_sxlm_token(&env);

        // Check user has enough sXLM
        let sxlm_client = token::Client::new(&env, &sxlm_token);
        let user_sxlm_balance = sxlm_client.balance(&user);
        if user_sxlm_balance < sxlm_amount {
            unlock_reentrancy(&env);
            panic!("Insufficient sXLM balance");
        }

        // Calculate XLM to return based on exchange rate
        let exchange_rate = Self::get_exchange_rate(env.clone());
        let xlm_to_return = sxlm_amount
            .checked_mul(exchange_rate)
            .expect("Overflow")
            .checked_div(PRECISION)
            .expect("Division error");

        if xlm_to_return == 0 {
            unlock_reentrancy(&env);
            panic!("Withdrawal too small");
        }

        // Check vault has enough XLM
        let xlm_client = token::Client::new(&env, &xlm_token);
        let vault_xlm_balance = xlm_client.balance(&env.current_contract_address());
        if vault_xlm_balance < xlm_to_return {
            unlock_reentrancy(&env);
            panic!("Insufficient vault liquidity");
        }

        // Burn sXLM from user
        env.invoke_contract::<()>(
            &sxlm_token,
            &Symbol::new(&env, "burn"),
            (user.clone(), sxlm_amount).into_val(&env),
        );

        // Update total deposits and yield tracking
        let total_deposits = get_total_deposits(&env);
        let yield_accrued = get_yield_accrued(&env);
        let total_assets = total_deposits + yield_accrued;

        // Proportionally reduce deposits and yield
        if total_assets > 0 {
            let deposit_portion = (xlm_to_return * total_deposits) / total_assets;
            let yield_portion = xlm_to_return - deposit_portion;

            set_total_deposits(&env, total_deposits.saturating_sub(deposit_portion));
            set_yield_accrued(&env, yield_accrued.saturating_sub(yield_portion));
        }

        // Update user deposit tracking
        let user_deposit = get_user_deposit(&env, &user);
        if user_deposit > 0 {
            let new_user_deposit = user_deposit.saturating_sub(xlm_to_return);
            set_user_deposit(&env, &user, new_user_deposit);
        }

        // Transfer XLM to user
        xlm_client.transfer(&env.current_contract_address(), &user, &xlm_to_return);

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "withdraw"),),
            (user.clone(), sxlm_amount, xlm_to_return),
        );

        unlock_reentrancy(&env);
        xlm_to_return
    }

    /// Withdraw with slippage protection
    pub fn withdraw_with_min_out(
        env: Env,
        user: Address,
        sxlm_amount: i128,
        min_xlm_out: i128,
    ) -> i128 {
        let xlm_received = Self::withdraw(env.clone(), user, sxlm_amount);

        if xlm_received < min_xlm_out {
            panic!("Slippage: received {} but expected at least {}", xlm_received, min_xlm_out);
        }

        xlm_received
    }

    /// Get current exchange rate (XLM per sXLM, scaled by PRECISION)
    pub fn get_exchange_rate(env: Env) -> i128 {
        extend_instance_ttl(&env);

        let sxlm_token = get_sxlm_token(&env);
        let total_sxlm: i128 = env.invoke_contract(
            &sxlm_token,
            &Symbol::new(&env, "total_supply"),
            ().into_val(&env),
        );

        if total_sxlm == 0 {
            return PRECISION; // 1:1 initially
        }

        let total_assets = Self::get_total_assets(env);
        total_assets
            .checked_mul(PRECISION)
            .expect("Overflow")
            .checked_div(total_sxlm)
            .expect("Division error")
    }

    /// Get total XLM managed by vault (deposits + yield)
    pub fn get_total_assets(env: Env) -> i128 {
        extend_instance_ttl(&env);
        let deposits = get_total_deposits(&env);
        let yield_accrued = get_yield_accrued(&env);
        deposits.checked_add(yield_accrued).expect("Overflow")
    }

    /// Get total deposits (without yield)
    pub fn get_total_deposits(env: Env) -> i128 {
        extend_instance_ttl(&env);
        get_total_deposits(&env)
    }

    /// Get yield accrued
    pub fn get_yield_accrued(env: Env) -> i128 {
        extend_instance_ttl(&env);
        get_yield_accrued(&env)
    }

    /// Get user's total deposit
    pub fn get_user_deposit(env: Env, user: Address) -> i128 {
        extend_instance_ttl(&env);
        get_user_deposit(&env, &user)
    }

    /// Add yield to vault (admin only - simulates strategy returns)
    pub fn add_yield(env: Env, amount: i128) {
        extend_instance_ttl(&env);

        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Transfer XLM from admin to vault
        let xlm_token = get_xlm_token(&env);
        let xlm_client = token::Client::new(&env, &xlm_token);
        xlm_client.transfer(&admin, &env.current_contract_address(), &amount);

        let current_yield = get_yield_accrued(&env);
        set_yield_accrued(&env, current_yield.checked_add(amount).expect("Overflow"));

        env.events().publish(
            (Symbol::new(&env, "yield_added"),),
            amount,
        );
    }

    /// Pause the vault (admin only)
    pub fn pause(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);

        env.events().publish(
            (Symbol::new(&env, "paused"),),
            true,
        );
    }

    /// Unpause the vault (admin only)
    pub fn unpause(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);

        env.events().publish(
            (Symbol::new(&env, "unpaused"),),
            true,
        );
    }

    /// Set maximum single deposit (admin only)
    pub fn set_max_deposit(env: Env, max_deposit: i128) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if max_deposit < MIN_AMOUNT {
            panic!("Max deposit too low");
        }

        env.storage().instance().set(&DataKey::MaxDeposit, &max_deposit);
    }

    /// Set total TVL cap (admin only)
    pub fn set_total_cap(env: Env, total_cap: i128) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        env.storage().instance().set(&DataKey::TotalCap, &total_cap);
    }

    /// Get admin address
    pub fn admin(env: Env) -> Address {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Get XLM token address
    pub fn xlm_token(env: Env) -> Address {
        extend_instance_ttl(&env);
        get_xlm_token(&env)
    }

    /// Get sXLM token address
    pub fn sxlm_token(env: Env) -> Address {
        extend_instance_ttl(&env);
        get_sxlm_token(&env)
    }

    /// Check if vault is paused
    pub fn is_paused(env: Env) -> bool {
        extend_instance_ttl(&env);
        is_paused(&env)
    }

    /// Get maximum single deposit
    pub fn max_deposit(env: Env) -> i128 {
        extend_instance_ttl(&env);
        get_max_deposit(&env)
    }

    /// Get total TVL cap
    pub fn total_cap(env: Env) -> i128 {
        extend_instance_ttl(&env);
        get_total_cap(&env)
    }

    /// Calculate how much sXLM user would receive for XLM deposit
    pub fn preview_deposit(env: Env, xlm_amount: i128) -> i128 {
        extend_instance_ttl(&env);
        let exchange_rate = Self::get_exchange_rate(env);
        xlm_amount
            .checked_mul(PRECISION)
            .expect("Overflow")
            .checked_div(exchange_rate)
            .expect("Division error")
    }

    /// Calculate how much XLM user would receive for sXLM withdrawal
    pub fn preview_withdraw(env: Env, sxlm_amount: i128) -> i128 {
        extend_instance_ttl(&env);
        let exchange_rate = Self::get_exchange_rate(env);
        sxlm_amount
            .checked_mul(exchange_rate)
            .expect("Overflow")
            .checked_div(PRECISION)
            .expect("Division error")
    }

    /// Upgrade contract (admin only) - for future upgrades
    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Transfer admin (admin only)
    pub fn transfer_admin(env: Env, new_admin: Address) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events().publish(
            (Symbol::new(&env, "admin_transferred"),),
            (admin, new_admin),
        );
    }
}

mod test;
