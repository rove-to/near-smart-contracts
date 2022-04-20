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
use near_contract_standards::non_fungible_token::{NonFungibleToken};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
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

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,

    pub admin_id: AccountId,
    pub operator_id: AccountId,
    pub treasury_id: AccountId,
    pub royalties: UnorderedMap<AccountId, u16>,

    pub max_supply: u64,
    pub token_price: u128,
    pub token_metadata: TokenMetadata,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    Royalties,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        admin_id: AccountId,
        operator_id: AccountId,
        treasury_id: AccountId,
        max_supply: u64,
        metadata: NFTContractMetadata,
        token_price: u128,
        token_metadata: TokenMetadata,
        init_royalties: Option<HashMap<AccountId, u16>>,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut royalties = UnorderedMap::new(StorageKey::Royalties);
        if let Some(init_royalties) = init_royalties {
            for (account, amount) in init_royalties {
                royalties.insert(&account, &amount);
            }
        }
        Self {
            admin_id: admin_id.into(),
            operator_id: operator_id.clone().into(),
            treasury_id: treasury_id.into(),
            royalties,
            max_supply,
            token_price,
            token_metadata,
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

    pub fn get_admin(self) -> AccountId {
        self.admin_id
    }

    pub fn get_operator(self) -> AccountId {
        self.tokens.owner_id
    }

    pub fn get_treasury(self) -> AccountId {
        self.treasury_id
    }

    #[payable]
    pub fn nft_create(&mut self, receiver_id: AccountId) -> Token {
        require!(
            self.tokens.owner_by_id.len() < self.max_supply,
            "REACH MAX SUPPLY"
        );
        let mut is_operator_mint = false;
        if env::predecessor_account_id() == self.operator_id {
            self.assert_operator_only();
            is_operator_mint = true;
        }

        let initial_storage_usage = env::storage_usage();

        let price: u128 = if is_operator_mint {
            0
        } else {
            self.token_price
        };

        let token_id = self.tokens.owner_by_id.len().to_string();
        let token = self.tokens.internal_mint_with_refund(
            token_id.clone(),
            receiver_id.clone(),
            Some(self.token_metadata.clone()),
            None,
        );

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

        let storage_used = env::storage_usage() - initial_storage_usage;
        let required_storage_cost = env::storage_byte_cost() * Balance::from(storage_used);

        require!(
            env::attached_deposit() >= required_storage_cost + price,
            "NOT ATTACHING ENOUGH DEPOSIT"
        );

        if !is_operator_mint {
            Promise::new(self.treasury_id.clone())
                .transfer(env::attached_deposit() - required_storage_cost);
        }

        token
    }

    #[payable]
    pub fn update_token_price(&mut self, updated_price: u128) {
        self.assert_operator_only();
        self.token_price = updated_price;
    }

    // update default token_metadata
    #[payable]
    pub fn update_token_metadata(&mut self, updated_token_metadata: TokenMetadata) {
        self.assert_operator_only();
        self.token_metadata = updated_token_metadata;
    }

    // update token_metadata of a minted token
    #[payable]
    pub fn update_minted_token_metadata(
        &mut self,
        token_id: TokenId,
        updated_token_metadata: TokenMetadata,
    ) {
        self.assert_operator_only();
        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.insert(&token_id, &updated_token_metadata);
        } else {
            env::panic_str("token_metadata_by_id is null");
        }
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
