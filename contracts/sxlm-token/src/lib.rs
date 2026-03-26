#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String,
};
use soroban_token_sdk::TokenUtils;

// Storage TTL constants
const DAY_IN_LEDGERS: u32 = 17280; // ~24 hours
const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS; // 7 days
const INSTANCE_LIFETIME_THRESHOLD: u32 = DAY_IN_LEDGERS; // 1 day
const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS; // 30 days
const BALANCE_LIFETIME_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS; // 7 days

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Minter,
    Balance(Address),
    Allowance(Address, Address),
    TotalSupply,
    Name,
    Symbol,
    Decimals,
}

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn extend_balance_ttl(env: &Env, addr: &Address) {
    let key = DataKey::Balance(addr.clone());
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    }
}

fn get_balance(env: &Env, addr: &Address) -> i128 {
    let key = DataKey::Balance(addr.clone());
    if let Some(balance) = env.storage().persistent().get(&key) {
        extend_balance_ttl(env, addr);
        balance
    } else {
        0
    }
}

fn set_balance(env: &Env, addr: &Address, amount: i128) {
    let key = DataKey::Balance(addr.clone());
    env.storage().persistent().set(&key, &amount);
    extend_balance_ttl(env, addr);
}

fn get_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn set_total_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::TotalSupply, &amount);
}

fn get_allowance(env: &Env, from: &Address, spender: &Address) -> i128 {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}

fn set_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
    let key = DataKey::Allowance(from.clone(), spender.clone());
    if amount > 0 {
        env.storage().persistent().set(&key, &amount);
        env.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    } else {
        env.storage().persistent().remove(&key);
    }
}

#[contract]
pub struct SxlmToken;

#[contractimpl]
impl SxlmToken {
    /// Initialize the sXLM token
    pub fn initialize(env: Env, admin: Address, minter: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Minter, &minter);
        env.storage().instance().set(&DataKey::Name, &String::from_str(&env, "Staked XLM"));
        env.storage().instance().set(&DataKey::Symbol, &String::from_str(&env, "sXLM"));
        env.storage().instance().set(&DataKey::Decimals, &7u32);
        set_total_supply(&env, 0);
        extend_instance_ttl(&env);
    }

    /// Mint sXLM tokens (only callable by vault/minter)
    pub fn mint(env: Env, to: Address, amount: i128) {
        extend_instance_ttl(&env);

        let minter: Address = env.storage().instance().get(&DataKey::Minter).unwrap();
        minter.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let balance = get_balance(&env, &to);
        set_balance(&env, &to, balance.checked_add(amount).expect("Overflow"));

        let total = get_total_supply(&env);
        set_total_supply(&env, total.checked_add(amount).expect("Overflow"));

        TokenUtils::new(&env).events().mint(minter, to, amount);
    }

    /// Burn sXLM tokens (only callable by vault/minter)
    pub fn burn(env: Env, from: Address, amount: i128) {
        extend_instance_ttl(&env);

        let minter: Address = env.storage().instance().get(&DataKey::Minter).unwrap();
        minter.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let balance = get_balance(&env, &from);
        if balance < amount {
            panic!("Insufficient balance");
        }

        set_balance(&env, &from, balance - amount);

        let total = get_total_supply(&env);
        set_total_supply(&env, total - amount);

        TokenUtils::new(&env).events().burn(from, amount);
    }

    /// User burn - user can burn their own tokens
    pub fn burn_self(env: Env, from: Address, amount: i128) {
        extend_instance_ttl(&env);
        from.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let balance = get_balance(&env, &from);
        if balance < amount {
            panic!("Insufficient balance");
        }

        set_balance(&env, &from, balance - amount);
        let total = get_total_supply(&env);
        set_total_supply(&env, total - amount);

        TokenUtils::new(&env).events().burn(from, amount);
    }

    /// Get the minter (vault) address
    pub fn minter(env: Env) -> Address {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::Minter).unwrap()
    }

    /// Update minter (admin only)
    pub fn set_minter(env: Env, new_minter: Address) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Minter, &new_minter);
    }

    /// Get admin address
    pub fn admin(env: Env) -> Address {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Get total supply of sXLM
    pub fn total_supply(env: Env) -> i128 {
        extend_instance_ttl(&env);
        get_total_supply(&env)
    }

    // ============ Standard Token Interface ============

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        extend_instance_ttl(&env);
        get_allowance(&env, &from, &spender)
    }

    pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        extend_instance_ttl(&env);
        from.require_auth();

        if amount < 0 {
            panic!("Amount cannot be negative");
        }

        set_allowance(&env, &from, &spender, amount);
        TokenUtils::new(&env).events().approve(from, spender, amount, expiration_ledger);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        extend_instance_ttl(&env);
        get_balance(&env, &id)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        extend_instance_ttl(&env);
        from.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        set_balance(&env, &from, from_balance - amount);
        set_balance(&env, &to, get_balance(&env, &to).checked_add(amount).expect("Overflow"));

        TokenUtils::new(&env).events().transfer(from, to, amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        extend_instance_ttl(&env);
        spender.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let allowance = get_allowance(&env, &from, &spender);
        if allowance < amount {
            panic!("Insufficient allowance");
        }

        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            panic!("Insufficient balance");
        }

        set_allowance(&env, &from, &spender, allowance - amount);
        set_balance(&env, &from, from_balance - amount);
        set_balance(&env, &to, get_balance(&env, &to).checked_add(amount).expect("Overflow"));

        TokenUtils::new(&env).events().transfer(from, to, amount);
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        extend_instance_ttl(&env);
        spender.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let allowance = get_allowance(&env, &from, &spender);
        if allowance < amount {
            panic!("Insufficient allowance");
        }

        let balance = get_balance(&env, &from);
        if balance < amount {
            panic!("Insufficient balance");
        }

        set_allowance(&env, &from, &spender, allowance - amount);
        set_balance(&env, &from, balance - amount);
        let total = get_total_supply(&env);
        set_total_supply(&env, total - amount);

        TokenUtils::new(&env).events().burn(from, amount);
    }

    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn name(env: Env) -> String {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::Name).unwrap()
    }

    pub fn symbol(env: Env) -> String {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::Symbol).unwrap()
    }
}

mod test;
