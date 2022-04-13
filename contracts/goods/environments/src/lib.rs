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
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::refund_deposit;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, log
};
use near_sdk::env::attached_deposit;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,

    // keep track of token's price after created
    pub token_prices: LookupMap<TokenId, u128>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    TokenPrice,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        Self::new(owner_id,
                  NFTContractMetadata {
                      spec: NFT_METADATA_SPEC.to_string(),
                      name: "Rove Environments NFT contract".to_string(),
                      symbol: "ROVE_ENV".to_string(),
                      icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                      base_uri: None,
                      reference: None,
                      reference_hash: None
                  })
    }

    #[init]
    pub fn new(owner_id : ValidAccountId, metadata : NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            token_prices: LookupMap::new(StorageKey::TokenPrice.try_to_vec().unwrap())
        }
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
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: ValidAccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        let token = self.tokens.mint(token_id, receiver_id, Some(token_metadata));
        token
    }

    #[payable]
    pub fn create_nft(&mut self, token_id: TokenId, receiver_id: ValidAccountId, token_metadata: TokenMetadata, price_in_string: String) -> Token {
        env::log(price_in_string.as_bytes());
        let mut price : u128 = 0;
        match u128::from_str_radix(&price_in_string, 10) {
            Ok(val) => {
                price = val;
            },
            Err(e) => {

            }
        }
        let token = self.tokens.mint(token_id.clone(), receiver_id, Some(token_metadata));
        self.token_prices.insert(&token_id, &price);
        token
    }

    #[payable]
    pub fn user_mint(&mut self, receiver_id : ValidAccountId) {
        let owner_id = self.tokens.owner_id.clone();
        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut owner_tokens = tokens_per_owner
                .get(&(self.tokens.owner_id))
                .expect("Unable to access owner's tokens in user mint call.");
            assert!(owner_tokens.len() > 0, "owner's tokens is empty now");
            let token_id = owner_tokens.as_vector().get(0).expect("Unable to access owner's first token");
            let token_price = self.token_prices.get(&token_id).expect("Can't find price of token");

            // make sure deposit money >= token price
            assert!(env::attached_deposit() >= token_price);
            let sender_id = env::predecessor_account_id();
            self.tokens.internal_transfer_unguarded(&token_id, &owner_id, &receiver_id.as_ref());

            log!("Transfer {} from {} to {}", token_id, sender_id, receiver_id);
        } else {
            assert!(false, "tokens_per_owner is null");
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


