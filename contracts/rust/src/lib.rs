#![deny(warnings)]

///
/// Plantary NFT Smart Contract
/// adapted from https://github.com/near-examples/NFT by mykle
///
/// Implements blockchain ledger for plants and their fruit
///

use near_sdk::{env, near_bindgen, AccountId, Balance, json_types};
use near_sdk::collections::{UnorderedMap, Vector};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::json_types::U64;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_seeder::{Seeder};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

mod token_bank;
use token_bank::{NEP4, TokenBank, TokenSet, TokenId};

mod constants;
use constants::{VeggieType, VeggieCategory, vtypes, P_PRICES, H_PRICES, seedstates};

///
/// the veggie section
/// veggie is like a superclass of both plant and harvest.
/// (not necessarily the right way to do this in rust, i'm still learning ...)
///

#[derive(PartialEq, Clone, Debug, Serialize, BorshDeserialize, BorshSerialize)]
pub struct Veggie {
    pub vid: TokenId,
    pub vtype: VeggieType,
    pub vcat: VeggieCategory,
    pub parent: TokenId,
    pub dna: u64,
    pub meta_url: String,
}

impl Veggie {
    pub fn new(vid: TokenId, parent_vid: TokenId, vtype: VeggieType, vcat:VeggieCategory, dna: u64, meta_url: &String) -> Self {

        Self {
            vid: vid,
            vtype: vtype,           // plant or harvest 
            vcat: vcat,
            parent: parent_vid,
            dna: dna,
            meta_url: meta_url.to_string(),
            // rarity ...
        }
    }
}

// this is the external, JSON-compatible version for method calls.  (u64s are strings.)

pub type TokenU64 = json_types::U64;

#[derive(PartialEq, Clone, Debug, Serialize, BorshDeserialize, BorshSerialize)]
pub struct VeggieU64 {
    pub vid: TokenU64,
    pub vtype: VeggieType,
    pub vcat: VeggieCategory,
    pub parent: TokenU64,
    pub dna: json_types::U64,
    pub meta_url: String,
}

impl From<Veggie> for VeggieU64 {
    fn from(v: Veggie) -> Self {
        Self {
            vid: v.vid.into(),
            vtype: v.vtype,
            vcat: v.vcat,
            parent: v.parent.into(),
            dna: v.dna.into(),
            meta_url: v.meta_url
        }
    }
}

// I thought Rust would give me this for free ...
impl From<VeggieU64> for Veggie {
    fn from(v: VeggieU64) -> Self {
        Self {
            vid: v.vid.into(),
            vtype: v.vtype,
            vcat: v.vcat,
            parent: v.parent.into(),
            dna: v.dna.into(),
            meta_url: v.meta_url,
        }
    }
}

pub trait Veggies {
    fn get_veggie_u64(&self, vid_u64: TokenU64) -> VeggieU64;
    fn count_owner_veggies(&self, owner_id: AccountId, vtype: VeggieType) -> u64;
    fn get_owner_veggies_page_u64(&self, owner_id: AccountId, vtype: VeggieType, page_size: u16, page: u16) -> Vec<VeggieU64>;

    fn mint_plant_u64(&mut self, 
                    vcat: VeggieCategory,
                    )->VeggieU64;

    fn delete_veggie_u64(&mut self, vid_u64: TokenU64);

    fn harvest_plant_u64(&mut self, parent_id: TokenU64) -> VeggieU64;
}

// public veggies implementation
//
#[near_bindgen]
impl Veggies for PlantaryContract {

    fn count_owner_veggies(&self, owner_id: AccountId, vtype: VeggieType) -> u64 {
        self.check_vtype(vtype);

        let tokens = self.token_bank.get_owner_tokens(&owner_id);
            // type 0 means "count all veggies"
        if vtype == 0  { 
            return tokens.len();
        }
        
        let mut count = 0;
        for t in tokens.iter() {
            if self.veggies.get(&t).unwrap().vtype == vtype {
                count += 1;
            }
        }
        
        count
    }

    fn get_veggie_u64(&self, vid: TokenU64) -> VeggieU64 {
        self.get_veggie(vid.into()).into()
    }

    fn delete_veggie_u64(&mut self, vid: TokenU64){
        self.delete_veggie(vid.into()).into()
    }

    #[payable]
    fn harvest_plant_u64(&mut self, parent_id_u64: TokenU64) -> VeggieU64 {
        // confirm that we were paid the right amount:
        let parent_id = TokenId::from(parent_id_u64);
        let parent = self.get_veggie(parent_id);
        self.paid_up(H_PRICES[parent.vcat as usize]);

        self.harvest_plant(parent_id).into()
    }

    fn get_owner_veggies_page_u64(&self, owner_id: AccountId, vtype: VeggieType, page_size: u16, page: u16) -> Vec<VeggieU64> {
        self.get_owner_veggies_page(owner_id, vtype, page_size, page).iter().map(|v| VeggieU64::from(v.clone())).collect()
    }

    #[payable]
    fn mint_plant_u64(&mut self, vcat: VeggieCategory) -> VeggieU64 {
        // TODO: only putting this here for now because I haven't figured out how to unit test payments properly ...
        // confirm that we were paid the right amount
        self.paid_up(P_PRICES[vcat as usize]);
        self.mint_plant(vcat).into()
    }

}

////////////////////////
// private methods used by Veggies
//
impl PlantaryContract {
    fn get_veggie(&self, vid: TokenId) -> Veggie {
        let veggie = match self.veggies.get(&vid) {
            Some(c) => {
                c
            },
            None => {
                env::panic(b"Veggie does not exist.") 
            }
        };
        return veggie.clone();
    }

    fn delete_veggie(&mut self, vid: TokenId) {
        // panic if we're not the contract owner!
        self.only_owner();

        // delete from global list
        self.veggies.remove(&vid);
        // remove from ownership (should use burn_token)
        self.token_bank.token_to_account.remove(&vid);
    }

    fn mint_plant(&mut self,
                    vcat: VeggieCategory,
                    ) -> Veggie {
        // plants have no parents
        let parent_vid = 0;

        return self.create_veggie(vtypes::PLANT, vcat, parent_vid);
    }

    // harvest_plant() here, a plant veggie gives birth to a harvest veggie
    // (harvest in this case is a verb.)
    fn harvest_plant(&mut self, parent_id: TokenId) -> Veggie {
        // Assert: user owns this plant
        // Assert: this type of plant can even have a harvest
        // Assert: correct money was paid
        
        let parent = self.get_veggie(parent_id);

        // Assert: parent is a plant
        if parent.vtype != vtypes::PLANT {
            env::panic(b"non-plant harvest");
        }
        // for now, the harvest subtype is the same subtype as the parent plant
        let h = self.create_veggie(vtypes::HARVEST, parent.vcat, parent.vid);
        return h;
    }

    fn get_owner_veggies_page(&self, owner_id: AccountId, vtype: VeggieType, page_size: u16, page: u16) -> Vec<Veggie> {
        self.check_vtype(vtype);
        // get all owner tokens
        let tokens:TokenSet = self.token_bank.get_owner_tokens(&owner_id); // TokenSet == UnorderedSet<TokenId>
        // convert to all owner plants
        let mut owner_veggies: Vec<Veggie> = Vec::new();
        for ot in tokens.iter() {
            let ov = self.get_veggie(ot);
            if (vtype == 0) || (vtype == ov.vtype) { owner_veggies.push(ov); }
        }

        // calculate page, return it
        let count = owner_veggies.len();

        // pagesize 0?  try to return all results 
        if page_size == 0 {
            return owner_veggies;
        }

        let startpoint: usize = page_size as usize * page as usize;
        if startpoint > count { return Vec::new(); }

        let mut endpoint : usize =  startpoint + page_size as usize;
        if endpoint > count { endpoint = count; }

        owner_veggies[startpoint .. endpoint].to_vec()
    }

    // panic if invalid veggie types are attempted.
    fn check_vtype(&self, vtype: VeggieType){
        if ! (vtype == 0 || vtype == vtypes::PLANT || vtype == vtypes::HARVEST) {
            panic!("Unknown veggie type {}.", vtype);
        }
    }

    // panic if non-root tries to do a root thing
    fn only_owner(&mut self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only contract owner can call this method.");
    }

    // panic unless exactly 'tokens' N are  attached
    fn paid_up(&self, tokens: Balance) {
        let yocto = tokens * 10u128.pow(24);
        let dep = env::attached_deposit();
        if dep != yocto {
            panic!("needed {} yn, received {}", yocto, dep);
        }
    }

    // create a veggie with tokenID and random properties
    fn create_veggie(&mut self, 
                    vtype: VeggieType,
                    vcat: VeggieCategory,
                    parent_vid: TokenId,
                    ) -> Veggie {

        self.assert_valid_vtype(vtype);
        // TODO: validate vcat, parent

        // seed RNG
        let mut rng: ChaCha8Rng = Seeder::from(env::random_seed()).make_rng();

        // generate veggie-unique id
        let mut vid: TokenId;
        loop { 
            vid = rng.gen();
            match self.veggies.get(&vid) {
                None => { break; }
                Some(_) => { continue; }
            }
        }

        // pick a meta URL at random from the plant pool for the given subtype
        let meta_url: String;
        let sids = self.get_sids_of_type(vtype, vcat).unwrap();

        let sid = sids.get(rng.gen_range(0, sids.len()) ).unwrap();
        let seed = self.seeds.get(&sid).unwrap();
        meta_url = seed.meta_url;

        let dna: u64 = rng.gen();

        let v = Veggie::new(vid, parent_vid, vtype, vcat, dna, &meta_url);
        assert_eq!(vid, v.vid, "vid mismatch!");

        // record in the static list of veggies
        self.veggies.insert(&vid, &v); // vid has Copy trait; v does not.
        // record ownership in the nft structure
        self.token_bank.mint_token(env::predecessor_account_id(), vid);

        v
    }
}

// seed section:
// seeds are the NFT-art records that get minted;
// they don't have a veggieID yet, and they can express rarity, editions, etc.

pub type SeedId = U64;

#[derive(PartialEq, Clone, Debug, Serialize, BorshDeserialize, BorshSerialize)]
pub struct Seed {
    pub sid: SeedId,
    pub vtype: VeggieType,
    pub vcat: VeggieCategory,
    pub meta_url: String,
    pub rarity: f64,
    pub edition: u32,
    pub state: u8,
}

pub trait Seeds {
    fn create_seed(&mut self, vtype:VeggieType, vcat:VeggieCategory, meta_url:String, rarity:f64, edition:u32) 
        -> SeedId;
    fn update_seed(&mut self, sid: SeedId, vtype:VeggieType, vcat:VeggieCategory, meta_url:String, rarity:f64, edition:u32, state: u8) 
        -> SeedId;
    fn get_seed(&self, sid: SeedId) 
        -> Option<Seed>;
    fn get_seeds_page(&self, page_size: u16, page: u16)
        -> Vec<Seed>;
    fn get_seeds_of_type_page(&self, vtype: VeggieType, vcat: VeggieCategory, page_size: u16, page: u16) 
        -> Vec<Seed>;
    fn delete_seed(&mut self, sid: SeedId);
}

    // a group of seed IDs
pub type SeedIdSet = Vector<SeedId>;  // NEAR vector on trie in blockchain
    // a map from vcat to seedIdSet
pub type SeedSubIndex = UnorderedMap< VeggieCategory, SeedIdSet >;
    // an array of those, indexed by vtype 
pub type SeedIndex = Vec<SeedSubIndex>;


#[near_bindgen]
impl Seeds for PlantaryContract {

    //
    // write methods:
    //
    
    fn create_seed(&mut self, vtype:VeggieType, vcat:VeggieCategory, meta_url:String, rarity:f64, edition:u32) 
            -> SeedId {
        self.assert_admin();
        self.assert_valid_vtype(vtype);

        let mut s = Seed { 
            sid: 0.into(),
            vtype:vtype, 
            vcat:vcat, 
            meta_url:meta_url, 
            rarity:rarity, 
            edition:edition,
            state: seedstates::WAITING 
        };

        // generate a seed-unique id
        let mut rng: ChaCha8Rng = Seeder::from(env::random_seed()).make_rng();
        loop { 
            s.sid = rng.gen::<u64>().into();
            match self.seeds.get(&s.sid) {
                None => { break; }
                Some(_) => { continue; }
            }
        }
        
        // store:
        self.seeds.insert(&s.sid, &s); 

        // index:
        let seed_set = self.get_sids_of_type(vtype, vcat);
        
        match seed_set {
            Some(mut set) => { 
                set.push(&s.sid); 
                // OK ...
                // I had to add this line to fix a certain bug. This Some branch was having no 
                // effect on state.  Apparently 'set' at this point is a copy of the vector, not pointing
                // into seed_index, and writing to it doesn't save anything.  
                //
                // I can solve that like so:
                self.seed_index[vtype as usize].insert(&vcat, &set);
                // ... which works because the old vector is replaced with the new one.
                // But isn't that churning the storage layer?  Expensive in gas?  Maybe I'm doing this wrong?
                // Revisit when I understand things better.  For now, tests pass.
            } ,
            None => {
                // no seeds of this subtype have been added before, so:
                let mut name = b"seedidx".to_vec();
                name.push(vtype);
                name.push(58); // ascii ':'
                name.push(vcat);
                let mut new_set = SeedIdSet::new(name);
                new_set.push(&s.sid);
                self.seed_index[vtype as usize].insert(&vcat, &new_set);
            }
        };

        s.sid
    }


    fn update_seed(&mut self, sid: SeedId, vtype:VeggieType, vcat:VeggieCategory, meta_url:String, rarity:f64, edition:u32, state: u8) 
        ->SeedId{
        self.assert_admin();
        self.assert_valid_vtype(vtype);
        self.assert_valid_rarity(rarity);

        // TODO: validate state? is edition==0 invalid?
        
        let old_seed = self.seeds.get(&sid);
        match old_seed {
            None => {
                // seed must already exist
                env::panic(b"seed not found");
            },
            Some(os) => {
                //  disallow changing of vtype or vcat!
                if (os.vtype != vtype) || (os.vcat != vcat)  {
                    env::panic(b"cannot change seed types");
                }
                // reinsert on the same ID to update.
                let new_seed = Seed {
                    sid: sid, 
                    vtype: vtype,
                    vcat: vcat,
                    meta_url: meta_url,
                    rarity: rarity,
                    edition: edition,
                    state: state,
                };
                // (index already exists)
                self.seeds.insert(&sid, &new_seed); 
            }
        }

        sid
    }

    fn delete_seed(&mut self, sid: SeedId) {
        self.assert_admin();
        self.seeds.remove(&sid);
    }

    //
    // Note: no security on view methods; 
    // they are accountless, and all blockchain data is public anyway. 
    //
    // view methods:
    //

    fn get_seed(&self, sid: SeedId) -> Option<Seed>{
        self.seeds.get(&sid)
    }

    fn get_seeds_page(&self, page_size: u16, page: u16) -> Vec<Seed>{
        let count: usize  = self.seeds.len() as usize;

        let seeds_vec = self.seeds.values().collect();

        if page_size == 0 {
            // try to return all results
            return seeds_vec;
        }

        let startpoint: usize = page_size as usize * page as usize;
        if startpoint > count { return Vec::new(); }

        let mut endpoint: usize  =  startpoint + page_size as usize ;
        if endpoint > count { endpoint = count; }

        seeds_vec[startpoint .. endpoint].to_vec()
    }

    // TODO: refactor this together with the prev, once it's working ... we need only one seed getter.
    fn get_seeds_of_type_page(&self, vtype: VeggieType, vcat: VeggieCategory, page_size: u16, page: u16) -> Vec<Seed>{
        if vtype==0 && vcat==0 {
            return self.get_seeds_page(page_size, page);
        }

        self.assert_valid_vtype(vtype);

        //let subtype_sids = self.seed_index[vtype as usize].get(&vcat).unwrap();
        //let sid_iter = subtype_sids.iter();
        //let seed_iter = sid_iter.map(|sid| self.seeds.get(&sid).unwrap());
        //let count = subtype_sids.len() as usize;
        //let seeds_vec: Vec<Seed> = seed_iter.collect();
        
        let seeds_vec = match self.get_seeds_of_type(vtype, vcat) {
            Some(v) => v,
            None => Vec::new()
        };

        let count = seeds_vec.len();

        if page_size == 0 {
            // try to return all results
            return seeds_vec;
        }

        let startpoint: usize = page_size as usize * page as usize;
        if startpoint > count { 
            return Vec::new() ;
        }

        let mut endpoint: usize  =  startpoint + page_size as usize ;
        if endpoint > count { endpoint = count; }

        seeds_vec[startpoint .. endpoint].to_vec()
    }

}

/////////////////////////
// private seed methods:
impl PlantaryContract { 

    fn get_sids_of_type(&self, vtype: VeggieType, vcat: VeggieCategory) -> Option<Vector<SeedId>>{
        self.seed_index[vtype as usize].get(&vcat)
    }

    fn get_seeds_of_type(&self, vtype: VeggieType, vcat: VeggieCategory) -> Option<Vec<Seed>>{
        match self.get_sids_of_type(vtype, vcat) {
            Some(v) => {
                return Some(v.iter().map(  |sid| self.seeds.get(&sid).unwrap()  ).collect());
            },
            None => {
                return None;
            }
        };
    }

}

// Access Control section

trait AccessControl {
    fn is_admin(&self, id: AccountId) -> bool; // test
    fn assert_admin(&self); // panic if not.
}

impl AccessControl for PlantaryContract {
    fn is_admin(&self, id: AccountId) -> bool {
        // simplest solution: owner is admin
        self.owner_id == id
    }
    fn assert_admin(&self) {
        if self.owner_id == env::predecessor_account_id() { return }
        if env::predecessor_account_id() == "mykletest.testnet" { return }
        if env::predecessor_account_id() == "lenara.testnet" { return }

        env::panic(b"Access Denied");
    }
}

// Validation section

trait Validation {
    fn assert_valid_vtype(&self, v: VeggieType);
    fn assert_valid_rarity(&self, r: f64);
}

impl Validation for PlantaryContract {
    fn assert_valid_vtype(&self, v: VeggieType) {
        if (v != vtypes::PLANT) && (v != vtypes::HARVEST) {
            env::panic(b"Invalid veggie type");
        }
        
    }
    fn assert_valid_rarity(&self, r: f64) {
        if (r < 1.0) || (r > 10.0){
            env::panic(b"Invalid rarity");
        }
    }
}

// Our main contract object is PlantaryContract

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct PlantaryContract {
    // first international bank of NFTs
    pub token_bank: TokenBank,
    // owner of the contract:
    pub owner_id: AccountId,
    // metadata storage
    pub veggies: UnorderedMap<TokenId, Veggie>,
    // seed storage
    pub seeds: UnorderedMap<SeedId, Seed>,
    // seed index: a (very short) array of umaps of sets
    pub seed_index: SeedIndex,
}

impl Default for PlantaryContract {
    fn default() -> Self {
        env::panic(b"plantary should be initialized before usage");
    }
}

// Public contract methods, callable on interwebs:
#[near_bindgen]
impl PlantaryContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Owner's account ID is invalid.");
        assert!(!env::state_exists(), "Already initialized");
        let vs0 = SeedSubIndex::new(b"seedSub0".to_vec()); // unused
        let vs1 = SeedSubIndex::new(b"seedSub1".to_vec()); // plants
        let vs2 = SeedSubIndex::new(b"seedSub2".to_vec()); // harvests

        let mut vt = UnorderedMap::new(b"seedIdx".to_vec());
        vt.insert(&vtypes::PLANT, &vs1);
        vt.insert(&vtypes::HARVEST, &vs2);

        Self {
            token_bank: TokenBank::new(),
            owner_id,
            veggies: UnorderedMap::new(b"veggies".to_vec()),
            seeds: UnorderedMap::new(b"seeds".to_vec()),
            seed_index: vec![ vs0, vs1, vs2 ]
        }

    }


    pub fn get_owner_tokens(&self, owner_id: &AccountId) -> Vec<TokenU64> {
        self.token_bank.get_owner_tokens(&owner_id).iter().map(|t| TokenU64::from(t)).collect()
    }

    // debug 
    pub fn get_veggie_keys(&self) -> Vec<TokenU64> {
        self.veggies.keys().map(|i| TokenU64::from(i)).collect()
    }

}

// Expose NEP-4 interface of TokenBank
//
// NOTE: these token_id values are specified by NEP4 as 64-bit unsigned ints,
// which Javascript will truncate to 58 bits (if not somehow solved with BigInt)
#[near_bindgen]
impl NEP4 for PlantaryContract {
    fn grant_access(&mut self, escrow_account_id: AccountId) {
        self.token_bank.grant_access(escrow_account_id)
    }

    fn revoke_access(&mut self, escrow_account_id: AccountId) {
        self.token_bank.revoke_access(escrow_account_id)
    }

    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId) {
        self.token_bank.transfer_from(owner_id, new_owner_id, token_id)
    }

    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId) {
        self.token_bank.transfer(new_owner_id, token_id) 
    }

    fn check_access(&self, account_id: &AccountId) -> bool {
        self.token_bank.check_access(account_id)
    }

    fn get_token_owner(&self, token_id: TokenId) -> String {
        self.token_bank.get_token_owner(token_id)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext, Balance};
    use constants::{vtypes, vcats, seedstates};

    fn to_ynear(near: Balance) -> Balance {
        near * 10u128.pow(24)
    }

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
            account_balance: 10u128.pow(28),
            account_locked_balance: 0,
            storage_usage,
            attached_deposit: 10u128.pow(27),
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    // loading some default seeds into the contract, for testing create_veggie
    // Look out, this sort of thing can break the bank ...
    fn load_default_seeds(contract: &mut PlantaryContract){
        // type, subtype, meta_url, rarity, edition

        assert!(contract.get_sids_of_type(vtypes::PLANT, vcats::ORACLE).is_none(), "seed index broken already");

        contract.create_seed(vtypes::PLANT, vcats::ORACLE, 
    "https://3bvdryfdm3sswevmvr3poka2ucda5dfqag3bz4td72affctbmaea.arweave.net/2Go44KNm5SsSrKx29ygaoIYOjLABthzyY_6AUophYAg".to_string(),
            5.0, 1,
        );
        assert_eq!(contract.get_sids_of_type(vtypes::PLANT, vcats::ORACLE).unwrap().len(), 1, "seed index broken after 1");

        contract.create_seed(vtypes::PLANT, vcats::ORACLE, 
    "https://vwanp7rn32rioq6ofcvglo52sgdrctcfkc4v7uiy7bbimtzijz3q.arweave.net/rYDX_i3eoodDziiqZbu6kYcRTEVQuV_RGPhChk8oTnc".to_string(),
            5.0, 1,
        );
        assert_eq!(contract.get_sids_of_type(vtypes::PLANT, vcats::ORACLE).unwrap().len(), 2, "seed index broken after 2");


        contract.create_seed(vtypes::PLANT, vcats::PORTRAIT, 
    "https://rsigfpny3j3uwohxfeo7tdkdvw6yhaefxt6d3uq7kajtpaqtdfwq.arweave.net/jJBivbjad0s49ykd-Y1Drb2DgIW8_D3SH1ATN4ITGW0".to_string(),
            5.0, 1,
        );
        contract.create_seed(vtypes::PLANT, vcats::PORTRAIT, 
    "https://arweave.net/fo--Wlh83Ka83zVQqliiwFq_4zbc1H7vrZNlvA_Gkek".to_string(),
            5.0, 1,
        );

        contract.create_seed(vtypes::PLANT, vcats::MONEY, 
    "https://rj32ukhcq4hdq7nux3rntp5ffdk3ff2kzjcalpy3mc7batjytoza.arweave.net/ineqKOKHDjh9tL7i2b-lKNWyl0rKRAW_G2C-EE04m7I".to_string(),
            5.0, 1,
        );
        contract.create_seed(vtypes::PLANT, vcats::MONEY, 
    "https://b2zjlf2zplj5we2bdar6p6smu3o6fdu7o7ed23takt63lck6peoq.arweave.net/DrKVl1l609sTQRgj5_pMpt3ijp93yD1uYFT9tYleeR0".to_string(),
            5.0, 1,
        );



        contract.create_seed(vtypes::HARVEST, vcats::ORACLE, 
    "https://arweave.net/v63RbTVHhGKr7UNMmwMjBtKepk1I26UB4yxPhJVSkcg".to_string(),
            5.0, 1,
        );
        contract.create_seed(vtypes::HARVEST, vcats::ORACLE, 
    "https://arweave.net/hvOKZAw3miEA8BE4VewzH9io4fNsSWyZpGZaSmhr-l8".to_string(),
            5.0, 1,
        );
        contract.create_seed(vtypes::HARVEST, vcats::ORACLE, 
    "https://arweave.net/B_c8uZaUFIA8hjLDVr3v4IR6aRT-zzvCaE0cqWgVURc".to_string(),
            5.0, 1,
        );

        contract.create_seed(vtypes::HARVEST, vcats::PORTRAIT, 
    "https://arweave.net/tmOUL9xwL8LQb_E5kOldLaF0mrZLg9rSMYpoTGgdkU8".to_string(),
            5.0, 1,
        );
        contract.create_seed(vtypes::HARVEST, vcats::PORTRAIT, 
    "https://arweave.net/tvCQax-rq-oDvRdy-QnBp5orrjSP04Y-dNxXC3maTkI".to_string(),
            5.0, 1,
        );
        contract.create_seed(vtypes::HARVEST, vcats::PORTRAIT, 
    "https://arweave.net/CJyoNeeDM_Vco0l4-7y434_pe4hBhWEE9vvh5XqMd4k".to_string(),
            5.0, 1,
        );
    }

    // access control tests:
    // test that mike can't admin robert's contract
    #[test]
    #[should_panic(
        expected = r#"Access Denied"#
    )]
    fn assert_admin() {
        testing_env!(get_context(joe(), 0));
        // owner == admin == robert
        let contract = PlantaryContract::new(robert());

        // this should panic because current_account_id == joe
        contract.assert_admin();
    }

    // these should all pass:
    #[test]
    fn assert_admin_2() {
        testing_env!(get_context("mykletest.testnet".to_string(), 0));
        let contract1 = PlantaryContract::new(robert());
        contract1.assert_admin();
    }

    #[test]
    fn is_admin() {
        testing_env!(get_context(joe(), 0));
        // owner == admin == robert
        let contract = PlantaryContract::new(robert());

        // So this should return true:
        assert!(contract.is_admin(robert()), "robert is not robert");

        // And this should return false:
        assert!(! contract.is_admin(mike()), "mike is robert");
    }


    // validation tests:
    #[test]
    #[should_panic(
        expected = r#"Invalid veggie type"#
    )]
    fn assert_valid_vtype() {
        testing_env!(get_context(robert(), 0));
        let contract = PlantaryContract::new(robert());
        contract.assert_valid_vtype(vtypes::PLANT);
        contract.assert_valid_vtype(vtypes::HARVEST);
        contract.assert_valid_vtype(0); // should panic
    }

    #[test]
    #[should_panic(
        expected = r#"Invalid veggie type"#
    )]
    fn assert_valid_vtype_2() {
        testing_env!(get_context(robert(), 0));
        let contract = PlantaryContract::new(robert());
        contract.assert_valid_vtype(3); // should panic
    }

    #[test]
    #[should_panic(
        expected = r#"Invalid rarity"#
    )]
    fn assert_valid_rarity() {
        testing_env!(get_context(robert(), 0));
        let contract = PlantaryContract::new(robert());
        contract.assert_valid_rarity(1.0); 
        contract.assert_valid_rarity(10.0);
        contract.assert_valid_rarity(9.999);
        contract.assert_valid_rarity(0.9); // should panic
    }

    // test we can create & get, and that we can't get what we haven't created
    #[test]
    fn crud_seed(){
        testing_env!(get_context(robert(), 0));
        let mut contract = PlantaryContract::new(robert());
        let t = Seed {
            sid: 0.into(),
            vtype: vtypes::PLANT, 
            vcat: vcats::ORACLE, 
            meta_url: "http://google.com".to_string(), 
            rarity: 3.14, 
            edition: 1,
            state: seedstates::WAITING,
        };
        // testing create, get
        let sid = contract.create_seed(t.vtype, t.vcat, t.meta_url.clone(), t.rarity, t.edition);
        let mut seed = contract.get_seed(sid).unwrap();
        assert_eq!(seed.sid, sid, "bad seed ID");
        assert_eq!(seed.vtype, t.vtype, "bad vtype");
        assert_eq!(seed.vcat, t.vcat, "bad vcat");
        assert_eq!(seed.meta_url, t.meta_url, "bad meta_url");
        assert_eq!(seed.rarity, t.rarity, "bad rarity");
        assert_eq!(seed.edition, t.edition, "bad edition");
        
        // testing update

        seed.meta_url = "http://youtube.com".to_string();
        seed.rarity = 5.0;
        seed.edition = 4;
        assert_eq!(contract.update_seed(sid, seed.vtype, seed.vcat, seed.meta_url.clone(), seed.rarity, seed.edition, seed.state), sid, "bad update"); 

        // testing delete
        contract.delete_seed(sid);
        let deleted_seed = contract.get_seed(sid);
        assert!(deleted_seed.is_none());

        // negative test of get:
        let seed2 = contract.get_seed(12345.into());
        assert!(seed2.is_none());
    }


    #[test]
    fn get_sids_of_type(){
        let c = get_context(robert(), 0);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract); // 6 plants, 6 harvests

        assert_eq!(contract.get_seeds_page(0,0).len(), 12, "bad seed count");
        assert_eq!(contract.get_sids_of_type(vtypes::PLANT, vcats::ORACLE).unwrap().len(), 2, "bad sid count");
        assert_eq!(contract.get_sids_of_type(vtypes::PLANT, vcats::PORTRAIT).unwrap().len(), 2, "bad sid count");
        assert_eq!(contract.get_sids_of_type(vtypes::PLANT, vcats::MONEY).unwrap().len(), 2, "bad sid count");
        assert_eq!(contract.get_sids_of_type(vtypes::HARVEST, vcats::ORACLE).unwrap().len(), 3, "bad sid count");
        assert_eq!(contract.get_sids_of_type(vtypes::HARVEST, vcats::PORTRAIT).unwrap().len(), 3, "bad sid count");
        assert!(contract.get_sids_of_type(vtypes::HARVEST, vcats::MONEY).is_none(), "bad sid count");
    }

    #[test]
    fn get_seeds_of_type(){
        let c = get_context(robert(), 0);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract); // 6 plants, 6 harvests

        assert_eq!(contract.get_seeds_page(0,0).len(), 12, "bad seed count");
        assert_eq!(contract.get_seeds_of_type(vtypes::PLANT, vcats::ORACLE).unwrap().len(), 2, "bad sid count");
        assert_eq!(contract.get_seeds_of_type(vtypes::PLANT, vcats::PORTRAIT).unwrap().len(), 2, "bad sid count");
        assert_eq!(contract.get_seeds_of_type(vtypes::PLANT, vcats::MONEY).unwrap().len(), 2, "bad sid count");
        assert_eq!(contract.get_seeds_of_type(vtypes::HARVEST, vcats::ORACLE).unwrap().len(), 3, "bad sid count");
        assert_eq!(contract.get_seeds_of_type(vtypes::HARVEST, vcats::PORTRAIT).unwrap().len(), 3, "bad sid count");
        assert!(contract.get_seeds_of_type(vtypes::HARVEST, vcats::MONEY).is_none(), "bad sid count");
    }

    #[test]
    fn get_seeds_page(){
        let c = get_context(robert(), 0);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());

        // plant 23 seeds
        for n in 0..23 {
            contract.create_seed(
                vtypes::PLANT,
                vcats::ORACLE,
                "http://google.com".to_string(),
                3.14,
                n,
            );
        }

        // test seeds:
        // get three pages of size 7
        // check that they are all full
        for p in 0..3 {
            let seeds = contract.get_seeds_page(7,p);
            assert_eq!(seeds.len(), 7, "bad seed page size");
        }

        // get another page of size 7
        // check that it is only 2 items long
        let seeds = contract.get_seeds_page(7,3);
        assert_eq!(seeds.len(), 2, "bad seed end page size");

        // get yet another page, should be empty.
        let seeds = contract.get_seeds_page(7,100);
        assert_eq!(seeds.len(), 0, "bad seed blank page size");

        // check that we can get the whole thing in one big slurp
        let seeds = contract.get_seeds_page(23,0);
        assert_eq!(seeds.len(), 23, "bad seed total page size");

        let seeds = contract.get_seeds_page(0,0);
        assert_eq!(seeds.len(), 23, "bad seed total page size");

        let seeds = contract.get_seeds_page(100,0);
        assert_eq!(seeds.len(), 23, "bad seed total page size");

    }

    // veggie tests:
    
    #[test]
    #[should_panic(
        expected = r#"Veggie does not exist."#
    )]
    fn create_delete_veggie() {
        testing_env!(get_context(robert(), 0));
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract);

            // create
        let v = contract.create_veggie(vtypes::PLANT, vcats::MONEY, 0);
            // inspect?
        assert_eq!(v.vtype, vtypes::PLANT, "vtype not saved");
        assert_eq!(v.vcat, vcats::MONEY, "vcat not found.");
            // find?
        let vid = v.vid;
            // confirm
        let _foundv = contract.get_veggie(vid); // should not panic
        assert_eq!(v, _foundv, "veggie did not fetch right");
            // delete
        contract.delete_veggie(vid); // TODO: should veggie have its own method? so like v.burn() ...
            // confirm deleted
        let _nov = contract.get_veggie(vid); // should panic
    }

    // TODO: test we can't delete a veggie we don't own (unless we are contract owner)
    // TODO: Test that we are charged some NEAR tokens when we mint a plant

    // Test for a certain bug with picking the wrong seeds ...
    // was due to non-unique names when creating near Vectors, bleh.
    #[test]
    fn veggie_scramble() {
        let c = get_context(robert(), 0);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());

        // exactly 1 plant seed & 1 harvest seed for 1 vcat
        contract.create_seed(vtypes::PLANT, vcats::ORACLE, 
    "https://url.com/planturl".to_string(),
            5.0, 1,
        );

        let mut plantseeds = contract.get_seeds_of_type_page(vtypes::PLANT, vcats::ORACLE, 0,0);
        assert_eq!(plantseeds.len(), 1, "wrong number of plant seeds");
        let mut harvestseeds = contract.get_seeds_of_type_page(vtypes::HARVEST, vcats::ORACLE, 0,0);
        assert_eq!(harvestseeds.len(), 0, "wrong number of harvest seeds");


        contract.create_seed(vtypes::HARVEST, vcats::ORACLE, 
    "https://url.com/harvesturl".to_string(),
            5.0, 1,
        );

        plantseeds = contract.get_seeds_of_type_page(vtypes::PLANT, vcats::ORACLE, 0,0);
        assert_eq!(plantseeds.len(), 1, "wrong number of plant seeds");
        harvestseeds = contract.get_seeds_of_type_page(vtypes::HARVEST, vcats::ORACLE, 0,0);
        assert_eq!(harvestseeds.len(), 1, "wrong number of harvest seeds");

        let plant = contract.create_veggie(vtypes::PLANT, vcats::ORACLE, 0);
        assert_eq!(plant.meta_url, "https://url.com/planturl", "bad plant url");
        let harvest = contract.create_veggie(vtypes::HARVEST, vcats::ORACLE, 0);
        assert_eq!(harvest.meta_url, "https://url.com/harvesturl", "bad harvest url");

    }

    #[test]
    fn harvest_plant(){
        let mut c = get_context(robert(), 0);
        c.attached_deposit = to_ynear(P_PRICES[vcats::PORTRAIT as usize]);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract);

            // create
        let p = contract.mint_plant(vcats::PORTRAIT);
        let h = contract.harvest_plant(p.vid);
            // inspect
        assert_eq!(p.vid, h.parent, "parentage suspect");
        assert_eq!(p.vcat, h.vcat, "mismatched subtype");
    }

    // TODO: test that we can't harvest a plant we don't own.

    #[test]
    #[should_panic(
        expected = r#"Veggie does not exist."#
    )]
    fn create_get_delete_veggie_u64(){
        testing_env!(get_context(robert(), 0));
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract);
            // create
        let v = contract.create_veggie(vtypes::PLANT, vcats::MONEY, 0);
            // inspect?
        assert_eq!(v.vtype, vtypes::PLANT, "vtype not saved");
        assert_eq!(v.vcat, vcats::MONEY, "vcat not found.");
            // find?
        let vid_u64 = TokenU64::from(v.vid);
            // confirm
        let _foundv: Veggie = contract.get_veggie_u64(vid_u64.clone()).into(); // should not panic
        assert_eq!(v, _foundv, "veggie did not fetch right");
            // delete
        contract.delete_veggie_u64(vid_u64.clone()); 
            // confirm deleted
        let _nov = contract.get_veggie_u64(vid_u64); // should panic
    }

    #[test]
    fn count_owner_veggies(){
        let c = get_context(robert(), 0);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract);

        // mint some plants
        let _p1 = contract.mint_plant(vcats::MONEY); 

        let _p2 = contract.mint_plant(vcats::ORACLE);

        let _p3 = contract.mint_plant(vcats::PORTRAIT);

        // harvest some fruit
        let _h1 = contract.harvest_plant(_p2.vid);
        let _h2 = contract.harvest_plant(_p3.vid);

        // count_owner_veggies should return 5 for type 0, which is "all"
        assert_eq!(5, contract.count_owner_veggies(robert(), 0));
        // count_owner_veggies should return 3 for type PLANT
        assert_eq!(3, contract.count_owner_veggies(robert(), vtypes::PLANT));
        // count_owner_veggies should return 2 for type HARVEST
        assert_eq!(2, contract.count_owner_veggies(robert(), vtypes::HARVEST));
        // this person has no veggies
        assert_eq!(0, contract.count_owner_veggies("jane.testnet".to_string(), 0));
    }

    #[test]
    #[should_panic(
        expected = r#"Unknown veggie type 23."#
    )]
    fn count_owner_veggies_unknown(){
        testing_env!(get_context(robert(), 0));
        let contract = PlantaryContract::new(robert());
        // count_owner_veggies() should panic for any unknown types
        assert_eq!(0, contract.count_owner_veggies(robert(), 23));
    }

    #[test]
    fn get_owner_veggies_page_1(){
        let c = get_context(robert(), 0);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract);

        // mint 23  plants
        for _n in 0..22 {
            contract.mint_plant(vcats::MONEY);
        }
        let _p23 = contract.mint_plant(vcats::ORACLE);

        // mint 5 harvests
        for _o in 0..5 {
            contract.harvest_plant(_p23.vid);
        }

        // test plants:
        // get three pages of size 7
        // check that they are all full
        for p in 0..3 {
            let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 7,p);
            assert_eq!(tokens.len(), 7, "bad plant page size");
        }

        // get another page of size 7
        // check that it is only 2 items long
        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 7,3);
        assert_eq!(tokens.len(), 2, "bad plant end page size");

        // get yet another page, should be empty.
        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 7,100);
        assert_eq!(tokens.len(), 0, "bad plant blank page size");

        // check that we can get the whole thing in one big slurp
        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 23,0);
        assert_eq!(tokens.len(), 23, "bad plant total page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 0,0);
        assert_eq!(tokens.len(), 23, "bad plant total page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 100,0);
        assert_eq!(tokens.len(), 23, "bad plant total page size");

    }

    #[test]
    fn get_owner_veggies_page_2(){
        testing_env!(get_context(robert(), 0));
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract);

        // mint 5 plants
        for _n in 0..3 {
            contract.mint_plant(vcats::MONEY);
        }
        let _p5 = contract.mint_plant(vcats::ORACLE);

        // mint 13 harvests
        for _o in 0..13 {
            contract.harvest_plant(_p5.vid);
        }

        // test harvests:
        for p in 0..2 {
            let tokens = contract.get_owner_veggies_page(robert(), vtypes::HARVEST, 4,p);
            assert_eq!(tokens.len(), 4, "bad harvest page size");
        }

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::HARVEST, 7,100);
        assert_eq!(tokens.len(), 0, "bad harvest blank page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::HARVEST, 13,0);
        assert_eq!(tokens.len(), 13, "bad harvest total page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::HARVEST, 0,0);
        assert_eq!(tokens.len(), 13, "bad harvest total page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::HARVEST, 100,0);
        assert_eq!(tokens.len(), 13, "bad harvest total page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::HARVEST, 6,2);
        assert_eq!(tokens.len(), 1, "bad harvest end page size");

    }

    #[test]
    fn get_owner_veggies_page_3(){
        let c = get_context(robert(), 0);
        testing_env!(c);
        let mut contract = PlantaryContract::new(robert());
        load_default_seeds(&mut contract);

        // mint 23  plants
        for _n in 0..22 {
            contract.mint_plant(vcats::MONEY);
        }
        let _p23 = contract.mint_plant(vcats::ORACLE);

        // check that we can get the whole thing in one big slurp
        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 23,0);
        assert_eq!(tokens.len(), 23, "bad plant total page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 0,0);
        assert_eq!(tokens.len(), 23, "bad plant total page size");

        let tokens = contract.get_owner_veggies_page(robert(), vtypes::PLANT, 100,0);
        assert_eq!(tokens.len(), 23, "bad plant total page size");

    }

    #[test]
    #[should_panic(
        expected = r#"Unknown veggie type 23."#
    )]
    fn get_owner_veggies_unknown(){
        testing_env!(get_context(robert(), 0));
        let contract = PlantaryContract::new(robert());
        // count_owner_veggies() should panic for any unknown types
        let _foo = contract.get_owner_veggies_page(robert(), 23, 1, 1); // panic!
    }

    // From here down I've just duplicated the unit tests in TokenBank.rs ,
    // to test our wrapper methods around that object.

    #[test]
    fn grant_access() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut tb = TokenBank::new();
        let length_before = tb.account_gives_access.len();
        assert_eq!(0, length_before, "Expected empty account access Map.");
        tb.grant_access(mike());
        tb.grant_access(joe());
        let length_after = tb.account_gives_access.len();
        assert_eq!(1, length_after, "Expected an entry in the account's access Map.");
        let predecessor_hash = env::sha256(robert().as_bytes());
        let num_grantees = tb.account_gives_access.get(&predecessor_hash).unwrap();
        assert_eq!(2, num_grantees.len(), "Expected two accounts to have access to predecessor.");
    }

    #[test]
    #[should_panic(
        expected = r#"Access does not exist."#
    )]
    fn revoke_access_and_panic() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut tb = TokenBank::new();
        tb.revoke_access(joe());
    }

    #[test]
    fn add_revoke_access_and_check() {
        // Joe grants access to Robert
        let mut context = get_context(joe(), 0);
        testing_env!(context);
        let mut tb = TokenBank::new();
        tb.grant_access(robert());

        // does Robert have access to Joe's account? Yes.
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        let mut robert_has_access = tb.check_access(&joe());
        assert_eq!(true, robert_has_access, "After granting access, check_access call failed.");

        // Joe revokes access from Robert
        context = get_context(joe(), env::storage_usage());
        testing_env!(context);
        tb.revoke_access(robert());

        // does Robert have access to Joe's account? No
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        robert_has_access = tb.check_access(&joe());
        assert_eq!(false, robert_has_access, "After revoking access, check_access call failed.");
    }

    #[test]
    fn mint_token_get_token_owner() {
        let context = get_context(robert(), 0);
        testing_env!(context);
        let mut tb = TokenBank::new();
        tb.mint_token(mike(), 19u64);
        let owner = tb.get_token_owner(19u64);
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
        let mut tb = TokenBank::new();
        let token_id = 19u64;
        tb.mint_token(mike(), token_id);
        tb.transfer_from(mike(), robert(), token_id.clone());
    }

    #[test]
    fn transfer_from_with_escrow_access() {
        // Escrow account: robert.testnet
        // Owner account: mike.testnet
        // New owner account: joe.testnet
        let mut context = get_context(mike(), 0);
        testing_env!(context);
        let mut tb = TokenBank::new();
        let token_id = 19u64;
        tb.mint_token(mike(), token_id);
        // Mike grants access to Robert
        tb.grant_access(robert());

        // Robert transfers the token to Joe
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        tb.transfer_from(mike(), joe(), token_id.clone());

        // Check new owner
        let owner = tb.get_token_owner(token_id.clone());
        assert_eq!(joe(), owner, "Token was not transferred after transfer call with escrow.");
    }

    #[test]
    #[should_panic(
        expected = r#"Attempt to transfer a token from wrong owner."#
    )]
    fn transfer_from_with_escrow_access_wrong_owner_id() {
        // Escrow account: robert.testnet
        // Owner account: mike.testnet
        // New owner account: joe.testnet
        let mut context = get_context(mike(), 0);
        testing_env!(context);
        let mut tb = TokenBank::new();
        let token_id = 19u64;
        tb.mint_token(mike(), token_id);
        // Mike grants access to Robert
        tb.grant_access(robert());

        // Robert transfers the token to Joe
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        tb.transfer_from(robert(), joe(), token_id.clone());
    }

    #[test]
    fn transfer_from_with_your_own_token() {
        // Owner account: robert.testnet
        // New owner account: joe.testnet

        testing_env!(get_context(robert(), 0));
        let mut tb = TokenBank::new();
        let token_id = 19u64;
        tb.mint_token(robert(), token_id);

        // Robert transfers the token to Joe
        tb.transfer_from(robert(), joe(), token_id.clone());

        // Check new owner
        let owner = tb.get_token_owner(token_id.clone());
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
        let mut tb = TokenBank::new();
        let token_id = 19u64;
        tb.mint_token(mike(), token_id);
        // Mike grants access to Robert
        tb.grant_access(robert());

        // Robert transfers the token to Joe
        context = get_context(robert(), env::storage_usage());
        testing_env!(context);
        tb.transfer(joe(), token_id.clone());
    }

    #[test]
    fn transfer_with_your_own_token() {
        // Owner account: robert.testnet
        // New owner account: joe.testnet

        testing_env!(get_context(robert(), 0));
        let mut tb = TokenBank::new();
        let token_id = 19u64;
        tb.mint_token(robert(), token_id);

        // Robert transfers the token to Joe
        tb.transfer(joe(), token_id.clone());

        // Check new owner
        let owner = tb.get_token_owner(token_id.clone());
        assert_eq!(joe(), owner, "Token was not transferred after transfer call with escrow.");
    }
}

