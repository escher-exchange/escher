use crate::mocks::runtime::{
    Assets, Balance, Event, ExtBuilder, MockRuntime, Origin, System, TokenizedOptions,
};

use crate::mocks::{accounts::*, assets::*};

use crate::{
    pallet::{self, OptionHashToOptionId},
    tests::{sell_option::sell_option_success_checks, *},
};

use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect};

use sp_core::sr25519::Public;
use sp_runtime::ArithmeticError;

// ----------------------------------------------------------------------------------------------------
//		Buy Options Tests
// ----------------------------------------------------------------------------------------------------

pub fn buy_option_success_checks(option_id: AssetId, option_amount: Balance, who: Public) {
    let option = OptionIdToOption::<MockRuntime>::get(option_id).unwrap();

    let asset_id = USDC;
    let option_premium = TokenizedOptions::fake_option_price() * option_amount;

    // ---------------------------
    // |  Data before extrinsic  |
    // ---------------------------
    let protocol_account = TokenizedOptions::account_id(asset_id);
    let initial_issuance_buyer = Assets::total_issuance(option_id);
    let initial_premium_paid = option.total_premium_paid;
    let initial_user_balance_options = Assets::balance(option_id, &who);
    let initial_user_balance = Assets::balance(asset_id, &who);
    let initial_protocol_balance = Assets::balance(asset_id, &protocol_account);

    // Call extrinsic and check event
    assert_ok!(TokenizedOptions::buy_option(
        Origin::signed(who),
        option_amount,
        option_id
    ));

    System::assert_last_event(Event::TokenizedOptions(pallet::Event::BuyOption {
        user: who,
        option_amount,
        option_id,
    }));

    // ---------------------------
    // |  Data after extrinsic  |
    // ---------------------------
    // Check buyer balance after sale has premium subtracted
    assert_eq!(
        Assets::balance(asset_id, &who),
        initial_user_balance - option_premium
    );

    // Check protocol balance after purchase is correct
    assert_eq!(
        Assets::balance(asset_id, &protocol_account),
        initial_protocol_balance + option_premium
    );

    // Check user owns the correct issuance of option token
    assert_eq!(
        Assets::balance(option_id, &who),
        initial_user_balance_options + option_amount
    );

    // Check position is updated correctly
    let updated_issuance_buyer = Assets::total_issuance(option_id);
    assert_eq!(
        updated_issuance_buyer,
        initial_issuance_buyer + option_amount
    );

    let update_premium_paid = OptionIdToOption::<MockRuntime>::get(option_id)
        .unwrap()
        .total_premium_paid;

    // Check premium is updated correctly
    assert_eq!(update_premium_paid, initial_premium_paid + option_premium);
}

#[test]
fn test_buy_option_with_initialization_success() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 1 * UNIT),
            (ALICE, USDC, 50000 * UNIT),
            (BOB, BTC, 1 * UNIT),
            (BOB, USDC, 50000 * UNIT),
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
            let option_config = OptionsConfigBuilder::default().build();

            assert_ok!(TokenizedOptions::create_option(
                Origin::signed(ADMIN),
                option_config.clone()
            ));

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
            assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
                option_hash
            ));

            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Make the option goes from NotStarted to Deposit phase
            run_to_block(2);

            // Sell option and make checks
            let option_amount = 1u128;
            sell_option_success_checks(option_id, option_amount, BOB);

            // Go to purchase window
            run_to_block(3);

            // Buy option
            buy_option_success_checks(option_id, option_amount, ALICE);
        });
}

#[test]
fn test_buy_option_success() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 3 * UNIT),
            (ALICE, USDC, 150000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
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

            let bob_option_amount = 5u128;
            let alice_option_amount = 3u128;

            sell_option_success_checks(option_id, bob_option_amount, BOB);
            run_to_block(3);
            buy_option_success_checks(option_id, alice_option_amount, ALICE);
        });
}

#[test]
fn test_buy_option_multiple_times() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 5 * UNIT),
            (ALICE, USDC, 250000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
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

            let bob_option_amount = 5u128;
            let alice_option_amount = 3u128;

            sell_option_success_checks(option_id, bob_option_amount, BOB);

            run_to_block(3);

            buy_option_success_checks(option_id, alice_option_amount, ALICE);

            let alice_option_amount = 2u128;
            buy_option_success_checks(option_id, alice_option_amount, ALICE);
        });
}

#[test]
fn test_buy_option_multiple_users() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 5 * UNIT),
            (ALICE, USDC, 250000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
            (CHARLIE, BTC, 5 * UNIT),
            (CHARLIE, USDC, 250000 * UNIT),
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

            let bob_option_amount = 5u128;
            let alice_option_amount = 3u128;
            let charlie_option_amount = 4u128;

            sell_option_success_checks(option_id, bob_option_amount, BOB);
            sell_option_success_checks(option_id, charlie_option_amount, CHARLIE);

            run_to_block(3);

            buy_option_success_checks(option_id, alice_option_amount, ALICE);

            let charlie_option_amount = 2u128;

            buy_option_success_checks(option_id, charlie_option_amount, CHARLIE);
            let alice_option_amount = 2u128;
            buy_option_success_checks(option_id, alice_option_amount, ALICE);
        });
}

#[test]
fn test_buy_option_error_option_not_exists() {
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
                TokenizedOptions::buy_option(Origin::signed(BOB), 1u128, 10000000000005u128),
                Error::<MockRuntime>::OptionDoesNotExists
            );
        });
}

// #[test]
// fn test_buy_option_error_not_into_purchase_window() {
//     ExtBuilder::default()
//         .initialize_balances(Vec::from([
//             (ALICE, BTC, 5 * UNIT),
//             (ALICE, USDC, 250000 * UNIT),
//             (BOB, BTC, 5 * UNIT),
//             (BOB, USDC, 250000 * UNIT),
//         ]))
//         .build()
//         .initialize_oracle_prices()
//         .initialize_all_vaults()
//         .initialize_all_options()
//         .execute_with(|| {
//             let option_config = OptionsConfigBuilder::default().build();

//             let option_hash = TokenizedOptions::generate_id(
//                 option_config.base_asset_id,
//                 option_config.quote_asset_id,
//                 option_config.base_asset_strike_price,
//                 option_config.quote_asset_strike_price,
//                 option_config.option_type,
//                 option_config.expiring_date,
//                 option_config.exercise_type,
//             );

//             assert!(OptionHashToOptionId::<MockRuntime>::contains_key(
//                 option_hash
//             ));
//             let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

//             let bob_option_amount = 5u128;
//             let alice_option_amount = 2u128;

//             sell_option_success_checks(option_id, bob_option_amount, BOB);

//             // Purchase window goes from block 3 <= x < 6. Now we are in block 3.
//             let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

//             assert_noop!(
//                 TokenizedOptions::buy_option(Origin::signed(ALICE), alice_option_amount,
// option_id),                 Error::<MockRuntime>::NotIntoPurchaseWindow
//             );

//             // Now it should work
//             run_to_block(3);
//             buy_option_success_checks(option_id, alice_option_amount, ALICE);

//             // Now we are out of purchase window again and should fail
//             run_to_block(6);
//             assert_noop!(
//                 TokenizedOptions::buy_option(Origin::signed(ALICE), alice_option_amount,
// option_id),                 Error::<MockRuntime>::NotIntoPurchaseWindow
//             );
//         });
// }

#[test]
fn test_buy_option_error_user_has_not_enough_funds() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 3 * UNIT),
            (ALICE, USDC, 3000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
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

            let bob_option_amount = 5u128;
            sell_option_success_checks(option_id, bob_option_amount, BOB);

            run_to_block(3);

            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            let alice_option_amount = 4u128; // Each option costs 1000 USDC, Alice has 3000

            assert_noop!(
                TokenizedOptions::buy_option(Origin::signed(ALICE), alice_option_amount, option_id),
                Error::<MockRuntime>::UserHasNotEnoughFundsToDeposit
            );

            let alice_option_amount = 3u128; // Each option costs 1000 USDC, Alice has 3000

            // Counter test
            buy_option_success_checks(option_id, alice_option_amount, ALICE);
        });
}

#[test]
fn test_buy_option_error_cannot_buy_zero_options() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 5 * UNIT),
            (ALICE, USDC, 250000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
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

            let bob_option_amount = 5u128;
            sell_option_success_checks(option_id, bob_option_amount, BOB);

            run_to_block(3);

            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            assert_noop!(
                TokenizedOptions::buy_option(Origin::signed(ALICE), 0u128, option_id),
                Error::<MockRuntime>::CannotPassZeroOptionAmount
            );
        });
}

#[test]
fn test_buy_option_error_overflow_asset_amount() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 5 * UNIT),
            (ALICE, USDC, 250000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
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

            let bob_option_amount = 5u128;
            sell_option_success_checks(option_id, bob_option_amount, BOB);

            run_to_block(3);

            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            // Balance: u128 contains until ~4 * 10^38. Considering 12 decimals,
            // the asset_amount to transfer should overflow with option amount > 3 * 10^26.
            // The fake option cost right now is fixed at 1000 USDC, so
            // option amount should be > 3 * 10^23 to cause overflow.
            // It works until 3 * 10^23.
            let alice_option_amount = 4 * 10u128.pow(23);

            assert_noop!(
                TokenizedOptions::buy_option(Origin::signed(ALICE), alice_option_amount, option_id),
                ArithmeticError::Overflow
            );
        });
}

#[test]
fn test_buy_option_error_not_enough_options_for_sale() {
    ExtBuilder::default()
        .initialize_balances(Vec::from([
            (ALICE, BTC, 5 * UNIT),
            (ALICE, USDC, 250000 * UNIT),
            (BOB, BTC, 5 * UNIT),
            (BOB, USDC, 250000 * UNIT),
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

            let bob_option_amount = 5u128;
            let alice_option_amount = 3u128;

            sell_option_success_checks(option_id, bob_option_amount, BOB);

            run_to_block(3);

            buy_option_success_checks(option_id, alice_option_amount, ALICE);

            let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

            assert_noop!(
                TokenizedOptions::buy_option(Origin::signed(ALICE), alice_option_amount, option_id),
                Error::<MockRuntime>::NotEnoughOptionsForSale
            );
        });
}
