#![deny(warnings)]

///
/// Plantary NFT Smart Contract
/// adapted from https://github.com/near-examples/NFT by mykle
///
/// Implements blockchain ledger for plants and their fruit
///


use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::collections::UnorderedSet;
use near_sdk::{env, near_bindgen, AccountId};
use rand::Rng;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// This trait provides the baseline of functions as described at:
/// https://github.com/nearprotocol/NEPs/blob/nep-4/specs/Standards/Tokens/NonFungibleToken.md
pub trait NEP4 {
    // Grant the access to the given `accountId` for the given `tokenId`.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn grant_access(&mut self, escrow_account_id: AccountId);

    // Revoke the access to the given `accountId` for the given `tokenId`.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn revoke_access(&mut self, escrow_account_id: AccountId);

    // Transfer the given `tokenId` to the given `accountId`. Account `accountId` becomes the new owner.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId); 

    // Transfer the given `tokenId` to the given `accountId`. Account `accountId` becomes the new owner.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should be the owner of the token. Callers who have
    // escrow access should use transfer_from.
    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId); 

    // Returns `true` or `false` based on caller of the function (`predecessor_id) having access to the token
    fn check_access(&self, account_id: AccountId) -> bool;

    // Get an individual owner by given `tokenId`.
    fn get_token_owner(&self, token_id: TokenId) -> String;
}


/// The token ID type is also defined in the NEP
pub type TokenId = u64;
pub type AccountIdHash = Vec<u8>;
pub type VeggieType = i8;

///
/// the veggie section
/// veggie is like a superclass of both plant and harvest.
/// (not necessarily the right way to do this in rust, i'm still learning ...)
///


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Veggie {
    pub id: TokenId,
    pub vtype: VeggieType,
    pub vsubtype: String,
    pub parent: TokenId,
}

impl Veggie {
    pub const VTYPE_PLANT: VeggieType = 1;
    pub const VTYPE_HARVEST: VeggieType = 1;

    pub fn new(id: TokenId, vtype: VeggieType, vsubtype: String) -> Self {
        Self {
            id: id,
            vtype: vtype,           // plant or harvest 
            vsubtype: vsubtype,
            parent: 0,
            // dna ...
            // rarity ...
        }
    }
}

// veggie traits
//

pub trait Veggies {
    fn create_veggie(&mut self, 
                    vtype: VeggieType,
                    vsubtype: String,
                    ) -> Veggie;

    fn get_veggie(&self, vid: TokenId) -> Veggie;

    fn delete_veggie(&mut self, vid: TokenId);
}

// veggie implementation
//

impl Veggies for NonFungibleTokenBasic {
    fn create_veggie(&mut self, 
                    vtype: VeggieType,
                    vsubtype: String,
                    ) -> Veggie {
        // make a local veggie
        let c = Veggie::new(self.gen_token_id(), vtype, vsubtype);
        // record in the static list of veggies
        self.veggies.insert(&c.id, &c);
        // record ownership in the nft structure
        self.mint_token(env::predecessor_account_id(), c.id);
        return c;
    }

    fn get_veggie(&self, vid: TokenId) -> Veggie {
        // TODO: check perms?
        let veggie = match self.veggies.get(&vid) {
            Some(c) => {
                c
            },
            None => {
                env::panic(b"Veggie does not exist.") // TODO: find pattern for throwing exception
            }
        };
        return veggie;
    }

    fn delete_veggie(&mut self, vid: TokenId) {
        // TODO: check perms?
        // delete from global list
        self.veggies.remove(&vid);
        // remove from ownership
        self.token_to_account.remove(&vid);
    }
}

/// end Veggie section

/// the Plant section, which delegates everything to Veggie
///
pub trait Plants {
    fn create_plant(&mut self, 
                    vsubtype: String,
                    )->Veggie;

    fn get_plant(&self, vid: TokenId) -> Veggie;

    fn delete_plant(&mut self, vid: TokenId);

}

impl Plants for NonFungibleTokenBasic {
    fn create_plant(&mut self,
                    vsubtype: String,
                    ) -> Veggie {
        //return self.create_veggie(PLANT_TYPE.to_string(), vsubtype);
        return self.create_veggie(Veggie::VTYPE_PLANT, vsubtype);
    }

    fn get_plant(&self, vid: TokenId) -> Veggie {
        return self.get_veggie(vid);
    }

    fn delete_plant(&mut self, vid: TokenId){
        return self.delete_veggie(vid);
    }
}

/// end Plant section

/// the Harvest section, which also delegates everything to Veggie
///

pub trait Harvests {
    fn create_harvest(&mut self,
                    vsubtype: String,
                    )->Veggie;

    fn get_harvest(self, vid: TokenId) -> Veggie;

    fn delete_harvest(&mut self, vid: TokenId);

    fn harvest_plant(&mut self, parent: Veggie) -> Veggie;
}

impl Harvests for NonFungibleTokenBasic {
    fn create_harvest(&mut self,
                    vsubtype: String,
                    ) -> Veggie {
        return self.create_veggie(Veggie::VTYPE_HARVEST, vsubtype);
    }

    fn get_harvest(self, vid: TokenId) -> Veggie {
        return self.get_veggie(vid);
    }

    fn delete_harvest(&mut self, vid: TokenId){
        return self.delete_veggie(vid);
    }

    // harvest_plant() a plant veggie gives birth to a harvest veggie
    // (harvest in this case is a verb.)
    //
    fn harvest_plant(&mut self, parent: Veggie) -> Veggie {
        // Assert: parent is a plant
        //if parent.vtype != PLANT_TYPE.to_string() {
        if parent.vtype != Veggie::VTYPE_PLANT {
            env::panic(b"non-plant harvest");
        }
        // TODO: enumerate all the types of harvest for each type of plant
        let mut h = self.create_harvest("generic harvest".to_string());
        h.parent = parent.id;
        return h;
    }
}

/// end Harvest section

// Begin implementation
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleTokenBasic {
    pub token_to_account: UnorderedMap<TokenId, AccountId>,
    pub account_gives_access: UnorderedMap<AccountIdHash, UnorderedSet<AccountIdHash>>, // Vec<u8> is sha256 of account, makes it safer and is how fungible token also works
    pub owner_id: AccountId,

    pub veggies: UnorderedMap<TokenId, Veggie>,
}

impl Default for NonFungibleTokenBasic {
    fn default() -> Self {
        panic!("NFT should be initialized before usage")
    }
}

#[near_bindgen]
impl NonFungibleTokenBasic {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Owner's account ID is invalid.");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            token_to_account: UnorderedMap::new(b"token-belongs-to".to_vec()),
            account_gives_access: UnorderedMap::new(b"gives-access".to_vec()),
            owner_id,
            veggies: UnorderedMap::new(b"veggies".to_vec()),
        }
    }

    fn gen_token_id(&self) -> TokenId {
        let mut rng = rand::thread_rng();
        let id: TokenId = rng.gen();
        // TODO: test that it doesn't already exist.
        return id; 
    }
}

#[near_bindgen]
impl NEP4 for NonFungibleTokenBasic {
    fn grant_access(&mut self, escrow_account_id: AccountId) {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        let predecessor = env::predecessor_account_id();
        let predecessor_hash = env::sha256(predecessor.as_bytes());

        let mut access_set = match self.account_gives_access.get(&predecessor_hash) {
            Some(existing_set) => {
                existing_set
            },
            None => {
                UnorderedSet::new(b"new-access-set".to_vec())
            }
        };
        access_set.insert(&escrow_hash);
        self.account_gives_access.insert(&predecessor_hash, &access_set);
    }

    fn revoke_access(&mut self, escrow_account_id: AccountId) {
        let predecessor = env::predecessor_account_id();
        let predecessor_hash = env::sha256(predecessor.as_bytes());
        let mut existing_set = match self.account_gives_access.get(&predecessor_hash) {
            Some(existing_set) => existing_set,
            None => env::panic(b"Access does not exist.")
        };
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        if existing_set.contains(&escrow_hash) {
            existing_set.remove(&escrow_hash);
            self.account_gives_access.insert(&predecessor_hash, &existing_set);
            env::log(b"Successfully removed access.")
        } else {
            env::panic(b"Did not find access for escrow ID.")
        }
    }

    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId) {
        let token_owner_account_id = self.get_token_owner(token_id);
        let predecessor = env::predecessor_account_id();
        if predecessor != token_owner_account_id {
            env::panic(b"Attempt to call transfer on tokens belonging to another account.")
        }
        self.token_to_account.insert(&token_id, &new_owner_id);
    }

    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId) {
        let token_owner_account_id = self.get_token_owner(token_id);
        if owner_id != token_owner_account_id {
            env::panic(b"Attempt to transfer a token from a different owner.")
        }

        if !self.check_access(token_owner_account_id) {
            env::panic(b"Attempt to transfer a token with no access.")
        }
        self.token_to_account.insert(&token_id, &new_owner_id);
    }

    fn check_access(&self, account_id: AccountId) -> bool {
        let account_hash = env::sha256(account_id.as_bytes());
        let predecessor = env::predecessor_account_id();
        if predecessor == account_id {
            return true;
        }
        match self.account_gives_access.get(&account_hash) {
            Some(access) => {
                let predecessor = env::predecessor_account_id();
                let predecessor_hash = env::sha256(predecessor.as_bytes());
                access.contains(&predecessor_hash)
            },
            None => false
        }
    }

    fn get_token_owner(&self, token_id: TokenId) -> String {
        match self.token_to_account.get(&token_id) {
            Some(owner_id) => owner_id,
            None => env::panic(b"No owner of the token ID specified")
        }
    }
}

/// Methods not in the strict scope of the NFT spec (NEP4)
#[near_bindgen]
impl NonFungibleTokenBasic {
    /// Creates a token for owner_id, doesn't use autoincrement, fails if id is taken
    pub fn mint_token(&mut self, owner_id: String, token_id: TokenId) {
        // make sure that only the owner can call this funtion
        self.only_owner();
        // Since Map doesn't have `contains` we use match
        let token_check = self.token_to_account.get(&token_id);
        if token_check.is_some() {
            env::panic(b"Token ID already exists.")
        }
        // No token with that ID exists, mint and add token to data structures
        self.token_to_account.insert(&token_id, &owner_id);
    }

    /// helper function determining contract ownership
    fn only_owner(&mut self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only contract owner can call this method.");
    }
}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn joe() -> AccountId {
        "joe.testnet".to_string()
    }
    fn robert() -> AccountId {
        "robert.testnet".to_string()
    }
    fn mike() -> AccountId {
        "mike.testnet".to_string()
    }

    // part of writing unit tests is setting up a mock context
    // this is a useful list to peek at when wondering what's available in env::*
    fn get_context(predecessor_account_id: String, storage_usage: u64) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "jane.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn grant_access() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(robert());
        let length_before = contract.account_gives_access.len();
        assert_eq!(0, length_before, "Expected empty account access Map.");
        contract.grant_access(mike());
        contract.grant_access(joe());
        let length_after = contract.account_gives_access.len();
        assert_eq!(1, length_after, "Expected an entry in the account's access Map.");
        let predecessor_hash = env::sha256(robert().as_bytes());
        let num_grantees = contract.account_gives_access.get(&predecessor_hash).unwrap();
        assert_eq!(2, num_grantees.len(), "Expected two accounts to have access to predecessor.");
    }

    #[test]
    #[should_panic(
        expected = r#"Access does not exist."#
    )]
    fn revoke_access_and_panic() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(robert());
        contract.revoke_access(joe());
    }

    #[test]
    fn add_revoke_access_and_check() {
        // Joe grants access to Robert
        let mut context = get_context(joe(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(joe());
        contract.grant_access(robert());

        // does Robert have access to Joe's account? Yes.
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        let mut robert_has_access = contract.check_access(joe());
        assert_eq!(true, robert_has_access, "After granting access, check_access call failed.");

        // Joe revokes access from Robert
        context = get_context(joe(), env::storage_usage());
        testing_env!(context);
        contract.revoke_access(robert());

        // does Robert have access to Joe's account? No
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        robert_has_access = contract.check_access(joe());
        assert_eq!(false, robert_has_access, "After revoking access, check_access call failed.");
    }

    #[test]
    fn mint_token_get_token_owner() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(robert());
        contract.mint_token(mike(), 19u64);
        let owner = contract.get_token_owner(19u64);
        assert_eq!(mike(), owner, "Unexpected token owner.");
    }

    #[test]
    #[should_panic(
        expected = r#"Attempt to transfer a token with no access."#
    )]
    fn transfer_from_with_no_access_should_fail() {
        // Mike owns the token.
        // Robert is trying to transfer it to Robert's account without having access.
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(robert());
        let token_id = 19u64;
        contract.mint_token(mike(), token_id);
        contract.transfer_from(mike(), robert(), token_id.clone());
    }

    #[test]
    fn transfer_from_with_escrow_access() {
        // Escrow account: robert.testnet
        // Owner account: mike.testnet
        // New owner account: joe.testnet
        let mut context = get_context(mike(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(mike());
        let token_id = 19u64;
        contract.mint_token(mike(), token_id);
        // Mike grants access to Robert
        contract.grant_access(robert());

        // Robert transfers the token to Joe
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        contract.transfer_from(mike(), joe(), token_id.clone());

        // Check new owner
        let owner = contract.get_token_owner(token_id.clone());
        assert_eq!(joe(), owner, "Token was not transferred after transfer call with escrow.");
    }

    #[test]
    #[should_panic(
        expected = r#"Attempt to transfer a token from a different owner."#
    )]
    fn transfer_from_with_escrow_access_wrong_owner_id() {
        // Escrow account: robert.testnet
        // Owner account: mike.testnet
        // New owner account: joe.testnet
        let mut context = get_context(mike(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(mike());
        let token_id = 19u64;
        contract.mint_token(mike(), token_id);
        // Mike grants access to Robert
        contract.grant_access(robert());

        // Robert transfers the token to Joe
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        contract.transfer_from(robert(), joe(), token_id.clone());
    }

    #[test]
    fn transfer_from_with_your_own_token() {
        // Owner account: robert.testnet
        // New owner account: joe.testnet

        testing_env!(get_context(robert(), 0));
        let mut contract = NonFungibleTokenBasic::new(robert());
        let token_id = 19u64;
        contract.mint_token(robert(), token_id);

        // Robert transfers the token to Joe
        contract.transfer_from(robert(), joe(), token_id.clone());

        // Check new owner
        let owner = contract.get_token_owner(token_id.clone());
        assert_eq!(joe(), owner, "Token was not transferred after transfer call with escrow.");
    }

    #[test]
    #[should_panic(
        expected = r#"Attempt to call transfer on tokens belonging to another account."#
    )]
    fn transfer_with_escrow_access_fails() {
        // Escrow account: robert.testnet
        // Owner account: mike.testnet
        // New owner account: joe.testnet
        let mut context = get_context(mike(), 0);
        testing_env!(context);
        let mut contract = NonFungibleTokenBasic::new(mike());
        let token_id = 19u64;
        contract.mint_token(mike(), token_id);
        // Mike grants access to Robert
        contract.grant_access(robert());

        // Robert transfers the token to Joe
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        contract.transfer(joe(), token_id.clone());
    }

    #[test]
    fn transfer_with_your_own_token() {
        // Owner account: robert.testnet
        // New owner account: joe.testnet

        testing_env!(get_context(robert(), 0));
        let mut contract = NonFungibleTokenBasic::new(robert());
        let token_id = 19u64;
        contract.mint_token(robert(), token_id);

        // Robert transfers the token to Joe
        contract.transfer(joe(), token_id.clone());

        // Check new owner
        let owner = contract.get_token_owner(token_id.clone());
        assert_eq!(joe(), owner, "Token was not transferred after transfer call with escrow.");
    }

    #[test]
    #[should_panic(
        expected = r#"Veggie does not exist."#
    )]
    fn create_delete_veggie() {
        testing_env!(get_context(robert(), 0));
        let mut contract = NonFungibleTokenBasic::new(robert());

            // create
        //let tc = contract.create_veggie(PLANT_TYPE.to_string(), "generic".to_string());
        let tc = contract.create_veggie(Veggie::VTYPE_PLANT, "generic".to_string());
            // inspect?
        let tcid = tc.id;
        assert_eq!(tc.vsubtype, "generic".to_string(), "subtype not found.");
            // delete
        contract.delete_veggie(tcid); // TODO: veggie should have own methods? so like tc.burn() ...
            // confirm deleted
        let _notc = contract.get_veggie(tcid); // should panic
    }

    /*
    #[test]
    fn create_delete_plant(){
        testing_env!(get_context(robert(), 0));
        let mut contract = NonFungibleTokenBasic::new(robert());

        let tc = contract.create_plant("generic")
    }

    #[test]
    fn harvest_plant(){
    }

    #[test]
    fn transfer_veggies(){
    }
    */
}