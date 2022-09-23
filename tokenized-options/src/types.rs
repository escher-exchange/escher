use crate::Config;
use frame_support::pallet_prelude::*;

use traits::tokenized_options::*;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash, Zero};
use sp_std::fmt::Debug;

/// Represent the option with the attributes to be configured
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct OptionToken<T: Config> {
	// Core attributes of an option, used to uniquely identify an option
	pub base_asset_id: T::MayBeAssetId,
	pub quote_asset_id: T::MayBeAssetId,
	pub base_asset_strike_price: T::Balance,
	pub quote_asset_strike_price: T::Balance,
	pub option_type: OptionType,
	pub expiring_date: T::Moment,
	pub exercise_type: ExerciseType,

	// Helper attributes
	pub epoch: Epoch<T::Moment>,
	pub status: Status,
	pub base_asset_amount_per_option: T::Balance,
	pub quote_asset_amount_per_option: T::Balance,
	pub total_issuance_seller: T::Balance,
	pub total_premium_paid: T::Balance,
	pub exercise_amount: T::Balance,
	pub base_asset_spot_price: T::Balance,
	pub total_issuance_buyer: T::Balance,
	pub total_shares_amount: T::Balance,
}

impl<T: Config> OptionToken<T> {
	pub fn generate_id(&self) -> H256 {
		BlakeTwo256::hash_of(&(
			self.base_asset_id,
			self.quote_asset_id,
			self.base_asset_strike_price,
			self.quote_asset_strike_price,
			self.option_type,
			self.expiring_date,
			self.exercise_type,
		))
	}
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct SellerPosition<T: Config> {
	pub option_amount: T::Balance,
	pub shares_amount: T::Balance,
}

impl<T: Config> Default for SellerPosition<T> {
	fn default() -> Self {
		SellerPosition { option_amount: T::Balance::zero(), shares_amount: T::Balance::zero() }
	}
}
