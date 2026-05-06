#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Token,
    CreatorBalance(Address),
    CreatorTotal(Address),
}

#[contract]
pub struct TipJar;

#[contractimpl]
impl TipJar {
    /// One-time initialisation: store the token contract address.
    pub fn init(env: Env, token: Address) {
        if env.storage().instance().has(&DataKey::Token) {
            panic!("already initialised");
        }
        env.storage().instance().set(&DataKey::Token, &token);
    }

    /// TODO: Transfer `amount` tokens from `sender` into escrow for `creator`.
    /// - Require sender auth
    /// - Validate amount > 0
    /// - Transfer tokens sender → contract
    /// - Update CreatorBalance and CreatorTotal
    /// - Emit ("tip", creator) event
    pub fn tip(_env: Env, _sender: Address, _creator: Address, _amount: i128) {
        unimplemented!("tip: not yet implemented")
    }

    /// TODO: Return cumulative total tips received by `creator`.
    pub fn get_total_tips(_env: Env, _creator: Address) -> i128 {
        unimplemented!("get_total_tips: not yet implemented")
    }

    /// TODO: Transfer creator's escrowed balance to their wallet.
    /// - Require creator auth
    /// - Validate balance > 0
    /// - Transfer tokens contract → creator
    /// - Reset CreatorBalance to 0
    /// - Emit ("withdraw", creator) event
    pub fn withdraw(_env: Env, _creator: Address) {
        unimplemented!("withdraw: not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::Address as _,
        token::StellarAssetClient,
        Address, Env,
    };

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TipJar, ());
        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
        (env, contract_id, token_id)
    }

    #[test]
    fn test_init() {
        let (env, contract_id, token_id) = setup();
        TipJarClient::new(&env, &contract_id).init(&token_id);
        // Token address stored — init succeeded without panic
    }

    #[test]
    #[should_panic(expected = "already initialised")]
    fn test_init_twice_panics() {
        let (env, contract_id, token_id) = setup();
        let client = TipJarClient::new(&env, &contract_id);
        client.init(&token_id);
        client.init(&token_id); // must panic
    }

    // TODO: test_tip_and_totals — blocked on tip() implementation
    // TODO: test_withdraw       — blocked on withdraw() implementation
    // TODO: test_invalid_tip_amount — blocked on tip() implementation
}
