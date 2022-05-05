use near_sdk::json_types::U128;
use near_sdk::require;
use crate::*;

//convert the royalty percentage and amount to pay into a payout (U128)
pub(crate) fn royalty_to_payout(royalty_percentage: u16, amount_to_pay: Balance) -> U128 {
    U128(royalty_percentage as u128 * amount_to_pay / ONE_HUNDRED_PERCENT_IN_BPS as u128)
}

pub(crate) fn assert_at_least_one_yocto() {
    require!(env::attached_deposit() >= 1, "Requires attached deposit of at least 1 yoctoNEAR")
}

pub(crate) fn gen_token_id(metaverse_id: &String, rock_id: &String) -> String {
    let token_id = format!("{}:{}", metaverse_id, rock_id);
    token_id
}
