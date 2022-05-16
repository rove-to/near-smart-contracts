/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
 */
use std::collections::HashMap;

use near_contract_standards::non_fungible_token::{NonFungibleToken, refund_deposit_to_account};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_sdk::{AccountId, assert_one_yocto, Balance, BorshStorageKey, env, Gas,
               near_bindgen, PanicOnDefault, Promise, PromiseOrValue, PromiseResult, require};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::ext_contract;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

pub use crate::events::*;
use crate::internal::*;
pub use crate::royalty::*;
pub use crate::types::*;

mod events;
mod internal;
mod royalty;
mod types;

const ONE_HUNDRED_PERCENT_IN_BPS: u16 = 10_000;
pub const NFT_METADATA_SPEC: &str = "1.0.0";
pub const NFT_STANDARD_NAME: &str = "nep171";
pub const NOT_FOUND_METAVERSE_ID_ERROR: &str = "Not found metaverse_id";
pub const NOT_FOUND_ZONE_INDEX_ERROR: &str = "Not found zone_index";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    pub royalties: UnorderedMap<String, HashMap<AccountId, u16>>,
    pub tokens_metadata: UnorderedMap<String, TokenMetadata>,

    // Parameter control
    pub admin_id: AccountId,
    pub operator_id: AccountId,
    pub treasury_id: AccountId,

    pub init_imo_fee: u128, // fee in yoctoNEAR 1e-24 NEAR

    // Map metaverse_id => MetaverseMetadata
    pub metaverses: UnorderedMap<String, Metaverse>,
    // Map metaverse_id => account_id
    pub metaverse_owners: UnorderedMap<String, AccountId>,

    // Map metaverse_id => [token_id => true/false]
    pub tokens_minted: UnorderedMap<String, HashMap<String, bool>>,

    // Map metaverse_id => [token_id => true]
    pub nft_checker: UnorderedMap<String, HashMap<String, bool>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Zone {
    pub zone_index: u16,
    // required, start from 1
    pub price: u128,
    // required for type=3
    pub core_team_addr: String,
    // required for type=1
    pub collection_addr: String,
    // required for type=2
    pub type_zone: u8,
    // 1: core_team, 2: nft_holder, 3: public
    pub rock_index_from: u128,
    // rock_index start from 1
    pub rock_index_to: u128, // required to >= from
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Metaverse {
    // Map zone_index => Zone
    pub zones: UnorderedMap<u16, Zone>,
}

#[ext_contract(collection_contract)]
trait ExtContract {
    fn nft_tokens_for_owner(&self, account_id: AccountId, from_index: Option<near_sdk::json_types::U128>, limit: Option<u64>) -> Vec<Token>;
}

#[ext_contract(rock_nft_contract)]
pub trait RockNFTContract {
    fn mint_nft_checker_rock(&mut self, metaverse_id: String, zone_index: u16, rock_index: u128, receiver_id: AccountId, token_metadata: TokenMetadata);
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    TokensMetadata,
    TokensMinted,
    Royalties,
    Metaverses,
    MetaverseOwner,
    Zone,
    NftChecker,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        admin_id: AccountId,
        operator_id: AccountId,
        treasury_id: AccountId,
        init_imo_fee: U128, // fee in yoctoNEAR
        metadata: NFTContractMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let init_imo_fee_in_128 = u128::from(init_imo_fee);

        Self {
            admin_id: admin_id.into(),
            operator_id: operator_id.clone().into(),
            treasury_id: treasury_id.into(),
            init_imo_fee: init_imo_fee_in_128,

            royalties: UnorderedMap::new(StorageKey::Royalties),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            tokens_metadata: UnorderedMap::new(StorageKey::TokensMetadata),

            metaverses: UnorderedMap::new(StorageKey::Metaverses),
            metaverse_owners: UnorderedMap::new(StorageKey::MetaverseOwner),
            tokens_minted: UnorderedMap::new(StorageKey::TokensMinted),
            nft_checker: UnorderedMap::new(StorageKey::NftChecker),

            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                operator_id.clone().into(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
        }
    }

    fn assert_admin_only(&mut self) {
        // assert that the user attached greater than or equal 1 yoctoNEAR. This is for security and so that user will be redirected to the NEAR wallet
        assert_at_least_one_yocto();
        assert_eq!(env::predecessor_account_id(), self.admin_id, "Unauthorized");
    }

    fn assert_operator_only(&mut self) {
        // assert that the user attached greater than or equal 1 yoctoNEAR. This is for security and so that user will be redirected to the NEAR wallet
        assert_at_least_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Unauthorized"
        );
    }

    fn assert_metaverse_exist(&self, metaverse_id: &String) -> Metaverse {
        self.metaverses
            .get(&metaverse_id)
            .expect(NOT_FOUND_METAVERSE_ID_ERROR);

        self.metaverses
            .get(&metaverse_id).unwrap()
    }

    fn assert_zone_exist(&self, metaverse_id: &String, zone_index: u16) -> Zone {
        self.assert_metaverse_exist(metaverse_id);
        self.metaverses.get(metaverse_id).unwrap()
            .zones.get(&zone_index)
            .expect(NOT_FOUND_ZONE_INDEX_ERROR);

        self.metaverses.get(metaverse_id).unwrap()
            .zones.get(&zone_index).unwrap()
    }

    fn assert_metaverse_owner(&self, metaverse_id: &String) {
        // metaverse_owner will attach greater than or equal 1 yoctoNEAR. This is for security and so that user will be redirected to the NEAR wallet
        assert_at_least_one_yocto();
        self.assert_metaverse_exist(metaverse_id);
        let metaverse_owner = self.metaverse_owners.get(metaverse_id).expect(NOT_FOUND_METAVERSE_ID_ERROR);
        assert_eq!(env::predecessor_account_id(), metaverse_owner, "Unauthorized");
    }

    /// change contract's admin, only current contract's admin can call this function
    #[payable]
    pub fn change_admin(&mut self, new_admin_id: AccountId) {
        self.assert_admin_only();
        self.admin_id = new_admin_id.into();
    }

    /// change tokens.owner_id and operator_id to new_operator_id
    /// move all tokens of current operator to new operator
    #[payable]
    pub fn change_operator(&mut self, new_operator_id: AccountId) {
        self.assert_admin_only();

        self.tokens.owner_id = new_operator_id.clone();
        self.operator_id = new_operator_id.into();
    }

    #[payable]
    pub fn change_treasury(&mut self, new_treasury_id: AccountId) {
        self.assert_admin_only();
        self.treasury_id = new_treasury_id.into();
    }

    // Only operator can change init_imo_fee
    #[payable]
    pub fn change_init_imo_fee(&mut self, init_imo_fee: U128) {
        self.assert_operator_only();
        let init_imo_fee_in_128 = u128::from(init_imo_fee);
        self.init_imo_fee = init_imo_fee_in_128;
    }

    #[payable]
    pub fn change_nft_collection_rock_price(&mut self, metaverse_id: String, zone_index: u16, price: U128) {
        self.assert_metaverse_owner(&metaverse_id);
        let mut zone = self.assert_zone_exist(&metaverse_id, zone_index);
        assert_eq!(zone.type_zone, 2, "zone_index invalid");
        assert!(zone.rock_index_to > 0, "zone_index invalid");

        let mut metaverse_data = self
            .metaverses
            .get(&metaverse_id).unwrap();
        let price_in_u128 = u128::from(price);
        zone.price = price_in_u128;

        metaverse_data.zones.insert(&zone_index, &zone);
        self.metaverses.insert(&metaverse_id, &metaverse_data);
    }

    #[payable]
    pub fn update_royalties(
        &mut self,
        nft_type_id: String,
        updated_royalties: HashMap<AccountId, u16>,
    ) {
        self.assert_admin_only();
        let initial_storage_usage = env::storage_usage();
        self.royalties.insert(&nft_type_id, &updated_royalties);
        if env::storage_usage() > initial_storage_usage {
            refund_deposit_to_account(
                env::storage_usage() - initial_storage_usage,
                env::predecessor_account_id(),
            );
        }
    }

    pub fn get_admin(self) -> AccountId {
        self.admin_id
    }

    pub fn get_operator(self) -> AccountId {
        self.tokens.owner_id
    }

    pub fn get_treasury(self) -> AccountId {
        self.treasury_id
    }

    fn check_zone(&self, _zone: &Zone) -> bool {
        if _zone.type_zone < 1 && _zone.type_zone > 3 {
            return false;
        }

        if _zone.rock_index_to > 0 {
            if _zone.type_zone == 1 {
                if _zone.core_team_addr == "".to_string() {
                    return false;
                }
            } else if _zone.type_zone == 2 {
                if _zone.collection_addr == "".to_string() {
                    return false;
                }
            } else if _zone.type_zone == 3 {
                if _zone.price == 0 {
                    return false;
                }
            }

            if _zone.rock_index_from > _zone.rock_index_to || _zone.rock_index_from == 0 {
                return false;
            }

            return true;
        } else {
            return false;
        }
    }

    // user init metaverse
    // user pay storage fee
    #[payable]
    pub fn init_metaverse(
        &mut self,
        metaverse_id: String,
        zone1: Zone,
        zone2: Zone,
        zone3: Zone,
    ) {
        // Make sure metaverse_id does NOT exist
        let metaverse_data = self.metaverses.get(&metaverse_id);
        match metaverse_data {
            Some(_metaverse) => {
                env::panic_str("metaverse already existed");
            }
            _ => {}
        }

        if zone1.rock_index_to == 0 || !self.check_zone(&zone1) {
            env::panic_str("Z1_invalid")
        }

        if zone2.rock_index_to == 0 || !self.check_zone(&zone2) {
            env::panic_str("Z2_invalid")
        }

        if zone3.rock_index_to == 0 || !self.check_zone(&zone3) {
            env::panic_str("Z3_invalid")
        }

        if zone2.rock_index_from <= zone1.rock_index_to {
            env::panic_str("Z2_invalid")
        }
        if zone3.rock_index_from <= zone2.rock_index_to {
            env::panic_str("Z3_invalid")
        }

        let initial_storage_usage = env::storage_usage();
        let mut total_rock_size: u128 = 0;
        if zone1.rock_index_to > 0 && zone1.rock_index_to >= zone1.rock_index_from {
            total_rock_size = total_rock_size + (zone1.rock_index_to - zone1.rock_index_from);
        }

        if zone2.rock_index_to > 0 && zone2.rock_index_to >= zone2.rock_index_from {
            total_rock_size = total_rock_size + (zone2.rock_index_to - zone2.rock_index_from);
        }

        if zone3.rock_index_to > 0 && zone3.rock_index_to >= zone3.rock_index_from {
            total_rock_size = total_rock_size + (zone3.rock_index_to - zone3.rock_index_from);
        }

        let total_init_imo_fee = self.init_imo_fee * total_rock_size;
        let mut zones = UnorderedMap::new(StorageKey::Zone);
        zones.insert(&zone1.zone_index, &zone1);
        zones.insert(&zone2.zone_index, &zone2);
        zones.insert(&zone3.zone_index, &zone3);

        let metaverse = Metaverse { zones };
        self.metaverses.insert(&metaverse_id, &metaverse);
        self.metaverse_owners.insert(&metaverse_id, &env::predecessor_account_id());
        self.tokens_minted.insert(&metaverse_id, &HashMap::new());
        self.nft_checker.insert(&metaverse_id, &HashMap::new());

        let storage_used = env::storage_usage() - initial_storage_usage;
        let storage_cost = env::storage_byte_cost() * Balance::from(storage_used);
        let total_cost = total_init_imo_fee + storage_cost;
        let attached_deposit = env::attached_deposit();

        require!(
            total_cost <= attached_deposit,
            format!("Must attach {} yoctoNEAR to cover fee", total_cost)
        );

        let refund = attached_deposit - total_cost;
        if refund > 1 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    }

    pub fn mint_nft_checker_rock(&mut self, metaverse_id: String, zone_index: u16, rock_index: u128, receiver_id: AccountId, token_metadata: TokenMetadata) {
        assert_eq!(env::promise_results_count(), 1, "This is a callback method");
        match env::promise_result(0) {
            PromiseResult::NotReady => { env::panic_str("You need to have an NFT to be able to mint this rock"); }
            PromiseResult::Failed => { env::panic_str("You need to have an NFT to be able to mint this rock"); }
            PromiseResult::Successful(result) => {
                let tokens
                    = near_sdk::serde_json::from_slice::<Vec<Token>>(&result).unwrap();
                if tokens.len() == 0 {
                    env::panic_str("You need to have an NFT to be able to mint this rock")
                }

                let nft_checker = self.nft_checker.get(&metaverse_id).unwrap();
                let mut mintable = false;
                let mut use_token_id: TokenId = "".parse().unwrap();
                for token in tokens {
                    let _token_id = token.token_id;
                    let checker = nft_checker.get(&_token_id.to_string());

                    match checker {
                        Some(_existed) => {} // Skip if that token used
                        None => {
                            mintable = true;
                            use_token_id = _token_id;
                            break;
                        }
                    }
                }
                if !mintable {
                    env::panic_str("You need to have an NFT to be able to mint this rock")
                }
                let zone = self.assert_zone_exist(&metaverse_id, zone_index);
                let token_id = gen_token_id(&metaverse_id, zone_index, rock_index);
                self._mint(
                  metaverse_id.clone(),
                    token_id.clone(),
                    receiver_id.clone(),
                    token_metadata.clone(),
                    zone.price,
                    zone.type_zone,
                    use_token_id.to_string().clone(),
                );
            }
        };
    }

    fn _mint(&mut self, metaverse_id: String, token_id: String, receiver_id: AccountId, token_metadata: TokenMetadata, token_price: u128,
             type_zone: u8, use_token_id: String) {
        let initial_storage_usage = env::storage_usage();
        let token = self.tokens.internal_mint_with_refund(
            token_id.clone(),
            receiver_id.clone(),
            Some(token_metadata.clone()),
            None,
        );

        let mut token_minted = self.tokens_minted.get(&metaverse_id).unwrap();
        token_minted.insert(token.token_id.to_string(), true);
        self.tokens_minted.insert(&metaverse_id, &token_minted);

        if type_zone == 2 {
            let mut nft_checker = self.nft_checker.get(&metaverse_id).unwrap();
            nft_checker.insert(use_token_id, true);
            self.nft_checker.insert(&metaverse_id, &nft_checker);
        }

        let attached_deposit = env::attached_deposit();
        let storage_used = env::storage_usage() - initial_storage_usage;
        let required_storage_cost = env::storage_byte_cost() * Balance::from(storage_used);
        let total_cost = required_storage_cost + token_price;
        require!(
                    total_cost <= attached_deposit,
                    format!("Must attach {} yoctoNEAR to cover fee", total_cost)
                );

        let metaverse_owner = self.metaverse_owners.get(&metaverse_id).unwrap();
        if token_price > 0 {
            Promise::new(metaverse_owner).transfer(token_price);
        }

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: token.token_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());
    }

    pub fn get_zone_info(&self, metaverse_id: String, zone_index: u16) -> String {
        let zone = self.assert_zone_exist(&metaverse_id, zone_index);
        format!("{}, {}, {}, {}, {}, {}, {}", zone.zone_index, zone.type_zone, zone.core_team_addr, zone.collection_addr,
               zone.price, zone.rock_index_from, zone.rock_index_to)
    }

    #[payable]
    pub fn mint_rock(
        &mut self,
        metaverse_id: String,
        zone_index: u16,
        rock_index: u128,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ) {
        let zone = self.assert_zone_exist(&metaverse_id, zone_index);
        assert!(zone.rock_index_from > 0 && zone.rock_index_to > 0, "zone rock index invalid");
        assert!(zone.rock_index_from <= rock_index && rock_index <= zone.rock_index_to, "rock_index invalid");
        let token_id = gen_token_id(&metaverse_id, zone_index, rock_index);
        let token_minted = self.tokens_minted.get(&metaverse_id);
        match token_minted {
            Some(_token_minted) => {
                if *_token_minted.get(&token_id).unwrap() {
                    env::panic_str("token_id is existed")
                }
            }
            _ => {}
        }

        let signer_id = env::signer_account_id();
        if zone.type_zone == 1 {
            assert_eq!(zone.core_team_addr, env::predecessor_account_id().to_string(), "require core team call this mint");
        } else if zone.type_zone == 2 {
            // NFT checker
            assert_ne!(zone.collection_addr, "".to_string(), "collection addr is empty");
            let collect_contract_account_id: AccountId = zone.collection_addr.parse().unwrap();
            collection_contract::nft_tokens_for_owner(
                signer_id,
                None,
                None,
                collect_contract_account_id,
                0,
                Gas(5_000_000_000_000))
                .then(rock_nft_contract::mint_nft_checker_rock(metaverse_id.clone(),
                                                               zone_index,
                                                               rock_index,
                                                               receiver_id.clone(),
                                                               token_metadata.clone(),
                                                               env::signer_account_id(),
                                                               0,
                                                               Gas(5_000_000_000_000)));
        } else if zone.type_zone == 3 {
           if zone.price <= 0 {
               env::panic_str("missing price for public zone");
           }
        } else {
            env::panic_str("does not support zone");
        }
        let mut price = zone.price;
        if zone.type_zone == 1 {
            price = 0;
        }

        if zone.type_zone != 2 {
            self._mint(
              metaverse_id.clone(),
                token_id.clone(),
                receiver_id.clone(),
                token_metadata.clone(),
              price,
                zone.type_zone,
                "".to_string(),
            );
        }
    }

    #[payable]
    pub fn update_contract_metadata(&mut self, updated_contract_metadata: NFTContractMetadata) {
        self.assert_operator_only();
        self.metadata.set(&updated_contract_metadata);
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}
