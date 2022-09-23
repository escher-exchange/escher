//! Benchmarks for TokenizedOptions Pallet
use crate::{self as pallet_tokenized_options, types::*, Pallet as TokenizedOptions, *};

use codec::{Decode, Encode, MaxEncodedLen};
use composable_traits::{
	defi::DeFiComposableConfig, oracle::Price, tokenized_options::*, vault::VaultConfig,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{
	fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
	fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
	EnsureOrigin, Hooks,
};
use frame_system::{pallet_prelude::*, EventRecord, Pallet as System, RawOrigin};
use sp_runtime::Perquintill;
use sp_std::collections::btree_map::BTreeMap;

// ----------------------------------------------------------------------------------------------------
//		Helper functions
// ----------------------------------------------------------------------------------------------------
const UNIT: u128 = 10u128.pow(12);
const A: u128 = 2;
const B: u128 = 2000;
const C: u128 = 131;
const MINIMUM_PERIOD: u32 = 6000;

fn encode_decode<D: Decode, E: Encode>(value: E) -> D {
	let asset_id = value.encode();
	D::decode(&mut &asset_id[..]).unwrap()
}

pub fn recode_unwrap_u128<
	O: Decode + MaxEncodedLen + Encode,
	I: Decode + MaxEncodedLen + Encode,
>(
	raw: I,
) -> O {
	O::decode(&mut &raw.encode()[..]).unwrap()
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn set_oracle_price<T: Config + pallet_oracle::Config>(asset_id: T::MayBeAssetId, price: u64) {
	let asset_id: T::AssetId = encode_decode(asset_id);

	pallet_oracle::Prices::<T>::insert(
		asset_id,
		Price { price: <T as pallet_oracle::Config>::PriceValue::from(price), block: 0_u32.into() },
	);
}

fn initial_setup<T: Config + pallet_timestamp::Config>() {
	System::<T>::set_block_number(0u32.into());
	System::<T>::on_initialize(System::<T>::block_number());
	TokenizedOptions::<T>::on_initialize(System::<T>::block_number());
	<pallet_timestamp::Pallet<T>>::set_timestamp(0u32.into());
	System::<T>::on_finalize(System::<T>::block_number());
	produce_block::<T>(1_u32.into(), MINIMUM_PERIOD.into());
}

fn produce_block<T: Config + pallet_timestamp::Config>(
	n: T::BlockNumber,
	time: <T as pallet_timestamp::Config>::Moment,
) {
	if System::<T>::block_number() > 0u32.into() {
		System::<T>::on_finalize(System::<T>::block_number());
	}

	System::<T>::set_block_number(n);
	System::<T>::on_initialize(System::<T>::block_number());
	TokenizedOptions::<T>::on_initialize(System::<T>::block_number());
	<pallet_timestamp::Pallet<T>>::set_timestamp(time);
}

fn vault_benchmarking_setup<T: Config + pallet_oracle::Config>(
	asset_id: T::MayBeAssetId,
	price: u64,
) {
	let origin = OriginFor::<T>::from(RawOrigin::Root);

	set_oracle_price::<T>(asset_id, price * (UNIT as u64));

	let vault_config: VaultConfig<T::AccountId, T::MayBeAssetId> = VaultConfig {
		asset_id,
		manager: whitelisted_caller(),
		reserved: Perquintill::one(),
		strategies: BTreeMap::new(),
	};

	TokenizedOptions::<T>::create_asset_vault(origin, vault_config).unwrap();
}

fn valid_option_config<T: Config>() -> OptionConfigOf<T> {
	OptionConfigOf::<T> {
		base_asset_id: recode_unwrap_u128(B),
		quote_asset_id: recode_unwrap_u128(C),
		base_asset_strike_price: BalanceOf::<T>::from(50000u128 * UNIT),
		quote_asset_strike_price: UNIT.into(),
		option_type: OptionType::Call,
		exercise_type: ExerciseType::European,
		expiring_date: recode_unwrap_u128(30000u64),
		// Use this when https://github.com/paritytech/substrate/pull/10128 is merged
		// epoch: Epoch {
		// 	deposit: recode_unwrap_u128(0u64),
		// 	purchase: recode_unwrap_u128(3000u64),
		// 	exercise: recode_unwrap_u128(6000u64),
		// 	end: recode_unwrap_u128(9000u64)
		// },
		epoch: Epoch {
			deposit: recode_unwrap_u128(0u64),
			purchase: recode_unwrap_u128(12000u64),
			exercise: recode_unwrap_u128(30000u64),
			end: recode_unwrap_u128(48000u64),
		},
		status: Status::NotStarted,
		base_asset_amount_per_option: UNIT.into(),
		quote_asset_amount_per_option: UNIT.into(),
		total_issuance_seller: 0u128.into(),
		total_premium_paid: 0u128.into(),
		exercise_amount: 0u128.into(),
		base_asset_spot_price: 0u128.into(),
		total_issuance_buyer: 0u128.into(),
		total_shares_amount: 0u128.into(),
	}
}

fn default_option_benchmarking_setup<T: Config + pallet_timestamp::Config>() -> OptionIdOf<T> {
	let origin = OriginFor::<T>::from(RawOrigin::Root);

	let option_config: OptionConfigOf<T> = valid_option_config::<T>();

	TokenizedOptions::<T>::create_option(origin, option_config.clone()).unwrap();

	let option_hash = TokenizedOptions::<T>::generate_id(
		option_config.base_asset_id,
		option_config.quote_asset_id,
		option_config.base_asset_strike_price,
		option_config.quote_asset_strike_price,
		option_config.option_type,
		option_config.expiring_date,
		option_config.exercise_type,
	);

	produce_block::<T>(2u32.into(), (2u32 * MINIMUM_PERIOD).into());

	OptionHashToOptionId::<T>::get(option_hash).unwrap()
}

// ----------------------------------------------------------------------------------------------------
//		Benchmark tests
// ----------------------------------------------------------------------------------------------------

benchmarks! {
	where_clause {
		where
			T: pallet_tokenized_options::Config
			+ frame_system::Config
			+ DeFiComposableConfig
			+ pallet_oracle::Config
			+ pallet_timestamp::Config
	}

	create_asset_vault {
		let origin = OriginFor::<T>::from(RawOrigin::Root);

		set_oracle_price::<T>(recode_unwrap_u128(B), 50_000 * (UNIT as u64));
		set_oracle_price::<T>(recode_unwrap_u128(C), UNIT as u64);

		let vault_config: VaultConfig<T::AccountId, T::MayBeAssetId> = VaultConfig {
			asset_id: recode_unwrap_u128(B),
			manager: whitelisted_caller(),
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		};
	}: {
		TokenizedOptions::<T>::create_asset_vault(
			origin,
			vault_config,
		)?
	}
	verify {
		assert_last_event::<T>(Event::CreatedAssetVault {
			vault_id: recode_unwrap_u128(1u128),
			asset_id: recode_unwrap_u128(B),
		}.into())
	}

	create_option {
		let origin = OriginFor::<T>::from(RawOrigin::Root);

		vault_benchmarking_setup::<T>(recode_unwrap_u128(B), 50_000);
		vault_benchmarking_setup::<T>(recode_unwrap_u128(C), 1);

		let option_config = OptionConfigOf::<T> {
			base_asset_id: recode_unwrap_u128(B),
			quote_asset_id: recode_unwrap_u128(C),
			base_asset_strike_price: BalanceOf::<T>::from(50000u128 * UNIT),
			quote_asset_strike_price: UNIT.into(),
			option_type: OptionType::Call,
			exercise_type: ExerciseType::European,
			expiring_date: recode_unwrap_u128(36000u64),
			epoch: Epoch {
				deposit: recode_unwrap_u128(0u64),
				purchase: recode_unwrap_u128(18000u64),
				exercise: recode_unwrap_u128(36000u64),
				end: recode_unwrap_u128(54000u64)
			},
			status: Status::NotStarted,
			base_asset_amount_per_option: UNIT.into(),
			quote_asset_amount_per_option: UNIT.into(),
			total_issuance_seller: 0u128.into(),
			total_premium_paid: 0u128.into(),
			exercise_amount: 0u128.into(),
			base_asset_spot_price: 0u128.into(),
			total_issuance_buyer: 0u128.into(),
			total_shares_amount: 0u128.into(),
		};
	}: {
		TokenizedOptions::<T>::create_option(
			origin,
			option_config.clone(),
		)?
	}
	verify {
		assert_last_event::<T>(Event::CreatedOption {
			// ...01 and ...02 are for vaults lp_tokens
			option_id: recode_unwrap_u128(100000000003u128),
			option_config,
		}.into())
	}

	sell_option {
		initial_setup::<T>();
		let seller_account: <T as frame_system::Config>::AccountId = whitelisted_caller::<T::AccountId>();
		let seller_origin = OriginFor::<T>::from(RawOrigin::Signed(seller_account.clone()));

		vault_benchmarking_setup::<T>(recode_unwrap_u128(B), 50_000);
		vault_benchmarking_setup::<T>(recode_unwrap_u128(C), 1);
		AssetsOf::<T>::mint_into(recode_unwrap_u128(B), &seller_account, (UNIT * 1u128).into())?;

		let option_id = default_option_benchmarking_setup::<T>();
		let option_amount: BalanceOf<T> = 1u128.into();
	}: {
		TokenizedOptions::<T>::sell_option(
			seller_origin,
			option_amount,
			option_id
		)?
	}
	verify {
		assert_last_event::<T>(Event::SellOption {
			user: seller_account,
			option_amount,
			option_id,
		}.into())
	}

	delete_sell_option {
		initial_setup::<T>();

		let seller_account: <T as frame_system::Config>::AccountId = whitelisted_caller::<T::AccountId>();
		let seller_origin = OriginFor::<T>::from(RawOrigin::Signed(seller_account.clone()));

		vault_benchmarking_setup::<T>(recode_unwrap_u128(B), 50_000);
		vault_benchmarking_setup::<T>(recode_unwrap_u128(C), 1);
		AssetsOf::<T>::mint_into(recode_unwrap_u128(B), &seller_account, (UNIT * 1u128).into())?;

		let option_id = default_option_benchmarking_setup::<T>();
		let option_amount: BalanceOf<T> = 1u128.into();
		TokenizedOptions::<T>::sell_option(seller_origin.clone(), option_amount, option_id).unwrap();
	}: {
		TokenizedOptions::<T>::delete_sell_option(
			seller_origin,
			option_amount,
			option_id
		)?
	}
	verify {
		assert_last_event::<T>(Event::DeleteSellOption {
			user: seller_account,
			option_amount,
			option_id,
		}.into())
	}

	buy_option {
		initial_setup::<T>();

		let seller_account: <T as frame_system::Config>::AccountId = whitelisted_caller::<T::AccountId>();
		let seller_origin = OriginFor::<T>::from(RawOrigin::Signed(seller_account.clone()));

		let buyer_account: <T as frame_system::Config>::AccountId = account("BUYER", 1, 0);
		let buyer_origin = OriginFor::<T>::from(RawOrigin::Signed(buyer_account.clone()));

		vault_benchmarking_setup::<T>(recode_unwrap_u128(B), 50_000);
		vault_benchmarking_setup::<T>(recode_unwrap_u128(C), 1);
		AssetsOf::<T>::mint_into(recode_unwrap_u128(B), &seller_account, (UNIT * 1u128).into())?;
		AssetsOf::<T>::mint_into(recode_unwrap_u128(C), &buyer_account, (UNIT * 1000u128).into())?;

		let option_id = default_option_benchmarking_setup::<T>();
		let option_amount: BalanceOf<T> = 1u128.into();
		TokenizedOptions::<T>::sell_option(seller_origin, option_amount, option_id).unwrap();
		produce_block::<T>(3u32.into(), (3u32 * MINIMUM_PERIOD).into());
	}: {
		TokenizedOptions::<T>::buy_option(
			buyer_origin,
			option_amount,
			option_id
		)?
	}
	verify {
		assert_last_event::<T>(Event::BuyOption {
			user: buyer_account,
			option_amount,
			option_id,
		}.into())
	}

	exercise_option {
		initial_setup::<T>();

		let seller_account: <T as frame_system::Config>::AccountId = whitelisted_caller::<T::AccountId>();
		let seller_origin = OriginFor::<T>::from(RawOrigin::Signed(seller_account.clone()));

		let buyer_account: <T as frame_system::Config>::AccountId = account("BUYER", 1, 0);
		let buyer_origin = OriginFor::<T>::from(RawOrigin::Signed(buyer_account.clone()));

		vault_benchmarking_setup::<T>(recode_unwrap_u128(B), 50_000);
		vault_benchmarking_setup::<T>(recode_unwrap_u128(C), 1);
		AssetsOf::<T>::mint_into(recode_unwrap_u128(B), &seller_account, (UNIT * 1u128).into())?;
		AssetsOf::<T>::mint_into(recode_unwrap_u128(C), &buyer_account, (UNIT * 1000u128).into())?;

		let option_id = default_option_benchmarking_setup::<T>();
		let option_amount: BalanceOf<T> = 1u128.into();

		TokenizedOptions::<T>::sell_option(seller_origin, option_amount, option_id).unwrap();
		produce_block::<T>(3u32.into(), (3u32 * MINIMUM_PERIOD).into());

		TokenizedOptions::<T>::buy_option(buyer_origin.clone(), option_amount, option_id).unwrap();

		// Set timestamp to 5000 (exercise phase can start)
		// This can be deleted when https://github.com/paritytech/substrate/pull/10128 is merged
		produce_block::<T>(5u32.into(), (5u32 * MINIMUM_PERIOD).into());

		// During this block's on_initialize, the option passes to exercise phase
		produce_block::<T>(6u32.into(), (6u32 * MINIMUM_PERIOD).into());
	}: {
		TokenizedOptions::<T>::exercise_option(
			buyer_origin,
			option_amount,
			option_id
		)?
	}
	verify {
		assert_last_event::<T>(Event::ExerciseOption {
			user: buyer_account,
			option_amount,
			option_id,
		}.into())
	}

	withdraw_collateral {
		initial_setup::<T>();

		let seller_account: <T as frame_system::Config>::AccountId = whitelisted_caller::<T::AccountId>();
		let seller_origin = OriginFor::<T>::from(RawOrigin::Signed(seller_account.clone()));

		let buyer_account: <T as frame_system::Config>::AccountId = account("BUYER", 1, 0);
		let buyer_origin = OriginFor::<T>::from(RawOrigin::Signed(buyer_account.clone()));

		vault_benchmarking_setup::<T>(recode_unwrap_u128(B), 50_000);
		vault_benchmarking_setup::<T>(recode_unwrap_u128(C), 1);
		AssetsOf::<T>::mint_into(recode_unwrap_u128(B), &seller_account, (UNIT * 1u128).into())?;
		AssetsOf::<T>::mint_into(recode_unwrap_u128(C), &buyer_account, (UNIT * 1000u128).into())?;

		let option_id = default_option_benchmarking_setup::<T>();
		let option_amount: BalanceOf<T> = 1u128.into();

		TokenizedOptions::<T>::sell_option(seller_origin.clone(), option_amount, option_id).unwrap();
		produce_block::<T>(3u32.into(), (3u32 * MINIMUM_PERIOD).into());

		TokenizedOptions::<T>::buy_option(buyer_origin.clone(), option_amount, option_id).unwrap();

		// Set timestamp to 5000 (exercise phase can start)
		// This can be deleted when https://github.com/paritytech/substrate/pull/10128 is merged
		produce_block::<T>(5u32.into(), (5u32 * MINIMUM_PERIOD).into());

		// During this block's on_initialize, the option passes to exercise phase
		produce_block::<T>(6u32.into(), (6u32 * MINIMUM_PERIOD).into());

		// Not needed, but why not
		TokenizedOptions::<T>::exercise_option(buyer_origin, option_amount, option_id).unwrap();

	}: {
		TokenizedOptions::<T>::withdraw_collateral(
			seller_origin,
			option_id
		)?
	}
	verify {
		assert_last_event::<T>(Event::WithdrawCollateral {
			user: seller_account,
			option_id,
		}.into())
	}

}

impl_benchmark_test_suite!(
	TokenizedOptions,
	crate::mocks::runtime::ExtBuilder::default().build(),
	crate::mocks::runtime::MockRuntime,
);
