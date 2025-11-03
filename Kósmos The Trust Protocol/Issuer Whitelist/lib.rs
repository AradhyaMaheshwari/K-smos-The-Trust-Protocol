#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Symbol, Vec};

#[contract]
pub struct IssuerWhitelistContract;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Admin = 1,
    // Stores the list (Vec<Address>) of whitelisted issuers
    IssuerList = 2,
}

impl soroban_sdk::IntoVal<Env, soroban_sdk::Val> for DataKey {
    fn into_val(self, env: &Env) -> soroban_sdk::Val {
        (self as u32).into_val(env)
    }
}

#[contractimpl]
impl IssuerWhitelistContract {
    /// Initializes the contract with an administrator.
    /// The admin is the only one who can add or remove issuers.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        // Initialize with an empty list of issuers
        env.storage().instance().set(&DataKey::IssuerList, &Vec::<Address>::new(&env));
    }

    /// Adds a new trusted issuer to the whitelist.
    /// Requires authorization from the contract admin.
    pub fn add_issuer(env: Env, issuer_address: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut issuers: Vec<Address> = env.storage().instance().get(&DataKey::IssuerList).unwrap();
        
        if let Some(_) = issuers.iter().find(|&x| x == issuer_address) {
             panic!("Issuer already whitelisted");
        }

        issuers.push_back(issuer_address.clone());
        env.storage().instance().set(&DataKey::IssuerList, &issuers);

        // Emit event
        env.events().publish(
            (symbol_short!("iss_add"),),
            issuer_address
        );
    }

    /// Removes an issuer from the whitelist.
    /// Requires authorization from the contract admin.
    pub fn remove_issuer(env: Env, issuer_address: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut issuers: Vec<Address> = env.storage().instance().get(&DataKey::IssuerList).unwrap();

        let index = issuers.iter().position(|x| x == issuer_address);
        
        if let Some(i) = index {
            issuers.remove(i as u32);
            env.storage().instance().set(&DataKey::IssuerList, &issuers);

            // Emit event
            env.events().publish(
                (symbol_short!("iss_rem"),),
                issuer_address
            );
        } else {
            panic!("Issuer not found in whitelist");
        }
    }

    /// Checks if a given address is a whitelisted issuer.
    /// This is a read-only function.
    pub fn is_whitelisted(env: Env, issuer_address: Address) -> bool {
        let issuers: Vec<Address> = env.storage().instance().get(&DataKey::IssuerList).unwrap();
        issuers.contains(issuer_address)
    }

    /// Gets the list of all whitelisted issuers.
    pub fn get_issuers(env: Env) -> Vec<Address> {
         env.storage().instance().get(&DataKey::IssuerList).unwrap_or(Vec::new(&env))
    }

    /// Transfers admin privileges to a new address.
    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);

        // Emit event
        env.events().publish(
            (symbol_short!("new_admin"),),
            new_admin
        );
    }
}
