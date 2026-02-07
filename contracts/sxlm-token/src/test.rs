#![cfg(test)]

use crate::{SxlmToken, SxlmTokenClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_test() -> (Env, SxlmTokenClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SxlmToken, ());
    let client = SxlmTokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &minter);

    (env, client, admin, minter, user)
}

#[test]
fn test_initialize() {
    let (env, client, admin, minter, _user) = setup_test();

    assert_eq!(client.symbol(), String::from_str(&env, "sXLM"));
    assert_eq!(client.name(), String::from_str(&env, "Staked XLM"));
    assert_eq!(client.decimals(), 7);
    assert_eq!(client.minter(), minter);
    assert_eq!(client.admin(), admin);
    assert_eq!(client.total_supply(), 0);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_cannot_reinitialize() {
    let (env, client, _admin, _minter, _user) = setup_test();
    let new_admin = Address::generate(&env);
    let new_minter = Address::generate(&env);
    client.initialize(&new_admin, &new_minter);
}

#[test]
fn test_mint() {
    let (_env, client, _admin, _minter, user) = setup_test();

    client.mint(&user, &1000_0000000);

    assert_eq!(client.balance(&user), 1000_0000000);
    assert_eq!(client.total_supply(), 1000_0000000);
}

#[test]
fn test_mint_multiple_users() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let user2 = Address::generate(&env);

    client.mint(&user1, &1000_0000000);
    client.mint(&user2, &500_0000000);

    assert_eq!(client.balance(&user1), 1000_0000000);
    assert_eq!(client.balance(&user2), 500_0000000);
    assert_eq!(client.total_supply(), 1500_0000000);
}

#[test]
fn test_burn() {
    let (_env, client, _admin, _minter, user) = setup_test();

    client.mint(&user, &1000_0000000);
    client.burn(&user, &400_0000000);

    assert_eq!(client.balance(&user), 600_0000000);
    assert_eq!(client.total_supply(), 600_0000000);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_burn_insufficient_balance() {
    let (_env, client, _admin, _minter, user) = setup_test();

    client.mint(&user, &100_0000000);
    client.burn(&user, &200_0000000);
}

#[test]
fn test_transfer() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let user2 = Address::generate(&env);

    client.mint(&user1, &1000_0000000);
    client.transfer(&user1, &user2, &300_0000000);

    assert_eq!(client.balance(&user1), 700_0000000);
    assert_eq!(client.balance(&user2), 300_0000000);
    assert_eq!(client.total_supply(), 1000_0000000);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_transfer_insufficient_balance() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let user2 = Address::generate(&env);

    client.mint(&user1, &100_0000000);
    client.transfer(&user1, &user2, &200_0000000);
}

#[test]
fn test_approve_and_transfer_from() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let user2 = Address::generate(&env);
    let spender = Address::generate(&env);

    client.mint(&user1, &1000_0000000);
    client.approve(&user1, &spender, &500_0000000, &1000);

    assert_eq!(client.allowance(&user1, &spender), 500_0000000);

    client.transfer_from(&spender, &user1, &user2, &300_0000000);

    assert_eq!(client.balance(&user1), 700_0000000);
    assert_eq!(client.balance(&user2), 300_0000000);
    assert_eq!(client.allowance(&user1, &spender), 200_0000000);
}

#[test]
#[should_panic(expected = "Insufficient allowance")]
fn test_transfer_from_insufficient_allowance() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let user2 = Address::generate(&env);
    let spender = Address::generate(&env);

    client.mint(&user1, &1000_0000000);
    client.approve(&user1, &spender, &100_0000000, &1000);

    client.transfer_from(&spender, &user1, &user2, &200_0000000);
}

#[test]
fn test_burn_self() {
    let (_env, client, _admin, _minter, user) = setup_test();

    client.mint(&user, &1000_0000000);
    client.burn_self(&user, &400_0000000);

    assert_eq!(client.balance(&user), 600_0000000);
    assert_eq!(client.total_supply(), 600_0000000);
}

#[test]
fn test_set_minter() {
    let (env, client, _admin, _minter, _user) = setup_test();
    let new_minter = Address::generate(&env);

    client.set_minter(&new_minter);

    assert_eq!(client.minter(), new_minter);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_mint_zero_amount() {
    let (_env, client, _admin, _minter, user) = setup_test();
    client.mint(&user, &0);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_transfer_zero_amount() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let user2 = Address::generate(&env);

    client.mint(&user1, &1000_0000000);
    client.transfer(&user1, &user2, &0);
}

#[test]
fn test_approve_zero_removes_allowance() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let spender = Address::generate(&env);

    client.mint(&user1, &1000_0000000);
    client.approve(&user1, &spender, &500_0000000, &1000);
    assert_eq!(client.allowance(&user1, &spender), 500_0000000);

    client.approve(&user1, &spender, &0, &1000);
    assert_eq!(client.allowance(&user1, &spender), 0);
}

#[test]
fn test_burn_from() {
    let (env, client, _admin, _minter, user1) = setup_test();
    let spender = Address::generate(&env);

    client.mint(&user1, &1000_0000000);
    client.approve(&user1, &spender, &500_0000000, &1000);

    client.burn_from(&spender, &user1, &300_0000000);

    assert_eq!(client.balance(&user1), 700_0000000);
    assert_eq!(client.total_supply(), 700_0000000);
    assert_eq!(client.allowance(&user1, &spender), 200_0000000);
}
