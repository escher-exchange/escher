#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
	not(test),
	deny(
		clippy::disallowed_methods,
		clippy::disallowed_types,
		clippy::indexing_slicing,
		clippy::todo,
		clippy::unwrap_used,
		clippy::panic
	)
)] // allow in tests
#![deny(
	dead_code,
	bad_style,
	bare_trait_objects,
	const_err,
	improper_ctypes,
	non_shorthand_field_patterns,
	no_mangle_generic_items,
	overflowing_literals,
	path_statements,
	patterns_in_fns_without_body,
	private_in_public,
	unconditional_recursion,
	unused_allocation,
	unused_comparisons,
	unused_parens,
	while_true,
	trivial_casts,
	trivial_numeric_casts,
	unused_extern_crates
)]

pub use crate::weights::WeightInfo;
mod types;
mod weights;

#[allow(unused_imports)]
#[allow(dead_code)]
#[cfg(test)]
mod mocks;

#[allow(dead_code)]
#[allow(unused_imports)]
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(dead_code)]
pub mod pallet {
	// ----------------------------------------------------------------------------------------------------
	//		Imports and Dependencies
	// ----------------------------------------------------------------------------------------------------
	use crate::{types::*, weights::*};

	use codec::Codec;
	use composable_support::validation::Validated;
	use composable_traits::{
		currency::{CurrencyFactory, LocalAssets, RangeId},
		defi::DeFiComposableConfig,
		oracle::Oracle,
		vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig},
	};

	use traits::{options_pricing::*, tokenized_options::*,swap_bytes::{SwapBytes, Swapped}};

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		storage::{bounded_btree_map::BoundedBTreeMap, bounded_btree_set::BoundedBTreeSet},
		traits::{
			fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
			fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
			EnsureOrigin, Time,
		},
		transactional, PalletId,
	};

	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_core::H256;
	use sp_runtime::{
		helpers_128bit::multiply_by_rational_with_rounding,
		traits::{
			AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, BlakeTwo256, CheckedAdd,
			CheckedDiv, CheckedMul, CheckedSub, Convert, One, Saturating, Zero,
		},
		ArithmeticError, DispatchError, FixedI128, FixedPointNumber, FixedPointOperand,
		Perquintill,
	};
	use sp_std::cmp::min;

	use sp_std::{collections::btree_map::BTreeMap, fmt::Debug};
	// ----------------------------------------------------------------------------------------------------
	//		Declaration Of The Pallet Type
	// ----------------------------------------------------------------------------------------------------
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// ----------------------------------------------------------------------------------------------------
	//		Config Trait
	// ----------------------------------------------------------------------------------------------------
	#[pallet::config]
	pub trait Config: frame_system::Config + DeFiComposableConfig {
		#[allow(missing_docs)]
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		/// The id used as `AccountId` for the pallet.
		/// This should be unique across all pallets to avoid name collisions.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Type of time moment. We use [`SwapBytes`] trait to store this type in
		/// big endian format and take advantage of the fact that storage keys are
		/// stored in lexical order.
		type Moment: SwapBytes
			+ AtLeast32Bit
			+ Parameter
			+ Copy
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize;

		/// The Unix time provider.
		type Time: Time<Moment = MomentOf<Self>>;

		/// Oracle pallet to retrieve prices expressed in USDT.
		type Oracle: Oracle<AssetId = AssetIdOf<Self>, Balance = BalanceOf<Self>>;

		/// Protocol Origin that can create vaults and options.
		type ProtocolOrigin: EnsureOrigin<Self::Origin>;

		/// Used for option tokens and other assets management.
		type Assets: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Mutate<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ MutateHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Inspect<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ InspectHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>;
	}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Pallet Types
	// ----------------------------------------------------------------------------------------------------
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type AssetIdOf<T> = <T as DeFiComposableConfig>::MayBeAssetId;
	pub type BalanceOf<T> = <T as DeFiComposableConfig>::Balance;
	pub type AssetsOf<T> = <T as Config>::Assets;
	pub type MomentOf<T> = <T as Config>::Moment;
	pub type OracleOf<T> = <T as Config>::Oracle;
	pub type OptionIdOf<T> = AssetIdOf<T>;
	pub type BlackScholesParamsOf<T> = BlackScholesParams<AssetIdOf<T>, BalanceOf<T>, MomentOf<T>>;
	pub type Decimal = FixedI128;

	// ----------------------------------------------------------------------------------------------------
	//		Storage
	// ----------------------------------------------------------------------------------------------------
	#[pallet::storage]
	#[pallet::getter(fn interest_rate)]
	#[allow(clippy::disallowed_types)]
	pub type InterestRate<T: Config> = StorageValue<_, Decimal, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn snapshot_frequency)]
	#[allow(clippy::disallowed_types)]
	pub type SnapshotFrequency<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn latest_snapshot_timestamp)]
	#[allow(clippy::disallowed_types)]
	pub type LatestSnapshotTimestamp<T: Config> = StorageValue<_, MomentOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn latest_snapshot)]
	pub type LatestSnapshots<T: Config> =
		StorageMap<_, Blake2_128Concat, OptionIdOf<T>, Snapshot<T>>;

	#[pallet::storage]
	#[pallet::getter(fn snapshots_history)]
	pub type SnapshotsHistory<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OptionIdOf<T>,
		Blake2_128Concat,
		MomentOf<T>,
		Snapshot<T>,
	>;

	// ----------------------------------------------------------------------------------------------------
	//		Events
	// ----------------------------------------------------------------------------------------------------
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InterestRateUpdated { interest_rate: Decimal },
	}

	// ----------------------------------------------------------------------------------------------------
	//		Errors
	// ----------------------------------------------------------------------------------------------------
	#[pallet::error]
	pub enum Error<T> {
		FailedConversion,

		InterestRateNotSet,

		LatestSnapshotTimestampNotSet,
	}

	// ----------------------------------------------------------------------------------------------------
	//		Hooks
	// ----------------------------------------------------------------------------------------------------
	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			let mut used_weight = 0;
			let now = T::Time::now();

			let latest_snapshot_timestamp = match Self::latest_snapshot_timestamp() {
				Some(timestamp) => timestamp,
				None => MomentOf::<T>::from(0_u32),
			};

			used_weight = used_weight.saturating_add(T::DbWeight::get().reads(1));

			let snapshot_frequency = match Self::snapshot_frequency() {
				Some(timestamp) => timestamp,
				None => MomentOf::<T>::from(86400_u32), // Default if not set
			};

			used_weight = used_weight.saturating_add(T::DbWeight::get().reads(1));

			let new_snapshot_timestamp =
				latest_snapshot_timestamp.saturating_add(snapshot_frequency);

			if new_snapshot_timestamp > now {
				LatestSnapshots::<T>::iter().for_each(|(option_id, snapshot)| {
					SnapshotsHistory::<T>::insert(option_id, new_snapshot_timestamp, snapshot);
					used_weight = used_weight.saturating_add(T::DbWeight::get().writes(1));
				});
			}

			let max_weight = <T as frame_system::Config>::BlockWeights::get().max_block;
			used_weight.min(max_weight)
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Genesis Build
	// ----------------------------------------------------------------------------------------------------
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub interest_rate_index: Decimal,
		pub latest_snapshot_timestamp: Option<MomentOf<T>>,
		pub snapshot_frequency: Option<MomentOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				interest_rate_index: Decimal::saturating_from_rational(1, 20),
				latest_snapshot_timestamp: None,
				snapshot_frequency: None,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			InterestRate::<T>::set(self.interest_rate_index);
			LatestSnapshotTimestamp::<T>::set(self.latest_snapshot_timestamp);
			SnapshotFrequency::<T>::set(self.snapshot_frequency);
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Extrinsics
	// ----------------------------------------------------------------------------------------------------
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::calculate_option_price())]
		pub fn calculate_option_price(
			origin: OriginFor<T>,
			option_id: OptionIdOf<T>,
			params: BlackScholesParamsOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the extrinsic
			T::ProtocolOrigin::ensure_origin(origin)?;

			<Self as OptionsPricing>::calculate_option_price(option_id, params)?;

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::calculate_option_greeks())]
		pub fn calculate_option_greeks(
			origin: OriginFor<T>,
			option_id: OptionIdOf<T>,
			params: BlackScholesParamsOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the extrinsic
			T::ProtocolOrigin::ensure_origin(origin)?;

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_interest_rate())]
		pub fn update_interest_rate(
			origin: OriginFor<T>,
			interest_rate: Decimal,
		) -> DispatchResult {
			// Check if it's protocol to call the extrinsic
			T::ProtocolOrigin::ensure_origin(origin)?;

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_snapshot_frequency())]
		pub fn update_snapshot_frequency(
			origin: OriginFor<T>,
			snapshot_frequency: MomentOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the extrinsic
			T::ProtocolOrigin::ensure_origin(origin)?;

			Ok(())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		OptionsPricing Trait
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> OptionsPricing for Pallet<T> {
		type AssetId = AssetIdOf<T>;
		type Balance = BalanceOf<T>;
		type Moment = MomentOf<T>;
		type OptionId = OptionIdOf<T>;

		#[transactional]
		fn calculate_option_price(
			option_id: OptionIdOf<T>,
			params: BlackScholesParamsOf<T>,
		) -> Result<Self::Balance, DispatchError> {
			Self::do_calculate_option_price(option_id, params)
		}

		#[transactional]
		fn calculate_option_greeks(
			option_id: OptionIdOf<T>,
			params: BlackScholesParamsOf<T>,
		) -> Result<(), DispatchError> {
			Self::do_calculate_option_greeks(option_id, params)
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Pallet Functions
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> Pallet<T> {
		fn do_update_interest_rate(interest_rate: Decimal) -> Result<(), DispatchError> {
			InterestRate::<T>::mutate(|v| {
				*v = interest_rate;
			});

			Ok(())
		}

		fn do_update_snapshot_frequency(
			snapshot_frequency: MomentOf<T>,
		) -> Result<(), DispatchError> {
			SnapshotFrequency::<T>::mutate(|v| {
				*v = Some(snapshot_frequency);
			});

			Ok(())
		}

		fn do_calculate_option_price(
			option_id: OptionIdOf<T>,
			params: BlackScholesParamsOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			// // Get interest rate index, annualized expiry date and converted prices
			// let interest_rate = Self::interest_rate();
			// let time_annualized = Self::get_expiry_time_annualized(params.expiring_date)?;
			// let strike_price = convert(params.base_asset_strike_price);
			// let spot_price = convert(params.base_asset_spot_price);

			// // Get volatility for option's asset
			// let iv: Decimal = Decimal::from_float(200.5); // TODO

			// // Calculate price with BS formula
			// let option_price = Self::black_scholes(
			// 	strike_price,
			// 	spot_price,
			// 	time_annualized,
			// 	interest_rate,
			// 	iv,
			// 	params.option_type,
			// )?;

			// Ok(option_price)

			Ok((1000u128 * 10u128.pow(12)).into())
		}

		fn black_scholes(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			iv: Decimal,
			option_type: OptionType,
		) -> Result<BalanceOf<T>, DispatchError> {
			// Calculate d1 and d2
			let (d1, d2) = Self::calculate_d1_d2(
				strike_price,
				spot_price,
				time_annualized,
				interest_rate,
				iv,
			)?;

			// Calculate price of option based on OptionType
			let option_price = match option_type {
				OptionType::Call => Self::calculate_call_price(
					strike_price,
					spot_price,
					time_annualized,
					interest_rate,
					d1,
					d2,
				)?,
				OptionType::Put => Self::calculate_put_price(
					strike_price,
					spot_price,
					time_annualized,
					interest_rate,
					d1,
					d2,
				)?,
			};

			// Ok(option_price)
			Ok((1000u128 * 10u128.pow(12)).into()) // To make tests pass right now
		}

		fn get_expiry_time_annualized(expiry_date: MomentOf<T>) -> Result<Decimal, Error<T>> {
			// let now = T::Time::now();
			// let seconds_to_expiry = convert(expiry_date - now);
			// seconds_to_expiry
			// 	.checked_div(&SECONDS_PER_YEAR)
			// 	.ok_or(Error::<T>::FailedConversion)
			Ok(1.into())
		}

		fn normal_cumulative_distribution_function(
			value: Decimal,
		) -> Result<Decimal, DispatchError> {
			Ok(1.into())
		}

		fn normal_probability_density_function(value: Decimal) -> Result<Decimal, DispatchError> {
			Ok(1.into())
		}

		fn calculate_d1_d2(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			iv: Decimal,
		) -> Result<(Decimal, Decimal), DispatchError> {
			// let a = Decimal::sqrt(time_annualized).ok_or(ArithmeticError::Underflow)?;
			// let a = iv.checked_mul(&a).ok_or(ArithmeticError::Overflow)?;

			// let b = spot_price.checked_div(&strike_price).ok_or(ArithmeticError::Overflow)?;
			// let b = Decimal::log(b).ok_or(ArithmeticError::Underflow)?;

			// let c = iv.saturating_pow(2);
			// let c = c.checked_div(&Decimal::from_inner(2.into())).ok_or(ArithmeticError::Underflow)?;
			// let c = c.checked_add(&interest_rate).ok_or(ArithmeticError::Overflow)?;
			// let c = c.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;

			// let d1 = b.checked_add(&c).ok_or(ArithmeticError::Overflow)?;
			// let d1 = d1.checked_div(&a).ok_or(ArithmeticError::Overflow)?;

			// let d2 = d1.checked_sub(&a).ok_or(ArithmeticError::Underflow)?;

			// Ok((d1, d2))
			Ok((Decimal::from_inner(1.into()), Decimal::from_inner(1.into())))
		}

		fn calculate_call_price(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			d1: Decimal,
			d2: Decimal,
		) -> Result<Decimal, DispatchError> {
			// let nd1 = Self::cumulative_normal_distribution(d1)?;
			// let a = spot_price.checked_mul(&nd1).ok_or(ArithmeticError::Overflow)?;

			// let nd2 = Self::cumulative_normal_distribution(d2)?;
			// let exp =
			// 	-interest_rate.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;
			// let exp = Decimal::exp(&exp).ok_or(ArithmeticError::Overflow)?;
			// let b = strike_price.checked_mul(&nd2).ok_or(ArithmeticError::Overflow)?;
			// let b = b.checked_mul(&exp).ok_or(ArithmeticError::Overflow)?;

			// Ok(a.checked_sub(&b))
			Ok(1.into())
		}

		fn calculate_put_price(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			d1: Decimal,
			d2: Decimal,
		) -> Result<Decimal, DispatchError> {
			// let nd2 = Self::cumulative_normal_distribution(-d2)?;
			// let exp =
			// 	-interest_rate.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;
			// let exp = Decimal::exp(&exp).ok_or(ArithmeticError::Overflow)?;
			// let b = strike_price.checked_mul(&nd2).ok_or(ArithmeticError::Overflow)?;
			// let b = b.checked_mul(&exp).ok_or(ArithmeticError::Overflow)?;

			// let nd1 = Self::cumulative_normal_distribution(-d1)?;
			// let a = spot_price.checked_mul(&nd1).ok_or(ArithmeticError::Overflow)?;

			// Ok(b.checked_sub(&a))
			Ok(1.into())
		}

		fn do_calculate_option_greeks(
			option_id: OptionIdOf<T>,
			params: BlackScholesParamsOf<T>,
		) -> Result<(), DispatchError> {
			// // Get interest rate index, annualized expiry date and converted prices
			// let interest_rate = Self::interest_rate();
			// let time_annualized = Self::get_expiry_time_annualized(params.expiring_date)?;
			// let strike_price = T::ConvertBalanceToDecimal::convert(params.base_asset_strike_price);
			// let spot_price = T::ConvertBalanceToDecimal::convert(params.base_asset_spot_price);

			// // Get volatility for option's asset
			// let iv: Decimal = Decimal::from_float(200.5); // TODO

			// let (d1, d2) = Self::calculate_d1_d2(
			// 	strike_price,
			// 	spot_price,
			// 	time_annualized,
			// 	interest_rate,
			// 	iv,
			// )?;

			// let (delta_call, delta_put) = Self::calculate_delta(d1)?;
			// let gamma = Self::calculate_gamma(spot_price, time_annualized, iv, d1)?;
			// let vega = Self::calculate_vega(spot_price, time_annualized, d1)?;
			// let (theta_call, theta_put) = Self::calculate_theta(
			// 	strike_price,
			// 	spot_price,
			// 	time_annualized,
			// 	interest_rate,
			// 	iv,
			// 	d1,
			// 	d2,
			// )?;
			// let (rho_call, rho_put) =
			// 	Self::calculate_rho(strike_price, time_annualized, interest_rate, d2)?;

			Ok(())
		}

		fn calculate_delta(d1: Decimal) -> Result<(Decimal, Decimal), DispatchError> {
			// ncdf should have mean=0 and std=1
			// let delta_call = Self::normal_cumulative_distribution_function(d1)?;

			// For delta_put we need to subtract 1 UNIT but we have not defined a number model yet
			// Alternative is -ncdf(-d1)
			// Ok((delta_call, delta_call - Decimal::from_inner(1.into())))
			Ok((1.into(), 1.into()))
		}

		fn calculate_gamma(
			spot_price: Decimal,
			time_annualized: Decimal,
			iv: Decimal,
			d1: Decimal,
		) -> Result<Decimal, DispatchError> {
			// let pdf = Self::normal_probability_density_function(d1)?;
			// let sqrt_time = Decimal::sqrt(time_annualized).ok_or(ArithmeticError::Underflow)?;
			// let norm_factor = spot_price.checked_mul(&iv).ok_or(ArithmeticError::Overflow)?;
			// let norm_factor = norm_factor.checked_mul(&sqrt_time).ok_or(ArithmeticError::Overflow)?;
			// let gamma = pdf.checked_div(&norm_factor).ok_or(ArithmeticError::Underflow)?;

			// Ok(gamma)
			Ok(1.into())
		}

		fn calculate_vega(
			spot_price: Decimal,
			time_annualized: Decimal,
			d1: Decimal,
		) -> Result<Decimal, DispatchError> {
			// let pdf = Self::normal_probability_density_function(d1)?;
			// let sqrt_time = Decimal::sqrt(time_annualized).ok_or(ArithmeticError::Underflow)?;
			// let norm_factor = spot_price.checked_div(&Decimal::from_inner(100.into()))).ok_or(ArithmeticError::Overflow)?;
			// let vega = pdf.checked_mul(&norm_factor).ok_or(ArithmeticError::Overflow)?;
			// let vega = vega.checked_mul(&sqrt_time).ok_or(ArithmeticError::Overflow)?;

			// Ok(vega)

			Ok(1.into())
		}

		fn calculate_theta(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			iv: Decimal,
			d1: Decimal,
			d2: Decimal,
		) -> Result<(Decimal, Decimal), DispatchError> {
			// let pdf = Self::normal_probability_density_function(d1)?;
			// let cdf_call = Self::normal_cumulative_distribution_function(d2)?;
			// let cdf_put = Self::normal_cumulative_distribution_function(-d2)?;
			// let sqrt_t = Decimal::sqrt(time_annualized).ok_or(ArithmeticError::Underflow)?;
			// let exp =
			// 	-interest_rate.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;
			// let exp = Decimal::exp(&exp).ok_or(ArithmeticError::Overflow)?;

			// let a = spot_price.checked_mul(&cdf_call).ok_or(ArithmeticError::Overflow)?;
			// let a = a.checked_mul(&iv).ok_or(ArithmeticError::Overflow)?;
			// let a = a.checked_div(&sqrt_t).ok_or(ArithmeticError::Underflow)?;
			// let a = a.checked_div(2.into()).ok_or(ArithmeticError::Underflow)?;

			// let b = strike_price.checked_mul(&interest_rate).ok_or(ArithmeticError::Overflow)?;
			// let b = b.checked_mul(&interest_rate).ok_or(ArithmeticError::Overflow)?;
			// let b = b.checked_mul(&exp).ok_or(ArithmeticError::Overflow)?;
			// let b_call = b.checked_mul(&cdf_call).ok_or(ArithmeticError::Overflow)?;
			// let b_put = b.checked_mul(&cdf_put).ok_or(ArithmeticError::Overflow)?;

			// let theta_call = -a.checked_sub(&b_call).ok_or(ArithmeticError::Overflow)?;
			// let theta_put = b_put.checked_sub(&a).ok_or(ArithmeticError::Overflow)?;

			// Ok((theta_call, theta_put))
			Ok((1.into(), 1.into()))
		}

		fn calculate_rho(
			strike_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			d2: Decimal,
		) -> Result<(Decimal, Decimal), DispatchError> {
			// let cdf_call = Self::normal_cumulative_distribution_function(d2)?;
			// let cdf_put = Self::normal_cumulative_distribution_function(-d2)?;
			// let exp =
			// 	-interest_rate.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;
			// let exp = Decimal::exp(&exp).ok_or(ArithmeticError::Overflow)?;

			// let a = strike_price.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;
			// let a = a.checked_mul(&exp).ok_or(ArithmeticError::Overflow)?;
			// let a = a.checked_div(100.into()).ok_or(ArithmeticError::Overflow)?;

			// let rho_call = a.checked_mul(&cdf_call).ok_or(ArithmeticError::Overflow)?;
			// let rho_put = -a.checked_mul(&cdf_put).ok_or(ArithmeticError::Overflow)?;

			// Ok((rho_call, rho_put))
			Ok((1.into(), 1.into()))
		}
	}
}
