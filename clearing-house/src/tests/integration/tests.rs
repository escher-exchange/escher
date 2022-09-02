use crate::{
    mock::integration::{
        AccountId, AssetId, Assets, Balance, BlockNumber, Decimal, ExtBuilder, MarketId, Moment,
        Oracle, Origin, Runtime, StalePrice, System, TestPallet, Timestamp, UnsignedDecimal, Vamm,
        VammId, ALICE, BOB, DOT, PICA, USDC,
    },
    tests::helpers,
    Direction::{Long, Short},
    Error, Event, Market, MarketConfig as MarketConfigGeneric, Position,
};
use composable_support::validation::Validated;
use composable_traits::time::{DurationSeconds, ONE_HOUR};
use frame_support::{
    assert_noop, assert_ok,
    error::BadOrigin,
    pallet_prelude::Hooks,
    traits::{fungibles::Transfer, UnixTime},
};
use pallet_vamm::VammStateOf;
use proptest::prelude::*;
use sp_runtime::{traits::Zero, FixedPointNumber, Percent};
use traits::vamm::{AssetType, Vamm as VammTrait, VammConfig as VammConfigGeneric};

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            native_balances: vec![],
            balances: vec![],
            collateral_type: Some(USDC),
            max_price_divergence: Decimal::from_inner(i128::MAX),
        }
    }
}

#[test]
fn externalities_builder_works() {
    ExtBuilder::default().build().execute_with(|| {});
}

// -------------------------------------------------------------------------------------------------
//                                  Helper Functions and Traits
// -------------------------------------------------------------------------------------------------

fn advance_blocks_by(blocks: BlockNumber, secs_per_block: DurationSeconds) {
    let mut curr_block = System::block_number();
    let mut time = Timestamp::get();
    for _ in 0..blocks {
        if curr_block > 0 {
            Timestamp::on_finalize(curr_block);
            Oracle::on_finalize(curr_block);
            System::on_finalize(curr_block);
        }
        curr_block += 1;
        System::set_block_number(curr_block);
        // Time is set in milliseconds
        time += 1000 * secs_per_block;
        let _ = Timestamp::set(Origin::none(), time);
        System::on_initialize(curr_block);
        Timestamp::on_initialize(curr_block);
        Oracle::on_initialize(curr_block);
    }
}

fn run_to_time(seconds: DurationSeconds) {
    let curr_block = System::block_number();
    if curr_block > 0 {
        Timestamp::on_finalize(curr_block);
        Oracle::on_finalize(curr_block);
        System::on_finalize(curr_block);
    }

    let next_block = curr_block + 1;
    System::set_block_number(next_block);
    // Time is set in milliseconds, so we multiply the seconds by 1000
    // Should fail if the current time is greater than or equal to the argument
    let _ = Timestamp::set(Origin::none(), 1_000 * seconds);
    System::on_initialize(next_block);
    Timestamp::on_initialize(next_block);
    Oracle::on_initialize(next_block);
}

fn set_oracle_for(asset_id: AssetId, price: Balance) {
    assert_ok!(Oracle::add_asset_and_info(
        Origin::signed(ALICE),
        asset_id,
        Validated::new(Percent::from_percent(80)).unwrap(), // threshold
        Validated::new(1).unwrap(),                         // min_answers
        Validated::new(3).unwrap(),                         // max_answers
        Validated::new(ORACLE_BLOCK_INTERVAL).unwrap(),     // block_interval
        5,                                                  // reward
        5,                                                  // slash
        false                                               // emit_price_changes
    ));

    assert_ok!(Oracle::set_signer(Origin::signed(ALICE), BOB));
    assert_ok!(Oracle::set_signer(Origin::signed(BOB), ALICE));

    assert_ok!(Oracle::add_stake(Origin::signed(ALICE), 50));
    assert_ok!(Oracle::add_stake(Origin::signed(BOB), 50));

    update_oracle_for(asset_id, price);
}

fn update_oracle_for(asset_id: AssetId, price: Balance) {
    // Must be strictly greater than block interval for price to be considered 'requested'
    advance_blocks_by(ORACLE_BLOCK_INTERVAL + 1, 1);

    assert_ok!(Oracle::submit_price(Origin::signed(BOB), price, asset_id));

    // Advance block so that Oracle block finalization hook is called
    advance_blocks_by(1, 1);
}

fn get_collateral(account_id: &AccountId) -> Balance {
    helpers::get_collateral::<Runtime>(account_id)
}

fn get_outstanding_profits(account_id: &AccountId) -> Balance {
    helpers::get_outstanding_profits::<Runtime>(account_id)
}

fn get_market(market_id: &MarketId) -> Market<Runtime> {
    helpers::get_market::<Runtime>(market_id)
}

fn get_market_fee_pool(market_id: MarketId) -> Balance {
    helpers::get_market_fee_pool::<Runtime>(market_id)
}

fn get_vamm(vamm_id: &VammId) -> VammStateOf<Runtime> {
    Vamm::get_vamm(vamm_id).unwrap()
}

fn get_insurance_acc_balance() -> Balance {
    helpers::get_insurance_acc_balance::<Runtime>()
}

fn get_position(account_id: &AccountId, market_id: &MarketId) -> Position<Runtime> {
    helpers::get_position::<Runtime>(account_id, market_id).unwrap()
}

fn get_unrealized_pnl(account_id: &AccountId, market_id: &MarketId) -> Decimal {
    let market = get_market(market_id);
    let position = get_position(account_id, market_id);
    let (_, pnl) = TestPallet::abs_position_notional_and_pnl(
        &market,
        &position,
        position.direction().unwrap(),
    )
    .unwrap();
    pnl
}

fn get_time_now() -> DurationSeconds {
    <Timestamp as UnixTime>::now().as_secs()
}

fn set_maximum_oracle_mark_divergence(fraction: Decimal) {
    helpers::set_maximum_oracle_mark_divergence::<Runtime>(fraction)
}

fn set_partial_liquidation_penalty(decimal: Decimal) {
    helpers::set_partial_liquidation_penalty::<Runtime>(decimal)
}

fn set_partial_liquidation_close(decimal: Decimal) {
    helpers::set_partial_liquidation_close::<Runtime>(decimal)
}

fn set_liquidator_share_partial(decimal: Decimal) {
    helpers::set_liquidator_share_partial::<Runtime>(decimal)
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            asset: DOT,
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 100,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            margin_ratio_initial: Decimal::from_float(0.1),
            margin_ratio_maintenance: Decimal::from_float(0.02),
            margin_ratio_partial: Decimal::from_float(0.04),
            minimum_trade_size: 0.into(),
            funding_frequency: ONE_HOUR,
            funding_period: ONE_HOUR * 24,
            taker_fee: 0,
            twap_period: ONE_HOUR,
        }
    }
}

// -------------------------------------------------------------------------------------------------
//                                      Types & Constants
// -------------------------------------------------------------------------------------------------

pub type MarketConfig = MarketConfigGeneric<AssetId, Balance, Decimal, Moment, VammConfig>;
pub type VammConfig = VammConfigGeneric<Balance, Moment>;

// Must be strictly greater than StalePrice
pub const ORACLE_BLOCK_INTERVAL: u64 = StalePrice::get() + 1;
pub const UNIT: Balance = UnsignedDecimal::DIV;

// -------------------------------------------------------------------------------------------------
//                                        Create Market
// -------------------------------------------------------------------------------------------------

mod create_market {
    use super::*;

    proptest! {
        #[test]
        fn should_succeed_in_creating_first_market(
            asset_id in prop_oneof![Just(DOT), Just(PICA)]
        ) {
            ExtBuilder {
                native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
                balances: vec![(ALICE, PICA, UNIT), (BOB, PICA, UNIT)],
                ..Default::default()
            }
            .build()
            .execute_with(|| {
                set_oracle_for(asset_id, 1_000); // 10 in cents
                let config = MarketConfig { asset: asset_id, ..Default::default() };
                assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

                let market_id = MarketId::zero();
                let market = TestPallet::get_market(&market_id).unwrap();
                assert_eq!(market.asset_id, asset_id);
                assert_eq!(market.last_oracle_price, Decimal::from(10));
                assert_eq!(market.last_oracle_twap, Decimal::from(10));
            })
        }
    }
}

// -------------------------------------------------------------------------------------------------
//                                     Open/Close Position
// -------------------------------------------------------------------------------------------------

mod open_position {
    use super::*;

    #[test]
    fn should_succeed_in_opening_first_position() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![
                (ALICE, PICA, UNIT),
                (BOB, PICA, UNIT),
                (BOB, USDC, UNIT * 100),
            ],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            set_oracle_for(DOT, 1_000);
            let config = MarketConfig {
                asset: DOT,
                vamm_config: VammConfig {
                    base_asset_reserves: UNIT * 100,
                    quote_asset_reserves: UNIT * 10_000,
                    peg_multiplier: 10,
                    twap_period: ONE_HOUR,
                },
                margin_ratio_initial: Decimal::from_float(0.1),
                margin_ratio_maintenance: Decimal::from_float(0.02),
                margin_ratio_partial: Decimal::from_float(0.04),
                minimum_trade_size: 0.into(),
                funding_frequency: ONE_HOUR,
                funding_period: ONE_HOUR * 24,
                taker_fee: 0,
                twap_period: ONE_HOUR,
            };
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(BOB),
                USDC,
                UNIT * 100
            ));

            let market = get_market(&MarketId::zero());
            let vamm_state = get_vamm(&market.vamm_id);

            assert_ok!(TestPallet::open_position(
                Origin::signed(BOB),
                Zero::zero(),
                Long,
                UNIT * 100,
                0
            ));

            assert_ne!(get_market(&MarketId::zero()), market);
            assert_ne!(get_vamm(&market.vamm_id), vamm_state);
        })
    }

    #[test]
    fn should_enforce_slippage_controls() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 10_000);

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    // Start with a mark price of 100
                    base_asset_reserves: UNIT * 100,
                    quote_asset_reserves: UNIT * 10_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(Origin::signed(BOB), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            assert_noop!(
                TestPallet::open_position(Origin::signed(ALICE), market_id, Long, UNIT * 100, UNIT),
                pallet_vamm::Error::<Runtime>::SwappedAmountLessThanMinimumLimit
            );
        })
    }

    #[test]
    fn should_succeed_with_two_traders_in_a_market() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![
                (ALICE, PICA, UNIT),
                (BOB, PICA, UNIT),
                (ALICE, USDC, UNIT * 100),
                (BOB, USDC, UNIT * 100),
            ],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = PICA;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    base_asset_reserves: UNIT * 100,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));
            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(BOB),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            let market = get_market(&market_id);
            let vamm_state_before = get_vamm(&market.vamm_id);

            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));
            assert_ok!(TestPallet::open_position(
                Origin::signed(BOB),
                market_id,
                Long,
                UNIT * 100,
                0
            ));

            assert_ok!(TestPallet::close_position(Origin::signed(ALICE), market_id));
            assert_ok!(TestPallet::close_position(Origin::signed(BOB), market_id));

            // Alice closes her position in profit, Bob closes his position in loss
            // However, since Alice closes her position first, there are no realized losses in the
            // market yet, so her profits are outstanding
            let alice_col = get_collateral(&ALICE);
            let alice_outstanding_profits = get_outstanding_profits(&ALICE);
            let bob_col = get_collateral(&BOB);
            assert!(alice_col + alice_outstanding_profits > bob_col);
            assert_eq!(alice_col + alice_outstanding_profits + bob_col, UNIT * 200);

            assert_ok!(TestPallet::withdraw_collateral(
                Origin::signed(ALICE),
                alice_col + alice_outstanding_profits
            ));

            // vAMM is back to its initial state due to path independence
            let vamm_state_after = get_vamm(&market.vamm_id);
            assert_eq!(
                vamm_state_before.base_asset_reserves,
                vamm_state_after.base_asset_reserves
            );
            assert_eq!(
                vamm_state_before.quote_asset_reserves,
                vamm_state_after.quote_asset_reserves
            );
        })
    }

    #[test]
    #[ignore = "FIXME: vAMM TWAP isn't updated if last twap timestamp is equal to the current \
	block's timestamp"]
    fn should_update_vamm_twap_in_the_same_block() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    // Mark price = 10.0
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            let market = get_market(&market_id);
            let vamm_before = get_vamm(&market.vamm_id);

            assert_eq!(vamm_before.base_asset_twap, 10.into());

            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));
            let vamm_after = get_vamm(&market.vamm_id);
            // open_position should update TWAP before swapping, therefore not changing the mark
            // TWAP
            assert_eq!(vamm_before.base_asset_twap, vamm_after.base_asset_twap);
            let vamm_before = vamm_after;

            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));
            let vamm_after = get_vamm(&market.vamm_id);
            // now the vAMM picks up the change caused by the previous swap
            assert!(vamm_before.base_asset_twap < vamm_after.base_asset_twap);
        })
    }

    #[test]
    fn should_update_vamm_twap_across_blocks() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    // Mark price = 10.0
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            let market = get_market(&market_id);
            let vamm_before = get_vamm(&market.vamm_id);

            assert_eq!(vamm_before.base_asset_twap, 10.into());

            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));
            let vamm_after = get_vamm(&market.vamm_id);
            // open_position should update TWAP before swapping, therefore not changing the mark
            // TWAP
            assert_eq!(vamm_before.base_asset_twap, vamm_after.base_asset_twap);
            let vamm_before = vamm_after;

            advance_blocks_by(1, 1);

            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));
            let vamm_after = get_vamm(&market.vamm_id);
            // now the vAMM picks up the change caused by the previous swap
            assert!(vamm_before.base_asset_twap < vamm_after.base_asset_twap);
        })
    }
}

// -------------------------------------------------------------------------------------------------
//                                     Update Funding
// -------------------------------------------------------------------------------------------------

mod update_funding {
    use super::*;

    #[test]
    fn should_update_oracle_twap() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000); // Index price = 10.0

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    // Mark price = 10.0
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(
                Origin::signed(ALICE),
                config.clone()
            ));

            let market_id = Zero::zero();
            let market = get_market(&market_id);
            let vamm = get_vamm(&market.vamm_id);

            assert_eq!(market.last_oracle_price, 10.into());
            assert_eq!(market.last_oracle_twap, 10.into());
            assert_eq!(vamm.base_asset_twap, 10.into());

            update_oracle_for(asset_id, 1_100); //  Index price = 11.0
            run_to_time(market.last_oracle_ts + config.twap_period);
            assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));
            let market = get_market(&market_id);
            // Oracle price updates are clipped at 10bps from the previous recorded price
            assert_eq!(market.last_oracle_price, (1001, 100).into());
            assert!(market.last_oracle_twap > 10.into());
        })
    }

    #[test]
    fn should_update_vamm_twap() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    // Mark price = 10.0
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(
                Origin::signed(ALICE),
                config.clone()
            ));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            let market = get_market(&market_id);
            let vamm_before = get_vamm(&market.vamm_id);

            assert_eq!(market.last_oracle_price, 10.into());
            assert_eq!(market.last_oracle_twap, 10.into());
            assert_eq!(vamm_before.base_asset_twap, 10.into());

            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));
            let vamm_after = get_vamm(&market.vamm_id);
            // open_position should update TWAP before swapping, therefore not changing the mark
            // TWAP
            assert_eq!(vamm_before.base_asset_twap, vamm_after.base_asset_twap);
            let vamm_before = vamm_after;

            run_to_time(market.last_oracle_ts + config.twap_period);
            assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));
            let vamm_after = get_vamm(&market.vamm_id);
            assert!(vamm_before.base_asset_twap < vamm_after.base_asset_twap);
        })
    }

    #[test]
    fn should_block_update_if_mark_index_too_divergent() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 10_000);

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    // Mark price = 111.0
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 1_110_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(
                Origin::signed(ALICE),
                config.clone()
            ));

            let market_id = Zero::zero();
            let market = get_market(&market_id);
            let vamm = get_vamm(&market.vamm_id);
            assert_eq!(market.last_oracle_twap, 100.into());
            assert_eq!(
                <Vamm as VammTrait>::get_price(market.vamm_id, AssetType::Base).unwrap(),
                111.into()
            );
            assert_eq!(vamm.base_asset_twap, 111.into());

            set_maximum_oracle_mark_divergence((1, 10).into());

            run_to_time(market.last_oracle_ts + config.twap_period);
            assert_noop!(
                TestPallet::update_funding(Origin::signed(ALICE), market_id),
                Error::<Runtime>::OracleMarkTooDivergent
            );
        })
    }

    #[test]
    fn clearing_house_should_receive_funding() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(
                Origin::signed(ALICE),
                config.clone()
            ));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            assert_eq!(get_market_fee_pool(market_id), 0);
            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));

            let market = get_market(&market_id);
            run_to_time(market.last_oracle_ts + config.twap_period);
            // update_funding updates the vAMM TWAP, which increases since the previous trade pushed
            // the price upwards
            assert_ok!(TestPallet::update_funding(Origin::signed(BOB), market_id));
            assert!(get_market_fee_pool(market_id) > 0);
        })
    }

    #[test]
    fn clearing_house_should_pay_funding() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100), (BOB, USDC, UNIT * 1_000_000)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            // Oracle price (and TWAP) start at 20.0
            set_oracle_for(asset_id, 2_000);

            // vAMM price (and TWAP start at 10.0)
            let config = MarketConfig {
                asset: asset_id,
                vamm_config: VammConfig {
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(
                Origin::signed(ALICE),
                config.clone()
            ));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            assert_eq!(get_market_fee_pool(market_id), 0);

            // Alice goes long, but not enough to bring mark price to index
            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));

            // Populate Fee Pool with funds
            let fee_pool_before = UNIT * 1_000_000;
            <Assets as Transfer<AccountId>>::transfer(
                USDC,
                &BOB,
                &TestPallet::get_fee_pool_account(market_id),
                fee_pool_before,
                false,
            )
            .unwrap();

            let market = get_market(&market_id);
            run_to_time(market.last_oracle_ts + config.twap_period);
            assert_ok!(TestPallet::update_funding(Origin::signed(BOB), market_id));
            assert!(get_market_fee_pool(market_id) < fee_pool_before);
        })
    }
}

// -------------------------------------------------------------------------------------------------
//                                          Liquidate
// -------------------------------------------------------------------------------------------------

mod liquidate {
    use super::*;

    #[test]
    fn should_not_liquidate_if_above_partial_margin_ratio() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 10), (BOB, USDC, UNIT * 1_000_000)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            set_partial_liquidation_close((25, 100).into());
            set_partial_liquidation_penalty((25, 1000).into());
            set_liquidator_share_partial((50, 100).into());

            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                vamm_config: VammConfig {
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                margin_ratio_initial: (100, 1000).into(), // 10x max leverage
                margin_ratio_partial: (99, 1000).into(),  // ~10.1x max leverage
                margin_ratio_maintenance: (80, 1000).into(),
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 10
            ));

            let market_id = Zero::zero();

            // Alice goes long with maximum leverage
            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));

            assert_noop!(
                TestPallet::liquidate(Origin::signed(BOB), ALICE),
                Error::<Runtime>::SufficientCollateral
            );
        })
    }

    #[test]
    fn should_liquidate_if_below_partial_margin_ratio_by_pnl() {
        let alice_col = UNIT * 10;
        let bob_col = UNIT * 1_000;

        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, alice_col), (BOB, USDC, bob_col)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            set_partial_liquidation_close((25, 100).into());
            set_partial_liquidation_penalty((25, 1000).into());
            set_liquidator_share_partial((50, 100).into());

            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                vamm_config: VammConfig {
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                margin_ratio_initial: (100, 1000).into(), // 10x max leverage
                margin_ratio_partial: (99, 1000).into(),  // ~10.1x max leverage
                margin_ratio_maintenance: (80, 1000).into(),
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                alice_col
            ));
            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(BOB),
                USDC,
                bob_col
            ));

            let market_id = Zero::zero();

            // Alice goes long with maximum leverage
            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                alice_col * 10,
                0
            ));

            // Bob goes short with size, pushing the price below Alice's partial liquidation
            // threshold
            assert_ok!(TestPallet::open_position(
                Origin::signed(BOB),
                market_id,
                Short,
                bob_col,
                Balance::MAX
            ));

            let unrealized_pnl = get_unrealized_pnl(&ALICE, &market_id);
            assert!(unrealized_pnl < Zero::zero());

            assert_ok!(TestPallet::liquidate(Origin::signed(BOB), ALICE));
            assert!(get_collateral(&BOB) > bob_col);
            assert!(get_insurance_acc_balance() > Zero::zero());
        })
    }

    #[test]
    fn should_liquidate_if_below_partial_margin_ratio_by_funding() {
        let collateral = UNIT * 10;
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, collateral)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            set_partial_liquidation_close((25, 100).into());
            set_partial_liquidation_penalty((25, 1000).into());
            set_liquidator_share_partial((50, 100).into());

            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig {
                vamm_config: VammConfig {
                    base_asset_reserves: UNIT * 10_000,
                    quote_asset_reserves: UNIT * 100_000,
                    peg_multiplier: 1,
                    twap_period: ONE_HOUR,
                },
                margin_ratio_initial: (100, 1000).into(), // 10x max leverage
                margin_ratio_partial: (99, 1000).into(),  // ~10.1x max leverage
                margin_ratio_maintenance: (80, 1000).into(),
                ..Default::default()
            };
            assert_ok!(TestPallet::create_market(
                Origin::signed(ALICE),
                config.clone()
            ));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                collateral
            ));

            let market_id = Zero::zero();

            // Alice goes long with maximum leverage
            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                collateral * 10,
                0
            ));

            // Index price moves against Alice's position
            update_oracle_for(asset_id, 900);
            let market = get_market(&market_id);
            run_to_time(market.last_oracle_ts + config.twap_period);
            assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));

            // Give time for TWAP to catch up to index
            for _ in 0..10 {
                advance_blocks_by(1, config.twap_period);
                assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));
                dbg!(get_market(&market_id).last_oracle_twap);
            }

            assert_ok!(TestPallet::liquidate(Origin::signed(BOB), ALICE));
            System::assert_last_event(Event::PartialLiquidation { user: ALICE }.into());
        })
    }
}

// -------------------------------------------------------------------------------------------------
//                                          Close Market
// -------------------------------------------------------------------------------------------------

mod close_market {
    use super::*;

    #[test]
    #[ignore = "Vamm::close not yet implemented"]
    fn should_close_market_and_vamm_under_normal_conditions() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig::default();
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            let market_id = Zero::zero();
            let now = get_time_now();
            assert_noop!(
                TestPallet::close_market(Origin::signed(BOB), market_id, now + 10),
                BadOrigin
            );
            assert_ok!(TestPallet::close_market(
                Origin::root(),
                market_id,
                now + 10
            ));

            let market = get_market(&market_id);
            let vamm = get_vamm(&market.vamm_id);
            assert_eq!(vamm.closed, Some(now + 10));
        })
    }

    #[test]
    fn should_block_closing_positions_only_after_market_close() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100), (BOB, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig::default();
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));
            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(BOB),
                USDC,
                UNIT * 100
            ));

            let market_id = Zero::zero();
            assert_ok!(TestPallet::open_position(
                Origin::signed(ALICE),
                market_id,
                Long,
                UNIT * 100,
                0
            ));
            assert_ok!(TestPallet::open_position(
                Origin::signed(BOB),
                market_id,
                Short,
                UNIT * 100,
                Balance::MAX
            ));

            let now = get_time_now();
            assert_ok!(TestPallet::close_market(
                Origin::root(),
                market_id,
                now + 12
            ));

            advance_blocks_by(1, 6);
            assert_ok!(TestPallet::close_position(Origin::signed(ALICE), market_id));

            advance_blocks_by(1, 6);
            assert_noop!(
                TestPallet::close_position(Origin::signed(BOB), market_id),
                Error::<Runtime>::MarketClosed
            );
        })
    }

    #[test]
    fn should_not_allow_opening_positions_after_close_market_call() {
        ExtBuilder {
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![(ALICE, USDC, UNIT * 100), (BOB, USDC, UNIT * 100)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            let asset_id = DOT;
            set_oracle_for(asset_id, 1_000);

            let config = MarketConfig::default();
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            let market_id = Zero::zero();
            let now = get_time_now();
            assert_ok!(TestPallet::close_market(
                Origin::root(),
                market_id,
                now + 12
            ));

            advance_blocks_by(1, 6);
            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(ALICE),
                USDC,
                UNIT * 100
            ));
            assert_noop!(
                TestPallet::open_position(Origin::signed(ALICE), market_id, Long, UNIT * 100, 0),
                Error::<Runtime>::MarketShuttingDown
            );

            advance_blocks_by(1, 6);
            assert_ok!(TestPallet::deposit_collateral(
                Origin::signed(BOB),
                USDC,
                UNIT * 100
            ));
            assert_noop!(
                TestPallet::open_position(
                    Origin::signed(BOB),
                    market_id,
                    Short,
                    UNIT * 100,
                    Balance::MAX
                ),
                Error::<Runtime>::MarketClosed
            );
        })
    }
}
