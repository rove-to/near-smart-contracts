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
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, log, assert_one_yocto};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,

    pub admin_id : AccountId,

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
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(admin_id: ValidAccountId, operator_id : ValidAccountId, metadata : NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            admin_id: admin_id.into(),
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                operator_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            token_prices: LookupMap::new(StorageKey::TokenPrice.try_to_vec().unwrap())
        }
    }

    fn assert_admin_only(&mut self) {
        // assert that the user attached exactly 1 yoctoNEAR. This is for security and so that user will be redirected to the NEAR wallet
        assert_one_yocto();
        assert_eq!(env::predecessor_account_id(), self.admin_id, "Unauthorized");
    }

    fn assert_operator_only(&mut self) {
        // assert that the user attached exactly 1 yoctoNEAR. This is for security and so that user will be redirected to the NEAR wallet
        assert_one_yocto();
        assert_eq!(env::predecessor_account_id(), self.tokens.owner_id, "Unauthorized");
    }

    /// change contract's admin, only current contract's admin can call this function
    #[payable]
    pub fn change_admin(&mut self, new_admin_id: ValidAccountId) {
        self.assert_admin_only();
        self.admin_id = new_admin_id.into();
    }

    #[payable]
    pub fn change_operator(&mut self, new_operator_id: ValidAccountId) {
        self.assert_admin_only();
        self.tokens.owner_id = new_operator_id.into();
    }

    pub fn get_admin(self) -> AccountId {
        self.admin_id
    }

    pub fn get_operator(self) -> AccountId {
        self.tokens.owner_id
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
    pub fn create_nft(&mut self, token_id: TokenId, receiver_id: ValidAccountId, token_metadata: TokenMetadata, price_in_string: String) -> Token {
        // check owner id
        env::log(price_in_string.as_bytes());
        let price : u128;
        match u128::from_str_radix(&price_in_string, 10) {
            Ok(val) => {
                price = val;
            },
            Err(_e) => {
                env::panic(b"error when parse price_in_string to u128");
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
            let owner_tokens = tokens_per_owner
                .get(&(self.tokens.owner_id))
                .expect("Unable to access owner's tokens in user mint call.");
            assert!(owner_tokens.len() > 0, "owner's tokens is empty now");
            let token_id = owner_tokens.as_vector().get(0).expect("Unable to access owner's first token");
            let token_price = self.token_prices.get(&token_id).expect("Can't find price of token");

            // make sure deposit money >= token price
            assert!(env::attached_deposit() >= token_price);
            self.tokens.internal_transfer_unguarded(&token_id, &owner_id, &receiver_id.as_ref());

            log!("Transfer {} from {} to {}", token_id, owner_id, receiver_id);
        } else {
            env::panic(b"tokens_per_owner is null");
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


