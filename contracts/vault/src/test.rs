#![cfg(test)]

// Integration tests require deployed contracts
// Run these tests via CLI on testnet:
//
// 1. Check exchange rate:
//    stellar contract invoke --id CCMUHDZ7OOKJW4RYKCLICF37VOYMB3USVUTJ5BZIVL3H5P34T2MHIU2N \
//      --source deployer --network testnet -- get_exchange_rate
//
// 2. Deposit test:
//    stellar contract invoke --id CCMUHDZ7OOKJW4RYKCLICF37VOYMB3USVUTJ5BZIVL3H5P34T2MHIU2N \
//      --source deployer --network testnet -- deposit \
//      --user GAUBEVSJBCR3LYNIK2JUQYWNEFLE43E73MBD3ZEXL6Q43EPHSERZXVNY \
//      --amount 1000000000
//
// 3. Withdraw test:
//    stellar contract invoke --id CCMUHDZ7OOKJW4RYKCLICF37VOYMB3USVUTJ5BZIVL3H5P34T2MHIU2N \
//      --source deployer --network testnet -- withdraw \
//      --user GAUBEVSJBCR3LYNIK2JUQYWNEFLE43E73MBD3ZEXL6Q43EPHSERZXVNY \
//      --sxlm_amount 500000000

use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_data_key_variants() {
    // Test that DataKey enum variants are correctly defined
    let env = Env::default();
    let addr = Address::generate(&env);

    let _ = DataKey::Admin;
    let _ = DataKey::XlmToken;
    let _ = DataKey::SxlmToken;
    let _ = DataKey::TotalDeposits;
    let _ = DataKey::YieldAccrued;
    let _ = DataKey::Initialized;
    let _ = DataKey::Paused;
}

#[test]
fn test_precision_constant() {
    // Verify precision is 7 decimals (1.0 = 10_000_000)
    assert_eq!(PRECISION, 1_0000000);
}
