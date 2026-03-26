#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec, Symbol, symbol_short};

#[contracttype]
pub enum DataKey {
    Admin,
    Vault,
    Router,
    Factory,
    XlmToken,
    UsdcToken,
    LpBalance,
    TotalDeposited,
}

// Soroswap Router interface
mod router {
    use soroban_sdk::{Address, Env, Vec};

    soroban_sdk::contractimport!(
        file = "../soroswap_router.wasm"
    );
}

#[contract]
pub struct YieldStrategy;

#[contractimpl]
impl YieldStrategy {
    /// Initialize the yield strategy
    pub fn initialize(
        env: Env,
        admin: Address,
        vault: Address,
        router: Address,
        factory: Address,
        xlm_token: Address,
        usdc_token: Address,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Vault, &vault);
        env.storage().instance().set(&DataKey::Router, &router);
        env.storage().instance().set(&DataKey::Factory, &factory);
        env.storage().instance().set(&DataKey::XlmToken, &xlm_token);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::LpBalance, &0i128);
        env.storage().instance().set(&DataKey::TotalDeposited, &0i128);
    }

    /// Deposit XLM into Soroswap liquidity pool
    /// 1. Receive XLM from vault
    /// 2. Swap half to USDC
    /// 3. Add liquidity to XLM/USDC pool
    /// 4. Hold LP tokens
    pub fn deposit(env: Env, from: Address, amount: i128) -> i128 {
        from.require_auth();

        let vault: Address = env.storage().instance().get(&DataKey::Vault).unwrap();
        if from != vault {
            panic!("Only vault can deposit");
        }

        let xlm_token: Address = env.storage().instance().get(&DataKey::XlmToken).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let router: Address = env.storage().instance().get(&DataKey::Router).unwrap();

        // Transfer XLM from vault to this contract
        let xlm_client = token::Client::new(&env, &xlm_token);
        xlm_client.transfer(&from, &env.current_contract_address(), &amount);

        // Swap half XLM to USDC
        let half_amount = amount / 2;
        let path: Vec<Address> = Vec::from_array(&env, [xlm_token.clone(), usdc_token.clone()]);

        // Approve router to spend XLM
        xlm_client.approve(&env.current_contract_address(), &router, &half_amount, &(env.ledger().sequence() + 1000));

        // Call router swap
        let swap_result: Vec<i128> = env.invoke_contract(
            &router,
            &symbol_short!("swap_ext"),  // swap_exact_tokens_for_tokens
            (
                half_amount,
                0i128,  // min out (accept any for demo)
                path,
                env.current_contract_address(),
                get_deadline(&env),
            ).into_val(&env)
        );

        let usdc_received = swap_result.get(1).unwrap();
        let xlm_for_lp = amount - half_amount;

        // Approve router for liquidity add
        let usdc_client = token::Client::new(&env, &usdc_token);
        xlm_client.approve(&env.current_contract_address(), &router, &xlm_for_lp, &(env.ledger().sequence() + 1000));
        usdc_client.approve(&env.current_contract_address(), &router, &usdc_received, &(env.ledger().sequence() + 1000));

        // Add liquidity
        let (amount_a, amount_b, liquidity): (i128, i128, i128) = env.invoke_contract(
            &router,
            &symbol_short!("add_liq"),  // add_liquidity
            (
                xlm_token,
                usdc_token,
                xlm_for_lp,
                usdc_received,
                0i128,  // min a
                0i128,  // min b
                env.current_contract_address(),
                get_deadline(&env),
            ).into_val(&env)
        );

        // Update LP balance
        let current_lp: i128 = env.storage().instance().get(&DataKey::LpBalance).unwrap_or(0);
        env.storage().instance().set(&DataKey::LpBalance, &(current_lp + liquidity));

        let total_deposited: i128 = env.storage().instance().get(&DataKey::TotalDeposited).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalDeposited, &(total_deposited + amount));

        liquidity
    }

    /// Withdraw from Soroswap and return XLM to vault
    pub fn withdraw(env: Env, to: Address, lp_amount: i128) -> i128 {
        let vault: Address = env.storage().instance().get(&DataKey::Vault).unwrap();
        if to != vault {
            panic!("Only vault can withdraw");
        }

        let xlm_token: Address = env.storage().instance().get(&DataKey::XlmToken).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let router: Address = env.storage().instance().get(&DataKey::Router).unwrap();

        // Remove liquidity
        let (xlm_received, usdc_received): (i128, i128) = env.invoke_contract(
            &router,
            &symbol_short!("rem_liq"),  // remove_liquidity
            (
                xlm_token.clone(),
                usdc_token.clone(),
                lp_amount,
                0i128,
                0i128,
                env.current_contract_address(),
                get_deadline(&env),
            ).into_val(&env)
        );

        // Swap USDC back to XLM
        let path: Vec<Address> = Vec::from_array(&env, [usdc_token, xlm_token.clone()]);
        let usdc_client = token::Client::new(&env, &env.storage().instance().get::<DataKey, Address>(&DataKey::UsdcToken).unwrap());
        usdc_client.approve(&env.current_contract_address(), &router, &usdc_received, &(env.ledger().sequence() + 1000));

        let swap_result: Vec<i128> = env.invoke_contract(
            &router,
            &symbol_short!("swap_ext"),
            (
                usdc_received,
                0i128,
                path,
                env.current_contract_address(),
                get_deadline(&env),
            ).into_val(&env)
        );

        let total_xlm = xlm_received + swap_result.get(1).unwrap();

        // Transfer XLM to vault
        let xlm_client = token::Client::new(&env, &xlm_token);
        xlm_client.transfer(&env.current_contract_address(), &to, &total_xlm);

        // Update LP balance
        let current_lp: i128 = env.storage().instance().get(&DataKey::LpBalance).unwrap_or(0);
        env.storage().instance().set(&DataKey::LpBalance, &(current_lp - lp_amount));

        total_xlm
    }

    /// Get current value of LP position in XLM terms
    pub fn get_value(env: Env) -> i128 {
        let lp_balance: i128 = env.storage().instance().get(&DataKey::LpBalance).unwrap_or(0);
        if lp_balance == 0 {
            return 0;
        }

        // In production, query the pool for actual value
        // For now, estimate with 0.3% fee accumulation simulation
        let total_deposited: i128 = env.storage().instance().get(&DataKey::TotalDeposited).unwrap_or(0);

        // Simulate ~0.05% yield per call (trading fees)
        total_deposited * 10005 / 10000
    }

    /// Harvest trading fees and return as yield
    pub fn harvest(env: Env) -> i128 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let current_value = Self::get_value(env.clone());
        let total_deposited: i128 = env.storage().instance().get(&DataKey::TotalDeposited).unwrap_or(0);

        if current_value > total_deposited {
            current_value - total_deposited
        } else {
            0
        }
    }

    pub fn get_lp_balance(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::LpBalance).unwrap_or(0)
    }
}

fn get_deadline(env: &Env) -> u64 {
    env.ledger().timestamp() + 3600 // 1 hour
}
