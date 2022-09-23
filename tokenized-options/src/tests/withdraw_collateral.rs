use crate::mocks::runtime::{
    Assets, Event, ExtBuilder, MockRuntime, Origin, System, TokenizedOptions, Vault,
};

use crate::mocks::{accounts::*, assets::*};
use sp_std::cmp::min;

use crate::{
    pallet::{self, OptionHashToOptionId, Sellers},
    tests::*,
};

use composable_traits::vault::Vault as VaultTrait;
use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect};

use sp_core::sr25519::Public;

// ----------------------------------------------------------------------------------------------------
//		Withdraw Collateral Tests
// ----------------------------------------------------------------------------------------------------
pub fn withdraw_collateral_success_checks(option_id: AssetId, who: Public) {
    // -------------------------------------------------------
    // |  These tests assume quote_asset_id = stablecoin_id  |
    // -------------------------------------------------------
    let option = OptionIdToOption::<MockRuntime>::get(option_id).unwrap();
    let initial_user_position = Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or_default();

    // Different behaviors based on Call or Put option
    let asset_id = match option.option_type {
        OptionType::Call => option.base_asset_id,
        OptionType::Put => option.quote_asset_id,
    };

    let stablecoin_id = USDC;

    // ---------------------------
    // |  Data before extrinsic  |
    // ---------------------------
    let vault_id = AssetToVault::<MockRuntime>::get(asset_id).unwrap();

    let initial_user_balance = Assets::balance(asset_id, &who);
    let initial_vault_balance = Assets::balance(asset_id, &Vault::account_id(&vault_id));
    let protocol_account = TokenizedOptions::account_id(asset_id);
    let protocol_account_stablecoin = TokenizedOptions::account_id(stablecoin_id);
    let initial_user_balance_stablecoin = Assets::balance(stablecoin_id, &who);
    let initial_protocol_account_balance_stablecoin =
        Assets::balance(stablecoin_id, &protocol_account_stablecoin);

    // Calculate user's shares and asset amount to receive
    let shares_for_buyers = option.total_shares_amount * initial_user_position.option_amount
        / option.total_issuance_seller;
    let user_shares_amount = initial_user_position.shares_amount - shares_for_buyers;
    let lp_token_issuance =
        Assets::balance(Vault::lp_asset_id(&vault_id).unwrap(), &protocol_account);
    let user_shares_amount = min(user_shares_amount, lp_token_issuance);
    let asset_amount = Vault::lp_share_value(&vault_id, user_shares_amount).unwrap();

    // Calculate user premium amount to receive
    let user_premium_amount = option.total_premium_paid * initial_user_position.option_amount
        / option.total_issuance_seller;

    // Call extrinsic and check event
    assert_ok!(TokenizedOptions::withdraw_collateral(
        Origin::signed(who),
        option_id
    ));

    // Check correct event
    System::assert_last_event(Event::TokenizedOptions(pallet::Event::WithdrawCollateral {
        user: who,
        option_id,
    }));

    // --------------------------
    // |  Data after extrinsic  |
    // --------------------------

    // Check seller position has been deleted
    assert!(!Sellers::<MockRuntime>::contains_key(option_id, who));

    // // Check vault balance after withdraw is correct
    assert_eq!(
        Assets::balance(asset_id, &Vault::account_id(&vault_id)),
        initial_vault_balance - asset_amount
    );

    match option.option_type {
        OptionType::Call => {
            // Check seller balance after sale is correct
            assert_eq!(
                Assets::balance(asset_id, &who),
                initial_user_balance + asset_amount
            );
            assert_eq!(
                Assets::balance(stablecoin_id, &protocol_account_stablecoin),
                initial_protocol_account_balance_stablecoin - user_premium_amount
            );
            // Check seller stablecoin balance after sale is correct
            assert_eq!(
                Assets::balance(stablecoin_id, &who),
                initial_user_balance_stablecoin + user_premium_amount
            );
        },

        OptionType::Put => {
            assert_eq!(
                Assets::balance(asset_id, &who),
                initial_user_balance + asset_amount + user_premium_amount
            );
            assert_eq!(
                Assets::balance(stablecoin_id, &protocol_account_stablecoin),
                initial_protocol_account_balance_stablecoin - user_premium_amount
            );
        },
    };
}

#[test]
fn test_withdraw_collateral_call_with_initialization_success() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 10 * UNIT),
            (ALICE, USDC, 500000 * UNIT),
            (CHARLIE, BTC, 10 * UNIT),
            (CHARLIE, USDC, 500000 * UNIT),
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
            let charlie_option_amount = 5u128;

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

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(CHARLIE),
                charlie_option_amount,
                option_id
            ));

            withdraw_collateral_success_checks(option_id, ALICE);
        });
}

#[test]
fn test_withdraw_collateral_put_with_initialization_success() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 10 * UNIT),
            (ALICE, USDC, 500000 * UNIT),
            (CHARLIE, BTC, 10 * UNIT),
            (CHARLIE, USDC, 500000 * UNIT),
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
            let charlie_option_amount = 5u128;

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

            // Go to exercise window (option has expired so settlement can start)
            run_to_block(6);

            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(CHARLIE),
                charlie_option_amount,
                option_id
            ));

            withdraw_collateral_success_checks(option_id, ALICE);
        });
}

#[test]
fn test_withdraw_collateral_call_multiple_times() {
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
            // Create default BTC option
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

            // Check creation ended correctly
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

            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(CHARLIE),
                charlie_option_amount,
                option_id
            ));
            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            withdraw_collateral_success_checks(option_id, ALICE);
            withdraw_collateral_success_checks(option_id, BOB);
        });
}

#[test]
fn test_withdraw_collateral_put_multiple_times() {
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
            // Create default BTC option
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

            // Check creation ended correctly
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

            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(CHARLIE),
                charlie_option_amount,
                option_id
            ));
            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            withdraw_collateral_success_checks(option_id, ALICE);
            withdraw_collateral_success_checks(option_id, BOB);
        });
}

#[test]
fn test_withdraw_collateral_error_option_not_exists() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([(BOB, BTC, 1 * UNIT), (BOB, USDC, 50000 * UNIT)]))
        .build()
        .execute_with(|| {
            assert_noop!(
                // 10000000000005u128 it's a meaningless number
                TokenizedOptions::withdraw_collateral(Origin::signed(BOB), 10000000000005u128),
                Error::<MockRuntime>::OptionDoesNotExists
            );
        });
}

#[test]
fn test_withdraw_collateral_error_not_into_exercise_window() {
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
            // Create default BTC option
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

            // Check creation ended correctly
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

            // Not yet exercise phase (block 6)
            run_to_block(5);

            assert_noop!(
                TokenizedOptions::withdraw_collateral(Origin::signed(ALICE), option_id),
                Error::<MockRuntime>::NotIntoExerciseWindow
            );

            assert_noop!(
                TokenizedOptions::withdraw_collateral(Origin::signed(BOB), option_id),
                Error::<MockRuntime>::NotIntoExerciseWindow
            );
        });
}

#[test]
fn test_withdraw_collateral_error_user_does_not_have_position() {
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
            // Create default BTC option
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

            // Check creation ended correctly
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

            // Not yet exercise phase (block 6)
            run_to_block(6);

            assert_noop!(
                TokenizedOptions::withdraw_collateral(Origin::signed(CHARLIE), option_id),
                Error::<MockRuntime>::UserDoesNotHaveSellerPosition
            );

            withdraw_collateral_success_checks(option_id, ALICE);
        });
}

#[test]
fn test_withdraw_collateral_call_out_of_money_multiple_times() {
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
            // Create default BTC option
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

            // Check creation ended correctly
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

            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(CHARLIE),
                charlie_option_amount,
                option_id
            ));
            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            withdraw_collateral_success_checks(option_id, ALICE);
            withdraw_collateral_success_checks(option_id, BOB);
        });
}

#[test]
fn test_withdraw_collateral_put_out_of_money_multiple_times() {
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
            // Create default BTC option
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

            // Check creation ended correctly
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

            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(CHARLIE),
                charlie_option_amount,
                option_id
            ));
            assert_ok!(TokenizedOptions::exercise_option(
                Origin::signed(DAVE),
                dave_option_amount,
                option_id
            ));

            withdraw_collateral_success_checks(option_id, ALICE);
            withdraw_collateral_success_checks(option_id, BOB);
        });
}

#[test]
fn test_withdraw_collateral_dust_issue() {
    ExtBuilder::default()
        .build()
        .initialize_oracle_prices()
        .initialize_all_vaults()
        .initialize_all_options()
        .execute_with(|| {
            // Create default BTC option
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

            // Check creation ended correctly
            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            let user_number = 500;
            let seller_option_number: u128 = 61;
            let buyer_option_number: u128 = 31;

            for i in 1..user_number {
                Assets::mint_into(
                    Origin::root(),
                    BTC,
                    account_id_from_u64(i),
                    seller_option_number * UNIT,
                )
                .unwrap();

                assert_ok!(TokenizedOptions::sell_option(
                    Origin::signed(account_id_from_u64(i)),
                    seller_option_number,
                    option_id
                ));
            }

            // Go to purchase window
            run_to_block(3);

            for i in 1..user_number {
                Assets::mint_into(
                    Origin::root(),
                    USDC,
                    account_id_from_u64(i),
                    buyer_option_number * 1000 * UNIT,
                )
                .unwrap();

                assert_ok!(TokenizedOptions::buy_option(
                    Origin::signed(account_id_from_u64(i)),
                    buyer_option_number,
                    option_id
                ));
            }

            // BTC price moves from 50k to 55k, buyers are in profit
            set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

            // Go to exercise window (option has expired so settlement can start)
            // Before settlement, simulate a gain or loss for the vault
            let vault_id = AssetToVault::<MockRuntime>::get(BTC).unwrap();
            // assert_ok!(Assets::mint_into(Origin::root(), BTC, Vault::account_id(&vault_id),
            // 1101113u128 * UNIT));
            assert_ok!(Assets::burn_from(
                Origin::root(),
                BTC,
                Vault::account_id(&vault_id),
                1000u128 * UNIT
            ));

            run_to_block(6);

            for i in 1..user_number {
                if i == user_number - 1 {
                    // Get last user position to compare how many shares is going to lose for
                    // accumulated dust
                    let initial_user_position =
                        Sellers::<MockRuntime>::try_get(option_id, account_id_from_u64(i)).unwrap();
                    let option = OptionIdToOption::<MockRuntime>::try_get(option_id).unwrap();

                    let shares_for_buyers = option.total_shares_amount
                        * initial_user_position.option_amount
                        / option.total_issuance_seller;

                    let user_shares = initial_user_position.shares_amount - shares_for_buyers;

                    let protocol_account = TokenizedOptions::account_id(BTC);
                    let lp_token_issuance =
                        Assets::balance(Vault::lp_asset_id(&vault_id).unwrap(), &protocol_account);

                    if user_shares > lp_token_issuance {
                        println!("user_shares: {:?}", user_shares - lp_token_issuance);
                        assert!(user_shares - lp_token_issuance < 1000);
                    } else {
                        println!("lp_tokens: {:?}", lp_token_issuance - user_shares);
                        assert!(lp_token_issuance - user_shares < 1000);
                    }
                }

                assert_ok!(TokenizedOptions::withdraw_collateral(
                    Origin::signed(account_id_from_u64(i)),
                    option_id
                ));
            }

            let v = Assets::balance(BTC, &Vault::account_id(&vault_id));

            assert_eq!(v, 0);
        });
}
