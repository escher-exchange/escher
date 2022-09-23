use crate::mocks::runtime::{
    Assets, Balance, Event, ExtBuilder, MockRuntime, Origin, System, TokenizedOptions,
};

use crate::mocks::{accounts::*, assets::*};

use crate::{
    pallet::{self, OptionHashToOptionId},
    tests::*,
};
use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect};

use sp_core::sr25519::Public;
use sp_runtime::ArithmeticError;

// ----------------------------------------------------------------------------------------------------
//		Exercise Options Tests
// ----------------------------------------------------------------------------------------------------
pub fn exercise_option_success_checks(option_id: AssetId, option_amount: Balance, who: Public) {
    let option = OptionIdToOption::<MockRuntime>::get(option_id).unwrap();

    // Different behaviors based on Call or Put option
    let asset_id = match option.option_type {
        OptionType::Call => option.base_asset_id,
        OptionType::Put => option.quote_asset_id,
    };
    // ---------------------------
    // |  Data before extrinsic  |
    // ---------------------------
    let protocol_account = TokenizedOptions::account_id(asset_id);
    let initial_total_issuance = Assets::total_issuance(option_id);
    let initial_user_balance_options = Assets::balance(option_id, &who);
    let initial_user_balance = Assets::balance(asset_id, &who);
    let initial_protocol_balance = Assets::balance(asset_id, &protocol_account);

    // Call extrinsic and check event
    assert_ok!(TokenizedOptions::exercise_option(
        Origin::signed(who),
        option_amount,
        option_id
    ));

    // Check correct event
    System::assert_last_event(Event::TokenizedOptions(pallet::Event::ExerciseOption {
        user: who,
        option_amount,
        option_id,
    }));

    // ---------------------------
    // |  Data after extrinsic  |
    // ---------------------------
    // Check buyer balance after exercise has increased
    assert_eq!(
        Assets::balance(asset_id, &who),
        initial_user_balance + option.exercise_amount * option_amount
    );

    // Check protocol balance after exercise is correct
    assert_eq!(
        Assets::balance(asset_id, &protocol_account),
        initial_protocol_balance - option.exercise_amount * option_amount
    );

    // Check user owns the correct issuance of option token
    assert_eq!(
        Assets::balance(option_id, &who),
        initial_user_balance_options - option_amount
    );

    // Check position is updated correctly
    let updated_total_issuance = Assets::total_issuance(option_id);
    assert_eq!(
        updated_total_issuance,
        initial_total_issuance - option_amount
    );
}

#[test]
fn test_exercise_option_call_with_initialization_success() {
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
            let option_config = OptionsConfigBuilder::default()
                .option_type(OptionType::Call)
                .build();

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
            let bob_option_amount = 4u128;
            let charlie_option_amount = 3u128;
            let dave_option_amount = 6u128;

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(ALICE),
                alice_option_amount,
                option_id
            ));

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(BOB),
                bob_option_amount,
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

            assert_ok!(TokenizedOptions::buy_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            // BTC price moves from 50k to 55k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            exercise_option_success_checks(option_id, charlie_option_amount, CHARLIE);
            exercise_option_success_checks(option_id, dave_option_amount, DAVE);

            // Check position is updated correctly
            let updated_total_issuance = Assets::total_issuance(option_id);
            assert_eq!(updated_total_issuance, 0u128);

            // Check protocol balance after exercise is correct
            let protocol_account = TokenizedOptions::account_id(BTC);
            assert_eq!(Assets::balance(BTC, &protocol_account), 0u128);
        });
}

#[test]
fn test_exercise_option_put_with_initialization_success() {
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
            let option_config = OptionsConfigBuilder::default()
                .option_type(OptionType::Put)
                .build();

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
            let bob_option_amount = 4u128;
            let charlie_option_amount = 3u128;
            let dave_option_amount = 6u128;

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(ALICE),
                alice_option_amount,
                option_id
            ));

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(BOB),
                bob_option_amount,
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

            assert_ok!(TokenizedOptions::buy_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            // BTC price moves from 50k to 55k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 45000u128 * UNIT);

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            exercise_option_success_checks(option_id, charlie_option_amount, CHARLIE);
            exercise_option_success_checks(option_id, dave_option_amount, DAVE);

            // Check position is updated correctly
            let updated_total_issuance = Assets::total_issuance(option_id);
            assert_eq!(updated_total_issuance, 0u128);

            // Check protocol balance after exercise is correct
            let protocol_account = TokenizedOptions::account_id(BTC);
            assert_eq!(Assets::balance(BTC, &protocol_account), 0u128);
        });
}

#[test]
fn test_exercise_option_call_success() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default()
                .option_type(OptionType::Call)
                .build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let bob_option_amount = 8u128;
            let charlie_option_amount = 4u128;
            let dave_option_amount = 7u128;

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(ALICE),
                alice_option_amount,
                option_id
            ));

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(BOB),
                bob_option_amount,
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

            assert_ok!(TokenizedOptions::buy_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            // BTC price moves from 50k to 55k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            exercise_option_success_checks(option_id, charlie_option_amount, CHARLIE);
            exercise_option_success_checks(option_id, dave_option_amount, DAVE);

            assert_eq!(Assets::total_issuance(option_id), 0u128);

            // Check protocol balance after exercise is correct
            let protocol_account = TokenizedOptions::account_id(BTC);
            assert_eq!(Assets::balance(BTC, &protocol_account), 0u128);
        });
}

#[test]
fn test_exercise_option_put_success() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default()
                .option_type(OptionType::Put)
                .build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let bob_option_amount = 8u128;
            let charlie_option_amount = 4u128;
            let dave_option_amount = 7u128;

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(ALICE),
                alice_option_amount,
                option_id
            ));

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(BOB),
                bob_option_amount,
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

            assert_ok!(TokenizedOptions::buy_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            // BTC price moves from 50k to 55k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 45000u128 * UNIT);

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            exercise_option_success_checks(option_id, charlie_option_amount, CHARLIE);
            exercise_option_success_checks(option_id, dave_option_amount, DAVE);

            assert_eq!(Assets::total_issuance(option_id), 0u128);

            // Check protocol balance after exercise is correct (11 premium should be here)
            let protocol_account = TokenizedOptions::account_id(USDC);
            assert_eq!(Assets::balance(USDC, &protocol_account), 11000u128 * UNIT);
        });
}

#[test]
fn test_exercise_option_call_multiple_times() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default()
                .option_type(OptionType::Call)
                .build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let bob_option_amount = 8u128;
            let charlie_option_amount = 4u128;
            let dave_option_amount = 7u128;

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(ALICE),
                alice_option_amount,
                option_id
            ));

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(BOB),
                bob_option_amount,
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

            assert_ok!(TokenizedOptions::buy_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            // BTC price moves from 50k to 55k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            exercise_option_success_checks(option_id, 3u128, CHARLIE);
            exercise_option_success_checks(option_id, 1u128, CHARLIE);
            exercise_option_success_checks(option_id, 1u128, DAVE);
            exercise_option_success_checks(option_id, 6u128, DAVE);

            assert_eq!(Assets::total_issuance(option_id), 0u128);

            // Check protocol balance after exercise is correct
            let protocol_account = TokenizedOptions::account_id(BTC);
            assert_eq!(Assets::balance(BTC, &protocol_account), 0u128);
        });
}

#[test]
fn test_exercise_option_put_multiple_times() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default()
                .option_type(OptionType::Put)
                .build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let bob_option_amount = 8u128;
            let charlie_option_amount = 4u128;
            let dave_option_amount = 7u128;

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(ALICE),
                alice_option_amount,
                option_id
            ));

            assert_ok!(TokenizedOptions::sell_option(
                Origin::signed(BOB),
                bob_option_amount,
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

            assert_ok!(TokenizedOptions::buy_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            // BTC price moves from 50k to 55k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            exercise_option_success_checks(option_id, 3u128, CHARLIE);
            exercise_option_success_checks(option_id, 1u128, CHARLIE);
            exercise_option_success_checks(option_id, 1u128, DAVE);
            exercise_option_success_checks(option_id, 6u128, DAVE);

            assert_eq!(Assets::total_issuance(option_id), 0u128);

            // Check protocol balance after exercise is correct
            let protocol_account = TokenizedOptions::account_id(USDC);
            assert_eq!(Assets::balance(USDC, &protocol_account), 11000u128 * UNIT);
        });
}

#[test]
fn test_exercise_option_out_of_money_tokens_burned() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default().build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let charlie_option_amount = 4u128;

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

            // BTC price moves from 50k to 45k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 45000u128 * UNIT);

            // Not yet exercise phase (starts at 6th block)
            run_to_block(6);

            exercise_option_success_checks(option_id, charlie_option_amount, CHARLIE);
        });
}

#[test]
fn test_exercise_option_error_option_not_exists() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 5 * UNIT),
            (ALICE, USDC, 250000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
        ]))
        .build()
        .execute_with(|| {
            assert_noop!(
                // 10000000000005u128 it's a meaningless number
                TokenizedOptions::exercise_option(Origin::signed(BOB), 1u128, 10000000000005u128),
                Error::<MockRuntime>::OptionDoesNotExists
            );
        });
}

#[test]
fn test_exercise_option_error_not_into_exercise_window() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default().build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let charlie_option_amount = 4u128;

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

            // Not yet exercise phase (starts at 6th block)
            run_to_block(5);

            assert_noop!(
                TokenizedOptions::exercise_option(
                    Origin::signed(CHARLIE),
                    charlie_option_amount,
                    option_id
                ),
                Error::<MockRuntime>::NotIntoExerciseWindow
            );
        });
}

#[test]
fn test_exercise_option_error_cannot_exercise_zero_options() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default().build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let charlie_option_amount = 4u128;

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

            // Not yet exercise phase (starts at 6th block)
            run_to_block(6);

            assert_noop!(
                TokenizedOptions::exercise_option(Origin::signed(CHARLIE), 0u128, option_id),
                Error::<MockRuntime>::CannotPassZeroOptionAmount
            );
        });
}

#[test]
fn test_exercise_option_error_cannot_exercise_too_much_options() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default().build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let charlie_option_amount = 4u128;

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

            // BTC price moves from 50k to 45k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 45000u128 * UNIT);

            // Not yet exercise phase (starts at 6th block)
            run_to_block(6);

            assert_noop!(
                TokenizedOptions::exercise_option(Origin::signed(CHARLIE), 5u128, option_id),
                Error::<MockRuntime>::UserHasNotEnoughOptionTokens
            );
        });
}

#[test]
fn test_exercise_option_error_overflow() {
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
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            let option_config = OptionsConfigBuilder::default()
                .option_type(OptionType::Call)
                .build();

            let option_hash = TokenizedOptions::generate_id(
                option_config.base_asset_id,
                option_config.quote_asset_id,
                option_config.base_asset_strike_price,
                option_config.quote_asset_strike_price,
                option_config.option_type,
                option_config.expiring_date,
                option_config.exercise_type,
            );

            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Sell option and make checks
            let alice_option_amount = 10u128;
            let charlie_option_amount = 4u128;

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

            // Not yet exercise phase (starts at 6th block)
            run_to_block(6);

            assert_noop!(
                TokenizedOptions::exercise_option(
                    Origin::signed(CHARLIE),
                    3 * 10u128.pow(28),
                    option_id
                ),
                ArithmeticError::Overflow
            );
        });
}
