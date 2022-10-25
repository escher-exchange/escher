use std::collections::HashMap;

use crate::mocks::runtime::{Assets, ExtBuilder, MockRuntime, Origin, TokenizedOptions, Vault};

use crate::mocks::{accounts::*, assets::*};

use crate::{pallet::OptionHashToOptionId, tests::*};

use composable_traits::vault::Vault as VaultTrait;
use frame_support::{assert_ok, traits::fungibles::Inspect};

// ----------------------------------------------------------------------------------------------------
//		Settle Options Tests
// ----------------------------------------------------------------------------------------------------
pub fn settle_options_success_checks() {
	let option_ids = OptionIdToOption::<MockRuntime>::iter_keys().collect::<Vec<AssetId>>();

	let mut initial_option_info: HashMap<AssetId, (u128, u128, u128, u128)> = HashMap::new();

	for option_id in &option_ids {
		let option = TokenizedOptions::option_id_to_option(option_id).unwrap();
		let base_asset_spot_price = get_oracle_price(option.base_asset_id, UNIT);
		let total_issuance_buyer = Assets::total_issuance(*option_id);

		let (asset_id, collateral_for_option) = match option.option_type {
			OptionType::Call => (
				option.base_asset_id,
				TokenizedOptions::call_option_collateral_amount(base_asset_spot_price, &option)
					.unwrap(),
			),
			OptionType::Put => (
				option.quote_asset_id,
				TokenizedOptions::put_option_collateral_amount(base_asset_spot_price, &option)
					.unwrap(),
			),
		};

		let vault_id = AssetToVault::<MockRuntime>::get(asset_id).unwrap();
		let total_collateral = collateral_for_option * total_issuance_buyer;
		let total_shares_amount =
			Vault::amount_of_lp_token_for_added_liquidity(&vault_id, total_collateral).unwrap();

		initial_option_info.insert(
			*option_id,
			(
				base_asset_spot_price,
				total_issuance_buyer,
				collateral_for_option,
				total_shares_amount,
			),
		);
	}

	// Go to exercise window (all the options have expired and have been settled)
	run_to_block(6);

	// Checks post settlement
	for option_id in &option_ids {
		let option = TokenizedOptions::option_id_to_option(*option_id).unwrap();
		let initial_option = initial_option_info.get(option_id).unwrap();

		assert_eq!(option.base_asset_spot_price, initial_option.0);

		assert_eq!(option.total_issuance_buyer, initial_option.1);

		assert_eq!(option.exercise_amount, initial_option.2);

		assert_eq!(option.total_shares_amount, initial_option.3);
	}
}

/// Case checked: one call option with sellers and buyers, ended ITM
#[test]
fn test_settle_options_call_with_initialization_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 10 * UNIT),
			(ALICE, USDC, 500000 * UNIT),
			(BOB, BTC, 10 * UNIT),
			(BOB, USDC, 500000 * UNIT),
			(CHARLIE, BTC, 10 * UNIT),
			(CHARLIE, USDC, 500000 * UNIT),
			(DAVE, BTC, 10 * UNIT),
			(DAVE, USDC, 500000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.execute_with(|| {
			// Get BTC and USDC vault config
			let btc_vault_config = VaultConfigBuilder::default().build();
			let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC).build();

			// Create BTC and USDC vaults
			assert_ok!(TokenizedOptions::create_asset_vault(
				Origin::signed(ADMIN),
				btc_vault_config
			));

			assert_ok!(TokenizedOptions::create_asset_vault(
				Origin::signed(ADMIN),
				usdc_vault_config
			));

			// Create default BTC option
			let option_config =
				OptionsConfigBuilder::default().option_type(OptionType::Call).build();

			assert_ok!(TokenizedOptions::create_option(
				Origin::signed(ADMIN),
				option_config.clone()
			));

			// Make the option goes from NotStarted to Deposit phase
			run_to_block(2);

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			// Check creation ended correctly
			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			// Sell option and make checks
			let alice_option_amount = 5u128;
			let charlie_option_amount = 3u128;

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id
			));

			// Go to purchase window
			run_to_block(3);

			// Buy option
			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id
			));

			// BTC price moves from 50k to 55k, buyers are in profit
			set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

			// Settle options
			settle_options_success_checks();
		});
}

/// Case checked: one call option with seller and buyer, ended ITM
#[test]
fn test_settle_options_put_with_initialization_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 10 * UNIT),
			(ALICE, USDC, 500000 * UNIT),
			(BOB, BTC, 10 * UNIT),
			(BOB, USDC, 500000 * UNIT),
			(CHARLIE, BTC, 10 * UNIT),
			(CHARLIE, USDC, 500000 * UNIT),
			(DAVE, BTC, 10 * UNIT),
			(DAVE, USDC, 500000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.execute_with(|| {
			// Get BTC and USDC vault config
			let btc_vault_config = VaultConfigBuilder::default().build();
			let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC).build();

			// Create BTC and USDC vaults
			assert_ok!(TokenizedOptions::create_asset_vault(
				Origin::signed(ADMIN),
				btc_vault_config
			));

			assert_ok!(TokenizedOptions::create_asset_vault(
				Origin::signed(ADMIN),
				usdc_vault_config
			));

			// Create default BTC option
			let option_config =
				OptionsConfigBuilder::default().option_type(OptionType::Put).build();

			assert_ok!(TokenizedOptions::create_option(
				Origin::signed(ADMIN),
				option_config.clone()
			));

			// Make the option goes from NotStarted to Deposit phase
			run_to_block(2);

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			// Check creation ended correctly
			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			// Sell option and make checks
			let alice_option_amount = 5u128;
			let charlie_option_amount = 3u128;

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id
			));

			// Go to purchase window
			run_to_block(3);

			// Buy option
			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id
			));

			// BTC price moves from 50k to 55k, buyers are in profit
			set_oracle_price(option_config.base_asset_id, 40000u128 * UNIT);

			// Settle options
			settle_options_success_checks();
		});
}

/// Case checked: all the options created
/// Two call options with sellers and buyers, ended ITM.
/// The others ended OTM or with 0 sold.
/// There shouldn't be cases with panic.
#[test]
fn test_settle_options_call_success_multiple_options_sold() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 30 * UNIT),
			(ALICE, USDC, 1500000 * UNIT),
			(BOB, BTC, 30 * UNIT),
			(BOB, USDC, 1500000 * UNIT),
			(CHARLIE, BTC, 30 * UNIT),
			(CHARLIE, USDC, 1500000 * UNIT),
			(DAVE, BTC, 30 * UNIT),
			(DAVE, USDC, 1500000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Create default BTC option
			let option_config_1 =
				OptionsConfigBuilder::default().option_type(OptionType::Call).build();
			let option_config_2 = OptionsConfigBuilder::default()
				.option_type(OptionType::Call)
				.base_asset_strike_price(55000u128 * UNIT)
				.build();

			let option_hash_1 = TokenizedOptions::generate_id(
				option_config_1.base_asset_id,
				option_config_1.quote_asset_id,
				option_config_1.base_asset_strike_price,
				option_config_1.quote_asset_strike_price,
				option_config_1.option_type,
				option_config_1.expiring_date,
				option_config_1.exercise_type,
			);

			let option_hash_2 = TokenizedOptions::generate_id(
				option_config_2.base_asset_id,
				option_config_2.quote_asset_id,
				option_config_2.base_asset_strike_price,
				option_config_2.quote_asset_strike_price,
				option_config_2.option_type,
				option_config_2.expiring_date,
				option_config_2.exercise_type,
			);

			let option_id_1 = OptionHashToOptionId::<MockRuntime>::get(option_hash_1).unwrap();
			let option_id_2 = OptionHashToOptionId::<MockRuntime>::get(option_hash_2).unwrap();

			// Sell option and make checks
			let alice_option_amount = 7u128;
			let bob_option_amount = 9u128;
			let charlie_option_amount = 1u128;
			let dave_option_amount = 15u128;

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_2
			));

			// Go to purchase window
			run_to_block(3);

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_2
			));

			// BTC price moves from 50k to 60k, all buyers are in profit
			set_oracle_price(option_config_1.base_asset_id, 60000u128 * UNIT);

			// Settle options
			settle_options_success_checks();
		});
}

/// Case checked: all the options created
/// Two put options with sellers and buyers, ended ITM.
/// The others ended OTM or with 0 sold.
/// There shouldn't be cases with panic.
#[test]
fn test_settle_options_put_success_multiple_options_sold() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 30 * UNIT),
			(ALICE, USDC, 1500000 * UNIT),
			(BOB, BTC, 30 * UNIT),
			(BOB, USDC, 1500000 * UNIT),
			(CHARLIE, BTC, 30 * UNIT),
			(CHARLIE, USDC, 1500000 * UNIT),
			(DAVE, BTC, 30 * UNIT),
			(DAVE, USDC, 1500000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Create default BTC option
			let option_config_1 =
				OptionsConfigBuilder::default().option_type(OptionType::Put).build();
			let option_config_2 = OptionsConfigBuilder::default()
				.option_type(OptionType::Put)
				.base_asset_strike_price(55000u128 * UNIT)
				.build();

			let option_hash_1 = TokenizedOptions::generate_id(
				option_config_1.base_asset_id,
				option_config_1.quote_asset_id,
				option_config_1.base_asset_strike_price,
				option_config_1.quote_asset_strike_price,
				option_config_1.option_type,
				option_config_1.expiring_date,
				option_config_1.exercise_type,
			);

			let option_hash_2 = TokenizedOptions::generate_id(
				option_config_2.base_asset_id,
				option_config_2.quote_asset_id,
				option_config_2.base_asset_strike_price,
				option_config_2.quote_asset_strike_price,
				option_config_2.option_type,
				option_config_2.expiring_date,
				option_config_2.exercise_type,
			);

			let option_id_1 = OptionHashToOptionId::<MockRuntime>::get(option_hash_1).unwrap();
			let option_id_2 = OptionHashToOptionId::<MockRuntime>::get(option_hash_2).unwrap();

			// Sell option and make checks
			let alice_option_amount = 7u128;
			let bob_option_amount = 9u128;
			let charlie_option_amount = 1u128;
			let dave_option_amount = 15u128;

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_2
			));

			// Go to purchase window
			run_to_block(3);

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_2
			));

			// BTC price moves from 50k to 40k, all buyers are in profit
			set_oracle_price(option_config_1.base_asset_id, 40000u128 * UNIT);

			// Settle options
			settle_options_success_checks();
		});
}

/// Case checked: all the options created
/// Two call options with sellers and buyers, ended ITM, not completely sold.
/// The others ended OTM or with 0 sold.
/// There shouldn't be cases with panic.
#[test]
fn test_settle_options_call_success_multiple_options_not_totally_sold() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 30 * UNIT),
			(ALICE, USDC, 1500000 * UNIT),
			(BOB, BTC, 30 * UNIT),
			(BOB, USDC, 1500000 * UNIT),
			(CHARLIE, BTC, 30 * UNIT),
			(CHARLIE, USDC, 1500000 * UNIT),
			(DAVE, BTC, 30 * UNIT),
			(DAVE, USDC, 1500000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Create default BTC option
			let option_config_1 =
				OptionsConfigBuilder::default().option_type(OptionType::Call).build();
			let option_config_2 = OptionsConfigBuilder::default()
				.option_type(OptionType::Call)
				.base_asset_strike_price(55000u128 * UNIT)
				.build();

			let option_hash_1 = TokenizedOptions::generate_id(
				option_config_1.base_asset_id,
				option_config_1.quote_asset_id,
				option_config_1.base_asset_strike_price,
				option_config_1.quote_asset_strike_price,
				option_config_1.option_type,
				option_config_1.expiring_date,
				option_config_1.exercise_type,
			);

			let option_hash_2 = TokenizedOptions::generate_id(
				option_config_2.base_asset_id,
				option_config_2.quote_asset_id,
				option_config_2.base_asset_strike_price,
				option_config_2.quote_asset_strike_price,
				option_config_2.option_type,
				option_config_2.expiring_date,
				option_config_2.exercise_type,
			);

			let option_id_1 = OptionHashToOptionId::<MockRuntime>::get(option_hash_1).unwrap();
			let option_id_2 = OptionHashToOptionId::<MockRuntime>::get(option_hash_2).unwrap();

			// Sell option and make checks
			let alice_option_amount = 10u128;
			let bob_option_amount = 6u128;
			let charlie_option_amount = 5u128;
			let dave_option_amount = 7u128;

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_2
			));

			// Go to purchase window
			run_to_block(3);

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_2
			));

			// BTC price moves from 50k to 60k, all buyers are in profit
			set_oracle_price(option_config_1.base_asset_id, 60000u128 * UNIT);

			// Settle options
			settle_options_success_checks();
		});
}

/// Case checked: all the options created
/// Two call options with sellers and buyers, ended ITM, not completely sold.
/// The others ended OTM or with 0 sold.
/// There shouldn't be cases with panic.
#[test]
fn test_settle_options_put_success_multiple_options_not_totally_sold() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 30 * UNIT),
			(ALICE, USDC, 1500000 * UNIT),
			(BOB, BTC, 30 * UNIT),
			(BOB, USDC, 1500000 * UNIT),
			(CHARLIE, BTC, 30 * UNIT),
			(CHARLIE, USDC, 1500000 * UNIT),
			(DAVE, BTC, 30 * UNIT),
			(DAVE, USDC, 1500000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Create default BTC option
			let option_config_1 =
				OptionsConfigBuilder::default().option_type(OptionType::Put).build();
			let option_config_2 = OptionsConfigBuilder::default()
				.option_type(OptionType::Put)
				.base_asset_strike_price(55000u128 * UNIT)
				.build();

			let option_hash_1 = TokenizedOptions::generate_id(
				option_config_1.base_asset_id,
				option_config_1.quote_asset_id,
				option_config_1.base_asset_strike_price,
				option_config_1.quote_asset_strike_price,
				option_config_1.option_type,
				option_config_1.expiring_date,
				option_config_1.exercise_type,
			);

			let option_hash_2 = TokenizedOptions::generate_id(
				option_config_2.base_asset_id,
				option_config_2.quote_asset_id,
				option_config_2.base_asset_strike_price,
				option_config_2.quote_asset_strike_price,
				option_config_2.option_type,
				option_config_2.expiring_date,
				option_config_2.exercise_type,
			);

			let option_id_1 = OptionHashToOptionId::<MockRuntime>::get(option_hash_1).unwrap();
			let option_id_2 = OptionHashToOptionId::<MockRuntime>::get(option_hash_2).unwrap();

			// Sell option and make checks
			let alice_option_amount = 10u128;
			let bob_option_amount = 6u128;
			let charlie_option_amount = 5u128;
			let dave_option_amount = 7u128;

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id_2
			));

			// Go to purchase window
			run_to_block(3);

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_1
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id_2
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id_2
			));

			// BTC price moves from 50k to 60k, all buyers are in profit
			set_oracle_price(option_config_1.base_asset_id, 40000u128 * UNIT);

			// Settle options
			settle_options_success_checks();
		});
}

/// Case checked: one call option with sellers and buyers, ended ITM
/// Changing option_amount or asset price causes overflow
#[test]
fn test_settle_options_error_overflow() {
	let exp: u32 = 38;

	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 3 * 10u128.pow(exp)),
			(CHARLIE, USDC, 3 * 10u128.pow(exp)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Create default BTC option
			let option_config =
				OptionsConfigBuilder::default().option_type(OptionType::Call).build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			// Check creation ended correctly
			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			// Balance: u128 contains until ~4 * 10^38. Considering 12 decimals,
			// the asset_amount to transfer should overflow with option amount > 3 * 10^26.
			// It works until 3 * 10^26.
			let alice_option_amount = 3 * 10u128.pow(26);

			// Buyer pays 1k premium for each option (fixed fake amount right now), so 3 exp less
			let charlie_option_amount = 3 * 10u128.pow(23);

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id
			));

			// Go to purchase window
			run_to_block(3);

			// Buy option
			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id
			));

			// BTC price moves from as far as it can
			set_oracle_price(option_config.base_asset_id, 3 * 10u128.pow(38));

			// Settle options
			settle_options_success_checks();
		});
}

/// Case checked: one call option with sellers and buyers, ended ITM
/// Changing option_amount or asset price causes overflow
#[test]
fn test_settle_options_error_overflow_with_value_accrual() {
	let exp: u32 = 38;

	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 3 * 10u128.pow(exp)),
			(CHARLIE, USDC, 3 * 10u128.pow(exp)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Create default BTC option
			let option_config =
				OptionsConfigBuilder::default().option_type(OptionType::Call).build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			// Check creation ended correctly
			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			// Balance: u128 contains until ~4 * 10^38. Considering 12 decimals,
			// the asset_amount to transfer should overflow with option amount > 3 * 10^26.
			// It works until 3 * 10^26.
			let alice_option_amount = 3 * 10u128.pow(26);

			// Buyer pays 1k premium for each option (fixed fake amount right now), so 3 exp less
			let charlie_option_amount = 3 * 10u128.pow(23);

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id
			));

			// Go to purchase window
			run_to_block(3);

			// Buy option
			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id
			));

			// Add as most BTC as I can to the vault to simulate vault value accrual
			// In this case, too much value accrual and too much price shift cause the
			// function `call_option_collateral_amount` to not be able to calculate `asset_amount`
			// correctly, returning 1*UNIT as an approximation. So `shares_amount` is calculated
			// incorrectly and this cause the withdraw of more funds than needed (because of value
			// accrual), which actually should belong to sellers since value accrued while
			// collateral is deposited is theirs. Of course this should never be the case since the
			// value accrual and the price shift should never be so high
			let vault_id = AssetToVault::<MockRuntime>::get(option_config.base_asset_id).unwrap();
			let vault_account = Vault::account_id(&vault_id);
			assert_ok!(Assets::mint_into(
				Origin::signed(ADMIN),
				option_config.base_asset_id,
				vault_account,
				3 * 10u128.pow(37),
			));

			// BTC price moves from as far as it can
			set_oracle_price(option_config.base_asset_id, 3 * 10u128.pow(38));

			// Settle options
			let option = OptionIdToOption::<MockRuntime>::get(option_id).unwrap();
			let protocol_account = TokenizedOptions::account_id(option.base_asset_id);

			run_to_block(6);
			let updated_protocol_balance = Assets::balance(option.base_asset_id, &protocol_account);

			assert!(updated_protocol_balance == 300000000000000000000000000000000000u128);
		});
}
