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

pub(crate) fn str_to_u128(val : string) -> u128 {
    let val_u128: u128;
    match u128::from_str_radix(&token_price_in_string, 10) {
        Ok(res) => {
            val_u128 = res;
        }
        Err(_e) => {
            env::panic_str("error when parse price_in_string to u128");
        }
    }
    val_u128
}
