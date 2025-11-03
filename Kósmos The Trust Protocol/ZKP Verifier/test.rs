#![no_std]
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, IntoVal, Vec, symbol_short,
};

use crate::{ZkpVerifierContract, ZkpVerifierContractClient};
use crate::IssuerWhitelistClient;

// Import the IssuerWhitelist contract to mock it
use issuer_whitelist::{IssuerWhitelistContract, IssuerWhitelistContractClient as WhitelistClient};


fn setup_test() -> (Env, Address, ZkpVerifierContractClient, WhitelistClient) {
    let env = Env::default();
    env.mock_all_auths(); // Simplify auth for cross-contract setup

    // 1. Deploy IssuerWhitelist contract
    let whitelist_contract_id = env.register_contract_wasm(None, issuer_whitelist::WASM);
    let whitelist_client = WhitelistClient::new(&env, &whitelist_contract_id);
    let admin = Address::generate(&env);
    whitelist_client.initialize(&admin);

    // 2. Deploy ZkpVerifier contract
    let verifier_contract_id = env.register_contract(None, ZkpVerifierContract);
    let verifier_client = ZkpVerifierContractClient::new(&env, &verifier_contract_id);
    
    // 3. Initialize ZkpVerifier with the address of the IssuerWhitelist
    verifier_client.initialize(&whitelist_contract_id);

    (env, admin, verifier_client, whitelist_client)
}

#[test]
fn test_verification_success() {
    let (env, admin, verifier_client, whitelist_client) = setup_test();

    let trusted_issuer = Address::generate(&env);
    
    // 1. Add issuer to whitelist
    whitelist_client.add_issuer(&trusted_issuer);
    assert!(whitelist_client.is_whitelisted(&trusted_issuer));

    // 2. Prepare dummy proof data
    let dummy_proof: BytesN<256> = BytesN::random(&env);
    let mut public_inputs = Vec::new(&env);
    public_inputs.push_back(700u32); // e.g., "score > 700"
    public_inputs.push_back(String::from_slice(&env, "vc_hash_123"));

    // 3. Run verification
    let is_valid = verifier_client.verify_proof(
        &trusted_issuer,
        &dummy_proof,
        &public_inputs
    );

    // 4. Check result
    // Our placeholder logic returns true if proof and inputs are not empty
    assert!(is_valid);

    // 5. Check event
    let mut events = env.events().all();
    let event = events.pop_back_unchecked();
     assert_eq!(
         event.topics,
         (symbol_short!("zkp_verify"), trusted_issuer.clone()).into_val(&env)
     );
     assert_eq!(event.data, public_inputs.into_val(&env));
}

#[test]
#[should_panic(expected = "Issuer is not trusted")]
fn test_verification_untrusted_issuer() {
    let (env, _, verifier_client, _) = setup_test();

    let untrusted_issuer = Address::generate(&env);
    let dummy_proof: BytesN<256> = BytesN::random(&env);
    let public_inputs = Vec::new(&env);

    // Run verification - this should panic
    verifier_client.verify_proof(
        &untrusted_issuer,
        &dummy_proof,
        &public_inputs
    );
}
