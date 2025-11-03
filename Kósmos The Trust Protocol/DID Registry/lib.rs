#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, String, Symbol, Vec, Address, Map};

// --- Data Structures for DID Document ---
// As per the blueprint, the DID document maps the DID to public keys, verification methods, and service endpoints.

#[contract]
pub struct DidRegistryContract;

/// Represents the status of a DID.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DidStatus {
    Active = 1,
    Revoked = 2,
}

impl soroban_sdk::TryFromVal<Env, soroban_sdk::Val> for DidStatus {
    type Error = soroban_sdk::ConversionError;
    fn try_from_val(env: &Env, val: &soroban_sdk::Val) -> Result<Self, Self::Error> {
        let u32_val = u32::try_from_val(env, val)?;
        match u32_val {
            1 => Ok(DidStatus::Active),
            2 => Ok(DidStatus::Revoked),
            _ => Err(soroban_sdk::ConversionError),
        }
    }
}

impl soroban_sdk::IntoVal<Env, soroban_sdk::Val> for DidStatus {
    fn into_val(self, env: &Env) -> soroban_sdk::Val {
        (self as u32).into_val(env)
    }
}


/// Storage keys for DID data.
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    // Stores the controller (Address) of a DID (String)
    Controller(String) = 1,
    // Stores the DID Document (Map<Symbol, Val>) for a DID (String)
    Document(String) = 2, 
    // Stores the Status (DidStatus) of a DID (String)
    Status(String) = 3,
}

impl soroban_sdk::IntoVal<Env, soroban_sdk::Val> for DataKey {
    fn into_val(self, env: &Env) -> soroban_sdk::Val {
        match self {
            DataKey::Controller(did) => {
                let mut vec = Vec::new(env);
                vec.push_back(1u32);
                vec.push_back(did);
                vec.into_val(env)
            }
            DataKey::Document(did) => {
                let mut vec = Vec::new(env);
                vec.push_back(2u32);
                vec.push_back(did);
                vec.into_val(env)
            }
            DataKey::Status(did) => {
                let mut vec = Vec::new(env);
                vec.push_back(3u32);
                vec.push_back(did);
                vec.into_val(env)
            }
        }
    }
}


/// Trait defining the DID Registry functions.
#[contractimpl]
impl DidRegistryContract {

    /// Registers a new 'did:kosmos' identifier.
    /// The controller is the address that has authority over this DID.
    /// The document contains public keys, service endpoints, etc.
    pub fn register_did(env: Env, controller: Address, did: String, document: Map<Symbol, soroban_sdk::Val>) {
        controller.require_auth();

        let controller_key = DataKey::Controller(did.clone());
        if env.storage().instance().has(&controller_key) {
            panic!("DID already registered");
        }

        env.storage().instance().set(&controller_key, &controller);
        env.storage().instance().set(&DataKey::Document(did.clone()), &document);
        env.storage().instance().set(&DataKey::Status(did.clone()), &DidStatus::Active);

        // Emit event
        env.events().publish(
            (symbol_short!("did_reg"), did),
            controller
        );
    }

    /// Updates the DID document for an existing DID.
    /// Only the current controller of the DID can perform this action.
    pub fn update_document(env: Env, did: String, new_document: Map<Symbol, soroban_sdk::Val>) {
        let controller_key = DataKey::Controller(did.clone());
        if !env.storage().instance().has(&controller_key) {
            panic!("DID not found");
        }

        let controller: Address = env.storage().instance().get(&controller_key).unwrap();
        controller.require_auth();

        let status: DidStatus = env.storage().instance().get(&DataKey::Status(did.clone())).unwrap();
        if status != DidStatus::Active {
            panic!("Cannot update a revoked DID");
        }

        env.storage().instance().set(&DataKey::Document(did.clone()), &new_document);

        // Emit event
        env.events().publish(
            (symbol_short!("did_upd"), did),
            symbol_short!("doc_upd")
        );
    }

    /// Revokes a DID. This is a permanent action.
    /// Only the current controller of the DID can perform this action.
    pub fn revoke_did(env: Env, did: String) {
        let controller_key = DataKey::Controller(did.clone());
        if !env.storage().instance().has(&controller_key) {
            panic!("DID not found");
        }

        let controller: Address = env.storage().instance().get(&controller_key).unwrap();
        controller.require_auth();

        env.storage().instance().set(&DataKey::Status(did.clone()), &DidStatus::Revoked);

        // Emit event
        env.events().publish(
            (symbol_short!("did_rev"), did),
            controller
        );
    }

    /// Resolves a DID string to its document and status.
    /// This is a read-only function.
    pub fn get_did(env: Env, did: String) -> (DidStatus, Map<Symbol, soroban_sdk::Val>) {
        let controller_key = DataKey::Controller(did.clone());
        if !env.storage().instance().has(&controller_key) {
            panic!("DID not found");
        }

        let status: DidStatus = env.storage().instance().get(&DataKey::Status(did.clone())).unwrap();
        let document: Map<Symbol, soroban_sdk::Val> = env.storage().instance().get(&DataKey::Document(did.clone())).unwrap();

        (status, document)
    }

     /// Resolves a DID string to its controller address.
    pub fn get_controller(env: Env, did: String) -> Address {
        let controller_key = DataKey::Controller(did.clone());
        if !env.storage().instance().has(&controller_key) {
            panic!("DID not found");
        }
        env.storage().instance().get(&controller_key).unwrap()
    }
}
