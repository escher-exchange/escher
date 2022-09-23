use crate::mocks::runtime::{
	Assets, Balance, ExtBuilder, MockRuntime, Moment, Origin, TokenizedOptions, Vault,
};

use crate::mocks::{accounts::*, assets::*};

use crate::{pallet::OptionHashToOptionId, tests::*};

use composable_traits::vault::Vault as VaultTrait;
use frame_support::{assert_ok, traits::fungibles::Inspect};

// ----------------------------------------------------------------------------------------------------
//		Epoch Tests
// ----------------------------------------------------------------------------------------------------
fn generate_option_and_get_id(
	option_type: OptionType,
	strike_price: Balance,
	epoch: Epoch<Moment>,
) -> AssetId {
	let option_config = OptionsConfigBuilder::default()
		.option_type(option_type)
		.base_asset_strike_price(strike_price)
		.epoch(epoch)
		.build();

	assert_ok!(TokenizedOptions::create_option(Origin::signed(ADMIN), option_config.clone()));

	let option_hash = TokenizedOptions::generate_id(
		option_config.base_asset_id,
		option_config.quote_asset_id,
		option_config.base_asset_strike_price,
		option_config.quote_asset_strike_price,
		option_config.option_type,
		option_config.expiring_date,
		option_config.exercise_type,
	);

	OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap()
}

#[test]
fn test_epoch_basic_example_totally_sold() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 2 * UNIT),
			(BOB, BTC, 3 * UNIT),
			(CHARLIE, USDC, 5000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.execute_with(|| {
			// BTC initial price: 50k

			// BTC CALL 50k strike - expire date 6000
			let option_id1 = generate_option_and_get_id(
				OptionType::Call,
				50000u128 * UNIT,
				// Use this when https://github.com/paritytech/substrate/pull/10128 is merged
				// Epoch { deposit: 0u64, purchase: 3000u64, exercise: 6000u64, end: 9000u64 },
				Epoch { deposit: 0u64, purchase: 2000u64, exercise: 5000u64, end: 9000u64 },
			);

			// Make the option goes from NotStarted to Deposit phase
			run_to_block(2);

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(ALICE), 2u128, option_id1));

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(BOB), 3u128, option_id1));

			run_to_block(3);

			assert_ok!(TokenizedOptions::buy_option(Origin::signed(CHARLIE), 5u128, option_id1));

			set_oracle_price(BTC, 55000u128 * UNIT);

			run_to_block(6);

			assert_ok!(TokenizedOptions::exercise_option(
				Origin::signed(CHARLIE),
				5u128,
				option_id1
			));

			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(ALICE), option_id1));

			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(BOB), option_id1));

			// ---------------
			//  FINAL CHECKS |
			// ---------------
			// For each option, (55k - 50k)/55k = 0.090909 BTC of collateral deposited by ALICE and
			// BOB should be given to CHARLIE since all the options for sale have been bought.
			// CHARLIE should end with 0.454545454545 BTC, ALICE with 1.818181818182 BTC and BOB
			// with 2.727272727273 BTC CHARLIE USDC balance should go to 0 given that each option
			// costs 1000 USDC. ALICE should receive her premium for selling 2 options, so 2000 USDC
			// BOB should receive his premium for selling 3 options, so 3000 USDC
			// All protocol accounts and vaults should be empty at the end

			let btc_vault_id = AssetToVault::<MockRuntime>::get(BTC).unwrap();
			let vault_btc_balance = Assets::balance(BTC, &Vault::account_id(&btc_vault_id));
			let usdc_vault_id = AssetToVault::<MockRuntime>::get(USDC).unwrap();
			let vault_usdc_balance = Assets::balance(USDC, &Vault::account_id(&usdc_vault_id));

			assert_eq!(vault_btc_balance, 0u128);
			assert_eq!(vault_usdc_balance, 0u128);

			let protocol_account = TokenizedOptions::account_id(BTC);
			let protocol_account_stablecoin = TokenizedOptions::account_id(USDC);
			let protocol_btc_balance = Assets::balance(BTC, &protocol_account);
			let protocol_usdc_balance = Assets::balance(USDC, &protocol_account_stablecoin);

			assert_eq!(protocol_btc_balance, 0u128);
			assert_eq!(protocol_usdc_balance, 0u128);

			let alice_btc_balance = Assets::balance(BTC, &ALICE);
			let alice_usdc_balance = Assets::balance(USDC, &ALICE);
			let bob_btc_balance = Assets::balance(BTC, &BOB);
			let bob_usdc_balance = Assets::balance(USDC, &BOB);
			let charlie_btc_balance = Assets::balance(BTC, &CHARLIE);
			let charlie_usdc_balance = Assets::balance(USDC, &CHARLIE);

			assert_eq!(alice_btc_balance, 1818181818182u128);
			assert_eq!(alice_usdc_balance, 2000u128 * UNIT);
			assert_eq!(bob_btc_balance, 2727272727273u128);
			assert_eq!(bob_usdc_balance, 3000u128 * UNIT);
			assert_eq!(charlie_btc_balance, 454545454545u128);
			assert_eq!(charlie_usdc_balance, 0u128);
		});
}

#[test]
fn test_epoch_basic_example_no_totally_sold() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 2 * UNIT),
			(BOB, BTC, 3 * UNIT),
			(CHARLIE, USDC, 4000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.execute_with(|| {
			// BTC initial price: 50k

			// BTC CALL 90k strike - expire date 6000
			let option_id1 = generate_option_and_get_id(
				OptionType::Call,
				90000u128 * UNIT,
				// Use this when https://github.com/paritytech/substrate/pull/10128 is merged
				// Epoch { deposit: 0u64, purchase: 3000u64, exercise: 6000u64, end: 20000u64 },
				Epoch { deposit: 0u64, purchase: 2000u64, exercise: 5000u64, end: 9000u64 },
			);

			// Make the option goes from NotStarted to Deposit phase
			run_to_block(2);

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(ALICE), 2u128, option_id1));

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(BOB), 3u128, option_id1));

			run_to_block(3);

			assert_ok!(TokenizedOptions::buy_option(Origin::signed(CHARLIE), 4u128, option_id1));

			set_oracle_price(BTC, 100000u128 * UNIT);

			run_to_block(6);

			assert_ok!(TokenizedOptions::exercise_option(
				Origin::signed(CHARLIE),
				4u128,
				option_id1
			));

			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(ALICE), option_id1));

			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(BOB), option_id1));

			// ---------------
			//  FINAL CHECKS |
			// ---------------
			// For each option bought, (100k - 90k)/100k = 0.1 BTC of collateral deposited by ALICE
			// and BOB should be given to CHARLIE CHARLIE should end with 0.4 BTC
			// Since there were 5 BTC for sale, but just 4 options have been bought, we consider the
			// 4 BTC bought to be bought for 2/5 from ALICE collateral and for 3/5 from BOB
			// collateral. So ALICE is actually selling 1.6BTC (so 1.6 options), while BOB is
			// selling 2.4BTC (so 2.4 options). They will receive back 0.4BTC and 0.6BTC from
			// collateral respectively. ALICE should end with 2 - 0.1*1.6 = 1,84BTC and BOB with 3 -
			// 0.1*2.4 = 2,76BTC CHARLIE USDC balance should go to 0 given that each option costs
			// 1000 USDC. ALICE should receive her premium for selling 1.6 options, so 1600 USDC
			// BOB should receive his premium for selling 2.4 options, so 2400 USDC
			// All protocol accounts and vaults should be empty at the end

			let btc_vault_id = AssetToVault::<MockRuntime>::get(BTC).unwrap();
			let vault_btc_balance = Assets::balance(BTC, &Vault::account_id(&btc_vault_id));
			let usdc_vault_id = AssetToVault::<MockRuntime>::get(USDC).unwrap();
			let vault_usdc_balance = Assets::balance(USDC, &Vault::account_id(&usdc_vault_id));

			assert_eq!(vault_btc_balance, 0u128);
			assert_eq!(vault_usdc_balance, 0u128);

			let protocol_account = TokenizedOptions::account_id(BTC);
			let protocol_account_stablecoin = TokenizedOptions::account_id(USDC);
			let protocol_btc_balance = Assets::balance(BTC, &protocol_account);
			let protocol_usdc_balance = Assets::balance(USDC, &protocol_account_stablecoin);

			assert_eq!(protocol_btc_balance, 0u128);
			assert_eq!(protocol_usdc_balance, 0u128);

			let alice_btc_balance = Assets::balance(BTC, &ALICE);
			let alice_usdc_balance = Assets::balance(USDC, &ALICE);
			let bob_btc_balance = Assets::balance(BTC, &BOB);
			let bob_usdc_balance = Assets::balance(USDC, &BOB);
			let charlie_btc_balance = Assets::balance(BTC, &CHARLIE);
			let charlie_usdc_balance = Assets::balance(USDC, &CHARLIE);

			assert_eq!(alice_btc_balance, 1840000000000u128);
			assert_eq!(alice_usdc_balance, 1600u128 * UNIT);
			assert_eq!(bob_btc_balance, 2760000000000u128);
			assert_eq!(bob_usdc_balance, 2400u128 * UNIT);
			assert_eq!(charlie_btc_balance, 400000000000u128);
			assert_eq!(charlie_usdc_balance, 0u128);
		});
}

// #[test]
// fn test_epoch() {
// 	ExtBuilder::default()
// 		.initialize_balances(Vec::from([
// 			(ALICE, BTC, 10000 * UNIT),
// 			(ALICE, USDC, 500000000 * UNIT),
// 			(BOB, BTC, 10000 * UNIT),
// 			(BOB, USDC, 500000000 * UNIT),
// 			(CHARLIE, BTC, 10000 * UNIT),
// 			(CHARLIE, USDC, 500000000 * UNIT),
// 			(DAVE, BTC, 10000 * UNIT),
// 			(DAVE, USDC, 500000000 * UNIT),
// 			(EVEN, BTC, 10000 * UNIT),
// 			(EVEN, USDC, 500000000 * UNIT),
// 		]))
// 		.build()
// 		.initialize_oracle_prices()
// 		.initialize_all_vaults()
// 		.execute_with(|| {
// 			// BTC initial price: 50k

// 			// BTC CALL 46k strike - expire date 6000
// 			let option_id1 = generate_option_and_get_id(
// 				OptionType::Call,
// 				46273u128 * UNIT,
// 				Epoch { deposit: 0u64, purchase: 3000u64, exercise: 6000u64, end: 20000u64 },
// 			);

// 			// BTC CALL 49k strike - expire date 9000
// 			let option_id2 = generate_option_and_get_id(
// 				OptionType::Call,
// 				49297u128 * UNIT,
// 				Epoch { deposit: 3000u64, purchase: 6000u64, exercise: 9000u64, end: 20000u64 },
// 			);

// 			// BTC PUT 56k strike - expire date 8000
// 			let option_id3 = generate_option_and_get_id(
// 				OptionType::Put,
// 				55903u128 * UNIT,
// 				Epoch { deposit: 0u64, purchase: 4000u64, exercise: 8000u64, end: 20000u64 },
// 			);

// 			// BTC PUT 59k strike - expire date 7000
// 			let option_id4 = generate_option_and_get_id(
// 				OptionType::Put,
// 				59107u128 * UNIT,
// 				Epoch { deposit: 3000u64, purchase: 5000u64, exercise: 7000u64, end: 20000u64 },
// 			);

// 			// -------------------------------------------
// 			//  BLOCK 0-2: deposit for option1 and option3 |
// 			// -------------------------------------------
// 			// -------------------
// 			//  OPTION 1 SUMMARY |
// 			// -------------------
// 			// ALICE sells 631
// 			// BOB sells 887

// 			// -------------------
// 			//  OPTION 3 SUMMARY |
// 			// -------------------
// 			// DAVE sells 977

// 			assert_ok!(TokenizedOptions::sell_option(Origin::signed(ALICE), 191u128, option_id1));

// 			assert_ok!(TokenizedOptions::sell_option(Origin::signed(BOB), 383u128, option_id1));

// 			assert_ok!(TokenizedOptions::sell_option(Origin::signed(DAVE), 1553u128, option_id3));

// 			assert_ok!(TokenizedOptions::delete_sell_option(
// 				Origin::signed(ALICE),
// 				137u128,
// 				option_id1
// 			));

// 			assert_ok!(TokenizedOptions::delete_sell_option(
// 				Origin::signed(DAVE),
// 				576u128,
// 				option_id3
// 			));

// 			run_to_block(1);

// 			assert_ok!(TokenizedOptions::delete_sell_option(
// 				Origin::signed(BOB),
// 				256u128,
// 				option_id1
// 			));

// 			assert_ok!(TokenizedOptions::delete_sell_option(
// 				Origin::signed(ALICE),
// 				19u128,
// 				option_id1
// 			));

// 			run_to_block(2);

// 			assert_ok!(TokenizedOptions::sell_option(Origin::signed(ALICE), 596u128, option_id1));

// 			assert_ok!(TokenizedOptions::sell_option(Origin::signed(BOB), 760u128, option_id1));

// 			run_to_block(3);
// 			// ----------------------------------------------------------------------------------------
// 			//  BLOCK 3: purchase window for option1, deposit window for option2, option3 and option4 |
// 			// ----------------------------------------------------------------------------------------
// 			// -------------------
// 			//  OPTION 1 SUMMARY |
// 			// -------------------
// 			// ALICE sells 631
// 			// BOB sells 887
// 			// EVEN buys 809

// 			// -------------------
// 			//  OPTION 2 SUMMARY |
// 			// -------------------
// 			// CHARLIE sells 1093

// 			// -------------------
// 			//  OPTION 3 SUMMARY |
// 			// -------------------
// 			// CHARLIE sells 641
// 			// DAVE sells 977

// 			// -------------------
// 			//  OPTION 4 SUMMARY |
// 			// -------------------
// 			// BOB sells 1013

// 			assert_ok!(TokenizedOptions::buy_option(Origin::signed(EVEN), 809u128, option_id1));

// 			assert_ok!(TokenizedOptions::sell_option(Origin::signed(BOB), 1013u128, option_id4));

// 			assert_ok!(TokenizedOptions::sell_option(
// 				Origin::signed(CHARLIE),
// 				1093u128,
// 				option_id2
// 			));

// 			assert_ok!(TokenizedOptions::sell_option(Origin::signed(CHARLIE), 641u128, option_id3));

// 			// -------------------------------------------------------------------------------------------
// 			//  BLOCK 4: purchase window for option1 and option3, deposit window for option2 and option4 |
// 			// -------------------------------------------------------------------------------------------
// 			// -------------------
// 			//  OPTION 1 SUMMARY |
// 			// -------------------
// 			// ALICE sells 631
// 			// BOB sells 887
// 			// EVEN buys 1213

// 			// -------------------
// 			//  OPTION 2 SUMMARY |
// 			// -------------------
// 			// CHARLIE sells 797

// 			// -------------------
// 			//  OPTION 3 SUMMARY |
// 			// -------------------
// 			// ALICE buys 1249
// 			// CHARLIE sells 641
// 			// DAVE sells 977
// 			// EVEN buys 239

// 			// -------------------
// 			//  OPTION 4 SUMMARY |
// 			// -------------------
// 			// BOB sells 1013
// 			run_to_block(4);

// 			assert_ok!(TokenizedOptions::buy_option(Origin::signed(ALICE), 1249u128, option_id3));

// 			assert_ok!(TokenizedOptions::buy_option(Origin::signed(EVEN), 404u128, option_id1));
// 			assert_ok!(TokenizedOptions::delete_sell_option(
// 				Origin::signed(CHARLIE),
// 				296u128,
// 				option_id2
// 			));

// 			assert_ok!(TokenizedOptions::buy_option(Origin::signed(EVEN), 239u128, option_id3));

// 			// ----------------------------------------------------------------------------------------
// 			//  BLOCK 5: purchase window for option1, option3 and option4, deposit window for option2 |
// 			// ----------------------------------------------------------------------------------------
// 			// -------------------
// 			//  OPTION 1 SUMMARY |
// 			// -------------------
// 			// ALICE sells 631
// 			// BOB sells 887
// 			// EVEN buys 1213

// 			// -------------------
// 			//  OPTION 2 SUMMARY |
// 			// -------------------
// 			// CHARLIE sells 797

// 			// -------------------
// 			//  OPTION 3 SUMMARY |
// 			// -------------------
// 			// ALICE buys 1249
// 			// CHARLIE sells 641
// 			// DAVE sells 977
// 			// EVEN buys 239

// 			// -------------------
// 			//  OPTION 4 SUMMARY |
// 			// -------------------
// 			// BOB sells 1013
// 			run_to_block(5);

// 			// -----------------------------------------------------------------------------------------
// 			//  BLOCK 6: purchase window for option2, option3 and option4, exercise window for option1 |
// 			// -----------------------------------------------------------------------------------------
// 			// -------------------
// 			//  OPTION 1 SUMMARY |
// 			// -------------------
// 			// ALICE sells 631
// 			// BOB sells 887
// 			// EVEN buys 1213

// 			// -------------------
// 			//  OPTION 2 SUMMARY |
// 			// -------------------
// 			// CHARLIE sells 797
// 			// DAVE buys 367

// 			// -------------------
// 			//  OPTION 3 SUMMARY |
// 			// -------------------
// 			// ALICE buys 1249
// 			// CHARLIE sells 641
// 			// DAVE sells 977
// 			// EVEN buys 239

// 			// -------------------
// 			//  OPTION 4 SUMMARY |
// 			// -------------------
// 			// BOB sells 1013

// 			// BTC price moves from 50k to 53623, buyers are in profit
// 			set_oracle_price(BTC, 53623u128 * UNIT);

// 			run_to_block(6);

// 			assert_ok!(TokenizedOptions::buy_option(Origin::signed(DAVE), 367u128, option_id2));

// 			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(ALICE), option_id1));

// 			assert_ok!(TokenizedOptions::exercise_option(
// 				Origin::signed(EVEN),
// 				1213u128,
// 				option_id1
// 			));

// 			// --------------------------------------------------------------------------------------------
// 			//  BLOCK 7: purchase window for option2 and option3, exercise window for option1 and option4 |
// 			// --------------------------------------------------------------------------------------------
// 			run_to_block(7);

// 			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(BOB), option_id4));

// 			// -----------------------------------------------------------------------------------------
// 			//  BLOCK 8: purchase window for option2, exercise window for option1, option3 and option4 |
// 			// -----------------------------------------------------------------------------------------
// 			run_to_block(8);

// 			assert_ok!(TokenizedOptions::exercise_option(
// 				Origin::signed(ALICE),
// 				1249u128,
// 				option_id3
// 			));

// 			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(DAVE), option_id3));

// 			assert_ok!(TokenizedOptions::exercise_option(
// 				Origin::signed(EVEN),
// 				239u128,
// 				option_id3
// 			));

// 			// ---------------------------------------------------------------------
// 			//  BLOCK 9: exercise window for option1, option2, option3 and option4 |
// 			// ---------------------------------------------------------------------
// 			run_to_block(9);

// 			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(CHARLIE), option_id3));

// 			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(CHARLIE), option_id2));

// 			assert_ok!(TokenizedOptions::exercise_option(
// 				Origin::signed(DAVE),
// 				367u128,
// 				option_id2
// 			));

// 			assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(BOB), option_id1));

// 			// ---------------
// 			//  FINAL CHECKS |
// 			// ---------------
// 			let btc_vault_id = AssetToVault::<MockRuntime>::get(BTC).unwrap();
// 			let vault_btc_balance = Assets::balance(BTC, &Vault::account_id(&btc_vault_id));
// 			let usdc_vault_id = AssetToVault::<MockRuntime>::get(USDC).unwrap();
// 			let vault_usdc_balance = Assets::balance(USDC, &Vault::account_id(&usdc_vault_id));

// 			assert_eq!(vault_btc_balance, 0u128);
// 			assert_eq!(vault_usdc_balance, 0u128);

// 			let protocol_account = TokenizedOptions::account_id(BTC);
// 			let protocol_account_stablecoin = TokenizedOptions::account_id(USDC);
// 			let protocol_btc_balance = Assets::balance(BTC, &protocol_account);
// 			let protocol_usdc_balance = Assets::balance(USDC, &protocol_account_stablecoin);

// 			assert_eq!(protocol_btc_balance, 0u128);
// 			assert_eq!(protocol_usdc_balance, 2u128); // Dust

// 			let alice_btc_balance = Assets::balance(BTC, &ALICE);
// 			let alice_usdc_balance = Assets::balance(USDC, &ALICE);
// 			let bob_btc_balance = Assets::balance(BTC, &BOB);
// 			let bob_usdc_balance = Assets::balance(USDC, &BOB);
// 			let charlie_btc_balance = Assets::balance(BTC, &CHARLIE);
// 			let charlie_usdc_balance = Assets::balance(USDC, &CHARLIE);
// 			let dave_btc_balance = Assets::balance(BTC, &DAVE);
// 			let dave_usdc_balance = Assets::balance(USDC, &DAVE);
// 			let even_btc_balance = Assets::balance(BTC, &EVEN);
// 			let even_usdc_balance = Assets::balance(USDC, &EVEN);

// 			assert_eq!(alice_btc_balance, 9930887815527220u128);
// 			assert_eq!(alice_usdc_balance, 502102938050065876152u128);
// 			assert_eq!(bob_btc_balance, 9902848640844126u128);
// 			assert_eq!(bob_usdc_balance, 500708781949934123847u128);
// 			assert_eq!(charlie_btc_balance, 9970392518136032u128);
// 			assert_eq!(charlie_usdc_balance, 499612442373300370827u128);
// 			assert_eq!(dave_btc_balance, 10029607481863968u128);
// 			assert_eq!(dave_usdc_balance, 498482917626699629172u128);
// 			assert_eq!(even_btc_balance, 10166263543628654u128);
// 			assert_eq!(even_usdc_balance, 499092920000000000000u128);
// 		});
// }
