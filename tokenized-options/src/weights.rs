#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(trivial_numeric_casts)]
#![allow(clippy::unnecessary_cast)]
use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

const WEIGHT: i32 = 1_000;

pub trait WeightInfo {
	fn create_asset_vault() -> Weight;
	fn create_option() -> Weight;
	fn sell_option() -> Weight;
	fn delete_sell_option() -> Weight;
	fn buy_option() -> Weight;
	fn exercise_option() -> Weight;
	fn withdraw_collateral() -> Weight;
}

/// Weights for pallet_tokenized_options using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: TokenizedOptions AssetToVault + Validation (r:1 w:1)
	// Storage: Oracle LocalAssets (r:1 w:0)
	// Storage: Vault VaultCount (r:1 w:1)
	// Storage: Factory CurrencyCounter (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Vault LpTokensToVaults (r:0 w:1)
	// Storage: Vault Allocations (r:0 w:1)
	// Storage: Vault Vaults (r:0 w:1)
	fn create_asset_vault() -> Weight {
		(1000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	// Storage: TokenizedOptions OptionIdToOption (r:0 w:1)
	// Storage: TokenizedOptions OptionHashToOptionId + Validation (r:1 w:1)
	// Storage: Oracle LocalAssets (r:2 w:0)
	// Storage: Factory AssetIdRanges (r:1 w:1)
	// Storage: Factory AssetEd (r:0 w:1)
	// Storage: TokenizedOptions Scheduler (r:0 w:4)
	fn create_option() -> Weight {
		(1000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	// Storage: TokenizedOptions OptionIdToOption (r:1 w:1)
	// Storage: TokenizedOptions AssetToVault (r:1 w:0)
	// Storage: Vault Vaults (r:2 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: TokenizedOptions Sellers (r:1 w:1)
	// Storage: Vault deposit() (r:9 w:5)
	fn sell_option() -> Weight {
		(1000 as Weight)
			.saturating_add(T::DbWeight::get().reads(16 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	// Storage: TokenizedOptions Sellers (r:1 w:1)
	// Storage: TokenizedOptions AssetToVault (r:1 w:0)
	// Storage: Vault Vaults (r:4 w:0)
	// Storage: Tokens TotalIssuance (r:2 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Vault withdraw() (r:8 w:4)
	fn delete_sell_option() -> Weight {
		(1000 as Weight)
			.saturating_add(T::DbWeight::get().reads(17 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}

	// TODO: depends on pricing pallet weights (TBD)
	fn buy_option() -> Weight {
		(1000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	// Storage: TokenizedOptions OptionIdToOption (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	fn exercise_option() -> Weight {
		(1000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}

	// Storage: TokenizedOptions OptionIdToOption (r:1 w:0)
	// Storage: TokenizedOptions Sellers (r:1 w:1)
	// Storage: TokenizedOptions AssetToVault (r:1 w:0)
	// Storage: Vault Vaults (r:1 w:0)
	// Storage: Assets Account (r:1 w:0)
	// Storage: Vault withdraw() (r:8 w:4)
	// Storage: System Account (r:2 w:2)
	fn withdraw_collateral() -> Weight {
		(1000 as Weight)
			.saturating_add(T::DbWeight::get().reads(15 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_asset_vault() -> Weight {
		WEIGHT as Weight
	}

	fn create_option() -> Weight {
		WEIGHT as Weight
	}

	fn sell_option() -> Weight {
		WEIGHT as Weight
	}

	fn delete_sell_option() -> Weight {
		WEIGHT as Weight
	}

	fn buy_option() -> Weight {
		WEIGHT as Weight
	}

	fn exercise_option() -> Weight {
		WEIGHT as Weight
	}

	fn withdraw_collateral() -> Weight {
		WEIGHT as Weight
	}
}
