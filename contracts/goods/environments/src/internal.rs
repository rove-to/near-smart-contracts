use near_sdk::json_types::U128;
use crate::*;

//convert the royalty percentage and amount to pay into a payout (U128)
pub(crate) fn royalty_to_payout(royalty_percentage: u16, amount_to_pay: Balance) -> U128 {
    U128(royalty_percentage as u128 * amount_to_pay / ONE_HUNDRED_PERCENT_IN_BPS as u128)
}
