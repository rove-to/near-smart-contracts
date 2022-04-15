use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_sdk::json_types::U128;
use crate::*;

pub trait NonFungibleTokenRoyalty {
    //calculates the payout for a token given the passed in balance. This is a view method
    fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout;

    //transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance.
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[near_bindgen]
impl NonFungibleTokenRoyalty for Contract {
    //calculates the payout for a token given the passed in balance. This is a view method
    fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout {
        let token_owner_id = self.tokens.owner_by_id.get(&token_id).expect("token not exist");
        //keep track of the total perpetual royalties
        let mut total_perpetual : u16 = 0;
        //get the u128 version of the passed in balance (which was U128 before)
        let balance_u128 = u128::from(balance);
        //keep track of the payout object to send back
        let mut payout_object = Payout {
            payout: HashMap::new()
        };
        //get the royalty object from token
        //make sure we're not paying out to too many people (GAS limits this)
        assert!(self.royalties.len() as u32 <= max_len_payout, "Market cannot payout to that many receivers");

        //go through each key and value in the royalty object
        for (k, v) in self.royalties.iter() {
            //get the key
            let key = k.clone();
            //only insert into the payout if the key isn't the token owner (we add their payout at the end)
            if key != token_owner_id {
                //
                payout_object.payout.insert(key, royalty_to_payout(v, balance_u128));
                total_perpetual += v;
            }
        }

        // payout to previous owner who gets 100% - total perpetual royalties
        payout_object.payout.insert(token_owner_id, royalty_to_payout(ONE_HUNDRED_PERCENT_IN_BPS - total_perpetual, balance_u128));

        //return the payout object
        payout_object
    }

    //transfers the token to the receiver ID and returns the payout object that should be payed given the passed in balance.
    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        //assert that the user attached 1 yocto NEAR for security reasons
        assert_one_yocto();

        let payout = self.nft_payout(token_id.clone(), balance, max_len_payout);

        self.tokens.nft_transfer(receiver_id.try_into().unwrap(), token_id, Some(approval_id), memo);

        payout
    }
}