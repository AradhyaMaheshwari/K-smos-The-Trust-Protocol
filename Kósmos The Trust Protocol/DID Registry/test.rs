#![no_std]
use soroban_sdk::{
    testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke},
    Address, Env, Map, symbol_short, String, IntoVal,
};
use crate::{DidRegistryContract, DidRegistryContractClient, DidStatus};

#[test]
fn test_did_registration_and_resolution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DidRegistryContract);
    let client = DidRegistryContractClient::new(&env, &contract_id);

    let controller = Address::generate(&env);
    let did_string = String::from_slice(&env, "did:kosmos:123456789");
    
    let mut document = Map::new(&env);
    document.set(symbol_short!("pubkey"), String::from_slice(&env, "G..."));
    document.set(symbol_short!("service"), String::from_slice(&env, "https://example.com/endpoint"));

    // Register the DID with mock authentication
    client
        .mock_auths(&[MockAuth {
            address: &controller,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register_did",
                args: (
                    controller.clone(),
                    did_string.clone(),
                    document.clone(),
                ).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register_did(&controller, &did_string, &document);

    // Resolve the DID
    let (status, resolved_document) = client.get_did(&did_string);

    assert_eq!(status, DidStatus::Active);
    assert_eq!(resolved_document, document);
    assert_eq!(client.get_controller(&did_string), controller);

    // Check events
     let mut events = env.events().all();
     let event = events.pop_back_unchecked();
     assert_eq!(
         event.topics,
         (symbol_short!("did_reg"), did_string.clone()).into_val(&env)
     );
     assert_eq!(event.data, controller.into_val(&env));
}

#[test]
fn test_did_update() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DidRegistryContract);
    let client = DidRegistryContractClient::new(&env, &contract_id);

    let controller = Address::generate(&env);
    let did_string = String::from_slice(&env, "did:kosmos:user-a");
    
    let mut doc_v1 = Map::new(&env);
    doc_v1.set(symbol_short!("key"), String::from_slice(&env, "value1"));

    // Register
    client
        .mock_auths(&[MockAuth {
            address: &controller,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register_did",
                args: (controller.clone(), did_string.clone(), doc_v1).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register_did(&controller, &did_string, &doc_v1);

    // Update
    let mut doc_v2 = Map::new(&env);
    doc_v2.set(symbol_short!("key"), String::from_slice(&env, "value2"));

    client
        .mock_auths(&[MockAuth {
            address: &controller,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "update_document",
                args: (did_string.clone(), doc_v2.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .update_document(&did_string, &doc_v2);

    // Resolve and check
    let (status, resolved_document) = client.get_did(&did_string);
    assert_eq!(status, DidStatus::Active);
    assert_eq!(resolved_document, doc_v2);
}

#[test]
fn test_did_revocation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DidRegistryContract);
    let client = DidRegistryContractClient::new(&env, &contract_id);

    let controller = Address::generate(&env);
    let did_string = String::from_slice(&env, "did:kosmos:user-to-revoke");
    
    let mut document = Map::new(&env);
    document.set(symbol_short!("key"), String::from_slice(&env, "any_value"));

    // Register
    client
        .mock_auths(&[MockAuth {
            address: &controller,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register_did",
                args: (controller.clone(), did_string.clone(), document).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register_did(&controller, &did_string, &document);

    // Revoke
    client
        .mock_auths(&[MockAuth {
            address: &controller,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "revoke_did",
                args: (did_string.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .revoke_did(&did_string);

    // Resolve and check
    let (status, _) = client.get_did(&did_string);
    assert_eq!(status, DidStatus::Revoked);
}

#[test]
#[should_panic(expected = "DID not found")]
fn test_get_nonexistent_did() {
    let env = Env::default();
    let client = DidRegistryContractClient::new(&env, &env.register_contract(None, DidRegistryContract));
    let did_string = String::from_slice(&env, "did:kosmos:nonexistent");
    client.get_did(&did_string);
}
