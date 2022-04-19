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
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{NonFungibleToken, refund_deposit_to_account};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::{assert_one_yocto, env, log, near_bindgen, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, Balance, AccountId};

use crate::internal::*;
pub use crate::types::*;
pub use crate::royalty::*;
pub use crate::events::*;

mod internal;
mod types;
mod royalty;
mod events;

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
    pub max_supply: u64,
    pub royalties: UnorderedMap<AccountId, u16>,

    // tokens that can be minted by calling user_mint method
    // always belongs to the set of tokens of operator
    pub user_mintable_tokens: UnorderedSet<TokenId>,

    // keep track of token's price after created
    pub token_prices: LookupMap<TokenId, u128>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    TokenPrice,
    UserMintableToken,
    Royalties
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
        init_royalties : Option<HashMap<AccountId, u16>>
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
            max_supply,
            user_mintable_tokens: UnorderedSet::new(StorageKey::UserMintableToken),
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                operator_id.clone().into(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            token_prices: LookupMap::new(StorageKey::TokenPrice),
            royalties
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

        let user_mintable_tokens_in_vec = self.user_mintable_tokens.to_vec();
        for token_id in &user_mintable_tokens_in_vec {
            self.tokens.internal_transfer_unguarded(
                token_id,
                &self.operator_id.clone(),
                &new_operator_id.clone(),
            );
        }

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

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. ` self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the ` owner_id` given in
    /// initialization call to `new`
    #[payable]
    pub fn nft_mint_batch(
        &mut self,
        init_supply: u64,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
        price_in_string: String,
    ) -> Vec<Token> {
        self.assert_operator_only();
        assert_eq!(self.tokens.owner_by_id.len(), 0, "TOKENS CREATED");

        let initial_storage_usage = env::storage_usage();

        let price: u128;
        match u128::from_str_radix(&price_in_string, 10) {
            Ok(val) => {
                price = val;
            }
            Err(_e) => {
                env::panic_str("error when parse price_in_string to u128");
            }
        }

        // vector of created tokens
        let mut tokens: Vec<Token> = Vec::new();
        let mut token_ids: Vec<String> = Vec::new();

        for i in 0..init_supply {
            let token_id: TokenId = i.to_string();
            let token = self.tokens.internal_mint_with_refund(
                token_id.clone(),
                receiver_id.clone().into(),
                Some(token_metadata.clone()),
                None
            );
            self.token_prices.insert(&token_id, &price);
            tokens.push(token);
        }
        let operator_id = self.operator_id.clone();
        for i in init_supply..self.max_supply {
            let token_id: TokenId = i.to_string();
            let token = self.tokens.internal_mint_with_refund(
                token_id.clone(),
                operator_id.clone().try_into().unwrap(),
                Some(token_metadata.clone()),
                None
            );
            self.token_prices.insert(&token_id, &price);
            self.user_mintable_tokens.insert(&token_id);

            token_ids.push(token.token_id.to_string());
            tokens.push(token);
        }

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),

            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: operator_id.to_string(),
                token_ids,
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        let storage_used = env::storage_usage() - initial_storage_usage;
        refund_deposit_to_account(storage_used, env::predecessor_account_id());

        tokens
    }

    #[payable]
    pub fn nft_user_mint(&mut self, receiver_id: AccountId) {
        assert!(
            self.user_mintable_tokens.len() > 0,
            "mintable tokens is empty now"
        );
        let token_id = self
            .user_mintable_tokens
            .as_vector()
            .get(0)
            .expect("Unable to access user_mintable_tokens's first token");
        let token_price = self
            .token_prices
            .get(&token_id)
            .expect("Can't find price of token");

        // make sure deposit money >= token price
        assert!(env::attached_deposit() >= token_price);
        self.tokens
            .internal_transfer_unguarded(&token_id, &self.operator_id, &receiver_id);

        self.user_mintable_tokens.remove(&token_id);

        Promise::new(self.treasury_id.clone()).transfer(env::attached_deposit());

        // Construct the transfer log as per the events standard.
        let nft_transfer_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                authorized_id: None,
                old_owner_id: self.operator_id.to_string(),
                new_owner_id: receiver_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_transfer_log.to_string());

        log!(
            "Transfer {} from {} to {}",
            token_id,
            &self.operator_id,
            receiver_id
        );
    }

    #[payable]
    pub fn update_token_metadata(
        &mut self,
        token_id: TokenId,
        update_token_metadata: TokenMetadata,
    ) {
        self.assert_operator_only();
        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.insert(&token_id, &update_token_metadata);
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::test_utils::{accounts, VMContextBuilder};
//     use near_sdk::testing_env;
//
//     const NFT_CONTRACT_ID : string = "nft-contract.testnet";
//
//     fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .predecessor_account_id(predecessor_account_id);
//         builder
//     }
// }
