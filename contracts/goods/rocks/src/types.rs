use crate::*;
use near_sdk::{
    serde::{Deserialize, Serialize},
};
use near_sdk::json_types::U128;

//defines the payout type we'll be returning as a part of the royalty standards.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}
