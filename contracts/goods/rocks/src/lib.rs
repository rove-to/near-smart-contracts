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
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{refund_deposit_to_account, NonFungibleToken};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise, PromiseOrValue,
};
use std::collections::HashMap;

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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,

    pub admin_id: AccountId,
    pub operator_id: AccountId,
    pub treasury_id: AccountId,

    // token_id => [account_id: percent]
    pub royalties: UnorderedMap<String, HashMap<AccountId, u16>>,

    // Map metaverse_id => MetaverseMetadata
    pub metaverses: UnorderedMap<String, MetaverseMetadata>,
    pub tokens_metadata: UnorderedMap<String, TokenMetadata>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MetaverseMetadata {
    pub external_nft_contract: String,
    pub amount_center_rock: u32,
    pub amount_public_rock: u32,
    pub price_center_rock: u128,
    pub price_public_rock: u128,

    // for tracking
    pub minted_center_rock: u32,
    pub minted_public_rock: u32,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    MaxSupplies,
    TokensPrice,
    TokensMetadata,
    TokensMinted,
    Royalties,
    Metaverses,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        admin_id: AccountId,
        operator_id: AccountId,
        treasury_id: AccountId,
        metadata: NFTContractMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();

        Self {
            admin_id: admin_id.into(),
            operator_id: operator_id.clone().into(),
            treasury_id: treasury_id.into(),
            royalties: UnorderedMap::new(StorageKey::Royalties),

            tokens_metadata: UnorderedMap::new(StorageKey::TokensMetadata),

            metaverses: UnorderedMap::new(StorageKey::Metaverses),

            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                operator_id.clone().into(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
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

    #[result_serializer(borsh)]
    pub fn get_metaverse_data(self,  metaverse_id: String) -> MetaverseMetadata {
        self.metaverses.get(&metaverse_id).expect(NOT_FOUND_METAVERSE_ID_ERROR)
    }

    pub fn get_operator(self) -> AccountId {
        self.tokens.owner_id
    }

    pub fn get_treasury(self) -> AccountId {
        self.treasury_id
    }

    // user init metaverse
    // user pay storage fee
    #[payable]
    pub fn init_metaverse(
        &mut self,
        metaverse_id: String,
        external_nft_contract: String,
        amount_center_rock: u32,
        amount_public_rock: u32,
        price_center_rock: U128, // U128 tuc la truyen len dang string
        price_public_rock: U128,
    ) {
        let metaverse_data = self.metaverses.get(&metaverse_id);
        match metaverse_data {
            Some(MetaverseMetadata) => {
                panic!("metaverse already inited")
            }
            _ => {}
        }

        let initial_storage_usage = env::storage_usage();

        let price_center_rock_u128 = u128::from(price_center_rock);
        let price_public_rock_u128 = u128::from(price_public_rock);

        self.metaverses.insert(
            &metaverse_id,
            &MetaverseMetadata {
                external_nft_contract,
                amount_center_rock,
                amount_public_rock,
                price_center_rock: price_center_rock_u128,
                price_public_rock: price_public_rock_u128,

                minted_center_rock: 0,
                minted_public_rock: 0,
            },
        );

        let storage_used = env::storage_usage() - initial_storage_usage;
        let required_storage_cost = env::storage_byte_cost() * Balance::from(storage_used);

        // thường là sẽ refund lại phí user trả thừa env::attached_deposit() - required_storage_cost
        require!(
            env::attached_deposit() >= required_storage_cost,
            "NOT ATTACHING ENOUGH DEPOSIT"
        );
    }

    #[payable]
    pub fn user_mint(
        &mut self,
        metaverse_id: String,
        rock_id: String,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        let initial_storage_usage = env::storage_usage();

        let mut metaverse_data = self
            .metaverses
            .get(&metaverse_id)
            .expect(NOT_FOUND_METAVERSE_ID_ERROR);

        require!(
            metaverse_data.minted_center_rock + metaverse_data.minted_public_rock
                < metaverse_data.amount_center_rock + metaverse_data.amount_public_rock,
            "REACH MAX SUPPLY"
        );

        let mut price: u128 = 0;
        let mut is_operator_mint = false;
        let mut mint_center_rock = false;

        if env::predecessor_account_id() == self.operator_id {
            self.assert_operator_only();
            is_operator_mint = true;
        }

        if is_operator_mint {
            price = 0;
        } else {
            if metaverse_data.minted_center_rock < metaverse_data.amount_center_rock {
                mint_center_rock = true;
                price = metaverse_data.price_center_rock;
            } else {
                price = metaverse_data.price_public_rock;
            }
        };

        // TODO
        if mint_center_rock {
            // Check env::predecessor_account_id() có sỡ hữu 1 item từ external contract ko
            // Nếu ko thì panic
            // confirm lai voi a ThaiBao

        }

        let token_id = gen_token_id(&metaverse_id, &rock_id);

        let token = self.tokens.internal_mint_with_refund(
            token_id.clone(),
            receiver_id.clone(),
            Some(token_metadata.clone()),
            None,
        );

        if mint_center_rock {
            metaverse_data.minted_center_rock = metaverse_data.minted_center_rock + 1;
        } else {
            metaverse_data.minted_public_rock = metaverse_data.minted_public_rock + 1;
        }

        let storage_used = env::storage_usage() - initial_storage_usage;
        let required_storage_cost = env::storage_byte_cost() * Balance::from(storage_used);

        // user pay only for token price
        require!(
            env::attached_deposit() >= price,
            "NOT ATTACHING ENOUGH DEPOSIT"
        );

        // transfer to treasury: token_price - storage_cost
        if !is_operator_mint && env::attached_deposit() > required_storage_cost {
            Promise::new(self.treasury_id.clone())
                .transfer(env::attached_deposit() - required_storage_cost);
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

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        token
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
