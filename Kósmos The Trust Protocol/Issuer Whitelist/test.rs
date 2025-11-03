#![no_std]
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, Vec,
};

use crate::{IssuerWhitelistContract, IssuerWhitelistContractClient};

fn setup_test() -> (Env, Address, IssuerWhitelistContractClient) {
    let env = Env::default();
    let contract_id = env.register_contract(None, IssuerWhitelistContract);
    let client = IssuerWhitelistContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    client.initialize(&admin);
    
    (env, admin, client)
}

#[test]
fn test_initialize() {
    let (env, admin, client) = setup_test();
    
    // Check if admin is set
    let issuers = client.get_issuers();
    assert_eq!(issuers.len(), 0);

    // Cannot re-initialize
    let will_panic = || client.initialize(&admin);
    assert!(soroban_sdk::panic_with_error(&env, &(), will_panic).is_ok());
}

#[test]
fn test_add_and_remove_issuer() {
    let (env, admin, client) = setup_test();
    let issuer_to_add = Address::generate(&env);

    // Add issuer
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.contract_id,
                fn_name: "add_issuer",
                args: (issuer_to_add.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .add_issuer(&issuer_to_add);

    // Check if whitelisted
    assert_eq!(client.is_whitelisted(&issuer_to_add), true);
    let issuers = client.get_issuers();
    assert_eq!(issuers.len(), 1);
    assert_eq!(issuers.get(0), Ok(issuer_to_add.clone()));

    // Remove issuer
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.contract_id,
                fn_name: "remove_issuer",
                args: (issuer_to_add.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .remove_issuer(&issuer_to_add);

    // Check if removed
    assert_eq!(client.is_whitelisted(&issuer_to_add), false);
    assert_eq!(client.get_issuers().len(), 0);
}

#[test]
#[should_panic(expected = "Issuer not found in whitelist")]
fn test_remove_nonexistent_issuer() {
    let (env, admin, client) = setup_test();
    let non_issuer = Address::generate(&env);

    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.contract_id,
                fn_name: "remove_issuer",
                args: (non_issuer.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .remove_issuer(&non_issuer);
}

#[test]
fn test_auth() {
    let (env, _, client) = setup_test();
    let unauthorized_user = Address::generate(&env);
    let issuer_to_add = Address::generate(&env);

    // Try to add issuer without auth
    let will_panic = || client.mock_auths(&[]).add_issuer(&issuer_to_add);
    assert!(soroban_sdk::panic_with_error(&env, &soroban_sdk::Error::from_contract_error(3), will_panic).is_ok());
}
