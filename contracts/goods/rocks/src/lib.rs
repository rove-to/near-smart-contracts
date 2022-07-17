/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - This is NFT contract for Public Zone Rocks (type_zone = 3) only Metaverse
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

use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{
    refund_deposit_to_account, NonFungibleToken, Token, TokenId,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue,
};

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

    pub init_imo_fee: u128,     // fee in yoctoNEAR 1e-24 NEAR
    pub rock_purchase_fee: u32, // in percent, with 0.01% = 1 = rock_purchase_fee

    // Map metaverse_id => Metaverse
    pub metaverses: UnorderedMap<String, Metaverse>,
    // Map metaverse_id => account_id
    pub metaverse_owners: UnorderedMap<String, AccountId>,

    // Map metaverse_id => [token_id => true/false]
    pub tokens_minted: UnorderedMap<String, HashMap<String, bool>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Zone {
    pub zone_index: u16,         // required, start from 1
    pub price: U128,             // required
    pub core_team_addr: String,  // required for type=1
    pub collection_addr: String, // required for type=2
    pub type_zone: u8,           // 1: core_team, 2: nft_holder, 3: public
    pub rock_index_from: u128,   // rock_index start from 1
    pub rock_index_to: u128,     // required to >= from
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Metaverse {
    // Map zone_index => Zone
    pub zones: HashMap<u16, Zone>,
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
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        admin_id: AccountId,
        operator_id: AccountId,
        treasury_id: AccountId,
        init_imo_fee: U128,     // fee in yoctoNEAR
        rock_purchase_fee: u32, // 1 = 0.01% = 0.0001
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
            rock_purchase_fee,

            royalties: UnorderedMap::new(StorageKey::Royalties),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            tokens_metadata: UnorderedMap::new(StorageKey::TokensMetadata),

            metaverses: UnorderedMap::new(StorageKey::Metaverses),
            metaverse_owners: UnorderedMap::new(StorageKey::MetaverseOwner),
            tokens_minted: UnorderedMap::new(StorageKey::TokensMinted),

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

        self.metaverses.get(&metaverse_id).unwrap()
    }

    fn assert_zone_exist(&self, metaverse_id: &String, zone_index: u16) -> Zone {
        self.assert_metaverse_exist(metaverse_id);
        self.metaverses
            .get(metaverse_id)
            .unwrap()
            .zones
            .get(&zone_index)
            .expect(NOT_FOUND_ZONE_INDEX_ERROR);

        let zone = self
            .metaverses
            .get(metaverse_id)
            .unwrap()
            .zones
            .get(&zone_index)
            .unwrap()
            .clone();
        return zone;
    }

    fn assert_metaverse_owner(&self, metaverse_id: &String) {
        // metaverse_owner will attach greater than or equal 1 yoctoNEAR. This is for security and so that user will be redirected to the NEAR wallet
        assert_at_least_one_yocto();
        self.assert_metaverse_exist(metaverse_id);
        let metaverse_owner = self
            .metaverse_owners
            .get(metaverse_id)
            .expect(NOT_FOUND_METAVERSE_ID_ERROR);
        assert_eq!(
            env::predecessor_account_id(),
            metaverse_owner,
            "Unauthorized"
        );
    }

    #[payable]
    pub fn change_rock_purchase_fee(&mut self, rock_purchase_fee: u32) {
        self.assert_operator_only();
        self.rock_purchase_fee = rock_purchase_fee;
    }

    /// change contract's admin, only current contract's admin can call this function
    #[payable]
    pub fn change_admin(&mut self, new_admin_id: AccountId) {
        self.assert_admin_only();
        self.admin_id = new_admin_id.into();
    }

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
        let zone_price = u128::from(_zone.price);
        if _zone.type_zone != 3 {
            return false;
        }
        if _zone.rock_index_to == 0 {
            return false;
        }

        if zone_price == 0 {
            return false;
        }

        if _zone.rock_index_from > _zone.rock_index_to {
            return false;
        }

        true
    }

    // user init metaverse
    // user pay storage fee
    #[payable]
    pub fn init_metaverse(&mut self, metaverse_id: String, zone3: Zone) {
        // Make sure metaverse_id does NOT exist
        let metaverse_data = self.metaverses.get(&metaverse_id);
        match metaverse_data {
            Some(_metaverse) => {
                env::panic_str("metaverse is already existed");
            }
            _ => {}
        }
        require!(zone3.zone_index == 3, "zone_index must == 3");
        require!(zone3.type_zone == 3, "must be public zone");
        // rock index = 1 for rove team
        require!(zone3.rock_index_from == 2, "rock_index_from must = 2");

        if zone3.rock_index_to < 2 || !self.check_zone(&zone3) {
            env::panic_str("Z3_invalid")
        }

        let initial_storage_usage = env::storage_usage();
        let total_rock_size: u128 = zone3.rock_index_to - zone3.rock_index_from + 1;
        require!(total_rock_size > 0, "total_rock_size is invalid");

        let total_init_imo_fee = self.init_imo_fee * total_rock_size;
        let attached_deposit = env::attached_deposit();
        require!(
            total_init_imo_fee <= attached_deposit,
            format!(
                "Need {} yoctoNEAR to init metaverse with {} rocks ({} yoctoNEAR per rock)",
                total_init_imo_fee, total_rock_size, self.init_imo_fee,
            )
        );
        let refund = attached_deposit - total_init_imo_fee;

        let mut zones = HashMap::new();
        zones.insert(zone3.zone_index, zone3);

        // center rock is for Rover (operator)
        let _zone1: Zone = Zone {
            zone_index: 1,
            price: U128(0),
            core_team_addr: self.operator_id.to_string(),
            collection_addr: "".to_string(),
            type_zone: 1,
            rock_index_from: 1,
            rock_index_to: 1,
        };
        zones.insert(_zone1.zone_index, _zone1);

        let metaverse = Metaverse { zones };
        self.metaverses.insert(&metaverse_id, &metaverse);
        self.metaverse_owners
            .insert(&metaverse_id, &env::signer_account_id());
        self.tokens_minted.insert(&metaverse_id, &HashMap::new());

        let storage_used = env::storage_usage() - initial_storage_usage;
        let storage_cost = env::storage_byte_cost() * Balance::from(storage_used);

        if refund > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }

        if total_init_imo_fee > storage_cost {
            let remain = total_init_imo_fee - storage_cost;
            if remain > 0 {
                Promise::new(self.treasury_id.clone()).transfer(remain);
            }
        }

        let init_metaverse_log: EventLog = EventLog {
            standard: "public_imo_init".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::IMOInit(vec![IMOInitLog {
                metaverse_id,
                owner_id: env::signer_account_id().to_string(),
                rock_size: total_rock_size,
                memo: None,
            }]),
        };

        env::log_str(&init_metaverse_log.to_string());
    }

    fn _mint(
        &mut self,
        metaverse_id: String,
        token_id: String,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
        token_price_str: U128,
    ) {
        let initial_storage_usage = env::storage_usage();
        let token_price = u128::from(token_price_str);
        let token = self.tokens.internal_mint_with_refund(
            token_id.clone(),
            receiver_id.clone(),
            Some(token_metadata.clone()),
            None,
        );

        let mut token_minted = self.tokens_minted.get(&metaverse_id).unwrap();
        token_minted.insert(token.token_id.to_string(), true);
        self.tokens_minted.insert(&metaverse_id, &token_minted);

        let attached_deposit = env::attached_deposit();
        require!(
            token_price <= attached_deposit,
            format!("Need {} yoctoNEAR to mint this rock", token_price)
        );
        let refund = attached_deposit - token_price;
        /*
        if token_price == 0 => contract account will pay storage cost
         */
        if token_price > 0 {
            let storage_used = env::storage_usage() - initial_storage_usage;
            let required_storage_cost = env::storage_byte_cost() * Balance::from(storage_used);
            let remain = token_price - required_storage_cost;
            if remain > 0 {
                if self.rock_purchase_fee > 0 {
                    let treasury_amount = remain * self.rock_purchase_fee as u128 / 10_000;
                    let metaverse_owner_amount = remain - treasury_amount;
                    if treasury_amount > 0 {
                        Promise::new(self.treasury_id.clone()).transfer(treasury_amount);
                    }
                    if metaverse_owner_amount > 0 {
                        let metaverse_owner = self.metaverse_owners.get(&metaverse_id).unwrap();
                        Promise::new(metaverse_owner).transfer(metaverse_owner_amount);
                    }
                }
            }
        }

        if refund > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: receiver_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo: None,
            }]),
        };

        env::log_str(&nft_mint_log.to_string());
    }

    pub fn get_zone_info(&self, metaverse_id: String, zone_index: u16) -> String {
        let zone = self.assert_zone_exist(&metaverse_id, zone_index);
        format!(
            "{}, {}, {}, {}, {:?}, {}, {}",
            zone.zone_index,
            zone.type_zone,
            zone.core_team_addr,
            zone.collection_addr,
            zone.price,
            zone.rock_index_from,
            zone.rock_index_to
        )
    }

    pub fn get_init_imo_fee(&self) -> U128 {
        return U128::from(self.init_imo_fee);
    }

    #[payable]
    pub fn update_init_imo_fee(&mut self, init_imo_fee: U128) {
        self.assert_operator_only();
        let init_imo_fee_u128 = u128::from(init_imo_fee);
        self.init_imo_fee = init_imo_fee_u128;
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
        assert!(
            zone.rock_index_from > 0 && zone.rock_index_to > 0,
            "zone rock index invalid"
        );
        assert!(
            zone.rock_index_from <= rock_index && rock_index <= zone.rock_index_to,
            "rock_index invalid"
        );
        let token_id = gen_token_id(&metaverse_id, zone_index, rock_index);
        let tokens_minted = self.tokens_minted.get(&metaverse_id).unwrap();
        let tokens_minted_checker = tokens_minted.get(&token_id);
        match tokens_minted_checker {
            Some(_token_minted) => env::panic_str("token_id is existed"),
            _ => {}
        }

        if zone.type_zone == 1 {
            assert_eq!(
                zone.core_team_addr,
                env::predecessor_account_id().to_string(),
                "require core team call this mint"
            );
        } else if zone.type_zone == 3 {
            let zone_price = u128::from(zone.price);
            if zone_price <= 0 {
                env::panic_str("missing price for public zone");
            }
        } else {
            env::panic_str("does not support zone");
        }

        self._mint(
            metaverse_id.clone(),
            token_id.clone(),
            receiver_id.clone(),
            token_metadata.clone(),
            zone.price,
        );
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
