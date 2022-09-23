use crate::{BalanceOf, Config, Decimal};
use frame_support::pallet_prelude::*;

use sp_std::fmt::Debug;

// ----------------------------------------------------------------------------------------------------
//		Structs and implementations
// ----------------------------------------------------------------------------------------------------
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Snapshot<T: Config> {
	pub interest_rate: Decimal,
	pub iv: Decimal,
	pub delta: Decimal,
	pub theta: Decimal,
	pub rho: Decimal,
	pub vega: Decimal,
	pub gamma: Decimal,
	pub option_price: BalanceOf<T>,
	pub asset_spot_price: BalanceOf<T>,
	pub total_issuance_buyer: BalanceOf<T>,
	pub total_premium_paid: BalanceOf<T>,
}

// ----------------------------------------------------------------------------------------------------
//		Constants
// ----------------------------------------------------------------------------------------------------
// pub const SECONDS_PER_YEAR: Decimal = Decimal::from_inner(31536000);
